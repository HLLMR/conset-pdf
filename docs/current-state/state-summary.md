# Current State Summary

**Version:** 2.10.0
**Date:** April 11, 2026
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

## Scope

This file summarizes where Conset PDF stands now, what is complete, and what is next.

## Current Baseline

- Phase 0 foundation is treated as complete for its intended scope.
- Documentation governance baseline is implemented:
  - authority order defined,
  - mandatory status tags enforced,
  - non-authoritative claim rule established.
- Canonical entry points are explicitly listed in the documentation index.

## Complete Now

- Cargo workspace and crate structure are present.
- IR, extraction, engine, and audit crates exist and are wired in the workspace.
- Foundational test suites exist across core crates.
- Phase A governance policy is active and tagged across docs.
- Phase B: current-state library hardened with evidence-linked assertions.
- Phase C: full repo doc triage complete — deprecated v3 merge-target docs retired; merge-target count = 0 (see `../DOC_GOVERNANCE.md` canonical phase closure snapshot).
- Phase D: selective postmortem migration complete — M-001 through M-015 (including d-ext cycle), SLM-001 through SLM-010 all closed; migrated constraints are reflected in canonical docs (see `../DOC_GOVERNANCE.md` and `../ARCHITECTURE.md`).
- Phase E: Gap Register complete — 39 gaps enumerated (G-001–G-039), with CRITICAL path gaps on the PDF→IR chain plus autonomous ROI, intake normalization, vector-first adaptive detection, assisted-intelligence/OCR/schedule-data, and operational trust/knowledge-layer tracks. See `gap-register.md`.
- Phase F: Alignment Audit complete — capability-matrix updated to reflect actual code state; critical stubs and missing wiring documented with canonical source mismatches; R-005 raised.
- Phase 0.5 implementation planning consolidated into an explicit priority stack: Band 0 runtime + Band 0.5 contract-shaping in `phase05Implementation.md`; later runtime bands deferred by design to prevent scope bleed.
- Phase 0.5 Phase B: `tools` crate restructured as multi-binary host (`classify-pdf` + `pattern-dev`); shared `tools/src/` module directory introduced; both binaries build and test-pass (9/9 unit tests).
- Phase 0.5 Phase C: `PdfiumExtractor::extract_page()` implemented with real pdfium-render calls; returns `PageData` with `Vec<SpanData>`, `RawBBox`, and page dimensions. G-004 closed.
- Phase 0.5 Phase D: Shared pattern model (`tools/src/pattern_model.rs`) defines `PatternSpec`, `HeuristicFamily` (6 variants), `RegionBand`, `NormalizedBBox`, `MatchEvidence`, `MatchedSpan`, `FailureCode`, `SourceTag` with locked sidecar schema version `0.5.0`.
- Phase 0.5 Phase E: `pattern-dev test-pattern` loop implemented with real per-page detection via PDFium `FPDFText` pipeline (transparent Form XObject descent); `footer-section-id` validated at 556/571 PASS on a 571-page AEC spec set. `validate-corpus --dry-run` confirms 27-fixture Tier 1 inventory.
- Phase 0.5 Phase F: Deterministic overlay PNGs (green/amber/red by confidence band) and per-page sidecar provenance fields complete. Schema version `0.5.0` locked.
- Phase 0.5 Phase G: Schema-complete sidecars for `title-block-anchor` (with partial runtime corner detector), `roi-candidate`, `spec-heading` (source=schema-only). 16/16 unit tests.
- Phase 0.5 Phase H: Six contract modules in `crates/contracts/src/` defining all downstream contract shapes. 24/24 unit tests. No runtime wiring — contract-only by design.
- Phase 0.5 Phase I: `validate-corpus` real detection loop implemented. Two confirmed runs: 27 Tier 1 fixtures, 2,892 pages, 0 errors, 0 determinism regressions. Artifacts in `audit_output/phase-i-smoke/`.
- Phase 0.5 Phase J: 32 unit tests + 5 integration tests added for `fnv1a_64`, `FamilyPageCounts`, `score_matches`, artifact naming, manifest schema, dry-run semantics, holdout rejection, golden-path sidecar, and byte-determinism. All passing; clippy clean.
- Phase 0.5 Phase K: Downstream handoff document published at `docs/PHASE_05_HANDOFF.md`. Covers `pattern-dev` workflow, locked sidecar schema, extended evidence payloads, all six contract boundaries, runtime-ready vs. contract-only surface map, and priority band assignments for gaps G-001–G-039.
- Phase 0.5 Phase L: Final verification complete. Definition-of-done confirmed: both binaries build and clippy-clean; workspace-wide `cargo test` exits 0; `classify-pdf` and `pattern-dev` smoke-tested on representative Tier 1 fixtures; PASS/FAIL sidecars carry all required v0.5.0 fields; schema-only families emit `source=schema-only`; `validate-corpus --dry-run` lists 27 fixtures and writes no files; all contract modules carry `**CONTRACT-ONLY**` markers. **Phase 0.5 is CLOSED.**
- Phase 1 Band 0: G-001, G-002, G-005, G-007, G-009 all closed. Real PDFium extraction wired in `crates/engine/src/pipeline/extraction.rs` (`SpanData`→`BoundingBox`→`normalize_bbox()`→`BBox`→`Span`); `validate_transcript()` called in `pipeline/parsing.rs`; `Validator::validate()` delegates to `validate_transcript()`; extract handler writes transcript JSON to `--output`; `visualize` subcommand added to `backend-cli` (`crates/engine/src/visualize.rs`, `apps/backend-cli/src/handlers/visualize.rs`); E2E + API tests enabled and passing (9/9).
- Phase 1 complete: 8/8 CLI integration tests pass across SPEC, DWG, NAR, SUB, and simple fixtures; coordinate system verified (no axis inversion); `source_path` canonicalized to absolute path at extraction time. Evidence: `apps/backend-cli/tests/cli_integration_test.rs`.
- Phase 2 complete: `SegmentIndex` IR types (`crates/ir/src/segment.rs`), CSI footer-oracle segmentation engine (`crates/engine/src/segment.rs`), `Segment` and `VisualizeSegments` operations wired in `backend-cli`; 5/5 Phase 2 integration tests pass (segment simple, segment spec, segment dry-run, visualize-segments round-trip, visualize-segments dry-run). Evidence: `apps/backend-cli/tests/cli_integration_test.rs`.
- Phase 3 complete: `ParsedDocument` AST IR types (`crates/ir/src/ast.rs`); line-clustering + CSI 3-part outline tree parser (`crates/engine/src/parse.rs`); HTML AST visualizer (`crates/engine/src/visualize_ast.rs`); `Parse` and `VisualizeAst` operations wired in `backend-cli`; 6/6 Phase 3 integration tests pass (parse dry-run, visualize-ast dry-run, parse simple, parse spec, parse spec with section filter, parse+visualize-ast round-trip). Evidence: `apps/backend-cli/tests/cli_integration_test.rs`.
- Phase 3 parser hardening sprint complete (April 5, 2026): span x-sort, LINE_Y_EPSILON 0.005→0.012, cluster-based section ID detection (16→89 sections), FOOTER_Y 0.85→0.90, noise-only line skipping, article_re major≥1 + uppercase-title guard, `inject_missing_parts` recovery pass. Post-hardening: 7,971 nodes, 0.2% unclassified, 0/70 wrong-PART sections, 19/19 integration tests passing. See `CHANGELOG.md` for full detail.
- Phase 4 complete (April 5, 2026): `SectionEditor` edit engine in `crates/engine/src/edit.rs` (27/27 unit tests); `crates/ir/src/edit.rs` IR types (8/8 unit tests); `edit` CLI subcommand in `apps/backend-cli`; `WorkflowOperation::Edit` contract variant; all 7 CLI handlers confirmed emitting `OperationStarted`/`OperationEnded` audit events (G-006 closed); G-003 accepted-closed; 26/26 integration tests pass. See `CHANGELOG.md` for full detail.

