## Phase 7 Completion Verification

**Phase 7 is fully complete.** All code is present, all tests pass. Baseline confirmed:

| Suite | Passing | Ignored |
|---|---|---|
| CLI integration | 41/41 | 2 (Chrome) |
| Engine unit (all files) | 79 | 2 (Chrome) |
| IR unit (lib.rs) | 30 | — |
| IR integration tests | 48 | — |
| pdf-extraction | 16 | — |
| audit | 5 | — |
| pattern-dev unit | 32 | — |
| pattern-dev integration | 3 | 2 (binary/corpus) |

---

## Sprint 8.1 Completion Record — April 7, 2026

**Sprint 8.1 is fully complete.** All tasks H, A, B, C, D, E, F, G, I implemented and tested.

| Suite | Passing | Ignored | Delta vs. Phase 7 baseline |
|---|---|---|---|
| CLI integration | 47/47 | 2 (Chrome) | +6 (8.1.D ×3, 8.1.G ×1, 8.1.H ×1, misc ×1) |
| Engine lib | 79 | 3 (Chrome) | +9 new passing; +1 new ignored (8.1.I) |
| IR lib | 37 | — | +7 (8.1.E diagnostic round-trips) |

**Implementation notes where actual code diverges from the plan spec:**

- **8.1.F** — `build_extraction_diagnostic` was factored out as a private helper in `specs_patch.rs` (not a new public function) to enable unit testing without going through the full orchestrator.
- **8.1.G** — Only the first test (`cli_apply_addendum_writes_diagnostics_jsonl`) was written. The second planned test (`apply_addendum_diagnostics_unclassified_nodes_traced`) is deferred; a dry-run fixture does not guarantee an unclassified section. Mark as a 8.3 corpus validation concern.
- **8.1.I** — `chrome_binary_version` is threaded via a separate `probe_chrome_version(&chrome_path)` call in `render/mod.rs` immediately after `render_html_to_pdf` returns the `PathBuf`. `find_chrome()` return type was **not** changed. `chrome_binary_version: String` added to both `RenderResult` and `RenderDiagnostic` (with `#[serde(default)]` on `RenderDiagnostic`).
- **8.1.H** — unit test is `orchestrator_temp_dir_cleaned_up_on_section_failure` (name differs slightly from plan spec `orchestrator_temp_dir_cleaned_up_on_parse_failure`).

---

---

## Pre-Phase 8 Work Not Yet Completed

There is exactly **one minor documentation hygiene item** and **zero functional gaps**.

### 1. capability-matrix.md version/date is stale (minor)

capability-matrix.md shows `Version: 2.1.0, Date: April 6, 2026`. Phase 7 landed April 7. The **content** correctly reflects Phase 7 complete (the `apply-addendum` row and end-to-end workflow row are both Implemented), but the header version/date predates it. Should be bumped to `2.2.0 / April 7, 2026`.

All other Phase 7 task 7.8 documentation (CHANGELOG, state-summary, MASTER_PLAN checkboxes, repo memory) is current.

---

## Architectural Foundation Assessment: Phase 8 Starting Point

> _Senior architect review of the current implementation vs. what's needed before Phase 9 consumers are built on top of this foundation. Conducted via direct code audit of `specs_patch.rs`, `stitch.rs`, `chrome_pdf.rs`, and the test suite before Phase 8 work begins._

### What's solid

The core pipeline is architecturally correct. The data model flows cleanly from `LayoutTranscript` → `SegmentIndex` → `ParsedDocument` → `EditResult` → rendered PDF bytes → stitched output, with typed boundaries at each stage. Error propagation is `Result`-based throughout the engine; no FFI errors or lopdf failures are silently discarded. The test suite is substantial (41 CLI integration + 81 engine unit/e2e tests passing) and Phase 3 parser hardening achieved 0.2% unclassified nodes against the primary corpus fixture (571 pages, 89 sections, 7,971 nodes). The last-to-first stitch ordering is mathematically correct for the page-replacement case — stitching later sections first means earlier section indices are unaffected by page count changes in sections downstream of them. `SectionEditor`, `PdfStitcher`, and `SpecsPatchOrchestrator` all have meaningful test coverage for their happy-path and error-path scenarios.

### Five structural risks

**Risk 1 — The primary production workflow has no CI coverage (Severity: HIGH)**

`cli_apply_addendum_produces_valid_pdf` and `cli_regenerate_produces_pdf` are both `#[ignore]` because they require a Chromium browser at test time. Every other test validates parts of the pipeline in isolation or validates graceful failure on error inputs. None automatically verify that a successful run produces a correctly structured output PDF. This is tolerable at Alpha but is not acceptable at Beta. For Phase 8, the determinism test **8.3.D** adds a Chrome-free regression that covers the full dry-run pipeline identity. The benchmark in **8.4.A** serves as a Chrome smoke test when run manually. The goal is that CI catches any regression in the pipeline stages that do not require Chrome, and the Chrome tests are `#[ignore]` but documented and regularly run in development.

**Risk 2 — Determinism is the project's #1 non-negotiable and is untested (Severity: HIGH)**

There is no automated test that runs the pipeline twice on the same input and verifies identical output. Determinism regressions can be introduced silently by: an unstable sort over floating-point values, `HashMap` iteration order leaking into output field ordering, or `SystemTime::now()` appearing in a serialized pipeline output. Phase 0.5 added a determinism test for the pattern-dev tool sidecar; the main apply-addendum pipeline has nothing equivalent. **8.3.D** closes this gap with a dry-run double-execution test that compares `section_results` ordering, node distributions, and diagnostic fields (excluding timestamps and elapsed durations).

