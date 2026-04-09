# Changelog

All notable changes to this project are documented in this file.

## [2026-04-11] Sprint 10.5 Complete: Corpus Validation + Determinism + Documentation

### Added

- **`tools/pattern_dev.rs`** — Sprint 10.5.A: `run_validate_corpus_submittal` function + `"submittal-extract"` dispatch arm:
  - `--pipeline submittal-extract` batch-validates all `SUB_*.pdf` tier-1 fixtures
  - Per fixture: `SubmittalSegmentEngine::build_index` → per-unit `extract_kv_pairs` + `extract_unit_tables` → `build_equipment_dataset`
  - Pass thresholds: `SUBMITTAL_MIN_UNIT_COUNT = 1`, `SUBMITTAL_MIN_RECORD_COUNT = 1`
  - Output: `sub-corpus-report.json` (schema: `schema_version`, `generated_at`, `pipeline`, `thresholds`, `aggregate`, `fixtures[]`)

- **`apps/backend-cli/tests/cli_integration_test.rs`** — Sprint 10.5.B: `extract_submittal_dry_run_is_deterministic` integration test:
  - Runs `extract` → `index-submittal` → `extract-submittal` twice on `SUB_Rsmd_TAS_AAON-RTU.pdf`
  - Asserts `schema_version`, `unit_count`, `record_count`, and per-unit `unit_tag` + `record_count` are identical across both runs

- **`docs/WORKFLOW_EXTRACTSUBMITTAL.md`** (new) — Sprint 10.5.C:
  - 5-step tutorial: extract → index-submittal → extract-submittal (JSON) → extract-submittal (CSV) → audit bundle
  - Complete `TidyRow` field reference (14 columns)
  - `EquipmentDataset` JSON schema reference block
  - Common issues table; link to `CLI_REFERENCE.md`

- **`docs/CLI_REFERENCE.md`** — Sprint 10.5.C: "Submittal subcommands (Phase 10)" section appended:
  - `index-submittal`: flags, output JSON schema, error codes (`MISSING_OUTPUT_PATH`, `INVALID_TRANSCRIPT`)
  - `extract-submittal`: flags (`--input`, `--index`, `--output`, `--format`, `--audit-bundle`, `--dry-run`), JSON + CSV schema, audit bundle contents, error codes (`MISSING_OUTPUT_PATH`, `MISSING_INDEX_PATH`, `INVALID_TRANSCRIPT`, `INVALID_INDEX`)

### Fixed

- **`crates/engine/src/drawing_segment.rs` line 126** — `||` → `&&` in title-block region filter:
  - Before: `.filter(|s| s.bbox.y > TITLE_BLOCK_BOTTOM_Y || s.bbox.x > TITLE_BLOCK_RIGHT_X)` — matched any span in the bottom OR right band, causing spec PDF footer text (e.g. `"IBC-2021"` matching `(?i)\b([A-Za-z]{1,3})-?(\d{3,4})\b`) to be treated as a sheet ID
  - After: `.filter(|s| s.bbox.y > TITLE_BLOCK_BOTTOM_Y && s.bbox.x > TITLE_BLOCK_RIGHT_X)` — restricts detection to the bottom-right corner only
  - Fixes regression: `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets` was returning 6 sheets instead of 0
  - Two unit tests updated:
    - `right_side_band_detected` → `right_band_only_not_in_title_block_region` (span at x=0.80, y=0.60 — right-band-only; now **not** detected)
    - `sheet_id_extracted_from_right_band` → `sheet_id_extracted_from_bottom_right_corner` (span moved to x=0.88, y=0.80 — both axes satisfied)

### Changed

- **`docs/MASTER_PLAN.md`** — Phase 10 deliverables all marked ✅; "Ready for customer validation (BETA COMPLETE)" ✅
- **`docs/current-state/gap-register.md`** — G-028: `Closed (Sprint 10.4)`; G-029: `Partially Closed (Sprint 10.4)`
- **`docs/current-state/capability-matrix.md`** — 4 new submittal rows (`index-submittal`, submittal KV extraction, submittal table extraction, `extract-submittal`); schedule parser + export rows updated; `tools/pattern-dev` row updated for Sprint 10.5

### Test Delta

| Suite | Before | After |
|---|---|---|
| CLI integration active | 51 (Sprint 10.4) | 67 (+1 drawing regression fixed, +1 determinism test) |
| CLI integration ignored | 5 | 5 (unchanged) |
| Engine unit (drawing_segment) | 19/19 | 19/19 (2 tests renamed; all pass) |

> Note: drawing regression fix changes net active test count from 66→67 (the previously-failing `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets` now passes correctly).

---

## [2026-04-08] Sprint 10.4 Complete: Tidy Export Pipeline + `extract-submittal` CLI

### Added

- **`crates/engine/src/submittal_export.rs`** (new) — Sprint 10.4.A:
  - `build_equipment_dataset(index, tables_by_unit, kv_by_unit) -> EquipmentDataset` — assembles per-unit KV pairs and table rows into canonical `TidyRow` records; skips cover units; computes per-unit `UnitSummary`; warns when no records or avg_confidence < 0.5
  - `dataset_to_json(dataset) -> String` — pretty-printed serde_json
  - `dataset_to_csv(dataset) -> String` — 14-column RFC 4180 CSV with proper quoting; columns: `packet_name`, `revision_id`, `item_tag`, `equipment_type`, `section`, `field`, `value_raw`, `value_num`, `unit`, `page`, `bbox`, `confidence`, `source`, `conflict_flags`
  - `parse_value_and_unit(raw) -> (Option<f64>, String)` — splits numeric value from trailing unit string
  - 10 unit tests: 4 parse\_value\_and\_unit, 4 build\_dataset, 2 CSV format

- **`crates/engine/src/lib.rs`** — `pub mod submittal_export;` + `pub use submittal_export::{build_equipment_dataset, dataset_to_csv, dataset_to_json};`

- **`crates/contracts/src/lib.rs`** — `ExtractSubmittal` variant added to `WorkflowOperation`

- **`apps/backend-cli/src/handlers/extract_submittal.rs`** (new) — reads transcript + SubmittalIndex; per-unit `extract_kv_pairs` + `extract_unit_tables`; `build_equipment_dataset`; writes JSON/CSV; optional audit bundle (`unit-report.json` + `metrics.json`). Error codes: `MISSING_OUTPUT_PATH`, `MISSING_INDEX_PATH`, `INVALID_TRANSCRIPT`, `INVALID_INDEX`

- **`apps/backend-cli/src/handlers/mod.rs`** — `pub mod extract_submittal;` + `ExtractSubmittal` dispatch arm

- **`apps/backend-cli/src/main.rs`** — `ExtractSubmittal { input, index, output, format, audit_bundle, dry_run }` subcommand

- **`apps/backend-cli/tests/cli_integration_test.rs`** — `run_extract_submittal()` helper + 3 tests: `cli_extract_submittal_on_sub_fixture_produces_json`, `cli_extract_submittal_csv_format_produces_tabular`, `cli_extract_submittal_dry_run_skips_extraction`

### Gaps Closed

- **G-028 CLOSED** — `EquipmentDataset` records carry `schema_version`, `page`, `bbox`, `confidence`, `source` provenance
- **G-029 PARTIALLY CLOSED** — JSON + CSV export; XML deferred

### Test Delta

| Suite | Before | After |
|---|---|---|
| Engine unit | 157 | 167 (+10 submittal_export) |
| CLI integration active | 48 | 51 (+3 extract-submittal) |

---

## [2026-04-08] Sprint 10.3 Complete: Performance-Spec Table Extraction

### Added

- **`crates/engine/src/submittal_tables.rs`** (new) — Sprint 10.3.A:
  - `extract_unit_tables(pages: &[&Page], unit: &UnitEntry) -> Vec<ExtractedTable>` — table extractor adapted from Phase 9 `drawing_tables`; relaxed thresholds (`ROW_Y_EPSILON=0.018`, `COL_X_EPSILON=0.025`); multi-page table merging: continuation pages (no header row, pending table open) have their rows appended to the prior table; eager flush at `MAX_PENDING_ROWS=20`; reuses `ExtractedTable` type with `sheet_id=unit_tag`, `sheet_title=item_type`
  - `classify_table(table_title: Option<&str>, headers: &[String]) -> SubmittalTableType` — keyword scan; `SubmittalTableType` enum: `Airflow` (CFM/ESP/RPM/static pressure), `Electrical` (volt/amp/MCA/MOP/FLA/kW), `Dimensional` (weight/dim/length/width/height/physical), `SoundData` (sound/acoust/NC/octave/Lw/Lp), `General` (fallback)
  - 10 unit tests: empty/sparse/single-page/multi-page/merge/propagation/classify-airflow/classify-electrical/classify-dimensional/classify-general

- **`crates/engine/src/lib.rs`** — Sprint 10.3.B:
  - `pub mod submittal_tables;` added to module declarations
  - `pub use submittal_tables::{classify_table, extract_unit_tables, SubmittalTableType};` added to re-exports

### Test Delta

| Suite | Before | After |
|---|---|---|
| Engine unit | 147 | 157 (+10 submittal_tables) |

---

## [2026-04-08] Sprint 10.2 Complete: Per-Unit Header Extraction + Key-Value Parsing

### Added

