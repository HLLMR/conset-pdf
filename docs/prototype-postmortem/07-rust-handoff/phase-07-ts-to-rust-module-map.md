# Phase 07 - TypeScript to Rust Module Map

**Document Type**: Rust Handoff — Module Mapping Reference  
**Date**: 2026-03-19

---

## Purpose

A complete, flat mapping from every TypeScript source file in `packages/core/src/` to its V4 Rust equivalent. Columns:

- **TS Module**: Source path relative to `packages/core/src/`
- **Rust Location**: Target path in `crates/` layout
- **Action**: Port / Replace / Do not port
- **Notes**: Critical porting constraints or design changes

---

## Dependency Rules Summary

The TypeScript layering must be preserved in Rust:

```
utils/         ← no internal deps
parser/        ← no internal deps (pure)
standards/     ← no internal deps (pure)
layout/        ← no internal deps
locators/      ← parser, layout, analyze (pageContext only), transcript (types)
analyze/       ← transcript, utils, layout
transcript/    ← utils
specs/         ← transcript, analyze, locators, parser, standards
narrative/     ← analyze, parser
bookmarks/     ← analyze, utils
core/          ← analyze, locators, parser, bookmarks, narrative, utils, standards
workflows/     ← core, specs, bookmarks, narrative, standards, types
```

In Rust crate terms:
- `crates/standards-data` and `crates/contracts` have zero internal deps
- `crates/core-engine` depends on `crates/contracts` and `crates/standards-data`
- `crates/workflows` depends on `crates/core-engine` and `crates/contracts`
- `apps/backend-cli` depends on `crates/workflows` and `crates/contracts`

---

## Module Map

### `analyze/` → `crates/core-engine::analyze`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `analyze/documentContext.ts` | `core-engine::analyze::document_context` | Port | Central document lifecycle cache. Holds `LayoutTranscript`, page contexts. Remove PDF.js bookmark shim (TD-001): in Rust, use lopdf to read bookmarks; do not keep a separate extractor for bookmarks. |
| `analyze/pageContext.ts` | `core-engine::analyze::page_context` | Port | Per-page caching (dimensions, rotation, spans, ROI views). Cache computed ROI views keyed by `LayoutProfile`. |
| `analyze/readingOrder.ts` | `core-engine::analyze::reading_order` | Port | Visual reading-order reconstruction for wrapped multi-span text. Critical for ROI-based ID assembly. Port the y-cluster + x-sort algorithm verbatim. |

---

### `text/` → `crates/core-engine::text`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `text/bandSlicer.ts` | `core-engine::text::band_slicer` | Port | Horizontal band slicing utilities used by chrome removal and bookmark heading resolution. Keep geometry behavior deterministic. |
| `text/pageRegions.ts` | `core-engine::text::page_regions` | Port | Named page-region extraction helpers (header/footer/body bands). Port as utility-tier geometry code with no workflow coupling. |

---

