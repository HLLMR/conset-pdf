# Phase 07 - Summary

**Phase**: Reference Library Assembly & Rust Handoff  
**Status**: COMPLETE  
**Owner**: GitHub Copilot (Claude Sonnet 4.6)  
**Date**: 2026-03-19

---

## Completion Status

Phase 7 is complete. Steps 38 through 41 are complete.

Completed artifacts:

1. `phase-07-rust-port-primer.md` — Primary entry point for V4 Rust developers
2. `phase-07-ts-to-rust-module-map.md` — Complete TS→Rust module-level mapping, including post-Phase 8 addendum coverage for `text/`, specs inventory helpers, bookmark settings/profile files, and GUI logging/merge IPC helpers
3. `phase-07-sidecar-replacement-plan.md` — Python sidecar replacement with native Rust
4. `phase-07-dataset-portability-matrix.md` — Dataset and regex portability classification
5. Updated `README.md` — Final navigation index covering all 9 phases

---

## What Was Captured

### Step 38: Organized Extraction Documents with Clear Index

Updated `prototype-postmortem/README.md` to serve as the definitive navigation index:
- Phase index with status (COMPLETE/NOT STARTED) for all 9 phases
- Explicit entry point for V4 Rust developers: `07-rust-handoff/phase-07-rust-port-primer.md`
- Per-phase artifact tables with one-line descriptions
- Admin file reference with purpose annotations

### Step 39: Rust Port Primer (Module and Dependency Mapping)

`phase-07-rust-port-primer.md` covers:

**V4 repository structure** per MASTER_PLAN_v4.md:
- `crates/core-engine` — computation kernel
- `crates/workflows` — workflow orchestration
- `crates/contracts` — shared serde types (GUI/CLI boundary)
- `crates/standards-data` — embedded static lookup tables
- `apps/backend-cli` — CLI binary
- `apps/desktop-gui` — Tauri GUI

**Crate ownership**: Each TS module directory mapped to its V4 Rust crate + submodule.

**External library choices**:
- `pdfium-render` (Apache-2.0) replaces PyMuPDF sidecar
- `lopdf` (MIT) replaces pikepdf/QPDF sidecar and pdf-lib merge assembly
- `serde` + `serde_json` for all contract serialization
- `regex` crate for verbatim pattern ports
- `tauri` for GUI
- `phf` for O(1) compile-time datasets (standards lookup tables)
- `tracing` for structured logging and audit trail

**Three-phase workflow pattern**: Documented as an audit-safety invariant. Captured the critical R-007 correction: `apply_corrections()` must mutate prior state without re-running `analyze()`.

**Coordinate system**: Documented the PDFium bottom-left → top-left transformation with exact Rust code. Rotation normalization requirement.

**Determinism**: Full rules — `BTreeMap` not `HashMap`, stable sort with total ordering, coordinate rounding to 2 decimal places, content hash field exclusions.

**Byte-verbatim page invariant** (NN-12): Documented the SHA-256 test requirement.

**Error handling** (NN-13): `catch_unwind` at workflow entry points, 256MB memory cap, per-page soft-fail.

**ADR cross-reference matrix**: V4 phase → ADR → risk mapping table.

**Submittal decision** (R-014): Recommended promoting submittals to first-class V4 Phase 4 workflow.

**Post-Phase 8 addenda folded back into Phase 7**:
- Closed module-map gaps for `text/bandSlicer.ts`, `text/pageRegions.ts`, `specs/extract/tableDetector.ts`, `specs/footerIndexBuilder.ts`, `specs/inventory/index.ts`, `specs/inventory/specFooterIndexer.ts`, `specs/inventory/specSectionizer.ts`, `bookmarks/settings.ts`, `bookmarks/profiles/raw.ts`, `src/main/ipc/logging.ts`, and `src/main/ipc/merge-internal.ts`
- Added explicit Phase 7 deferral notes for Report Viewer GUI, automated ROI detection UI, and schedule extraction UI
- Bound `AuditBundle` usage in Phase 7 to the explicit Phase 4 schema doc

### TypeScript–to–Rust Module Map

`phase-07-ts-to-rust-module-map.md` covers:

Every source file in `packages/core/src/` and `conset-pdf-gui/src/` mapped to its Rust equivalent with classification (Port / Replace / Do not port) and porting constraints.