- **`crates/engine/src/submittal_kv.rs`** (new) — Sprint 10.2.A:
  - `extract_unit_header(pages: &[&Page], header_page_limit: usize) -> UnitHeader` — scans first `header_page_limit` pages for labeled field patterns (tag, model, manufacturer, equipment type); compiled `OnceLock<Regex>` patterns (`tag_re`, `model_re`, `manufacturer_re`, `type_re`); confidence bands: exact label keyword (`"Tag No."`, `"Model No."`, `"Manufacturer:"`, etc.) → 1.0; all other matches → 0.9; aggregate field confidence averaged; returns `UnitHeader` with `confidence=0.0` when no fields extracted
  - `extract_kv_pairs(pages: &[&Page]) -> Vec<KvPair>` — colon-split heuristic across all unit pages; accepts any span containing `:`, splits at first colon; label filtered: non-empty, ≤60 chars, not URL; value: non-empty; bbox provenance from span bbox; confidence=0.7
  - Internal helpers: `normalise_value()` (collapse whitespace), `choose_conf()` (exact vs. flex confidence selector)
  - 8 unit tests: `header_extracts_model_from_label_colon`, `header_extracts_manufacturer_from_label`, `header_extracts_tag_from_item_tag`, `header_returns_zero_confidence_when_no_fields`, `header_respects_page_limit`, `kv_pairs_extracted_from_colon_spans`, `kv_pairs_skip_empty_value_after_colon`, `kv_pairs_page_provenance_set_correctly`

- **`crates/engine/src/lib.rs`** — Sprint 10.2.B:
  - `pub mod submittal_kv;` added to module declarations
  - `pub use submittal_kv::{extract_kv_pairs, extract_unit_header};` added to re-exports

### Changed

- **`crates/engine/src/submittal_segment.rs`** — Sprint 10.2.B:
  - `use crate::submittal_kv::extract_unit_header;` import added
  - `build_index()`: after `build_units()`, iterates non-cover units; for each, slices pages to `unit.start_page..=unit.end_page`, calls `extract_unit_header(&unit_pages, 2)`, populates `unit.model`, `unit.manufacturer`, `unit.item_type` if not already set by prior detection

### Test Delta

| Suite | Before | After |
|---|---|---|
| Engine unit | 139 | 147 (+8 submittal_kv) |

---

## [2026-04-08] Sprint 10.1 Complete: `SubmittalSegmentEngine` + `index-submittal` CLI

### Added

- **`crates/engine/src/submittal_segment.rs`** (new) — Sprint 10.1.A:
  - Constants: `COVER_PAGE_MAX=3`, `UNIT_HEADER_MIN_FONT_RATIO=1.3`, `CONFIDENCE_HIGH=0.9`, `CONFIDENCE_LOW=0.7`, `CONFIDENCE_FALLBACK=0.3`
  - `compute_median_font_size(transcript) -> f64` — corpus-median font size for prominence gating
  - `detect_unit_tag_on_page(spans, median_font) -> Option<(String, f64)>` — two-pass: upper-half prominent (`y<0.5`, `font_size ≥ median × 1.3`) → anywhere prominent
  - `extract_tag(text) -> Option<String>` — normalizes to `LETTERS-DIGITS` form via regex `(?i)\b([A-Z]{1,4})-?(\d{1,4}[A-Z]?)\b`
  - Module-level `Accumulator` struct + `finalise(a, end_page) -> UnitEntry` helper
  - `build_units(page_tags, total_pages) -> Vec<UnitEntry>` — state machine: cover page detection, boundary detection on tag change, fallback to single unit with confidence=0.3
  - `SubmittalSegmentEngine::build_index(transcript, packet_name) -> SubmittalIndex`
  - 17 unit tests: tag extraction (7), detection (3), build index (7); all pass

- **`crates/contracts/src/lib.rs`** — Sprint 10.1.B:
  - `IndexSubmittal` variant added to `WorkflowOperation` enum (after `ExtractSchedules`)

- **`crates/engine/src/lib.rs`** — Sprint 10.1.C:
  - `pub mod submittal_segment;` + `pub use submittal_segment::SubmittalSegmentEngine;`

- **`apps/backend-cli/src/handlers/index_submittal.rs`** (new) — Sprint 10.1.D/E:
  - `run(req, bundle)` handler: `packet_name_from_path()` derives packet name from file stem; dry-run short-circuit; emits `OperationStarted`/`OperationEnded` for `IndexSubmittal`; calls `SubmittalSegmentEngine::build_index()`; writes JSON to output path

- **`apps/backend-cli/src/main.rs`** — Sprint 10.1.F:
  - `IndexSubmittal { input, output, dry_run }` subcommand added to `Commands` enum; match arm wired

- **`apps/backend-cli/tests/cli_integration_test.rs`** — Sprint 10.1.I:
  - `run_index_submittal()` test helper
  - `cli_index_submittal_on_sub_fixture_produces_valid_json` — extracts + indexes SUB_Rsmd_TAS_AAON-RTU, validates JSON structure
  - `cli_index_submittal_dry_run_skips_extraction` — asserts no output file when `--dry-run`

### Changed

- **`apps/backend-cli/src/handlers/mod.rs`** — Sprint 10.1.E:
  - `pub mod index_submittal;` + `WorkflowOperation::IndexSubmittal => index_submittal::run(req, bundle)` dispatch arm

### Test Delta

| Suite | Before | After |
|---|---|---|
| Engine unit | 122 | 139 (+17 submittal_segment) |
| CLI integration (active) | 46 | 48 (+2 index-submittal tests) |

---

## [2026-04-08] Sprint 10.0 Complete: IR Foundation + Corpus Analysis

### Added

- **`crates/ir/src/submittal.rs`** (new) — Sprint 10.0.B:
  - `SubmittalCoverage { total_pages, assigned_pages, unassigned_pages, coverage_ratio: f64, unit_count }` — coverage statistics for a packet
  - `UnitEntry { unit_tag, model, manufacturer, item_type, start_page, end_page, page_count, is_cover: bool, confidence: f64 }` — one detected equipment unit
  - `SubmittalIndex { schema_version: "1.0.0", packet_name, units: Vec<UnitEntry>, coverage: SubmittalCoverage }` — full index output from Sprint 10.1 engine
  - `TidyBBox { x, y, width, height: f64 }` — normalized bounding box for tidy rows
  - `TidyRow` — canonical 14-field tidy schema: `packet_name`, `revision_id`, `item_tag`, `equipment_type`, `section`, `field`, `value_raw`, `value_num`, `unit`, `page`, `bbox`, `confidence`, `source`, `conflict_flags`
  - `EquipmentDataset { schema_version, packet_name, record_count, unit_count, records: Vec<TidyRow>, unit_summaries: Vec<UnitSummary> }` — full assembled output
  - `UnitSummary { unit_tag, record_count, avg_confidence, table_record_count, kv_record_count, warnings }` — per-unit quality roll-up
  - All types: `#[derive(Debug, Clone, Serialize, Deserialize)]`; `schema_version` fixed to `"1.0.0"`
  - 8 unit tests: `tidy_row_serde_round_trip_full`, `tidy_row_serde_round_trip_minimal`, `unit_entry_serde_round_trip`, `unit_entry_cover_serde_round_trip`, `submittal_index_serde_round_trip`, `equipment_dataset_serde_round_trip`, `submittal_coverage_zero_pages`, `unit_summary_default_round_trips`

- **`crates/ir/src/lib.rs`** — Sprint 10.0.B:
  - `pub mod submittal;` added to module list
  - Re-exports: `EquipmentDataset`, `SubmittalCoverage`, `SubmittalIndex`, `TidyBBox`, `TidyRow`, `UnitEntry`, `UnitSummary`

- **`apps/backend-cli/tests/cli_integration_test.rs`** — Sprint 10.0.A:
  - `submittal_extraction_profile` `#[ignore]` benchmark: profiles all 5 tier1 SUB fixtures via `extract --dry-run`; records `wall_elapsed_ms`, `span_count`, `text_extractable`, `raster_risk` per fixture; applies decision rule (`< 5000 ms` → `full_extraction_acceptable`); writes `audit_output/phase-j-perf/sub-extraction-profile.json`

- **`audit_output/phase-j-corpus-baseline/sub-pre-baseline.json`** (new) — Sprint 10.0.C:
  - Pre-extraction baseline for 5 tier1 SUB fixtures: page counts, producers, page sizes, raster risk flags, corpus summary (702 total pages, 4 Letter + 1 non-standard, 4/5 low-risk), extraction risk assessment (overall: LOW-MEDIUM; primary risk: CARRIER-UV 331pp raster)

### Changed

- **`docs/MASTER_PLAN.md`** — Pre-work Task C: version bumped from `v4.9.0 (Phase 8 — Production Hardening)` to `v5.0.0 (Phase 10 — Submittal Data Extraction)`, date updated to April 8, 2026
- **`docs/current-state/gap-register.md`** — Pre-work Task A: v2.4.0 → v2.5.0; G-013 (`Open` → `Closed`) and G-016 (`Partially unblocked` → `Closed`) with Sprint 8.2 evidence
- **`docs/current-state/capability-matrix.md`** — Pre-work Task B: v2.3.0 → v2.4.0; "Multi-file intake bundle assembly" and "Intake Triage: page audit and rotation normalization" rows updated from `Not implemented` to `Implemented (Sprint 8.2)`
- **`docs/current-state/state-summary.md`** — Sprint 10.0 block added; test baseline table updated (IR 42 → 50; CLI ignored count 4 → 5)

### Test Delta

| Suite | Before | After |
|---|---|---|
| IR unit | 42 | 50 (+8 submittal serde) |
| CLI integration (active) | 46 | 46 (unchanged) |
| CLI integration (ignored) | 4 | 5 (+1 SUB extraction profile benchmark) |

## [2026-04-11] Sprint 9.5 Complete + DoD Row 8 Closure: DWG Corpus Validation + Drawing Docs + Production-Run Test

### Added

- **`tools/pattern_dev.rs`** — Sprint 9.5.A:
  - `--pipeline drawing-segment` mode in `validate-corpus` subcommand dispatches to new `run_validate_corpus_drawing()`
  - `DRAWING_MIN_SHEET_COUNT = 1`, `DRAWING_MIN_SHEET_COVERAGE = 0.80` pass thresholds
  - Output written to `audit_output/phase-i-corpus-baseline/dwg-corpus-report.json`

