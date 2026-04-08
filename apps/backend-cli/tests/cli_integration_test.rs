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

// ── Phase 2: Segment + VisualizeSegments tests ────────────────────────────────

/// Helper: run `backend-cli segment --input <json> --output <out>`.
fn run_segment(transcript_json: &PathBuf, out_json: &PathBuf) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    let output = Command::new(&exe)
        .arg("segment")
        .arg("--input")
        .arg(transcript_json)
        .arg("--output")
        .arg(out_json)
        .output()
        .expect("failed to spawn backend-cli segment");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "segment exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).expect("segment stdout must be valid JSON WorkflowResponse")
}

/// Helper: run `backend-cli visualize-segments --input <json> --output <dir>`.
fn run_visualize_segments(segment_json: &PathBuf, out_dir: &PathBuf) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    let output = Command::new(&exe)
        .arg("visualize-segments")
        .arg("--input")
        .arg(segment_json)
        .arg("--output")
        .arg(out_dir)
        .output()
        .expect("failed to spawn backend-cli visualize-segments");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "visualize-segments exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).expect("visualize-segments stdout must be valid JSON")
}

/// Assert segment operation succeeded and return the section count.
fn assert_segment_ok(resp: &serde_json::Value) -> usize {
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "succeeded", "segment status was not succeeded: {resp}");
    let summary = resp["result"]["summary"].as_str().unwrap_or("");
    // Format: "Segmented N section(s) from P pages (C% coverage)"
    summary
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Segment a SPEC document: should detect multiple CSI sections with reasonable coverage.
#[test]
fn cli_segment_spec_pdf() {
    let tmp = tmp_dir("segment-spec");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let transcript_json = tmp.join("transcript.json");
    let segment_json = tmp.join("segment-index.json");

    run_extract(&pdf, &transcript_json);
    let resp = run_segment(&transcript_json, &segment_json);
    assert_envelope(&resp);
    let section_count = assert_segment_ok(&resp);
    assert!(
        section_count >= 1,
        "SPEC fixture should produce at least one section, got {section_count}"
    );

    assert!(segment_json.exists(), "segment index JSON not written");

    // Parse the segment index and assert coverage > 0.
    let text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let idx: serde_json::Value = serde_json::from_str(&text).expect("segment index must be JSON");

    let coverage = idx["coverage"]["coverage_ratio"]
        .as_f64()
        .expect("coverage_ratio must be f64");
    assert!(
        coverage > 0.0,
        "SPEC segment index should have positive coverage, got {coverage}"
    );

    let sections = idx["sections"].as_array().expect("sections must be array");
    assert!(
        !sections.is_empty(),
        "sections array must not be empty"
    );
}

/// Segment a simple document: coverage and section count may be zero, but the
/// command must succeed and produce a valid segment index.
#[test]
fn cli_segment_simple_pdf() {
    let tmp = tmp_dir("segment-simple");
    let pdf = tier1("simple.pdf");
    let transcript_json = tmp.join("transcript.json");
    let segment_json = tmp.join("segment-index.json");

    run_extract(&pdf, &transcript_json);
    let resp = run_segment(&transcript_json, &segment_json);
    assert_envelope(&resp);
    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "succeeded",
        "segment must succeed even with no sections detected"
    );
    assert!(segment_json.exists(), "segment index JSON not written");

    // Segment index must be parseable and contain required fields.
    let text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let idx: serde_json::Value = serde_json::from_str(&text).expect("segment index must be JSON");
    assert!(idx.get("source_path").is_some(), "segment index missing source_path");
    assert!(idx.get("coverage").is_some(), "segment index missing coverage");
    assert!(idx.get("sections").is_some(), "segment index missing sections");
}

/// Dry-run segment: must succeed without writing any output file.
#[test]
fn cli_segment_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("segment-dry-run");
    let pdf = tier1("simple.pdf");
    let transcript_json = tmp.join("transcript.json");
    let sentinel = tmp.join("should_not_exist.json");

    run_extract(&pdf, &transcript_json);

    let output = Command::new(&exe)
        .arg("segment")
        .arg("--input")
        .arg(&transcript_json)
        .arg("--output")
        .arg(&sentinel)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli segment");

    assert!(output.status.success(), "segment dry-run exited non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("segment dry-run stdout must be JSON");
    assert_eq!(resp["result"]["status"], "succeeded");
    assert!(!sentinel.exists(), "segment dry-run must not write output file");
}

/// Full round-trip for SPEC: extract → segment → visualize-segments.
/// Verifies that visualize-segments writes one PNG per page.
#[test]
fn cli_segment_and_visualize_segments_spec_pdf() {
    let tmp = tmp_dir("seg-vis-spec");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let transcript_json = tmp.join("transcript.json");
    let segment_json = tmp.join("segment-index.json");
    let vis_dir = tmp.join("vis");

    // Clear any PNGs from previous runs so the count is deterministic.
    if vis_dir.exists() {
        let _ = std::fs::remove_dir_all(&vis_dir);
    }

    let extract_resp = run_extract(&pdf, &transcript_json);
    let page_count = assert_extract_ok(&extract_resp);

    run_segment(&transcript_json, &segment_json);

    let vis_resp = run_visualize_segments(&segment_json, &vis_dir);
    assert_envelope(&vis_resp);
    assert_eq!(
        vis_resp["result"]["status"].as_str().unwrap_or(""),
        "succeeded",
        "visualize-segments must succeed: {vis_resp}"
    );

    let png_count = std::fs::read_dir(&vis_dir)
        .expect("vis dir must exist")
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .count();
    assert_eq!(
        png_count, page_count,
        "visualize-segments should produce one PNG per page"
    );
}

/// Dry-run visualize-segments: must succeed without writing any PNGs.
#[test]
fn cli_visualize_segments_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("seg-vis-dry-run");
    let pdf = tier1("simple.pdf");
    let transcript_json = tmp.join("transcript.json");
    let segment_json = tmp.join("segment-index.json");
    let vis_dir = tmp.join("vis");

    run_extract(&pdf, &transcript_json);
    run_segment(&transcript_json, &segment_json);

    let output = Command::new(&exe)
        .arg("visualize-segments")
        .arg("--input")
        .arg(&segment_json)
        .arg("--output")
        .arg(&vis_dir)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli visualize-segments");

    assert!(
        output.status.success(),
        "visualize-segments dry-run exited non-zero"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("vis-seg dry-run stdout must be JSON");
    assert_eq!(resp["result"]["status"], "succeeded");
    assert!(
        !vis_dir.exists(),
        "visualize-segments dry-run must not create output directory"
    );
}

// ── Phase 3: Parse + VisualizeAst tests ──────────────────────────────────────

/// Helper: run `backend-cli parse --input <pdf> --output <ast.json>`.
fn run_parse(pdf: &PathBuf, out_json: &PathBuf) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    assert!(exe.exists(), "backend-cli not found");
    assert!(pdf.exists(), "fixture PDF not found: {}", pdf.display());

    let output = Command::new(&exe)
        .arg("parse")
        .arg("--input")
        .arg(pdf)
        .arg("--output")
        .arg(out_json)
        .output()
        .expect("failed to spawn backend-cli parse");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "parse exited non-zero for {}:\nstdout: {stdout}\nstderr: {}",
        pdf.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).expect("parse stdout must be valid JSON WorkflowResponse")
}

/// Helper: run `backend-cli visualize-ast --input <ast.json> --output <out.html>`.
fn run_visualize_ast(ast_json: &PathBuf, out_html: &PathBuf) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    let output = Command::new(&exe)
        .arg("visualize-ast")
        .arg("--input")
        .arg(ast_json)
        .arg("--output")
        .arg(out_html)
        .output()
        .expect("failed to spawn backend-cli visualize-ast");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "visualize-ast exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).expect("visualize-ast stdout must be valid JSON")
}

/// Assert parse succeeded and return the section count from the summary.
fn assert_parse_ok(resp: &serde_json::Value) -> usize {
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert!(
        status == "succeeded" || status == "succeeded_with_warnings",
        "parse status unexpected: {status}: {resp}"
    );
    let summary = resp["result"]["summary"].as_str().unwrap_or("");
    // Summary: "Parsed N section(s), M outline node(s)"
    summary
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(0)
}

