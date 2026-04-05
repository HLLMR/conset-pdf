//! Phase J integration tests for the `pattern-dev` binary.
//!
//! These tests invoke the compiled `pattern-dev` binary via
//! [`std::process::Command`] to verify:
//!
//! - `--dry-run` mode exits successfully without writing any files.
//! - Out-of-range or holdout tiers are rejected with a non-zero exit code.
//! - The golden-path sidecar JSON output has the required schema fields
//!   (requires PDFium + corpus; marked `#[ignore]`).
//! - Two consecutive `test-pattern` runs on the same fixture produce
//!   byte-identical sidecar files, proving determinism (requires PDFium +
//!   corpus; marked `#[ignore]`).
//!
//! # Running the ignored tests
//!
//! ```powershell
//! $env:PDFIUM_LIB_PATH = "f:\Projects\conset-pdf"
//! cargo test --package classify-pdf -- --include-ignored
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Absolute path to the workspace root.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tools/ must have a parent (workspace root)")
        .to_path_buf()
}

/// Path to the debug `pattern-dev` binary (built by `cargo build --bin pattern-dev`).
fn pattern_dev_exe() -> PathBuf {
    let root = workspace_root();
    let name = if cfg!(windows) { "pattern-dev.exe" } else { "pattern-dev" };
    root.join("target").join("debug").join(name)
}

/// Create (or recreate) a clean scratch directory under `target/test-j/`.
fn scratch_dir(label: &str) -> PathBuf {
    let dir = workspace_root()
        .join("target")
        .join("test-j")
        .join(label);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Count files recursively under `dir`. Returns 0 when `dir` does not exist.
fn count_files(dir: &Path) -> usize {
    walkdir(dir).count()
}

/// Minimal recursive file walk (avoids adding the `walkdir` dep).
fn walkdir(dir: &Path) -> impl Iterator<Item = PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                files.extend(walkdir(&path));
            } else {
                files.push(path);
            }
        }
    }
    files.into_iter()
}

/// Path to the smallest Tier 1 DWG fixture (6 pages) — used in fast tests.
fn small_fixture() -> PathBuf {
    workspace_root()
        .join("tests/corpus/tier1/DWG_RWB_LHHS_ALL_ADD4.pdf")
}

/// Returns `true` when the compiled binary and the small fixture both exist.
fn prerequisites_met() -> bool {
    pattern_dev_exe().exists() && small_fixture().exists()
}

// ── Tests that do NOT require PDFium or corpus fixtures ───────────────────────

/// `--dry-run` must exit 0, print "[dry-run]" to stdout, and write no files.
#[test]
fn dry_run_validate_corpus_exits_zero_and_writes_no_files() {
    let exe = pattern_dev_exe();
    assert!(
        exe.exists(),
        "pattern-dev binary not found at {}\n\
         Run: cargo build --bin pattern-dev",
        exe.display()
    );

    let out_dir = scratch_dir("dry-run-no-write");

    let output = Command::new(&exe)
        .current_dir(workspace_root())
        .args([
            "validate-corpus",
            "--tier", "1",
            "--output-dir", out_dir.to_str().unwrap(),
            "--dry-run",
        ])
        .output()
        .expect("spawn pattern-dev");

    assert!(
        output.status.success(),
        "Expected exit 0 for --dry-run, got {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[dry-run]"),
        "Expected '[dry-run]' in stdout, got:\n{stdout}"
    );

    // No sidecar JJSONs, no manifest — the output dir should be empty.
    assert_eq!(
        count_files(&out_dir),
        0,
        "--dry-run must not write any files; found {} file(s)",
        count_files(&out_dir)
    );
}

