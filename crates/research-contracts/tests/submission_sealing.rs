use research_contracts::submission::{seal, verify, FileRecord, SealOptions, SubmissionManifest};
use sha2::{Digest, Sha256};
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("seal-test-{}-{}", std::process::id(), unique()));
    fs::create_dir_all(&p).unwrap();
    p
}

fn unique() -> u128 {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    ((timestamp & u128::from(u64::MAX)) << 64)
        | u128::from(TEMP_COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn manifest_identity(m: &SubmissionManifest) -> String {
    let mut h = Sha256::new();
    for value in [
        &m.program_id,
        &m.role_id,
        &m.scientific_base_sha,
        &m.assignment_sha256,
    ] {
        h.update((value.len() as u64).to_le_bytes());
        h.update(value.as_bytes());
    }
    for f in &m.files {
        h.update((f.relative_path.len() as u64).to_le_bytes());
        h.update(f.relative_path.as_bytes());
        h.update((8u64).to_le_bytes());
        h.update(f.byte_length.to_le_bytes());
        h.update((f.sha256.len() as u64).to_le_bytes());
        h.update(f.sha256.as_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn git(args: &[&str], cwd: &std::path::Path) -> String {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap().trim().to_owned()
}

#[test]
fn valid_submission_seals_deterministically() {
    let root = temp();
    let repo = root.join("repo");
    let workspace = root.join("runs/AP-001/A-01/submission");
    let output = root.join("sealed");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(repo.join("agent_harness/assignments/AP-001")).unwrap();
    fs::write(
        repo.join("agent_harness/assignments/AP-001/A-01.json"),
        br#"{"program_id":"AP-001","role_id":"A-01","revision":1}"#,
    )
    .unwrap();
    git(&["init", "-q"], &repo);
    git(&["config", "user.email", "test@example.invalid"], &repo);
    git(&["config", "user.name", "test"], &repo);
    git(&["add", "."], &repo);
    git(&["commit", "-qm", "assignment"], &repo);
    let base = git(&["rev-parse", "HEAD"], &repo);
    fs::write(workspace.join("answer.md"), "answer\n").unwrap();
    let opts = SealOptions {
        repo,
        program: "AP-001".into(),
        role: "A-01".into(),
        scientific_base_sha: base,
        workspace_root: root.join("runs"),
        output_root: output,
        forbidden_terms: None,
    };
    let first = seal(&opts).unwrap();
    let second = seal(&opts).unwrap();
    assert_eq!(
        first.logical_submission_identity,
        second.logical_submission_identity
    );
}

#[test]
fn frozen_assignment_is_loaded_from_claimed_commit() {
    let root = temp();
    let repo = root.join("repo");
    let workspace = root.join("runs/AP-001/A-01/submission");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(repo.join("agent_harness/assignments/AP-001")).unwrap();
    let assignment = repo.join("agent_harness/assignments/AP-001/A-01.json");
    fs::write(
        &assignment,
        br#"{"program_id":"AP-001","role_id":"A-01","revision":1}"#,
    )
    .unwrap();
    git(&["init", "-q"], &repo);
    git(&["config", "user.email", "test@example.invalid"], &repo);
    git(&["config", "user.name", "test"], &repo);
    git(&["add", "."], &repo);
    git(&["commit", "-qm", "one"], &repo);
    let base = git(&["rev-parse", "HEAD"], &repo);
    fs::write(
        &assignment,
        br#"{"program_id":"WRONG","role_id":"A-01","revision":2}"#,
    )
    .unwrap();
    fs::write(workspace.join("answer.md"), "answer").unwrap();
    let opts = SealOptions {
        repo,
        program: "AP-001".into(),
        role: "A-01".into(),
        scientific_base_sha: base,
        workspace_root: root.join("runs"),
        output_root: root.join("sealed"),
        forbidden_terms: None,
    };
    assert!(seal(&opts).is_ok());
}

fn fixture() -> (std::path::PathBuf, SealOptions) {
    let root = temp();
    let repo = root.join("repo");
    let workspace = root.join("runs/AP-001/A-01/submission");
    fs::create_dir_all(&workspace).unwrap();
    fs::create_dir_all(repo.join("agent_harness/assignments/AP-001")).unwrap();
    fs::write(
        repo.join("agent_harness/assignments/AP-001/A-01.json"),
        br#"{"program_id":"AP-001","role_id":"A-01"}"#,
    )
    .unwrap();
    fs::write(workspace.join("answer.md"), "answer").unwrap();
    git(&["init", "-q"], &repo);
    git(&["config", "user.email", "test@example.invalid"], &repo);
    git(&["config", "user.name", "test"], &repo);
    git(&["add", "."], &repo);
    git(&["commit", "-qm", "assignment"], &repo);
    let base = git(&["rev-parse", "HEAD"], &repo);
    (
        root.clone(),
        SealOptions {
            repo,
            program: "AP-001".into(),
            role: "A-01".into(),
            scientific_base_sha: base,
            workspace_root: root.join("runs"),
            output_root: root.join("sealed"),
            forbidden_terms: None,
        },
    )
}

#[test]
fn fake_commit_and_contained_output_are_rejected() {
    let (_, mut opts) = fixture();
    opts.scientific_base_sha = "a".repeat(40);
    assert!(seal(&opts).is_err());
    let (_, mut opts) = fixture();
    opts.output_root = opts.repo.join("sealed");
    assert!(seal(&opts).is_err());
    let (_, mut opts) = fixture();
    opts.output_root = opts.workspace_root.join("sealed");
    assert!(seal(&opts).is_err());
}

#[cfg(unix)]
#[test]
fn submission_root_symlink_is_rejected() {
    use std::os::unix::fs::symlink;
    let (root, opts) = fixture();
    let real = root.join("elsewhere");
    fs::create_dir_all(&real).unwrap();
    let submission = opts.workspace_root.join("AP-001/A-01/submission");
    fs::remove_dir(&submission).unwrap();
    symlink(&real, &submission).unwrap();
    assert!(seal(&opts).is_err());
}

#[cfg(windows)]
#[test]
fn submission_root_reparse_is_rejected_when_available() {
    use std::os::windows::fs::symlink_dir;
    let (root, opts) = fixture();
    let real = root.join("elsewhere");
    fs::create_dir_all(&real).unwrap();
    let submission = opts.workspace_root.join("AP-001/A-01/submission");
    fs::remove_file(submission.join("answer.md")).unwrap();
    fs::remove_dir(&submission).unwrap();
    if symlink_dir(&real, &submission).is_err() {
        return;
    }
    assert!(seal(&opts).is_err());
}

#[test]
fn tamper_and_renamed_directory_fail_verification() {
    let (root, opts) = fixture();
    let manifest = seal(&opts).unwrap();
    let dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&manifest.logical_submission_identity);
    fs::write(dir.join("files/answer.md"), "tampered").unwrap();
    assert!(research_contracts::submission::verify(&opts.repo, &dir).is_err());
    fs::write(dir.join("files/answer.md"), "answer").unwrap();
    let renamed = dir.with_file_name("misleading");
    fs::rename(&dir, &renamed).unwrap();
    assert!(research_contracts::submission::verify(&opts.repo, &renamed).is_err());
    let _ = root;
}

#[cfg(windows)]
#[test]
fn nested_reparse_point_is_rejected() {
    use std::os::windows::fs::symlink_dir;
    let (root, opts) = fixture();
    let real = root.join("elsewhere");
    fs::create_dir_all(&real).unwrap();
    let nested = opts.workspace_root.join("AP-001/A-01/submission/nested");
    fs::remove_file(opts.workspace_root.join("AP-001/A-01/submission/answer.md")).unwrap();
    fs::create_dir(&nested).unwrap();
    fs::remove_dir(&nested).unwrap();
    if symlink_dir(&real, &nested).is_err() {
        return;
    }
    assert!(seal(&opts).is_err());
}

#[cfg(unix)]
#[test]
fn sealed_files_root_symlink_is_rejected() {
    use std::os::unix::fs::symlink;
    let (root, opts) = fixture();
    let manifest = seal(&opts).unwrap();
    let dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&manifest.logical_submission_identity);
    let real = root.join("elsewhere");
    fs::create_dir_all(&real).unwrap();
    fs::remove_dir_all(dir.join("files")).unwrap();
    symlink(&real, dir.join("files")).unwrap();
    assert!(research_contracts::submission::verify(&opts.repo, &dir).is_err());
}

#[cfg(windows)]
#[test]
fn sealed_files_root_reparse_is_rejected_when_available() {
    use std::os::windows::fs::symlink_dir;
    let (root, opts) = fixture();
    let manifest = seal(&opts).unwrap();
    let dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&manifest.logical_submission_identity);
    let real = root.join("elsewhere");
    fs::create_dir_all(&real).unwrap();
    fs::remove_dir_all(dir.join("files")).unwrap();
    if symlink_dir(&real, dir.join("files")).is_err() {
        return;
    }
    assert!(research_contracts::submission::verify(&opts.repo, &dir).is_err());
}

#[cfg(unix)]
#[test]
fn sealed_manifest_symlink_is_rejected() {
    use std::os::unix::fs::symlink;
    let (root, opts) = fixture();
    let manifest = seal(&opts).unwrap();
    let dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&manifest.logical_submission_identity);
    let external = root.join("external-manifest.json");
    fs::write(
        &external,
        fs::read(dir.join("submission_manifest.json")).unwrap(),
    )
    .unwrap();
    fs::remove_file(dir.join("submission_manifest.json")).unwrap();
    symlink(external, dir.join("submission_manifest.json")).unwrap();
    assert!(research_contracts::submission::verify(&opts.repo, &dir).is_err());
}

#[cfg(windows)]
#[test]
fn sealed_manifest_reparse_is_rejected_when_available() {
    use std::os::windows::fs::symlink_file;
    let (root, opts) = fixture();
    let manifest = seal(&opts).unwrap();
    let dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&manifest.logical_submission_identity);
    let external = root.join("external-manifest.json");
    fs::write(
        &external,
        fs::read(dir.join("submission_manifest.json")).unwrap(),
    )
    .unwrap();
    fs::remove_file(dir.join("submission_manifest.json")).unwrap();
    if symlink_file(external, dir.join("submission_manifest.json")).is_err() {
        return;
    }
    assert!(research_contracts::submission::verify(&opts.repo, &dir).is_err());
}