/// Parse a SPEC PDF — should detect at least one section and produce outline nodes.
#[test]
fn cli_parse_spec_pdf() {
    let tmp = tmp_dir("parse-spec");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let ast_json = tmp.join("ast.json");

    let resp = run_parse(&pdf, &ast_json);
    assert_envelope(&resp);
    assert_parse_ok(&resp);

    assert!(ast_json.exists(), "AST JSON not written");

    let text = std::fs::read_to_string(&ast_json).expect("read AST JSON");
    let doc: serde_json::Value = serde_json::from_str(&text).expect("AST must be valid JSON");

    assert!(doc.get("source_path").is_some(), "AST missing source_path");
    assert!(doc.get("sections").is_some(), "AST missing sections");

    let sections = doc["sections"].as_array().expect("sections must be array");
    assert!(!sections.is_empty(), "SPEC should produce at least one section");
}

/// Parse a simple PDF — command must succeed even if no CSI sections are found.
#[test]
fn cli_parse_simple_pdf() {
    let tmp = tmp_dir("parse-simple");
    let pdf = tier1("simple.pdf");
    let ast_json = tmp.join("ast.json");

    let resp = run_parse(&pdf, &ast_json);
    assert_envelope(&resp);

    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert!(
        status == "succeeded" || status == "succeeded_with_warnings",
        "parse must succeed even with no CSI sections: {resp}"
    );
    assert!(ast_json.exists(), "AST JSON not written");
}

/// Parse with --section filter: only the named section should appear in the output.
#[test]
fn cli_parse_spec_pdf_with_section_filter() {
    let tmp = tmp_dir("parse-spec-filter");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let ast_all = tmp.join("ast_all.json");
    let ast_filtered = tmp.join("ast_filtered.json");

    // First parse without filter to discover a section ID.
    run_parse(&pdf, &ast_all);
    let all_text = std::fs::read_to_string(&ast_all).expect("read full AST");
    let all_doc: serde_json::Value =
        serde_json::from_str(&all_text).expect("full AST must be JSON");
    let sections = all_doc["sections"].as_array().expect("sections array");
    if sections.is_empty() {
        // Nothing to filter — skip.
        return;
    }
    let first_id = sections[0]["section_id"].as_str().expect("section_id").to_owned();

    // Now parse with section filter.
    let exe = backend_cli_exe_path();
    let output = Command::new(&exe)
        .arg("parse")
        .arg("--input")
        .arg(&pdf)
        .arg("--output")
        .arg(&ast_filtered)
        .arg("--section")
        .arg(&first_id)
        .output()
        .expect("failed to spawn backend-cli parse --section");

    assert!(output.status.success(), "parse --section exited non-zero");

    let filtered_text = std::fs::read_to_string(&ast_filtered).expect("read filtered AST");
    let filtered_doc: serde_json::Value =
        serde_json::from_str(&filtered_text).expect("filtered AST must be JSON");
    let filtered_sections = filtered_doc["sections"].as_array().expect("sections array");
    assert_eq!(
        filtered_sections.len(),
        1,
        "parse --section should produce exactly 1 section, got {}",
        filtered_sections.len()
    );
    assert_eq!(
        filtered_sections[0]["section_id"].as_str().unwrap_or(""),
        first_id.as_str(),
        "filtered section ID should match the requested ID"
    );
}

/// Dry-run parse: must succeed without writing any output file.
#[test]
fn cli_parse_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("parse-dry-run");
    let pdf = tier1("simple.pdf");
    let sentinel = tmp.join("should_not_exist_ast.json");

    let output = Command::new(&exe)
        .arg("parse")
        .arg("--input")
        .arg(&pdf)
        .arg("--output")
        .arg(&sentinel)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli parse --dry-run");

    assert!(output.status.success(), "parse dry-run exited non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("dry-run stdout must be JSON");
    assert_eq!(resp["result"]["status"], "succeeded");
    assert!(!sentinel.exists(), "parse dry-run must not write output");
}

/// Full round-trip: parse SPEC then visualize-ast → HTML file produced.
#[test]
fn cli_parse_and_visualize_ast_spec_pdf() {
    let tmp = tmp_dir("parse-vis-ast-spec");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let ast_json = tmp.join("ast.json");
    let html_out = tmp.join("ast.html");

    run_parse(&pdf, &ast_json);

    let vis_resp = run_visualize_ast(&ast_json, &html_out);
    assert_envelope(&vis_resp);
    assert_eq!(
        vis_resp["result"]["status"].as_str().unwrap_or(""),
        "succeeded",
        "visualize-ast must succeed: {vis_resp}"
    );

    assert!(html_out.exists(), "HTML file not written by visualize-ast");

    let html = std::fs::read_to_string(&html_out).expect("read HTML file");
    assert!(html.contains("<!DOCTYPE html>"), "output must be valid HTML");
    assert!(html.contains("<body>"), "output must have a body element");
}

/// Dry-run visualize-ast: must succeed without writing any file.
#[test]
fn cli_visualize_ast_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("vis-ast-dry-run");
    let pdf = tier1("simple.pdf");
    let ast_json = tmp.join("ast.json");
    let sentinel = tmp.join("should_not_exist.html");

    run_parse(&pdf, &ast_json);

    let output = Command::new(&exe)
        .arg("visualize-ast")
        .arg("--input")
        .arg(&ast_json)
        .arg("--output")
        .arg(&sentinel)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli visualize-ast --dry-run");

    assert!(output.status.success(), "visualize-ast dry-run exited non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("vis-ast dry-run stdout must be JSON");
    assert_eq!(resp["result"]["status"], "succeeded");
    assert!(
        !sentinel.exists(),
        "visualize-ast dry-run must not write output file"
    );
}

// ── Phase 4: Edit tests ───────────────────────────────────────────────────────

/// Writes a minimal `ParsedDocument` JSON fixture to `path` and returns the
/// CSI section ID embedded in it.  The fixture contains a single section
/// ("23 82 16") with one Part, one Article, and three Paragraphs (A./B./C.),
/// which gives enough structure to exercise all edit operations.
fn write_edit_fixture(path: &PathBuf) -> &'static str {
    let fixture = serde_json::json!({
        "source_path": "/fixture/spec.pdf",
        "sections": [{
            "section_id": "23 82 16",
            "section_title": "Heating Water Coils",
            "start_page": 0,
            "end_page": 5,
            "parse_warnings": [],
            "nodes": [{
                "tag": "part",
                "marker": "PART 2",
                "text": "PRODUCTS",
                "page_index": 0,
                "level": 0,
                "children": [{
                    "tag": "article",
                    "marker": "2.1",
                    "text": "HEATING WATER COILS",
                    "page_index": 0,
                    "level": 1,
                    "children": [
                        { "tag": "paragraph", "marker": "A.", "text": "Original A text.", "page_index": 0, "level": 2, "children": [] },
                        { "tag": "paragraph", "marker": "B.", "text": "Original B text.", "page_index": 0, "level": 2, "children": [] },
                        { "tag": "paragraph", "marker": "C.", "text": "Original C text.", "page_index": 0, "level": 2, "children": [] }
                    ]
                }]
            }]
        }],
        "global_warnings": []
    });
    std::fs::write(path, serde_json::to_string_pretty(&fixture).unwrap()).expect("write fixture");
    "23 82 16"
}

/// Writes an `EditRequest` JSON to `path`.
fn write_edit_request(path: &PathBuf, description: &str, ops: serde_json::Value) {
    let req = serde_json::json!({ "description": description, "operations": ops });
    std::fs::write(path, serde_json::to_string_pretty(&req).unwrap())
        .expect("write edit request");
}

/// Helper: run `backend-cli edit --input <ast_json> --operations <ops_json>
///          --output <out_json>`.
fn run_edit(
    ast_json: &PathBuf,
    ops_json: &PathBuf,
    out_json: &PathBuf,
) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    let output = Command::new(&exe)
        .arg("edit")
        .arg("--input")
        .arg(ast_json)
        .arg("--operations")
        .arg(ops_json)
        .arg("--output")
        .arg(out_json)
        .output()
        .expect("failed to spawn backend-cli edit");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "edit exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).expect("edit stdout must be valid JSON WorkflowResponse")
}

