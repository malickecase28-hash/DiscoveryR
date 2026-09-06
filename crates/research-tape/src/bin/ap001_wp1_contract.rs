//! AP-001 WP1 contract-and-benchmark scanner binary.
//!
//! Defaults follow the frozen contract; all arguments are optional overrides:
//!   --contract <path>    frozen contract JSON (default: AP-001_WP1_CONTRACT.json)
//!   --lake-root <path>   lake root override (default: contract input_identity)
//!   --out-dir <path>     output directory (default: contract directory)
//!   --batch-size <n>     parquet batch size (default: 16384)

use research_tape::ap001_drift_burst::{self, Wp1Config, CONTRACT_DEFAULT};
use std::{path::PathBuf, process};

fn main() {
    let code = match parse_args() {
        Ok(config) => match ap001_drift_burst::run(config) {
            Ok(code) => code,
            Err(error) => {
                eprintln!("ap001_wp1_contract failed: {error}");
                2
            }
        },
        Err(error) => {
            eprintln!("ap001_wp1_contract: {error}");
            2
        }
    };
    process::exit(code);
}

fn parse_args() -> Result<Wp1Config, String> {
    let mut config = Wp1Config {
        contract_path: PathBuf::from(CONTRACT_DEFAULT),
        lake_root: None,
        out_dir: None,
        batch_size: 16384,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--contract" => {
                config.contract_path = PathBuf::from(args.next().ok_or("missing value for --contract")?);
            }
            "--lake-root" => {
                config.lake_root = Some(PathBuf::from(args.next().ok_or("missing value for --lake-root")?));
            }
            "--out-dir" => {
                config.out_dir = Some(PathBuf::from(args.next().ok_or("missing value for --out-dir")?));
            }
            "--batch-size" => {
                let value = args.next().ok_or("missing value for --batch-size")?;
                config.batch_size = value.parse().map_err(|_| format!("invalid --batch-size: {value}"))?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(config)
}
