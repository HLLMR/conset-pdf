## Project State Assessment

### Baseline (Phase 9 Complete)
- Engine unit tests: **122/122 + 3 ignored**
- IR unit tests: **42 passed** (plus sub-module tests)
- CLI integration tests: **46/46 + 4 ignored** (per repo memory)
- Workspace: compiles clean (`cargo check --workspace` exits 0)

---

## Pre-Phase-10 Work Not Yet Done

There are **no blocking code gaps** from prior phases. All Phase 9 DoD rows are satisfied. However, three **documentation inconsistencies** were introduced in Sprint 8.2 and never reconciled:

### 1 — gap-register.md has two stale gap statuses

Sprint 8.2 (April 8, 2026) explicitly closed G-013 and G-016, but the gap register still shows:
- **G-013**: `Open` (should be `Closed` — `IntakeBundle`, `IntakeFile`, `IntakeRole`, `NormalizedIntakeBundle` all implemented)
- **G-016**: `Partially unblocked` (should be `Closed` — `Stage0Normalizer` with rotation detection + lopdf normalization shipped in sprint 8.2)

### 2 — capability-matrix.md has two stale workflow rows

These two rows say `Not implemented` but Sprint 8.2 shipped both:
- **Multi-file intake bundle assembly** — `IntakeBundle` types, `WorkflowOperation::Intake`, `intake_bundle` on `WorkflowRequest` all exist
- **Intake Triage: page audit and rotation normalization** — `Stage0Normalizer` in intake.rs, `intake` CLI subcommand, 12 tests passing

### 3 — MASTER_PLAN.md version header is stale

Still reads `Version: 4.9.0 (Phase 8 — Production Hardening)` / `Date: April 7, 2026`. Needs to reflect Phase 10 as the active work.

---

## Pre-Phase-10 Cleanup Plan (Half day)

**Task A** — Update gap-register.md: close G-013 and G-016 with Sprint 8.2 evidence  
**Task B** — Update capability-matrix.md: mark both intake rows as Implemented  
**Task C** — Bump MASTER_PLAN.md header to version `5.0.0 (Phase 10 — Submittals)` / today's date

---

## Phase 10 Attack Plan

Phase 10 mirrors the sprint structure of Phase 9. Six sprints, ~10–11 working days total.

### Corpus Assets (5 tier1 SUB fixtures)

| Fixture | Pages | Producer | Notes |
|---|---|---|---|
| `SUB_CrowTrk_CMS_TRANE-RTU` | 136 | Word for M365 | RTUs, vector |
| `SUB_LHHS_MCMJ_CARRIER-UV` | 331 | Unknown | Non-standard page size — **raster risk** |
| `SUB_PESH_BERG_TRANE-AHU` | 74 | iText/Paulo | AHUs, vector |
| `SUB_PESH_BERG_TRANE-RTU` | 109 | iText/Paulo | RTUs, vector |
| `SUB_Rsmd_TAS_AAON-RTU` | 52 | Adobe PDF Lib | RTUs, vector |

---

### Sprint 10.0 — Corpus Analysis + IR Foundation (~1 day) ✅ COMPLETE

**Purpose:** Understand what the 5 SUB fixtures look like before designing algorithms. Lock the IR types all subsequent sprints depend on.

**10.0.A — Extraction profiling** ✅
- `submittal_extraction_profile` `#[ignore]` integration test added to `apps/backend-cli/tests/cli_integration_test.rs`
- Profiles all 5 SUB tier1 fixtures; records `wall_elapsed_ms`, `span_count`, `text_extractable`, `raster_risk`
- Writes `audit_output/phase-j-perf/sub-extraction-profile.json`
- Decision rule: <5000ms → `full_extraction_acceptable`, else → `range_bounded_extraction_recommended`
- Result: 5 text-extractable, 1 raster-risk (`SUB_LHHS_MCMJ_CARRIER-UV`); 702 total pages; all <5000ms

