# Current State Summary

**Version:** 1.6.0  
**Date:** March 23, 2026  
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

## Next Focus

- Close G-001/G-004/G-005 critical chain: implement `PdfiumExtractor::extract_page()`, wire PDF text→IR span conversion, and implement `Extractor::extract()` to produce a real LayoutTranscript from a PDF file.
- Close G-002: implement `Processor::process()` with coordinate normalization and validate_transcript call.
- Close G-003: implement CLI argument parsing in `main.rs` to orchestrate the full pipeline.
- Close G-009: replace manual transcript construction in end-to-end test with real PDF fixture invocation.
- Start autonomous deterministic ROI workstream: implement candidate generation, scoring, and audit diagnostics as first-class detection policy.
- Keep manual ROI/profile tooling constrained to admin-only refinement and controlled fallback UX in Phase 0.5 surfaces.
- Continue Phase D Extension: mine remaining algorithm contracts (ID parsing thresholds, merge planning modes, ROI detection rules) into an `AEC_ALGORITHMS.md` canonical doc.
- Implement Intake Triage Stage 0: close G-013 (`IntakeBundle` contract) then G-014 (batch addenda assembly) and G-016 (rotation detection + `lopdf` normalization) before any multi-file workflow can proceed.
- Keep autonomous document-type classification (G-015) deferred until furniture patterns are stable (Phase 2); define advisory classification manifest schema during Phase 0.5 planning.
- Start vector-first drawing detection track: close G-018/G-019 (frame + corner-band + table-score title-block localization) and G-020 (sheet field scoring) with deterministic diagnostics.
- Define and stage the template lifecycle contract (G-021): unconfirmed/confirmed/deprecated states, drift checks, and explicit review thresholds.
- Keep AI-enhanced fallback optional-only (G-023): explicit user opt-in, low-confidence gate, and cropped-region payload manifest.
- Add local micro-ML confidence assist track: implement G-024 runtime and G-025 deterministic confidence fusion with model/version provenance.
- Add power-user LLM API track: define G-026 validation/instruction contracts with advisory-by-default and explicit promotion semantics.
- Add raster OCR track: implement G-027 low-text/raster routing with source tagging and confidence-aware parsing.
- Add schedule extraction contract track: implement G-028 canonical table schema and G-029 schema-versioned JSON/CSV/XML export adapters.
- Add replay and automation track: implement G-030 replayable correction manifests and G-035 typed instruction DSL with dry-run validation.
- Add operational trust layer: implement G-031 native diff/compare reports, G-032 provenance-first review objects, and G-039 exception triage/drift metrics contracts.
- Add production operations layer: implement G-033 privacy/redaction controls, G-034 resumable orchestration, and G-036 confidence policy profiles.
- Extend the existing standards scaffold into runtime normalization: implement G-037 against `AEC_STANDARDS.md`, UDS/NCS, and MasterFormat references rather than new parallel mappings.
- Add project intelligence layer: implement G-038 cross-document entity resolution feeding searchable project knowledge surfaces.

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