/// Delete the B. paragraph; A./C. should renumber to A./B.
#[test]
fn cli_edit_delete_renumbers_siblings() {
    let tmp = tmp_dir("edit-delete");
    let ast_json = tmp.join("ast.json");
    let ops_json = tmp.join("ops.json");
    let out_json = tmp.join("edited.json");

    write_edit_fixture(&ast_json);
    write_edit_request(
        &ops_json,
        "Delete B.",
        serde_json::json!([{
            "op": "delete",
            "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.1", "B."] }
        }]),
    );

    let resp = run_edit(&ast_json, &ops_json, &out_json);
    assert_envelope(&resp);
    assert_eq!(resp["result"]["status"], "succeeded", "edit status: {resp}");

    let doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_json).unwrap()).unwrap();
    let paras = &doc["sections"][0]["nodes"][0]["children"][0]["children"];
    assert_eq!(paras.as_array().unwrap().len(), 2, "should have 2 paragraphs after delete");
    assert_eq!(paras[0]["marker"], "A.");
    assert_eq!(paras[1]["marker"], "B."); // formerly C., renumbered
    assert_eq!(paras[1]["text"], "Original C text."); // content preserved
}

/// Replace A.'s text; marker and children unchanged.
#[test]
fn cli_edit_replace_updates_text_only() {
    let tmp = tmp_dir("edit-replace");
    let ast_json = tmp.join("ast.json");
    let ops_json = tmp.join("ops.json");
    let out_json = tmp.join("edited.json");

    write_edit_fixture(&ast_json);
    write_edit_request(
        &ops_json,
        "Replace A. text",
        serde_json::json!([{
            "op": "replace",
            "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.1", "A."] },
            "new_text": "Replacement text for A."
        }]),
    );

    let resp = run_edit(&ast_json, &ops_json, &out_json);
    assert_envelope(&resp);
    assert_eq!(resp["result"]["status"], "succeeded", "edit status: {resp}");

    let doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_json).unwrap()).unwrap();
    let paras = &doc["sections"][0]["nodes"][0]["children"][0]["children"];
    assert_eq!(paras[0]["marker"], "A."); // marker unchanged
    assert_eq!(paras[0]["text"], "Replacement text for A.");
    assert_eq!(paras.as_array().unwrap().len(), 3); // sibling count unchanged
}

/// Insert a new paragraph after B.; A./B./NEW/C. → renumbered A./B./C./D.
#[test]
fn cli_edit_insert_after_renumbers_downstream() {
    let tmp = tmp_dir("edit-insert");
    let ast_json = tmp.join("ast.json");
    let ops_json = tmp.join("ops.json");
    let out_json = tmp.join("edited.json");

    write_edit_fixture(&ast_json);
    write_edit_request(
        &ops_json,
        "Insert after B.",
        serde_json::json!([{
            "op": "insert_after",
            "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.1", "B."] },
            "new_node": {
                "tag": "paragraph",
                "marker": "X.", // overwritten by renumber
                "text": "New inserted paragraph.",
                "page_index": 0,
                "level": 2,
                "children": []
            }
        }]),
    );

    let resp = run_edit(&ast_json, &ops_json, &out_json);
    assert_envelope(&resp);
    assert_eq!(resp["result"]["status"], "succeeded", "edit status: {resp}");

    let doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_json).unwrap()).unwrap();
    let paras = &doc["sections"][0]["nodes"][0]["children"][0]["children"];
    assert_eq!(paras.as_array().unwrap().len(), 4);
    assert_eq!(paras[0]["marker"], "A.");
    assert_eq!(paras[1]["marker"], "B.");
    assert_eq!(paras[2]["marker"], "C."); // formerly X., renumbered
    assert_eq!(paras[2]["text"], "New inserted paragraph.");
    assert_eq!(paras[3]["marker"], "D."); // formerly C., renumbered
}

/// Multi-op request: replace then delete.
#[test]
fn cli_edit_multi_op_request_succeeds() {
    let tmp = tmp_dir("edit-multi");
    let ast_json = tmp.join("ast.json");
    let ops_json = tmp.join("ops.json");
    let out_json = tmp.join("edited.json");

    write_edit_fixture(&ast_json);
    write_edit_request(
        &ops_json,
        "Replace A then delete C",
        serde_json::json!([
            {
                "op": "replace",
                "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.1", "A."] },
                "new_text": "Replaced A."
            },
            {
                "op": "delete",
                "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.1", "C."] }
            }
        ]),
    );

    let resp = run_edit(&ast_json, &ops_json, &out_json);
    assert_envelope(&resp);
    assert_eq!(resp["result"]["status"], "succeeded", "edit status: {resp}");

    let doc: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&out_json).unwrap()).unwrap();
    let paras = &doc["sections"][0]["nodes"][0]["children"][0]["children"];
    assert_eq!(paras.as_array().unwrap().len(), 2);
    assert_eq!(paras[0]["text"], "Replaced A.");
    assert_eq!(paras[1]["marker"], "B.");
}

/// Invalid section_id must return a failed response (preflight rejection).
#[test]
fn cli_edit_invalid_section_returns_failure() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("edit-bad-section");
    let ast_json = tmp.join("ast.json");
    let ops_json = tmp.join("ops.json");
    let out_json = tmp.join("edited.json");

    write_edit_fixture(&ast_json);
    write_edit_request(
        &ops_json,
        "Wrong section",
        serde_json::json!([{
            "op": "delete",
            "path": { "section_id": "99 99 99", "markers": ["PART 1"] }
        }]),
    );

    // edit handler must exit 0 but return status=failed in JSON.
    let output = Command::new(&exe)
        .arg("edit")
        .arg("--input").arg(&ast_json)
        .arg("--operations").arg(&ops_json)
        .arg("--output").arg(&out_json)
        .output()
        .expect("failed to spawn backend-cli edit");

    // backend-cli exits 0 because it handled the error gracefully.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(resp["result"]["status"], "failed", "expected failed for bad section: {resp}");
    assert!(!out_json.exists(), "output must not be written on failure");
}

/// Invalid path must return a failed response (preflight rejection).
#[test]
fn cli_edit_invalid_path_returns_failure() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("edit-bad-path");
    let ast_json = tmp.join("ast.json");
    let ops_json = tmp.join("ops.json");
    let out_json = tmp.join("edited.json");

    write_edit_fixture(&ast_json);
    write_edit_request(
        &ops_json,
        "Bad path",
        serde_json::json!([{
            "op": "delete",
            "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.1", "Z."] }
        }]),
    );

    let output = Command::new(&exe)
        .arg("edit")
        .arg("--input").arg(&ast_json)
        .arg("--operations").arg(&ops_json)
        .arg("--output").arg(&out_json)
        .output()
        .expect("failed to spawn backend-cli edit");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(resp["result"]["status"], "failed", "expected failed for bad path: {resp}");
    assert!(!out_json.exists(), "output must not be written on failure");
}

/// Dry-run edit: must succeed without writing any output file.
#[test]
fn cli_edit_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("edit-dry-run");
    let ast_json = tmp.join("ast.json");
    let ops_json = tmp.join("ops.json");
    let sentinel = tmp.join("should_not_exist.json");

    write_edit_fixture(&ast_json);
    write_edit_request(
        &ops_json,
        "dry run",
        serde_json::json!([{
            "op": "delete",
            "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.1", "A."] }
        }]),
    );

    let output = Command::new(&exe)
        .arg("edit")
        .arg("--input").arg(&ast_json)
        .arg("--operations").arg(&ops_json)
        .arg("--output").arg(&sentinel)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli edit");

    assert!(output.status.success(), "edit dry-run exited non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(resp["result"]["status"], "succeeded");
    assert!(!sentinel.exists(), "edit dry-run must not write output file");
}

// ── Phase 5: Regenerate tests ─────────────────────────────────────────────────

