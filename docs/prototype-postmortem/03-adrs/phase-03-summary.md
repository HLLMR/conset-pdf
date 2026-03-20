# Phase 03 - Summary

**Phase**: Architecture Decision Records  
**Status**: IN_PROGRESS  
**Owner**: GitHub Copilot (GPT-5.4)  
**Date**: 2026-03-19

## Completion Status

Phase 3 has started. Steps 14 through 21 are complete.

Completed artifact:

1. `phase-03-adr-001-python-sidecar.md`
2. `phase-03-adr-002-transcript-first.md`
3. `phase-03-adr-003-workflow-engine.md`
4. `phase-03-adr-004-disk-streaming-merge.md`
5. `phase-03-adr-005-determinism.md`
6. `phase-03-adr-006-profile-driven-detection.md`
7. `phase-03-adr-007-privacy-abstraction.md`
8. `phase-03-adr-008-abandoned-choices.md`

## What Was Captured

- why the prototype introduced Python process boundaries instead of keeping specialized PDF operations inside the TypeScript runtime
- the actual affected subsystems: PyMuPDF transcript extraction, pikepdf/QPDF bookmark writing, and pikepdf passthrough writing
- operational costs of the pattern: interpreter discovery, packaging, temp files, subprocess overhead, and version coupling
- the Rust replacement intent: preserve the narrow backend boundary while replacing Python with native PDFium/object-manipulation subsystems
- why extraction moved from PDF.js-centered text access to a canonical transcript model
- how `DocumentContext` now performs one transcript extraction at initialization and serves downstream consumers from cached, canonicalized transcript data
- the real current fallback contract: PyMuPDF first, PDF.js on availability/runtime failure, with quality scoring reported separately rather than driving automatic retry
- why the codebase standardized on `analyze -> applyCorrections -> execute` as the shared user-facing orchestration contract
- where the current implementation falls short of the full plan-caching ideal: concrete workflows re-run `analyze()` during corrections, and merge execute does not rely primarily on previously analyzed plan state
- why merge architecture moved toward a planner-driven, disk-based pikepdf sidecar model for large-file safety
- where current merge implementation still falls short of that direction: final output is written through pikepdf, but page assembly remains in-memory in `pdf-lib`
- which non-determinism sources are actively controlled today: timestamp exclusion in canonical hashes, stable ordering/tie-breaks, coordinate normalization, and deterministic planner ordering
- where determinism remains conditional in current implementation: PDF.js fallback tolerates coordinate-level variance in tests, and numeric precision normalization before hashing could be stronger
- why profile-driven ROI detection is the canonical drawings path in current code, and why full auto-layout/profile-selection should be treated as planned/partial rather than complete
- where profile-related behavior diverges between docs and code: CLI exposes `--auto-layout`/`--save-layout`, but merge command execution currently only consumes explicit layout file or inline ROI input
- why TokenVault + sanitize abstraction is required before ML compiler calls, including placeholder-first payload design and structural-signal preservation
- where privacy guarantees are conditional in implementation: strict mode is default, but full-text opt-in and caller-provided context can widen exposure if used without policy controls
- which "abandoned" technology choices are fully retired versus still partially active in runtime paths (bookmarks, extraction fallback surfaces, merge assembly, scoped AST usage)
- where status-language drift exists between docs and executable command/workflow registration

## Evidence Reviewed

Primary code evidence:

