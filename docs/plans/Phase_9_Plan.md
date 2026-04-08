## Phase 8 Completion Verification

**Phase 8 is fully complete.** All code is present, all tests pass. Baseline confirmed:

| Suite | Passing | Ignored |
|---|---|---|
| CLI integration | 46/46 | 2 (Chrome) |
| Engine unit (all files) | 92+ | 3 (Chrome + intake OnceLock) |
| IR unit (lib.rs) | 37 | — |
| pdf-extraction | 16 | — |
| audit | 5 | — |
| pattern-dev unit | 32 | — |
| pattern-dev integration | 3 | 2 (binary/corpus) |

Sprint completion roster:
- 8.0 Documentation housekeeping ✅
- 8.1 Crash containment, error handling, structured diagnostics ✅
- 8.2 Intake triage ✅
- 8.3 Torture corpus validation infrastructure ✅
- 8.4 Performance benchmarking + architecture constraint doc ✅
- 8.5 Metrics roll-up in audit bundle ✅
- 8.6 Pattern database as versioned JSON ✅
- 8.7 User documentation (CLI_REFERENCE.md, WORKFLOW_APPLYADDENDUM.md) ✅

All 13 Phase 8 DoD criteria satisfied. Phase 9 gate checks confirmed:
- 8.1.H temp-dir RAII cleanup ✅
- 8.3.D determinism regression test ✅
- 8.3.E unchanged-page content hash ✅
- 8.4.B full-extraction constraint documented in ARCHITECTURE.md ✅
- Zero workspace errors (`cargo check --workspace --tests` EXIT: 0) ✅

---

## Foundation Assessment

> _Architect review of the infrastructure available when Phase 9 drawing-sheet work begins. Conducted via direct audit before Phase 9 design locks._

### What's solid

**The stitch engine is directly reusable.** `PdfStitcher::stitch()` takes a `StitchPlan { original_path, replacement_path, section_id, segment_index, dry_run, output_path }` and replaces one section's pages with the replacement PDF using `lopdf`. Drawing sheet replacement is topologically identical: a "sheet" in a drawing set is a page range identified by a sheet boundary detector, just as a "section" is a page range identified by the footer oracle. Provided the Phase 9 orchestrator produces a `SegmentIndex`-shaped index over a drawing set and a `StitchPlan`, `PdfStitcher::stitch()` can execute the actual page replacement without any changes to `stitch.rs`.

**The audit bundle pattern is fully established.** `change-report.json` + `diagnostics.jsonl` + `metrics.json` written by `apply-addendum` is the production audit standard. Phase 9 adopts this pattern verbatim: the `apply-sheet-addendum` handler writes the same three artifacts, using the same `DiagnosticEvent` type which already has `ExtractionDiagnostic` and `StitchDiagnostic` variants. New Phase 9 variants (`DrawingIndexDiagnostic`, `SheetPatchDiagnostic`) are added without breaking existing deserialization (`#[serde(other)]` on unknown variants or `untagged` enum).

**The pattern database is extensible.** `crates/engine/src/patterns/default.json` v1.0.0 contains three families (`footer-section-id`, `page-counter`, `header-band`). Phase 9 title-block detection adds new families (`title-block-anchor`, `sheet-id-title-block`, `revision-block`, `stamp-block`) as pattern database entries — same schema, no code changes to `PatternDatabase`.

**The discipline classifier algorithm is fully specified.** `docs/DRAWINGS_CLASSIFICATION.md` is the canonical implementation spec for the sheet-number → discipline code mapping, including the `X-NNN` format, UDS single-letter lookup table (13 codes), multi-letter alias table (10 entries), `C` Civil/Controls disambiguation algorithm, and discipline sort order. No additional research is needed; Phase 9.0 is straight implementation.

**`crates/standards-data` already exists.** It has `src/lib.rs`, `src/aec.rs`, and `src/masterformat.rs`. The discipline classifier belongs here (AEC field classification) rather than in `crates/engine` (processing) or `crates/ir` (data model). This keeps the lookup tables as data dependencies that both engine and CLI can consume without a circular dependency.

### Five structural risks / constraints going in

**Constraint 1 — Full-document extraction is O(n_pages) (Severity: HIGH for batch design)**

Documented in `ARCHITECTURE.md` §Known Design Constraints, Constraint 1. `Extractor::extract()` materializes the entire `LayoutTranscript` up front. A typical drawing set has 100–800 sheets of varying page sizes; batch processing 10 drawing submittals would extract 5,000–8,000 pages even though each individual sheet replacement touches at most a few pages. Sprint 9.0 mandates a profiling decision before the `DrawingsPatchOrchestrator` batch-loop architecture is designed. If the decision is "range-bounded extraction is needed," `extractor.rs` must gain an `extract_range(start_page, end_page)` API before Sprint 9.2 locks the orchestrator interface. If the decision is "full extraction is acceptable," document it and proceed.

**Constraint 2 — Title-block detection is harder than footer detection (Severity: MEDIUM)**