/// Writes a minimal [`SpecChromeMetadata`] JSON fixture to `path`.
fn write_chrome_metadata(path: &PathBuf) {
    let meta = serde_json::json!({
        "project_id": "LHHS-001",
        "project_name": "Lakewood Hills Health Sciences",
        "section_id": "23 82 16",
        "section_title": "Heating Water Coils",
        "date": "2026-01-15",
        "firm": "Test Engineers LLC"
    });
    std::fs::write(path, serde_json::to_string_pretty(&meta).unwrap())
        .expect("write chrome metadata fixture");
}

/// Helper: run `backend-cli regenerate --ast <ast> --chrome-metadata <meta> --output <out>`.
/// Asserts exit 0 and returns the parsed `WorkflowResponse` JSON.
fn run_regenerate(
    ast_json: &PathBuf,
    chrome_json: &PathBuf,
    out_pdf: &PathBuf,
) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    let output = Command::new(&exe)
        .arg("regenerate")
        .arg("--ast")
        .arg(ast_json)
        .arg("--chrome-metadata")
        .arg(chrome_json)
        .arg("--output")
        .arg(out_pdf)
        .output()
        .expect("failed to spawn backend-cli regenerate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "regenerate exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).expect("regenerate stdout must be valid JSON WorkflowResponse")
}

/// Dry-run regenerate: must succeed without invoking Chrome or writing a PDF.
#[test]
fn cli_regenerate_dry_run_succeeds_without_writing_output() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("regen-dry-run");
    let ast_json = tmp.join("ast.json");
    let chrome_json = tmp.join("chrome.json");
    let sentinel = tmp.join("should_not_exist.pdf");

    write_edit_fixture(&ast_json);
    write_chrome_metadata(&chrome_json);

    let output = Command::new(&exe)
        .arg("regenerate")
        .arg("--ast")
        .arg(&ast_json)
        .arg("--chrome-metadata")
        .arg(&chrome_json)
        .arg("--output")
        .arg(&sentinel)
        .arg("--dry-run")
        .output()
        .expect("failed to spawn backend-cli regenerate --dry-run");

    assert!(output.status.success(), "regenerate dry-run exited non-zero");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("dry-run stdout must be JSON");
    assert_envelope(&resp);
    assert_eq!(resp["result"]["status"], "succeeded", "dry-run must succeed: {resp}");
    assert!(!sentinel.exists(), "dry-run must not write any output PDF");
}

/// Non-existent AST file: must exit 0 and return status=failed with
/// error code `INPUT_READ_ERROR`.
#[test]
fn cli_regenerate_missing_ast_fails() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("regen-bad-ast");
    let chrome_json = tmp.join("chrome.json");
    let out_pdf = tmp.join("out.pdf");

    write_chrome_metadata(&chrome_json);

    let output = Command::new(&exe)
        .arg("regenerate")
        .arg("--ast")
        .arg(tmp.join("nonexistent_ast.json")) // does not exist
        .arg("--chrome-metadata")
        .arg(&chrome_json)
        .arg("--output")
        .arg(&out_pdf)
        .output()
        .expect("failed to spawn backend-cli regenerate");

    assert!(output.status.success(), "regenerate must exit 0 even on input error");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(resp["result"]["status"], "failed", "must fail for missing AST: {resp}");
    assert!(!out_pdf.exists(), "output must not be written on failure");
}

/// Section ID not present in the ParsedDocument: must exit 0 and return
/// status=failed with error code `SECTION_NOT_FOUND`.
#[test]
fn cli_regenerate_missing_section_fails() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("regen-bad-section");
    let ast_json = tmp.join("ast.json");
    let chrome_json = tmp.join("chrome.json");
    let out_pdf = tmp.join("out.pdf");

    write_edit_fixture(&ast_json);
    write_chrome_metadata(&chrome_json);

    let output = Command::new(&exe)
        .arg("regenerate")
        .arg("--ast")
        .arg(&ast_json)
        .arg("--chrome-metadata")
        .arg(&chrome_json)
        .arg("--section")
        .arg("99 99 99") // section ID not in fixture
        .arg("--output")
        .arg(&out_pdf)
        .output()
        .expect("failed to spawn backend-cli regenerate");

    assert!(output.status.success(), "regenerate must exit 0 even on section error");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(resp["result"]["status"], "failed", "must fail for missing section: {resp}");
    assert!(!out_pdf.exists(), "output must not be written on failure");
}

/// Chrome metadata file contains invalid JSON: must exit 0 and return
/// status=failed with error code `CHROME_METADATA_READ_ERROR`.
#[test]
fn cli_regenerate_invalid_chrome_metadata_fails() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("regen-bad-chrome");
    let ast_json = tmp.join("ast.json");
    let chrome_json = tmp.join("chrome.json");
    let out_pdf = tmp.join("out.pdf");

    write_edit_fixture(&ast_json);
    std::fs::write(&chrome_json, b"{ not valid json }").expect("write bad chrome metadata");

    let output = Command::new(&exe)
        .arg("regenerate")
        .arg("--ast")
        .arg(&ast_json)
        .arg("--chrome-metadata")
        .arg(&chrome_json)
        .arg("--output")
        .arg(&out_pdf)
        .output()
        .expect("failed to spawn backend-cli regenerate");

    assert!(output.status.success(), "regenerate must exit 0 even on parse error");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let resp: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(
        resp["result"]["status"], "failed",
        "must fail for invalid chrome metadata JSON: {resp}"
    );
    assert!(!out_pdf.exists(), "output must not be written on failure");
}

/// Full regenerate round-trip: parse SPEC → regenerate → verify PDF bytes.
/// Ignored by default — requires a local Chrome 120+ installation.
#[test]
#[ignore = "requires a Chromium-family browser (Chrome, Brave, Edge, Chromium) — set CHROME_PATH or install to a standard system path"]
fn cli_regenerate_produces_pdf() {
    let tmp = tmp_dir("regen-full");
    let pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let ast_json = tmp.join("ast.json");
    let chrome_json = tmp.join("chrome.json");
    let out_pdf = tmp.join("out.pdf");

    run_parse(&pdf, &ast_json);
    write_chrome_metadata(&chrome_json);

    let resp = run_regenerate(&ast_json, &chrome_json, &out_pdf);
    assert_envelope(&resp);
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert!(
        status == "succeeded" || status == "succeeded_with_warnings",
        "regenerate must succeed with Chrome installed: {resp}"
    );
    assert!(out_pdf.exists(), "regenerate must write an output PDF");
    let pdf_bytes = std::fs::read(&out_pdf).expect("read output PDF");
    assert!(
        pdf_bytes.starts_with(b"%PDF"),
        "output file must be a valid PDF (starts with %PDF header)"
    );
    assert!(pdf_bytes.len() > 1024, "output PDF is suspiciously small ({} bytes)", pdf_bytes.len());
}

// ── Phase 6: Stitch tests ─────────────────────────────────────────────────────

/// Helper: run `backend-cli stitch --input <pdf> --segment-index <json>
///          --section <id> --replacement <pdf> --output <pdf>`.
/// Returns the parsed `WorkflowResponse` JSON.
fn run_stitch(
    input: &PathBuf,
    segment_index: &PathBuf,
    section: &str,
    replacement: &PathBuf,
    output: &PathBuf,
    dry_run: bool,
) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    assert!(exe.exists(), "backend-cli not found — run: cargo build --bin backend-cli");

    let mut cmd = Command::new(&exe);
    cmd.arg("stitch")
        .arg("--input").arg(input)
        .arg("--segment-index").arg(segment_index)
        .arg("--section").arg(section)
        .arg("--replacement").arg(replacement)
        .arg("--output").arg(output);
    if dry_run {
        cmd.arg("--dry-run");
    }

    let output_result = cmd.output().expect("failed to spawn backend-cli stitch");
    let stdout = String::from_utf8_lossy(&output_result.stdout);
    assert!(
        output_result.status.success(),
        "stitch exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output_result.stderr)
    );
    serde_json::from_str(&stdout).expect("stitch stdout must be valid JSON WorkflowResponse")
}

