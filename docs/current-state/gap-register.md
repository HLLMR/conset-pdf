# Gap Register

**Version:** 1.0.0
**Date:** March 22, 2026
**Owner:** HLLMR LLC
**Status:** ACTIVE
**Doc Status Tag:** Implemented

## Scope

Enumeration of all known code-level gaps: stubs, no-ops, missing wiring, and trivial tests. Each entry is compared against its canonical doc claim. This register is the primary output of Phase E and the input to Phase F reconciliation.

Gap status:
- `Open` — known gap, not yet closed by implementation evidence
- `Accepted` — deliberate scope deferral with documented rationale
- `Closed` — closed by implementation evidence (reference to test or commit)

---

## Gap Inventory

| Gap ID | Crate | File | Item | Gap Type | Severity | Canonical Claim | Status |
|---|---|---|---|---|---|---|---|
| G-001 | engine | `crates/engine/src/extractor.rs` | `Extractor::extract()` | PARTIAL | CRITICAL | `ARCHITECTURE_v4_2.md` — Compiler pipeline: Lexer stage converts raw PDF input to LayoutTranscript | Open |
| G-002 | engine | `crates/engine/src/processor.rs` | `Processor::process()` | NOOP | CRITICAL | `ARCHITECTURE_v4_2.md` — Parser/Optimizer stages normalize, validate, and enrich transcript | Open |
| G-003 | engine | `crates/engine/src/main.rs` | `fn main()` | STUB | HIGH | `ARCHITECTURE_v4_2.md` — CLI orchestrates full extraction/processing pipeline | Open |
| G-004 | pdf-extraction | `crates/pdf-extraction/src/extractor.rs` | `PdfiumExtractor::extract_page()` | STUB | CRITICAL | `TRANSCRIPT_ARCHITECTURE_v4_2.md` — Extraction backend produces structured PageData with bboxes and text runs | Open |
| G-005 | pdf-extraction / engine | (boundary) | PDF text → IR span conversion | MISSING-WIRING | CRITICAL | `ARCHITECTURE_v4_2.md` — Pipeline connects pdf-extraction output to IR crate types | Open |
| G-006 | audit | `crates/audit/src/` | Audit hook integration | MISSING-WIRING | MEDIUM | `ARCHITECTURE_v4_2.md` — Audit events generated during extraction and processing | Open |
| G-007 | ir | `crates/ir/src/validation.rs` | `Validator::validate()` | NOOP | MEDIUM | `DEV_STANDARDS_v4_2.md` — Validation layer enforces all IR invariants at the crate boundary | Open |
| G-008 | ir | `crates/ir/src/types.rs` | `Document`, `Element` structs | STUB | LOW | `ARCHITECTURE_v4_2.md` — Document type models structured document content; purpose currently unclear | Open |
| G-009 | engine | `crates/engine/tests/end_to_end_test.rs` | E2E test pipeline invocation | TRIVIAL-TEST | HIGH | `MASTER_PLAN_v4.md` — End-to-end pipeline test validates real PDF → LayoutTranscript path | Open |
| G-010 | audit | `crates/audit/src/bundle.rs` | Audit unit tests | TRIVIAL-TEST | LOW | `DEV_STANDARDS_v4_2.md` — Tests validate behavior including JSON persistence and event ordering | Open |

---

## Severity Definitions

| Severity | Meaning |
|---|---|
| CRITICAL | Blocks any real end-to-end extraction; the pipeline cannot produce output without this |
| HIGH | Required for Phase 0 completion per `MASTER_PLAN_v4.md` acceptance criteria |
| MEDIUM | Required for correctness or auditability guarantees stated in canonical docs |
| LOW | Maintenance debt or minor capability gap; does not block pipeline |

---

## Gap Type Definitions

| Type | Meaning |
|---|---|
| STUB | Function exists; returns a dummy/empty/hardcoded value |
| NOOP | Function exists; body is an identity function or always-Ok return |
| MISSING-WIRING | Both source and target components exist; integration layer is absent |
| PARTIAL | Function is partially implemented with placeholder comments |
| TRIVIAL-TEST | Test exists but validates only trivial construction, not behavior |

---

## Critical Path

The minimum gap closure sequence for a functional end-to-end pipeline:

```
G-004 (extract_page stub)
  → G-005 (PDF text → IR conversion wiring)
    → G-001 (Extractor::extract() real implementation)
      → G-002 (Processor::process() normalization)
        → G-003 (main.rs CLI orchestration)
          → G-009 (E2E test with real pipeline invocation)
```

G-006 (audit hooks), G-007 (Validator), G-008 (Document/Element), and G-010 (audit tests) are parallel work, not on the critical path.

---

## Revision History

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-03-22 | Initial gap enumeration from Phase E code audit |
