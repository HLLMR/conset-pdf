# Phase 08 - Validation Report

**Phase**: Verification  
**Document Type**: Cross-Phase Validation and Final Codebase Pass  
**Date**: 2026-03-19  
**Reviewer**: GitHub Copilot (Claude Sonnet 4.6)

---

## Validation Scope

This report addresses the four validation objectives of Phase 8:

1. **Module coverage** — are any codebase modules missing from documentation?
2. **ADR completeness** — do all 8 ADRs address the highest-impact implementation decisions?
3. **Phase 5 resolution** — does every incomplete-work item have a Phase 7 Rust answer or explicit deferral?
4. **Final codebase pass** — anything a Rust developer would need that is not yet captured?

The gap details are in `phase-08-coverage-checklist.md`. This report provides findings, verdicts, and resolution actions.

---

## Validation 1: Module Coverage

### Method

- Ran `Get-ChildItem -Recurse -File` against all `packages/core/src/` subdirectories.
- Ran `Get-ChildItem -File` against `conset-pdf-gui/src/main/ipc/`.
- Cross-referenced results against Phase 1 inventory and Phase 7 module map.

### Findings

**10 source files confirmed absent from Phase 7 module map** (see gap register G-001 through G-010 in checklist). Summary by category:

#### A. `text/` module — entirely absent from Phase 7

`text/bandSlicer.ts` and `text/pageRegions.ts` were flagged in Phase 1 as "additional observations" (not in the extraction plan) and never added to Phase 7. These two files provide:

- `bandSlicer.ts`: Slices a page transcript into horizontal bands (top/bottom margin, body) used by chrome removal and bookmark heading resolution.
- `pageRegions.ts`: Returns named region extracts (header band, footer band, body) for a given page dimensions and span list.

**Impact for Rust**: Both files must be in `crates/core-engine`. They have no internal package dependencies and are straightforward geometry utilities. The Rust equivalents are `core-engine::text::band_slicer` and `core-engine::text::page_regions`. Port action for both.

#### B. `specs/` module — 4 files absent from Phase 7

1. `specs/extract/tableDetector.ts` — Detects table structures in spec pages (coordinate-cluster based). This feeds the specs AST and is listed in the extraction plan (Phase 2 algorithmic capture) but was not included in Phase 7 module map. V4 action: `core-engine::specs::extract::table_detector` (Port).

2. `specs/footerIndexBuilder.ts` — Distinct from `footerSectionMap.ts`. Builds the internal tabular footer index data structure from parsed footer IDs. Used as an input to `footerSectionMap.ts`. V4 action: `core-engine::specs::footer_index_builder` (Port).

3. `specs/inventory/specFooterIndexer.ts` — Part of the inventory submodule. Provides footer-indexed section entry generation. V4 action: `core-engine::specs::inventory::spec_footer_indexer` (Port).

4. `specs/inventory/specSectionizer.ts` — Part of the inventory submodule. Divides a spec document into logical sections for inventory assembly. V4 action: `core-engine::specs::inventory::spec_sectionizer` (Port).

**Path note**: The Phase 7 module map references `specs/inventory.ts` (a flat file), but the actual codebase has an `specs/inventory/` subdirectory with `index.ts` at its root. The mapping must be corrected to `specs/inventory/index.ts` → `core-engine::specs::inventory` (module root).

#### C. `bookmarks/` module — 2 files absent from Phase 7

5. `bookmarks/settings.ts` — Contains bookmark output configuration (title format, fit type, color settings). Used by all bookmark writers. V4 action: `core-engine::bookmarks::settings` (Port into `crates/contracts::bookmarks` as a settings type, or as `core-engine::bookmarks::settings` if it has behavior).

6. `bookmarks/profiles/raw.ts` — Provides the raw bookmark shaping profile that preserves outline structure with minimal normalization. V4 action: `core-engine::bookmarks::profiles::raw` (Port).

#### D. GUI IPC — 2 handler files absent from Phase 7