- Phase 5 complete (April 6, 2026): Section regeneration pipeline implemented. `crates/ir/src/render.rs` — `SpecChromeMetadata`, `RenderConfig`, `PageSize`, `RenderResult`, `RenderError` IR types; `crates/engine/src/render/` — `body.rs` (AST→HTML fragment, 7 OutlineTag CSS classes), `chrome.rs` (full HTML with CSS `@page` margin-box rules for running headers/footers), `chrome_pdf.rs` (Chrome subprocess renderer with `CHROME_PATH`/system-path discovery), `mod.rs` (`SectionRenderer` public API + `dry_run()`); `regenerate` CLI subcommand with `--ast`, `--chrome-metadata`, `--output`, `--section`, `--dry-run`, `--font`, `--font-size`; `WorkflowOperation::Regenerate` contract variant; G-008 accepted-closed (no defined consumer); G-010 closed (4 behavioral audit tests added). 30/30 integration tests pass (4 Phase 5 non-Chrome tests added, 1 Chrome test ignored). See `CHANGELOG.md`.

- Phase 5 supplement: Layout geometry capture (April 6, 2026): `SectionLayout` struct added to `crates/ir/src/ast.rs` with `body_left`, `body_right`, `font_size_pt`, `line_gap_norm` fields; `x_indent: f64` added to `AstNode`; both fields are `#[serde(default)]` for backward compatibility. `parse.rs` now populates these from raw span geometry during `parse_section()` via `compute_section_layout()` and `median_val()` helpers; `cluster_lines()` returns x_min as a third tuple element; `classify_lines()` and `build_tree()` propagate it through to `AstNode`. `build_body_html()` and `build_full_html()` updated to accept `Option<&SectionLayout>` and use measured values for inline CSS `margin-left`, `font-size`, and `line-height` when layout data is present. Render pipeline threads `ast.layout.as_ref()` through both builders. All 30/30 integration + 43/43 engine unit + 13/13 IR unit tests pass. See `CHANGELOG.md`.
- Phase 5.5 supplement: Font typography extraction (April 6, 2026): `SpanData` extended with `font_weight: f32` (from `PdfFontWeight` enum via `text_obj.font().weight()`) and `is_italic: bool` (from `text_obj.font().is_italic()`); both wired into `Span` IR type (`font_weight` was previously hardcoded to 400, `is_italic` is a new field with `#[serde(default)]`). `SectionLayout` gains `body_font_name: String` — the modal (most-frequent) font family name among body spans, computed by new `modal_font_name()` helper; `#[serde(default)]` defaults to `"Unknown"`. `build_full_html()` now uses `body_font_name` as the CSS `font-family` when it is a real name, falling back to `config.font_family`. All 30/30 integration + 43/43 engine unit + 13/13 IR unit tests pass. See `CHANGELOG.md`.