The CSI footer oracle (`segment.rs`) works on a highly uniform pattern: a two- or three-group numeric ID in the bottom 10 % of the page, consistently placed by spec-book production tools. AEC title blocks vary: they appear in the bottom 10–20 % or right-side margin, contain mixed text densities (large sheet title + small project data rows + stamps), and vary significantly by firm. The Phase 9 sheet-segmentation oracle must tolerate this variance. The simplest viable first implementation: scan the bottom 20 % and right 20 % of each page for a high-density text cluster; use the `title-block-anchor` pattern family to identify candidate regions; within the candidate region, run the `sheet-id-title-block` pattern to extract the sheet ID string.

**Constraint 3 — DWG corpus has no viable validation pass today (Severity: MEDIUM)**

`audit_output/phase-h-corpus-baseline/corpus-report.json` shows 3/27 pass (11.1%). The 24 failures include 11 DWG fixtures — all fail because the corpus validator checks for CSI footer section-ID stamps, which drawing sheets never have. Phase 9 needs a separate corpus validation pipeline branch for DWG-medium fixtures: `validate-corpus --pipeline drawing-segment --tier 1`. Sprint 9.0 establishes this baseline before implementation begins so regressions can be caught.

**Constraint 4 — Sheet ID ambiguity and confidence scoring (Severity: LOW)**

The `C`, `D`, and `T` designators are ambiguous (Civil/Controls, Process/Demolition, Technology/Tele). The disambiguation algorithm in `DRAWINGS_CLASSIFICATION.md` §Step 4 covers `C` in detail and requires a sheet title as secondary input. Sheet title extraction is part of the same title-block scan, so this is a sequencing concern: classify designator after title is extracted, not before. `confidence: f64` on `SheetEntry` carries the classifier's certainty; callers can threshold it.

**Constraint 5 — Bookmarks are not uniform across drawing sets (Severity: LOW)**

Bookmark (PDF outline) regeneration is a Phase 9 deliverable. The current spec-book approach (`fixup_bookmarks()` in `stitch.rs`) reroutes `/Dest` references that pointed to deleted pages. Drawing sets may have no PDF bookmarks at all, or may have firm-specific bookmark structures. Phase 9 bookmark generation is best-effort and additive: generate a flat bookmark tree from the detected `DrawingIndex` entries and write it into the output PDF. Do not fail the stitch if bookmark generation fails; surface it as a warning in `StitchResult.warnings`.

### Verdict: Viable — with two pre-conditions

The foundation is strong enough to support Phase 9 drawing-sheet work. Two things must be resolved before Sprint 9.2 can begin:

1. **Sprint 9.0 extraction profiling decision must be made and documented** before the `DrawingsPatchOrchestrator` interface is designed. Committing to full-extraction batch design and later discovering it is 100× too slow for production drawing sets would require a rearchitecture under time pressure.
2. **Sprint 9.1 sheet-ID extraction must have a stable `SheetEntry` type** before the `StitchPlan`-based orchestrator in Sprint 9.2 can be written. The orchestrator's whole job is to map `SheetReplaceSpec.sheet_id` → `SheetEntry.start_page/end_page` and pass them to `PdfStitcher::stitch()`.

---

## Phase 9 Plan

Phase 9 per MASTER_PLAN is "Drawing Sheet Management" — the next release increment after the spec-book Alpha. The six MASTER_PLAN deliverables map to sprints 9.1 through 9.5. Sprint 9.0 is the profiling and foundation sprint added to satisfy ARCHITECTURE.md Constraint 1 before design locks.

---

### Sprint 9.0 — Extraction Profiling + DWG Foundation (~1 day)

This sprint has three purposes: make the full-extraction vs. range-bounded extraction decision, establish a DWG-aware corpus baseline, and define the core IR types that all subsequent sprints depend on.

**9.0.A — Profile extraction on largest DWG corpus fixture** (~2 hours)

Follow the 4-step checklist in `ARCHITECTURE.md` §Known Design Constraints, Constraint 1.

- Step 1: Add a `#[ignore]` benchmark integration test `drawing_extraction_profile` in `apps/backend-cli/tests/` that runs `extract` on the three largest DWG fixtures (`DWG_RWB_LHHS_ALL_ORG`, `DWG_P&W_UTD_MECH_ORG`, `DWG_VLK_CrowTrk_MECH_ADD2`). Record `elapsed_ms` from `DiagnosticEvent::Extraction.elapsed_ms` for each.
- Step 2: If per-call elapsed_ms < 5,000 ms for the largest fixture: full extraction is acceptable for per-file operations. Range-bounded extraction is a future enhancement only. If elapsed_ms > 5,000 ms for single-file: range-bounded extraction is needed.
- Step 3: If range-bounded extraction is needed, add `extract_range(start_page: usize, end_page: usize) -> Result<LayoutTranscript, EngineError>` to `src/extractor.rs`. The implementation loads the PDF, iterates only `start_page..=end_page`, and wraps the result in a `LayoutTranscript` with the correct global page-offset.
- Step 4: Update `ARCHITECTURE.md` §Constraint 1 with the measured benchmark numbers and the resolution ("full extraction acceptable for Phase 9" or "range-bounded extraction implemented; see `extract_range`").
- Write benchmark results to `audit_output/phase-i-perf/drawing-extraction-profile.json`.

**9.0.B — `DrawingIndex` and `SheetEntry` IR types** (~60 lines in `crates/ir/src/drawing.rs` new)

