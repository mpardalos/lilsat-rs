use std::fs;
use std::path::{Path, PathBuf};

pub const TEST_DIR: &str = "tests/satlib";

pub struct SatlibSuite {
    pub name: &'static str,
    pub category: &'static str,  // "sat" or "unsat"
    pub url: &'static str,
}

pub const SATLIB_SUITES: &[SatlibSuite] = &[
    SatlibSuite {
        name: "flat30-60",
        category: "sat",
        url: "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/GCP/flat30-60.tar.gz",
    },
    SatlibSuite {
        name: "sw100-8-lp0-c5",
        category: "sat",
        url: "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/SW-GCP/sw100-8-lp0-c5.tar.gz",
    },
    SatlibSuite {
        name: "planning",
        category: "sat",
        url: "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/PLANNING/BlocksWorld/blocksworld.tar.gz",
    },
    SatlibSuite {
        name: "uniform-unsat75",
        category: "unsat",
        url: "https://www.cs.ubc.ca/~hoos/SATLIB/Benchmarks/SAT/RND3SAT/uuf75-325.tar.gz",
    },
];

/// Downloads and extracts a SATLIB test suite if not already present
pub fn ensure_satlib_suite(suite: &SatlibSuite) -> std::io::Result<PathBuf> {
    let target_dir = PathBuf::from(TEST_DIR)
        .join(suite.name)
        .join(suite.category);

    if target_dir.exists() {
        println!("Skipping {} (already exists)", suite.name);
        return Ok(target_dir);
    }

    println!("Downloading {} from {}", suite.name, suite.url);
    fs::create_dir_all(&target_dir)?;

    // Download tarball
    let response = reqwest::blocking::get(suite.url)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let tar_gz_bytes = response.bytes()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    // Decompress and extract
    let tar_gz = flate2::read::GzDecoder::new(&tar_gz_bytes[..]);
    let mut archive = tar::Archive::new(tar_gz);

    // Extract directly to target directory, filtering for .cnf files
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;

        if path.extension().and_then(|s| s.to_str()) == Some("cnf") {
            let filename = path.file_name()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "Invalid path"))?;
            let dest = target_dir.join(filename);

            let mut file = fs::File::create(dest)?;
            std::io::copy(&mut entry, &mut file)?;
        }
    }

    println!("Extracted {} CNF files to {:?}", suite.name, target_dir);
    Ok(target_dir)
}

/// Downloads all SATLIB test suites
pub fn ensure_all_satlib_suites() -> std::io::Result<()> {
    for suite in SATLIB_SUITES {
        ensure_satlib_suite(suite)?;
    }
    Ok(())
}

/// Discovers all .cnf files in a directory recursively
pub fn find_cnf_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut cnf_files = Vec::new();

    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().and_then(|s| s.to_str()) == Some("cnf") {
            cnf_files.push(entry.path().to_path_buf());
        }
    }

    Ok(cnf_files)
}

/// Categorizes test file based on parent directory name
pub fn categorize_test(path: &Path) -> Result<String, String> {
    let parent = path.parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("Invalid path: {:?}", path))?;

    if parent == "sat" || parent == "unsat" {
        Ok(parent.to_string())
    } else {
        Err(format!(
            "Test file {:?} is not in a 'sat' or 'unsat' directory (found: {})",
            path, parent
        ))
    }
}