Key decisions documented:
- `transcript/types.ts` → `crates/contracts::transcript` (shared type, not internal)
- `pdfjsExtractor.ts` → removed entirely (no V4 fallback path)
- `extract-transcript.py` → replaced by `pdfium-render` Rust crate
- `bookmark-writer.py` → replaced by `lopdf` indirect objects
- `pdfLibBookmarkWriter.ts` → not ported (superseded, TD-002)
- `utils/bookmarks.ts` → not ported (deprecated, TD-003)
- `assembleSet.ts` → not ported (deprecated, ADR-008)
- `legacyTitleblockLocator.ts` → not ported (deprecated)
- Electron `preload.ts` → replaced by Tauri `invoke` IPC
- `specsPatchWorkflow.ts` → restore in V4 Phase 2 with native rendering

IPC handler-to-Tauri-command mapping provided for all 16 IPC channels.

### Sidecar Replacement Plan

`phase-07-sidecar-replacement-plan.md` covers:

**Sidecar 1 (PyMuPDF extraction)**:
- What the sidecar does (JSON payload via temp file, `rawdict` extraction)
- Why PyMuPDF was chosen over PDF.js (bounding box accuracy)
- `pdfium-render` extraction pattern with Rust code sample
- Coordinate transformation code (PDFium bottom-left → normalized top-left)
- Rotation normalization requirements
- Stable sort algorithm
- PDFium distribution strategy per-platform

**Sidecar 2 (pikepdf/QPDF bookmark writing)**:
- Why pikepdf was required (indirect objects, bidirectional link chains)
- `lopdf` replacement with the exact PDF outline dictionary structure requirement  
- GoTo destination format (`XYZ null null null` with page object references)
- Post-write validation requirements (mandatory, not optional)

**Sidecar 3 (pikepdf passthrough write)**:
- How V4 eliminates this with native `lopdf` writes

**Disk-streaming merge** (ADR-004 restoration):
- Rust code outline for `apply_merge_plan()` using `lopdf` page-stream copy
- Memory constraint: one page in memory at a time
- Atomic write: temp-file-then-rename

### Step 40: ADR and Lessons Cross-References

`phase-07-rust-port-primer.md` includes:
- ADR cross-reference table: each V4 architectural decision → prototype ADR
- V4 Phase–to–ADR reference matrix: each implementation phase → relevant ADRs → relevant lessons → key risk
- Open risk carry-forwards: R-008, R-010, R-014 explicitly addressed

### Step 41: Dataset Portability Matrix

`phase-07-dataset-portability-matrix.md` covers every dataset, regex, and schema with portability classification (V = verbatim, A = adapted, R = re-expression, X = do not port):

**Standard datasets** (all V — port verbatim):
- `UDS_DESIGNATORS` (12 entries) → `phf::Map`
- `ALIAS_MAPPINGS` (10 entries) → const slice
- `CONTROLS_KEYWORDS` / `CIVIL_KEYWORDS` → const slices
- `disciplines.generated` table → const slice
- `divisions.generated` table → const slice
- `masterformatDivisions` (50 divisions) → const slice
- `drawingsOrderHeuristic` sort order → const slice
- `legacySections.generated` mapping → const slice

**Regex patterns** (all V — port verbatim):
- `DEFAULT_DRAWINGS_PATTERN` → `regex` crate with `once_cell::sync::Lazy`
- Spec section ID patterns (modern + legacy) → `regex` crate
- Narrative parser regexes → `regex` crate
- Division-23 footer ID pattern → `regex` crate (extend to all divisions in V4)

**Schemas** (V with serde adaptation):
- `LayoutProfile` → `crates/contracts::layout` with `serde(rename_all = "camelCase")`
- `LayoutTranscript` hierarchy → `crates/contracts::transcript`
- `IpcResponse<T>` envelope → `crates/contracts::ipc` (structured error shape, not plain string)
- `InventoryResult` and workflow types → `crates/contracts::workflows`
- `MergePlan` / `MergeAction` → fully serializable for cross-process handoff

**Algorithmic re-expression required** (R):
- `isConstructionDrawingSize()` — ANSI/ARCH size comparison logic
- Near-match scoring for narrative suggestions
- All-divisions footer ID pattern (R-003 extension)
- ID normalization (whitespace collapse, uppercase)

---

## Evidence Reviewed

Reference documents read for Phase 7:

