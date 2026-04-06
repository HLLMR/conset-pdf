# Current State Summary

**Version:** 2.4.0
**Date:** April 5, 2026
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
- Phase 0.5 Phase F: Deterministic overlay PNGs (green/amber/red by confidence band) and per-page sidecar provenance fields complete. Schema version `0.5.0` locked.
- Phase 0.5 Phase G: Schema-complete sidecars for `title-block-anchor` (with partial runtime corner detector), `roi-candidate`, `spec-heading` (source=schema-only). 16/16 unit tests.
- Phase 0.5 Phase H: Six contract modules in `crates/contracts/src/` defining all downstream contract shapes. 24/24 unit tests. No runtime wiring — contract-only by design.
- Phase 0.5 Phase I: `validate-corpus` real detection loop implemented. Two confirmed runs: 27 Tier 1 fixtures, 2,892 pages, 0 errors, 0 determinism regressions. Artifacts in `audit_output/phase-i-smoke/`.
- Phase 0.5 Phase J: 32 unit tests + 5 integration tests added for `fnv1a_64`, `FamilyPageCounts`, `score_matches`, artifact naming, manifest schema, dry-run semantics, holdout rejection, golden-path sidecar, and byte-determinism. All passing; clippy clean.
- Phase 0.5 Phase K: Downstream handoff document published at `docs/PHASE_05_HANDOFF.md`. Covers `pattern-dev` workflow, locked sidecar schema, extended evidence payloads, all six contract boundaries, runtime-ready vs. contract-only surface map, and priority band assignments for gaps G-001–G-039.
- Phase 0.5 Phase L: Final verification complete. Definition-of-done confirmed: both binaries build and clippy-clean; workspace-wide `cargo test` exits 0; `classify-pdf` and `pattern-dev` smoke-tested on representative Tier 1 fixtures; PASS/FAIL sidecars carry all required v0.5.0 fields; schema-only families emit `source=schema-only`; `validate-corpus --dry-run` lists 27 fixtures and writes no files; all contract modules carry `**CONTRACT-ONLY**` markers. **Phase 0.5 is CLOSED.**
- Phase 1 Band 0: G-001, G-002, G-005, G-007, G-009 all closed. Real PDFium extraction wired in `crates/engine/src/pipeline/extraction.rs` (`SpanData`→`BoundingBox`→`normalize_bbox()`→`BBox`→`Span`); `validate_transcript()` called in `pipeline/parsing.rs`; `Validator::validate()` delegates to `validate_transcript()`; extract handler writes transcript JSON to `--output`; `visualize` subcommand added to `backend-cli` (`crates/engine/src/visualize.rs`, `apps/backend-cli/src/handlers/visualize.rs`); E2E + API tests enabled and passing (9/9).
- Phase 1 complete: 8/8 CLI integration tests pass across SPEC, DWG, NAR, SUB, and simple fixtures; coordinate system verified (no axis inversion); `source_path` canonicalized to absolute path at extraction time. Evidence: `apps/backend-cli/tests/cli_integration_test.rs`.
- Phase 2 complete: `SegmentIndex` IR types (`crates/ir/src/segment.rs`), CSI footer-oracle segmentation engine (`crates/engine/src/segment.rs`), `Segment` and `VisualizeSegments` operations wired in `backend-cli`; 5/5 Phase 2 integration tests pass (segment simple, segment spec, segment dry-run, visualize-segments round-trip, visualize-segments dry-run). Evidence: `apps/backend-cli/tests/cli_integration_test.rs`.
- Phase 3 complete: `ParsedDocument` AST IR types (`crates/ir/src/ast.rs`); line-clustering + CSI 3-part outline tree parser (`crates/engine/src/parse.rs`); HTML AST visualizer (`crates/engine/src/visualize_ast.rs`); `Parse` and `VisualizeAst` operations wired in `backend-cli`; 6/6 Phase 3 integration tests pass (parse dry-run, visualize-ast dry-run, parse simple, parse spec, parse spec with section filter, parse+visualize-ast round-trip). Evidence: `apps/backend-cli/tests/cli_integration_test.rs`.
- Phase 3 parser hardening sprint complete (April 5, 2026): span x-sort, LINE_Y_EPSILON 0.005→0.012, cluster-based section ID detection (16→89 sections), FOOTER_Y 0.85→0.90, noise-only line skipping, article_re major≥1 + uppercase-title guard, `inject_missing_parts` recovery pass. Post-hardening: 7,971 nodes, 0.2% unclassified, 0/70 wrong-PART sections, 19/19 integration tests passing. See `CHANGELOG.md` for full detail.

## Next Focus

- **Phase 3 hardening is COMPLETE. Phase 3 is fully CLOSED.**
- **Phase 4 entry points (Band 3):** Edit operations — `SectionEditor` implementation with insert/delete/replace on `ParsedDocument` AST nodes, paragraph renumbering (cascading), CLI `edit` subcommand, new `WorkflowOperation::Edit` contract variant. See `docs/MASTER_PLAN.md` Phase 4 deliverables.

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
