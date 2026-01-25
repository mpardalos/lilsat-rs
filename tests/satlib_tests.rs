mod common;

use libtest_mimic::{Arguments, Trial, Failed};
use lilsat_rs::*;
use std::fs;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

fn run_test_file(path: &Path, expected_category: &str) -> Result<(), Failed> {
    let test_name = path.file_name().unwrap().to_string_lossy();

    // Read and parse CNF file
    let content = fs::read_to_string(path)
        .map_err(|e| Failed::from(format!("Failed to read CNF file: {}", e)))?;
    let formula: Formula = content.parse()
        .map_err(|e| Failed::from(format!("Failed to parse CNF formula: {}", e)))?;

    if formula.0.is_empty() {
        return Err("Formula should not be empty".into());
    }

    // Run solver with 10-second timeout
    let (tx, rx) = mpsc::channel();
    let formula_clone = formula.clone();

    std::thread::spawn(move || {
        let answer = Lilsat::solve(&formula_clone);
        let _ = tx.send(answer);
    });

    let answer = match rx.recv_timeout(Duration::from_secs(10)) {
        Ok(ans) => ans,
        Err(_) => {
            return Err("Timeout (10s exceeded)".into());
        }
    };

    // Check answer matches expected category
    match (&answer, expected_category) {
        (Answer::SAT(valuation), "sat") => {
            // Validate that valuation satisfies the formula
            if eval_formula(valuation, &formula) != Some(true) {
                return Err(format!("SAT answer but valuation doesn't satisfy formula for {}", test_name).into());
            }
            Ok(())
        }
        (Answer::UNSAT, "unsat") => {
            Ok(())
        }
        (Answer::SAT(_), "unsat") => {
            Err(format!("Expected UNSAT but got SAT for {}", test_name).into())
        }
        (Answer::UNSAT, "sat") => {
            Err(format!("Expected SAT but got UNSAT for {}", test_name).into())
        }
        _ => Err(format!("Invalid category: {}", expected_category).into()),
    }
}

fn main() {
    let mut args = Arguments::from_args();

    // Force sequential test execution to prevent timeouts from parallel runs
    // Users can override with --test-threads=N if desired
    if args.test_threads.is_none() {
        args.test_threads = Some(1);
    }

    // Download all test suites first
    println!("Downloading SATLIB benchmarks...");
    common::ensure_all_satlib_suites().expect("Failed to download SATLIB suites");

    // Discover all CNF files
    let all_files = common::find_cnf_files(Path::new(common::TEST_DIR))
        .expect("Failed to discover CNF files");

    // Create individual test cases for each file
    let tests: Vec<Trial> = all_files
        .iter()
        .filter_map(|path| {
            let category = common::categorize_test(path).ok()?;
            let test_name = format!(
                "{}/{}",
                category,
                path.file_name()?.to_string_lossy()
            );

            let path_clone = path.clone();
            let category_clone = category.clone();

            Some(Trial::test(test_name, move || {
                run_test_file(&path_clone, &category_clone)
            }))
        })
        .collect();

    println!("Running {} tests", tests.len());

    libtest_mimic::run(&args, tests).exit();
}