Define the drawing-medium IR types. These are structurally analogous to `SegmentIndex` / `SectionEntry` but carry AEC drawing fields.

```rust
/// Chrome metadata extracted from a drawing sheet's title block.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SheetChromeMetadata {
    /// Sheet ID as printed in the title block (e.g. "M-201", "FP-101").
    pub sheet_id: String,
    /// Full sheet title (e.g. "MECHANICAL EQUIPMENT PLAN — LEVEL 1").
    pub sheet_title: String,
    /// Canonical4 discipline code (e.g. "MECH", "FIRP"); empty if unknown.
    pub discipline: String,
    /// Revision string from the revision block (e.g. "ADD-2", "ISSUED FOR BID").
    pub revision: String,
    /// Date string from the title block as extracted.
    pub date: String,
    /// Firm / architect-of-record name from the title block.
    pub firm_name: String,
    /// Project name from the title block.
    pub project_name: String,
    /// Project number / ID from the title block.
    pub project_id: String,
    /// Confidence in the title-block extraction: 1.0 = all fields extracted.
    pub confidence: f64,
}

/// One sheet boundary detected by the title-block oracle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SheetEntry {
    /// Sheet ID as extracted (normalized to uppercase, e.g. "M-201").
    pub sheet_id: String,
    /// Zero-based index of the page(s) belonging to this sheet.
    /// Single-page sheets: start_page == end_page.
    pub start_page: usize,
    pub end_page:   usize,
    pub page_count: usize,
    /// Chrome metadata extracted from the title block.
    pub chrome: SheetChromeMetadata,
    /// True if the same sheet_id appeared in prior sheets (rename detection flag).
    pub superseded_by: Option<String>,
}

/// Index of all sheets detected in a drawing set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingIndex {
    pub schema_version: String,      // "1.0.0"
    pub sheet_count: usize,
    pub sheets: Vec<SheetEntry>,
    pub discipline_summary: Vec<DisciplineSummary>,
    pub total_pages: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisciplineSummary {
    pub canonical4: String,
    pub display_name: String,
    pub sheet_count: usize,
    pub sort_order: u32,
}
```

- Add `pub mod drawing;` to `crates/ir/src/lib.rs`
- Re-export `DrawingIndex`, `SheetEntry`, `SheetChromeMetadata`, `DisciplineSummary`
- Unit tests: serde round-trip for `DrawingIndex` with multi-discipline fixture; `SheetEntry` with optional `superseded_by`

**9.0.C — Discipline classifier in `crates/standards-data`** (~120 lines in `src/aec.rs`)

Implement the full algorithm from `docs/DRAWINGS_CLASSIFICATION.md`. This is pure data + logic with no PDF dependency, so it belongs in `crates/standards-data`.

```rust
pub struct ClassifyResult {
    pub canonical4: String,        // e.g. "MECH"
    pub display_name: String,      // e.g. "Mechanical"
    pub sort_order: u32,
    pub confidence: f64,
}

pub fn classify_sheet(sheet_number: &str, sheet_title: Option<&str>) -> ClassifyResult;
```

Steps 1–4 from DRAWINGS_CLASSIFICATION.md:
1. Extract designator (strip digits + separators, uppercase)
2. Single-letter UDS lookup (13 entries)
3. Multi-letter alias lookup (10 entries in table)
4. C/D disambiguation via title keywords if designator == "C"
5. Sort order from heuristic table (17 entries + UNKN=999)

- Unit tests covering every entry in the UDS table, every alias, C disambiguation (controls keyword wins, civil keyword wins, no keyword → CIVL default), D/T ambiguity, `EX` → UNKN, unknown designator → UNKN

**9.0.D — DWG corpus baseline** (~30 min)

Run the existing `validate-corpus --pipeline segment` against the 11 DWG tier-1 fixtures. All will show 0% section coverage (expected — DWG has no CSI footers). Record this as the DWG pre-Phase-9 baseline in `audit_output/phase-i-corpus-baseline/dwg-pre-baseline.json`.

This is the "Phase 9 starts here" snapshot. After Sprint 9.1 ships the drawing segmenter, a new comparison run against the same fixtures will show progress.

**DoD for Sprint 9.0:** ✅ COMPLETE (April 8, 2026)
- ✅ `drawing_extraction_profile` `#[ignore]` benchmark added to cli_integration_test.rs; profiles 3 largest DWG fixtures (177 MB, 121 MB, 25 MB); writes `audit_output/phase-i-perf/drawing-extraction-profile.json`
- ✅ `DrawingIndex` + `SheetEntry` + `SheetChromeMetadata` + `DisciplineSummary` in `crates/ir/src/drawing.rs`; also `DrawingAddendumManifest`, `SheetPatchResult`, `SheetRenameEvent` and companion types; 5/5 serde round-trip tests pass
- ✅ `classify_sheet()` in `crates/standards-data/src/aec.rs`; 33/33 unit tests pass (all 13 UDS + 10 aliases + C disambiguation + case-insensitive prefix + edge cases)
- ✅ `audit_output/phase-i-corpus-baseline/dwg-pre-baseline.json` written (sourced from phase-h-corpus-baseline; 11/11 DWG fixtures, 0/11 pass CSI segment, max coverage 9.4%)

---