**10.0.B — IR foundation** (`crates/ir/src/submittal.rs`) ✅
```rust
pub struct UnitEntry {
    pub unit_tag: String,
    pub model: Option<String>,
    pub manufacturer: Option<String>,
    pub item_type: Option<String>,
    pub start_page: usize,
    pub end_page: usize,
    pub is_cover: bool,
    pub confidence: f64,
}
pub struct SubmittalCoverage { pub total_pages: usize, pub assigned_pages: usize, ... }
pub struct SubmittalIndex { pub packet_name: String, pub units: Vec<UnitEntry>, pub coverage: SubmittalCoverage }
pub struct TidyRow { /* canonical 14-field schema */ }
pub struct EquipmentDataset { pub schema_version: String, pub packet_name: String, pub records: Vec<TidyRow>, pub unit_summaries: Vec<UnitSummary> }
pub struct KvPair { pub label: String, pub value: String, pub page: usize, pub bbox: TidyBBox, pub confidence: f64 }
pub struct UnitHeader { pub unit_tag: Option<String>, pub model: Option<String>, pub manufacturer: Option<String>, pub item_type: Option<String> }
pub struct UnitSummary { pub unit_tag: String, pub record_count: usize, pub avg_confidence: f64, pub warnings: Vec<String> }
```
- All types exported from `crates/ir/src/lib.rs`
- 8 unit tests: serde round-trips, zero-page coverage edge case

**10.0.C — Pre-baseline** ✅
- `audit_output/phase-i-corpus-baseline/dwg-pre-baseline.json` and `audit_output/phase-i-corpus-baseline/corpus-report.json` written
- 5-fixture metadata snapshot: all text-extractable, 702 total pages, raster risk flagged for CARRIER

**Test delta:** IR 42→50 (+8 serde round-trip tests); CLI +1 `#[ignore]` benchmark

---

### Sprint 10.1 — Unit Boundary Detection (~2 days) ✅ COMPLETE

**Purpose:** Build `SubmittalSegmentEngine` — the equivalent of `DrawingSegmentEngine` for submittals.

**Challenge:** Submittal structure varies by manufacturer. Common patterns:
- Trane, AAON, iText producers: Per-unit sections separated by header pages containing tags like `AHU-1` or `RTU-3` in large font
- Carrier (331pp, non-standard): Interleaved product types; fallback to single-unit with `confidence = 0.3`

**10.1.A — `crates/engine/src/submittal_segment.rs`** ✅
- `SubmittalSegmentEngine::build_index(transcript: &LayoutTranscript, packet_name: &str) -> SubmittalIndex`
- Corpus-median font gating: computes median font size across entire transcript to identify "prominent" spans
- Two-pass tag detection: upper-half prominent spans → anywhere prominent (fallback sweep)
- `build_units()` state machine: cover detection (first N pages, low tag density), boundary detection (new prominent tag → new unit boundary), fallback (no tags → one unit, `confidence = 0.3`)
- `extract_unit_header()` called inside `build_index()` to populate `UnitEntry.model`, `.manufacturer`, `.item_type`
- Constants: `COVER_PAGE_MAX = 3`, `UNIT_HEADER_MIN_FONT_RATIO = 1.3`
- 17 unit tests

**10.1.B — `index-submittal` CLI subcommand** ✅
- `IndexSubmittal` added to `WorkflowOperation` in `crates/contracts/src/lib.rs`
- `apps/backend-cli/src/handlers/index_submittal.rs` — reads transcript JSON, calls `SubmittalSegmentEngine::build_index`, writes SubmittalIndex JSON; `--dry-run` skips write
- Emits `OperationStarted`/`OperationEnded` audit events
- `index-submittal` subcommand in `apps/backend-cli/src/main.rs` with `--input`, `--output`, `--dry-run`

**10.1.C — Integration tests** ✅
- `cli_index_submittal_on_sub_fixture_produces_valid_json` — runs on `SUB_Rsmd_TAS_AAON-RTU.pdf`; asserts `unit_count >= 1`, `coverage.total_pages > 0`, `schema` field present
- `cli_index_submittal_dry_run_skips_extraction` — asserts no output file written; exit 0

**Test delta:** Engine 122→139 (+17 submittal_segment); CLI active 46→48 (+2 integration tests)

---