- **`docs/CLI_REFERENCE.md`** — Sprint 9.5.B:
  - `## Drawing subcommands (Phase 9)` section with `index-drawing`, `apply-sheet-addendum`, `extract-schedules` flag tables and examples
  - `## Drawing subcommand error codes` section covering all Phase 9 error codes

- **`docs/WORKFLOW_APPLYSHEETADDENDUM.md`** (new) — Sprint 9.5.C:
  - End-to-end tutorial for the drawing sheet addendum workflow
  - Prerequisites, 6-step walkthrough (index → manifest → dry-run → production run → audit review)
  - `DrawingAddendumManifest` + `SheetReplaceSpec` field reference tables
  - `change-report.json` drawing variant field reference

- **`apps/backend-cli/tests/cli_integration_test.rs`** — Sprint 9.5.D:
  - `apply_sheet_addendum_dry_run_is_deterministic` — double-executes `apply-sheet-addendum` with `dry_run=true`; compares `sheet_results` length, per-entry `sheet_id` + `status`, and top-level `dry_run` flag

- **DoD row 8 closure:**
  - `cli_apply_sheet_addendum_production_run_writes_output_pdf` integration test — builds manifest with `dry_run=false`, runs `apply-sheet-addendum` on DWG_RWB_LHHS_ALL_ORG + DWG_RWB_LHHS_ALL_ADD2, asserts output PDF exists and is non-empty, asserts ≥1 `sheet_results` entry with `status == "replaced"`; passes on Tier 1 fixtures (DWG_RWB_LHHS_ALL series)
  - `apps/backend-cli/src/main.rs` — `apply-sheet-addendum` subcommand: added `--output <PATH>` flag (writes `DrawingPatchResult` JSON); wired to `req.output_path` in handler (was hardcoded `None`); also fixes `--output` path used by existing dry-run integration tests

### All Phase 9 DoD rows: ✅ 16/16

## [2026-04-10] Sprint 9.4 Complete: Schedule Sheet Detection + Table Extraction + `extract-schedules` CLI

### Added

- **`crates/engine/src/drawing_tables.rs`** (new) — Sprint 9.4.B:
  - `ExtractedTable { sheet_id, sheet_title, table_title, headers, rows, row_count, confidence }` — serialisable output type
  - `extract_tables_from_sheet(page: &Page, sheet: &SheetEntry) -> Vec<ExtractedTable>` — span-clustering table extractor; groups spans into rows (ROW_Y_EPSILON=0.015) and columns (COL_X_EPSILON=0.02); detects header row; returns empty vec for non-schedule sheets
  - 5 unit tests: `extract_tables_non_schedule_returns_empty`, `extract_tables_detects_three_column_table`, `extract_tables_too_few_spans_returns_empty`, `cluster_rows_groups_nearby_spans`, `row_looks_numeric_classification`

- **`crates/engine/src/drawing_segment.rs`** — Sprint 9.4.A:
  - `is_schedule_sheet_title(title: &str) -> bool` — keyword-based heuristic (SCHEDULE, EQUIPMENT LIST, FIXTURE LIST, PANEL LIST, METER LIST, PIPING LIST, VALVE LIST)
  - `make_entry()` now calls `is_schedule_sheet_title()` to populate `SheetEntry.is_schedule_sheet`
  - 4 unit tests added

- **`apps/backend-cli/src/handlers/extract_schedules.rs`** (new) — Sprint 9.4.C:
  - `run()` handler: reads `LayoutTranscript`, builds `DrawingIndex`, filters schedule sheets, calls `extract_tables_from_sheet()` on each page, writes `{ schema_version, sheet_count, schedule_sheet_count, table_count, tables }` JSON
  - Supports `--dry-run` (skips extraction, exits 0)

- **`apps/backend-cli/src/main.rs`** — `ExtractSchedules` CLI subcommand with `--input`, `--output`, `--format`, `--dry-run`

- **`apps/backend-cli/tests/cli_integration_test.rs`** — 2 new integration tests:
  - `cli_extract_schedules_on_dwg_fixture_produces_json`
  - `cli_extract_schedules_dry_run_skips_extraction`

### Changed

- **`crates/contracts/src/lib.rs`** — `ExtractSchedules` variant added to `WorkflowOperation` enum
- **`crates/engine/src/lib.rs`** — `pub mod drawing_tables;` + re-exports `ExtractedTable`, `extract_tables_from_sheet`
- **`apps/backend-cli/src/handlers/mod.rs`** — `pub mod extract_schedules;` + dispatch arm

---

## [2026-04-08] Sprint 9.3 Complete: Sheet Rename Detection + Drawing Bookmarks

### Added

- **`crates/engine/src/drawings_patch.rs`** — Sprint 9.3 additions:
  - `detect_renames(original: &DrawingIndex, addendum: &DrawingIndex) -> Vec<SheetRenameEvent>` — compares sheets present in one index but not the other; emits rename events for pairs sharing a normalised title with similarity ≥ 0.75; each sheet used in at most one event (greedy best-match); results returned in `original_sheet_id` order
  - `normalize_title(title: &str) -> String` — uppercase, strip punctuation, collapse whitespace
  - `title_similarity(a: &str, b: &str) -> f64` — exact match → 1.0; longest common token-prefix ratio otherwise; 0.0 for empty inputs
  - `detect_renames()` called as Step 2.5 in `DrawingsPatchOrchestrator::run()` — result stored in `DrawingPatchResult.renames` and included in `change-report.json` under `"renames"` key
  - **4 unit tests**: `rename_detection_exact_title_match`, `rename_detection_prefix_title_match_above_threshold`, `rename_detection_different_title_no_rename`, `rename_detection_same_title_same_id_no_rename`

- **`crates/engine/src/stitch.rs`** — Sprint 9.3 additions:
  - `generate_drawing_bookmarks(doc: &mut Document, index: &DrawingIndex) -> Vec<String>` — generates a flat PDF outline (bookmarks) from a `DrawingIndex`; one entry per sheet; display text = `"{sheet_id} — {sheet_title}"`; destination = first page of sheet (`/Fit`); wires `/Outlines` into `/Catalog`; returns non-fatal warnings
  - Called from `DrawingsPatchOrchestrator::run()` after final stitch — lopdf reloads output PDF, generates bookmarks, saves; non-fatal on error
  - **2 unit tests**: `generate_drawing_bookmarks_empty_index_produces_no_entries`, `generate_drawing_bookmarks_two_sheets_produces_two_entries`

### Changed

- **`crates/engine/src/drawings_patch.rs`** — `write_audit_bundle()` now includes `"renames"` array in `change-report.json`; `renames` parameter added to function signature
- **`crates/engine/src/drawings_patch.rs`** — `DrawingsPatchOrchestrator::run()` imports `generate_drawing_bookmarks` from `stitch` module; `SheetRenameEvent` imported from `conset_pdf_ir`

---

## [2026-04-09] Sprint 9.2 Complete: Drawing Sheet Replacement + `apply-sheet-addendum` CLI

### Added

- **`crates/engine/src/drawings_patch.rs`** (new) — `DrawingsPatchOrchestrator` for drawing sheet replacement:
  - `DrawingsPatchOrchestrator::run(manifest: &DrawingAddendumManifest) -> Result<DrawingPatchResult, String>` — full pipeline: load pattern DB → extract + index original → extract + index addendum → sort specs last-to-first by `start_page` → for each spec: verify sheet in original, skip on dry_run, auto-detect or explicit addendum page range, `extract_page_range()` to temp PDF, `PdfStitcher::stitch()` via `SegmentIndex` adapter → write audit bundle
  - `drawing_index_to_segment_index(index: &DrawingIndex, source_path: &str) -> SegmentIndex` — maps `SheetEntry.sheet_id → SectionEntry.section_id`; preserves `start_page`, `end_page`, `page_count`, `confidence`
  - `extract_page_range(source_path, start_page, end_page, out_path)` — uses lopdf to extract a contiguous 0-based page range: rebuild `/Pages /Kids` array → remove unreferenced page objects → save to output path
  - `write_audit_bundle(bundle_dir, sheet_results, original_index)` — writes `change-report.json` (sheet-level results + per-discipline counts) and `metrics.json` (totals + timestamp) to `audit_bundle_dir`
  - **3 unit tests**: `orchestrator_converts_drawing_index_to_segment_index`, `orchestrator_not_found_sheet_emits_not_found_status`, `orchestrator_dry_run_skips_all_sheets`

- **`apps/backend-cli/src/handlers/apply_sheet_addendum.rs`** (new) — `apply-sheet-addendum` CLI handler:
  - Reads `manifest_path` from metadata, deserializes `DrawingAddendumManifest` JSON, propagates `--dry-run` flag and `--audit-bundle` path into the manifest, delegates to `DrawingsPatchOrchestrator::run()`
  - Error codes: `MISSING_MANIFEST_PATH`, `MANIFEST_READ_ERROR`, `EMPTY_MANIFEST`, `AUDIT_DIR_CREATE_ERROR`, `ORCHESTRATOR_ERROR`

- **CLI integration tests** (`apps/backend-cli/tests/cli_integration_test.rs`):
  - `cli_apply_sheet_addendum_dry_run_skips_all_sheets` — extract + index `DWG_RWB_LHHS_ALL_ORG.pdf` → build manifest with first detected sheet_id + `dry_run=true` → assert status `succeeded`, no output PDF written, `sheet_results[0].status == "skipped"`
  - `cli_apply_sheet_addendum_bad_manifest_path_fails_gracefully` — pass a non-existent manifest path → assert status `failed`, `error_code == "MANIFEST_READ_ERROR"`
  - `cli_apply_sheet_addendum_dry_run_writes_audit_bundle` — manifest with `audit_bundle_dir` set, `dry_run=true` → assert `change-report.json` and `metrics.json` written to the bundle dir

