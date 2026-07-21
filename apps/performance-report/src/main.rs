use echlub_performance::{
    assess_synthetic_observation, pair_live_endpoints, parse_and_validate,
    parse_and_validate_live_endpoint, summarize_run, validate_live_directory,
    validate_live_endpoint, validate_run, PerformanceRunV1,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn usage() {
    eprintln!(
        "Usage: performance-report <command> [args]\n\
Commands:\n\
  validate <file>\n\
  assess-synthetic <file>\n\
  summarize <file>\n\
  verify-directory <dir>\n\
  validate-live-endpoint <file>\n\
  validate-live-draft <file>\n\
  summarize-live-endpoint <file> [--output <dir>]\n\
  pair-live-endpoints <peer-a.json> <peer-b.json> [--output <dir>]\n\
  verify-live-directory <dir>"
    );
    process::exit(1);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
    }

    match args[1].as_str() {
        "validate" => validate_file(require_path(&args, 2)),
        "assess-synthetic" => assess_file(require_path(&args, 2)),
        "summarize" => summarize_file(require_path(&args, 2)),
        "verify-directory" => verify_directory(require_path(&args, 2)),
        "validate-live-endpoint" => validate_live_file(require_path(&args, 2), true),
        "validate-live-draft" => validate_live_file(require_path(&args, 2), false),
        "summarize-live-endpoint" => {
            let path = require_path(&args, 2);
            let out = output_dir(&args, 3)
                .unwrap_or_else(|| path.parent().unwrap_or(Path::new(".")).to_path_buf());
            summarize_live_file(path, &out);
        }
        "pair-live-endpoints" => {
            let peer_a = require_path(&args, 2);
            let peer_b = require_path(&args, 3);
            let out =
                output_dir(&args, 4).unwrap_or_else(|| PathBuf::from("evidence/live-two-peer/out"));
            pair_files(peer_a, peer_b, &out);
        }
        "verify-live-directory" => verify_live_dir(require_path(&args, 2)),
        _ => usage(),
    }
}

fn require_path(args: &[String], idx: usize) -> &Path {
    args.get(idx)
        .map(String::as_str)
        .map(Path::new)
        .unwrap_or_else(|| {
            eprintln!("missing path argument");
            process::exit(1);
        })
}

fn output_dir(args: &[String], start: usize) -> Option<PathBuf> {
    args.iter()
        .skip(start)
        .collect::<Vec<_>>()
        .windows(2)
        .find(|w| w[0] == "--output")
        .map(|w| PathBuf::from(w[1].clone()))
}

fn validate_file(path: &Path) {
    let json = read_file(path);
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

fn assess_file(path: &Path) {
    let json = read_file(path);
    let result = assess_synthetic_observation(&json);
    if result.pass {
        println!("ASSESS PASS: {}", path.display());
    } else {
        eprintln!("ASSESS FAIL: {}", path.display());
        for err in &result.errors {
            eprintln!("  - {err}");
        }
        process::exit(1);
    }
}

fn validate_live_file(path: &Path, require_finalized: bool) {
    let json = read_file(path);
    let result = validate_live_endpoint(&json, require_finalized);
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
    let json = read_file(path);
    let run = parse_and_validate(&json).unwrap_or_else(|r| fail_validation(r.errors));
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

fn summarize_live_file(path: &Path, out_dir: &Path) {
    fs::create_dir_all(out_dir).ok();
    let json = read_file(path);
    let endpoint = parse_and_validate_live_endpoint(&json, false)
        .unwrap_or_else(|r| fail_live_validation(r.errors));
    let summary_path = out_dir.join("summary.json");
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&endpoint.derived.unwrap_or_default()).unwrap(),
    )
    .unwrap();
    println!("Summary written to {}", summary_path.display());
}

fn pair_files(peer_a: &Path, peer_b: &Path, out_dir: &Path) {
    fs::create_dir_all(out_dir).unwrap_or_else(|e| {
        eprintln!("mkdir error: {e}");
        process::exit(1);
    });
    let a_json = read_file(peer_a);
    let b_json = read_file(peer_b);
    let artifacts = pair_live_endpoints(&a_json, &b_json).unwrap_or_else(|r| {
        eprintln!("pair validation failed:");
        for err in r.errors {
            eprintln!("  - {err}");
        }
        process::exit(1);
    });

    fs::write(
        out_dir.join("peer-a.validated.json"),
        serde_json::to_string_pretty(&artifacts.peer_a).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("peer-b.validated.json"),
        serde_json::to_string_pretty(&artifacts.peer_b).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("peer-a.summary.json"),
        serde_json::to_string_pretty(&artifacts.peer_a_summary).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("peer-b.summary.json"),
        serde_json::to_string_pretty(&artifacts.peer_b_summary).unwrap(),
    )
    .unwrap();
    fs::write(
        out_dir.join("pair-summary.json"),
        serde_json::to_string_pretty(&artifacts.pair).unwrap(),
    )
    .unwrap();
    fs::write(out_dir.join("report.md"), &artifacts.report_markdown).unwrap();
    fs::write(
        out_dir.join("artifact-manifest.json"),
        serde_json::to_string_pretty(&artifacts.manifest).unwrap(),
    )
    .unwrap();
    println!("Paired artifacts written to {}", out_dir.display());
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
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
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

fn verify_live_dir(dir: &Path) {
    let result = validate_live_directory(dir);
    if result.valid {
        println!("VALID live directory: {}", dir.display());
    } else {
        for err in result.errors {
            eprintln!("  - {err}");
        }
        process::exit(1);
    }
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("read error: {e}");
        process::exit(1);
    })
}

fn fail_validation(errors: Vec<echlub_performance::ValidationError>) -> ! {
    eprintln!("validation failed:");
    for err in errors {
        eprintln!("  - {err}");
    }
    process::exit(1);
}

fn fail_live_validation(errors: Vec<echlub_performance::LiveValidationError>) -> ! {
    eprintln!("live validation failed:");
    for err in errors {
        eprintln!("  - {err}");
    }
    process::exit(1);
}