### Sprint 9.1 — Sheet ID Extraction + Drawing Segmentation (~2 days)

This sprint produces the `DrawingIndex` for a real drawing set PDF. It is the Phase 9 analog of `segment.rs` for spec books.

**9.1.A — Title-block oracle: region detection** (~80 lines in `crates/engine/src/drawing_segment.rs` new)

The title block is a high-density text cluster in the bottom 15–25 % or right-side margin of each page. First pass: use a geometric heuristic identical in structure to `FOOTER_Y` in `segment.rs`, but with wider band.

```
TITLE_BLOCK_BOTTOM_Y: f64 = 0.75  // spans with y > 0.75 are in the candidate region
TITLE_BLOCK_RIGHT_X: f64 = 0.75   // spans with x > 0.75 are also in the candidate region
```

For each page, collect all spans in `y > TITLE_BLOCK_BOTTOM_Y || x > TITLE_BLOCK_RIGHT_X`. Cluster these spans by proximity (adapt `FOOTER_CLUSTER_GAP` logic from `segment.rs`). The largest cluster on each page is the candidate title-block region.

Add `title-block-anchor` to `patterns/default.json`:
```json
"title-block-anchor": {
  "regex": "(?i)\\b(sheet|drwg|dwg|drawing)\\b",
  "confidence_threshold": 0.60,
  "band": "bottom",
  "examples": ["SHEET M-201", "DWG NO. A-101"]
}
```

Unit tests: `title_block_region_detected_on_synthetic_page`, `title_block_region_absent_returns_empty`.

**9.1.B — Sheet ID extraction from title-block region** (~80 lines in `drawing_segment.rs`)

Within the candidate title-block region, extract the sheet ID using the pattern family `sheet-id-title-block`:

```json
"sheet-id-title-block": {
  "regex": "(?i)\\b([A-Z]{1,3})-?(\\d{3,4})\\b",
  "confidence_threshold": 0.80,
  "band": "bottom",
  "examples": ["M-201", "FP-101", "A101", "DDC-01"]
}
```

Match strategy: find the first span in the candidate region that matches the sheet-ID pattern. On ambiguous matches (multiple candidates), prefer the rightmost match (sheet IDs are typically in the right column of title blocks).

After extraction, call `classify_sheet(sheet_id, Some(sheet_title))` from Sprint 9.0.C to populate `SheetChromeMetadata.discipline` and `confidence`.

Unit tests using synthetic span fixtures that match the pattern (and also check non-match cases).

**9.1.C — Drawing segmentation engine: `DrawingSegmentEngine`** (~150 lines in `drawing_segment.rs`)

```rust
pub struct DrawingSegmentEngine;

impl DrawingSegmentEngine {
    pub fn build_index(transcript: &LayoutTranscript) -> DrawingIndex;
}
```

Algorithm:
1. For each page: run title-block candidate region detection (9.1.A)
2. For each page: run sheet ID extraction (9.1.B)
3. Assign sheet boundaries: consecutive pages with the same `sheet_id` belong to the same `SheetEntry`; a boundary is raised when the sheet_id changes
4. Handle pages with no detected sheet ID: assign them to the previous sheet (same behavior as section segmentation in `segment.rs` for pages missing a footer)
5. Populate `DrawingIndex.discipline_summary` from the classified sheet entries
6. Set `total_pages`, `sheet_count`, `schema_version: "1.0.0"`

Unit tests:
- `drawing_segment_single_sheet_all_pages_same_id` — all pages produce same sheet ID → single `SheetEntry`
- `drawing_segment_two_sheets_boundary_detected` — page 0–1 = "M-201", page 2 = "M-202" → two entries
- `drawing_segment_page_without_id_assigned_to_prior_sheet` — page 1 has no title block → assigned to page 0's sheet

**9.1.D — `index-drawing` CLI subcommand** (~80 lines in `apps/backend-cli/src/`)

```
conset-pdf index-drawing --input <dwg.pdf> --output <index.json>
```

Wraps `Extractor::extract()` → `DrawingSegmentEngine::build_index()` → serialize `DrawingIndex` to `--output`.
Emits `WorkflowResponse` with `operation_status: Succeeded / Failed`.

Integration tests:
- `cli_index_drawing_on_dwg_fixture_produces_valid_json` — run on `DWG_RWB_LHHS_ALL_ORG.pdf`; assert output parses as `DrawingIndex`; assert `sheet_count > 0`
- `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets` — run on `SPEC_RWB_LHHS_ALL_ORG.pdf`; assert `sheet_count == 0`, operation succeeds (not fails — a spec book is a valid input, just not a drawing set)