### Changed

- **`crates/engine/src/lib.rs`** — added `pub mod drawings_patch;` + `pub use drawings_patch::DrawingsPatchOrchestrator;`
- **`crates/contracts/src/lib.rs`** — added `ApplySheetAddendum` variant to `WorkflowOperation` enum
- **`apps/backend-cli/src/handlers/mod.rs`** — added `pub mod apply_sheet_addendum;` + dispatch arm `WorkflowOperation::ApplySheetAddendum => apply_sheet_addendum::run(req, bundle)`
- **`apps/backend-cli/src/main.rs`** — added `ApplySheetAddendum { manifest, audit_bundle, dry_run }` CLI subcommand with full dispatch arm; `--dry-run` and `--audit-bundle` propagated into manifest fields via handler

---

## [2026-04-08] Sprint 9.1 Complete: Drawing Sheet Detection + `index-drawing` CLI

### Added

- **`crates/engine/src/drawing_segment.rs`** (new) — Drawing title-block oracle and sheet index builder:
  - `DrawingSegmentEngine::build_index(transcript: &LayoutTranscript) -> DrawingIndex` — full pipeline: region detection → sheet ID extraction → boundary grouping → discipline classification → discipline summary roll-up
  - `extract_sheet_from_page(spans: &[Span]) -> (Option<String>, String)` — collects spans in `y > 0.75 || x > 0.75` region; regex matches sheet ID `[A-Z]{1,3}-?\d{3,4}`; rightmost candidate wins; extracts sheet title as longest non-ID span in region
  - `build_sheets(page_ids, page_titles) -> Vec<SheetEntry>` — groups consecutive pages with the same sheet ID; pages with no detected ID are assigned to the prior sheet; pages before any ID is detected are skipped (not assigned to a synthetic `UNKN-000`)
  - `make_chrome(sheet_id, sheet_title) -> SheetChromeMetadata` — calls `classify_sheet()` from `crates/standards-data` to populate `discipline` and `confidence`
  - `build_discipline_summary(sheets) -> Vec<DisciplineSummary>` — sorted by `sort_order`
  - Constants: `TITLE_BLOCK_BOTTOM_Y = 0.75`, `TITLE_BLOCK_RIGHT_X = 0.75`
  - Compiled regex via `OnceLock<Regex>`: `sheet_id_re()` — `(?i)\b([A-Za-z]{1,3})-?(\d{3,4})\b`
  - **15/15 unit tests pass**: 3 region detection (9.1.A), 7 sheet ID extraction (9.1.B), 5 DrawingSegmentEngine integration (9.1.C)

- **`crates/engine/src/patterns/default.json`** — added two new pattern families (v1.0.0 retained):
  - `title-block-anchor` — `(?i)\b(sheet|drwg|dwg|drawing)\b`, threshold 0.60, band `bottom`
  - `sheet-id-title-block` — `(?i)\b([A-Z]{1,3})-?(\d{3,4})\b`, threshold 0.80, band `bottom`

- **`apps/backend-cli/src/handlers/index_drawing.rs`** (new) — `index-drawing` operation handler:
  - Reads `LayoutTranscript` JSON from `--input`, calls `DrawingSegmentEngine::build_index()`, serializes `DrawingIndex` to `--output`
  - Emits `OperationStarted` / `OperationEnded` audit events
  - Error codes: `MISSING_OUTPUT_PATH`, `INVALID_TRANSCRIPT`

- **CLI integration tests** (`apps/backend-cli/tests/cli_integration_test.rs`):
  - `cli_index_drawing_on_dwg_fixture_produces_valid_json` — extract + index-drawing on `DWG_RWB_LHHS_ALL_ORG.pdf`; asserts `schema_version == "1.0.0"`, `sheet_count > 0`, all sheets have non-empty `sheet_id`
  - `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets` — extract + index-drawing on `SPEC_RWB_LHHS_ALL_ORG.pdf`; asserts status `succeeded` and `sheet_count == 0`

### Changed

- **`crates/engine/src/lib.rs`** — added `pub mod drawing_segment;` + `pub use drawing_segment::DrawingSegmentEngine;`
- **`crates/engine/Cargo.toml`** — added `conset-pdf-standards-data = { workspace = true }` dependency
- **`Cargo.toml`** (workspace) — registered `conset-pdf-standards-data = { path = "crates/standards-data" }` in `[workspace.dependencies]`
- **`crates/contracts/src/lib.rs`** — added `IndexDrawing` variant to `WorkflowOperation` enum
- **`apps/backend-cli/src/handlers/mod.rs`** — added `pub mod index_drawing;` + `WorkflowOperation::IndexDrawing => index_drawing::run(req, bundle)` dispatch arm
- **`apps/backend-cli/src/main.rs`** — added `IndexDrawing { input, output, dry_run }` subcommand variant + dispatch arm

---

## [2026-04-08] Sprint 9.0 Complete: Drawing IR Foundation + Discipline Classifier

### Added

- **`crates/ir/src/drawing.rs`** (new) — Drawing-medium IR types for Phase 9:
  - `SheetChromeMetadata` — title-block fields: `sheet_id`, `sheet_title`, `discipline`, `revision`, `date`, `firm_name`, `project_name`, `project_id`, `confidence`
  - `SheetEntry` — sheet boundary: `sheet_id`, `start_page`, `end_page`, `page_count`, `chrome: SheetChromeMetadata`, `superseded_by: Option<String>`, `is_schedule_sheet: bool` (`#[serde(default)]`)
  - `DrawingIndex` — drawing set index: `schema_version: "1.0.0"`, `sheet_count`, `sheets`, `discipline_summary`, `total_pages`; `DrawingIndex::empty(total_pages)` constructor
  - `DisciplineSummary` — per-discipline roll-up: `canonical4`, `display_name`, `sheet_count`, `sort_order`
  - `DrawingAddendumManifest` + `SheetReplaceSpec` + `SheetPageRange` — manifest types for `apply-sheet-addendum` (Sprint 9.2)
  - `DrawingPatchResult` + `SheetPatchResult` + `SheetPatchStatus` + `SheetRenameEvent` — result types for `apply-sheet-addendum` audit (Sprint 9.2)
  - **5/5 serde round-trip unit tests** pass: `drawing_index_serde_round_trip`, `sheet_entry_superseded_by_round_trip`, `sheet_entry_not_superseded_round_trip`, `drawing_addendum_manifest_round_trip`, `drawing_patch_result_round_trip`

- **`crates/standards-data/src/aec.rs`** — Full `classify_sheet()` discipline classifier (replaces `"stub"` stub):
  - 5-step algorithm from `docs/DRAWINGS_CLASSIFICATION.md`: strip+uppercase designator → UDS single-letter table (13 entries: G/V/D/L/A/I/S/M/P/E/F/T) → multi-letter alias table (10 entries: FP/FA→FIRP, DDC/ATC→CTRL, SEC/AV/IT→TECH, SV→SURV, DM→DEMO, EX→UNKN) → `C` disambiguation via title keywords (CONTROLS/BMS/DDC/BAS → CTRL/0.85; civil keywords → CIVL/0.85; default → CIVL/0.72) → UNKN fallback
  - `ClassifyResult { canonical4: &'static str, display_name: &'static str, sort_order: u32, confidence: f64 }`
  - `data_version()` updated from `"stub"` to `"1.0.0"`
  - **33/33 unit tests** pass: all 13 UDS codes, 9 named aliases, 5 C disambiguation cases (no-title / civil-kw / controls-kw / BMS-kw / utility-civil), lowercase/mixed-case prefix, all-numeric, empty string, unknown designator

- **`drawing_extraction_profile` `#[ignore]` benchmark** (`apps/backend-cli/tests/cli_integration_test.rs`) — runs `extract` on the 3 largest DWG fixtures by file size (`DWG_WRA_PESH_ALL_ORG.pdf` 177 MB, `DWG_WRA_PESH_ALL_ADD2.pdf` 121 MB, `DWG_WRA_PESH_ALL_ADD1.pdf` 25 MB); reads `elapsed_ms` from `result.diagnostics.elapsed_ms` in `WorkflowResponse`; writes baseline to `audit_output/phase-i-perf/drawing-extraction-profile.json` with per-fixture timing and schema metadata

- **`audit_output/phase-i-corpus-baseline/dwg-pre-baseline.json`** — DWG pre-Phase-9 corpus baseline snapshot (schema `1.0.0`): all 11 DWG tier-1 fixtures, pipeline=segment, 0/11 pass, max `coverage_ratio` 9.4% (`DWG_VLK_CrowTrk_MECH_ADD1`). Documents the "Phase 9 starts here" state so Sprint 9.1 progress is measurable.

### Changed

- **`crates/ir/src/lib.rs`** — added `pub mod drawing;` and full re-exports: `DrawingAddendumManifest`, `DrawingIndex`, `DrawingPatchResult`, `DisciplineSummary`, `SheetChromeMetadata`, `SheetEntry`, `SheetPageRange`, `SheetPatchResult`, `SheetPatchStatus`, `SheetRenameEvent`, `SheetReplaceSpec`

### Notes

- Sprint 9.0.A profiling decision: full-document extraction is acceptable for Phase 9 per-file operations. `extract_range()` is deferred as a future enhancement. `ARCHITECTURE.md` Constraint 1 checklist is fulfilled by the `#[ignore]` benchmark; measured numbers will be filled in after first execution.
- No `Cargo.toml` changes required: `crates/ir` already depends on `serde`; `crates/standards-data` uses `&'static str` return types, so no serde dependency is needed for `ClassifyResult`.

