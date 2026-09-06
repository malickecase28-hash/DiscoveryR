#[path = "../development_view.rs"]
mod development_view;

use development_view::materialize;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut source: Option<PathBuf> = None;
    let mut inventory: Option<PathBuf> = None;
    let mut scope: Option<PathBuf> = None;
    let mut view: Option<PathBuf> = None;
    let mut audit: Option<PathBuf> = None;
    let mut batch = development_view::DEFAULT_BATCH_SIZE;
    let mut copy = true;
    let mut verify = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source-root" => source = Some(args.next().ok_or("missing --source-root")?.into()),
            "--inventory" => inventory = Some(args.next().ok_or("missing --inventory")?.into()),
            "--scope" => scope = Some(args.next().ok_or("missing --scope")?.into()),
            "--view-root" => view = Some(args.next().ok_or("missing --view-root")?.into()),
            "--audit-root" => audit = Some(args.next().ok_or("missing --audit-root")?.into()),
            "--batch-size" => batch = args.next().ok_or("missing --batch-size")?.parse()?,
            "--copy-full-parts" => copy = true,
            "--verify" => verify = true,
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    let expected =
        development_view::load_expected_scope(scope.as_deref().ok_or("missing --scope")?)?;
    if verify {
        let view = view.as_deref().ok_or("missing --view-root")?;
        let manifest: development_view::Manifest =
            serde_json::from_reader(std::fs::File::open(view.join("view_manifest.json"))?)?;
        development_view::verify_view_with_expected(view, &manifest, batch, &expected)?;
        println!("{} DEVELOPMENT VIEW VERIFIED", expected.instrument);
        return Ok(());
    }
    let code = std::env::var("DISCOVERYR_CODE_IDENTITY")
        .map_err(|_| "DISCOVERYR_CODE_IDENTITY must be set to the worker commit SHA")?;
    let identity = materialize(
        source.as_deref().ok_or("missing --source-root")?,
        inventory.as_deref().ok_or("missing --inventory")?,
        view.as_deref().ok_or("missing --view-root")?,
        audit.as_deref().ok_or("missing --audit-root")?,
        &expected,
        &code,
        batch,
        copy,
    )?;
    println!("{identity}");
    Ok(())
}