- Phase 6 complete (April 7, 2026): PDF stitching pipeline implemented. `crates/ir/src/stitch.rs` — `StitchPlan`, `StitchResult`, `StitchError` IR types. `crates/engine/src/stitch.rs` — `PdfStitcher::stitch()` using lopdf 0.40.0: loads original + replacement PDFs, resolves section page range from `SegmentIndex`, renumbers replacement objects, splices `/Pages` root `/Kids` array, fixes up `/Parent` refs, removes deleted page objects, re-routes outline `/Dest` bookmarks, validates unchanged pages, writes output. `WorkflowOperation::Stitch` contract variant. `stitch` CLI subcommand in `apps/backend-cli` (`--input`, `--segment-index`, `--section`, `--replacement`, `--output`, `--dry-run`). 6 Phase 6 integration tests pass. 36/36 integration + 53/53 engine unit + 13/13 IR unit tests pass. See `CHANGELOG.md`.

## Next Focus

- **Phase 10 (Submittal Data Extraction)** can begin — all Phase 9 DoD rows are satisfied.
  - **Submittals** — Extract structured data from equipment submittals: unit boundary detection, table extraction (performance specs), key-value extraction (tags, models), CSV/JSON export.
  - **See:** `docs/MASTER_PLAN.md` Phase 10 section for the full deliverable list.


  - **IR types:** `crates/ir/src/addendum.rs` — `AddendumManifest`, `SectionEditSpec`, `SectionPatchStatus`, `SectionPatchResult`, `AddendumResult`; 12 new unit tests.
  - **Orchestrator:** `crates/engine/src/specs_patch.rs` — `SpecsPatchOrchestrator::run()` — Extract → Segment → Parse → Edit → Render → Stitch (last-to-first); partial-success semantics; chrome metadata merge (base + manifest override + per-section override); 5 unit tests.
  - **Handler:** `apps/backend-cli/src/handlers/apply_addendum.rs` — loads addendum manifest, delegates to orchestrator, writes audit bundle.
  - **CLI:** `apply-addendum` subcommand — `--original`, `--addendum`, `--output` (optional), `--audit-bundle` (optional), `--dry-run`.
  - **Integration tests:** 5 non-Chrome tests pass (dry-run, missing-original, missing-manifest, unknown-section continues, audit artifacts); 1 Chrome test `#[ignore]`.
  - **Test baseline:** 41/41 CLI integration (36 pre-Phase-7 + 5 Phase-7), **77/77** engine unit (65 pre-Phase-7 + 5 specs_patch + 7 prior), 30/30 IR unit. See `CHANGELOG.md`.

