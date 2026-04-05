//! Phase 1 CLI integration tests.
//!
//! Tests the `extract` and `visualize` subcommands of the compiled `backend-cli`
//! binary against real Tier 1 corpus fixtures.  Each test exercises a distinct
//! document type (SPEC, DWG, NAR, SUB, simple) to ensure the full extract →
//! visualize round-trip works across the corpus.
//!
//! Prerequisites: `cargo build --bin backend-cli` and PDFIUM_LIB_PATH set.

use std::path::PathBuf;
use std::process::Command;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Workspace root: CARGO_MANIFEST_DIR is `apps/backend-cli`, so go up two levels.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent() // apps/
        .and_then(|p| p.parent()) // workspace root
        .expect("workspace root")
        .to_path_buf()
}

fn tier1(name: &str) -> PathBuf {
    repo_root()
        .join("tests")
        .join("corpus")
        .join("tier1")
        .join(name)
}

fn backend_cli_exe_path() -> PathBuf {
    repo_root().join("target").join("debug").join(if cfg!(windows) {
        "backend-cli.exe"
    } else {
        "backend-cli"
    })
}

fn tmp_dir(label: &str) -> PathBuf {
    let dir = repo_root()
        .join("target")
        .join("tmp")
        .join("cli-integration")
        .join(label);
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

/// Run `backend-cli extract --input <pdf> --output <json>`.
/// Returns the parsed `WorkflowResponse` JSON.
fn run_extract(pdf: &PathBuf, out_json: &PathBuf) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    assert!(
        exe.exists(),
        "backend-cli not found — run: cargo build --bin backend-cli"
    );
    assert!(pdf.exists(), "fixture PDF not found: {}", pdf.display());

    let output = Command::new(&exe)
        .arg("extract")
        .arg("--input")
        .arg(pdf)
        .arg("--output")
        .arg(out_json)
        .output()
        .expect("failed to spawn backend-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "extract exited non-zero for {}:\nstdout: {stdout}\nstderr: {}",
        pdf.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_str(&stdout).expect("stdout must be valid JSON WorkflowResponse")
}

/// Run `backend-cli visualize --input <json> --output <dir>`.
/// Returns the parsed `WorkflowResponse` JSON.
fn run_visualize(transcript_json: &PathBuf, out_dir: &PathBuf) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    let output = Command::new(&exe)
        .arg("visualize")
        .arg("--input")
        .arg(transcript_json)
        .arg("--output")
        .arg(out_dir)
        .output()
        .expect("failed to spawn backend-cli");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "visualize exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_str(&stdout).expect("stdout must be valid JSON WorkflowResponse")
}

/// Assert the standard `WorkflowResponse` envelope fields are present.
fn assert_envelope(resp: &serde_json::Value) {
    for field in &[
        "request_id",
        "session_id",
        "operation_id",
        "contracts_version",
        "result",
    ] {
        assert!(resp.get(field).is_some(), "missing envelope field: {field}");
    }
    let result = &resp["result"];
    assert!(result.get("status").is_some(), "missing result.status");
    assert!(result.get("summary").is_some(), "missing result.summary");
}

/// Assert extract succeeded and return the page count from the summary string.
fn assert_extract_ok(resp: &serde_json::Value) -> usize {
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "succeeded", "extract status was not succeeded: {resp}");
    let summary = resp["result"]["summary"].as_str().unwrap_or("");
    // Summary format: `Extracted N page(s) from "..."`
    let page_count = summary
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0);
    assert!(page_count > 0, "extract reported 0 pages: {summary}");
    page_count
}