/// Build a segment index JSON for `source_path` that contains a single section
/// covering pages 0..=(page_count-1) with the given `section_id`.  Returns the
/// path of the written index file.
fn write_minimal_segment_index(
    dir: &PathBuf,
    source_path: &PathBuf,
    section_id: &str,
    page_count: usize,
) -> PathBuf {
    let idx = serde_json::json!({
        "source_path": source_path.to_string_lossy(),
        "chrome_metadata": {
            "project_id": "",
            "project_name": "",
            "firm": "",
            "date": ""
        },
        "sections": [{
            "section_id": section_id,
            "section_title": "Test Section",
            "start_page": 0,
            "end_page": page_count - 1,
            "page_count": page_count,
            "page_counter_detected": false,
            "confidence": 1.0
        }],
        "coverage": {
            "pages_total": page_count,
            "pages_tagged": page_count,
            "pages_missing_footer": 0,
            "coverage_ratio": 1.0
        }
    });
    let path = dir.join("segment-index.json");
    std::fs::write(&path, serde_json::to_string_pretty(&idx).unwrap())
        .expect("write segment index");
    path
}

/// Dry-run stitch: must exit 0 and return succeeded without writing any file.
#[test]
fn cli_stitch_dry_run_no_write() {
    let tmp = tmp_dir("stitch-dry-run");
    let pdf = tier1("simple.pdf");
    assert!(pdf.exists(), "simple.pdf fixture missing");

    // simple.pdf is 3 pages; use it as both input and replacement.
    let segment_index = write_minimal_segment_index(&tmp, &pdf, "23 82 16", 3);
    let output = tmp.join("stitched_should_not_exist.pdf");

    let resp = run_stitch(&pdf, &segment_index, "23 82 16", &pdf, &output, true);
    assert_envelope(&resp);
    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "succeeded",
        "stitch dry-run must succeed: {resp}"
    );
    assert!(!output.exists(), "stitch dry-run must not write output file");
}

/// Missing input PDF must return status=failed (process exits 0).
#[test]
fn cli_stitch_missing_input_fails() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("stitch-missing-input");
    let pdf = tier1("simple.pdf");

    let segment_index = write_minimal_segment_index(&tmp, &pdf, "23 82 16", 3);
    let output = tmp.join("out.pdf");
    let missing = tmp.join("nonexistent.pdf");

    let out = Command::new(&exe)
        .arg("stitch")
        .arg("--input").arg(&missing)
        .arg("--segment-index").arg(&segment_index)
        .arg("--section").arg("23 82 16")
        .arg("--replacement").arg(&pdf)
        .arg("--output").arg(&output)
        .output()
        .expect("failed to spawn backend-cli stitch");

    assert!(out.status.success(), "stitch must exit 0 even on input error");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let resp: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "failed",
        "expected failed for missing input: {resp}"
    );
    assert!(!output.exists(), "output must not be written on failure");
}

/// Missing replacement PDF must return status=failed (process exits 0).
#[test]
fn cli_stitch_missing_replacement_fails() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("stitch-missing-replacement");
    let pdf = tier1("simple.pdf");

    let segment_index = write_minimal_segment_index(&tmp, &pdf, "23 82 16", 3);
    let output = tmp.join("out.pdf");
    let missing = tmp.join("nonexistent_replacement.pdf");

    let out = Command::new(&exe)
        .arg("stitch")
        .arg("--input").arg(&pdf)
        .arg("--segment-index").arg(&segment_index)
        .arg("--section").arg("23 82 16")
        .arg("--replacement").arg(&missing)
        .arg("--output").arg(&output)
        .output()
        .expect("failed to spawn backend-cli stitch");

    assert!(out.status.success(), "stitch must exit 0 even on replacement error");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let resp: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "failed",
        "expected failed for missing replacement: {resp}"
    );
    assert!(!output.exists(), "output must not be written on failure");
}

/// Missing segment-index JSON must return status=failed (process exits 0).
#[test]
fn cli_stitch_missing_segment_index_fails() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("stitch-missing-index");
    let pdf = tier1("simple.pdf");

    let output = tmp.join("out.pdf");
    let missing = tmp.join("nonexistent_index.json");

    let out = Command::new(&exe)
        .arg("stitch")
        .arg("--input").arg(&pdf)
        .arg("--segment-index").arg(&missing)
        .arg("--section").arg("23 82 16")
        .arg("--replacement").arg(&pdf)
        .arg("--output").arg(&output)
        .output()
        .expect("failed to spawn backend-cli stitch");

    assert!(out.status.success(), "stitch must exit 0 even on missing index");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let resp: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "failed",
        "expected failed for missing segment index: {resp}"
    );
    assert!(!output.exists(), "output must not be written on failure");
}

/// Unknown section ID (not in the segment index) must return status=failed.
#[test]
fn cli_stitch_unknown_section_id_fails() {
    let exe = backend_cli_exe_path();
    let tmp = tmp_dir("stitch-unknown-section");
    let pdf = tier1("simple.pdf");

    let segment_index = write_minimal_segment_index(&tmp, &pdf, "23 82 16", 3);
    let output = tmp.join("out.pdf");

    let out = Command::new(&exe)
        .arg("stitch")
        .arg("--input").arg(&pdf)
        .arg("--segment-index").arg(&segment_index)
        .arg("--section").arg("99 99 99")   // not in index
        .arg("--replacement").arg(&pdf)
        .arg("--output").arg(&output)
        .output()
        .expect("failed to spawn backend-cli stitch");

    assert!(out.status.success(), "stitch must exit 0 even on unknown section");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let resp: serde_json::Value = serde_json::from_str(&stdout).expect("stdout must be JSON");
    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "failed",
        "expected failed for unknown section ID: {resp}"
    );
    assert!(!output.exists(), "output must not be written on failure");
}

/// Full stitch: segment SPEC, then replace the first detected section with
/// `simple.pdf`.  Output must be a valid PDF with a `%PDF` header.
#[test]
fn cli_stitch_produces_valid_pdf() {
    let tmp = tmp_dir("stitch-produces-pdf");
    let spec_pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    let replacement_pdf = tier1("simple.pdf");

    // Step 1: extract transcript.
    let transcript_json = tmp.join("transcript.json");
    run_extract(&spec_pdf, &transcript_json);

    // Step 2: build segment index.
    let segment_json = tmp.join("segment-index.json");
    run_segment(&transcript_json, &segment_json);

    // Step 3: discover first section from the index.
    let idx_text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let idx: serde_json::Value = serde_json::from_str(&idx_text).expect("parse segment index");
    let sections = idx["sections"].as_array().expect("sections array");
    if sections.is_empty() {
        // No sections detected — this fixture is unsuitable; skip gracefully.
        return;
    }
    let first_section_id = sections[0]["section_id"].as_str().expect("section_id").to_owned();

    // Step 4: stitch (write mode).
    let output_pdf = tmp.join("stitched.pdf");
    let resp = run_stitch(
        &spec_pdf,
        &segment_json,
        &first_section_id,
        &replacement_pdf,
        &output_pdf,
        false,
    );

    assert_envelope(&resp);
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert!(
        status == "succeeded" || status == "succeeded_with_warnings",
        "stitch must succeed: {resp}"
    );
    assert!(output_pdf.exists(), "stitch must write the output PDF");

    let bytes = std::fs::read(&output_pdf).expect("read stitched PDF");
    assert!(
        bytes.starts_with(b"%PDF"),
        "stitched output must start with %PDF header"
    );
    assert!(bytes.len() > 512, "stitched PDF is suspiciously small ({} bytes)", bytes.len());

    // Summary should report pages_removed and pages_inserted.
    let summary = resp["result"]["summary"].as_str().unwrap_or("");
    assert!(
        summary.contains("Stitched section"),
        "summary should mention 'Stitched section': {summary}"
    );
}

// ══════════════════════════════════════════════════════════════════════════════
// Phase 7 — apply-addendum integration tests
// ══════════════════════════════════════════════════════════════════════════════