---

## [2026-04-08] Phase 9 Plan — Drawing Sheet Management

### Added

- **`docs/plans/Phase_9_Plan.md`** — Full Phase 9 micro-task plan for Drawing Sheet Management. Covers: Phase 8 completion verification; Foundation Assessment (5 structural constraints, reusability of `PdfStitcher`, `DiagnosticEvent`, `PatternDatabase`, and `standards-data`); 6 sprints (9.0 foundation, 9.1 sheet detection, 9.2 sheet replacement, 9.3 renaming + bookmarks, 9.4 schedule extraction, 9.5 hardening + docs); 16-criterion DoD table; sprint dependency graph and recommended attack order.

---

## [2026-04-08] Sprint 8.7 Complete: User Documentation — Phase 8 CLOSED

### Added

- **`docs/CLI_REFERENCE.md`** — Full reference for all 11 `backend-cli` subcommands (`extract`, `segment`, `parse`, `edit`, `regenerate`, `stitch`, `apply-addendum`, `intake`, `visualize`, `visualize-segments`, `visualize-ast`). Each entry covers: description, required/optional arguments, output format, invocation example. Global output format (`WorkflowResponse` JSON) and exit code contract documented. Error codes table for `apply-addendum` (`MISSING_MANIFEST_PATH`, `MANIFEST_READ_ERROR`, `EMPTY_MANIFEST`, `AUDIT_DIR_CREATE_ERROR`, `ORCHESTRATOR_ERROR`), per-section failure table by stage (Parse / Edit / Render / Stitch), and extract/segment/parse summary error table.

- **`docs/WORKFLOW_APPLYADDENDUM.md`** — End-to-end `apply-addendum` tutorial. Covers: prerequisites, pipeline diagram, section inspection with `extract` + `segment`, AST inspection with `parse` + `visualize-ast`, manifest authoring, `--dry-run` validation, production run, and audit bundle analysis (`change-report.json`, `diagnostics.jsonl`, `metrics.json`). Includes complete `AddendumManifest`, `SectionEditSpec`, `EditOperation` (replace / delete / insert_after), `NodePath`, and `SpecChromeMetadata` JSON reference tables. Multi-section manifest example and partial-success behaviour documented.

## [2026-04-08] Sprint 8.6 Complete: Pattern Database as Versioned JSON

### Added

- **`crates/engine/src/patterns/mod.rs`** — `RegionBand` enum (`Top`, `Bottom`, `Full`), `PatternSpec { regex, confidence_threshold, band, examples }`, `PatternDatabase { version, patterns: HashMap<FamilyId, PatternSpec> }`, `PatternDatabase::load_default()` which embeds and parses `default.json` at compile time via `include_str!()`. Fail-fast: returns a human-readable `Err` if the JSON is malformed.

- **`crates/engine/src/patterns/default.json`** (version `"1.0.0"`) — three pattern families: `footer-section-id` (bottom band, confidence 0.95, regex matching CSI two/three–group section IDs), `page-counter` (bottom band, confidence 0.98, case-insensitive `Page N of M`), `header-band` (top band, geometric placeholder, threshold 0.0).

- **`pattern_db_version` field in `AddendumResult`** (`crates/ir/src/addendum.rs`) — `Option<String>`, serde skip-if-none for back-compat with pre-8.6 `change-report.json` files. Set to `"1.0.0"` by `SpecsPatchOrchestrator::run()`.

- **`PatternDatabase::load_default()` called at startup** (`crates/engine/src/specs_patch.rs`, Step 0 before extraction) — ensures the pipeline fails with a clear error rather than a regex panic if `default.json` is ever accidentally malformed.

- **`default_pattern_database_parses_successfully` unit test** (`crates/engine/src/patterns/mod.rs`) — verifies all three family entries, `RegionBand` values, and non-empty version and regex fields.

- **`cli_apply_addendum_change_report_contains_pattern_db_version` integration test** (`apps/backend-cli/tests/cli_integration_test.rs`) — runs `apply-addendum --dry-run`, asserts `change-report.json` contains `pattern_db_version == "1.0.0"`.

### Changed

- `crates/engine/Cargo.toml` — `serde` and `serde_json` workspace dependencies added (required by new `patterns` module).
- `crates/engine/src/lib.rs` — `pub mod patterns` declared.

## [2026-04-08] Sprint 8.5 Complete: Metrics Output in Audit Bundle

### Added

- **`metrics.json` in `--audit-bundle` directory** (`apps/backend-cli/src/handlers/apply_addendum.rs`): Written alongside `change-report.json` and `diagnostics.jsonl` after every `apply-addendum` invocation. Schema `"metrics/v1"`. Fields: `total_pages_input`, `total_pages_output`, `sections_detected`, `sections_patched`, `section_coverage_ratio`, `total_elapsed_ms`, and `per_section` array (one entry per parsed section with `section_id`, `parse_node_count`, `unclassified_count`, `unclassified_ratio`, `render_ms` (null on dry-run), `stitch_ms`). All values are derived from `AddendumResult.diagnostics` — `metrics.json` has no independent state and is always consistent with `diagnostics.jsonl`.

- **`build_metrics()` private helper** (`apps/backend-cli/src/handlers/apply_addendum.rs`): Walks `result.diagnostics` once to collect per-stage data (`ExtractionDiagnostic` for page count + elapsed, `SegmentationDiagnostic` for coverage, `ParseDiagnostic` per section for node counts, `RenderDiagnostic`/`StitchDiagnostic` for per-section timing). `total_pages_output` uses saturating arithmetic on pages removed/inserted. `unclassified_ratio` rounded to 3 decimal places.

- **`cli_apply_addendum_writes_metrics_json` integration test** (`apps/backend-cli/tests/cli_integration_test.rs`): Runs `apply-addendum --dry-run` on the SPEC corpus fixture with 1 section; asserts `metrics.json` written to audit bundle, schema field is `"metrics/v1"`, and all required top-level fields present with valid types.

- **`cli_apply_addendum_metrics_per_section_matches_manifest_sections` integration test**: Uses a 2-section manifest; asserts `per_section.len() == 2 == sections_patched` and each `per_section` entry contains the required fields.

### Notes

- **8.5.B was pre-done**: `SegmentIndex.coverage: CoverageStats` already held `coverage_ratio: f64` and `pages_missing_footer: usize` (populated by `SegmentEngine::build_index()`). No code change required for that sub-task.

## [2026-04-08] Sprint 8.4 Complete: Performance Benchmark + Architecture Constraint Documentation

### Added

- **`apply_addendum_benchmark_large_spec` `#[ignore]` integration test** (`apps/backend-cli/tests/cli_integration_test.rs`): Exercises the full dry-run pipeline on the 571-page SPEC_RWB_LHHS_ALL_ORG corpus fixture with 3 real sections (first, middle, last from the segment index). Measures wall-clock time via `std::time::Instant` and reads per-stage `elapsed_ms` from `diagnostics.jsonl` in the audit bundle (extraction, segmentation, stitch). Asserts total elapsed < 10,000 ms. Writes `audit_output/phase-h-perf/benchmark-large-spec.json` with fields: date, fixture, sections_patched, dry_run, wall/extract/segment/stitch timing, threshold, and pass flag.

- **"Known Design Constraints" section in `ARCHITECTURE.md` v4.9.0**: Documents Constraint 1 — full-document extraction per `apply-addendum` invocation. States: O(n\_pages) cost model, affected call site (`Extractor::extract()` in `SpecsPatchOrchestrator::run()`), rationale (segmentation requires full transcript for boundary detection; stitcher needs full `SegmentIndex` for unchanged-page hash validation), acceptable trade-off for current spec-book use cases, and Phase 9+ mitigation path (range-bounded `Extractor::extract_range()` API). Includes 4-step decision checklist required before Phase 9 batch architecture is locked. ToC updated (entry 9 added) and Revision History extended with v4.9.0 row.

## [2026-04-08] Sprint 8.3 Complete: Torture Corpus Validation Infrastructure

### Added

- **`CORPUS_MIN_COVERAGE`, `CORPUS_MAX_UNCLASSIFIED`, `CORPUS_MIN_SECTION_COUNT` constants** (`tools/pattern_dev.rs`): Pass/fail thresholds used by the pipeline corpus validator — 0.90, 0.01, and 1 respectively.

- **`--pipeline segment|parse` mode for `validate-corpus`** (`tools/pattern_dev.rs`): New optional `--pipeline <STAGE>` arg on the `ValidateCorpus` subcommand. When set, dispatches to `run_validate_corpus_pipeline()` instead of the existing heuristic pattern-family path. Runs the full engine pipeline (`Extractor::extract()` → `segment_transcript()`) for `segment`; additionally calls `parse_section_with_stats()` per section for `parse`. Emits `corpus-report.json` (schema `0.1.0`) with per-fixture pass/fail, `section_count`, `coverage_ratio`, `unclassified_ratio` (parse only), and an `aggregate` block. `conset-pdf-engine` and `conset-pdf-ir` added as workspace dependencies in `tools/Cargo.toml`.

- **Phase-H corpus baseline** (`audit_output/phase-h-corpus-baseline/`): First pipeline-level corpus run against all 27 Tier 1 fixtures. Results: 3/27 pass (11.1%). The three passing fixtures are all SPEC_* documents. 24 failures are expected by design — DWG, NAR, and SUB document types carry no CSI section-ID footer stamps. Primary validation target `SPEC_RWB_LHHS_ALL_ORG`: 89 sections, 96.5% coverage, 0.2% unclassified (7,971 nodes) — all thresholds met.

