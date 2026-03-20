# Phase 09 — Summary

**Phase**: Ops and Drift  
**Status**: COMPLETE  
**Owner**: GitHub Copilot (Claude Sonnet 4.6)  
**Date**: 2026-03-19

---

## Completion Status

Phase 9 is complete. Steps 46 through 50 are complete.

Completed artifacts:

1. `phase-09-doc-drift-matrix.md` — Full "Doc Claim vs Reality" matrix comparing README.md, ROADMAP.md, docs/WORKFLOWS.md, and docs/CLI.md against code truth. 9 sections, 6 open doc-fixing items identified, migration-gate truth table.
2. `phase-09-deprecation-gates.md` — All deprecation controls, defaults, failure messaging, and Rust feature-toggle implications. Covers `ENABLE_LEGACY_LOCATOR`, `logDeprecation*` utilities, and dead CLI flags.
3. `phase-09-ops-telemetry-lessons.md` — Logging architecture lessons from GUI prototype (5×10MB rotation, dual-process IPC logging, export bundle), alpha bug-reporting process, and minimum support telemetry baseline for Rust successor.
4. `phase-09-roi-coordinate-spec.md` — Canonical coordinate-space reference. Documents all four coordinate spaces, two conversions, rotation normalization, and Rust implementation requirements. Resolves all ambiguous wording in existing docs.
5. `phase-09-ux-critical-path.md` — Workflow-critical test path, IPC envelope constraints, analysis caching lifecycle, ROI overlay lifecycle, wizard state persistence rules, multi-lane behavior, stacking flow, and correction row stability invariant. 10 acceptance criteria for Rust/Tauri.

---

## What Was Done

### Step 46: Documentation Drift Matrix

Compared four canonical docs against code and found the following significant drift:

1. **ROADMAP summary table vs GUI section**: The summary table claims ✅ Complete for Extract Documents (Split) GUI and Fix Bookmarks GUI. The GUI narrative section says these are "⚠️ Placeholder UI Exists / Blocked By: Core workflow engines not implemented." Code confirms the GUI wizards are placeholder-only. **Table is wrong; GUI section is correct.**

2. **README stale "Partially Implemented" entry**: README lists "Split/Assemble workflows - CLI commands exist, workflow engine not yet implemented" as partially implemented, but also lists Extract Documents (Split) as fully implemented in the same document. The split workflow engine IS complete. This is an unremoved stale entry.

3. **ROADMAP duplicate Specs Patch entry**: The workflow appears once in the ✅ Fully Complete section and again in the ❌ Abandoned section. Should be collapsed to abandoned-only.

4. **Dead CLI flags `--auto-layout` / `--save-layout`**: Both options are declared in `mergeAddenda.ts` and documented in CLI.md, but `options.autoLayout` and `options.saveLayout` are never referenced in the action handler. Confirmed dead flags.

5. **Undocumented commands**: `debugWalkthrough.ts` and `specsInventory.ts` exist in `packages/cli/src/commands/` but are not mentioned in any public documentation.

### Step 47: Deprecation Gates

Captured all deprecation controls:
- `ENABLE_LEGACY_LOCATOR` feature flag: default OFF, env var `ENABLE_LEGACY_LOCATOR=true` to re-enable; failure message when blocked
- `logDeprecation()`, `logLegacyLocatorUsage()`, `logPdfAstDeprecation()` utilities
- Dead code inventory: `pdfLibBookmarkWriter`, `utils/bookmarks.ts`, assemble-set, specs-patch, `--auto-layout`/`--save-layout` flags
- Rust implications: legacy locator must not be ported; dead flags must not ship unless implemented

### Step 48: Operational Telemetry Lessons

Extracted concrete logging constraints from `LOGGING_IMPLEMENTATION.md`:
- Rotation: 5 files × 10 MB (resolved discrepancy — LOGGING_IMPLEMENTATION.md says 10 MB, not 2 MB)
- Session markers: startup logged with session ID + system info; no session-end marker (identified gap)
- Unhandled error capture: both processes covered
- Export bundle format documented
- Defined minimum telemetry baseline: session lifecycle, operation timing, failure context, reproducibility bundle, privacy rules
- Identified tracing crate recommendations for Rust successor
- Resolved R-018 (logging constraints not captured in extraction library)

### Step 49: UX Critical Path and Sharp Edges

Documented:
- Minimum smoke test path (5-step: profile → PDF → ROI → save → merge)
- Full 4-step merge wizard critical path
- IPC envelope rules: never throw, always return IpcResponse<T>, error properties preserved
- Analysis caching key and invalidation triggers
- ROI overlay lifecycle: viewport reset → overlay cleanup → rebind on next canvas attach
- Profile type switching (drawings vs specs key mapping)
- Wizard state persistence: profiles and history on disk; analysis results in-memory only
- Multi-lane behavior and profile-type validation requirement
- "Next Addendum" stacking: corrections discarded between cycles
- Row stability invariant: `row.id` must be stable across re-analysis
- 10 acceptance criteria for Rust/Tauri front ends