- `packages/core/src/transcript/extractors/pymupdfExtractor.ts`
- `packages/core/src/transcript/sidecar/extract-transcript.py`
- `packages/core/src/bookmarks/pikepdfBookmarkWriter.ts`
- `packages/core/src/bookmarks/sidecar/bookmark-writer.py`
- `packages/core/src/utils/pikepdfWriter.ts`
- `packages/core/src/core/applyPlan.ts`
- `packages/core/src/analyze/documentContext.ts`
- `packages/core/src/transcript/factory.ts`
- `packages/core/src/transcript/extractors/pdfjsExtractor.ts`
- `packages/core/src/transcript/quality.ts`
- `packages/core/src/workflows/engine.ts`
- `packages/core/src/workflows/types.ts`
- `packages/core/src/workflows/merge/mergeWorkflow.ts`
- `packages/core/src/workflows/specs-patch/specsPatchWorkflow.ts`
- `packages/core/src/workflows/bookmarks/bookmarksWorkflow.ts`
- `packages/core/src/core/mergeAddenda.ts`
- `packages/core/src/core/planner.ts`
- `packages/core/src/transcript/canonicalize.ts`
- `packages/core/src/analyze/readingOrder.ts`
- `packages/core/src/utils/sort.ts`
- `packages/core/src/locators/roiSheetLocator.ts`
- `tests/transcript/determinism.test.ts`
- `packages/core/src/layout/types.ts`
- `packages/core/src/layout/load.ts`
- `layouts/layout-template.json`
- `packages/core/src/locators/compositeLocator.ts`
- `packages/core/src/locators/legacyTitleblockLocator.ts`
- `packages/core/src/utils/pdf.ts`
- `packages/core/src/workflows/merge/mergeWorkflow.ts`
- `packages/cli/src/commands/mergeAddenda.ts`
- `packages/core/src/transcript/profiles/types.ts`
- `packages/core/src/transcript/profiles/registry.ts`
- `packages/core/src/transcript/abstraction/abstractTranscript.ts`
- `packages/core/src/transcript/abstraction/tokenVault.ts`
- `packages/core/src/transcript/abstraction/sanitize.ts`
- `packages/core/src/transcript/abstraction/shapeFeatures.ts`
- `packages/core/src/transcript/abstraction/repetitionMetrics.ts`
- `packages/core/src/transcript/abstraction/lineGrouping.ts`
- `packages/core/src/transcript/ml/promptBuilder.ts`
- `packages/core/src/transcript/ml/types.ts`
- `packages/core/src/transcript/ml/apiCompiler.ts`
- `tests/transcript/abstraction/abstractTranscript.test.ts`
- `tests/transcript/ml/promptBuilder.test.ts`
- `tests/transcript/ml/apiCompiler.test.ts`
- `packages/core/src/utils/pdfLibBookmarkWriter.ts`
- `packages/core/src/bookmarks/pikepdfBookmarkWriter.ts`
- `packages/core/src/core/applyPlan.ts`
- `packages/core/src/transcript/factory.ts`
- `packages/core/src/analyze/documentContext.ts`
- `packages/core/src/locators/legacyTitleblockLocator.ts`
- `packages/core/src/config/featureFlags.ts`
- `packages/core/src/workflows/specs-patch/specsPatchWorkflow.ts`
- `packages/cli/src/cli.ts`
- `packages/cli/src/commands/specsPatch.ts`
- `packages/core/src/narrative/parse-algorithmic.ts`
- `packages/core/scripts/copy-sidecars.mjs`
- `packages/core/package.json`

Supporting references:

- `docs/TRANSCRIPT_ARCHITECTURE.md`
- `docs/ARCHITECTURE.md`
- `docs/MIGRATION_V3.md`
- `docs/largeFileRefactorPlan.md`
- `docs/MASTER_PLAN_v4.md`

## Source-of-Truth Notes

Applied source-of-truth rule: executable code was treated as canonical over planning docs.

Logged this phase:

- `SOT-004` clarifies that fully disk-streaming merge via sidecar is planned but not the current executable merge path.
- `SOT-005` clarifies that workflow staging is implemented, but correction application still re-runs analysis and merge plan reuse is limited.
- `SOT-006` clarifies that absolute determinism language must be read with extractor-path context in the prototype: PyMuPDF is strongly deterministic, PDF.js fallback is tolerated with weaker strictness.
- `SOT-007` clarifies that auto-layout/profile suggestion wording in CLI/docs overstates implementation: profile-driven ROI is active, but auto-layout options are not currently wired through merge command execution.
- `SOT-008` clarifies that privacy claims must be mode-aware: strict placeholder-first flow is implemented, but `FULL_TEXT_OPT_IN` and caller-supplied context can intentionally bypass strict no-literal-text behavior.
- `SOT-009` clarifies that "abandoned" status language is mixed in implementation: several items are superseded but still active in partial runtime paths, and some command-level abandonment labels do not match current CLI registration.

## Open Questions and Carry-Forward Risks

Added this phase:

- `R-006` merge sidecar maturity gap between planning docs and implementation
- `R-007` workflow state reuse gap between architecture intent and implementation
- `R-008` extractor-path determinism gap between strict architectural promise and tolerated PDF.js fallback variance
- `R-009` auto-layout capability drift between CLI/docs claims and executable merge/profile-selection behavior
- `R-010` privacy-mode policy risk: strict defaults exist, but full-text opt-in or unsafe caller context can widen exposure if not explicitly governed
- `R-011` technology-lifecycle labeling risk: docs may overstate "abandoned"/"replaced" status, causing migration assumptions that skip still-active legacy or partial paths

Previously open Phase 2 risks remain active and relevant to later ADRs.

## Next Step Readiness

Phase 3 ADR set is now complete (ADR-001 through ADR-008). Next phase can start at Phase 4 contracts using these ADRs as architecture baseline.