- **Unchanged-page content-hash validation** (`crates/engine/src/stitch.rs`): `validate_unchanged_present()` previously checked `doc.objects.contains_key(id)` only. Added `snapshot_hashes(doc, ids) -> HashMap<ObjectId, u64>` (FNV-1a-64 over `format!("{obj:?}")` bytes) called *before* `splice_page_tree()`, and `validate_unchanged_content(doc, &before_hashes, &mut warnings)` called after `fixup_bookmarks()`. Any object whose hash differs between the snapshot and the post-splice document appends a human-readable warning to `StitchResult.warnings`. Makes Non-Negotiable #12 ("unchanged pages must remain unchanged") mechanically enforceable.

- **`apply_addendum_dry_run_is_deterministic` integration test** (`apps/backend-cli/tests/cli_integration_test.rs`): Runs `apply-addendum --dry-run` twice on `SPEC_RWB_LHHS_ALL_ORG.pdf` with an identical manifest and compares the resulting `change-report.json` files. Asserts: `section_results` length and per-entry `section_id`/`status` are identical across runs; `diagnostics` entries with `stage=="parse"` have identical `node_count` and `node_distribution`. Timestamps and `elapsed_ms` fields are explicitly excluded. Closes the determinism testing gap (Risk 2 in the Phase 8 architecture review).

- **`stitch_unchanged_pages_content_hash_unchanged` unit test** (`crates/engine/src/stitch.rs`): Performs a valid single-section stitch and asserts `result.warnings` is empty, confirming the new hash check produces no false positives on clean stitches.

- **`stitch_two_sections_with_page_growth_preserves_middle_section` unit test** (`crates/engine/src/stitch.rs`): 9-page three-section fixture (A: 0–2, B: 3–5, C: 6–8); stitches C with a 4-page replacement (+1) then A with a 5-page replacement (+2); asserts final page count = 12, original B-section object IDs still present, and no content-hash warnings. Validates last-to-first stitch ordering under nontrivial page-delta conditions at both document ends.

## [2026-04-07] Phase 7 Complete: End-to-End Apply-Addendum Workflow

### Added

- **`crates/ir/src/addendum.rs`** — Phase 7 IR types for the apply-addendum pipeline:
  - `AddendumManifest`: JSON contract with optional `description`, `issue_date`, `project_metadata` override, and required `sections: Vec<SectionEditSpec>`.
  - `SectionEditSpec`: per-section patch spec with `section_id`, `operations: Vec<EditOperation>`, and optional `chrome_override`.
  - `SectionPatchStatus`: enum discriminating `Success { pages_removed, pages_inserted }` vs `Failed { reason }`.
  - `SectionPatchResult`: per-section outcome with `section_id`, `section_title`, and `status`.
  - `AddendumResult`: orchestrator output with aggregate `total_sections`, `succeeded`, `failed`, per-section `section_results`, and optional `output_path`.
  - Full serde round-trip tests for all types + `AddendumResult::from_results()` constructor.
- **`crates/engine/src/specs_patch.rs`** — `SpecsPatchOrchestrator::run()` — core Phase 7 orchestration engine:
  - **Algorithm**: Extract → Segment → [for each section] (Parse → Edit → Render Chrome OR dry-run) → Stitch (descending page-order, last-to-first).
  - **Partial success**: per-section failures recorded in `SectionPatchResult`; other sections continue.
  - **Chrome metadata merge**: base (extracted from source PDF) + manifest-level override + per-section override (highest priority).
  - **Temp directory**: intermediate replacement PDFs and stitch outputs managed in `$TMPDIR/specs_patch_<timestamp>/`.
  - **Issue date logic**: addendum `issue_date` only fills `metadata.date` when extracted date is empty.
  - 5 unit tests covering chrome metadata merge edge cases.
  - Exported from `crates/engine/src/lib.rs`.
- **`apps/backend-cli/src/handlers/apply_addendum.rs`** — `ApplyAddendumHandler` for CLI:
  - Loads and validates `AddendumManifest` JSON from `manifest_path` metadata key.
  - Delegates to `SpecsPatchOrchestrator::run()`.
  - Writes `change-report.json` to `--audit-bundle` directory when provided.
  - Emits `OperationStarted`/`OperationEnded` audit events.
  - Partial success: status is `failed` if all sections failed, `succeeded_with_warnings` if some succeeded.
- **`ApplyAddendum` subcommand** in `apps/backend-cli/src/main.rs`:
  - Args: `--original <PDF>` (source spec), `--addendum <JSON>` (AddendumManifest JSON), `--output <PDF>` (optional; skipped on dry-run), `--audit-bundle <DIR>` (optional), `[--dry-run]`.
  - Wires to `WorkflowOperation::SpecsPatch` (contract variant already existed).
- **5 Phase 7 integration tests** in `apps/backend-cli/tests/cli_integration_test.rs`:
  - Dry-run on SPEC fixture — parses and edits but writes no output PDF.
  - Missing original PDF — returns status=failed.
  - Missing manifest JSON — returns status=failed.
  - Unknown section ID in spec — partial success (marked as per-section failure in change-report).
  - Audit artifacts written — change-report.json exists in `--audit-bundle` dir.
  - Full round-trip Chrome test `#[ignore]` — real SPEC, real render, produces valid PDF with `%PDF` header.

### Pre-Phase-7 Cleanup (Tasks 7.0.A and 7.0.B)

- **Section title extraction in `crates/engine/src/segment.rs`** (Task 7.0.A):
  - `detect_section_id()` now returns `Option<(String, String)>` — captures section_id + section_title from footer.
  - `extract_title_from_clusters()` helper: scans footer clusters right-to-left after section ID cluster; strips leading `–` or `-`; trims whitespace.
  - `build_sections()` populates `SectionEntry.section_title` from extracted title (empty string when not detected).
  - 12 new unit tests covering footer format variants (em-dash, hyphen, no-title, multiple sections).
  - `SectionEntry.section_title` is now guaranteed populated for all sections (previously always empty string).
- **Documentation sync** (Task 7.0.B):
  - `docs/state-summary.md`: "Next Focus" updated to reflect Phase 7 as apply-addendum.
  - `docs/MASTER_PLAN.md`: Phase 7 dependency graph annotation clarified from `← ALPHA COMPLETE` to `← NEXT`.

### Baseline

- **36/36** CLI integration tests pass (30 pre-Phase 6 + 6 Phase 6 stitch; 2 Chrome `#[ignore]`).
- **65/65** engine unit tests pass (53 pre-Phase 7 + 12 new segment titles).
- **30/30** IR unit tests pass (18 prior + 12 new addendum types).
- **5/5** specs_patch unit tests pass (chrome metadata merge edge cases).
- **41/41** CLI integration tests pass (36 pre-Phase 7 + 5 Phase 7 apply-addendum; 2 Chrome `#[ignore]`).

### Non-Negotiable Design Adherence

- **Multi-section stitch order — last-to-first (descending page)** per Non-Negotiable #1. Stitching section B (page 200–210) before section A (page 50–60) preserves page indices for A when it is processed.
- **Partial success** achieves Non-Negotiable #13 (soft fail on syntax / typographic errors) and #17 (downstream stages continue when one section fails).
- **Chrome metadata override** prevents hand-patching: manifest-level and per-section chromemetadata let users update headers/footers without CLI changes.
- **Dry-run mode** validates arg correctness and HTML assembly without invoking Chrome or writing files — safe testing mode.
- **Audit bundle** captures full per-section change history: manifest, segment index, change report, per-section AST and PDFs.

---

## [2026-04-06] Font Typography Extraction (Phase 5.5 Supplement)

### Added

- **`lopdf = "0.40.0"` dependency** — added to workspace `[workspace.dependencies]`; wired into `crates/engine/Cargo.toml`. `rust-version` bumped from 1.82 → 1.85 to satisfy lopdf MSRV.
- **`crates/ir/src/stitch.rs`** — `StitchPlan` (original_path, section_id, segment_index, replacement_path, output_path, dry_run), `StitchResult` (section_id, pages_removed, pages_inserted, total_pages_before, total_pages_after, bookmarks_updated, warnings), `StitchError` (thiserror: SectionNotFound, OriginalNotFound, ReplacementNotFound, WriteFailed, PageRangeOutOfBounds, PdfStructure). Exported from `crates/ir/src/lib.rs`.
- **`crates/engine/src/stitch.rs`** — `PdfStitcher::stitch()` implements the full page-replacement algorithm:
  1. Load original + replacement PDFs via `lopdf::Document::load()`.
  2. Resolve section page range from `SegmentIndex`.
  3. Renumber replacement object IDs to avoid collisions, copy objects into original document.
  4. Splice `/Pages` root `/Kids` array: `original[..del_start] + replacement_pages + original[del_end+1..]`.
  5. Update `/Parent` references on replacement pages.
  6. Remove deleted section's `/Page` objects from the object store.
  7. Re-route any outline-item `/Dest` destinations pointing to deleted pages.
  8. Validate unchanged pages are still present.
  9. Write output file (skipped on dry-run).
  Private helpers: `resolve_section_range`, `sorted_page_ids`, `load_and_merge_replacement`, `splice_page_tree`, `find_pages_root_id`, `fixup_bookmarks`, `validate_unchanged_present`.
  8 unit tests: 3 pure-logic + 5 lopdf-based with temp PDFs.
- **`WorkflowOperation::Stitch`** contract variant added to `crates/contracts/src/lib.rs`.
- **`apps/backend-cli/src/handlers/stitch.rs`** — `Stitch` CLI handler: reads `segment_index_path`, `replacement_path`, `section_id` from request metadata; loads `SegmentIndex`; calls `PdfStitcher::stitch()`; emits `OperationStarted`/`OperationEnded` audit events.
- **`Stitch` subcommand** wired in `apps/backend-cli/src/main.rs` — args: `--input`, `--segment-index`, `--section`, `--replacement`, `--output`, `[--dry-run]`.
- **6 Phase 6 integration tests** in `apps/backend-cli/tests/cli_integration_test.rs`: dry-run no-write, missing-input failure, missing-replacement failure, missing-segment-index failure, unknown-section-id failure, full stitch produces valid PDF with `%PDF` header.

