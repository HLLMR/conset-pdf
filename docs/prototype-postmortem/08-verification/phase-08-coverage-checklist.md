# Phase 08 - Coverage Checklist

**Phase**: Verification  
**Document Type**: Module, ADR, and Contract Coverage Checklist  
**Date**: 2026-03-19  
**Method**: Cross-reference live `packages/core/src/` and `conset-pdf-gui/src/` file trees against Phases 1–7 artifacts.

---

## Coverage Status Key

| Symbol | Meaning |
|---|---|
| COVERED | Module/file is inventoried AND has Phase 7 porting guidance |
| INVENTORIED | Module/file is in Phase 1 inventory but has no Phase 7 porting decision |
| PARTIAL | Mentioned in docs but individual file decisions are missing |
| GAP | Present in codebase, not mentioned anywhere in extraction docs |

---

## Core Module Coverage (`packages/core/src/`)

### `analyze/` — 3 source files
Phase 1 status: Complete  
Phase 7 module map: All 3 files explicitly mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `analyze/documentContext.ts` | Yes | Port (remove PDF.js shim) | COVERED |
| `analyze/pageContext.ts` | Yes | Port | COVERED |
| `analyze/readingOrder.ts` | Yes | Port verbatim | COVERED |

---

### `bookmarks/` — ~12 TS source files + Python scripts
Phase 1 status: Complete  
Phase 7 module map: Core files mapped; style profiles covered as a directory group.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `bookmarks/reader.ts` | Yes | Port | COVERED |
| `bookmarks/validator.ts` | Yes | Port | COVERED |
| `bookmarks/treeBuilder.ts` | Yes | Port | COVERED |
| `bookmarks/corrections.ts` | Yes | Port | COVERED |
| `bookmarks/headingResolver.ts` | Yes | Port | COVERED |
| `bookmarks/pikepdfBookmarkWriter.ts` | Yes | Replace (lopdf) | COVERED |
| `bookmarks/specsV1.ts` | Yes (as "profiles/") | PARTIAL — grouped as `bookmarks/profiles/` in Phase 7 | PARTIAL |
| `bookmarks/specsV2Detailed.ts` | Yes (as "profiles/") | PARTIAL — grouped as `bookmarks/profiles/` in Phase 7 | PARTIAL |
| `bookmarks/settings.ts` | No | No | GAP |
| `bookmarks/profiles/raw.ts` | No | No | GAP |
| `bookmarks/dumpPageRegions.ts` | No (diagnostic) | No | GAP (diagnostic util; no port needed) |
| `bookmarks/sidecar/bookmark-writer.py` | Yes | Replace (lopdf) | COVERED |
| Python debug scripts (4 files) | No | No | GAP (development tooling; do not port) |

**Notes**:
- `settings.ts` and `bookmarks/profiles/raw.ts` were not individually inventoried or mapped in the initial pass. `bookmarks/profiles/raw.ts` is the raw bookmark shaping profile; `settings.ts` contains bookmark output settings (fit type, title format). Both should be added to the module map as Port entries.
- Development Python scripts (`debug_outline*.py`, `test_outline*.py`, `verify_outline*.py`) are tooling artifacts with no Rust porting requirement.

---

### `config/` — 1 source file
Phase 1 status: Complete  
Phase 7 module map: Explicitly mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `config/featureFlags.ts` | Yes | Replace (Cargo features + RuntimeConfig) | COVERED |

---

### `core/` — 6 source files
Phase 1 status: Partial  
Phase 7 module map: All mapped (assembleSet → do not port).

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `core/mergeAddenda.ts` | Yes | Port | COVERED |
| `core/splitSet.ts` | Yes | Port | COVERED |
| `core/assembleSet.ts` | Yes (deprecated) | Do not port | COVERED |
| `core/planner.ts` | Yes | Port | COVERED |
| `core/applyPlan.ts` | Yes | Replace (disk-streaming lopdf) | COVERED |
| `core/report.ts` | Yes | Port (AuditBundle type) | COVERED |

---

### `layout/` — 2 source files
Phase 1 status: Complete  
Phase 7 module map: Both files mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `layout/types.ts` | Yes | Port → contracts::layout | COVERED |
| `layout/load.ts` | Yes | Port | COVERED |

---