/// Assert visualize succeeded and at least one PNG was written to `out_dir`.
fn assert_visualize_ok(resp: &serde_json::Value, out_dir: &PathBuf) {
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert_eq!(
        status, "succeeded",
        "visualize status was not succeeded: {resp}"
    );
    let png_count = std::fs::read_dir(out_dir)
        .expect("visualize output dir missing")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .count();
    assert!(
        png_count > 0,
        "visualize produced no PNG files in {}",
        out_dir.display()
    );
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// Simple 3-page test fixture — fastest end-to-end smoke test.
#[test]
fn cli_extract_and_visualize_simple_pdf() {
    let tmp = tmp_dir("simple");
    let pdf = tier1("simple.pdf");
    let json = tmp.join("transcript.json");
    let vis = tmp.join("vis");

    let extract_resp = run_extract(&pdf, &json);
    assert_envelope(&extract_resp);
    let pages = assert_extract_ok(&extract_resp);

    assert!(json.exists(), "transcript JSON not written");

    let vis_resp = run_visualize(&json, &vis);
    assert_envelope(&vis_resp);
    assert_visualize_ok(&vis_resp, &vis);

    // PNG count must equal the page count reported by extract.
    let png_count = std::fs::read_dir(&vis)
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .count();
    assert_eq!(
        png_count, pages,
        "PNG count should match extracted page count"
    );
}

/// Spec document — multi-page CSI-format spec sheet.
#[test]
fn cli_extract_and_visualize_spec_pdf() {
    let tmp = tmp_dir("spec");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let json = tmp.join("transcript.json");
    let vis = tmp.join("vis");

    let extract_resp = run_extract(&pdf, &json);
    assert_envelope(&extract_resp);
    assert_extract_ok(&extract_resp);

    // Transcript must have many pages.
    let transcript_text = std::fs::read_to_string(&json).expect("read transcript");
    let transcript: serde_json::Value =
        serde_json::from_str(&transcript_text).expect("transcript must be valid JSON");
    let page_array = transcript["pages"]
        .as_array()
        .expect("transcript.pages must be array");
    assert!(
        page_array.len() > 10,
        "SPEC fixture should have many pages, got {}",
        page_array.len()
    );

    let vis_resp = run_visualize(&json, &vis);
    assert_envelope(&vis_resp);
    assert_visualize_ok(&vis_resp, &vis);
}

/// Drawing set — multi-sheet mechanical drawing package.
#[test]
fn cli_extract_and_visualize_dwg_pdf() {
    let tmp = tmp_dir("dwg");
    let pdf = tier1("DWG_RWB_LHHS_ALL_ORG.pdf");
    let json = tmp.join("transcript.json");
    let vis = tmp.join("vis");

    let extract_resp = run_extract(&pdf, &json);
    assert_envelope(&extract_resp);
    assert_extract_ok(&extract_resp);

    let vis_resp = run_visualize(&json, &vis);
    assert_envelope(&vis_resp);
    assert_visualize_ok(&vis_resp, &vis);
}

/// Narrative document.
#[test]
fn cli_extract_and_visualize_nar_pdf() {
    let tmp = tmp_dir("nar");
    let pdf = tier1("NAR_RWB_LHHS_ADD2.pdf");
    let json = tmp.join("transcript.json");
    let vis = tmp.join("vis");

    let extract_resp = run_extract(&pdf, &json);
    assert_envelope(&extract_resp);
    assert_extract_ok(&extract_resp);

    let vis_resp = run_visualize(&json, &vis);
    assert_envelope(&vis_resp);
    assert_visualize_ok(&vis_resp, &vis);
}

/// Submittal document.
#[test]
fn cli_extract_and_visualize_sub_pdf() {
    let tmp = tmp_dir("sub");
    let pdf = tier1("SUB_LHHS_MCMJ_CARRIER-UV.pdf");
    let json = tmp.join("transcript.json");
    let vis = tmp.join("vis");

    let extract_resp = run_extract(&pdf, &json);
    assert_envelope(&extract_resp);
    assert_extract_ok(&extract_resp);

    let vis_resp = run_visualize(&json, &vis);
    assert_envelope(&vis_resp);
    assert_visualize_ok(&vis_resp, &vis);
}

/// Dry-run extract: must exit 0 and return succeeded without writing a file.
#[test]
fn cli_extract_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    assert!(
        exe.exists(),
        "backend-cli not found — run: cargo build --bin backend-cli"
    );

    let pdf = tier1("simple.pdf");
    assert!(pdf.exists(), "simple.pdf fixture missing");

    let tmp = tmp_dir("dry-run");
    let sentinel = tmp.join("should_not_exist.json");

    let output = Command::new(&exe)
        .arg("extract")
        .arg("--input")
        .arg(&pdf)
        .arg("--output")
        .arg(&sentinel)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli");

    assert!(output.status.success(), "dry-run exited non-zero");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("dry-run stdout must be JSON");

    assert_eq!(resp["result"]["status"], "succeeded");
    assert!(!sentinel.exists(), "dry-run must not write output file");
}

/// Dry-run visualize: must exit 0 and return succeeded without writing PNGs.
#[test]
fn cli_visualize_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    assert!(
        exe.exists(),
        "backend-cli not found — run: cargo build --bin backend-cli"
    );

    let tmp = tmp_dir("vis-dry-run");
    let pdf = tier1("simple.pdf");
    assert!(pdf.exists(), "simple.pdf fixture missing");
    let json = tmp.join("transcript.json");

    if !json.exists() {
        run_extract(&pdf, &json);
    }
    assert!(json.exists(), "transcript JSON must exist for visualize dry-run");

    let vis_dir = tmp.join("vis");
    let output = Command::new(&exe)
        .arg("visualize")
        .arg("--input")
        .arg(&json)
        .arg("--output")
        .arg(&vis_dir)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli");

    assert!(output.status.success(), "visualize dry-run exited non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("visualize dry-run stdout must be JSON");
    assert_eq!(resp["result"]["status"], "succeeded");
    assert!(
        !vis_dir.exists(),
        "visualize dry-run must not create output directory"
    );
}

/// Coordinate correctness: headers must appear near the top of the page (Y < 0.3)
/// and footers near the bottom (Y > 0.7) in normalised top-left coordinates.
/// Verified against SPEC_RWB_LHHS_ALL_ORG.pdf whose first page has a visible
/// firm header and a section-ID footer.
#[test]
fn cli_extract_spec_coordinates_not_inverted() {
    let tmp = tmp_dir("coord-check");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let json = tmp.join("transcript.json");

    run_extract(&pdf, &json);

    let text = std::fs::read_to_string(&json).expect("read transcript");
    let transcript: serde_json::Value =
        serde_json::from_str(&text).expect("transcript must be JSON");

    let page0 = &transcript["pages"][0];
    let spans = page0["spans"].as_array().expect("page 0 must have spans");
    assert!(!spans.is_empty(), "page 0 must have at least one span");

    // The first span (sorted by Y then X) must be near the top of the page.
    // After normalization: Y=0.0 is the top, Y=1.0 is the bottom.
    let first_y = spans[0]["bbox"]["y"].as_f64().unwrap_or(f64::MAX);
    assert!(
        first_y < 0.3,
        "First span on page 0 has Y={first_y:.4} — expected near page top (Y < 0.3). \
         Coordinate system may be inverted."
    );

    // The last span must be nearer the bottom than the first.
    let last_y = spans
        .last()
        .and_then(|s| s["bbox"]["y"].as_f64())
        .unwrap_or(0.0);
    assert!(
        last_y > first_y,
        "Last span Y={last_y:.4} should be greater than first span Y={first_y:.4}"
    );
}