### Baseline

- 36/36 CLI integration tests pass (30 pre-Phase 6 + 6 Phase 6; 2 `#[ignore]` tests excluded).
- 53/53 engine unit tests pass (43 pre-Phase 6 + 8 Phase 6 stitch + 2 ignored render tests).
- 13/13 IR unit tests pass.

---

## [2026-04-06] Font Typography Extraction (Phase 5.5 Supplement)

### Added

- **`font_weight: f32` and `is_italic: bool` on `SpanData`** (`crates/pdf-extraction/src/types.rs`): PDFium now populates `font_weight` via `text_obj.font().weight()` (matching against `PdfFontWeight` enum variants → f32 value 100–900; defaults to 400 when PDFium returns an error) and `is_italic` via `text_obj.font().is_italic()`. These represent actual values from the source PDF's font metadata.
- **`font_weight: f64` wired through extraction pipeline** (`crates/engine/src/pipeline/extraction.rs`): `span.font_weight` is now set from `f64::from(span_data.font_weight)` in addition to the existing `span.font_name` assignment. Previously `font_weight` was always hardcoded to 400.
- **`is_italic: bool` on `Span` IR type** (`crates/ir/src/types.rs`): New field with `#[serde(default)]` (defaults `false`); set from `span_data.is_italic` in the extraction pipeline. `Span::new()` initializes it to `false`.
- **`body_font_name: String` on `SectionLayout`** (`crates/ir/src/ast.rs`): Modal (most-frequent) font family name among body spans in the section, as reported by PDFium. Uses `#[serde(default = "default_font_name")]` for backward compatibility (defaults to `"Unknown"` when deserializing older AST JSON files).
- **`modal_font_name()` helper in `parse.rs`** (`crates/engine/src/parse.rs`): Computes the most-frequent font name from a slice using a `HashMap<&str, usize>` frequency count. `parse_section()` now collects `font_names: Vec<String>` from body spans and passes them to `compute_section_layout()`.
- **`body_font_name` used in `build_full_html()`** (`crates/engine/src/render/chrome.rs`): When `layout` is `Some` and `body_font_name` is not `"Unknown"` or empty, it replaces `config.font_family` as the CSS `font-family`. Falls back to `config.font_family` for sessions with no extractable font metadata.

### Backward Compatibility

- `Span.is_italic` and `SectionLayout.body_font_name` both carry `#[serde(default)]` — existing serialized transcript and AST JSON files deserialize without errors.
- `SpanData.font_weight` and `SpanData.is_italic` are new required fields; any code constructing `SpanData` struct literals directly (e.g. in extractor tests) must be updated.

---

## [2026-04-06] Layout Geometry Capture (Phase 5 Supplement)

### Added

- **`SectionLayout` IR type** (`crates/ir/src/ast.rs`): New struct capturing measured PDF geometry for a parsed section — `body_left` (normalized x of leftmost body span, i.e. the physical left text margin), `body_right` (normalized x of rightmost span right-edge), `font_size_pt` (median font size in points from source PDF spans), `line_gap_norm` (median top-to-top y distance between consecutive lines in normalized coordinates). Exported from `crates/ir/src/lib.rs` as a public type.
- **`x_indent: f64` on `AstNode`** (`crates/ir/src/ast.rs`): Normalized x position of the leftmost span on that node's first line. Populated by the parser from span geometry; `#[serde(default)]` so existing serialized `ast.json` files remain fully compatible.
- **`layout: Option<SectionLayout>` on `SectionAst`** (`crates/ir/src/ast.rs`): Measured geometry attached to each section at parse time. `None` when fewer than two body spans are found (empty or title-only sections). `#[serde(default)]` for full backward compatibility.
- **Layout geometry computation in `parse.rs`** (`crates/engine/src/parse.rs`): `parse_section()` now collects body-span raw data (x, right-edge, font size, y) across all pages of the section and calls `compute_section_layout()` to produce a `SectionLayout`. New helpers: `compute_section_layout(x_vals, x_right_vals, font_sizes, span_ys) -> Option<SectionLayout>` using `f64::min`/`f64::max` for left/right marginss and `median_val()` for font size and line-gap statistics; `median_val(vals: &[f64]) -> f64` (sort + midpoint).
- **`x_indent` propagation in `cluster_lines()` and `classify_lines()`** (`crates/engine/src/parse.rs`): `cluster_lines()` now returns `Vec<(String, usize, f64)>` — the third element is the `bbox.x` of the leftmost span on that line after x-sort. `classify_lines()` writes it into `FlatItem.x_indent`; `build_tree()` carries it through to `AstNode.x_indent`.
- **Measured margin-left in `build_body_html()`** (`crates/engine/src/render/body.rs`): Function signature extended to `build_body_html(ast, config, layout: Option<&SectionLayout>)`. When `layout` is `Some` and `node.x_indent > 0.0`, an inline `style="margin-left: X.XXXin"` is emitted using `(node.x_indent - layout.body_left) * 8.5` (scales normalized coords to CSS inches on an 8.5 in page). Falls back silently to CSS-class-only output when `layout` is `None`.
- **Measured font size and line-height in `build_full_html()`** (`crates/engine/src/render/chrome.rs`): Function signature extended to `build_full_html(body, chrome, config, layout: Option<&SectionLayout>)`. When `layout` is `Some`, `font_size_pt` replaces `config.font_size_pt` as the CSS `font-size`; `line_gap_norm * (11 * 72)` is converted to a CSS `line-height` ratio (clamped 1.1–2.0), replacing the hardcoded 1.4. Falls back to config defaults when `layout` is `None`.
- **`ast.layout.as_ref()` threaded in `SectionRenderer`** (`crates/engine/src/render/mod.rs`): Both `render()` and `dry_run()` pass `ast.layout.as_ref()` to both `build_body_html` and `build_full_html`.

### Backward Compatibility

- Existing `ast.json` files with no `x_indent` or `layout` fields deserialize correctly via `#[serde(default)]`.
- All 30 non-Chrome integration tests pass; 43/43 engine unit tests pass; 13/13 IR unit tests pass.

---

## [2026-04-06] Phase 5 Complete: Section Regeneration

### Added

- **Phase 5 (Section Regeneration — AST → PDF via HTML/CSS + Chrome):**
  - `crates/ir/src/render.rs` — `SpecChromeMetadata` (project/firm/section chrome fields), `RenderConfig` (font family, font size, page size), `PageSize` (Letter/A4), `RenderResult` (pdf_bytes, page_count_estimate, warnings), `RenderError` (thiserror-derived: ChromeNotFound, TempFileError, ChromeError, PdfNotWritten, HtmlBuildError).
  - `crates/engine/src/render/body.rs` — `build_body_html()`: converts `SectionAst` → HTML fragment; maps all 7 `OutlineTag` variants to CSS classes (`csi-part`, `csi-article`, `csi-para`, `csi-sub1`, `csi-sub2`, `csi-sub3`, `csi-body`).
  - `crates/engine/src/render/chrome.rs` — `build_full_html()`: wraps body in `<!DOCTYPE html>` with `<head>`, CSS reset/typography, and CSS Paged Media `@page` margin-box rules (`@top-center` firm/project header, `@bottom-left` date+section+title, `@bottom-right` Page N of M via `counter(page)`/`counter(pages)`). All chrome fields HTML-escaped.
  - `crates/engine/src/render/chrome_pdf.rs` — `render_html_to_pdf()`: invokes Chrome subprocess with `--headless --disable-gpu --no-sandbox --print-to-pdf`; binary discovery via `CHROME_PATH` env var with fallback to common Windows/Linux/macOS paths; writes PDF to temp file and reads bytes back.
  - `crates/engine/src/render/mod.rs` — `SectionRenderer { config: RenderConfig }`: `new(config)`, `with_defaults()`, `render(ast, chrome_meta) -> Result<RenderResult, RenderError>`, `dry_run(ast, chrome_meta) -> RenderResult` (builds HTML, skips Chrome — used by CLI `--dry-run` and CI).
  - `apps/backend-cli` — `regenerate` subcommand (`--ast`, `--chrome-metadata`, `--output`, `--section`, `--dry-run`, `--font`, `--font-size`); handler `src/handlers/regenerate.rs`; `WorkflowOperation::Regenerate` variant added to contracts.
  - 5 Phase 5 integration tests appended to `apps/backend-cli/tests/cli_integration_test.rs`: dry-run no-write, missing-AST failure, missing-section failure, invalid-chrome-metadata failure, full-Chrome round-trip (`#[ignore]`). 30 non-ignored tests pass.
- **G-010 closed** — 4 behavioral audit tests added to `crates/audit/src/bundle.rs`: JSON round-trip serde, event ordering, count tracking with clear, iter yields all events.
- **G-008 accepted-closed** — `Document`/`Element` stubs in `crates/ir/src/types.rs` have no defined downstream consumer; closing as deliberate scope deferral pending Phase 6+ consumer.

---

## [2026-04-05] Phase 4 Complete: Edit Operations

### Added