### `locators/` — 6 source files
Phase 1 status: Complete  
Phase 7 module map: All mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `locators/compositeLocator.ts` | Yes | Port | COVERED |
| `locators/roiSheetLocator.ts` | Yes | Port | COVERED |
| `locators/roiSpecsSectionLocator.ts` | Yes | Port | COVERED |
| `locators/sheetLocator.ts` | Yes | Port (trait) | COVERED |
| `locators/specsSectionLocator.ts` | Yes | Port | COVERED |
| `locators/legacyTitleblockLocator.ts` | Yes (deprecated) | Do not port | COVERED |

---

### `narrative/` — 6 source files
Phase 1 status: Complete  
Phase 7 module map: All 4 TS source files mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `narrative/text-extract.ts` | Yes | Port | COVERED |
| `narrative/parse-algorithmic.ts` | Yes | Port | COVERED |
| `narrative/normalize.ts` | Yes | Port | COVERED |
| `narrative/validate.ts` | Yes | Port | COVERED |

---

### `parser/` — 3 source files
Phase 1 status: Complete  
Phase 7 module map: All 3 mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `parser/drawingsSheetId.ts` | Yes | Port | COVERED |
| `parser/specsSectionId.ts` | Yes | Port | COVERED |
| `parser/normalize.ts` | Yes | Port | COVERED |

---

### `specs/` — ~24 TS source files
Phase 1 status: Partial  
Phase 7 module map: Most extract/patch/render files mapped; inventory submodule files missing.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `specs/ast/types.ts` | Yes | Port → contracts::specs | COVERED |
| `specs/extract/chromeRemoval.ts` | Yes | Port | COVERED |
| `specs/extract/sectionDetector.ts` | Yes | Port | COVERED |
| `specs/extract/anchorDetector.ts` | Yes | Port | COVERED |
| `specs/extract/listDetector.ts` | Yes | Port | COVERED |
| `specs/extract/paragraphNormalizer.ts` | Yes | Port | COVERED |
| `specs/extract/hierarchyBuilder.ts` | Yes | Port | COVERED |
| `specs/extract/textExtractor.ts` | Yes | Port | COVERED |
| `specs/extract/bookmarkTreeGenerator.ts` | Yes | Port | COVERED |
| **`specs/extract/tableDetector.ts`** | Yes (from plan) | **Not in module map** | **GAP** |
| `specs/footerSectionIdParser.ts` | Yes | Port + Extend | COVERED |
| `specs/footerSectionMap.ts` | Yes | Port + Complete | COVERED |
| `specs/footerValidation.ts` | Yes | Port | COVERED |
| **`specs/footerIndexBuilder.ts`** | Yes (footerIndex) | **Not in module map** | **GAP** |
| `specs/inventory/index.ts` | Yes (as "inventory") | Port (listed as `specs/inventory.ts`) — path discrepancy | PARTIAL |
| **`specs/inventory/specFooterIndexer.ts`** | No | **Not in module map** | **GAP** |
| **`specs/inventory/specSectionizer.ts`** | No | **Not in module map** | **GAP** |
| `specs/patch/apply.ts` | Yes | Port | COVERED |
| `specs/patch/validator.ts` | Yes | Port | COVERED |
| `specs/patch/types.ts` | Yes | Port → contracts | COVERED |
| `specs/render/htmlGenerator.ts` | Yes | Port | COVERED |
| `specs/render/pdfRenderer.ts` | Yes | Replace (headless_chrome) | COVERED |
| `specs/render/templates/specs.css` | Yes (implied) | Data file embedded in html_generator | COVERED |

**Notes**:
- `specs/extract/tableDetector.ts` is in the plan inventory list and in the extract directory but has no Phase 7 entry. V4 must add `core-engine::specs::extract::table_detector` (Port).
- `specs/footerIndexBuilder.ts` at root level differs from `footerSectionMap.ts`. Based on name, it likely builds the footer index data structure used by the rest of the footer pipeline. Requires an explicit Phase 7 entry: `core-engine::specs::footer_index_builder` (Port).
- `specs/inventory/specFooterIndexer.ts` and `specs/inventory/specSectionizer.ts` are the inventory submodule files not listed in the Phase 7 map. Both require entries: `core-engine::specs::inventory::spec_footer_indexer` and `core-engine::specs::inventory::spec_sectionizer` (Port).
- The Phase 7 module map references `specs/inventory.ts` but the actual path is `specs/inventory/index.ts` (a subdirectory). This is a path discrepancy to resolve.