- `docs/postMortemDocExtraction.md` — Phase 7 step definitions (steps 38–41)
- `00-admin/phase-manifest.md` — current phase status
- `00-admin/open-questions-and-risks.md` — R-001 through R-014
- `06-lessons/phase-06-summary.md` — Phase 7 entry point requirements
- `06-lessons/phase-06-architecture-overview.md` — complete module map and data flows
- `06-lessons/phase-06-non-negotiables-rust-constraints.md` — NN priority matrix
- `06-lessons/phase-06-lessons-learned.md` — what to replicate vs. replace
- `03-adrs/phase-03-adr-001-python-sidecar.md` — sidecar boundary and replacement direction
- `04-contracts/phase-04-workflow-types-contract.md` — type shapes to port
- `04-contracts/phase-04-layout-transcript-contract.md` — transcript wire contract
- `04-contracts/phase-04-layout-profile-schema.md` — profile schema fields
- `05-gaps-and-limitations/phase-05-technical-debt-register.md` — TD items to not port
- `docs/MASTER_PLAN_v4.md` — V4 technology stack, repository structure, non-negotiables
- `packages/core/package.json` — prototype npm dependencies (pdf-lib, pdfjs-dist, playwright)
- `packages/core/src/standards/datasets/drawingsDesignators.ts` — actual designator table
- `packages/core/src/parser/drawingsSheetId.ts` — actual regex patterns
- `layouts/layout-template.json` — canonical profile schema example

---

## ADR and Risk Resolution

### Risks targeted by Phase 7

**R-008 (Extractor-path Determinism Gap)**: Addressed. `phase-07-rust-port-primer.md` documents that V4 has a single extraction path (PDFium) with no fallback that relaxes determinism. The prototype's path-conditional determinism is eliminated.

**R-010 (Privacy Mode Policy Drift)**: Addressed. `phase-07-rust-port-primer.md` documents that `FULL_TEXT_OPT_IN` must be guarded by explicit CLI/GUI user confirmation. `phase-07-ts-to-rust-module-map.md` notes this in the token_vault entry.

**R-014 (Submittal Orchestration Gap)**: Addressed. `phase-07-rust-port-primer.md` documents the recommendation to promote submittals to first-class V4 Phase 4 workflow with minimum scope definition (struct stubs in `contracts`).

**R-015 (Module Map Gaps)**: Addressed. `phase-07-ts-to-rust-module-map.md` now includes the previously missing `text/`, specs inventory/helper, bookmark settings/profile, and GUI IPC support files.

**R-016 (Phase 5 GUI Deferral Gaps)**: Addressed. `phase-07-rust-port-primer.md` now explicitly defers Report Viewer, automated ROI detection UI, and schedule extraction UI while preserving the required contract dependencies for later GUI work.

**R-017 (AuditBundle Schema Undefined)**: Addressed via Phase 4 addendum. `phase-07-rust-port-primer.md` now points to `04-contracts/phase-04-audit-bundle-schema.md` as the authoritative wire contract.

### Risks remaining open

The following risks from the backlog remain unresolved as they are outside Phase 7 scope:

- R-001 (V4 GUI runtime decision) — product/architecture decision
- R-002 (specs footer anchoring stub) — V4 Phase 2 implementation work
- R-003 (footer parser coverage) — V4 Phase 2 implementation work
- R-004 (quality-driven fallback) — retired by the single-path PDFium architecture
- R-005 (schedule extraction completeness) — V4 Phase 3 work
- R-006 (merge sidecar maturity) — resolved by disk-streaming mandate in Phase 7
- R-007 (workflow state reuse) — V4 design correction documented in Phase 7
- R-009 (auto-layout capability drift) — V4 Phase 1.5 work
- R-011 (technology labeling drift) — later resolved by Phase 9 drift matrix and migration-gate truth table
- R-012 (IPC error envelope drift) — documented in portability matrix; targeted at Phase 9
- R-013 (roadmap status drift) — targeted at Phase 9

---

## Open Questions and Carry-Forward

No new risks were raised from Phase 7 content itself.

**Spec renderer decision**: `headless_chrome` vs. `printpdf`+`ab_glyph` for V4 spec section rendering is flagged in the primer as unresolved. This decision should be made in V4 Phase 2 kickoff before rendering implementation begins.

**PDFium distribution**: The exact mechanism for statically linking or bundling `libpdfium` for Tauri packaging on all three platforms (Windows, macOS, Linux) should be prototyped early in V4 Phase 1 to surface any packaging complications.

---

## Next Step Readiness

Phase 7 is complete and Phase 8 (Verification) can start.

Phase 8 entry point artifacts:
- `07-rust-handoff/phase-07-rust-port-primer.md` — primary reference
- `07-rust-handoff/phase-07-ts-to-rust-module-map.md` — coverage verification base
- `06-lessons/phase-06-non-negotiables-rust-constraints.md` — constraint parity checklist

Phase 8 agent must verify:
1. Every module in `packages/core/src/` appears in the module map with a disposition
2. All 8 ADRs are cross-referenced in Phase 7 or Phase 6 documents
3. Every Phase 5 incomplete item has either a Rust design answer in Phase 7 or an explicit deferred status with rationale
4. No codebase module that a Rust developer would need to know about is missing from coverage
