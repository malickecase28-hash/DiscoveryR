//! Generic artifact hash registry generator.  Produces a deterministic,
//! Rust-computed SHA-256 registry for a declared set of artifacts.
//!
//! Hard rule (PIPELINE_RULES.md): hash computation for committed artifact
//! registries is Rust-only.  This tool is the sanctioned generator.
//!
//! Guarantees:
//! - files are streamed (constant memory);
//! - output is sorted by logical name and JSON-serialized stably;
//! - missing files, duplicate logical names, and absolute or parent-traversing
//!   paths fail closed;
//! - with `--expect`, the tool fails closed unless every logical name in the
//!   expected registry matches the freshly computed hash;
//! - registry paths are stored relative to `--root` (never absolute).

use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::{Component, Path, PathBuf},
    process,
};

#[derive(Deserialize)]
struct ManifestEntry {
    logical_name: String,
    path: String,
}

#[derive(Deserialize)]
struct Manifest {
    entries: Vec<ManifestEntry>,
}

fn stream_sha256(path: &Path) -> Result<(String, u64), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(1 << 16, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1 << 16];
    let mut bytes = 0_u64;
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
        bytes += read as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), bytes))
}

fn safe_relative(path: &str) -> Result<PathBuf, String> {
    let relative = Path::new(path);
    if relative.is_absolute()
        || relative
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::RootDir))
    {
        return Err(format!("registry path must be relative and contained: {path}"));
    }
    Ok(relative.to_owned())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut root = PathBuf::new();
    let mut manifest_path = PathBuf::new();
    let mut output = PathBuf::new();
    let mut expect_path = PathBuf::new();
    let mut code_identity = String::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--root" => root = args.next().ok_or("missing --root")?.into(),
            "--manifest" => manifest_path = args.next().ok_or("missing --manifest")?.into(),
            "--output" => output = args.next().ok_or("missing --output")?.into(),
            "--expect" => expect_path = args.next().ok_or("missing --expect")?.into(),
            "--code-identity" => code_identity = args.next().ok_or("missing --code-identity")?,
            other => return Err(format!("unknown argument: {other}").into()),
        }
    }
    if root.as_os_str().is_empty()
        || manifest_path.as_os_str().is_empty()
        || output.as_os_str().is_empty()
        || code_identity.len() != 40
        || !code_identity.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(
            "--root, --manifest, --output and a 40-character --code-identity are required".into(),
        );
    }
    let manifest: Manifest = serde_json::from_reader(File::open(&manifest_path)?)?;
    let mut entries: BTreeMap<String, (String, String, u64)> = BTreeMap::new();
    for entry in &manifest.entries {
        if entry.logical_name.is_empty() {
            return Err("empty logical name in manifest".into());
        }
        if entries.contains_key(&entry.logical_name) {
            return Err(format!("duplicate logical name: {}", entry.logical_name).into());
        }
        let relative = safe_relative(&entry.path)?;
        let path = root.join(&relative);
        if !path.is_file() {
            return Err(format!("missing artifact: {}", entry.path).into());
        }
        let (sha, bytes) = stream_sha256(&path)?;
        entries.insert(
            entry.logical_name.clone(),
            (entry.path.clone(), sha, bytes),
        );
    }
    if !expect_path.as_os_str().is_empty() {
        let expected: BTreeMap<String, String> =
            serde_json::from_reader(File::open(&expect_path)?)?;
        let mut mismatches = Vec::new();
        for (name, (_, sha, _)) in &entries {
            match expected.get(name) {
                Some(expected_sha) if expected_sha == sha => {}
                Some(expected_sha) => mismatches.push(format!(
                    "hash mismatch for {name}: registry {expected_sha} vs computed {sha}"
                )),
                None => mismatches.push(format!("expected registry has no entry for {name}")),
            }
        }
        for name in expected.keys() {
            if !entries.contains_key(name) {
                mismatches.push(format!("computed registry is missing expected entry {name}"));
            }
        }
        if !mismatches.is_empty() {
            return Err(format!(
                "registry verification failed ({}): {}",
                mismatches.len(),
                mismatches.join("; ")
            )
            .into());
        }
        println!(
            "registry verification: {}/{} hashes equal the expected values",
            entries.len(),
            expected.len()
        );
    }
    let registry = json!({
        "artifact_type": "ARTIFACT_HASH_REGISTRY",
        "generator": "artifact_hash_registry",
        "generator_code_identity": code_identity,
        "root": root.to_string_lossy().replace('\\', "/"),
        "entries": entries.iter().map(|(name, (path, sha, bytes))| json!({
            "logical_name": name,
            "path": path,
            "sha256": sha,
            "bytes": bytes,
        })).collect::<Vec<_>>(),
    });
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output, serde_json::to_vec_pretty(&registry)?)?;
    println!("wrote {} registry entries to {}", entries.len(), output.display());
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("artifact_hash_registry failed: {error}");
        process::exit(1);
    }
}