---

### `standards/` — ~14 source files
Phase 1 status: Complete  
Phase 7 module map: All files mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `standards/normalizeDrawingsDiscipline.ts` | Yes | Port | COVERED |
| `standards/normalizeSpecsMasterformat.ts` | Yes | Port | COVERED |
| `standards/compare.ts` | Yes | Port | COVERED |
| `standards/registry.ts` | Yes | Port | COVERED |
| `standards/types.ts` | Yes | Port → contracts | COVERED |
| `standards/datasets/drawingsDesignators.ts` | Yes | Port verbatim | COVERED |
| `standards/datasets/disciplines.generated.ts` | Yes | Port verbatim | COVERED |
| `standards/datasets/divisions.generated.ts` | Yes | Port verbatim | COVERED |
| `standards/datasets/masterformatDivisions.ts` | Yes | Port verbatim | COVERED |
| `standards/datasets/drawingsOrderHeuristic.ts` | Yes | Port verbatim | COVERED |
| `standards/datasets/legacySections.generated.ts` | Yes | Port verbatim | COVERED |

---

### `submittals/` — 2 source files
Phase 1 status: Complete  
Phase 7 module map: Mapped with V4 promotion plan.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `submittals/extract/submittalParser.ts` | Yes | Port (V4 Phase 4 promotion) | COVERED |

---

### `text/` — 2 source files  ← **UNLISTED MODULE**
Phase 1 status: Noted as "Additional Observation" (not in plan inventory)  
Phase 7 module map: **Absent entirely.**

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| **`text/bandSlicer.ts`** | Extra obs only | **Not in module map** | **GAP** |
| **`text/pageRegions.ts`** | Extra obs only | **Not in module map** | **GAP** |

**Notes**:
- `text/bandSlicer.ts` provides band-slicing over page transcript spans (used by chrome removal and bookmark logic).
- `text/pageRegions.ts` provides page region extraction (top/bottom/left/right bands).
- These are utility-tier modules with no external deps. They should be added to Phase 7 module map as: `core-engine::text::band_slicer` (Port) and `core-engine::text::page_regions` (Port).

---

### `transcript/` — ~27 source files
Phase 1 status: Complete  
Phase 7 module map: All key files mapped. Single new file vs plan count.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `transcript/types.ts` | Yes | Port → contracts::transcript | COVERED |
| `transcript/factory.ts` | Yes | Replace (single PDFium path) | COVERED |
| `transcript/canonicalize.ts` | Yes | Port | COVERED |
| `transcript/quality.ts` | Yes | Port | COVERED |
| `transcript/candidates.ts` | Yes | Port | COVERED |
| `transcript/extractors/pymupdfExtractor.ts` | Yes | Replace | COVERED |
| `transcript/extractors/pdfjsExtractor.ts` | Yes | Do not port | COVERED |
| `transcript/sidecar/extract-transcript.py` | Yes | Replace | COVERED |
| `transcript/abstraction/tokenVault.ts` | Yes | Port | COVERED |
| `transcript/abstraction/sanitize.ts` | Yes | Port | COVERED |
| `transcript/abstraction/lineGrouping.ts` | Yes | Port | COVERED |
| `transcript/abstraction/repetitionMetrics.ts` | Yes | Port | COVERED |
| `transcript/abstraction/shapeFeatures.ts` | Yes | Port | COVERED |
| `transcript/ml/rulesetCompiler.ts` | Yes | Port | COVERED |
| `transcript/ml/apiCompiler.ts` | Yes | Port | COVERED |
| `transcript/ml/promptBuilder.ts` | Yes | Port | COVERED |
| `transcript/profiles/types.ts` | Yes | Port → contracts | COVERED |
| `transcript/profiles/registry.ts` | Yes | Port | COVERED |
| `transcript/profiles/validation.ts` | Yes | Port | COVERED |
| `transcript/schedules/extractor.ts` | Yes | Port | COVERED |
| `transcript/schedules/tableBuilder.ts` | Yes | Port | COVERED |

---

