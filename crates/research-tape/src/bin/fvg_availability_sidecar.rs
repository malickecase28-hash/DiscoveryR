use research_tape::fvg_availability::{default_timeframes, materialize, parse_timeframes};
use std::{env, path::PathBuf, process};

fn main() {
    if let Err(error) = run() {
        eprintln!("fvg availability sidecar failed: {error}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut view_root = PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development");
    let mut output = PathBuf::from(
        r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\authority\fvg_availability_v1.jsonl",
    );
    let mut timeframes = default_timeframes()?;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--view-root" => view_root = args.next().ok_or("missing --view-root")?.into(),
            "--output" => output = args.next().ok_or("missing --output")?.into(),
            "--timeframes" => {
                timeframes = parse_timeframes(&args.next().ok_or("missing --timeframes")?)?
            }
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    let summary = materialize(&view_root, &output, timeframes)?;
    let manifest_path = output.with_extension("manifest.json");
    if manifest_path.exists() {
        return Err(format!(
            "sidecar manifest already exists: {}",
            manifest_path.display()
        )
        .into());
    }
    std::fs::write(&manifest_path, serde_json::to_vec_pretty(&summary)?)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
