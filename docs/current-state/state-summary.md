# Current State Summary

**Version:** 1.7.0  
**Date:** April 4, 2026  
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

## Next Focus

- Phase 0.5 Phase F: Render deterministic overlay PNGs (green/yellow/red bbox by confidence band) and finalize per-page sidecar provenance fields.
- Phase 0.5 Phase G: Schema-complete sidecars for `title-block-anchor`, `roi-candidate`, `spec-heading` (source=schema-only).
- Phase 0.5 Phase H: Draft downstream contracts (NormalizedIntakeBundle, ROI sidecar, schedule schema, correction manifest, etc.).
- Phase 0.5 Phase I: `validate-corpus` real detection loop over Tier 1/Tier 2 with determinism checks and aggregate manifest.
- Close G-002: implement `Processor::process()` with coordinate normalization and validate_transcript call.
- Close G-003: implement CLI argument parsing in `main.rs` to orchestrate the full pipeline.
- Close G-005: wire PDF extraction output to IR span conversion in shared engine.
- Close G-009: replace manual transcript construction in end-to-end test with real PDF fixture invocation.

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