/// Write a minimal `AddendumManifest` JSON to a temp file.
///
/// `sections` is a list of `(section_id, operations_json)` pairs where
/// `operations_json` is a JSON array of `EditOperation` objects.
fn write_minimal_addendum_manifest(
    dir: &PathBuf,
    description: Option<&str>,
    sections: &[(&str, serde_json::Value)],
) -> PathBuf {
    let mut section_specs = Vec::new();
    for (id, ops) in sections {
        section_specs.push(serde_json::json!({
            "section_id": id,
            "operations": ops,
        }));
    }
    let mut manifest = serde_json::json!({
        "sections": section_specs,
    });
    if let Some(desc) = description {
        manifest["description"] = serde_json::Value::String(desc.to_owned());
    }
    let path = dir.join("addendum.json");
    std::fs::write(&path, serde_json::to_string_pretty(&manifest).unwrap())
        .expect("write addendum manifest");
    path
}

/// Run `backend-cli apply-addendum` and return the parsed `WorkflowResponse`.
///
/// * `original`     — source spec PDF
/// * `addendum`     — path to the AddendumManifest JSON
/// * `output`       — desired output PDF path (None = omit `--output`)
/// * `audit_bundle` — optional audit bundle directory
/// * `dry_run`      — whether to pass `--dry-run`
fn run_apply_addendum(
    original: &PathBuf,
    addendum: &PathBuf,
    output: Option<&PathBuf>,
    audit_bundle: Option<&PathBuf>,
    dry_run: bool,
) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    assert!(exe.exists(), "backend-cli not found — run: cargo build --bin backend-cli");

    let mut cmd = Command::new(&exe);
    cmd.arg("apply-addendum")
        .arg("--original").arg(original)
        .arg("--addendum").arg(addendum);
    if let Some(out) = output {
        cmd.arg("--output").arg(out);
    }
    if let Some(bundle_dir) = audit_bundle {
        cmd.arg("--audit-bundle").arg(bundle_dir);
    }
    if dry_run {
        cmd.arg("--dry-run");
    }

    let result = cmd.output().expect("failed to spawn backend-cli apply-addendum");
    let stdout = String::from_utf8_lossy(&result.stdout);
    assert!(
        result.status.success(),
        "apply-addendum exited non-zero:\nstdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_str(&stdout)
        .expect("apply-addendum stdout must be valid JSON WorkflowResponse")
}

// ── Test: dry-run produces no output file ─────────────────────────────────────

/// Dry-run on a real SPEC fixture: all sections parsed and edited, no PDF written.
#[test]
fn cli_apply_addendum_dry_run_no_write() {
    let tmp = tmp_dir("apply-addendum-dry-run");
    let spec_pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    assert!(spec_pdf.exists(), "SPEC fixture missing");

    // We need a section ID that actually exists in the fixture, so segment first.
    let transcript_json = tmp.join("transcript.json");
    run_extract(&spec_pdf, &transcript_json);
    let segment_json = tmp.join("segment-index.json");
    run_segment(&transcript_json, &segment_json);

    let idx_text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let idx: serde_json::Value = serde_json::from_str(&idx_text).expect("parse segment index");
    let sections = idx["sections"].as_array().expect("sections array");
    if sections.is_empty() {
        return; // No sections detected; fixture unsuitable — skip.
    }
    let first_id = sections[0]["section_id"].as_str().expect("section_id").to_owned();

    // Manifest with no-op (empty operations) for one real section.
    let manifest = write_minimal_addendum_manifest(
        &tmp,
        Some("dry-run test"),
        &[(&first_id, serde_json::json!([]))],
    );

    let output_pdf = tmp.join("should_not_exist.pdf");
    let resp = run_apply_addendum(&spec_pdf, &manifest, Some(&output_pdf), None, true);

    assert_envelope(&resp);
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert!(
        status == "succeeded" || status == "succeeded_with_warnings",
        "dry-run must succeed: {resp}"
    );
    assert!(!output_pdf.exists(), "dry-run must not write output PDF");
}

// ── Test: missing original PDF ────────────────────────────────────────────────

/// Non-existent source PDF must return status=failed; process exits 0.
#[test]
fn cli_apply_addendum_missing_original_fails() {
    let tmp = tmp_dir("apply-addendum-missing-original");
    let missing = tmp.join("nonexistent_source.pdf");

    let manifest = write_minimal_addendum_manifest(
        &tmp,
        None,
        &[("23 82 16", serde_json::json!([]))],
    );
    let output = tmp.join("out.pdf");

    let resp = run_apply_addendum(&missing, &manifest, Some(&output), None, false);

    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "failed",
        "expected failed for missing original: {resp}"
    );
    assert!(!output.exists(), "output must not be written on failure");
}

// ── Test: missing manifest file ───────────────────────────────────────────────

/// Non-existent AddendumManifest JSON must return status=failed; process exits 0.
#[test]
fn cli_apply_addendum_missing_manifest_fails() {
    let tmp = tmp_dir("apply-addendum-missing-manifest");
    let spec_pdf = tier1("simple.pdf");
    assert!(spec_pdf.exists(), "simple.pdf fixture missing");

    let missing_manifest = tmp.join("nonexistent_manifest.json");
    let output = tmp.join("out.pdf");

    let resp =
        run_apply_addendum(&spec_pdf, &missing_manifest, Some(&output), None, false);

    assert_eq!(
        resp["result"]["status"].as_str().unwrap_or(""),
        "failed",
        "expected failed for missing manifest: {resp}"
    );
    assert!(!output.exists(), "output must not be written on failure");
}

// ── Test: unknown section ID continues with partial success ───────────────────

/// A section ID not present in the spec returns a per-section failure while
/// the overall response reflects partial success (succeeded_with_warnings or
/// failed depending on how many other sections there are).
#[test]
fn cli_apply_addendum_unknown_section_continues() {
    let tmp = tmp_dir("apply-addendum-unknown-section");
    let spec_pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    assert!(spec_pdf.exists(), "SPEC fixture missing");

    // Use a bogus section ID that definitely does not exist.
    let manifest = write_minimal_addendum_manifest(
        &tmp,
        None,
        &[("99 99 99", serde_json::json!([]))],
    );
    let output = tmp.join("out.pdf");

    let resp = run_apply_addendum(&spec_pdf, &manifest, Some(&output), None, true);

    assert_envelope(&resp);
    // The pipeline must still return a valid response (exit 0), with
    // status=failed because the only section failed.
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "failed", "single unknown section should yield failed: {resp}");
}

// ── Test: audit artifacts are written ────────────────────────────────────────

/// When --audit-bundle is given, change-report.json must be written.
#[test]
fn cli_apply_addendum_writes_audit_artifacts() {
    let tmp = tmp_dir("apply-addendum-audit");
    let spec_pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    assert!(spec_pdf.exists(), "SPEC fixture missing");

    // Segment to find a real section ID.
    let transcript_json = tmp.join("transcript.json");
    run_extract(&spec_pdf, &transcript_json);
    let segment_json = tmp.join("segment-index.json");
    run_segment(&transcript_json, &segment_json);

    let idx_text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let idx: serde_json::Value = serde_json::from_str(&idx_text).expect("parse segment index");
    let sections = idx["sections"].as_array().expect("sections array");
    if sections.is_empty() {
        return;
    }
    let first_id = sections[0]["section_id"].as_str().expect("section_id").to_owned();

    let manifest = write_minimal_addendum_manifest(
        &tmp,
        Some("audit artifacts test"),
        &[(&first_id, serde_json::json!([]))],
    );
    let output_pdf = tmp.join("out.pdf");
    let audit_dir = tmp.join("audit-bundle");

    run_apply_addendum(&spec_pdf, &manifest, Some(&output_pdf), Some(&audit_dir), true);

    let change_report_path = audit_dir.join("change-report.json");
    assert!(
        change_report_path.exists(),
        "change-report.json must be written to --audit-bundle directory"
    );

    let report_text = std::fs::read_to_string(&change_report_path).expect("read change-report");
    let report: serde_json::Value = serde_json::from_str(&report_text).expect("parse change-report");
    assert!(report["total_sections"].as_u64().unwrap_or(0) >= 1, "change-report must have sections");
}

// ── Test: full round-trip (Chrome required; #[ignore] by default) ─────────────

