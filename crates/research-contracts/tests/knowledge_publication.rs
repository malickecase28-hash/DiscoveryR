use research_contracts::submission::publish_atomic_no_replace;
use std::{
    fs,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    thread,
};

fn temp_dir() -> std::path::PathBuf {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    loop {
        let path = std::env::temp_dir().join(format!(
            "knowledge-publication-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        match fs::create_dir(&path) {
            Ok(()) => return path,
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => panic!("create temp directory: {error}"),
        }
    }
}

#[test]
fn concurrent_same_id_publishes_one_complete_file() {
    let dir = temp_dir();
    let dir = Arc::new(dir);
    let mut joins = Vec::new();
    for bytes in [b"first".as_slice(), b"second".as_slice()] {
        let dir = Arc::clone(&dir);
        joins.push(thread::spawn(move || {
            publish_atomic_no_replace(&dir, "K-1.json", bytes).map_err(|error| error.to_string())
        }));
    }
    let results: Vec<_> = joins.into_iter().map(|join| join.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let value = fs::read(dir.join("K-1.json")).unwrap();
    assert!(value == b"first" || value == b"second");
    assert_eq!(fs::read_dir(&*dir).unwrap().count(), 1);
    let _ = fs::remove_dir_all(&*dir);
}

#[test]
fn different_ids_publish_without_partial_canonical_files() {
    let dir = temp_dir();
    publish_atomic_no_replace(&dir, "K-1.json", b"one").unwrap();
    publish_atomic_no_replace(&dir, "K-2.json", b"two").unwrap();
    assert_eq!(fs::read(dir.join("K-1.json")).unwrap(), b"one");
    assert_eq!(fs::read(dir.join("K-2.json")).unwrap(), b"two");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 2);
    let _ = fs::remove_dir_all(dir);
}
