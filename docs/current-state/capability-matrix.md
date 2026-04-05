# Capability Matrix

**Version:** 1.6.0  
**Date:** April 5, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

## Scope

Implemented vs planned capabilities by crate and workflow.

Classification rule:

- `Implemented` requires code presence plus test coverage evidence.
- `Planned` requires canonical-source intent in master or derived canonical docs.

## Crate Capability Matrix

| Area | Implemented | Planned | Evidence | Open Gaps |
|---|---|---|---|---|
| `crates/ir` data model and validation | **Partial** — types, BBox, Span, Page, LayoutTranscript, normalize_bbox, sort_spans implemented and tested; `Validator::validate()` is a no-op; `Document` and `Element` are empty stubs | Full invariant enforcement; complete Document/Element types | ../crates/ir/src/, ../crates/ir/tests/ (8 test files, comprehensive) | G-007 (Validator no-op), G-008 (Document/Element stubs) |
| `crates/pdf-extraction` extraction baseline | **Implemented** — `PdfiumExtractor::extract_page()` returns real `PageData` with `Vec<SpanData>` (text, font name, font size, `RawBBox`), `width_pts`, `height_pts`; `load_document`, `get_page_count`, `extract_text` all implemented and tested | PDF→IR span conversion wiring; Form XObject-transparent extraction in shared engine path | `crates/pdf-extraction/src/extractor.rs`, `crates/pdf-extraction/src/types.rs` | G-005 (PDF→IR conversion wiring in shared engine) |
| `crates/engine` processing baseline | **Stub** — Extractor creates dummy single-page transcript; Processor is identity function; main.rs prints version only | Real extraction pipeline; CLI orchestration; medium-specific processing | ../crates/engine/src/, ../crates/engine/tests/ | G-001, G-002, G-003, G-009 |
| `crates/audit` bundle/event writer baseline | **Mostly implemented** — AuditEvent, AuditBundle, event types, JSON writer/reader all present; no integration hooks in pipeline | Event hooks in extractor/processor; stronger event ordering/persistence tests | ../crates/audit/src/ | G-006 (no pipeline hooks), G-010 (trivial tests) |
| `crates/contracts` downstream contract surfaces | **Implemented** — 6 modules: `intake`, `ocr_routing`, `schedule`, `assisted_intelligence`, `operational_trust`, `knowledge`; 24/24 unit tests (round-trip serde for every major type); all types `#[derive(Debug, Clone, Serialize, Deserialize)]`; **CONTRACT-ONLY** by design; no runtime execution wired | Runtime implementation in Band 1–4 (see gap assignments) | `crates/contracts/src/` | Runtime: G-013–G-016 (intake), G-027 (OCR), G-028–G-029 (schedule), G-023–G-026 (AI), G-030–G-036 (trust), G-037–G-039 (knowledge) |

## Workflow-Level Matrix (Canonical Intent)