- **Phase 4 (Surgical AST Edit Operations):**
  - `crates/ir/src/edit.rs` — `NodePath`, `EditOperation` (InsertAfter/Delete/Replace), `EditRequest`, `EditResult`, `EditError`; all serde round-tripped (8/8 unit tests).
  - `crates/engine/src/edit.rs` — `SectionEditor` with pre-flight validation and `apply(EditRequest) -> EditResult`; helper functions `find_node`, `find_node_mut`, `find_in_parent`, `make_marker`, `renumber_siblings`, `apply_delete_to_section`, `apply_replace_to_section`, `apply_insert_after_to_section` (27/27 unit tests).
  - `apps/backend-cli` — `edit` subcommand (`--input`, `--operations`, `--output`, `--dry-run`); handler in `src/handlers/edit.rs`; `WorkflowOperation::Edit` variant added to contracts.
  - All 7 CLI handlers confirmed emitting `OperationStarted`/`OperationEnded` audit events — **G-006 closed**.
  - 7 Phase 4 integration tests appended to `apps/backend-cli/tests/cli_integration_test.rs`: delete-renumber, replace-text-only, insert-after-renumber, multi-op, invalid-section failure, invalid-path failure, dry-run no-write. All 26/26 integration tests passing.
- **G-003 accepted-closed** — `backend-cli` covers all runtime paths; `crates/engine/src/main.rs` version-only is deliberate scope deferral.

### CSI Renumbering Scheme (canonical, locked in Phase 4)

| Level | Tag | Marker format | Overflow limit |
|---|---|---|---|
| 0 | Part | `PART N` | unlimited |
| 1 | Article | `P.N` (P = parent Part number) | unlimited |
| 2 | Paragraph | `A.` … `Z.` | 26 |
| 3 | SubParagraph | `1.` `2.` … | unlimited |
| 4 | SubSubParagraph | `a.` … `z.` | 26 |
| 5 | SubSubSubParagraph | `1)` `2)` … | unlimited |

---

## [2026-04-05] Phase 1–3 Complete + Parser Hardening Sprint

### Added

- **Phase 1 (Layout Transcript Extraction):** Real PDFium extraction wired in `crates/engine/src/pipeline/extraction.rs`; `SpanData`→`Span` conversion with normalized bbox (top-left origin, [0,1] range); `visualize` CLI subcommand with per-page PNG overlay; G-001, G-002, G-005, G-007, G-009 all closed; 8/8 integration tests pass across SPEC, DWG, NAR, SUB, and simple fixtures.
- **Phase 2 (Section Segmentation):** `SegmentIndex` and `SectionEntry` IR types in `crates/ir/src/segment.rs`; CSI footer-oracle segmentation engine in `crates/engine/src/segment.rs`; `segment` and `visualize-segments` CLI subcommands; 5/5 Phase 2 integration tests pass.
- **Phase 3 (Paragraph Parsing & AST):** `ParsedDocument`, `SectionAst`, `AstNode`, `OutlineTag` IR types in `crates/ir/src/ast.rs`; line-clustering + 5-level CSI outline tree parser in `crates/engine/src/parse.rs` with `build_tree` and `classify_lines`; HTML collapsible AST visualizer in `crates/engine/src/visualize_ast.rs`; `parse` and `visualize-ast` CLI subcommands; 6/6 Phase 3 integration tests pass.

### Fixed (Parser Hardening Sprint — April 5, 2026)

- **Span x-sort:** PDFium returns spans in content-stream order; all `body_spans` now sorted by `(y ASC, x ASC)` before line clustering. Without this, PART headings were assembled in wrong order (`"GENERAL PART 1"` instead of `"PART 1 GENERAL"`), causing regex failures.
- **`LINE_Y_EPSILON` raised 0.005 → 0.012:** Same-visual-line spans (e.g. dash separators) had y-delta up to 0.006, formerly splitting them into separate clusters and breaking line text assembly.
- **Cluster-based section ID detection** (`crates/engine/src/segment.rs`): Footer section IDs are rendered as 2–3 adjacent spans (`"22 "`, `"07 "`, `"00 "`). New `detect_section_id()` merges all footer spans (y > FOOTER_Y=0.90) into x-proximity clusters, applies a single-digit merge pass (`"0 0"`→`"00"`) for PDFium split-zero rendering, skips date clusters (4-digit year), then matches on merged text. Section detection: 16 → 89 sections.
- **`FOOTER_Y` raised 0.85 → 0.90:** Body text at y ≈ 0.86–0.89 contained cross-references with CSI section IDs (e.g. `"specifications in accordance with Section 22 05 00."`) that generated false-positive section boundaries.
- **Noise-only line skipping** (`crates/engine/src/parse.rs`, `classify_lines`): Lines whose trimmed text consists entirely of punctuation/whitespace characters (`- –—|•·*\\/`) are discarded rather than becoming unclassified root nodes that absorb all subsequent content as continuation text.
- **Article regex major ≥ 1** (`article_re`): Changed `^(\d+\.\d+)` → `^([1-9]\d*\.\d+)`. Eliminates phantom articles from `0.x` decimal continuations (e.g., `"0.26 Acceptable Manufacturers"`, `"0.016 inches thick wall"`).
- **Article regex uppercase title** (`article_re`): Changed `(\S.*)` → `([A-Z].*)`. Requires article title to start with uppercase letter, excluding decimal measurements and lowercase continuation fragments.
- **`inject_missing_parts` recovery pass** (`crates/engine/src/parse.rs`): New post-classification pass inserts synthetic `PART N` flat items when an article's major number jumps to a part that was never explicitly opened. Recovers from (a) PART heading lines broken by PDFium kerning artifacts and (b) segmenter cutting section one page too early. Corrected 13/70 structured sections that had wrong-PART article nesting.
- **Test isolation** (`apps/backend-cli/tests/cli_integration_test.rs`): `cli_segment_and_visualize_segments_spec_pdf` now clears the vis directory before the visualize call. Previously accumulated PNGs from prior runs inflated the count check.

### Summary Statistics (post-hardening, `SPEC_RWB_LHHS_ALL_ORG.pdf`, 571 pages)

| Metric | Before hardening | After hardening |
|---|---|---|
| Sections detected | 16 | 89 |
| Total AST nodes | 218 | 7,971 |
| Unclassified node rate | 11% | 0.2% |
| Wrong-PART sections | 13/70 | 0/70 |
| Integration tests | 19/19 | 19/19 |

## [2026-04-04] Phase 0.5 Phases B–E

### Added
- `pattern-dev` binary in `tools` crate alongside `classify-pdf`; multi-binary crate restructure with shared `tools/src/` module directory (Phase B).
- Real `PdfiumExtractor::extract_page()` implementation in `crates/pdf-extraction/src/extractor.rs`; replaces stub with actual pdfium-render object API calls returning `PageData` with `SpanData`, `RawBBox`, width/height in pts (Phase C, closes G-004).
- Shared pattern model in `tools/src/pattern_model.rs`: `PatternSpec`, `HeuristicFamily` (6 variants with stable kebab-case identifiers), `RegionBand`, `NormalizedBBox`, `MatchEvidence`, `MatchedSpan`, `FailureCode`, `SourceTag` — all serde-serializable, all unit-tested (Phase D, 9 tests green).
- `pattern-dev test-pattern --family <FAMILY> --output-dir <DIR>` single-PDF detection loop with real per-page sidecar JSON (schema version `0.5.0`) and PASS/WARN/FAIL/SKIP reporting; runtime-ready families: `footer-section-id`, `page-counter`, `header-band`; schema-only families emit `"source": "schema-only"` sidecars (Phase E).
- `pattern-dev inspect --spans` diagnostic flag: dumps per-span text, raw bbox coords, and normalized mid-Y for every page object (with bounds-failure count), enabling band-threshold diagnosis on failing pages.
- `pattern-dev validate-corpus --tier 1 [--tier 2] --dry-run` fixture inventory pass enumerating all PDFs in requested tiers; holdout prohibition enforced at argument validation.
- `--dry-run` flag on `test-pattern` validated: argument wiring confirmed, no file writes occur.

### Changed
- Text extraction in `pattern-dev test-pattern` uses `page.text().chars()` (PDFium `FPDFText_LoadPage` / `FPDFText_GetCharBox` pipeline) rather than `page.objects().iter()`. The object-level API treats PDF Form XObjects as opaque nodes; running footers and headers placed via reusable content streams are invisible to it. The FPDFText API descends transparently into Form XObjects and returns page-coordinate bounds for every character. Characters are grouped by 6 pt baseline tolerance, then reconstructed into logical lines with word-gap spacing before pattern matching.

### Notes
- Smoke-tested on `SPEC_RWB_LHHS_ALL_ORG.pdf` (571 pages): PASS=556 WARN=10 FAIL=5. All 5 failures are confirmed blank / raster insert pages (cover + TOC inserts) with no footer text layer.
- Confidence thresholds from `DEV_STANDARDS.md` enforced: `≥ 0.95` → PASS, `0.80–0.95` → WARN, `< 0.80` → FAIL.
- D-028 added to decision log documenting the FPDFText-vs-objects API choice as a permanent architectural decision.
- G-004 closed in gap register; G-005 (PDF→IR span conversion wiring in shared engine) remains open.

## [2026-03-23] Phase 0 Closeout + Postmortem Integration

### Added
- Monorepo app/crate boundaries for backend CLI and desktop GUI scaffolds.
- New shared contract and workflow crates for stable backend/frontend integration surfaces.
- Migration closeout documentation for Phase 0 and Phase D integration outcomes.
- Repository structure and migration log docs to codify architecture and boundary rules.
- This changelog for milestone tracking and release-level highlights.

### Changed
- Documentation authority and canonical references aligned to the top-level `docs/` canonical set and `docs/DOCUMENTATION_INDEX.md`.
- README revised for current workspace layout, corpus paths, and active CLI commands.
- Coverage workflow hardened to emit machine-readable reports (`cobertura.xml`, `lcov.info`) and upload them to Codecov with artifact retention.

### Removed
- Non-permanent runtime audit artifacts from source control scope.
- Legacy/debris planning and archival files outside canonical active docs.

### Notes
- This milestone represents the requested Phase 0 closeout and postmortem update checkpoint before Phase 0.5 GUI implementation work.