**Risk 3 — Temp directory resource leak (Severity: MEDIUM)**

`SpecsPatchOrchestrator::run()` creates `$TMPDIR/specs_patch_{timestamp}/` via `std::fs::create_dir_all()` and holds the path as `Option<PathBuf>`. `PathBuf::drop()` does not touch the filesystem. If the function returns `Err(...)` at any point after directory creation — or if the process is killed mid-run — the directory and all intermediate rendered PDFs in it persist indefinitely. In a long-running production environment processing many jobs in sequence, this produces unbounded temp disk accumulation. The fix is idiomatic Rust: `tempfile::TempDir` implements `Drop` to call `remove_dir_all()` on scope exit, including `?` propagation, panics, and normal returns. **8.1.H** closes this gap.

**Risk 4 — `validate_unchanged_present()` verifies object presence, not content identity (Severity: MEDIUM)**

Non-Negotiable #12 states: "Unchanged pages must remain unchanged." The current implementation in `stitch.rs::validate_unchanged_present()` checks `doc.objects.contains_key(id)` — object presence in the objects map only. It does not verify that the object's byte content is identical to the original. A lopdf mutation bug — dictionary key reordering, cross-reference renumbering of an unrelated object leaking into a shared page stream — could silently corrupt an unchanged page object while this check passes cleanly. **8.3.E** adds a per-object content-hash comparison.

**Risk 5 — Full-document extraction for partial operations is an undocumented architecture constraint (Severity: LOW now; HIGH for Phase 9 planning)**

`SpecsPatchOrchestrator::run()` calls `Extractor::extract()` on the entire source PDF before processing any section. For a 571-page spec book being patched at 3 sections, all 571 pages are extracted and materialized in memory as a `LayoutTranscript`, even though only the pages belonging to those three sections are needed for parsing and editing. The remainder are only needed by the stitcher to confirm unchanged-page presence — a check that does not require them to be in the `LayoutTranscript` at all. This is a known and acceptable tradeoff for single-run spec-book use cases. The risk is not current performance. It is that this constraint is invisible in the architecture documentation. Phase 9 (Drawing Sheet Management) will involve batch processing of large drawing sets. If Phase 9 architects assume extraction is naturally range-bounded, they will design batch pipelines that are one to two orders of magnitude slower than they need to be. **8.4.B** includes a mandatory documentation task in `ARCHITECTURE.md`.

### Verdict: Viable — with four conditions

The foundation is solid enough to support Phase 8 work and is already serving real spec-book addenda use cases. It is not ready to have Phase 9+ drawing-sheet batch consumers designed on top of it without closing the following gaps first:

1. **8.1.H must land before any Phase 9 batch-mode code is written** — resource leaks compound in batch systems; a single-run leak becomes an unbounded growth problem in overnight batch jobs.
2. **8.3.D must land before Beta is declared** — an untested determinism guarantee is not a guarantee.
3. **8.3.E must land before Beta** — the unchanged-page contract is a non-negotiable that currently has no teeth.
4. **8.4.B constraint documentation must be written before Phase 9 design begins** — not to block Phase 9, but to ensure Phase 9 drawing batch architecture decisions are made with accurate cost model information.

All four are addressable within Phase 8 at low implementation cost. None require architectural redesign.

---

## Phase 8 Plan

Phase 8 per MASTER_PLAN is "Production Hardening" — the Alpha → Beta gate. The Phase 7 plan explicitly deferred three item families here: **intake triage** (G-013/G-014/G-016), **pattern database as versioned JSON**, and the standard hardening work (error handling, corpus validation, performance, metrics, docs). Below is a micro-task decomposition in sprint order.

---

### Sprint 8.0 — Documentation Housekeeping ✅ COMPLETE (30 min)

**8.0.A — capability-matrix.md version bump** ✅
- Bump to v2.2.0, date April 7, 2026
- This is the only real doc debt from Phase 7

**8.0.B — MASTER_PLAN version bump to 4.9.0** ✅
- Update version header and date in MASTER_PLAN.md to reflect Phase 8 planning cycle

---

### Sprint 8.1 — Crash Containment, Error Handling & Structured Diagnostics ✅ COMPLETE (~3 days)

This sprint has two intertwined goals. The first is classic hardening: eliminate panics, sharpen error messages, cap memory. The second is more forward-looking: deploy a structured diagnostic trace system that turns the audit bundle from a "did it work?" receipt into a "here is exactly what happened at every stage" post-mortem. These goals are inseparable — a well-structured `DiagnosticEvent` type is the foundation that makes error context unambiguous, not just at the top-level handler, but at the exact stage and input where something went sideways.

**Design principle:** every stage decision that can produce a silent wrong result emits a typed event with its evidence. Text log lines are not enough; the goal is a machine-readable, stable-schema JSONL file that future diagnostics tooling and test assertions can parse without screen-scraping.

**8.1.A — Panic audit across the pipeline** ✅ (+3 unit tests)
- Search all `unwrap()` / `expect()` / `panic!()` sites in engine, ir, backend-cli
- Replace panic-able sites in non-test code with `Result` propagation to the nearest `SectionPatchResult::Failed` or CLI error handler
- Target: zero panics reachable from valid-but-corrupt PDF input
- Unit tests: 3 tests with intentionally malformed PDFs (zero-page, truncated, corrupt page tree)

