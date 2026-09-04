use research_contracts::submission::{seal, verify, SealOptions};
use std::{env, path::PathBuf};

fn value(name: &str, args: &mut impl Iterator<Item = String>) -> Result<String, String> {
    if args.next().as_deref() != Some(name) {
        return Err(format!("missing {name}"));
    }
    args.next()
        .ok_or_else(|| format!("missing value for {name}"))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    if args.next().as_deref() == Some("verify") {
        let repo = args
            .next()
            .ok_or("usage: seal_submission verify <repo> <sealed-dir>")?;
        let dir = args
            .next()
            .ok_or("usage: seal_submission verify <repo> <sealed-dir>")?;
        verify(PathBuf::from(repo).as_path(), PathBuf::from(dir).as_path())?;
        println!("VERIFIED");
        return Ok(());
    }
    let mut args = env::args().skip(1);
    let repo = PathBuf::from(value("--repo", &mut args)?);
    let program = value("--program", &mut args)?;
    let role = value("--role", &mut args)?;
    let scientific_base_sha = value("--scientific-base-sha", &mut args)?;
    let workspace_root = PathBuf::from(value("--workspace-root", &mut args)?);
    let output_root = PathBuf::from(value("--output-root", &mut args)?);
    let forbidden_terms = match args.next() {
        Some(flag) if flag == "--forbidden-terms" => {
            Some(PathBuf::from(args.next().ok_or("missing denylist path")?))
        }
        Some(flag) => return Err(format!("unexpected argument {flag}").into()),
        None => None,
    };
    let result = seal(&SealOptions {
        repo,
        program,
        role,
        scientific_base_sha,
        workspace_root,
        output_root,
        forbidden_terms,
    })?;
    println!("{}", result.logical_submission_identity);
    Ok(())
}