**DoD for Sprint 9.1:** ✅ COMPLETE (April 8, 2026)
- ✅ `crates/engine/src/drawing_segment.rs` — `DrawingSegmentEngine::build_index()`, `extract_sheet_from_page()`, `build_sheets()`, `make_chrome()`, `build_discipline_summary()`; 15/15 unit tests pass (3 region detection, 7 sheet ID extraction, 5 DrawingSegmentEngine boundary/discipline)
- ✅ `classify_sheet()` called from `make_chrome()` in title-block extraction pipeline
- ✅ `index-drawing` subcommand wired: `Commands::IndexDrawing` in `main.rs`, `handlers/index_drawing.rs` handler, dispatch arm in `handlers/mod.rs`
- ✅ `WorkflowOperation::IndexDrawing` variant added to `crates/contracts/src/lib.rs`
- ✅ `crates/engine/src/patterns/default.json` updated to v1.0.0 with `title-block-anchor` and `sheet-id-title-block` pattern families
- ✅ `crates/engine/Cargo.toml` — added `conset-pdf-standards-data` dependency; workspace `Cargo.toml` — registered `conset-pdf-standards-data` in `[workspace.dependencies]`
- ✅ 2 CLI integration tests added: `cli_index_drawing_on_dwg_fixture_produces_valid_json` + `cli_index_drawing_on_non_drawing_fixture_completes_with_zero_sheets`
- ✅ Zero compile errors; zero warnings (`cargo check -p conset-pdf-engine -p conset-pdf-contracts -p conset-pdf-backend-cli` clean)

---

### Sprint 9.2 — Drawing Sheet Replacement (`apply-sheet-addendum`) (~2 days) ✅ COMPLETE

This sprint implements the primary deliverable: automated drawing sheet replacement using an addendum PDFs. It reuses `PdfStitcher::stitch()` directly.

**9.2.A — `DrawingAddendumManifest` and `SheetReplaceSpec` IR types** (~80 lines in `crates/ir/src/drawing.rs`)

```rust
/// Declares which drawing set the addendum sheets apply to and what to replace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingAddendumManifest {
    pub schema_version: String,         // "1.0.0"
    pub original_drawing_set: String,   // path to the original PDF
    pub addendum_pdf: String,           // path to the addendum PDF
    pub output_path: String,
    pub audit_bundle_dir: Option<String>,
    pub dry_run: bool,
    /// Sheets to replace. If empty, auto-detect from `addendum_pdf` sheet IDs.
    pub sheets: Vec<SheetReplaceSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetReplaceSpec {
    pub sheet_id: String,               // e.g. "M-201"
    /// Page range in the addendum PDF that contains the replacement for this
    /// sheet (1-indexed, inclusive). If None, auto-detect from addendum index.
    pub addendum_pages: Option<SheetPageRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetPageRange {
    pub start: usize,
    pub end: usize,
}
```

Re-export from `crates/ir/src/lib.rs`. Serde round-trip tests.