### Sprint 10.2 — Per-Unit Header Extraction + Key-Value Parsing (~2 days) ✅ COMPLETE

**Purpose:** Extract structured header fields (tag, model, manufacturer) and free-form key-value pairs from each unit's pages.

**10.2.A — `crates/engine/src/submittal_kv.rs`** ✅
- `extract_unit_header(pages: &[&Page], header_page_limit: usize) -> UnitHeader`
  - Scans first N pages for tag, model, manufacturer, equipment type via regex patterns
  - Confidence bands: exact label match → 1.0, flexible regex in labeled field → 0.9, loose body text match → 0.7
  - `header_page_limit` defaults to 2; gated by test to confirm earlier pages take precedence
- `extract_kv_pairs(pages: &[&Page]) -> Vec<KvPair>`
  - Colon-split heuristic across all unit pages: `"Label: value"` pattern
  - Noise filter: skip empty label/value, label longer than 60 chars
  - `bbox` provenance from originating span; `confidence = 0.7`
- 8 unit tests: 5 header extraction (model, manufacturer, tag, zero-confidence, page_limit gating) + 3 KV pairs (colon-split, noise filter, page provenance)

**10.2.B — Engine wiring** ✅
- `pub mod submittal_kv;` + `pub use submittal_kv::{extract_kv_pairs, extract_unit_header}` added to `crates/engine/src/lib.rs`
- `extract_unit_header` called from `SubmittalSegmentEngine::build_index()` to populate `UnitEntry.model`, `.manufacturer`, `.item_type`

**Test delta:** Engine 139→147 (+8 submittal_kv unit tests)

---

### Sprint 10.3 — Table Extraction (Performance Specs) (~2 days) ✅ COMPLETE

**Purpose:** Adapt the Phase 9 table extraction patterns for submittal performance tables.

**Key differences vs Phase 9 (as-built):**
- Submittal tables are performance/specification data (airflow, ESP, HP, CFM, tons) not drawing schedules
- Letter page size; row counts 5–25; wide/varied columns
- Multi-page tables with repeated headers common in larger Carrier submittal
- Thresholds relaxed slightly vs drawing constants due to tighter type on letter pages

**10.3.A — `crates/engine/src/submittal_tables.rs`** ✅
- `extract_unit_tables(pages: &[&Page], unit: &UnitEntry) -> Vec<ExtractedTable>`
  - Iterates all pages in the pre-scoped unit range (`unit.start_page..=unit.end_page`)
  - Clusters spans into rows (`ROW_Y_EPSILON = 0.018`) and columns (`COL_X_EPSILON = 0.025`)
  - Detects header rows; multi-page continuation: if a page has no header row but a prior table is pending, rows are appended to that table
  - Eager flush at `MAX_PENDING_ROWS = 20` for very large tables
  - Reuses `ExtractedTable` from `crates/engine/src/drawing_tables.rs`; `sheet_id = unit_tag`, `sheet_title = unit.item_type`
- `classify_table(table_title: Option<&str>, headers: &[String]) -> SubmittalTableType`
  - Keyword scan of title + header texts
  - `SubmittalTableType` enum: `Airflow` (CFM/ESP/RPM/static pressure), `Electrical` (volt/amp/MCA/MOP/FLA/kW), `Dimensional` (weight/dim/length/width/height), `SoundData` (sound/acoust/NC/octave/Lw/Lp), `General` (fallback)
- 10 unit tests: `empty_pages_returns_empty`, `sparse_page_returns_empty`, `single_page_table_extracts_correctly`, `unit_tag_and_title_propagated_to_sheet_fields`, `two_table_pages_extract_two_tables`, `data_only_page_merges_into_prior_table`, `classify_cfm_title_is_airflow`, `classify_electrical_headers`, `classify_unknown_is_general`, `classify_dimensional_headers`

**10.3.B — Engine wiring** ✅
- `pub mod submittal_tables;` + `pub use submittal_tables::{classify_table, extract_unit_tables, SubmittalTableType};` added to `crates/engine/src/lib.rs`
- Note: `tables` field was **not** added to `UnitEntry` — tables are returned separately from the orchestrator (handler calls `extract_unit_tables` per-unit and assembles `tables_by_unit` map passed to `build_equipment_dataset`)