- **Sprint 8.1 COMPLETE (April 7, 2026):** Crash containment, error hardening, structured diagnostics pipeline.
  - Panic audit: zero panics reachable from valid-but-corrupt PDF input (`unwrap()`/`expect()` replaced with `Result` propagation).
  - `MAX_PDF_PAGES = 2_000` page cap with `EngineError::PdfTooLarge { page_count, max }`.
  - Actionable error messages audit for `StitchError`, `ParseError`, `SectionPatchResult::Failed`.
  - `DiagnosticEvent` IR type (`crates/ir/src/diagnostics.rs`) with 6 stage variants: `ExtractionDiagnostic`, `SegmentationDiagnostic`, `ParseDiagnostic` (with `UnclassifiedNodeTrace`), `EditDiagnostic`, `RenderDiagnostic` (with `RenderOutcome`), `StitchDiagnostic`; all `#[non_exhaustive]`; added to `AddendumResult.diagnostics: Vec<DiagnosticEvent>`.
  - Per-stage diagnostic emission wired into `SpecsPatchOrchestrator::run()`.
  - `diagnostics.jsonl` written to `--audit-bundle` dir (newline-delimited JSON, schema header line, 8 MB cap).
  - Temp directory RAII cleanup: `tempfile::TempDir` replaces bare `PathBuf` in orchestrator.
  - Chrome binary version captured: `probe_chrome_version()` in `render/chrome_pdf.rs`; stored in `RenderDiagnostic.chrome_binary_version`.
  - **Test delta:** +6 CLI integration; +9 engine unit (+1 ignored); +7 IR serde round-trips.

- **Sprint 8.2 COMPLETE (April 8, 2026):** Intake triage Stage 0 (closes G-013, G-016).
  - `IntakeBundle`, `IntakeFile`, `IntakeRole`, `NormalizedIntakeBundle`, `IntakeIssue` contract types in `crates/contracts/src/intake.rs`.
  - `WorkflowOperation::Intake` added; `intake_bundle: Option<IntakeBundle>` added to `WorkflowRequest` (`#[serde(default)]`).
  - `Stage0Normalizer` in `crates/engine/src/intake.rs` — rotation detection (PDFium `/Rotate`) + lopdf in-place normalization.
  - `intake` CLI subcommand with `--input`, `--output`, `--dry-run`.
  - **Test delta:** +2 contracts; +7 engine; +3 CLI integration.

- **Sprint 8.3 COMPLETE (April 8, 2026):** Corpus validation infrastructure + structural contracts.
  - `--pipeline segment|parse` mode added to `pattern-dev validate-corpus`; pass/fail constants: `CORPUS_MIN_COVERAGE=0.90`, `CORPUS_MAX_UNCLASSIFIED=0.01`, `CORPUS_MIN_SECTION_COUNT=1`.
  - Corpus baseline run: `audit_output/phase-h-corpus-baseline/corpus-report.json`; 3/27 pass (SPEC_* pass; DWG/NAR/SUB fail by design — no CSI footer IDs). Primary fixture `SPEC_RWB_LHHS_ALL_ORG`: 89 sections, 96.5% coverage, 0.2% unclassified.
  - Determinism regression test: `apply_addendum_dry_run_is_deterministic` — two identical dry-run invocations, compares `change-report.json` (section_results, node_count, node_distribution; timestamps/elapsed excluded).
  - Unchanged-page content-hash validation: `snapshot_hashes()` + `validate_unchanged_content()` + FNV-1a-64 over `Debug` bytes in `stitch.rs`.
  - Multi-section page-growth regression test: 9-page three-section fixture, stitch C (+1) then A (+2), assert 12-page output with B-section objects intact.
  - **Test delta:** +1 CLI integration; +2 engine unit (`stitch.rs`).

