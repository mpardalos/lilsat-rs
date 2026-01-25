use lilsat_rs::*;
use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <cnf-file>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];

    let contents = fs::read_to_string(filename).unwrap_or_else(|err| {
        eprintln!("Error reading file '{}': {}", filename, err);
        std::process::exit(1);
    });

    let formula: Formula = contents.parse().unwrap_or_else(|err| {
        eprintln!("Error parsing CNF: {}", err);
        std::process::exit(1);
    });

    println!("{}", Lilsat::solve(&formula));
}