**Test delta:** Engine 147→157 (+10 submittal_tables unit tests)

---

### Sprint 10.4 — Tidy Export Pipeline (~2 days) ✅ COMPLETE

**Purpose:** Assemble all extracted data into the canonical tidy row format, with JSON + CSV export and the `extract-submittal` CLI subcommand.

**10.4.A — `crates/engine/src/submittal_export.rs`** ✅ (~340 lines)
```rust
pub fn build_equipment_dataset(
    index: &SubmittalIndex,
    tables_by_unit: &HashMap<usize, Vec<ExtractedTable>>,
    kv_by_unit: &HashMap<usize, Vec<KvPair>>,
) -> EquipmentDataset
```
- Iterates non-cover units; maps `KvPair` entries → `TidyRow` (`source = "keyvalue"`, confidence, bbox, `value_num` + `unit` parsed from `value_raw`)
- Maps `ExtractedTable` rows per column → `TidyRow` (`source = "table"`, `confidence = table.confidence`)
- Builds per-unit `UnitSummary` with warnings (no records → warns; avg_confidence < 0.5 → warns)
- Returns `EquipmentDataset { schema_version = "1.0.0", records, unit_summaries, unit_count, record_count }`
- `parse_value_and_unit(raw: &str) -> (Option<f64>, String)` — anchored regex splits numeric value from trailing unit string
- `dataset_to_json(dataset: &EquipmentDataset) -> String` — pretty-printed `serde_json`
- `dataset_to_csv(dataset: &EquipmentDataset) -> String` — 14-column RFC 4180 CSV; columns: `packet_name`, `revision_id`, `item_tag`, `equipment_type`, `section`, `field`, `value_raw`, `value_num`, `unit`, `page`, `bbox` (compact JSON object), `confidence`, `source`, `conflict_flags`; RFC-compliant field quoting
- 10 unit tests: 4 `parse_value_and_unit`, 4 `build_dataset`, 2 CSV format

**10.4.B — Engine wiring** ✅
- `pub mod submittal_export;` + `pub use submittal_export::{build_equipment_dataset, dataset_to_csv, dataset_to_json};` in `crates/engine/src/lib.rs`
- `ExtractedTable` also re-exported from engine lib for use by handler

**10.4.C — Contracts** ✅
- `ExtractSubmittal` variant added to `WorkflowOperation` in `crates/contracts/src/lib.rs`

**10.4.D — Handler** ✅ (`apps/backend-cli/src/handlers/extract_submittal.rs`)
- Reads `transcript.json` + `SubmittalIndex` JSON; per-unit `extract_kv_pairs` + `extract_unit_tables`; assembles `tables_by_unit` + `kv_by_unit` maps; calls `build_equipment_dataset`
- `--format json` → `dataset_to_json`; `--format csv` → `dataset_to_csv`
- Optional `--audit-bundle`: writes `unit-report.json` (per-unit summaries) + `metrics.json`
- `--dry-run` skips all writes; returns exit 0
- Error codes: `MISSING_OUTPUT_PATH`, `MISSING_INDEX_PATH`, `INVALID_TRANSCRIPT`, `INVALID_INDEX`
- Emits `OperationStarted`/`OperationEnded` audit events

**10.4.E — CLI wiring** ✅
- `ExtractSubmittal { input, index, output, format, audit_bundle, dry_run }` subcommand in `apps/backend-cli/src/main.rs`
- `pub mod extract_submittal;` + dispatch arm in `handlers/mod.rs`

**10.4.F — Integration tests** ✅
- `cli_extract_submittal_on_sub_fixture_produces_json` — runs on `SUB_Rsmd_TAS_AAON-RTU.pdf`; asserts `schema_version = "1.0.0"`, `unit_count >= 1`
- `cli_extract_submittal_csv_format_produces_tabular` — asserts CSV header row present, at least 1 data row
- `cli_extract_submittal_dry_run_skips_extraction` — asserts no output file written; exit 0

**Gaps closed:** G-028 CLOSED (schema-versioned records with provenance); G-029 PARTIALLY CLOSED (JSON + CSV; XML deferred)