| Workflow | Current State | Next Milestone | Source of Truth | Open Gaps |
|---|---|---|---|---|
| Deterministic extraction pipeline | **Not functional** — no real PDF→IR path exists | Implement G-004 → G-005 → G-001 critical chain | ../MASTER_PLAN.md | G-001, G-004, G-005 |
| Validation and invariants | **Partial** — IR validation framework present but engine doesn't call it; `Validator::validate()` is no-op | Wire engine Processor to call validate_transcript; implement Validator | ../TRANSCRIPT_ARCHITECTURE.md | G-002, G-007 |
| Auditability outputs | **Not integrated** — audit crate fully implemented in isolation; not called during extraction or processing | Hook AuditBundle into Extractor::extract() and Processor::process() | ../ARCHITECTURE.md | G-006 |
| Autonomous ROI detection policy | **Not implemented** — no deterministic ROI candidate/ranking engine in runtime path | Implement deterministic candidate generation, ranking, diagnostics, and tie-break policy | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-011 |
| Admin-only manual ROI refinement | **Not implemented** — no admin override manifest or restricted operator flow yet | Add admin-only override/repair path without making manual profiles a baseline requirement | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-012 |
| Multi-file intake bundle assembly | **Not implemented** — `WorkflowRequest.input_path` is singular; no `IntakeBundle` or `IntakeIssue` types | Implement G-013 contract then G-014 batch addenda assembly | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-013, G-014 |
| Intake Triage: page audit and rotation normalization | **Not implemented** — no Stage 0 pre-Lexer pipeline; no rotation detection or `lopdf` write path | Implement Stage 0 Intake Triage: page audit, rotation detect/correct via `lopdf`, bundle manifest | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-016 |
| Autonomous document-type classification | **Not implemented** — no furniture-pattern-based page classification; classification manifest not defined | Implement G-015 after furniture patterns are stable (Phase 2-gated); advisory output only | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-015 |
| Cross-medium addenda merge | **Not implemented** — `merge_addenda.rs` is a stub; no multi-medium workflow | Implement G-017 after G-014 and Phase 6 stitch capability are complete | ../MASTER_PLAN.md | G-017 |
| Drawing title-block localization (vector-first) | **Not implemented** — no frame/corner-band candidate generator or table-structure scoring pass exists | Implement G-018 then G-019 for deterministic title-block ROI selection and diagnostics | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-018, G-019 |
| Drawing sheet field extraction (sheet number/title) | **Not implemented** — no deterministic intra-title-block field scorer currently wired | Implement G-020 with pattern + typography + relative-position scoring and tie-break evidence | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-020 |
| Auto-learned firm/layout templates | **Not implemented** — no internal template store, status model, or drift checks | Implement G-021 schema and replay/drift contract; keep profile concepts out of primary UX | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-021 |
| Spec heading segmentation (vector-first) | **Not implemented** — no deterministic heading detector based on line/typography features in parser path | Implement G-022 and integrate with section-boundary logic | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-022 |
| AI-enhanced fallback contract (optional) | **Not implemented** — no explicit opt-in gate or cropped-region payload manifest exists | Implement G-023 as optional fallback contract with audit events; non-baseline path | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-023 |
| Local micro-ML confidence assist | **Not implemented** — no on-device model runtime or version-pinned assist path integrated | Implement G-024 runtime path and G-025 deterministic fusion with audit provenance | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-024, G-025 |
| Power-user LLM validation/instruction API | **Not implemented** — no contract endpoints for ambiguity validation or instruction-set generation | Implement G-026 contracts and advisory-to-executable promotion guardrails | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-026 |
| Raster OCR extraction path | **Not implemented** — no low-text/raster routing and no `ocr` vs `vector` source tagging in transcript path | Implement G-027 OCR routing with confidence/provenance tags | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-027 |
| Schedule parser canonical schema | **Not implemented** — no table parser producing stable schema records with provenance | Implement G-028 canonical schedule schema and parser outputs | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-028 |
| Schedule export adapters (JSON/CSV/XML) | **Not implemented** — JSON/CSV/XML schedule export contracts and versioning are not wired | Implement G-029 export adapters with stable field mappings and schema version tags | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-029 |
| Replayable correction manifests | **Not implemented** — no typed correction manifest or deterministic replay/reapply path exists | Implement G-030 schema and replay engine with scope guards and divergence reporting | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-030 |
| Native diff/compare workflows | **Not implemented** — no structured spec/sheet/packet compare reports are produced | Implement G-031 compare workflows and integrate with review outputs | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-031 |
| Provenance-first review contract | **Not implemented** — no unified review object model links fields, decisions, and corrections back to evidence | Implement G-032 review contract with evidence refs and branch reasons | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-032 |
| Privacy/redaction outbound controls | **Not implemented** — no region-level redaction policy or outbound payload manifest for external API paths | Implement G-033 redaction policy, payload manifests, and privacy modes | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-033 |
| Batch orchestration and resumability | **Not implemented** — no job queue, retry, or resume semantics exist | Implement G-034 job/file/page state accounting and resumable orchestration | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-034 |
| Instruction DSL / manifest language | **Not implemented** — executable automation is not standardized on one typed manifest surface | Implement G-035 parser, validator, dry-run, and execution bridge | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-035 |
| Confidence policy profiles | **Not implemented** — no stable user-facing operational modes exist for review strictness and escalation behavior | Implement G-036 calibrated policy profiles and UI/contract surface | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-036 |
| Standards normalization wiring | **Not implemented** — existing canonical standards scaffold is documented but not yet wired into normalization runtime paths | Implement G-037 using existing UDS/NCS/MasterFormat references and raw-to-canonical mapping evidence | ../MASTER_PLAN.md, ../ARCHITECTURE.md, ../AEC_STANDARDS.md | G-037 |
| Cross-document entity resolution | **Not implemented** — no linked entity layer exists across sheets, sections, tags, firms, and revisions | Implement G-038 entity linking, evidence basis, and confidence model | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-038 |
| Project knowledge index and triage queue | **Not implemented** — no searchable knowledge surface, exception queue, or drift dashboard contract exists | Implement G-039 searchable index/triage contracts and metrics surface | ../MASTER_PLAN.md, ../ARCHITECTURE.md | G-039 |
| `tools` pattern-dev developer CLI | **Implemented** — all Phase 0.5 phases B–L complete; `inspect`, `test-pattern`, `validate-corpus` subcommands runtime-ready; `footer-section-id`/`page-counter`/`header-band` detection live; `title-block-anchor` partial runtime (corner scoring, Phase G bonus); `roi-candidate`/`spec-heading` schema-only; overlay PNGs at 1400 px; sidecar schema v0.5.0 locked; 32 unit + 5 integration tests passing; corpus validated: 27 Tier 1 fixtures, 2,892 pages, `det_regressions=0` | — | `tools/pattern_dev.rs`, `tools/src/pattern_model.rs`, `tools/tests/pattern_dev_integration.rs` | Band 1: G-011 (ROI ranking runtime), G-018–G-019 (title-block full scoring) |
| Documentation governance | **Implemented** — authority order, tags, triage matrix all active | Phase E/F gap closure; Phase D-ext continuation | ../DOC_GOVERNANCE.md | None |

## Notes

- Mark capability as implemented only when code plus tests exist.
- Planned entries must map to canonical docs, not archival references.
- Re-check this table at each phase boundary and update evidence paths if code moves.
- Open gap IDs (G-NNN) reference `../current-state/gap-register.md` for full detail and criticality classification.