### Step 50: ROI Coordinate Space Canonical Specification

Produced the definitive coordinate system reference:
- Four spaces defined: Profile/ROI (BL normalized), Screen/Canvas (TL pixels), Text Item (TL points), Transcript Bbox (TL points)
- Conversion chain: Screen → Profile via `screenToNormalizedROI()` (Y-flip); Profile → Text Items via `getTextItemsInROI()` (Y-flip + denormalize)
- Rotation normalization table for 0°/90°/180°/270°
- `roiSpace: "visual"` vs `"pdf"` disambiguation: only "visual" is implemented; "pdf" path is a hard no-op trap
- Regression guard tests identified (existing: `roi-coordinates.test.ts`; required addition: end-to-end ROI query test)
- Resolved all ambiguous wording in QUICK_START.md, canonicalize.ts, layout-template.json comments

---

## Evidence Reviewed

- `conset-pdf/README.md` (repo root)
- `conset-pdf/ROADMAP.md`
- `conset-pdf/docs/WORKFLOWS.md`
- `conset-pdf/docs/CLI.md`
- `conset-pdf/DEPRECATION_CHANGES.md`
- `conset-pdf/packages/core/src/config/featureFlags.ts`
- `conset-pdf/packages/cli/src/commands/mergeAddenda.ts` (action handler; confirmed dead flags)
- `conset-pdf/packages/cli/src/commands/` directory listing (found undocumented commands)
- `conset-pdf/packages/core/src/layout/types.ts` (NormalizedROI type definition)
- `conset-pdf/packages/core/src/analyze/pageContext.ts` (getTextItemsInROI conversion)
- `conset-pdf/packages/core/src/transcript/canonicalize.ts` (rotation normalization)
- `conset-pdf/packages/core/src/locators/roiSheetLocator.ts` (ROI query usage)
- `conset-pdf/layouts/layout-template.json`
- `conset-pdf/docs/QUICK_START.md`
- `conset-pdf-gui/docs/LOGGING.md`
- `conset-pdf-gui/docs/LOGGING_IMPLEMENTATION.md`
- `conset-pdf-gui/docs/ALPHA_TESTING_GUIDE.md`
- `conset-pdf-gui/docs/PRE_ALPHA_CHECKLIST.md`
- `conset-pdf-gui/docs/DEVELOPMENT.md`
- `conset-pdf-gui/docs/UI_WORKFLOWS.md`
- `conset-pdf-gui/src/shared/ipc-response.ts`
- `conset-pdf-gui/src/modules/roi/roiOverlayController.js`
- `conset-pdf-gui/tests/integration/roi-coordinates.test.ts`
- Prior phase summaries: `00-admin/phase-manifest.md`, `08-verification/phase-08-summary.md`
- `00-admin/open-questions-and-risks.md`

---

## Open Questions and Carry-Forward Risks

| Item | Phase Raised | Status |
|------|-------------|--------|
| R-012: IPC error envelope doc drift (plain string vs structured) | Phase 4 | RESOLVED — phase-09-ux-critical-path.md §2 documents the correct structured contract |
| R-013: Roadmap status drift (split/bookmark/specs-patch stale labels) | Phase 5 | RESOLVED — phase-09-doc-drift-matrix.md §1 fully documents divergences and migration-gate truth table |
| R-018: Logging constraints not captured in extraction library | Phase 8 | RESOLVED — phase-09-ops-telemetry-lessons.md §1 documents all logging constraints |
| R-009: `--auto-layout` CLI cap drift | Phase 3 | RESOLVED — phase-09-doc-drift-matrix.md §3 confirms dead flag; phase-09-deprecation-gates.md §4 classifies it |
| ROI coordinate ambiguity across docs | Phase 9 | RESOLVED — phase-09-roi-coordinate-spec.md is the canonical reference |
| 6 doc-fixing items (ROADMAP table, README stale entry, etc.) | Phase 9 | OPEN — not blocking Rust planning; doc improvements deferred |

---

## Notes for Post-Phase Work

1. The ROADMAP summary table must be corrected to show ⚠️ for Extract Documents (Split) GUI and Fix Bookmarks GUI. This should be done before any external review of the prototype repo.

2. The `phase-09-roi-coordinate-spec.md` document should be referenced from `docs/QUICK_START.md` and `docs/ARCHITECTURE.md` once those are refreshed for V4.

3. The minimum telemetry baseline in `phase-09-ops-telemetry-lessons.md §4` should be promoted to a formal V4 non-negotiable or added to `docs/MASTER_PLAN_v4.md`.

4. The acceptance criteria in `phase-09-ux-critical-path.md §10` should be incorporated into the V4 Tauri/GUI architecture specification.