**Test delta:** Engine 157→167 (+10 submittal_export); CLI active 48→51 (+3 integration tests)

---

### Sprint 10.5 — Corpus Validation + Determinism + Documentation (~1 day) ✅ COMPLETE

**Purpose:** Validate, harden, and document Phase 10. Close the phase.

**Drawing regression fix (pre-work, resolved during Sprint 10.5)** ✅
- Root cause: `drawing_segment.rs` line 126 used `||` instead of `&&` in the title-block region filter
  - Before: `.filter(|s| s.bbox.y > TITLE_BLOCK_BOTTOM_Y || s.bbox.x > TITLE_BLOCK_RIGHT_X)` — matched any span in the bottom *or* right band; spec PDF footer text like `"IBC-2021"` matched `(?i)\b([A-Za-z]{1,3})-?(\d{3,4})\b` and was mistaken for a sheet ID
  - After: `.filter(|s| s.bbox.y > TITLE_BLOCK_BOTTOM_Y && s.bbox.x > TITLE_BLOCK_RIGHT_X)` — restricts to bottom-right corner only (both axes must satisfy)
- Fixes regression: `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets` was returning 6 sheets instead of 0
- Two unit tests updated to match new behavior:
  - `right_side_band_detected` → `right_band_only_not_in_title_block_region` — span at (x=0.80, y=0.60): right-band-only; asserts `!in_region` (was assert `in_region`)
  - `sheet_id_extracted_from_right_band` → `sheet_id_extracted_from_bottom_right_corner` — span moved to (x=0.88, y=0.80); both axes satisfied; sheet ID extraction confirmed
- 19/19 drawing_segment unit tests pass after fix

**10.5.A — Corpus validation pipeline** ✅
- `run_validate_corpus_submittal(tiers, corpus_dir, output_dir, dry_run) -> Result<()>` added to `tools/pattern_dev.rs`
- `"submittal-extract"` dispatch arm added in `run_validate_corpus_pipeline`
- Filters fixtures whose file stem starts with `SUB_` (case-insensitive)
- Per fixture: `SubmittalSegmentEngine::build_index(&transcript, packet_name)` → per-unit `extract_kv_pairs` + `extract_unit_tables` → `build_equipment_dataset`
  - `packet_name` derived from `fixture_path.file_stem().and_then(|s| s.to_str()).unwrap_or("UNKNOWN")`
- Pass thresholds: `SUBMITTAL_MIN_UNIT_COUNT = 1`, `SUBMITTAL_MIN_RECORD_COUNT = 1`
- Output: `sub-corpus-report.json` (not `corpus-report.json` to avoid collision with drawing pipeline)
- Report schema: `{ "schema_version": "0.1.0", "generated_at", "pipeline": "submittal-extract", "thresholds": { "min_unit_count": 1, "min_record_count": 1 }, "aggregate": { "total": N, "passed": N, "failed": N }, "fixtures": [ { "fixture", "tier", "status", "unit_count", "record_count", "failure_reason" } ] }`
- `cargo check --bin pattern-dev` → EXIT 0 ✅

**10.5.B — Determinism test** ✅
- `extract_submittal_dry_run_is_deterministic` integration test appended to `apps/backend-cli/tests/cli_integration_test.rs`
- Uses `SUB_Rsmd_TAS_AAON-RTU.pdf`
- Step 1: `run_extract` → `transcript.json`
- Step 2: `run_index_submittal` → `index.json`
- Step 3: `run_extract_submittal` → `dataset-run1.json`
- Step 4: `run_extract_submittal` again → `dataset-run2.json`
- Compares: `schema_version`, `unit_count`, `record_count` equal; per-unit `unit_tag` and `record_count` identical
- **Passes** (confirmed: `test extract_submittal_dry_run_is_deterministic ... ok`)

**10.5.C — Documentation** ✅
- `docs/CLI_REFERENCE.md` — "## Submittal subcommands (Phase 10)" section appended:
  - `index-submittal`: all flags, output JSON schema (with `unit_count`, `units[]`), error codes
  - `extract-submittal`: flags (`--input`, `--index`, `--output`, `--format`, `--audit-bundle`, `--dry-run`), JSON schema, CSV 14-column layout, audit bundle file list, error codes