7. `ipc/logging.ts` — Handles log browsing/export, system info, log clearing, and renderer-to-main logging bridge calls. Key IPC channels include `log:getFiles`, `log:readFile`, `log:export`, `log:openDirectory`, `log:clear`, `log:getStats`, and `log:getSystemInfo`.

   **Rust developer relevance**: The logging IPC is the primary user-facing mechanism for the diagnostic/supportability requirement (see N-N #13 and Phase 6 ops telemetry lessons). V4 Tauri must implement equivalent commands backed by the `tracing` + `tracing-appender` logging stack.

8. `ipc/merge-internal.ts` — Shared merge adapter and normalization layer backing the public merge handlers. It should map to Tauri-side internal helpers plus progress events emitted from long-running workflow execution, rather than a standalone end-user command.

   **Rust developer relevance**: Progress streaming from long-running Tauri commands must be implemented with Tauri events (`tauri::Window::emit()`), not HTTP streaming. This is an implementation constraint that a Rust developer building the merge command would need.

### Coverage Verdict

> **PASS with addenda.** No entire workflow, algorithm, contract type, or architectural decision is undocumented. The 10 gap items are individually small (utility files, inventory submodule files, IPC handlers). The coverage baseline is sufficient for V4 Rust planning. The gap register in the checklist serves as a mandatory addendum to Phase 7.

---

## Validation 2: ADR Completeness

### Method

Reviewed each of the 8 ADR documents against the twelve highest-impact implementation decisions from the codebase (collected from Phase 3 plan, Phase 5 debt register, and Phase 6 lessons).

### Findings

#### ADR-001: Python Sidecar Pattern — VALID AND SUFFICIENTLY SCOPED

Impact level: **High**. The sidecar pattern governs PyMuPDF extraction, pikepdf bookmark writing, and the disk-based merge target. Phase 7 fully resolves: pdfium-render replaces PyMuPDF; lopdf replaces pikepdf. ADR-001 correctly identifies the licensing constraint that motivated the pattern and the Rust path forward. No gaps.

#### ADR-002: Transcript-First Extraction — VALID AND SUFFICIENTLY SCOPED

Impact level: **High**. Correctly identifies the PDF.js accuracy failure and the PyMuPDF migration rationale. R-008 (path-conditional determinism) is addressed: V4 has a single path (PDFium). The ADR is accurate. One note: ADR-002 should mention that PDF.js is retained for bookmark reading in the current prototype (TD-001) even after transcript extraction moved to PyMuPDF. This is a subtlety that distinguishes "transcript extraction" from "document analysis." The Phase 7 module map addresses it (TD-001 annotation on documentContext.ts), so the risk is documented even if ADR-002 does not explicitly call it out.

#### ADR-003: Workflow Engine Pattern — VALID AND SUFFICIENTLY SCOPED

Impact level: **Critical**. Three-phase analyze/applyCorrections/execute pattern is the fundamental correctness invariant for audit-safe operation. R-007 (analyze re-runs in applyCorrections) is called out and resolved in Phase 7 (`apply_corrections()` mutates without re-running). ADR-003 is accurate. No gaps.

#### ADR-004: Disk-Based Merge via Pikepdf Sidecar — VALID, SCOPE REQUIRES NOTE

Impact level: **High**. ADR-004 correctly captures the OOM motivation and the pikepdf intended target. As noted in R-006, the current executable path still uses in-memory pdf-lib assembly. Phase 7 declares this resolved by mandating lopdf disk-streaming from day one. The ADR accurately reflects the design intent; R-006 documents the prototype/intent gap. No gaps.

**Note for Rust developer**: ADR-004 describes the _intended_ architecture. The actual prototype executes in-memory assembly with pdf-lib for plan execution, then routes output through pikepdf only for the final write. When porting, do not read the prototype's actual `applyPlan.ts` code as the V4 architecture — follow ADR-004 and the Phase 7 sidecar replacement plan instead.

#### ADR-005: Determinism as a Design Invariant — VALID AND SUFFICIENTLY SCOPED

Impact level: **Critical**. Enumerates all non-determinism sources (date fields, sort tie-breaks, float rounding, random seeding). Phase 7 adds Rust-specific rules (`BTreeMap` not `HashMap`, stable sort with total ordering, SHA-256 test requirement). ADR-005 is the canonical reference. No gaps.

#### ADR-006: Profile-Driven Detection — VALID AND SUFFICIENTLY SCOPED

Impact level: **High**. Auto-detection was prototyped and failed; explicit JSON profiles are the validated approach. Phase 7 maps the profile schema to `crates/contracts::layout`. The open risk R-009 (--auto-layout capability drift in CLI docs) does not affect the ADR validity; it is a doc drift issue for Phase 9. No gaps.

#### ADR-007: Privacy-Preserving ML Abstraction — VALID, SCOPE NOTE

Impact level: **Medium-High**. TokenVault architecture and three privacy modes are documented. R-010 addressed: `FULL_TEXT_OPT_IN` requires explicit CLI flag + user confirmation. **Scope note**: The ADR documents the _mechanism_ (TokenVault) but does not specify the CLI/UI enforcement contract. Phase 7 adds the enforcement requirement. Together, ADR-007 + Phase 7 primer section on R-010 provide the complete picture. A Rust developer needs both sources; the cross-reference should be explicit.

#### ADR-008: Abandoned Technology Choices — VALID AND SUFFICIENTLY SCOPED

Impact level: **Medium** (primarily a risk-prevention ADR). Documents pdf-lib bookmark path, PDF.js demotion, in-memory merge retention, PDF AST abandonment, legacy locator deprecation, and LLM narrative deferral. All corresponding `Do not port` entries in Phase 7 module map are consistent with ADR-008. No gaps.

### ADR Verdict

> **PASS.** All 8 ADRs are present and address the decisions with the highest implementation impact. Minor cross-referencing gaps (ADR-002 bookmark surface note, ADR-007 enforcement contract) are covered in other documents and do not represent knowledge gaps for V4.

---

## Validation 3: Phase 5 Incomplete Work vs Phase 7 Resolution

### Method

Each Phase 5 item reviewed against:
1. Phase 7 documents (primer, module map, sidecar plan, dataset matrix)
2. Phase 6 lessons-learned (deferred items section)
3. Open-questions-and-risks log

### Findings

#### Phase 5 Unimplemented GUI Workflows

**Report Viewer (GUI)**
- Phase 5 status: Partial placeholder.
- Phase 7 answer: **Not explicitly addressed.**
- Verdict: OPEN. The Rust developer has no guidance on this feature. Required action: add explicit deferral — Report Viewer is a V4 Phase 3 GUI scope item. The `AuditBundle` type in `crates/contracts` is the data source for the viewer; actual UI design is deferred.

**Placeholder workflow shell**
- Phase 5 status: Routes unsupported workflow states to coming-soon shell.
- Phase 7 answer: Implied by wizard port entries.
- Verdict: ACCEPTABLE. Tauri eliminates the concept of an Electron placeholder shell; V4 route handling will natively absent-case to an error state or empty view. No explicit risk.

**Specs Patch GUI wizard**
- Phase 5 status: Not wired in GUI.
- Phase 7 answer: "Restore in V4 Phase 2" in module map entry for `specsPatchWorkflow.ts`.
- Verdict: **DEFERRED WITH RATIONALE.** ✓

**Submittal workflow (end-to-end)**
- Phase 5 status: Parser only, no workflow/CLI/GUI.
- Phase 7 answer: R-014 resolved — "V4 Phase 4 first-class workflow." Minimum scope defined (SubmittalWorkflow struct + SubmittalChromeMetadata stub in contracts crate).
- Verdict: **DEFERRED WITH RATIONALE.** ✓

#### Phase 5 Planned But Not Started Items

**Automated ROI detection and profile generation**
- Phase 5 status: Plan document only (`automatedRoiRefactorPlan.md`).
- Phase 7 answer: Not addressed. The dataset portability matrix classifies profile schemas but does not provide V4 design guidance for auto-detection.
- Verdict: OPEN. Required action: add explicit deferral — automated ROI detection is a V4 Phase 3+ machine learning pipeline item. The algorithmic design is in `automatedRoiRefactorPlan.md`; no prototype evidence validates it.

**Equipment schedule extraction UI**
- Phase 5 status: Core extractor implemented; no GUI surface.
- Phase 7 answer: Phase 7 module map ports `schedules/extractor.ts` and `schedules/tableBuilder.ts` but does not address a GUI workflow for schedule viewing.
- Verdict: OPEN. Required action: add explicit deferral — schedule extraction GUI is a V4 Phase 3 item. The `ScheduleTable` type needs to be in `crates/contracts` to support future GUI surfacing.

**Web/SaaS mode**
- Phase 7 answer: Explicitly in V4 MASTER_PLAN_v4.md as a future phase.
- Verdict: **DEFERRED WITH RATIONALE.** ✓

**Pattern Development Tool**
- Phase 7 answer: Explicitly in V4 Phase 0.5 as a prerequisite (MASTER_PLAN_v4.md).
- Verdict: **DEFERRED WITH RATIONALE.** ✓

**Audit bundle and visual overlay export**
- Phase 5 status: Aspirational; no complete artifact writer.
- Phase 7 answer: `AuditBundle` type stub mentioned but no serialization spec.
- Verdict: PARTIAL. The `AuditBundle` struct is defined as a V4 contracts type, but its full schema (what fields, what file format, what export path) is not documented. This is a V4 Phase 1 deliverable blocker — the audit trail requirement (N-N #8) depends on it. Minimum acceptable resolution: a JSON schema doc for `AuditBundle` in `04-contracts/`.

**LLM-assisted narrative integration**
- Phase 7 answer: Explicitly deferred. Algorithmic path covers most cases.
- Verdict: **DEFERRED WITH RATIONALE.** ✓

#### Phase 5 Technical Debt Items

| Debt | Phase 7 Resolution | Verdict |
|---|---|---|
| TD-001 (PDF.js bookmark shim) | Module map: Remove in Rust (use lopdf) | RESOLVED ✓ |
| TD-002 (pdf-lib in merge path) | Module map: Do not port (replace with lopdf) | RESOLVED ✓ |
| TD-003 (legacy bookmark util) | Module map: Do not port | RESOLVED ✓ |
| TD-004 (Playwright runtime) | Module map: Replace with headless_chrome | RESOLVED ✓ |
| TD-005 (feature flag discipline) | Phase 7: Cargo features + RuntimeConfig | RESOLVED ✓ |
| TD-006 (merge plan contract drift) | Phase 7 R-007 fix in workflow trait | RESOLVED ✓ |
| TD-007 (legacy docs status drift) | Deferred to Phase 9 doc-drift matrix | DEFERRED ✓ |

### Phase 5 Verdict

> **PASS with 3 OPEN items** (Report Viewer, Automated ROI UI, Schedule Extraction UI) and 1 PARTIAL (AuditBundle schema). All OPEN items are GUI scope items with no algorithmic complexity — they are V4 Phase 3 surface features. Resolution actions are documented in the gap register (G-011, G-012, G-013) and below. The PARTIAL on AuditBundle needs a follow-up contract doc.

---

## Validation 4: Final Codebase Pass

### Method

Searched for module types, patterns, and source paths not surfaced by the extraction library. Reviewed scripts/, tests/, test-output/, and layouts/ directories.

### Findings

#### Scripts (`scripts/`)

The `scripts/` directory contains 5 developer diagnostic scripts:

| Script | Purpose | V4 Relevance |
|---|---|---|
| `inspect-uds.ts` | Interactive UDS discipline lookup diagnostic | V4 standards CLI can include an equivalent `inspect` subcommand |
| `inspect-narrative.ts` | Interactive narrative parser probe | Useful as V4 development tooling |
| `test-v3-extraction.ts` | Manual transcript extraction tester | Maps to V4 integration test scaffold |
| `show-ml-input.ts` | Dumps ML abstraction layer output for a PDF | Development tooling for ML feature |
| `verify-invariants.js` | Byte-verbatim page invariant checker | **High V4 value**: should be promoted to a proper V4 integration test, not just a script |

**Action**: `verify-invariants.js` byte-verbatim checker logic should be captured as a formal V4 test requirement. This directly implements the SHA-256 page hash check for N-N #12. A Rust developer may otherwise re-derive this from scratch.

#### Test Fixtures (`tests/fixtures/`)

Phase 6 edge case catalog references test fixtures but does not enumerate them. The fixture inventory is relevant for migration parity testing.

Fixture categories identified:
- `tests/fixtures/` — input PDFs and JSON outputs for workflow regression tests
- `tests/standards/` — standards normalization test datasets
- `tests/transcript/` — transcript canonicalization regression cases
- `tests/narrative/` — narrative parser fixtures
- `tests/smoke/` — smoke test entry points

**Action**: No capture needed now, but V4 test corpus planning should include porting these fixtures to `tests/corpus/` as specified by MASTER_PLAN_v4.md.

#### Test Output (`test-output/23_MECH_FULL_fresh_run/`)

Real run artifacts for a 23 MECH full book processing run. These include:

- `04b-footer-validation.json` — shows the footer parser coverage gap (R-003) with real data
- Merge report JSON — validates output structure against Phase 4 contracts
- Detection output JSON — validates locator confidence scores and ID parsing under real conditions

These artifacts confirm the failure modes documented in Phase 5. No additional gaps surfaced.

#### Layout Template (`layouts/layout-template.json`)

Reviewed against the Phase 4 layout profile schema contract. The template is the canonical schema source. Phase 4 contract correctly reflects all fields. No gaps.

#### GUI `docs/LOGGING_IMPLEMENTATION.md`

Not reviewed in prior phases. Contains concrete constraints for the logging IPC:

Key details relevant to the G-009 gap:
- Log rotation: max 5 files × 10MB each
- Log export: compressed bundle with session logs + system metadata JSON
- Unhandled error capture: `process.on('uncaughtException')` and `unhandledRejection` both forward to log with full stack
- Session start/end markers required for supportability

These constraints translate directly to V4 Tauri requirements:
- Use `tracing-appender` with rolling file sink, 5-file × 10MB cap
- `export_log_bundle` Tauri command: zip rolling log files + emit `system::get_info` output
- Register `std::panic::set_hook` in Rust main to capture panics as structured log entries before crash
- Emit session start marker at process launch (timestamp, version, OS, config hash)

**This is a V4 implementation constraint that is not currently captured in any extraction document.** It is not covered by Phase 9's planned telemetry lessons; it should be captured now.

#### GUI `docs/PRE_ALPHA_CHECKLIST.md` and `docs/ALPHA_TESTING_GUIDE.md`

These confirm the prototype was deployed to real users. The alpha testing guide defines bug report format (steps to reproduce, log bundle, version). This is the operational context for the logging and supportability requirements.

No new algorithmic gaps. Confirms that the V4 supportability baseline needs to meet or exceed the alpha-era logging design.

#### `DEPRECATION_CHANGES.md`

Documents `ENABLE_LEGACY_LOCATOR` as the one active deprecation gate with explicit CLI behavior and error messaging. Phase 7 correctly marks legacyTitleblockLocator as "do not port." The deprecation gate itself has no V4 equivalent needed (the deprecated path is removed). No gaps.

---

## Consolidated Gap Resolution Plan

All gaps from G-001 through G-014 identified in the coverage checklist are categorized here with resolution priority:

### Priority 1 — Required before V4 Phase 1 kickoff

| Gap ID | Description | Recommended Action |
|---|---|---|
| G-001, G-002 | `text/` module absent from Phase 7 | Add 2 entries to Phase 7 module map |
| G-003 | `specs/extract/tableDetector.ts` | Add entry to Phase 7 module map |
| G-004 | `specs/footerIndexBuilder.ts` | Add entry to Phase 7 module map |
| G-005, G-006 | `specs/inventory/` submodule files | Add 2 entries + fix path discrepancy in Phase 7 map |
| G-007, G-008 | `bookmarks/settings.ts`, `bookmarks/profiles/raw.ts` | Add 2 entries to Phase 7 module map |
| G-009 | `ipc/logging.ts` — logging IPC | Add Tauri command mapping; capture logging constraints from LOGGING_IMPLEMENTATION.md |

### Priority 2 — Required before V4 Phase 2 kickoff

| Gap ID | Description | Recommended Action |
|---|---|---|
| G-010 | `ipc/merge-internal.ts` — merge progress events | Add Tauri event mapping (not command) |
| G-011 | Report Viewer GUI — no Phase 7 answer | Add deferral note to Phase 7 Rust Port Primer: "V4 Phase 3 GUI scope" |
| G-012 | Automated ROI detection — no Phase 7 answer | Add deferral note to Phase 7 Rust Port Primer: "V4 Phase 3+ ML pipeline" |
| G-013 | Schedule extraction UI — no Phase 7 answer | Add deferral note to Phase 7 Rust Port Primer: "V4 Phase 3 GUI scope; ScheduleTable type should be in contracts from Phase 1" |
| G-014 | `specs/inventory/` path discrepancy | Fix path in Phase 7 module map |

### Priority 3 — Before Phase 9 / Migration Planning

| Item | Description | Recommended Action |
|---|---|---|
| AuditBundle schema | Partial Phase 7 coverage | Add `phase-04-audit-bundle-schema.md` contract doc |
| Logging constraints | Not captured anywhere | Add logging constraint summary to Phase 9 ops-telemetry doc or Phase 7 primer |

---

## Validation Summary

| Validation Objective | Verdict | Open Items |
|---|---|---|
| Module Coverage | PASS with addenda | 14 gaps (all minor, all documented) |
| ADR Completeness | PASS | 0 blocking gaps |
| Phase 5 Resolution | PASS with 3 open + 1 partial | G-011, G-012, G-013, AuditBundle schema |
| Final Codebase Pass | PASS with 1 new finding | Logging constraints from LOGGING_IMPLEMENTATION.md |

**Overall Phase 8 verdict: The extraction library is structurally complete and valid for V4 Rust planning.** All gaps are enumerated, bounded, and actionable. No unknown algorithmic territory or missing contract type was discovered in the final codebase pass. The gap register and priority resolution plan above are the concrete V4 Phase 0 addendum items.

Post-phase addendum note: the documentation-specific gap items identified here were later closed by the Phase 4 AuditBundle schema addendum, the Phase 7 module-map/primer updates, and the Phase 9 telemetry capture. What remains open is outside the extraction library itself.