### `utils/` — ~9 source files
Phase 1 status: Complete  
Phase 7 module map: All mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `utils/bookmarks.ts` | Yes (deprecated) | Do not port | COVERED |
| `utils/bookmarkWriter.ts` | Yes | Port (trait) | COVERED |
| `utils/pdfLibBookmarkWriter.ts` | Yes | Do not port | COVERED |
| `utils/pikepdfWriter.ts` | Yes | Replace | COVERED |
| `utils/pdf.ts` | Yes | Port | COVERED |
| `utils/fs.ts` | Yes | Port | COVERED |
| `utils/sort.ts` | Yes | Port | COVERED |

---

### `workflows/` — ~16 source files
Phase 1 status: Partial  
Phase 7 module map: All active workflows mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `workflows/engine.ts` | Yes | Port | COVERED |
| `workflows/types.ts` | Yes | Port → contracts | COVERED |
| `workflows/merge/mergeWorkflow.ts` | Yes | Port (fix R-007) | COVERED |
| `workflows/specs-patch/specsPatchWorkflow.ts` | Yes (deprecated) | Restore (V4 Phase 2) | COVERED |
| `workflows/bookmarks/bookmarksWorkflow.ts` | Yes | Port | COVERED |
| `workflows/mappers/merge.ts` | Yes | Port | COVERED |

---

## GUI Module Coverage (`conset-pdf-gui/src/`)

### IPC Handlers (`src/main/ipc/`)
Phase 1 status: Inventoried  
Phase 7 module map: Most handlers mapped.

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `ipc/dialogs.ts` | Yes | Mapped (open_file_dialog) | COVERED |
| `ipc/pdf.ts` | Yes | Mapped (get_page_count, get_transcript) | COVERED |
| `ipc/profiles.ts` | Yes | Mapped (list_profiles, import_profile) | COVERED |
| `ipc/detection.ts` | Yes | Mapped (analyze_workflow) | COVERED |
| `ipc/operations.ts` | Yes | Mapped (execute_workflow) | COVERED |
| `ipc/history.ts` | Yes | Mapped (list_history) | COVERED |
| `ipc/system.ts` | Yes | Mapped (get_system_info) | COVERED |
| `ipc/merge.ts` | Yes | Mapped (plan_merge) | COVERED |
| `ipc/debug.ts` | Yes | Mapped (debug_walkthrough) | COVERED |
| `ipc/settings.ts` | Yes | Mapped (get_settings, set_settings) | COVERED |
| **`ipc/logging.ts`** | No | **Not in module map** | **GAP** |
| **`ipc/merge-internal.ts`** | No | **Not in module map** | **GAP** |

**Notes**:
- `logging.ts` handles log browsing/export, system info, log clearing, and renderer-to-main log forwarding. In V4 Tauri, this maps to logging/supportability commands plus Rust tracing integration.
- `merge-internal.ts` is a shared merge adapter/normalization layer backing public merge handlers and progress emission rather than a standalone end-user command.

### Phase 7 IPC Channel Map — Missing Files Note

The Phase 7 module map lists channels `cache:invalidate`, `naming:preview`, and `standards:normalize` as distinct Tauri commands. In the prototype, these are handled inline within existing IPC handlers (operations.ts, merge.ts, or system.ts), not as separate files. The mapping is logically correct but the source-file origin should be clarified for each.

### GUI Stores

| File | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `src/main/profiles/store.ts` | Yes | Port to serde_json + file I/O | COVERED |
| `src/main/history/store.ts` | Yes | Port | COVERED |

### GUI Frontend Modules

| Module | Phase 1 | Phase 7 Map | Status |
|---|---|---|---|
| `src/modules/roi/roiOverlayController.js` | Yes | Port (frontend JS/TS) | COVERED |
| Wizard UIs (merge, split, bookmarks, placeholder) | Yes | Port (frontend) | COVERED |
| `src/preload.ts` | Yes | Replace (Tauri invoke) | COVERED |

---

## ADR Coverage

All 8 ADRs required by Phase 3 are present on disk.

