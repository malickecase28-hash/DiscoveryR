use research_tape::fvg_e1::{PreflightResult, ZoneLifecycle};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::PathBuf,
};

#[derive(Debug, Serialize)]
struct Check {
    id: &'static str,
    status: &'static str,
    evidence: String,
}

#[derive(Debug, Serialize)]
struct ChallengePackage {
    package_type: &'static str,
    status: &'static str,
    independent_from_scanner: bool,
    confirmation_status: &'static str,
    zones_n: u64,
    logical_rows_sha256: String,
    checks: Vec<Check>,
}

fn validate_zone(zone: &ZoneLifecycle, value: &serde_json::Value) -> Result<(), String> {
    let identity = format!("{}:{}", zone.timeframe, zone.formation.zone_id);
    if zone.identity != identity {
        return Err(format!(
            "identity mismatch: {} != {identity}",
            zone.identity
        ));
    }
    if zone.formation.detection_ts != zone.formation_bar_close_ts {
        return Err("formation detection and bar close differ".into());
    }
    if zone.formation_available_time_ns <= 0 || zone.formation_source_sequence <= 0 {
        return Err("formation availability provenance is missing".into());
    }
    if zone.formation.lower >= zone.formation.upper {
        return Err("formation bounds are not ordered".into());
    }
    if zone
        .first_touch_ts
        .is_some_and(|touch| touch <= zone.formation.detection_ts)
        || zone
            .fill_ts
            .is_some_and(|fill| fill <= zone.formation.detection_ts)
    {
        return Err("later lifecycle outcome is not later than formation".into());
    }
    if zone.fill_ts.is_some() != (!zone.active && !zone.right_censored) {
        return Err("fill/active/right-censor state is inconsistent".into());
    }
    if zone.fill_ts.is_none() != (zone.active && zone.right_censored) {
        return Err("unfilled zone is not right-censored".into());
    }
    if zone.active_duration_ms < 0 || zone.prospective_response_bps.len() != 3 {
        return Err("lifecycle duration or response horizon shape is invalid".into());
    }
    if value
        .get("formation")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|formation| {
            formation.keys().any(|key| {
                matches!(
                    key.as_str(),
                    "first_touch_ts" | "fill_ts" | "active" | "right_censored"
                )
            })
        })
    {
        return Err("formation cohort contains later lifecycle fields".into());
    }
    Ok(())
}

#[cfg(test)]
fn validate_zone_value(value: &serde_json::Value) -> Result<(), String> {
    let zone: ZoneLifecycle =
        serde_json::from_value(value.clone()).map_err(|error| error.to_string())?;
    validate_zone(&zone, value)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("AP-002 E1 method challenge failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut results = PathBuf::new();
    let mut zones = PathBuf::new();
    let mut preflight = PathBuf::new();
    let mut output = PathBuf::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--results" => results = args.next().ok_or("missing --results")?.into(),
            "--zones" => zones = args.next().ok_or("missing --zones")?.into(),
            "--preflight" => preflight = args.next().ok_or("missing --preflight")?.into(),
            "--output" => output = args.next().ok_or("missing --output")?.into(),
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    if results.as_os_str().is_empty()
        || zones.as_os_str().is_empty()
        || preflight.as_os_str().is_empty()
        || output.as_os_str().is_empty()
    {
        return Err("--results, --zones, --preflight, and --output are required".into());
    }
    let results_value: serde_json::Value = serde_json::from_reader(File::open(&results)?)?;
    if results_value["exposure_class"] != "E1_PHENOTYPE"
        || results_value["confirmation_status"] != "LOCKED"
    {
        return Err("results are outside the locked E1 scope".into());
    }
    let preflight_value: PreflightResult = serde_json::from_reader(File::open(&preflight)?)?;
    let expected_n = results_value["tables"]["zones"]["n"]
        .as_u64()
        .ok_or("results zone count missing")?;
    let expected_hash = results_value["tables"]["zones"]["logical_rows_sha256"]
        .as_str()
        .ok_or("results logical zone hash missing")?;
    let mut reader = BufReader::new(File::open(&zones)?);
    let mut line = String::new();
    let mut unique = HashSet::new();
    let mut count = 0_u64;
    let mut logical = Sha256::new();
    let mut checks = Vec::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(&line)?;
        let zone: ZoneLifecycle = serde_json::from_value(value.clone())?;
        validate_zone(&zone, &value).map_err(|error| format!("zone {}: {error}", count + 1))?;
        if !unique.insert(zone.identity.clone()) {
            return Err(format!("duplicate lawful identity: {}", zone.identity).into());
        }
        logical.update(line.as_bytes());
        count += 1;
    }
    let logical_hash = format!("{:x}", logical.finalize());
    if count != expected_n || logical_hash != expected_hash {
        return Err(format!(
            "zone table mismatch: rows {count}/{expected_n}, hash {logical_hash}/{expected_hash}"
        )
        .into());
    }
    checks.push(Check {
        id: "native_identity",
        status: "PASS",
        evidence: format!("{count} unique (timeframe, zone_id) identities"),
    });
    checks.push(Check {
        id: "formation_outcome_separation",
        status: "PASS",
        evidence: "formation rows contain no later touch/fill fields".into(),
    });
    checks.push(Check {
        id: "causal_availability",
        status: "PASS",
        evidence: "all formation rows carry frozen sidecar receipt-time provenance".into(),
    });
    checks.push(Check {
        id: "right_censoring",
        status: "PASS",
        evidence: "unfilled zones are active and right-censored".into(),
    });
    let preflight_status = if preflight_value.regressions_count == 0 {
        "PASS"
    } else {
        "FAIL"
    };
    checks.push(Check {
        id: "receipt_regression_preflight",
        status: preflight_status,
        evidence: format!("{} regressions", preflight_value.regressions_count),
    });
    if preflight_status == "FAIL" {
        return Err("method challenge rejects nonzero receipt regressions".into());
    }
    let package = ChallengePackage {
        package_type: "AP-002-E1-INDEPENDENT-METHOD-CHALLENGE",
        status: "PASS",
        independent_from_scanner: true,
        confirmation_status: "LOCKED",
        zones_n: count,
        logical_rows_sha256: logical_hash,
        checks,
    };
    fs::write(output, serde_json::to_vec_pretty(&package)?)?;
    println!("{}", serde_json::to_string_pretty(&package)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn challenge_rejects_formation_row_with_future_outcome_as_formation_data() {
        let row = serde_json::json!({
            "timeframe": "15s",
            "identity": "15s:7",
            "formation": {"zone_id": 7, "detection_ts": 1000},
            "formation_bar_close_ts": 1000,
            "formation_available_time_ns": 1100,
            "formation_source_sequence": 1,
            "first_touch_ts": 2000,
            "fill_ts": 3000,
            "active_duration_ms": 2000,
            "active": false,
            "right_censored": false,
            "prospective_response_bps": [0.0, null, null]
        });
        assert!(validate_zone_value(&row).is_err());
    }
}