- **Sprint 8.4 COMPLETE (April 8, 2026):** Performance benchmark + architecture constraint documentation.
  - `apply_addendum_benchmark_large_spec` `#[ignore]` integration test: SPEC_RWB_LHHS_ALL_ORG.pdf (571 pp); picks first/middle/last sections from segment index; runs dry-run; measures wall time via `Instant` + per-stage `elapsed_ms` from `diagnostics.jsonl`; asserts < 10,000 ms; writes `audit_output/phase-h-perf/benchmark-large-spec.json`.
  - `ARCHITECTURE.md` v4.9.0: new "Known Design Constraints" section — Constraint 1: full-document extraction per operation (O(n\_pages) cost model, affected call site, Phase 9 range-bounded mitigation path, 4-step decision checklist). ToC updated; Revision History extended.
  - **Test delta:** +1 CLI integration (`#[ignore]`).

- **Sprint 8.5 COMPLETE (April 8, 2026):** `metrics.json` executive summary roll-up in every audit bundle.
  - `build_metrics()` helper in `apply_addendum.rs` derives all values from `result.diagnostics` (no independent state); writes `metrics.json` (schema `"metrics/v1"`) alongside `change-report.json` + `diagnostics.jsonl`.
  - Fields: `total_pages_input`, `total_pages_output`, `sections_detected`, `sections_patched`, `section_coverage_ratio`, `total_elapsed_ms`, `per_section[]` (per-section node counts + timings).
  - 8.5.B pre-done: `SegmentIndex.coverage.coverage_ratio` + `pages_missing_footer` already populated.
  - **Test delta:** +2 CLI integration.

- **Sprint 8.6 COMPLETE (April 8, 2026):** Pattern database as versioned JSON.
  - `crates/engine/src/patterns/mod.rs`: `RegionBand`, `PatternSpec`, `PatternDatabase`, `PatternDatabase::load_default()` (embedded via `include_str!()`).
  - `crates/engine/src/patterns/default.json` (v1.0.0): three families — `footer-section-id` (bottom, 0.95), `page-counter` (bottom, 0.98), `header-band` (top, 0.0).
  - Fail-fast load at start of `SpecsPatchOrchestrator::run()`; `pattern_db_version: "1.0.0"` written to `change-report.json`.
  - **Test delta:** +1 engine unit (`default_pattern_database_parses_successfully`); +1 CLI integration (`cli_apply_addendum_change_report_contains_pattern_db_version`).

- **Sprint 8.7 COMPLETE (April 8, 2026):** User documentation — Phase 8 fully closed.
  - `docs/CLI_REFERENCE.md`: all 11 subcommands (args, output format, exit codes, examples); `apply-addendum` audit bundle contents table; error codes + resolution guide (`MISSING_MANIFEST_PATH`, `MANIFEST_READ_ERROR`, `EMPTY_MANIFEST`, `AUDIT_DIR_CREATE_ERROR`, `ORCHESTRATOR_ERROR`); per-section failure table by pipeline stage.
  - `docs/WORKFLOW_APPLYADDENDUM.md`: end-to-end tutorial from section inspection through manifest authoring, dry-run, production run, and audit bundle interpretation; full `AddendumManifest` / `SectionEditSpec` / `EditOperation` / `SpecChromeMetadata` JSON reference; multi-section + partial-success examples.
  - **No code changes** (documentation sprint only).

## Current Test Baseline (Phase 9 COMPLETE — April 11, 2026)