**8.1.B — Actionable error message audit** ✅
- Review all `StitchError`, `EngineError`, `ParseError`, and `SectionPatchResult::Failed { reason }` message strings
- Each must state: what failed + what the user should check
- Current weakest spots: `PdfStructure` errors in `stitch.rs` (bare lopdf errors leak through), and `SectionPatchResult::Failed` reasons that could be more specific about which pipeline stage failed

**8.1.C — Memory bounds for large PDFs** ✅ (+2 unit tests; `MAX_PDF_PAGES = 2_000`; `EngineError::PdfTooLarge { page_count, max }`)
- Add configurable page-count cap (e.g., max 2,000 pages) at intake to prevent unbounded allocation
- Return a structured `PdfTooLarge { page_count, max }` error rather than OOM
- Tests: verify cap enforced, verify reasonable default

**8.1.D — Integration tests for malformed inputs** ✅ (+3 CLI integration tests)
- `apply_addendum_corrupt_original_fails_gracefully` — corrupt PDF bytes → `Failed` result, not crash
- `apply_addendum_zero_page_pdf_fails_gracefully`
- `apply_addendum_mismatched_pdf_version_fails_gracefully`

**8.1.E — `DiagnosticEvent` IR type and collector** ✅ (+7 serde round-trip tests; `crates/ir/src/diagnostics.rs` new; `AddendumResult.diagnostics: Vec<DiagnosticEvent>` with `#[serde(default)]`)

The pipeline currently produces no structured intermediate record of what happened at each stage — only a pass/fail outcome. `DiagnosticEvent` fills that gap: a typed, serializable event emitted by each stage and accumulated into `AddendumResult.diagnostics`, then written as newline-delimited JSON to the audit bundle. The variants are designed so that any failure reproducible in the wild can be triaged by reading it without access to the original PDF.

Define the following variants:

```rust
pub enum DiagnosticEvent {
    Extraction(ExtractionDiagnostic),
    Segmentation(SegmentationDiagnostic),
    Parse(ParseDiagnostic),
    Edit(EditDiagnostic),
    Render(RenderDiagnostic),
    Stitch(StitchDiagnostic),
}

pub struct ExtractionDiagnostic {
    pub page_count: usize,
    pub total_spans: usize,
    pub zero_span_pages: Vec<usize>,   // pages where PDFium returned 0 spans — raster/blank suspect
    pub low_span_pages: Vec<usize>,    // pages with < 5 spans — Form XObject or near-blank suspect
    pub elapsed_ms: u64,
}

pub struct SegmentationDiagnostic {
    pub section_count: usize,
    pub coverage_ratio: f64,
    pub pages_missing_footer: Vec<usize>,  // 0-indexed page numbers with no footer match
    pub sections: Vec<SegmentTrace>,
}

pub struct SegmentTrace {
    pub section_id: String,
    pub section_title: String,
    pub start_page: usize,
    pub end_page: usize,
    pub footer_match_count: usize,     // pages in this section run with a confirmed footer match
    pub page_counter_detected: bool,
}

pub struct ParseDiagnostic {
    pub section_id: String,
    pub total_lines: usize,
    pub noise_lines_skipped: usize,          // lines discarded by noise-only filter
    pub inject_missing_parts_count: usize,   // synthetic PART nodes injected by recovery pass
    pub node_count: usize,
    pub node_distribution: NodeDistribution,
    pub unclassified_nodes: Vec<UnclassifiedNodeTrace>,  // only populated when count > 0
}

pub struct NodeDistribution {
    pub part: usize,
    pub article: usize,
    pub paragraph: usize,
    pub sub_paragraph: usize,
    pub sub_sub_paragraph: usize,
    pub sub_sub_sub_paragraph: usize,
    pub unclassified: usize,
}

pub struct UnclassifiedNodeTrace {
    pub page_index: usize,
    pub x_indent: f64,
    pub text_snippet: String,   // first 80 chars; enough to identify the line without full dump
}

pub struct EditDiagnostic {
    pub section_id: String,
    pub operations_attempted: usize,
    pub operations_applied: usize,
    pub failures: Vec<EditFailureTrace>,
}

pub struct EditFailureTrace {
    pub operation_index: usize,
    pub op_type: String,       // "insert_after" | "delete" | "replace"
    pub path: Vec<String>,     // the NodePath markers attempted
    pub reason: String,
}

pub struct RenderDiagnostic {
    pub section_id: String,
    pub chrome_binary: String,         // actual path selected; "dry-run" when --dry-run
    pub chrome_binary_version: String, // from `probe_chrome_version()`; "unknown" on failure/dry-run
    pub html_size_bytes: usize,
    pub elapsed_ms: u64,
    pub outcome: RenderOutcome,
}

pub enum RenderOutcome {
    Success { output_size_bytes: usize },
    DryRun,
    Failed { exit_code: Option<i32>, stderr_tail: String },  // last 500 chars of Chrome stderr
}

pub struct StitchDiagnostic {
    pub section_id: String,
    pub pages_removed: usize,
    pub pages_inserted: usize,
    pub bookmarks_rerouted: usize,
    pub elapsed_ms: u64,
    pub warnings: Vec<String>,
}
```

- Mark `DiagnosticEvent` and `RenderOutcome` with `#[non_exhaustive]` — Phase 9 will add a `DrawingStage(DrawingStageDiagnostic)` variant; `#[non_exhaustive]` forces downstream `match` arms to include a `_` catchall now rather than breaking silently when the new variant arrives
- Add `pub diagnostics: Vec<DiagnosticEvent>` to `AddendumResult` with `#[serde(default)]` for backward compat
- Export all diagnostic types from `crates/ir/src/lib.rs`
- Serde round-trip unit tests for each variant (7 tests: one per event type)