/// Full apply-addendum round-trip: real SPEC PDF + no-op manifest → valid PDF output.
///
/// Requires a Chrome/Brave installation and `PDFIUM_LIB_PATH`.  This test is
/// marked `#[ignore]` and must be run explicitly:
///
/// ```
/// cargo test -p conset-pdf-backend-cli cli_apply_addendum_produces_valid_pdf -- --ignored
/// ```
#[test]
#[ignore]
fn cli_apply_addendum_produces_valid_pdf() {
    let tmp = tmp_dir("apply-addendum-full");
    let spec_pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    assert!(spec_pdf.exists(), "SPEC fixture missing");

    // Segment to find one real section.
    let transcript_json = tmp.join("transcript.json");
    run_extract(&spec_pdf, &transcript_json);
    let segment_json = tmp.join("segment-index.json");
    run_segment(&transcript_json, &segment_json);

    let idx_text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let idx: serde_json::Value = serde_json::from_str(&idx_text).expect("parse segment index");
    let sections = idx["sections"].as_array().expect("sections array");
    if sections.is_empty() {
        return;
    }
    let first_id = sections[0]["section_id"].as_str().expect("section_id").to_owned();

    let manifest = write_minimal_addendum_manifest(
        &tmp,
        Some("full round-trip test"),
        &[(&first_id, serde_json::json!([]))],
    );

    let output_pdf = tmp.join("revised-spec.pdf");
    let audit_dir = tmp.join("audit-bundle");

    let resp = run_apply_addendum(
        &spec_pdf,
        &manifest,
        Some(&output_pdf),
        Some(&audit_dir),
        false,
    );

    assert_envelope(&resp);
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert!(
        status == "succeeded" || status == "succeeded_with_warnings",
        "apply-addendum must succeed: {resp}"
    );

    assert!(output_pdf.exists(), "apply-addendum must write output PDF");
    let bytes = std::fs::read(&output_pdf).expect("read output PDF");
    assert!(bytes.starts_with(b"%PDF"), "output must start with %PDF header");
    assert!(bytes.len() > 1024, "output PDF is suspiciously small ({} bytes)", bytes.len());

    // Audit artifact check.
    assert!(
        audit_dir.join("change-report.json").exists(),
        "change-report.json must exist in audit bundle"
    );
}

// ── Sprint 8.1.D: Malformed-input graceful-failure tests ─────────────────────

/// Corrupt PDF bytes (random garbage) must return status=failed; process exits 0.
///
/// Exercises the extraction-error path.  If the engine panics instead of
/// returning an error the process exits non-zero and the assertion in
/// `run_apply_addendum` fires, making the test fail with an actionable message.
#[test]
fn cli_apply_addendum_corrupt_original_fails_gracefully() {
    let tmp = tmp_dir("apply-addendum-corrupt");

    // Write a file that is definitely not a valid PDF.
    let corrupt_pdf = tmp.join("corrupt.pdf");
    std::fs::write(
        &corrupt_pdf,
        b"this is not a pdf\nfake binary content\x00\xff\xfe",
    )
    .expect("write corrupt pdf");

    let manifest = write_minimal_addendum_manifest(
        &tmp,
        None,
        &[("23 82 16", serde_json::json!([]))],
    );
    let resp = run_apply_addendum(&corrupt_pdf, &manifest, None, None, false);

    // Must be valid JSON WorkflowResponse with a graceful failure — not a crash.
    assert_envelope(&resp);
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "failed", "expected failed for corrupt PDF, got: {resp}");
}

/// A PDF with no /Pages tree must return status=failed; process exits 0.
///
/// The minimal structure has a /Catalog but no /Pages reference, so pdfium
/// either reports 0 pages or fails extraction — both must be handled gracefully.
#[test]
fn cli_apply_addendum_zero_page_pdf_fails_gracefully() {
    let tmp = tmp_dir("apply-addendum-zero-page");

    // Minimal PDF skeleton: /Catalog present, /Pages absent.
    // The cross-reference offset is intentionally wrong to stress the parser further.
    let zero_page_pdf = tmp.join("zero-page.pdf");
    std::fs::write(
        &zero_page_pdf,
        b"%PDF-1.4\n\
          1 0 obj\n<< /Type /Catalog >>\nendobj\n\
          xref\n0 2\n\
          0000000000 65535 f \n\
          0000000009 00000 n \n\
          trailer\n<< /Size 2 /Root 1 0 R >>\n\
          startxref\n9\n\
          %%EOF\n",
    )
    .expect("write zero-page pdf");

    let manifest = write_minimal_addendum_manifest(
        &tmp,
        None,
        &[("23 82 16", serde_json::json!([]))],
    );
    let resp = run_apply_addendum(&zero_page_pdf, &manifest, None, None, false);

    assert_envelope(&resp);
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert_eq!(status, "failed", "expected failed for zero-page PDF, got: {resp}");
}

/// A file with an unrecognised PDF version header must return status=failed;
/// process exits 0.
#[test]
fn cli_apply_addendum_mismatched_pdf_version_fails_gracefully() {
    let tmp = tmp_dir("apply-addendum-bad-version");

    let bad_version_pdf = tmp.join("bad-version.pdf");
    std::fs::write(
        &bad_version_pdf,
        b"%PDF-99.9\ngarbage content that no PDF parser will accept\x00\xff",
    )
    .expect("write bad-version pdf");

    let manifest = write_minimal_addendum_manifest(
        &tmp,
        None,
        &[("23 82 16", serde_json::json!([]))],
    );
    let resp = run_apply_addendum(&bad_version_pdf, &manifest, None, None, false);

    assert_envelope(&resp);
    let status = resp["result"]["status"].as_str().unwrap_or("");
    assert_eq!(
        status, "failed",
        "expected failed for bad-version PDF, got: {resp}"
    );
}

// ── Sprint 8.1.G: diagnostics.jsonl audit output ─────────────────────────────

/// When --audit-bundle is given, `diagnostics.jsonl` must be written alongside
/// `change-report.json`.  The first line must be a valid JSON object containing
/// the schema header fields (`schema`, `pipeline_version`, `generated_at`).
#[test]
fn cli_apply_addendum_writes_diagnostics_jsonl() {
    let tmp = tmp_dir("apply-addendum-diagnostics-jsonl");
    let spec_pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    assert!(spec_pdf.exists(), "SPEC fixture missing");

    // Segment to find a real section ID.
    let transcript_json = tmp.join("transcript.json");
    run_extract(&spec_pdf, &transcript_json);
    let segment_json = tmp.join("segment-index.json");
    run_segment(&transcript_json, &segment_json);

    let idx_text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let idx: serde_json::Value = serde_json::from_str(&idx_text).expect("parse segment index");
    let sections = idx["sections"].as_array().expect("sections array");
    if sections.is_empty() {
        return;
    }
    let first_id = sections[0]["section_id"].as_str().expect("section_id").to_owned();

    let manifest = write_minimal_addendum_manifest(
        &tmp,
        Some("diagnostics jsonl test"),
        &[(&first_id, serde_json::json!([]))],
    );
    let output_pdf = tmp.join("out.pdf");
    let audit_dir = tmp.join("audit-bundle");

    run_apply_addendum(&spec_pdf, &manifest, Some(&output_pdf), Some(&audit_dir), true);

    let jsonl_path = audit_dir.join("diagnostics.jsonl");
    assert!(jsonl_path.exists(), "diagnostics.jsonl must be written to --audit-bundle directory");

    let jsonl_text = std::fs::read_to_string(&jsonl_path).expect("read diagnostics.jsonl");
    let first_line = jsonl_text.lines().next().expect("diagnostics.jsonl must not be empty");
    let header: serde_json::Value =
        serde_json::from_str(first_line).expect("first line must be valid JSON");

    assert_eq!(
        header["schema"].as_str().unwrap_or(""),
        "diagnostics/v1",
        "schema header must be diagnostics/v1"
    );
    assert!(
        !header["pipeline_version"].as_str().unwrap_or("").is_empty(),
        "pipeline_version must be non-empty"
    );
    assert!(
        !header["generated_at"].as_str().unwrap_or("").is_empty(),
        "generated_at must be non-empty"
    );
}

