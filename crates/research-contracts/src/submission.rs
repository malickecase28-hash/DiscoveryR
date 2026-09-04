use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

const MAX_FILE: u64 = 8 * 1024 * 1024;
const MAX_TOTAL: u64 = 32 * 1024 * 1024;
const MAX_FILES: usize = 128;
const FORBIDDEN_KEYS: [&str; 9] = [
    "provider",
    "provider_id",
    "provider_identity",
    "model",
    "model_id",
    "model_identity",
    "runtime",
    "runtime_slot",
    "provider_mapping",
];

#[derive(Debug, Clone)]
pub struct SealOptions {
    pub repo: PathBuf,
    pub program: String,
    pub role: String,
    pub scientific_base_sha: String,
    pub workspace_root: PathBuf,
    pub output_root: PathBuf,
    pub forbidden_terms: Option<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileRecord {
    pub relative_path: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SubmissionManifest {
    pub schema_version: u32,
    pub program_id: String,
    pub role_id: String,
    pub scientific_base_sha: String,
    pub assignment_relative_path: String,
    pub assignment_sha256: String,
    pub files: Vec<FileRecord>,
    pub total_bytes: u64,
    pub logical_submission_identity: String,
}

struct TempGuard {
    path: PathBuf,
    keep: bool,
}
impl Drop for TempGuard {
    fn drop(&mut self) {
        if !self.keep {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

fn err(s: impl Into<String>) -> Box<dyn std::error::Error> {
    s.into().into()
}

fn valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

fn sha_bytes(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn framed(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_le_bytes());
    h.update(bytes);
}

fn identity(m: &SubmissionManifest) -> String {
    let mut h = Sha256::new();
    for value in [
        &m.program_id,
        &m.role_id,
        &m.scientific_base_sha,
        &m.assignment_sha256,
    ] {
        framed(&mut h, value.as_bytes());
    }
    for f in &m.files {
        framed(&mut h, f.relative_path.as_bytes());
        framed(&mut h, &f.byte_length.to_le_bytes());
        framed(&mut h, f.sha256.as_bytes());
    }
    hex(&h.finalize())
}

fn reject_json(v: &serde_json::Value, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    match v {
        serde_json::Value::Object(map) => {
            for (key, value) in map {
                let key_lower = key.to_ascii_lowercase();
                if FORBIDDEN_KEYS.iter().any(|bad| key_lower == *bad) {
                    return Err(err(format!("forbidden operational identity key in {path}")));
                }
                reject_json(value, path)?;
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                reject_json(value, path)?
            }
        }
        _ => {}
    }
    Ok(())
}

fn relative(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    if path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(err("unsafe submission path"));
    }
    Ok(path
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/"))
}

fn reparse_or_symlink(meta: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        meta.file_attributes() & 0x400 != 0 || meta.file_type().is_symlink()
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}

fn reject_unsafe_ancestors(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        if current.exists() && reparse_or_symlink(&fs::symlink_metadata(&current)?) {
            return Err(err("symlink or reparse point in submission path"));
        }
    }
    Ok(())
}

fn git_bytes(repo: &Path, args: &[String]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(err(format!(
            "git lookup failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}

fn collect(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(String, PathBuf)>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            return Err(err("symlinks are not allowed"));
        }
        if meta.is_dir() {
            collect(root, &path, out)?;
        } else if meta.is_file() {
            out.push((relative(path.strip_prefix(root)?)?, path));
        } else {
            return Err(err("only regular files are allowed"));
        }
    }
    Ok(())
}

fn scan_denylist(
    files: &[(String, PathBuf)],
    terms: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    if terms.is_empty() {
        return Ok(());
    }
    for (name, source) in files {
        if !matches!(
            Path::new(name).extension().and_then(|e| e.to_str()),
            Some("md" | "txt" | "json")
        ) {
            continue;
        }
        let text = String::from_utf8(fs::read(source)?)
            .map_err(|_| err(format!("non-UTF-8 text in {name}")))?;
        for (line_no, line) in text.lines().enumerate() {
            let lower = line.to_ascii_lowercase();
            if terms.iter().any(|term| lower.contains(term)) {
                return Err(err(format!(
                    "forbidden term in {name} line {}",
                    line_no + 1
                )));
            }
        }
    }
    Ok(())
}

pub fn seal(opts: &SealOptions) -> Result<SubmissionManifest, Box<dyn std::error::Error>> {
    if !valid_id(&opts.program) || !valid_id(&opts.role) {
        return Err(err("invalid program or role"));
    }
    if opts.scientific_base_sha.len() != 40
        || !opts
            .scientific_base_sha
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
    {
        return Err(err("scientific base SHA must be 40 hex characters"));
    }
    let repo = fs::canonicalize(&opts.repo)?;
    let workspace_root = fs::canonicalize(&opts.workspace_root)?;
    let role_workspace_raw = workspace_root.join(&opts.program).join(&opts.role);
    let submission_raw = role_workspace_raw.join("submission");
    reject_unsafe_ancestors(&submission_raw)?;
    let role_workspace = fs::canonicalize(&role_workspace_raw)?;
    let workspace = fs::canonicalize(&submission_raw)?;
    if !workspace.starts_with(&role_workspace) {
        return Err(err("submission root escapes role workspace"));
    }
    let commit = format!("{}^{{commit}}", opts.scientific_base_sha);
    let _ = git_bytes(&repo, &["cat-file".into(), "-e".into(), commit])?;
    let assignment_spec = format!(
        "{}:agent_harness/assignments/{}/{}.json",
        opts.scientific_base_sha, opts.program, opts.role
    );
    let assignment = git_bytes(&repo, &["show".into(), assignment_spec])?;
    let assignment_value: serde_json::Value = serde_json::from_slice(&assignment)?;
    if assignment_value.get("program_id").and_then(|v| v.as_str()) != Some(&opts.program)
        || assignment_value.get("role_id").and_then(|v| v.as_str()) != Some(&opts.role)
    {
        return Err(err("assignment identity mismatch"));
    }
    reject_json(&assignment_value, "assignment")?;
    let mut paths = Vec::new();
    collect(&workspace, &workspace, &mut paths)?;
    paths.sort_by(|a, b| a.0.cmp(&b.0));
    if paths.len() > MAX_FILES {
        return Err(err("too many submission files"));
    }
    let terms = opts
        .forbidden_terms
        .as_ref()
        .map(|p| {
            fs::read_to_string(p).map(|s| {
                s.lines()
                    .map(|l| l.trim().to_ascii_lowercase())
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
            })
        })
        .transpose()?
        .unwrap_or_default();
    scan_denylist(&paths, &terms)?;
    let mut files = Vec::new();
    let mut total: u64 = 0;
    for (name, path) in &paths {
        let length = fs::metadata(path)?.len();
        if length > MAX_FILE
            || total
                .checked_add(length)
                .filter(|n| *n <= MAX_TOTAL)
                .is_none()
        {
            return Err(err("submission size bound exceeded"));
        }
        let bytes = fs::read(path)?;
        if Path::new(name).extension().and_then(|e| e.to_str()) == Some("json") {
            reject_json(&serde_json::from_slice(&bytes)?, name)?;
        }
        files.push(FileRecord {
            relative_path: name.clone(),
            byte_length: length,
            sha256: sha_bytes(&bytes),
        });
        total += length;
    }
    let mut manifest = SubmissionManifest {
        schema_version: 1,
        program_id: opts.program.clone(),
        role_id: opts.role.clone(),
        scientific_base_sha: opts.scientific_base_sha.clone(),
        assignment_relative_path: format!(
            "agent_harness/assignments/{}/{}.json",
            opts.program, opts.role
        ),
        assignment_sha256: sha_bytes(&assignment),
        files,
        total_bytes: total,
        logical_submission_identity: String::new(),
    };
    manifest.logical_submission_identity = identity(&manifest);
    fs::create_dir_all(&opts.output_root)?;
    let output_root = fs::canonicalize(&opts.output_root)?;
    for forbidden in [&repo, &workspace_root, &role_workspace, &workspace] {
        if output_root.starts_with(forbidden) {
            return Err(err("sealed output root must be external"));
        }
    }
    let role_root = output_root.join(&opts.program).join(&opts.role);
    fs::create_dir_all(&role_root)?;
    let final_dir = role_root.join(&manifest.logical_submission_identity);
    if final_dir.exists() {
        verify(&final_dir)?;
        return Ok(manifest);
    }
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temp = role_root.join(format!(".tmp-{}-{nonce}", std::process::id()));
    fs::create_dir(&temp)?;
    let mut temp_guard = TempGuard {
        path: temp.clone(),
        keep: false,
    };
    let files_dir = temp.join("files");
    fs::create_dir(&files_dir)?;
    for (record, (_, source)) in manifest.files.iter().zip(paths.iter()) {
        let target = files_dir.join(&record.relative_path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&target)?;
        let mut input = File::open(source)?;
        std::io::copy(&mut input, &mut out)?;
        out.flush()?;
        drop(input);
        drop(out);
        if sha_bytes(&fs::read(&target)?) != record.sha256 {
            return Err(err("sealed copy verification failed"));
        }
    }
    let text = serde_json::to_string_pretty(&manifest)? + "\n";
    let mut out = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(temp.join("submission_manifest.json"))?;
    out.write_all(text.as_bytes())?;
    out.flush()?;
    drop(out);
    match fs::rename(&temp, &final_dir) {
        Ok(()) => {
            temp_guard.keep = true;
            Ok(manifest)
        }
        Err(_) if final_dir.exists() => {
            verify(&final_dir)?;
            Ok(manifest)
        }
        Err(e) => Err(format!("publish {} -> {}: {e}", temp.display(), final_dir.display()).into()),
    }
}

pub fn verify(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest: SubmissionManifest =
        serde_json::from_slice(&fs::read(dir.join("submission_manifest.json"))?)?;
    if identity(&manifest) != manifest.logical_submission_identity {
        return Err(err("logical submission identity mismatch"));
    }
    if dir.file_name().and_then(|n| n.to_str()) != Some(&manifest.logical_submission_identity)
        || dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            != Some(&manifest.role_id)
        || dir
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            != Some(&manifest.program_id)
    {
        return Err(err("sealed directory identity or layout mismatch"));
    }
    if manifest.schema_version != 1
        || manifest.program_id.is_empty()
        || !valid_id(&manifest.program_id)
        || !valid_id(&manifest.role_id)
        || manifest.scientific_base_sha.len() != 40
        || !manifest
            .scientific_base_sha
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
        || manifest.assignment_relative_path
            != format!(
                "agent_harness/assignments/{}/{}.json",
                manifest.program_id, manifest.role_id
            )
        || manifest.files.len() > MAX_FILES
    {
        return Err(err("invalid sealed manifest metadata"));
    }
    let root_entries = fs::read_dir(dir)?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<Vec<_>, _>>()?;
    if root_entries.len() != 2
        || !root_entries.iter().any(|name| name == "files")
        || !root_entries
            .iter()
            .any(|name| name == "submission_manifest.json")
    {
        return Err(err("sealed root contains unexpected files"));
    }
    let mut found = Vec::new();
    collect(&dir.join("files"), &dir.join("files"), &mut found)?;
    found.sort_by(|a, b| a.0.cmp(&b.0));
    if manifest
        .files
        .iter()
        .any(|f| match relative(Path::new(&f.relative_path)) {
            Ok(path) => path != f.relative_path,
            Err(_) => true,
        })
        || manifest
            .files
            .windows(2)
            .any(|w| w[0].relative_path >= w[1].relative_path)
        || manifest.files.iter().any(|f| {
            manifest
                .files
                .iter()
                .filter(|x| x.relative_path == f.relative_path)
                .count()
                > 1
        })
    {
        return Err(err("manifest file list is unsafe, unsorted, or duplicated"));
    }
    if found.len() != manifest.files.len()
        || found.iter().map(|x| &x.0).collect::<Vec<_>>()
            != manifest
                .files
                .iter()
                .map(|x| &x.relative_path)
                .collect::<Vec<_>>()
    {
        return Err(err("sealed file set mismatch"));
    }
    let mut total: u64 = 0;
    for (record, (_, path)) in manifest.files.iter().zip(found) {
        let bytes = fs::read(path)?;
        if record.byte_length > MAX_FILE
            || total.checked_add(record.byte_length).is_none()
            || bytes.len() as u64 != record.byte_length
            || sha_bytes(&bytes) != record.sha256
        {
            return Err(err("sealed file hash mismatch"));
        }
        total += record.byte_length;
    }
    if total != manifest.total_bytes {
        return Err(err("sealed total size mismatch"));
    }
    Ok(())
}