### `transcript/` → `crates/core-engine::transcript`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `transcript/types.ts` | `crates/contracts::transcript` | Port | `LayoutTranscript`, `LayoutPage`, `LayoutSpan`, `TranscriptMetadata`, `QualityMetrics`, `LineSegment`. Export from `contracts` crate so both CLI and GUI can use the types without depending on `core-engine`. |
| `transcript/factory.ts` | `core-engine::transcript::factory` | Replace | The factory in TS managed PyMuPDF-primary + PDF.js-fallback. In Rust, there is only one extractor: PDFium. The factory becomes a thin wrapper that calls `pdfium_extractor` and errors out if PDFium fails (no fallback). |
| `transcript/canonicalize.ts` | `core-engine::transcript::canonicalize` | Port | Rotation normalization, stable sort, content hash. Port the coordinate flip (PDFium bottom-left → top-left) and the total-ordering sort (y, x, content_hash). See coordinate system section in primer. |
| `transcript/quality.ts` | `core-engine::transcript::quality` | Port | Per-page quality metrics (char_count, whitespace_ratio, replacement_char_ratio, ordering_sanity). Aggregate acceptance gate. Make all thresholds configurable (not hardcoded). |
| `transcript/candidates.ts` | `core-engine::transcript::candidates` | Port | Generates chrome band detection inputs (repetition scoring, y-cluster inputs). Used by `specs::chrome`. |
| `transcript/extractors/pymupdfExtractor.ts` | `core-engine::transcript::extractor` | Replace | PyMuPDF-backed extractor. Replace with `pdfium-render` native Rust extraction. See `phase-07-sidecar-replacement-plan.md` for full replacement spec. |
| `transcript/extractors/pdfjsExtractor.ts` | — | Do not port | PDF.js fallback extractor. Eliminated in V4. No fallback path that reduces determinism. See ADR-002. |
| `transcript/sidecar/extract-transcript.py` | — | Replace | Python PyMuPDF sidecar script. Replaced entirely by `pdfium-render` Rust crate. See sidecar replacement plan. |
| `transcript/abstraction/tokenVault.ts` | `core-engine::transcript::abstraction::token_vault` | Port | Sensitive text → structural token mapping for LLM privacy. Port all three privacy modes (STRICT_STRUCTURE_ONLY, WHITELIST_ANCHORS, FULL_TEXT_OPT_IN). Add explicit user confirmation gate on FULL_TEXT_OPT_IN (R-010). |
| `transcript/abstraction/sanitize.ts` | `core-engine::transcript::abstraction::sanitize` | Port | Abstract transcript generation with privacy modes. |
| `transcript/abstraction/lineGrouping.ts` | `core-engine::transcript::abstraction::line_grouping` | Port | Line grouping from raw spans for abstraction quality. |
| `transcript/abstraction/repetitionMetrics.ts` | `core-engine::transcript::abstraction::repetition_metrics` | Port | Repetition scoring used by chrome band detection. |
| `transcript/abstraction/shapeFeatures.ts` | `core-engine::transcript::abstraction::shape_features` | Port | Shape-level features for span classification. |
| `transcript/ml/rulesetCompiler.ts` | `core-engine::transcript::ml::ruleset_compiler` | Port | ML ruleset compiler interface. Low priority. |
| `transcript/ml/apiCompiler.ts` | `core-engine::transcript::ml::api_compiler` | Port | LLM API backend for ruleset generation. Low priority. |
| `transcript/ml/promptBuilder.ts` | `core-engine::transcript::ml::prompt_builder` | Port | Prompt construction for LLM calls. Low priority. |
| `transcript/profiles/types.ts` | `crates/contracts::transcript_profiles` | Port | `SpecProfile`, `SheetTemplateProfile` types. |
| `transcript/profiles/registry.ts` | `core-engine::transcript::profiles::registry` | Port | Profile registry with versioning. |
| `transcript/profiles/validation.ts` | `core-engine::transcript::profiles::validation` | Port | Profile validation. |
| `transcript/schedules/extractor.ts` | `core-engine::transcript::schedules::extractor` | Port | Geometry-first schedule table extractor. Port as-is (geometry core); the pdfplumber/camelot TODOs become V4 backlog items, not stubs. |
| `transcript/schedules/tableBuilder.ts` | `core-engine::transcript::schedules::table_builder` | Port | Column/row assembly from coordinate clusters. |

---

