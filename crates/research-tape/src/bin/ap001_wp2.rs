use research_tape::ap001_wp2::{self, Wp2Config, CONTRACT_DEFAULT};
use std::{path::PathBuf, process};

fn main() {
    let code = match parse_args() {
        Ok(config) => match ap001_wp2::run(config) { Ok(code) => code, Err(error) => { eprintln!("ap001_wp2 failed: {error}"); 2 } },
        Err(error) => { eprintln!("ap001_wp2: {error}"); 2 }
    };
    process::exit(code);
}

fn parse_args() -> Result<Wp2Config, String> {
    let mut config = Wp2Config { contract_path: PathBuf::from(CONTRACT_DEFAULT), ..Default::default() };
    let mut args=std::env::args().skip(1);
    while let Some(arg)=args.next() { match arg.as_str() {
        "--contract" => config.contract_path=PathBuf::from(args.next().ok_or("missing value for --contract")?),
        "--lake-root" => config.lake_root=Some(PathBuf::from(args.next().ok_or("missing value for --lake-root")?)),
        "--out-dir" => config.out_dir=Some(PathBuf::from(args.next().ok_or("missing value for --out-dir")?)),
        "--bar-view-root" => config.bar_view_root=PathBuf::from(args.next().ok_or("missing value for --bar-view-root")?),
        "--sidecar" => config.sidecar_path=PathBuf::from(args.next().ok_or("missing value for --sidecar")?),
        "--sidecar-manifest" => config.sidecar_manifest_path=PathBuf::from(args.next().ok_or("missing value for --sidecar-manifest")?),
        "--batch-size" => { let v=args.next().ok_or("missing value for --batch-size")?; config.batch_size=v.parse().map_err(|_| format!("invalid --batch-size: {v}"))?; }
        "--single-run" => config.single_run=true,
        other => return Err(format!("unknown argument: {other}")),
    }}
    Ok(config)
}
