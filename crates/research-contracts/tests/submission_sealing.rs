use research_contracts::submission::{seal, SealOptions};
use std::fs;

fn temp() -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("seal-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
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
        br#"{"program_id":"AP-001","role_id":"A-01"}"#,
    )
    .unwrap();
    fs::write(workspace.join("answer.md"), "answer\n").unwrap();
    let opts = SealOptions {
        repo,
        program: "AP-001".into(),
        role: "A-01".into(),
        scientific_base_sha: "a".repeat(40),
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