### `locators/` → `crates/core-engine::locators`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `locators/compositeLocator.ts` | `core-engine::locators::composite_locator` | Port | ROI-first with legacy fallback chain. In Rust: ROI locator is the only production path. Legacy fallback is removed unless feature-flagged for compatibility testing. |
| `locators/roiSheetLocator.ts` | `core-engine::locators::roi_sheet_locator` | Port | Profile-driven ROI region extraction (primary drawings locator). Critical production path. |
| `locators/roiSpecsSectionLocator.ts` | `core-engine::locators::roi_specs_section_locator` | Port | ROI-driven specs section detection. |
| `locators/sheetLocator.ts` | `core-engine::locators::sheet_locator` | Port | Abstract locator trait. Becomes a Rust `trait SheetLocator`. |
| `locators/specsSectionLocator.ts` | `core-engine::locators::specs_section_locator` | Port | Text-based specs section detector (no profile required). |
| `locators/legacyTitleblockLocator.ts` | — | Do not port | Heuristic fallback, deprecated. Gated behind `ENABLE_LEGACY_LOCATOR` feature flag in prototype. Do not carry forward to Rust production codebase. |

---

### `parser/` → `crates/core-engine::parser`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `parser/drawingsSheetId.ts` | `core-engine::parser::drawings_sheet_id` | Port | All regex patterns for drawing sheet IDs. Port patterns verbatim from `DEFAULT_DRAWINGS_PATTERN` and the false-positive filter list. See portability matrix for pattern-by-pattern decisions. |
| `parser/specsSectionId.ts` | `core-engine::parser::specs_section_id` | Port | Regex patterns for spec section IDs. Port verbatim. Modern and legacy formats. |
| `parser/normalize.ts` | `core-engine::parser::normalize` | Port | Canonical ID format converters (uppercase, whitespace collapse, format detection). Port all normalization rules as pure functions. |

---

### `layout/` → `crates/core-engine::layout`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `layout/types.ts` | `crates/contracts::layout` | Port | `LayoutProfile`, `NormalizedROI`, profile type enums. Export from `contracts`. |
| `layout/load.ts` | `core-engine::layout::load` | Port | Profile loader with validation. Use `serde_json` for deserialization; reproduce `validateROI()` bounds checks exactly. |

---

### `standards/` → `crates/core-engine::standards` + `crates/standards-data`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `standards/normalizeDrawingsDiscipline.ts` | `core-engine::standards::drawings_discipline` | Port | Pure normalization function; no I/O. |
| `standards/normalizeSpecsMasterformat.ts` | `core-engine::standards::specs_masterformat` | Port | Pure normalization function; no I/O. |
| `standards/compare.ts` | `core-engine::standards::compare` | Port | Deterministic row comparators. These define output file ordering; must match prototype behavior exactly. |
| `standards/registry.ts` | `core-engine::standards::registry` | Port | Lookup table facades over `standards-data` crate. |
| `standards/types.ts` | `crates/contracts::standards` | Port | `DrawingsDisciplineMeta`, `SpecsMasterformatMeta`, `StandardsBasis`, `SpecsBasis`. |
| `standards/datasets/drawingsDesignators.ts` | `crates/standards-data::drawings_designators` | Port verbatim | Static UDS designator table + alias map + CONTROLS_KEYWORDS + CIVIL_KEYWORDS. See portability matrix. |
| `standards/datasets/disciplines.generated.ts` | `crates/standards-data::disciplines` | Port verbatim | Generated discipline table. |
| `standards/datasets/divisions.generated.ts` | `crates/standards-data::divisions` | Port verbatim | Generated MasterFormat division table. |
| `standards/datasets/masterformatDivisions.ts` | `crates/standards-data::masterformat_divisions` | Port verbatim | All 50 MasterFormat divisions and sort order. |
| `standards/datasets/drawingsOrderHeuristic.ts` | `crates/standards-data::drawings_order_heuristic` | Port verbatim | Canonical discipline sort order. |
| `standards/datasets/legacySections.generated.ts` | `crates/standards-data::legacy_sections` | Port verbatim | Legacy 5-digit → modern 6-digit section mapping. |

---