| Suite | Passing | Ignored |
|---|---|---|
| CLI integration | 46+ | 4 (2 Chrome + 1 spec benchmark + 1 drawing benchmark) |
| Engine unit | 122 | 3 (Chrome) |
| IR unit (drawing + others) | 42+ | — |
| standards-data unit | 33 | — |
| DWG corpus (`drawing-segment`) | ≥4/11 DWG tier-1 pass | (VLK fixtures fail coverage threshold; WRA run pending) |

## Phase 9: Drawing Sheet Management — COMPLETE

**Status:** ALL SPRINTS COMPLETE (9.0–9.5). One DoD criterion (row 8 — non-dry-run production test) remains pending; see gap note below.

**Sprint roster:**
- Sprint 9.0 — Foundation: extraction profiling + DWG corpus baseline + `DrawingIndex`/`SheetEntry` IR + discipline classifier ✅
- Sprint 9.1 — Sheet detection: title-block oracle + `DrawingSegmentEngine` + `index-drawing` CLI ✅
- Sprint 9.2 — Sheet replacement: `DrawingsPatchOrchestrator` + `apply-sheet-addendum` CLI ✅ (dry-run confirmed; non-dry-run production test ⬜ — see gap register)
- Sprint 9.3 — Sheet renaming detection + bookmark generation ✅
- Sprint 9.4 — Basic schedule extraction + `extract-schedules` CLI ✅
- Sprint 9.5 — Corpus hardening, docs, determinism test ✅

**Sprint 9.1 delivered:**
- `crates/engine/src/drawing_segment.rs` — `DrawingSegmentEngine` + `build_index()`, title-block keyword-anchor region detector, span-density sheet boundary algorithm, `SheetEntry` population (sheet_id, sheet_title, start_page, end_page, is_schedule_sheet, discipline)
- `crates/standards-data/src/aec.rs` — `classify_sheet()` called from `make_entry()` to set discipline
- `index-drawing` CLI subcommand — `--input <transcript.json>`, `--output <drawing-index.json>`, `--dry-run`
- Integration tests: `cli_index_drawing_on_dwg_fixture_produces_valid_json`, `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets`

**Sprint 9.2 delivered:**
- `crates/engine/src/drawings_patch.rs` — `DrawingsPatchOrchestrator::run()`: extract ORG + ADD transcripts → index → sort specs descending by page → stitch per sheet (multi-step working copy chain) → rename detection → bookmarks → write audit bundle
- `apply-sheet-addendum` CLI subcommand — `--manifest <DrawingAddendumManifest.json>`, `--output <patch-result.json>`, dry_run controlled by manifest field
- Integration tests: `cli_apply_sheet_addendum_dry_run_skips_all_sheets`, `cli_apply_sheet_addendum_bad_manifest_path_fails_gracefully`, `cli_apply_sheet_addendum_dry_run_writes_audit_bundle`

**Sprint 9.3 delivered:**
- `detect_renames()` + `normalize_title()` + `title_similarity()` in `drawings_patch.rs`; 4 unit tests
- `generate_drawing_bookmarks()` in `stitch.rs`; 2 unit tests
- `SheetRenameEvent` in `DrawingPatchResult.renames`; included in `change-report.json`

**Sprint 9.4 delivered:**
- `is_schedule_sheet_title()` in `drawing_segment.rs`; populates `SheetEntry.is_schedule_sheet`; 4 unit tests
- `crates/engine/src/drawing_tables.rs` (new) — `ExtractedTable`, `extract_tables_from_sheet()`; ROW_Y_EPSILON=0.015, COL_X_EPSILON=0.02; 5 unit tests
- `extract-schedules` CLI subcommand — `--input`, `--output`, `--format`, `--dry-run`
- Integration tests: `cli_extract_schedules_on_dwg_fixture_produces_json`, `cli_extract_schedules_dry_run_skips_extraction`