/// Tier 3 (holdout-adjacent) must be rejected with a non-zero exit code.
#[test]
fn holdout_adjacent_tier_is_rejected() {
    let exe = pattern_dev_exe();
    assert!(
        exe.exists(),
        "pattern-dev binary not found at {}",
        exe.display()
    );

    let out_dir = scratch_dir("tier-reject");

    let output = Command::new(&exe)
        .current_dir(workspace_root())
        .args([
            "validate-corpus",
            "--tier", "3",
            "--output-dir", out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("spawn pattern-dev");

    assert!(
        !output.status.success(),
        "Expected non-zero exit for --tier 3, got {}\nstdout: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Error message should mention holdout or tier restriction.
    assert!(
        stderr.to_lowercase().contains("tier") || stderr.to_lowercase().contains("holdout"),
        "Expected tier/holdout error in stderr, got:\n{stderr}"
    );
}

/// Calling validate-corpus with no --tier at all must be rejected.
#[test]
fn validate_corpus_with_no_tier_is_rejected() {
    let exe = pattern_dev_exe();
    assert!(exe.exists(), "pattern-dev binary not found at {}", exe.display());

    let out_dir = scratch_dir("no-tier-reject");

    let output = Command::new(&exe)
        .current_dir(workspace_root())
        .args([
            "validate-corpus",
            "--output-dir", out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("spawn pattern-dev");

    assert!(
        !output.status.success(),
        "Expected non-zero exit when no --tier given, got {}",
        output.status
    );
}

// ── Tests that require PDFium + corpus fixtures ───────────────────────────────

/// Golden-path: run `test-pattern` on a small fixture and verify that the
/// sidecar JSON contains all required schema fields.
#[test]
#[ignore = "Requires built pattern-dev binary, PDFium library, and Tier 1 corpus fixtures"]
fn golden_path_sidecar_has_required_schema_fields() {
    assert!(
        prerequisites_met(),
        "binary or fixture not found — skipping"
    );

    let out_dir = scratch_dir("golden-path-sidecar");

    let output = Command::new(pattern_dev_exe())
        .current_dir(workspace_root())
        .env("PDFIUM_LIB_PATH", workspace_root())
        .args([
            "test-pattern",
            "--family", "footer-section-id",
            "--output-dir", out_dir.to_str().unwrap(),
            small_fixture().to_str().unwrap(),
        ])
        .output()
        .expect("spawn pattern-dev");

    assert!(
        output.status.success(),
        "test-pattern failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    // Find at least one sidecar JSON in the output directory.
    let sidecars: Vec<PathBuf> = walkdir(&out_dir)
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();

    assert!(
        !sidecars.is_empty(),
        "No sidecar JSON files found under {}", out_dir.display()
    );

    // Validate the required schema fields on the first sidecar.
    let content = std::fs::read_to_string(&sidecars[0]).expect("read sidecar");
    let v: serde_json::Value = serde_json::from_str(&content).expect("parse sidecar JSON");

    for field in ["schema_version", "pdf_path", "page_index", "family",
                   "matched_spans", "source", "engine_version", "pattern_version"] {
        assert!(
            v.get(field).is_some(),
            "Required field '{}' missing from sidecar: {}",
            field,
            sidecars[0].display()
        );
    }
    assert_eq!(v["schema_version"], "0.5.0", "schema_version mismatch");
    assert_eq!(v["family"], "footer-section-id", "family mismatch");
}

/// Determinism: run `test-pattern` twice on the same fixture; every sidecar
/// file produced by run 2 must be byte-identical to its counterpart from run 1.
#[test]
#[ignore = "Requires built pattern-dev binary, PDFium library, and Tier 1 corpus fixtures"]
fn determinism_two_test_pattern_runs_produce_byte_identical_sidecars() {
    assert!(
        prerequisites_met(),
        "binary or fixture not found — skipping"
    );

    let run1_dir = scratch_dir("determinism-run1");
    let run2_dir = scratch_dir("determinism-run2");

    let run = |out_dir: &Path| {
        Command::new(pattern_dev_exe())
            .current_dir(workspace_root())
            .env("PDFIUM_LIB_PATH", workspace_root())
            .args([
                "test-pattern",
                "--family", "footer-section-id",
                "--output-dir", out_dir.to_str().unwrap(),
                small_fixture().to_str().unwrap(),
            ])
            .output()
            .expect("spawn pattern-dev")
    };

    let r1 = run(&run1_dir);
    assert!(r1.status.success(), "run 1 failed:\n{}", String::from_utf8_lossy(&r1.stderr));

    let r2 = run(&run2_dir);
    assert!(r2.status.success(), "run 2 failed:\n{}", String::from_utf8_lossy(&r2.stderr));

    // Collect sidecar JSON files from run 1, keyed by relative path.
    let run1_sidecars: Vec<PathBuf> = walkdir(&run1_dir)
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();

    assert!(!run1_sidecars.is_empty(), "no sidecars from run 1");

    let mut byte_mismatches = 0usize;
    let mut missing_in_run2 = 0usize;

    for run1_path in &run1_sidecars {
        // Strip run1_dir prefix to get relative path, then join onto run2_dir.
        let rel = run1_path.strip_prefix(&run1_dir).expect("strip prefix");
        let run2_path = run2_dir.join(rel);

        if !run2_path.exists() {
            missing_in_run2 += 1;
            continue;
        }

        let bytes1 = std::fs::read(run1_path).expect("read run1 sidecar");
        let bytes2 = std::fs::read(&run2_path).expect("read run2 sidecar");

        if bytes1 != bytes2 {
            byte_mismatches += 1;
            eprintln!("DETERMINISM DRIFT: {}", rel.display());
        }
    }

    assert_eq!(
        missing_in_run2, 0,
        "{missing_in_run2} sidecar(s) present in run 1 but missing from run 2"
    );
    assert_eq!(
        byte_mismatches, 0,
        "{byte_mismatches} sidecar(s) differ between run 1 and run 2"
    );
}