### `specs/` → `crates/core-engine::specs`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `specs/ast/types.ts` | `crates/contracts::specs` | Port | `SpecDoc`, `SpecNode`, `SpecSection`, `BookmarkAnchorTree`. Export from contracts; this type crosses workflow boundaries. |
| `specs/extract/chromeRemoval.ts` | `core-engine::specs::extract::chrome_removal` | Port | Header/footer band suppression (Y-cluster + repetition analysis). Make all thresholds configurable params (not hardcoded 12%/12%/50%). See NN-04. |
| `specs/extract/sectionDetector.ts` | `core-engine::specs::extract::section_detector` | Port | Section heading detection via heading regexes + hierarchy inference. |
| `specs/extract/anchorDetector.ts` | `core-engine::specs::extract::anchor_detector` | Port | Hierarchical anchor detection. |
| `specs/extract/listDetector.ts` | `core-engine::specs::extract::list_detector` | Port | List item detection. |
| `specs/extract/paragraphNormalizer.ts` | `core-engine::specs::extract::paragraph_normalizer` | Port | Wrap-join and hyphen repair. Critical for text accuracy (NN-19). |
| `specs/extract/hierarchyBuilder.ts` | `core-engine::specs::extract::hierarchy_builder` | Port | Indentation-based hierarchy from section nodes. |
| `specs/extract/textExtractor.ts` | `core-engine::specs::extract::text_extractor` | Port | Node creation from raw spans. |
| `specs/extract/bookmarkTreeGenerator.ts` | `core-engine::specs::extract::bookmark_tree_generator` | Port | `SpecDoc` AST → `BookmarkAnchorTree`. This is the primary stable output of the extraction pipeline. |
| `specs/extract/tableDetector.ts` | `core-engine::specs::extract::table_detector` | Port | Coordinate-cluster-based table detection for spec pages. Keep geometry-first behavior; do not replace with OCR/ML heuristics by default. |
| `specs/patch/types.ts` | `crates/contracts::specs_patch` | Port | `SpecPatch`, `SpecPatchOperation` types. |
| `specs/patch/validator.ts` | `core-engine::specs::patch::validator` | Port | Patch validation (target section must exist, operations are well-formed). |
| `specs/patch/apply.ts` | `core-engine::specs::patch::apply` | Port | Patch application: insert, delete, move, renumber, replace. |
| `specs/render/htmlGenerator.ts` | `core-engine::specs::render::html_generator` | Port | `SpecDoc` AST → HTML/CSS. Port the template logic; the CSS rules are portable. |
| `specs/render/pdfRenderer.ts` | `core-engine::specs::render::pdf_renderer` | Replace | Playwright HTML→PDF. Replace with `headless_chrome` crate (V4 Phase 2). Do not re-implement Playwright dependency. |
| `specs/footerIndexBuilder.ts` | `core-engine::specs::footer_index_builder` | Port | Intermediate footer index construction feeding footer boundary mapping. Keep separate from final `footer_section_map` so Phase 2 parity remains testable. |
| `specs/footerSectionIdParser.ts` | `core-engine::specs::footer_section_id_parser` | Port + Extend | Footer section ID pattern parser. Prototype is Division-23-only (R-003). V4 must extend to all MasterFormat divisions before specs workflow is complete. |
| `specs/footerSectionMap.ts` | `core-engine::specs::footer_section_map` | Port + Complete | Footer-first section boundary map. Currently a stub. Must be fully implemented in V4 (complete `buildFooterSectionMap()`). See R-002. |
| `specs/footerValidation.ts` | `core-engine::specs::footer_validation` | Port | Footer consistency validation. |
| `specs/inventory/index.ts` | `core-engine::specs::inventory` | Port | Spec inventory module root from extracted AST. |
| `specs/inventory/specFooterIndexer.ts` | `core-engine::specs::inventory::spec_footer_indexer` | Port | Footer-indexed section entry generation used by inventory assembly. |
| `specs/inventory/specSectionizer.ts` | `core-engine::specs::inventory::spec_sectionizer` | Port | Logical sectionizer for inventory assembly. Preserve deterministic ordering. |

---

