# Capability Matrix

**Version:** 1.0.0  
**Date:** March 22, 2026  
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
| `crates/pdf-extraction` extraction baseline | **Partial** — PdfExtractor trait, load_document, get_page_count, extract_text (real API calls) implemented and tested; `extract_page()` returns empty PageData | Structured bbox/text-run extraction; IR type conversion | ../crates/pdf-extraction/src/, ../crates/pdf-extraction/tests/ | G-004 (extract_page stub), G-005 (no PDF→IR conversion) |
| `crates/engine` processing baseline | **Stub** — Extractor creates dummy single-page transcript; Processor is identity function; main.rs prints version only | Real extraction pipeline; CLI orchestration; medium-specific processing | ../crates/engine/src/, ../crates/engine/tests/ | G-001, G-002, G-003, G-009 |
| `crates/audit` bundle/event writer baseline | **Mostly implemented** — AuditEvent, AuditBundle, event types, JSON writer/reader all present; no integration hooks in pipeline | Event hooks in extractor/processor; stronger event ordering/persistence tests | ../crates/audit/src/ | G-006 (no pipeline hooks), G-010 (trivial tests) |

## Workflow-Level Matrix (Canonical Intent)

| Workflow | Current State | Next Milestone | Source of Truth | Open Gaps |
|---|---|---|---|---|
| Deterministic extraction pipeline | **Not functional** — no real PDF→IR path exists | Implement G-004 → G-005 → G-001 critical chain | ../MASTER_PLAN.md | G-001, G-004, G-005 |
| Validation and invariants | **Partial** — IR validation framework present but engine doesn't call it; `Validator::validate()` is no-op | Wire engine Processor to call validate_transcript; implement Validator | ../TRANSCRIPT_ARCHITECTURE.md | G-002, G-007 |
| Auditability outputs | **Not integrated** — audit crate fully implemented in isolation; not called during extraction or processing | Hook AuditBundle into Extractor::extract() and Processor::process() | ../ARCHITECTURE.md | G-006 |
| Documentation governance | **Implemented** — authority order, tags, triage matrix all active | Phase E/F gap closure; Phase D-ext continuation | ../DOC_GOVERNANCE.md | None |

## Notes

- Mark capability as implemented only when code plus tests exist.
- Planned entries must map to canonical docs, not archival references.
- Re-check this table at each phase boundary and update evidence paths if code moves.
- Open gap IDs (G-NNN) reference `../current-state/gap-register.md` for full detail and criticality classification.