| ADR | Title | Artifact Present | Coverage Assessment |
|---|---|---|---|
| ADR-001 | Python Sidecar Pattern | Yes (`phase-03-adr-001-python-sidecar.md`) | **COVERED** — Phase 7 sidecar replacement plan provides full V4 resolution. |
| ADR-002 | Transcript-First Extraction | Yes (`phase-03-adr-002-transcript-first.md`) | **COVERED** — Phase 7 module map removes PDF.js, R-008 addressed. |
| ADR-003 | Workflow Engine Pattern | Yes (`phase-03-adr-003-workflow-engine.md`) | **COVERED** — Phase 7 primer documents Workflow trait with R-007 fix. |
| ADR-004 | Disk-Based Merge | Yes (`phase-03-adr-004-disk-streaming-merge.md`) | **COVERED** — Phase 7 primer mandates lopdf streaming from day 1; R-006 addressed. |
| ADR-005 | Determinism Invariant | Yes (`phase-03-adr-005-determinism.md`) | **COVERED** — Phase 7 primer enumerates all Rust determinism requirements. |
| ADR-006 | Profile-Driven Detection | Yes (`phase-03-adr-006-profile-driven-detection.md`) | **COVERED** — Phase 7 maps layout profile schema and crate contracts. |
| ADR-007 | Privacy-Preserving ML | Yes (`phase-03-adr-007-privacy-abstraction.md`) | **COVERED** — R-010 addressed; FULL_TEXT_OPT_IN gated by explicit flag + confirmation. |
| ADR-008 | Abandoned Technology Choices | Yes (`phase-03-adr-008-abandoned-choices.md`) | **COVERED** — Do-not-port entries in Phase 7 module map align with ADR-008 decisions. |

---

## Phase 4 Contract Coverage

All 7 required Phase 4 contracts are present on disk.

| Contract | Artifact Present | Phase 7 Coverage |
|---|---|---|
| Layout profile schema | Yes | `crates/contracts::layout` mapped |
| LayoutTranscript types | Yes | `crates/contracts::transcript` mapped |
| Workflow types | Yes | `crates/contracts::workflows` mapped |
| Bookmark types | Yes | `crates/contracts::bookmarks` mapped |
| IPC envelope contract | Yes | `crates/contracts::ipc` mapped with R-012 noted |
| Output formats contract | Yes | `crates/workflows::report` + `AuditBundle` mapped |
| Standards types | Yes | `crates/contracts::standards` mapped |

---

## Phase 5 Item Coverage Against Phase 7

| Phase 5 Item | Phase 7 Rust Answer | Assessment |
|---|---|---|
| Report Viewer GUI | Not explicitly addressed | OPEN — no V4 GUI design for report viewer. Needs deferral note. |
| Specs Patch GUI wizard | "Restore in V4 Phase 2" | DEFERRED WITH RATIONALE |
| Submittal workflow | "V4 Phase 4 first-class workflow" (R-014) | DEFERRED WITH RATIONALE |
| Automated ROI detection | Not addressed in Phase 7 | OPEN — no Rust design answer. Needs deferral note. |
| Equipment schedule extraction UI | Extractor ported; GUI surface not addressed | OPEN — GUI surface needs deferral note. |
| Web/SaaS mode | Future V4 phase (MASTER_PLAN_v4.md) | DEFERRED ✓ |
| Pattern Development Tool | V4 Phase 0.5 prerequisite | DEFERRED ✓ |
| Audit bundle and overlay export | `AuditBundle` type stub in contracts crate | PARTIAL — struct stub only; no full serialization spec |
| LLM-assisted narrative integration | Explicitly deferred in Phase 6 lessons | DEFERRED ✓ |
| Technical debt TD-001 through TD-007 | All addressed in Phase 7 module map or noted as open risks | COVERED |

---

## Open Risk Coverage

Unresolved items from `00-admin/open-questions-and-risks.md` and their Phase 8 disposition:

| Risk ID | Status | Phase 8 Action |
|---|---|---|
| R-001 (V4 GUI runtime) | OPEN | Deferred to architecture decision. Not a blocking extraction gap. |
| R-002 (footer section map stub) | OPEN | Captured in Phase 5 failure mode #1 and Phase 7 module map "Port + Complete." Rust developer has full context. |
| R-003 (footer parser Division-23) | OPEN | Captured in Phase 5 failure mode #2 and Phase 7 "Port + Extend" note. Rust developer has full context. |
| R-004 (quality-driven fallback) | OPEN | R-008 addressed extractor determinism. Quality gate fallback is moot in V4 (single path). Risk is retired. |
| R-005 (schedule extraction completeness) | OPEN | Phase 7 ports geometry-first extractor; TODOs become V4 backlog. Risk is appropriately residual. |
| R-009 (auto-layout capability drift) | OPEN | Phase 9 doc-drift matrix should capture this. Residual. |
| R-011 (technology lifecycle labeling) | OPEN | Phase 9 doc-drift matrix. Residual. |
| R-012 (IPC error envelope doc drift) | OPEN | Phase 7 module map ports code shape (structured error). Doc shape discrepancy remains for Phase 9. |
| R-013 (roadmap status drift) | OPEN | Phase 9. Residual. |