### `bookmarks/` → `crates/core-engine::bookmarks`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `bookmarks/types.ts` | `crates/contracts::bookmarks` | Port | `BookmarkNode`, `BookmarkDestination`, `BookmarkTree`. `BookmarkAnchorTree` is in `contracts::specs`. |
| `bookmarks/reader.ts` | `core-engine::bookmarks::reader` | Port | Read existing bookmarks from PDF. In Rust: use `lopdf` to traverse the outline dictionary. |
| `bookmarks/validator.ts` | `core-engine::bookmarks::validator` | Port | Validate bookmark destinations against PDF page count. |
| `bookmarks/treeBuilder.ts` | `core-engine::bookmarks::tree_builder` | Port | Build `BookmarkTree` from `BookmarkAnchorTree` or inventory. |
| `bookmarks/corrections.ts` | `core-engine::bookmarks::corrections` | Port | Apply correction overlays (rename/reorder/delete/retarget/rebuild). |
| `bookmarks/headingResolver.ts` | `core-engine::bookmarks::heading_resolver` | Port | Layout-aware heading-based section destination resolution. |
| `bookmarks/settings.ts` | `core-engine::bookmarks::settings` | Port | User-configurable bookmark shaping settings. Consider exporting pure data structs into `crates/contracts::bookmarks` if GUI/API surfaces this directly. |
| `bookmarks/pikepdfBookmarkWriter.ts` | `core-engine::bookmarks::writer` | Replace | pikepdf/QPDF sidecar-backed writer. Replace with `lopdf` native indirect-object outline construction. See sidecar replacement plan for the exact bookmark structure requirement. |
| `bookmarks/profiles/` | `core-engine::bookmarks::profiles` | Port | Bookmark style profiles (raw, specs-v1, specs-v2-detailed). |
| `bookmarks/profiles/raw.ts` | `core-engine::bookmarks::profiles::raw` | Port | Raw bookmark profile that preserves outline structure with minimal normalization. Keep as the “do not reshape” compatibility profile. |
| `bookmarks/sidecar/bookmark-writer.py` | — | Replace | Python pikepdf sidecar script. Replaced by `lopdf` Rust implementation. |

---

### `narrative/` → `crates/core-engine::narrative`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `narrative/text-extract.ts` | `core-engine::narrative::text_extract` | Port | Page-aware text extraction from `DocumentContext`. |
| `narrative/parse-algorithmic.ts` | `core-engine::narrative::parse_algorithmic` | Port | Pattern-matching parser for drawing/spec revision instructions in addendum text. Port regex patterns verbatim. |
| `narrative/normalize.ts` | `core-engine::narrative::normalize` | Port | ID normalization from narrative text. |
| `narrative/validate.ts` | `core-engine::narrative::validate` | Port | Narrative instructions vs. detected inventory validation (advisory-only). |

---

### `core/` → `crates/workflows` (business logic)

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `core/mergeAddenda.ts` | `crates/workflows::merge::merge_addenda` | Port | Legacy direct merge API. Thin wrapper over `MergeWorkflow`. |
| `core/splitSet.ts` | `crates/workflows::split::split_set` | Port | Split PDF into named subsets. |
| `core/assembleSet.ts` | — | Do not port | Deprecated. See ADR-008. |
| `core/planner.ts` | `crates/workflows::merge::planner` | Port | Merge plan construction (mode semantics, conflict resolution). `MergePlan` and `MergeAction` types go in `crates/contracts`. |
| `core/applyPlan.ts` | `crates/workflows::merge::apply_plan` | Replace | Plan execution. Prototype: in-memory `pdf-lib` assembly + pikepdf passthrough. V4: disk-streaming via `lopdf` page-stream copy from day one. See ADR-004. |
| `core/report.ts` | `crates/workflows::report` | Port | Run report / audit JSON generation. Port schema; use `crates/contracts::AuditBundle` type. |

---

