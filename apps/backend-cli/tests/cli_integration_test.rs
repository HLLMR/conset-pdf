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
