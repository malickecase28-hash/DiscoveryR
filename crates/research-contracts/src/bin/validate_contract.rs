use research_contracts::parse_and_validate_contract;
use std::{env, fs, process};

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: validate_contract <path>");
        process::exit(2)
    };
    match fs::read_to_string(&path)
        .map_err(Into::into)
        .and_then(|json| parse_and_validate_contract(&json))
    {
        Ok(_) => println!("valid experiment contract: {path}"),
        Err(error) => {
            eprintln!("invalid experiment contract: {error}");
            process::exit(1);
        }
    }
}