### `workflows/` → `crates/workflows`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `workflows/types.ts` | `crates/contracts::workflows` | Port | `WorkflowId`, `Severity`, `RowStatus`, `Confidence`, `InventoryRowBase`, `Issue`, `Conflict`, `InventoryResult`, `CorrectionOverlay`, `ExecuteResult`. All exported from `contracts`. |
| `workflows/engine.ts` | `crates/workflows::engine` | Port | `Workflow` trait and `WorkflowRunner` factory. See `Workflow` trait in primer. |
| `workflows/merge/mergeWorkflow.ts` | `crates/workflows::merge::merge_workflow` | Port | Merge workflow (analyze/applyCorrections/execute). Fix R-007: `apply_corrections` must not re-run `analyze`. |
| `workflows/specs-patch/specsPatchWorkflow.ts` | `crates/workflows::specs_patch::specs_patch_workflow` | Restore | Abandoned in prototype; restore in V4 Phase 2 with native rendering. |
| `workflows/bookmarks/bookmarksWorkflow.ts` | `crates/workflows::bookmarks::bookmarks_workflow` | Port | Bookmarks workflow. |
| `workflows/mappers/merge.ts` | `crates/workflows::merge::mappers` | Port | `ParseResult` → `InventoryRowBase` mapping + standards enrichment. |

---

### `submittals/` → `crates/core-engine::submittals`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `submittals/extract/submittalParser.ts` | `core-engine::submittals::extract::submittal_parser` | Port | Parser stub. Promote to a production-grade parser in V4 Phase 4. Add `SubmittalWorkflow` in `crates/workflows`. |

---

### `config/` → `crates/core-engine::config`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `config/featureFlags.ts` | `core-engine::config::feature_flags` | Replace | Single `ENABLE_LEGACY_LOCATOR` flag. In Rust: use Cargo features for compile-time exclusions; use a `RuntimeConfig` struct for runtime toggles. Remove legacy locator toggle (do not port the legacy locator itself). |

---

### `utils/` → `crates/core-engine::utils`

| TS Module | Rust Location | Action | Notes |
|---|---|---|---|
| `utils/bookmarks.ts` | — | Do not port | Legacy bookmark generation helpers, deprecated. See TD-003. |
| `utils/bookmarkWriter.ts` | `crates/core-engine::bookmarks::writer_trait` | Port | Abstract `BookmarkWriter` trait. |
| `utils/pdfLibBookmarkWriter.ts` | — | Do not port | pdf-lib bookmark writer (development/testing only, superseded). See TD-002 and ADR-008. |
| `utils/pikepdfWriter.ts` | — | Replace | `pikepdf` passthrough writer. Replaced by `lopdf` native writer. See sidecar replacement plan. |
| `utils/pdf.ts` | `core-engine::utils::pdf` | Port | PDF utility helpers (page dimensions, orientation checks, etc.). |
| `utils/fs.ts` | `core-engine::utils::fs` | Port | File system helpers (temp-file pattern, atomic rename). Port the temp-file-then-rename pattern for safe output writes. |
| `utils/sort.ts` | `core-engine::utils::sort` | Port | Deterministic sort utilities. All sort comparators must produce total ordering with no undefined tie-break. |

---

## GUI Module Map

### `conset-pdf-gui/src/` → `apps/desktop-gui/`