**Key design notes:**
- `zero_span_pages` and `low_span_pages` in `ExtractionDiagnostic` are the fastest signal for raster content infiltrating a vector pipeline — pages where PDFium found nothing or almost nothing indicate either blank pages, full-page images, or Form XObjects the FPDFText path can't descend into
- `pages_missing_footer` in `SegmentationDiagnostic` is the list of 0-indexed page numbers with no footer match — this can be cross-referenced against the visual overlay PNGs to identify where the segmenter went blind
- `UnclassifiedNodeTrace.text_snippet` (80 chars) is deliberately not the full line — full content of every unclassified line across hundreds of sections would bloat the bundle; 80 chars is enough to identify what pattern it resembles and why the classifier rejected it
- `RenderDiagnostic.stderr_tail` (last 500 chars of Chrome stderr) is the most useful piece of data when regeneration fails or produces a malformed PDF — Chrome's error messages are specific and actionable, but they're currently discarded entirely
- `StitchDiagnostic` mirrors `StitchResult` but with `elapsed_ms` added — the existing `StitchResult.warnings` list (unchanged page objects missing after splice) maps directly in

**8.1.F — Per-stage diagnostic emission wired into `SpecsPatchOrchestrator`** ✅ (+3 unit tests: extraction zero-span, parse unclassified, render stderr-tail)

Wire `DiagnosticEvent` emission at each stage. Diagnostics are accumulated in a local `Vec<DiagnosticEvent>` within `run()` and moved into `AddendumResult.diagnostics` at return. No file I/O at this level — the handler owns the file write (8.1.G).

- **Extraction**: After `Extractor::extract()` completes, walk the `LayoutTranscript` once to compute `ExtractionDiagnostic`. `zero_span_pages` = pages where `page.spans.is_empty()`; `low_span_pages` = pages where `page.spans.len() < 5`. Record `elapsed_ms` via `std::time::Instant` wrapping the extract call. This single pass costs O(n_pages) which is negligible.