**Sprint 9.5 delivered:**
- `--pipeline drawing-segment` in `pattern-dev validate-corpus`; `DRAWING_MIN_SHEET_COUNT=1`, `DRAWING_MIN_SHEET_COVERAGE=0.80`; output `dwg-corpus-report.json`
- `docs/CLI_REFERENCE.md` drawing subcommands section + error codes
- `docs/WORKFLOW_APPLYSHEETADDENDUM.md` (new) — end-to-end tutorial
- `apply_sheet_addendum_dry_run_is_deterministic` integration test
- **DoD row 8 gap closure:** `cli_apply_sheet_addendum_production_run_writes_output_pdf` integration test added; `--output` arg added to `apply-sheet-addendum` subcommand; production-path (non-dry-run) confirmed on DWG_RWB_LHHS_ALL_ORG + ADD2 fixtures

**All Phase 9 DoD rows: ✅ 16/16**

---

## Phase 4 Execution Plan: Edit Operations

**Goal:** Working `SectionEditor` that applies insert/delete/replace to a `ParsedDocument` AST, renumbers affected siblings deterministically, exposes an `edit` CLI subcommand, wires `OperationStarted`/`OperationEnded` audit events (closes G-006), and has full integration test coverage.

### Key Design Decisions

**Node addressing:** `NodePath { section_id: String, markers: Vec<String> }` — vector of marker strings from the section root. Example: `{ section_id: "23 82 16", markers: ["PART 2", "2.7", "A."] }`. Mirrors how AEC practitioners describe edits verbally; unambiguous given the CSI level-marker grammar.

**Renumbering scheme** after insert/delete — re-derive all sibling markers at the affected level:
- Level 0 (Part): `PART 1`, `PART 2`, ...
- Level 1 (Article): `N.1`, `N.2`, ... (N = parent Part number)
- Level 2 (Paragraph): `A.`, `B.`, `C.`, ...
- Level 3 (SubParagraph): `1.`, `2.`, `3.`, ...
- Level 4 (SubSubParagraph): `a.`, `b.`, `c.`, ...
- Level 5 (SubSubSubParagraph): `1)`, `2)`, `3)`, ...

**Validation:** Pre-flight check on every `EditRequest` — section must exist, path must resolve, no renumbering overflows (>26 uppercase letters, etc.).

**Audit:** Wire `OperationStarted`/`OperationEnded` into all CLI handlers simultaneously (satisfies G-006 while making edit audit events consistent with extract/segment/parse).

### Micro-Task Checklist

- [x] **4.1 — IR: edit types** (`crates/ir/src/edit.rs`, ~65 lines)
  Define `NodePath`, `EditOperation { InsertAfter | Delete | Replace }`, `EditRequest`, `EditResult`, `EditError`. Export from `crates/ir/src/lib.rs`. Tests: round-trip serde for each type.

- [x] **4.2 — Engine: node resolution** (`crates/engine/src/edit.rs`, ~75 lines)
  `fn find_node<'a>(nodes: &'a [AstNode], markers: &[&str]) -> Option<&'a AstNode>` and mutable variant. Unit tests: found, not found, partial path, wrong section.

- [x] **4.3 — Engine: delete operation** (~60 lines)
  `fn apply_delete(ast: &mut SectionAst, path: &[&str]) -> Result<AstNode, EditError>` — removes target node, returns it. Unit tests: delete middle sibling, leaf node, root-level node.

- [x] **4.4 — Engine: renumbering** (~80 lines)
  `fn renumber_children(nodes: &mut Vec<AstNode>, level: u8)` — deterministically re-derives all sibling markers after structural change. Unit tests: renumber paragraphs after delete, renumber articles after insert, no-op when nothing changes.

- [x] **4.5 — Engine: replace operation** (~50 lines)
  `fn apply_replace(ast: &mut SectionAst, path: &[&str], new_content: AstNode) -> Result<(), EditError>` — replaces node content preserving position and marker. Unit tests: replace text, replace with children preserved.

- [x] **4.6 — Engine: insert_after operation** (~70 lines)
  `fn apply_insert_after(ast: &mut SectionAst, path: &[&str], new_node: AstNode) -> Result<(), EditError>` — inserts sibling after target, calls `renumber_children`. Unit tests: insert in middle, insert at end, verify downstream sibling markers updated.

- [x] **4.7 — Engine: SectionEditor public API** (~80 lines)
  `struct SectionEditor { doc: ParsedDocument }` with `fn apply(request: EditRequest) -> EditResult`. Validates full request pre-flight, dispatches ops in order, returns `EditResult` with applied ops and warnings. Unit tests: multi-op request, partial failure, empty request.