#[test]
fn forged_manifest_provenance_is_rejected() {
    let (_, opts) = fixture();
    let manifest = seal(&opts).unwrap();
    let dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&manifest.logical_submission_identity);
    let mut forged: SubmissionManifest =
        serde_json::from_slice(&fs::read(dir.join("submission_manifest.json")).unwrap()).unwrap();
    forged.assignment_sha256 = "0".repeat(64);
    forged.logical_submission_identity = manifest_identity(&forged);
    fs::write(
        dir.join("submission_manifest.json"),
        serde_json::to_vec_pretty(&forged).unwrap(),
    )
    .unwrap();
    assert!(research_contracts::submission::verify(&opts.repo, &dir).is_err());
}

#[test]
fn forged_manifest_over_total_size_bound_is_rejected() {
    let (_, opts) = fixture();
    let manifest = seal(&opts).unwrap();
    let dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&manifest.logical_submission_identity);
    let mut forged: SubmissionManifest =
        serde_json::from_slice(&fs::read(dir.join("submission_manifest.json")).unwrap()).unwrap();
    for i in 0..4 {
        let name = format!("extra-{i}.bin");
        let bytes = vec![i as u8; 8 * 1024 * 1024];
        fs::write(dir.join("files").join(&name), &bytes).unwrap();
        forged.files.push(FileRecord {
            relative_path: name,
            byte_length: bytes.len() as u64,
            sha256: hex(&Sha256::digest(&bytes)),
        });
        forged.total_bytes += bytes.len() as u64;
    }
    forged
        .files
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    forged.logical_submission_identity = manifest_identity(&forged);
    fs::write(
        dir.join("submission_manifest.json"),
        serde_json::to_vec_pretty(&forged).unwrap(),
    )
    .unwrap();
    let final_dir = opts
        .output_root
        .join("AP-001/A-01")
        .join(&forged.logical_submission_identity);
    fs::rename(&dir, &final_dir).unwrap();
    assert!(verify(&opts.repo, &final_dir).is_err());
}