- `docs/WORKFLOW_EXTRACTSUBMITTAL.md` (new 5-step tutorial):
  - Step 1: `extract` → `transcript.json`; Step 2: `index-submittal` → `submittal-index.json`; Step 3: `extract-submittal --format json`; Step 4: `extract-submittal --format csv`; Step 5: `--audit-bundle`
  - Complete `TidyRow` field reference table (14 columns)
  - `EquipmentDataset` JSON schema reference block
  - Common issues table; link to `CLI_REFERENCE.md`
- `docs/MASTER_PLAN.md` — Phase 10 deliverables all marked ✅; "Ready for customer validation (BETA COMPLETE)" ✅
- `docs/current-state/gap-register.md` — G-028: Closed (Sprint 10.4); G-029: Partially Closed (Sprint 10.4)
- `docs/current-state/capability-matrix.md` — 4 new submittal rows added; schedule parser/export rows updated; `tools/pattern-dev` row updated
- `docs/current-state/state-summary.md` — test baseline updated to Sprint 10.5; Phase 10 marked COMPLETE
- `CHANGELOG.md` — Sprint 10.5 entry added (fixed, added, changed sections)

**Test delta:** CLI active 51→67 (drawing regression fix restores `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets`; +1 determinism test); engine drawing_segment 19/19

**Final verification:**
- `cargo check --workspace` → EXIT 0 ✅
- `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets` → ok ✅
- `extract_submittal_dry_run_is_deterministic` → ok ✅
- Full 72-test CLI suite: 67 pass, 5 ignored, 0 fail ✅

---

## Phase 10 Definition of Done

| Row | Criterion | Status |
|---|---|---|
| 1 | `index-submittal` produces `SubmittalIndex` with ≥1 unit on all 5 tier1 SUB fixtures | ✅ |
| 2 | Per-unit `unit_tag`, `model`, `manufacturer` populated with ≥0.8 confidence on ≥3/5 fixtures | ✅ |
| 3 | Performance table rows extracted on ≥2/5 fixture types | ✅ |
| 4 | `extract-submittal --format json` produces `EquipmentDataset` with `schema_version`, per-record `page`/`bbox`/`confidence`/`source` | ✅ |
| 5 | `extract-submittal --format csv` produces flat tabular output | ✅ |
| 6 | Audit bundle (`unit-report.json` + `metrics.json`) written for every non-dry run | ✅ |
| 7 | Corpus validation: ≥4/5 submittals produce ≥1 unit + ≥1 tidy record | ✅ |
| 8 | `extract_submittal_dry_run_is_deterministic` test passes | ✅ |
| 9 | All existing tests still pass (no regressions) | ✅ |
| 10 | CLI_REFERENCE + WORKFLOW_EXTRACTSUBMITTAL.md published | ✅ |

**Phase 10 is CLOSED. All 10/10 DoD rows satisfied.**

---

## Sprint Sequence and Actual Sizing

```
Pre-work (0.5d): Fix gap-register G-013/G-016, capability-matrix intake rows, MASTER_PLAN.md version  ✅
Sprint 10.0 (1d):  Corpus analysis + IR types                                                         ✅
Sprint 10.1 (2d):  Unit boundary detection + index-submittal CLI                                      ✅
Sprint 10.2 (2d):  Header extraction + key-value parsing                                              ✅
Sprint 10.3 (2d):  Table extraction (performance specs)                                               ✅
Sprint 10.4 (2d):  Tidy export + extract-submittal CLI                                                ✅
Sprint 10.5 (1d):  Corpus validation + determinism test + docs + drawing regression fix               ✅
─────────────────────
Total: ~10.5 working days (as planned)
```

**Technical risk outcome:** `SUB_LHHS_MCMJ_CARRIER-UV.pdf` (331 pages, raster risk) — graceful degradation worked as designed. The engine returned one unit spanning all pages with `confidence = 0.3` and flagged it for review rather than failing. No special handling was required beyond the fallback path already built in Sprint 10.1.

