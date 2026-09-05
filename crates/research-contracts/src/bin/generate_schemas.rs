use research_contracts::{generated_additive_schemas, generated_schemas};
use std::{env, fs, path::PathBuf, process};

fn main() {
    let dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("contracts"));
    let schemas = generated_schemas()
        .into_iter()
        .chain(generated_additive_schemas());
    for (name, schema) in schemas {
        if let Err(error) = fs::write(dir.join(name), schema) {
            eprintln!("failed to write {name}: {error}");
            process::exit(1);
        }
    }
    println!("generated schemas in {}", dir.display());
}
