use research_contracts::generated_schemas;
use std::{env, fs, path::PathBuf, process};

fn main() {
    let dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("contracts"));
    for (name, schema) in generated_schemas() {
        if let Err(error) = fs::write(dir.join(name), schema) {
            eprintln!("failed to write {name}: {error}");
            process::exit(1);
        }
    }
    println!("generated four schemas in {}", dir.display());
}