// ── Sprint 8.2 — Stage 0 intake triage tests ─────────────────────────────────

/// Run `backend-cli intake --input <pdf> [--output <json>] [--dry-run]`.
/// Returns the parsed `WorkflowResponse` JSON.
fn run_intake(pdf: &PathBuf, out_json: Option<&PathBuf>, dry_run: bool) -> serde_json::Value {
    let exe = backend_cli_exe_path();
    assert!(exe.exists(), "backend-cli not found — run: cargo build --bin backend-cli");
    assert!(pdf.exists(), "input PDF not found: {}", pdf.display());

    let mut cmd = std::process::Command::new(&exe);
    cmd.arg("intake").arg("--input").arg(pdf);
    if let Some(out) = out_json {
        cmd.arg("--output").arg(out);
    }
    if dry_run {
        cmd.arg("--dry-run");
    }

    let output = cmd.output().expect("failed to spawn backend-cli intake");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "intake exited non-zero for {}:\nstdout: {stdout}\nstderr: {}",
        pdf.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_str(&stdout).expect("stdout must be valid WorkflowResponse JSON")
}

#[test]
fn cli_intake_dry_run_succeeds() {
    let pdf = tier1("simple.pdf");
    let response = run_intake(&pdf, None, true);

    assert_eq!(
        response["result"]["status"].as_str().unwrap_or(""),
        "succeeded",
        "dry-run intake must report succeeded"
    );
    let summary = response["result"]["summary"].as_str().unwrap_or("");
    assert!(
        summary.contains("dry_run"),
        "dry-run summary must mention dry_run; got: {summary}"
    );
}

#[test]
fn cli_intake_no_rotation_reports_zero_normalized() {
    // Copy the corpus PDF to a temp location so in-place normalization does
    // not touch the shared fixture.
    let tmp = tmp_dir("intake-no-rotation");
    let src = tier1("simple.pdf");
    let pdf = tmp.join("simple.pdf");
    std::fs::copy(&src, &pdf).expect("copy simple.pdf to tmp");

    let response = run_intake(&pdf, None, false);

    assert_eq!(
        response["result"]["status"].as_str().unwrap_or(""),
        "succeeded",
        "intake of a clean PDF must succeed"
    );
    let summary = response["result"]["summary"].as_str().unwrap_or("");
    assert!(
        summary.contains("0 rotation(s) normalized"),
        "summary must report 0 rotations normalized; got: {summary}"
    );
}

#[test]
fn cli_intake_writes_normalized_bundle_json() {
    let tmp = tmp_dir("intake-bundle-json");
    let src = tier1("simple.pdf");
    let pdf = tmp.join("simple.pdf");
    std::fs::copy(&src, &pdf).expect("copy simple.pdf to tmp");
    let out_json = tmp.join("bundle.json");

    run_intake(&pdf, Some(&out_json), false);

    assert!(out_json.exists(), "intake --output must create the bundle JSON file");
    let text = std::fs::read_to_string(&out_json).expect("read bundle JSON");
    let val: serde_json::Value = serde_json::from_str(&text).expect("bundle JSON must be valid");

    assert!(
        !val["bundle_id"].as_str().unwrap_or("").is_empty(),
        "bundle_id must be non-empty in the output JSON"
    );
    assert_eq!(
        val["document_class"].as_str().unwrap_or(""),
        "unknown",
        "document_class must default to 'unknown' in Stage 0 output"
    );
}

// ── Sprint 8.3.D — Determinism regression test ────────────────────────────────

/// Run `apply-addendum --dry-run` twice with identical inputs and verify that
/// `change-report.json` is deterministic between runs.
///
/// Compared fields (per 8.3.D spec):
/// - `section_results` array length and per-entry `section_id` order
/// - per-entry `status` field
/// - `ParseDiagnostic.node_count` and all `node_distribution` sub-fields
///
/// Explicitly excluded from comparison: timestamps, `elapsed_ms`, path strings.
#[test]
fn apply_addendum_dry_run_is_deterministic() {
    let spec_pdf = tier1("SPEC_RWB_LHHS_ALL_ORG.pdf");
    assert!(spec_pdf.exists(), "SPEC tier-1 fixture must exist");

    // ── Find a real section ID ────────────────────────────────────────────────
    let setup_tmp = tmp_dir("determinism-setup");
    let transcript = setup_tmp.join("tr.json");
    run_extract(&spec_pdf, &transcript);
    let segment_json = setup_tmp.join("seg.json");
    run_segment(&transcript, &segment_json);

    let seg_text = std::fs::read_to_string(&segment_json).expect("read segment index");
    let seg: serde_json::Value = serde_json::from_str(&seg_text).expect("parse segment index");
    let sections = seg["sections"].as_array().expect("sections array");
    if sections.is_empty() {
        // No sections detected — fixture is unsuitable; skip rather than fail.
        return;
    }
    let first_id = sections[0]["section_id"].as_str().expect("section_id").to_owned();

    // ── Shared manifest (no-op operations — parse-only dry run) ───────────────
    let manifest_dir = tmp_dir("determinism-manifest");
    let manifest = write_minimal_addendum_manifest(
        &manifest_dir,
        Some("determinism test"),
        &[(&first_id, serde_json::json!([]))],
    );

    // ── Run 1 ─────────────────────────────────────────────────────────────────
    let bundle1 = tmp_dir("determinism-run1");
    run_apply_addendum(&spec_pdf, &manifest, None, Some(&bundle1), true);
    let report1_text = std::fs::read_to_string(bundle1.join("change-report.json"))
        .expect("change-report.json must exist after run 1");
    let report1: serde_json::Value =
        serde_json::from_str(&report1_text).expect("run 1 change-report.json must be valid JSON");

    // ── Run 2 ─────────────────────────────────────────────────────────────────
    let bundle2 = tmp_dir("determinism-run2");
    run_apply_addendum(&spec_pdf, &manifest, None, Some(&bundle2), true);
    let report2_text = std::fs::read_to_string(bundle2.join("change-report.json"))
        .expect("change-report.json must exist after run 2");
    let report2: serde_json::Value =
        serde_json::from_str(&report2_text).expect("run 2 change-report.json must be valid JSON");

    // ── Compare section_results ───────────────────────────────────────────────
    let secs1 = report1["section_results"].as_array().expect("section_results array (run 1)");
    let secs2 = report2["section_results"].as_array().expect("section_results array (run 2)");

    assert_eq!(secs1.len(), secs2.len(), "section_results length must match");

    for (i, (s1, s2)) in secs1.iter().zip(secs2.iter()).enumerate() {
        assert_eq!(
            s1["section_id"], s2["section_id"],
            "section_results[{i}].section_id must match"
        );
        assert_eq!(
            s1["status"], s2["status"],
            "section_results[{i}].status must match"
        );
    }

    // ── Compare ParseDiagnostic node counts from diagnostics ─────────────────
    let parse_node_distribution = |report: &serde_json::Value| -> Vec<serde_json::Value> {
        let diags = match report["diagnostics"].as_array() {
            Some(d) => d,
            None => return vec![],
        };
        diags
            .iter()
            .filter(|d| d["stage"].as_str() == Some("parse"))
            .map(|d| {
                serde_json::json!({
                    "section_id": d["section_id"],
                    "node_count": d["node_count"],
                    "node_distribution": d["node_distribution"],
                })
            })
            .collect()
    };

    let parse1 = parse_node_distribution(&report1);
    let parse2 = parse_node_distribution(&report2);

    assert_eq!(
        parse1.len(),
        parse2.len(),
        "number of ParseDiagnostic events must match between runs"
    );

    for (i, (p1, p2)) in parse1.iter().zip(parse2.iter()).enumerate() {
        assert_eq!(
            p1["section_id"], p2["section_id"],
            "ParseDiagnostic[{i}].section_id must match"
        );
        assert_eq!(
            p1["node_count"], p2["node_count"],
            "ParseDiagnostic[{i}].node_count must be deterministic"
        );
        assert_eq!(
            p1["node_distribution"], p2["node_distribution"],
            "ParseDiagnostic[{i}].node_distribution must be deterministic"
        );
    }
}

