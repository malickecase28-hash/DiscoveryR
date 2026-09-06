#[path = "../development_view.rs"]
mod development_view;

use development_view::materialize;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = std::env::var("TRINITYR_ANALYTICAL_LAKE")
        .ok()
        .map(|lake| PathBuf::from(lake).join("fusion_markets/xauusd"));
    let mut inventory = PathBuf::from("instruments/XAUUSD/source_inventory.json");
    let mut view = PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\development");
    let mut audit = PathBuf::from(r"F:\TrinityR-views\XAUUSD\XAUUSD_DATA_SCOPE_V1\build-audit");
    let mut batch = development_view::DEFAULT_BATCH_SIZE;
    let mut copy = false;
    let mut verify = false;
    let mut code: Option<String> = std::env::var("DISCOVERYR_CODE_IDENTITY").ok();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source-root" => source = Some(args.next().ok_or("missing --source-root")?.into()),
            "--inventory" => inventory = args.next().ok_or("missing --inventory")?.into(),
            "--view-root" => view = args.next().ok_or("missing --view-root")?.into(),
            "--audit-root" => audit = args.next().ok_or("missing --audit-root")?.into(),
            "--batch-size" => batch = args.next().ok_or("missing --batch-size")?.parse()?,
            "--copy-full-parts" => copy = true,
            "--verify" => verify = true,
            "--code-identity" => code = Some(args.next().ok_or("missing --code-identity")?),
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    if verify {
        let manifest: development_view::Manifest =
            serde_json::from_reader(std::fs::File::open(view.join("view_manifest.json"))?)?;
        development_view::verify_view(&view, &manifest, batch)?;
        println!("XAUUSD DEVELOPMENT VIEW VERIFIED");
        return Ok(());
    }
    let code = code.ok_or(
        "DISCOVERYR_CODE_IDENTITY must be set to the worker commit SHA, or supply --code-identity",
    )?;
    let source = source.ok_or("TRINITYR_ANALYTICAL_LAKE must be set, or supply --source-root")?;
    let identity = materialize(&source, &inventory, &view, &audit, &code, batch, copy)?;
    println!("{identity}");
    Ok(())
}