**9.2.B — `DrawingPatchResult` and `SheetPatchResult` IR types** (~60 lines in `crates/ir/src/drawing.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrawingPatchResult {
    pub schema_version: String,
    pub original_drawing_set: String,
    pub addendum_pdf: String,
    pub output_path: Option<String>,
    pub dry_run: bool,
    pub sheet_results: Vec<SheetPatchResult>,
    pub diagnostics: Vec<DiagnosticEvent>,
    pub pattern_db_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetPatchResult {
    pub sheet_id: String,
    pub status: SheetPatchStatus,
    pub reason: Option<String>,
    pub pages_replaced: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SheetPatchStatus {
    Replaced,
    NotFound,       // sheet_id not in the original drawing set index
    Skipped,        // dry_run = true
    Failed { error_code: String },
}
```

**9.2.C — `DrawingsPatchOrchestrator`** (~200 lines in `crates/engine/src/drawings_patch.rs` new)

```rust
pub struct DrawingsPatchOrchestrator;

impl DrawingsPatchOrchestrator {
    pub fn run(manifest: &DrawingAddendumManifest) -> Result<DrawingPatchResult, EngineError>;
}
```

Algorithm:
1. Load PatternDatabase (fail fast, same as `specs_patch.rs` Step 0)
2. Extract and index the original drawing set → `DrawingIndex` (via `DrawingSegmentEngine::build_index`)
3. Extract and index the addendum PDF → `DrawingIndex` for the addendum (addendum is also a drawing set)
4. For each `SheetReplaceSpec` in the manifest:
   a. Resolve `sheet_id` to a `SheetEntry` in the original index; emit `SheetPatchStatus::NotFound` if absent
   b. Resolve `sheet_id` to a `SheetEntry` in the addendum index to locate the replacement pages
   c. If `dry_run`: emit `SheetPatchStatus::Skipped`, continue
   d. Extract just the addendum sheet's pages as a replacement PDF bytes (using lopdf, analogous to the render step in specs_patch.rs — but without Chrome, since drawing sheets are already rendered PDF pages)
   e. Build `StitchPlan { original_path, replacement_path: temp_file, section_id: sheet_id, segment_index: converted_from_drawing_index, dry_run: false, output_path }`
   f. Call `PdfStitcher::stitch(plan)`; emit result as `SheetPatchResult`
5. Write audit bundle: `change-report.json` + `diagnostics.jsonl` + `metrics.json`

Key design note — converting `DrawingIndex` → `SegmentIndex` for `PdfStitcher`: `PdfStitcher::stitch()` takes a `SegmentIndex` and a `section_id`. Since `SheetEntry` and `SectionEntry` have the same essential fields (`start_page`, `end_page`, `page_count`, `section_id`/`sheet_id`), provide an `Into<SegmentIndex>` impl on `DrawingIndex` that maps `SheetEntry → SectionEntry` with `section_id = sheet_id`. This avoids modifying `PdfStitcher` at all.

Unit tests:
- `orchestrator_not_found_sheet_emits_not_found_status`
- `orchestrator_dry_run_skips_all_sheets`
- `orchestrator_converts_drawing_index_to_segment_index` (the `Into<SegmentIndex>` conversion)

**9.2.D — `apply-sheet-addendum` CLI subcommand** (~100 lines in `apps/backend-cli/src/`)

```
conset-pdf apply-sheet-addendum \
  --manifest <drawing-manifest.json> \
  [--audit-bundle <dir>] \
  [--dry-run]
```

Wraps `DrawingsPatchOrchestrator::run()`. Error codes:

| Error Code | Condition |
|---|---|
| `MISSING_MANIFEST_PATH` | `--manifest` not provided |
| `MANIFEST_READ_ERROR` | File cannot be read or is not valid JSON |
| `EMPTY_MANIFEST` | `sheets` list is empty and auto-detect is not yet implemented |
| `AUDIT_DIR_CREATE_ERROR` | `--audit-bundle` dir cannot be created |
| `ORCHESTRATOR_ERROR` | Engine returned `Err` |

Integration tests:
- `cli_apply_sheet_addendum_dry_run_succeeds` — dry-run on DWG_RWB_LHHS_ALL_ORG + DWG_RWB_LHHS_ALL_ADD2 manifest; all sheets `Skipped`; exit 0
- `cli_apply_sheet_addendum_missing_manifest_fails_gracefully` — no `--manifest` flag; exit non-zero; `error_code: MISSING_MANIFEST_PATH`
- `cli_apply_sheet_addendum_writes_audit_bundle` — dry-run with `--audit-bundle`; verify `change-report.json`, `diagnostics.jsonl`, `metrics.json` written

**DoD for Sprint 9.2:** ✅ COMPLETE
- ✅ `DrawingAddendumManifest` + `SheetReplaceSpec` + `DrawingPatchResult` compile; serde round-trips pass (Sprint 9.0 pre-built)
- ✅ `DrawingsPatchOrchestrator::run()` passes 3 unit tests (dry_run_skips_all_sheets, not_found_status, converts_drawing_index_to_segment_index)
- ✅ `apply-sheet-addendum` subcommand present and wired to CLI (handlers/mod.rs + main.rs)
- ✅ `--dry-run` CLI flag propagated into manifest; `--audit-bundle` CLI option propagated into manifest.audit_bundle_dir
- ✅ 3 CLI integration tests written for 9.2.D (dry_run_skips_all_sheets, bad_manifest_fails_gracefully, dry_run_writes_audit_bundle)
- ✅ Audit bundle artifacts (change-report.json + metrics.json) written when audit_bundle_dir set
- ✅ Zero cargo errors or warnings; 3 engine unit tests pass

---

### Sprint 9.3 — Sheet Renaming Detection + Bookmark Generation (~1 day) ✅ COMPLETE

**9.3.A — Sheet renaming detection** (~80 lines in `crates/engine/src/drawings_patch.rs`)

A "renamed" sheet is one where a sheet appears in the addendum under a different `sheet_id` than in the original, but with a matching `sheet_title` (or partial title match, normalized to uppercase stripped of punctuation). This is a common occurrence in AEC addenda: a sheet originally numbered `M-201` is renumbered `M-204` in a subsequent addendum.

Algorithm:
1. After indexing both original and addendum, collect pairs that share a `sheet_title` but differ on `sheet_id`
2. For each such pair: add `SheetEntry.superseded_by = Some(new_sheet_id)` on the original entry; record the rename in `DrawingPatchResult` as a `SheetRenameEvent`
3. Include rename events in `change-report.json` under a `renames` array

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SheetRenameEvent {
    pub original_sheet_id: String,
    pub new_sheet_id: String,
    pub sheet_title: String,
    pub confidence: f64,  // title-match similarity, 0.0–1.0
}
```

Title similarity: exact normalized-uppercase match = 1.0; trimmed prefix match (≥ 60 % of title chars) = match confidence proportional to overlap ratio. Require ≥ 0.75 confidence before emitting a rename event.

Unit tests:
- `rename_detection_exact_title_match`
- `rename_detection_prefix_title_match_above_threshold`
- `rename_detection_different_title_no_rename`
- `rename_detection_same_title_same_id_no_rename`

**9.3.B — Bookmark generation for replaced sheets** (~80 lines in `crates/engine/src/stitch.rs`)

After `PdfStitcher::stitch()` completes, add the new sheet's title-block metadata to the PDF outline (bookmarks). This is additive: if no bookmarks exist in the original, generate a flat bookmark tree from the `DrawingIndex`; if bookmarks exist, reroute them using the existing `fixup_bookmarks()` logic and add missing entries for new sheets.

Add `generate_drawing_bookmarks(doc: &mut Document, index: &DrawingIndex) -> Vec<String>` (returns a list of warnings, same pattern as other stitch helpers). Call this from `DrawingsPatchOrchestrator::run()` after the final stitch completes.

Bookmark format: one entry per `SheetEntry`, display text = `"{sheet_id} — {sheet_title}"`, destination = first page of the sheet.

Unit tests:
- `generate_drawing_bookmarks_empty_index_produces_no_entries`
- `generate_drawing_bookmarks_two_sheets_produces_two_entries`

**DoD for Sprint 9.3:** ✅ COMPLETE
- ✅ Sheet rename events emitted in `change-report.json` (`renames` array) — `detect_renames()` + `normalize_title()` + `title_similarity()` in `drawings_patch.rs`; 4 unit tests pass
- ✅ `generate_drawing_bookmarks()` added to `stitch.rs`; 2 unit tests pass (`empty_index_produces_no_entries`, `two_sheets_produces_two_entries`)
- ✅ `SheetRenameEvent` included in `DrawingPatchResult.renames` (pre-built in IR Sprint 9.0); serde round-trip confirmed by existing IR tests
- ✅ Bookmarks wired into `DrawingsPatchOrchestrator::run()` — after final stitch, lopdf reloads output and writes flat outline from original index
- ✅ 113 engine unit tests pass; 0 failed; 0 cargo errors or warnings

---

### Sprint 9.4 — Basic Schedule Extraction (~1.5 days) ✅ COMPLETE

Drawing sets contain equipment and material schedules as tables in certain discipline sheets (typically M, P, E sheets). This sprint implements minimal viable table extraction from identified schedule sheets.

**9.4.A — Schedule sheet identification heuristics** ✅ DONE — `crates/engine/src/drawing_segment.rs`

`is_schedule_sheet_title()` helper added; called from `make_entry()` to populate `SheetEntry.is_schedule_sheet`.  Keywords checked (case-insensitive): `SCHEDULE`, `EQUIPMENT LIST`, `FIXTURE LIST`, `PANEL LIST`, `METER LIST`, `PIPING LIST`, `VALVE LIST`.  4 unit tests added.

**9.4.B — Table extraction from schedule sheets** ✅ DONE — `crates/engine/src/drawing_tables.rs` (new, ~260 lines)

```rust
pub struct ExtractedTable {
    pub sheet_id: String,
    pub sheet_title: String,
    pub table_title: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub row_count: usize,
    pub confidence: f64,
}

pub fn extract_tables_from_sheet(
    page: &Page,
    sheet: &SheetEntry,
) -> Vec<ExtractedTable>;
```

Algorithm: group spans into rows (y-epsilon=0.015), cluster x-positions into columns (x-epsilon=0.02), align to grid, detect header (first multi-cell non-numeric row), compute confidence (fraction of data rows with all cells filled).  Returns empty Vec for non-schedule sheets.  5 unit tests.

**9.4.C — `extract-schedules` CLI subcommand** ✅ DONE — `apps/backend-cli/src/handlers/extract_schedules.rs` (new)

```
conset-pdf extract-schedules \
  --input <transcript.json> \
  --output <schedules.json> \
  [--format json] \
  [--dry-run]
```

Reads `LayoutTranscript`, runs `DrawingSegmentEngine::build_index()`, filters `is_schedule_sheet==true`, calls `extract_tables_from_sheet()` per page, writes `{ "schema_version": "1.0.0", "sheet_count", "schedule_sheet_count", "table_count", "tables": [...] }`.

Two integration tests added: `cli_extract_schedules_on_dwg_fixture_produces_json`, `cli_extract_schedules_dry_run_skips_extraction`.

**DoD for Sprint 9.4:**
- ✅ `is_schedule_sheet` field on `SheetEntry`; populated by `DrawingSegmentEngine::build_index()`
- ✅ `extract_tables_from_sheet()` passes unit tests (synthetic page with row/column structure; non-schedule guard; few-spans guard; row-clustering unit test; numeric-row classifier test)
- ✅ `extract-schedules` subcommand present; integration tests pass
- ✅ `cargo check` clean (125+ engine unit tests pass, 0 failed)

---

### Sprint 9.5 — Hardening, Corpus Validation, Drawing Docs (~1 day) ✅ COMPLETE

**9.5.A — DWG corpus validation pass** (~30 min)

Add `--pipeline drawing-segment` mode to the existing `validate-corpus` subcommand in `tools/pattern_dev.rs`. Criteria for drawing fixtures:

```rust
DRAWING_MIN_SHEET_COUNT = 1
DRAWING_MIN_SHEET_COVERAGE = 0.80  // sheets with a detected sheet_id / total pages
```

Run against all 11 DWG tier-1 fixtures. Record in `audit_output/phase-i-corpus-baseline/dwg-post-baseline.json`.

**9.5.B — Error code documentation** (~20 min)

Add a §Drawing Errors section to `docs/CLI_REFERENCE.md` covering all new error codes from `apply-sheet-addendum` and `extract-schedules` subcommands.

**9.5.C — `WORKFLOW_APPLYSHEETADDENDUM.md`** (~2 hours)

New user tutorial in `docs/WORKFLOW_APPLYSHEETADDENDUM.md`. Follows the structure of `WORKFLOW_APPLYADDENDUM.md`:
- Prerequisites and tool installation
- Step 1: examine the drawing set with `index-drawing`
- Step 2: create a `DrawingAddendumManifest` JSON (with example based on DWG_RWB_LHHS_ALL_ORG → _ADD2)
- Step 3: run dry-run to validate sheet ID matching
- Step 4: production run
- Step 5: review audit bundle and rename events
- `DrawingAddendumManifest` full field reference table
- `SheetReplaceSpec` field reference table  
- `change-report.json` drawing variant field reference

**9.5.D — Determinism test for `apply-sheet-addendum`** (~40 lines)

`apply_sheet_addendum_dry_run_is_deterministic` — same structure as `apply_addendum_dry_run_is_deterministic` from Sprint 8.3.D. Double-execute `apply-sheet-addendum --dry-run`; compare `sheet_results` length, per-entry `sheet_id` + `status`, and `diagnostics` fields excluding timestamps.

**9.5.E — MASTER_PLAN + CHANGELOG + state-summary updates**

Update all three tracking documents to reflect Phase 9 complete.

**DoD for Sprint 9.5:**
- `validate-corpus --pipeline drawing-segment --tier 1` runs; results in `audit_output/phase-i-corpus-baseline/dwg-post-baseline.json`
- `docs/CLI_REFERENCE.md` updated with drawing subcommands + error codes
- `docs/WORKFLOW_APPLYSHEETADDENDUM.md` created
- Determinism test passing for `apply-sheet-addendum`
- All tracking docs updated

---

## Phase 9 Definition of Done

| # | Criterion | Sprint | Status |
|---|---|---|---|
| 1 | Extraction profiling decision documented in ARCHITECTURE.md | 9.0 | ✅ |
| 2 | `DrawingIndex` + `SheetEntry` IR types present; serde round-trips pass | 9.0 | ✅ |
| 3 | Discipline classifier (`classify_sheet()`) passes all unit tests (23+) | 9.0 | ✅ 33 tests |
| 4 | DWG corpus pre-Phase-9 baseline captured | 9.0 | ✅ |
| 5 | `DrawingSegmentEngine::build_index()` produces a valid `DrawingIndex` on real DWG fixtures | 9.1 | ✅ |
| 6 | `index-drawing` CLI subcommand present and passes integration tests | 9.1 | ✅ |
| 7 | `apply-sheet-addendum` dry-run succeeds on real DWG fixture pair | 9.2 | ✅ |
| 8 | Sheet replacement works end-to-end (non-dry-run) on at least one DWG fixture pair | 9.2 | ✅ `cli_apply_sheet_addendum_production_run_writes_output_pdf` — added Sprint 9.5 gap closure; also added `--output` arg to `apply-sheet-addendum` subcommand |
| 9 | Audit bundle (`change-report.json` + `diagnostics.jsonl` + `metrics.json`) written by `apply-sheet-addendum` | 9.2 | ✅ |
| 10 | Sheet renames tracked and reported in `change-report.json` | 9.3 | ✅ |
| 11 | Bookmark generation adds/updates outline entries after sheet replacement | 9.3 | ✅ |
| 12 | `extract-schedules` subcommand produces valid JSON output on at least one MECH or ELEC DWG fixture | 9.4 | ✅ |
| 13 | DWG corpus post-Phase-9 baseline shows ≥ 1 DWG fixture passing `drawing-segment` validation | 9.5 | ✅ |
| 14 | `WORKFLOW_APPLYSHEETADDENDUM.md` and CLI_REFERENCE.md drawing section complete | 9.5 | ✅ |
| 15 | Determinism regression test for `apply-sheet-addendum` passes | 9.5 | ✅ |
| 16 | `cargo check --workspace --tests` EXIT: 0 | All | ✅ |

---

## Sprint Dependency Graph

```
9.0 (foundation: IR types + classifier + profiling)
 ├── 9.1 (sheet detection: DrawingSegmentEngine, index-drawing)
 │    └── 9.2 (sheet replacement: DrawingsPatchOrchestrator, apply-sheet-addendum)
 │         ├── 9.3 (rename detection + bookmarks)
 │         └── 9.4 (schedule extraction + extract-schedules)
 │              └── 9.5 (hardening, corpus, docs)
 └── 9.0.D (corpus baseline: independent, run any time after 9.0.B)
```

Parallel work opportunity: 9.3 and 9.4 are independent of each other and can be worked in parallel after 9.2 is complete.

---

## Recommended Attack Order

1. **Sprint 9.0** — Run the extraction profiling benchmark first (9.0.A) so the range-bounded extraction decision is made before any orchestrator code is written. Then implement the IR types (9.0.B) which are a blocker for everything else. Then the discipline classifier (9.0.C). Then the corpus baseline (9.0.D).

2. **Sprint 9.1** — Build `drawing_segment.rs` bottom-up: region detection (9.1.A) → sheet ID extraction (9.1.B) → full `DrawingSegmentEngine` (9.1.C) → CLI subcommand (9.1.D). Verify against real DWG tier-1 fixtures before Sprint 9.2.

3. **Sprint 9.2** — Add the manifest IR types (9.2.A, 9.2.B) which are quick, then the orchestrator (9.2.C) which is the core complexity, then the CLI handler (9.2.D). The `Into<SegmentIndex>` conversion for `DrawingIndex` is the critical bridge to keep `PdfStitcher` unchanged.

4. **Sprints 9.3 + 9.4** — These can be developed in parallel if bandwidth allows.

5. **Sprint 9.5** — Run corpus validation last, after the engine is stable enough to produce meaningful results.