- [x] **4.8 — Contract + CLI wiring + G-006 audit** (~90 lines across 3 files)
  - Add `WorkflowOperation::Edit` variant to `crates/contracts/src/lib.rs`
  - Add `Edit` subcommand to `apps/backend-cli/src/main.rs` (`--input` ParsedDocument JSON, `--operations` JSON file, `--output`, `--dry-run`)
  - Create `apps/backend-cli/src/handlers/edit.rs` — drives `SectionEditor`, returns `WorkflowResponse`
  - Wire `OperationStarted`/`OperationEnded` audit events into all handlers (`mod.rs`) — closes G-006

- [x] **4.9 — Integration tests** (~130 lines appended to `apps/backend-cli/tests/cli_integration_test.rs`)
  - `cli_edit_insert_after_succeeds` — insert new paragraph, verify sibling renumbering
  - `cli_edit_delete_succeeds` — delete paragraph, verify count and renumbering
  - `cli_edit_replace_succeeds` — replace node text, marker unchanged
  - `cli_edit_renumber_cascades` — insert mid-list, all subsequent markers shifted
  - `cli_edit_invalid_section_fails` — error on unknown section_id
  - `cli_edit_invalid_path_fails` — error on path that doesn't resolve
  - `cli_edit_dry_run_succeeds_without_writing` — dry-run returns success, no file written

- [x] **4.10 — Gap register + docs update**
  - Close G-003 as Accepted in `gap-register.md` (backend-cli covers runtime path by design)
  - Close G-006 as Closed in `gap-register.md` (evidence: Task 4.8 handler wiring)
  - Update `capability-matrix.md` — Phase 4 edit operations row
  - Update `CHANGELOG.md` — Phase 4 milestone entry
  - Update this file's "Phase 4 micro-task checklist" checkboxes above
  - Update `state-summary.md` "Complete Now" section

### Phase 4 File Map

| File | Status | Task |
|------|--------|------|
| `crates/ir/src/edit.rs` | New | 4.1 |
| `crates/ir/src/lib.rs` | Edit | 4.1 (add edit re-exports) |
| `crates/engine/src/edit.rs` | New | 4.2–4.7 |
| `crates/engine/src/lib.rs` | Edit | 4.7 (export edit module) |
| `crates/contracts/src/lib.rs` | Edit | 4.8 (add `WorkflowOperation::Edit`) |
| `apps/backend-cli/src/main.rs` | Edit | 4.8 (add `Edit` subcommand) |
| `apps/backend-cli/src/handlers/edit.rs` | New | 4.8 |
| `apps/backend-cli/src/handlers/mod.rs` | Edit | 4.8 (G-006 audit wiring + edit dispatch) |
| `apps/backend-cli/tests/cli_integration_test.rs` | Edit | 4.9 (append Phase 4 tests) |
| `docs/current-state/gap-register.md` | Edit | 4.10 |
| `docs/current-state/capability-matrix.md` | Edit | 4.10 |
| `CHANGELOG.md` | Edit | 4.10 |

**Total scope:** ~700 lines of new/modified code across 10 tasks. Each task is self-contained and testable before moving to the next. Start with Task 4.1 (IR types) since all other tasks depend on those shapes being locked.

---

## Evidence-Linked Assertions

- Governance model implemented: see `../DOC_GOVERNANCE.md` and `../DOCUMENTATION_INDEX.md`.
- Phase 0 accepted baseline: see `../dev/PHASE_0_IMPLEMENTATION_PLAN_v4_2_1.md` and `../MASTER_PLAN.md`.
- Crate foundation present: see `../../crates/ir/`, `../../crates/engine/`, `../../crates/pdf-extraction/`, `../../crates/audit/`.

## Evidence Pointers

- Master plan: ../MASTER_PLAN.md
- Architecture: ../ARCHITECTURE.md
- Standards: ../DEV_STANDARDS.md
- Governance: ../DOC_GOVERNANCE.md
- Phase 0 plan: ../dev/PHASE_0_IMPLEMENTATION_PLAN_v4_2_1.md