#[test]
fn concurrent_identical_seals_share_one_directory() {
    let (_, opts) = fixture();
    let shared = std::sync::Arc::new(opts);
    let mut joins = Vec::new();
    for _ in 0..4 {
        let opts = shared.clone();
        joins.push(std::thread::spawn(move || {
            seal(&opts).unwrap().logical_submission_identity
        }));
    }
    let ids: Vec<_> = joins.into_iter().map(|j| j.join().unwrap()).collect();
    assert!(ids.windows(2).all(|w| w[0] == w[1]));
}

#[test]
fn concurrent_different_seals_survive() {
    let (_, first) = fixture();
    let (_, mut second) = fixture();
    second.output_root = first.output_root.clone();
    let output = first.output_root.clone();
    fs::write(
        second
            .workspace_root
            .join("AP-001/A-01/submission/answer.md"),
        "different",
    )
    .unwrap();
    let a = std::thread::spawn(move || seal(&first).unwrap().logical_submission_identity);
    let b = std::thread::spawn(move || seal(&second).unwrap().logical_submission_identity);
    let a = a.join().unwrap();
    let b = b.join().unwrap();
    assert_ne!(a, b);
    assert_eq!(fs::read_dir(output.join("AP-001/A-01")).unwrap().count(), 2);
}

#[test]
fn provider_keys_and_denylist_terms_are_rejected() {
    let (_, opts) = fixture();
    fs::write(
        opts.workspace_root
            .join("AP-001/A-01/submission/claim.json"),
        br#"{"nested":{"MoDeL":"x"}}"#,
    )
    .unwrap();
    assert!(seal(&opts).is_err());
    let (root, mut opts) = fixture();
    let denylist = root.join("private-denylist.txt");
    fs::write(&denylist, "private-provider").unwrap();
    opts.forbidden_terms = Some(denylist.clone());
    fs::write(
        opts.workspace_root.join("AP-001/A-01/submission/review.md"),
        "private-provider appears here",
    )
    .unwrap();
    assert!(seal(&opts).is_err());
    assert!(!opts.output_root.exists());
    assert!(denylist.exists());
}

#[test]
fn file_and_count_bounds_are_enforced() {
    let (_, opts) = fixture();
    fs::write(
        opts.workspace_root.join("AP-001/A-01/submission/large.bin"),
        vec![0u8; 8 * 1024 * 1024 + 1],
    )
    .unwrap();
    assert!(seal(&opts).is_err());
    let (_, opts) = fixture();
    for i in 0..129 {
        fs::write(
            opts.workspace_root
                .join(format!("AP-001/A-01/submission/{i}.txt")),
            "x",
        )
        .unwrap();
    }
    assert!(seal(&opts).is_err());
}
