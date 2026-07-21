use echlub_performance::{parse_and_validate, summarize_run, validate_run, PerformanceRunV1};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn usage() {
    eprintln!("Usage: performance-report <validate|summarize|verify-directory> [path]");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }

    match args[1].as_str() {
        "validate" => {
            let path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
                eprintln!("validate requires a file path");
                process::exit(1);
            });
            validate_file(&path);
        }
        "summarize" => {
            let path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
                eprintln!("summarize requires a file path");
                process::exit(1);
            });
            summarize_file(&path);
        }
        "verify-directory" => {
            let path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
                eprintln!("verify-directory requires a directory path");
                process::exit(1);
            });
            verify_directory(&path);
        }
        _ => usage(),
    }
}

fn validate_file(path: &Path) {
    let json = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("read error: {e}");
        process::exit(1);
    });
    let result = validate_run(&json);
    if result.valid {
        println!("VALID: {}", path.display());
    } else {
        eprintln!("INVALID: {}", path.display());
        for err in &result.errors {
            eprintln!("  - {err}");
        }
        process::exit(1);
    }
}

fn summarize_file(path: &Path) {
    let json = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("read error: {e}");
        process::exit(1);
    });
    let run = parse_and_validate(&json).unwrap_or_else(|r| {
        eprintln!("validation failed:");
        for err in r.errors {
            eprintln!("  - {err}");
        }
        process::exit(1);
    });
    let run_with_derived = PerformanceRunV1 {
        derived: Some(echlub_performance::compute_derived_metrics(&run)),
        ..run
    };
    let report = summarize_run(&run_with_derived);
    let out_dir = path.parent().unwrap_or(Path::new("."));
    let report_path = out_dir.join("report.md");
    fs::write(&report_path, &report.markdown).unwrap_or_else(|e| {
        eprintln!("write error: {e}");
        process::exit(1);
    });
    println!("Report written to {}", report_path.display());
}

fn verify_directory(dir: &Path) {
    if !dir.is_dir() {
        eprintln!("not a directory: {}", dir.display());
        process::exit(1);
    }
    let mut valid = 0;
    let mut invalid = 0;
    for entry in fs::read_dir(dir).unwrap_or_else(|e| {
        eprintln!("read_dir error: {e}");
        process::exit(1);
    }) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json")
            && path.file_name().and_then(|n| n.to_str()) != Some("report.md")
        {
            let json = fs::read_to_string(&path).unwrap();
            let result = validate_run(&json);
            if result.valid {
                valid += 1;
                println!("VALID: {}", path.display());
            } else {
                invalid += 1;
                eprintln!("INVALID: {}", path.display());
            }
        }
    }
    if invalid > 0 {
        process::exit(1);
    }
    println!("Verified {valid} artifact(s) in {}", dir.display());
}