| TS Module | Rust/Tauri Location | Action | Notes |
|---|---|---|---|
| `src/main.ts` | `apps/desktop-gui/src/main.ts` | Port | Tauri main process entry point. |
| `src/preload.ts` | — | Replace | Electron preload is not needed in Tauri. Tauri's `invoke` + event system replaces the Node.js preload/contextBridge pattern. |
| `src/shared/ipc-response.ts` | `crates/contracts::ipc` | Port | `IpcResponse<T>` envelope. Serialize to JSON for Tauri command responses. Preserve the structured error shape: `{ message, code?, stack?, context? }`. Note: docs describe plain string; code uses structured shape. Port the code shape. See R-012. |
| `src/main/ipc/` | `apps/desktop-gui/src-tauri/src/ipc/` | Port | All IPC handlers. Each IPC handler becomes a Tauri `#[tauri::command]`. |
| `src/main/ipc/logging.ts` | `apps/desktop-gui/src-tauri/src/ipc/logging.rs` | Port | Log browsing/export/system-info IPC surface. Port the supportability commands and keep bundle export aligned with Phase 9 telemetry requirements. |
| `src/main/ipc/merge-internal.ts` | `apps/desktop-gui/src-tauri/src/ipc/merge_internal.rs` | Port | Shared merge adapter/normalization layer backing public merge commands and progress emission. Not a separate user-facing workflow command. |
| `src/main/profiles/store.ts` | `apps/desktop-gui/src-tauri/src/profiles/store.rs` | Port | Profile storage with migration. Use `serde_json` + file I/O. |
| `src/main/history/store.ts` | `apps/desktop-gui/src-tauri/src/history/store.rs` | Port | Run history persistence. |
| `src/modules/roi/roiOverlayController.js` | `apps/desktop-gui/src/roi_overlay_controller.ts` | Port | ROI drawing overlay UI logic. Stays in frontend JS/TS. The ROI overlay is a renderer concern, not a Rust concern. |
| Wizard UIs | `apps/desktop-gui/src/` | Port | Merge, split, bookmarks, placeholder wizards. Stays in frontend. |
| `src/app.js` | `apps/desktop-gui/src/app.ts` | Port | App root. |

---

## IPC Handler Map

Each Electron IPC handler (`ipcMain.handle(...)`) becomes a Tauri command (`#[tauri::command]`):

| Electron IPC Channel | Tauri Command Name | Calls |
|---|---|---|
| `dialogs:open-file` | `open_file_dialog` | Tauri dialog API |
| `pdf:get-page-count` | `get_page_count` | `core-engine::analyze` |
| `pdf:get-transcript` | `get_transcript` | `core-engine::transcript` |
| `profiles:list` | `list_profiles` | profiles store |
| `profiles:import` | `import_profile` | profiles store |
| `detection:analyze` | `analyze_workflow` | `workflows::MergeWorkflow::analyze` |
| `operations:execute` | `execute_workflow` | `workflows::MergeWorkflow::execute` |
| `history:list` | `list_history` | history store |
| `system:get-info` | `get_system_info` | OS metadata |
| `log:getFiles` | `get_log_files` | logging store |
| `log:readFile` | `read_log_file` | logging store |
| `log:export` | `export_log_bundle` | logging store + system info bundle |
| `log:openDirectory` | `open_log_directory` | OS shell integration |
| `log:clear` | `clear_logs` | logging store |
| `log:getStats` | `get_log_stats` | logging store |
| `log:getSystemInfo` | `get_debug_system_info` | system-info helper |
| `merge:plan` | `plan_merge` | `workflows::merge::planner` |
| `debug:walkthrough` | `debug_walkthrough` | diagnostic |
| `settings:get` | `get_settings` | settings store |
| `settings:set` | `set_settings` | settings store |
| `naming:preview` | `preview_naming` | `workflows::merge::mappers` |
| `standards:normalize` | `normalize_standards` | `core-engine::standards` |
| `cache:invalidate` | `invalidate_cache` | `analyze::document_context` |

`log:debug`, `log:info`, `log:warn`, `log:error`, and `log:performance` are renderer-to-backend logging bridge calls rather than domain workflow commands. In Tauri, these can be implemented as lightweight commands or replaced with a frontend logger that forwards structured events into the Rust tracing pipeline.

`merge-internal.ts` is a shared adapter, not a standalone IPC endpoint. In Tauri it should back the public merge commands plus progress events emitted from long-running workflow execution.

---

## Summary Counts

| Action | Count |
|---|---|
| Port exactly | ~55 modules |
| Port with modifications/fixes | ~8 modules |
| Replace with native Rust equivalent | ~6 modules |
| Do not port (deprecated/removed) | ~7 modules |
| Move to `contracts` crate | ~12 type files |