---

## Coverage Summary

| Category | Total Items | COVERED | PARTIAL | GAP |
|---|---|---|---|---|
| Core modules (`packages/core/src/`) | 15 directories | 12 | 2 (`specs/inventory`, `bookmarks/profiles` grouping) | 1 (`text/` absent from Phase 7 map) |
| Significant source files | ~90 TS+Python | ~78 | ~6 | ~6 |
| ADRs | 8 | 8 | 0 | 0 |
| Phase 4 Contracts | 7 | 7 | 0 | 0 |
| Phase 5 Deferred Items | 9 | 6 deferred with rationale | 1 partial (audit bundle) | 2 open (report viewer, auto-ROI, schedule GUI) |
| GUI IPC Handlers | 13 | 11 | 0 | 2 (logging.ts, merge-internal.ts) |

---

## Gap Register

Items confirmed as present in the codebase but absent or incomplete in the extraction library:

| Gap ID | Module | Missing From | Action Required |
|---|---|---|---|
| G-001 | `text/bandSlicer.ts` | Phase 7 module map | Add entry: `core-engine::text::band_slicer` (Port) |
| G-002 | `text/pageRegions.ts` | Phase 7 module map | Add entry: `core-engine::text::page_regions` (Port) |
| G-003 | `specs/extract/tableDetector.ts` | Phase 7 module map | Add entry: `core-engine::specs::extract::table_detector` (Port) |
| G-004 | `specs/footerIndexBuilder.ts` | Phase 7 module map | Add entry: `core-engine::specs::footer_index_builder` (Port) |
| G-005 | `specs/inventory/specFooterIndexer.ts` | Phase 7 module map | Add entry: `core-engine::specs::inventory::spec_footer_indexer` (Port) |
| G-006 | `specs/inventory/specSectionizer.ts` | Phase 7 module map | Add entry: `core-engine::specs::inventory::spec_sectionizer` (Port) |
| G-007 | `bookmarks/settings.ts` | Phase 7 module map | Add entry: `core-engine::bookmarks::settings` (Port) |
| G-008 | `bookmarks/profiles/raw.ts` | Phase 7 module map | Add entry: `core-engine::bookmarks::profiles::raw` (Port — raw bookmark shaping profile) |
| G-009 | `ipc/logging.ts` | Phase 7 IPC handler map | Add Tauri commands: `get_logs`, `export_log_bundle`, `set_log_level` |
| G-010 | `ipc/merge-internal.ts` | Phase 7 IPC handler map | Add Tauri command mapping for internal merge state channels |
| G-011 | Phase 5 open item: Report Viewer GUI | Phase 7 Rust answer | Add explicit deferral note: V4 Phase 3 GUI scope item |
| G-012 | Phase 5 open item: Automated ROI detection | Phase 7 Rust answer | Add explicit deferral note: V4 Phase 3+ ML pipeline item |
| G-013 | Phase 5 open item: Schedule extraction UI | Phase 7 Rust answer | Add explicit deferral note: V4 Phase 3 GUI scope item |
| G-014 | `specs/inventory/` path discrepancy | Phase 7 module map | Correct path from `specs/inventory.ts` → `specs/inventory/index.ts` |

---

## Checklist Verdict

**Phase 7 module map adequately covers ~90% of source modules.** Ten source-file-level gaps (G-001 through G-010) exist, all of which are well-understood utility or inventory submodule files. No entire workflow, no contract type, and no algorithm is missing from the documentation library. The gaps are additive annotations, not architectural omissions.

All 8 ADRs are present and address decisions with high implementation impact. All 7 type contracts are present. The Phase 5 open items have either Phase 7 Rust design answers or explicit deferral rationale for all but three items (G-011, G-012, G-013) which need deferral notes added.

**The documentation library is suitable for V4 Rust implementation planning with the gap register above as a known-gaps addendum.**

Post-phase addendum status: G-001 through G-014 were subsequently closed by updates to Phases 04 and 07 plus the Phase 9 telemetry capture. The checklist remains as the verification record of what had to be closed.