- **Segmentation**: After `SegmentEngine::build_index()` completes, derive `SegmentationDiagnostic` from the returned `SegmentIndex`. `pages_missing_footer` is the complement of `(0..page_count).filter(|p| any section contains p)`. Per-section `SegmentTrace` maps directly from `SectionEntry` fields — section_id, section_title (now populated since 7.0.A), start_page, end_page; `footer_match_count` = `end_page - start_page + 1` as a conservative estimate (the segment engine currently doesn't track per-page match confidence separately; 8.5.B will surface the internal count).

- **Parse** (per section): Extend `parse_section()` to return `(SectionAst, ParseStats)` where `ParseStats { total_lines, noise_lines_skipped, inject_missing_parts_count }`. Thread these counters up from `cluster_lines()` (noise-skip count) and `inject_missing_parts()` (invocation count). `unclassified_nodes` collected by a post-parse walk: `ast.nodes.iter().filter(|n| n.tag == OutlineTag::Unclassified).map(|n| UnclassifiedNodeTrace {...})`. This is the fastest debugging path for any parser regression — the unclassified node's text snippet immediately shows what pattern the classifier saw and rejected.

- **Edit** (per section): After `SectionEditor::apply()` returns, populate `EditDiagnostic` — `operations_attempted` = `request.operations.len()`, `operations_applied` counts `Ok` results, `failures` maps from `EditResult::Err` entries with the operation index, `EditOperation` type string, `NodePath.markers`, and `EditError` display string.

- **Render** (per section): Expose `chrome_binary_selected: String` from `render_html_to_pdf()` return (it currently discards this). Pass it up through `SectionRenderer::render()` into `RenderResult`. Capture `html_size_bytes` from `build_full_html()` output length. On Chrome failure, capture exit code from `ExitStatus` and the last 500 chars of Chrome's stderr — currently these are discarded entirely, which is the single biggest diagnostic blind spot in the pipeline.

- **Stitch** (per section): `StitchResult` already has `pages_removed`, `pages_inserted`, `bookmarks_updated`, `warnings` — map directly into `StitchDiagnostic` and wrap the `stitch()` call with `Instant` for `elapsed_ms`.

- _Files:_ `crates/engine/src/specs_patch.rs`, `crates/engine/src/parse.rs` (ParseStats return), `crates/engine/src/render/chrome_pdf.rs` (expose chrome binary path + stderr), `crates/engine/src/render/mod.rs` (thread through RenderResult)
- _Unit tests (3):_ `orchestrator_diagnostics_extraction_zero_span_pages_detected`, `orchestrator_diagnostics_parse_unclassified_nodes_traced`, `orchestrator_diagnostics_render_failure_stderr_captured`

**8.1.G — `diagnostics.jsonl` written to audit bundle** ✅ (+1 CLI integration test: `cli_apply_addendum_writes_diagnostics_jsonl`)

After `SpecsPatchOrchestrator::run()` returns, serialize `result.diagnostics` to `{audit_bundle}/diagnostics.jsonl` — one JSON object per line, UTF-8, no trailing comma. Cap at 8 MB: if the running serialized size would exceed the cap, append a sentinel line `{"truncated":true,"reason":"size_cap_exceeded","events_written":N}` and stop. The cap is a safety valve for pathological inputs, not an expected limit for normal runs.

The **first line is always a schema header** (not a `DiagnosticEvent`): `{"schema":"diagnostics/v1","pipeline_version":"<cargo pkg version>","generated_at":"<ISO-8601 UTC>"}`. This makes the file self-describing — future tooling and parsers can branch on `schema` version without heuristics, and the `generated_at` timestamp provides a run correlation key for matching `diagnostics.jsonl` against `change-report.json` from the same invocation.

The triad of audit artifacts and their distinct roles:
- `change-report.json` — **what happened**: section-level pass/fail outcomes and page counts
- `metrics.json` (Sprint 8.5) — **how well it went**: section-level stats roll-ups and coverage ratios
- `diagnostics.jsonl` — **exactly what the pipeline saw**: every stage, every decision, every anomaly

Any production failure should be fully diagnosable from these three files without access to the original PDF.

- _Tests:_ `apply_addendum_writes_diagnostics_jsonl` — verifies file created, every line is valid JSON, event types `Extraction`/`Segmentation`/`Parse`/`Stitch` all present for a dry-run; `apply_addendum_diagnostics_unclassified_nodes_traced` — fixture with a known-unclassified section verifies the `UnclassifiedNodeTrace` entries appear with correct text snippets

**8.1.H — Temp directory RAII cleanup** ✅ (+1 unit test: `orchestrator_temp_dir_cleaned_up_on_section_failure`)

`SpecsPatchOrchestrator::run()` currently creates `$TMPDIR/specs_patch_{timestamp}/` and stores the path in `Option<std::path::PathBuf>`. `PathBuf::drop()` does not touch the filesystem, so any `?`-propagated error after directory creation leaves the directory and all rendered intermediate PDFs behind permanently. This task replaces the plain `PathBuf` with `tempfile::TempDir`, whose `Drop` implementation calls `remove_dir_all()` on scope exit, providing RAII cleanup on all exit paths — normal return, early `Err(...)` return, and process panic.

- Add `tempfile = "3"` to workspace `[dependencies]` in the root `Cargo.toml` (confirmed not present as transitive dep) — add it at workspace level rather than engine-only because Sprint 8.2 intake staging (`NormalizedIntakeBundle` temp write path) and any future batch worker will need it too
- Replace `temp_dir: Option<PathBuf>` with `temp_dir: Option<tempfile::TempDir>` in `run()` body
- Update the two `temp_dir.as_ref().unwrap()` call sites to use `temp_dir.as_ref().unwrap().path()` — these `.unwrap()`s are safe by the `dry_run` invariant but converting them to use the `TempDir` accessor eliminates the code smell
- _Unit test:_ `orchestrator_temp_dir_cleaned_up_on_parse_failure` — invoke `run()` with a valid source PDF and a manifest referencing a non-existent section ID; assert `run()` returns `Ok` with a failed section result; assert the temp directory that would have been created does NOT persist after return

**8.1.I — Chrome binary version captured in audit bundle** ✅ (+1 `#[ignore]` unit test: `chrome_version_probe_returns_nonempty_string`; `chrome_binary_version: String` added to `RenderDiagnostic` and `RenderResult`)

`find_chrome()` returns the selected binary `PathBuf` but never queries its version. The audit bundle therefore cannot distinguish between a Chrome 120 render (which correctly applies CSS `@page` rules) and an older version that silently drops them. After `find_chrome()` succeeds, run `{path} --version` with a hard 3-second timeout; capture the first line of stdout as the version string. Store it as `chrome_binary_version: String` in `RenderDiagnostic` (default `"unknown"` if the probe fails — failure must not abort the render pipeline). Write `chrome_binary_version` to `change-report.json`.

- Add a `probe_chrome_version(path: &Path) -> String` helper in `chrome_pdf.rs` — `Command::new(path).arg("--version").output()` with 3s `timeout` would require `std::process::Command` which has no native timeout on stable Rust; use a thread + channel with `recv_timeout` pattern, or simply accept that `--version` exits fast on all Chrome versions and use `.output()` directly (Chrome `--version` exits in <50ms in practice)
- Thread `chrome_binary_version` through `find_chrome()` return type (change to `(PathBuf, String)`) or add a separate call at the render site
- _Unit test:_ `chrome_version_probe_returns_nonempty_string` — `#[ignore]` when no Chrome present; verifies `probe_chrome_version()` returns a non-empty string when Chrome is available

---

### Sprint 8.2 — Intake Triage: Stage 0 (G-013, G-016) (~2–3 days)

Explicitly deferred from Phase 7, now needed for real production use and as a prerequisite for Phase 9 (drawing sheets). This adds the pre-Lexer normalization stage.

**8.2.A — `IntakeBundle` contract type** (~60 lines in intake.rs)
- Define `IntakeBundle { files: Vec<IntakeFile>, declared_order: Option<Vec<String>> }` and `IntakeFile { path, role: IntakeRole }`
- `IntakeRole`: `OriginalSpec | AddendumSpec | DrawingSet | Unknown`
- `NormalizedIntakeBundle` output type with resolved, rotation-corrected file paths and page audit manifest
- `IntakeIssue` type: `BlankPage`, `CorruptPage`, `RotatedPage { degrees }`, `UnknownMedium`
- Serde round-trip tests

**8.2.B — Multi-input update to `WorkflowRequest`** (~30 lines in contracts)
- Add optional `intake_bundle: Option<IntakeBundle>` to `WorkflowRequest`
- Keep `input_path` as the single-file backward-compat path
- Tests: existing contract round-trips still pass

**8.2.C — Page rotation detection** (~80 lines in `crates/engine/src/intake.rs` new)
- Read PDFium page `/Rotate` attribute per page
- Classify: needs rotation if `degrees` ∉ {0, 360}
- Emit `IntakeIssue::RotatedPage { page_index, degrees }` per affected page
- Unit tests: 4 tests (0°, 90°, 180°, 270° pages)

**8.2.D — `lopdf` rotation normalization** (~60 lines, extends `crates/engine/src/intake.rs`)
- For each rotated page: use `lopdf` to write or clear the `/Rotate` dictionary entry
- Uses existing `lopdf 0.40.0` workspace dependency (already present from Phase 6)
- Dry-run mode: emit issues without writing
- Unit tests: rotation corrected, output page count unchanged, unchanged pages byte-identical

**8.2.E — `intake` CLI subcommand** (~50 lines in backend-cli)
- `intake --input <file-or-dir> --output <normalized-bundle.json> [--dry-run]`
- Wraps `Stage0Normalizer::normalize(&bundle) -> NormalizedIntakeBundle`
- Emits `OperationStarted`/`OperationEnded` audit events
- Integration tests: 3 tests (directory scan, dry-run, rotation detected)

---

### Sprint 8.3 — Torture Corpus Validation Infrastructure (~1 day)

Addresses the "torture corpus passes ≥95%" DoD item, which is the primary Beta gate.

**8.3.A — Pipeline-level corpus validation in `pattern-dev`** (~100 lines in pattern_dev.rs)
- Add `validate-corpus --pipeline segment|parse|apply-addendum` mode
- For `segment`: run full `SegmentEngine::build_index()` on each spec fixture; measure section count, coverage ratio, missing footer count
- For `parse`: run `parse_section()` on every section in every spec fixture; measure unclassified rate, node distribution
- Emit `corpus-report.json` with per-fixture pass/fail and aggregate stats
- Target: ≥95% pass rate (section coverage ≥90% per fixture, unclassified ≤1%)

**8.3.B — Pass/fail criteria definitions** (~30 lines config/constants)
- `CORPUS_MIN_COVERAGE = 0.90` — fraction of pages with a detected section footer
- `CORPUS_MAX_UNCLASSIFIED = 0.01` — fraction of AST nodes that are `Unclassified`
- `CORPUS_MIN_SECTION_COUNT = 1` — at least one section per spec fixture
- These become the CI gate thresholds

**8.3.C — Run full corpus and establish baseline**
- Execute `validate-corpus --pipeline parse --tier 1` against all 27 Tier 1 fixtures
- Record baseline metrics per fixture in `audit_output/phase-h-corpus-baseline/`
- Document any fixtures below threshold as known exceptions with root-cause

**8.3.D — Determinism regression test** (~60 lines in `cli_integration_test.rs`)

The project's #1 non-negotiable — "same input + same rules = identical output" — has no automated validation. This test closes that gap without requiring Chrome.

- `apply_addendum_dry_run_is_deterministic` — run `apply-addendum --dry-run` twice consecutively on the same SPEC corpus fixture with an identical `AddendumManifest`; parse both `AddendumResult` JSONs from stdout; assert:
  - `section_results` arrays are the same length and `section_id` values appear in the same order on both runs
  - All `status` fields match between runs
  - All `DiagnosticEvent` `ParseDiagnostic.node_count` and every `NodeDistribution` field match between runs (node counts are the most likely output to exhibit non-determinism from HashMap iteration order or floating-point sort instability)
  - Timestamps, `elapsed_ms`, and any path strings are explicitly excluded from the comparison
- This test runs in normal CI (no Chrome required — `--dry-run` skips render and stitch)
- If it fails, the mismatched `NodeDistribution` fields in the `diagnostics.jsonl` output will identify exactly which section and which parse stage diverged

**8.3.E — Unchanged-page content hash validation** (~50 lines in `stitch.rs` + stitch test)

`validate_unchanged_present()` currently checks `doc.objects.contains_key(id)` — presence only. This task adds a content-identity check that makes Non-Negotiable #12 actually enforceable.

- Before `splice_page_tree()` mutates the document, collect `before_hashes: HashMap<ObjectId, u64>` — compute a simple FNV-1a hash over the `lopdf` object's bytes for each object ID in the set of pages that will NOT be replaced (i.e., `orig_page_ids[..del_start]` and `orig_page_ids[del_end+1..]`)
- After save/return, reload those same object IDs from the result document and recompute hashes
- On mismatch: append `"page object {id:?} content changed unexpectedly (before: {before:#016x}, after: {after:#016x})"` to the `warnings` vec
- Implement FNV-1a as a 6-line inline accumulator rather than adding a crate dependency — FNV-1a is trivial and this is the only use site
- _Unit test:_ `stitch_unchanged_pages_content_hash_unchanged` — use the existing `build_test_pdf_bytes()` helper, perform a valid single-section stitch, assert `result.warnings` is empty (implying no content hash mismatches)

**8.3.F — Multi-section stitch page-growth regression test** (~40 lines in `stitch.rs` unit tests)

The last-to-first ordering guarantee prevents page-index invalidation when multiple sections of differing sizes are stitched sequentially. This invariant is documented in comments but not tested against a case where sections grow beyond their original page count.

- `stitch_two_sections_with_page_growth_preserves_middle_section` — using `build_test_pdf_bytes()`, construct a 9-page "three-section" PDF (A: pages 0–2, B: pages 3–5, C: pages 6–8) with a `SegmentIndex` reflecting these ranges; stitch C with a 4-page replacement (net +1); then stitch A with a 5-page replacement (net +2); after both stitches:
  - Total pages = 5 + 3 + 4 = 12 (not 9)
  - The object IDs corresponding to original B pages (positions 3–5 in the original document) are still present in the output document
  - No content-hash warnings (B's page objects are unchanged)
- This directly validates the last-to-first ordering guarantee under nontrivial page delta conditions at both ends of the document

---

### Sprint 8.4 — Performance Benchmarking (~0.5 day)

**8.4.A — Benchmark apply-addendum on large spec** (~30 lines in a new bench-like integration test)
- Add `#[ignore]` benchmark test: `apply_addendum_benchmark_large_spec` that uses the 571-page corpus fixture
- Measures: extraction time, segment time, parse time for 3 sections, stitch time
- Assert total < 10,000 ms (10 sec) for a 3-section dry-run patch
- Document baseline in `audit_output/phase-h-perf/`

**8.4.B — Profile bottlenecks (identify, document action items)**
- Run the benchmark with a simple timing wrapper in `SpecsPatchOrchestrator::run()`
- Likely bottleneck: PDFium extraction of the full transcript (all 571 pages are extracted even for a 1-section patch)
- Action item if needed: lazy extraction — only extract the pages in the target section range rather than the full document
- This is a possible optimization only if baseline fails the <10s target
- **Architecture constraint documentation (required regardless of benchmark result):** Whether or not lazy extraction is needed to meet the <10s performance target, document the full-extraction architecture as a known design constraint in `ARCHITECTURE.md` under a new "Known Design Constraints" section. Entry must state: the full-document extraction pattern, the O(n_pages) cost model, the affected call site (`Extractor::extract()` in `SpecsPatchOrchestrator::run()`), and the Phase 9+ mitigation path (range-bounded extraction that materializes only the pages within `start_page..=end_page` for the target sections). Phase 9 drawing-sheet batch operations will make costly planning errors if this constraint is invisible in the architecture documentation.

---

### Sprint 8.5 — Metrics Output in Audit Bundle (~0.5 day)

Addresses the "metrics dashboard (confidence, coverage, failures)" deliverable.

**8.5.A — `metrics.json` in apply-addendum audit bundle** (~60 lines in handler)
- Add `metrics.json` written to `--audit-bundle` dir alongside `change-report.json` and `diagnostics.jsonl`
- Fields: `total_pages_input`, `total_pages_output`, `sections_detected`, `section_coverage_ratio`, `per_section: [{ section_id, parse_node_count, unclassified_count, unclassified_ratio, render_ms_or_null, stitch_ms }]`, `total_elapsed_ms`
- `metrics.json` is the **executive summary** roll-up — aggregate numbers suitable for a quick pass/fail read or graphing over time. The raw event-level detail lives in `diagnostics.jsonl` (Sprint 8.1.G); `metrics.json` is derived from it, not a replacement.
- Tests: 1 test verifying `metrics.json` written and contains required fields; 1 test verifying `per_section` entries count matches `sections_detected`

**8.5.B — Segment coverage field in SegmentIndex output** (~20 lines in segment.rs)
- Add `coverage_ratio: f64` and `pages_missing_footer: usize` to `SegmentIndex` struct
- Populated by `SegmentEngine::build_index()` — already computed internally, just not surfaced
- Tests: existing segment tests verify new fields present

---

### Sprint 8.6 — Pattern Database as Versioned JSON (~1 day)

Phase 7 plan explicitly called this out as Phase 8 polish.

**8.6.A — Pattern JSON format definition** (~40 lines in `crates/engine/src/patterns/`)
- Define `PatternDatabase { version: String, patterns: HashMap<FamilyId, PatternSpec> }`
- `PatternSpec { regex: String, confidence_threshold: f64, band: RegionBand, examples: Vec<String> }`
- Matches existing `PatternSpec` shape in pattern_model.rs — reuse the existing type
- Default embedded database: load `patterns/default.json` via `include_str!()` at compile time

**8.6.B — Extract current hardcoded patterns to `crates/engine/src/patterns/default.json`**
- `footer-section-id`: current regex `\b\d{2}\s+\d{2}(?:\s+\d{2})?\b`, threshold 0.95
- `page-counter`: current regex `(?i)\bpage\s+(\d+)\s+of\s+(\d+)\b`, threshold 0.98
- `header-band`: geometric family, top 15% of page

**8.6.C — Load and validate at startup** (~20 lines)
- `PatternDatabase::load_default()` — parses embedded JSON, returns `Result`
- Called once in `SegmentEngine::new()` — fail fast if pattern file is malformed
- Unit test: `default_pattern_database_parses_successfully`

**8.6.D — Version lock in audit bundle** (~10 lines)
- Include `pattern_db_version` field in `change-report.json`
- Tests: verify version field present in audit output

---

### Sprint 8.7 — User Documentation (~1 day)

**8.7.A — CLI reference** (new `docs/CLI_REFERENCE.md`)
All 10 subcommands documented with: description, required/optional args, output format, exit codes, example invocation. Focus on `apply-addendum` as the primary workflow.

**8.7.B — `apply-addendum` workflow tutorial** (new `docs/WORKFLOW_APPLYADDENDUM.md`)
End-to-end example from real spec book through addendum manifest authoring to result PDF and audit bundle inspection. Includes `AddendumManifest` JSON reference.

**8.7.C — Error codes and resolution guide** (section in CLI_REFERENCE.md or standalone)
Top 10 most likely errors from each stage with: what it means, most likely cause, what to check.

---

### Definition of Done for Phase 8

Per MASTER_PLAN:

| Criterion | Target | Status |
|---|---|---|
| Torture corpus pass rate | ≥95% on Tier 1 spec fixtures | ⏳ 8.3 |
| Crash on malformed PDF | Zero — every reachable error path returns `Result` | ✅ 8.1.A/D |
| Performance (typical doc) | <10 sec for 3-section dry-run on 571-page spec | ⏳ 8.4 |
| Error messages | Every error is actionable (stage + cause + what to check) | ✅ 8.1.B |
| Structured diagnostics | `diagnostics.jsonl` in every audit bundle; all 6 stage variants populated; unclassified nodes traced with text snippets; Chrome stderr tail captured on render failure | ✅ 8.1.E/F/G/I |
| Pattern database | Versioned JSON, version locked in audit bundle | ⏳ 8.6 |
| Intake triage | G-013/G-016 closed: rotation normalization + `IntakeBundle` contract | ⏳ 8.2 |
| Metrics | `metrics.json` executive summary roll-up in every audit bundle | ⏳ 8.5 |
| User docs | CLI reference + workflow tutorial published | ⏳ 8.7 |
| Determinism | Full pipeline dry-run produces identical `AddendumResult` on two consecutive runs with same input (timestamps and `elapsed_ms` excluded from comparison) | ⏳ 8.3.D |
| Unchanged-page contract | Stitch content-hash check: unchanged page object bytes are identical before and after stitching; zero hash-mismatch warnings on all corpus fixtures | ⏳ 8.3.E |
| Resource management | Temp directories cleaned up on all exit paths (success, early error, partial stitch failure, panic); no orphaned `specs_patch_*/` entries accumulate in `$TMPDIR` | ✅ 8.1.H |
| Full-extraction constraint | Documented in `ARCHITECTURE.md` "Known Design Constraints" section with Phase 9 lazy-extraction migration path | ⏳ 8.4.B |

---

### Phase 8 Dependency Graph

```
Sprint 8.0 — doc housekeeping ✅ COMPLETE
    ↓
Sprint 8.1 — crash containment + error hardening + structured diagnostics ✅ COMPLETE
           (8.1.A–D: panics/errors/memory ✅; 8.1.E–G: DiagnosticEvent IR + wiring + JSONL ✅;
            8.1.H: temp RAII cleanup ✅ [Phase 9 prerequisite]; 8.1.I: Chrome version capture ✅)
    ← parallel with →
Sprint 8.2 — intake triage  ⬜ PENDING
    ↓
Sprint 8.3 — corpus validation + structural contract tests  ⬜ NEXT
           (8.3.A–C: corpus infra + baseline;
            8.3.D: determinism regression [Beta prerequisite];
            8.3.E: unchanged-page hash [Beta prerequisite];
            8.3.F: multi-section page-growth regression)
    ↓
Sprint 8.4 — performance benchmarking + architecture constraint documentation
           (8.4.B arch doc: required before Phase 9 design begins)
    ↓
Sprints 8.5, 8.6, 8.7 — metrics roll-up, pattern DB, docs (parallel)
    ↓
Phase 8 DoD gate check
    ↓
Phase 9: Drawing Sheet Management
```

Sprint 8.1 and 8.2 can run in parallel. Sprints 8.5–8.7 can all run in parallel after 8.3 confirms the engine is stable. Note 8.1.E (the `DiagnosticEvent` IR type) must land before 8.5.A (`metrics.json`) because metrics are derived from diagnostic data — but 8.1.E is part of Sprint 8.1 so this dependency is automatically satisfied.

**Phase 9 prerequisites within Phase 8** (from the Foundation Assessment):
- **8.1.H** (temp RAII cleanup) — must land before any Phase 9 batch-mode code exists
- **8.3.D** (determinism regression) — must pass before Beta is declared
- **8.3.E** (unchanged-page hash) — must pass before Beta is declared
- **8.4.B** (full-extraction constraint doc) — must be written before Phase 9 architecture design begins

---

### Recommended Attack Order

1. ~~**Start with 8.0** (30 min) — close the doc hygiene item, bump versions~~ **✅ DONE**
2. ~~**8.1 and 8.2 in parallel** — error hardening and intake triage are independent tracks. Within 8.1, prioritize: 8.1.H first (temp cleanup — low effort, high consequence if missed), then 8.1.A–D (panic removal, error messages, memory cap), then 8.1.E–G (diagnostic types and wiring), then 8.1.I (Chrome version). The diagnostic work is easier once the error paths are clean.~~ **✅ 8.1 DONE** (8.2 still pending)
3. **8.3 after 8.1** — corpus validation against the hardened engine establishes the Beta quality bar. `diagnostics.jsonl` from Sprint 8.1.G becomes the primary triage tool when a fixture fails. Run 8.3.D (determinism) and 8.3.E (unchanged-page hash) early in this sprint — they will fail fast on any regression introduced during 8.1 changes. **← CURRENT**
4. **8.4 after 8.3** — benchmark only once the engine is stable. Regardless of benchmark results, write the full-extraction architecture constraint doc (8.4.B) before Phase 9 planning begins.
5. **8.5, 8.6, 8.7 in parallel** — `metrics.json` (8.5) derives from the diagnostic event data already populated by 8.1.G, so it is straightforward to implement at this point. Pattern DB (8.6) and docs (8.7) have no ordering constraints between them or relative to 8.5.

Phase 9 (Drawing Sheet Management) can begin as soon as 8.2 is done, since intake triage is the primary prerequisite for drawing addenda workflows. The rest of Phase 8 hardening can continue in parallel with Phase 9 planning if timeline pressure exists — but 8.1.H and 8.4.B must be complete before any Phase 9 batch architecture decisions are made.