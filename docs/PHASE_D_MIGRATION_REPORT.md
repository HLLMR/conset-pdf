# Phase D Migration Report

**Version:** 1.0.0
**Date:** March 22, 2026
**Owner:** HLLMR LLC
**Doc Status Tag:** Implemented

## Purpose

This report is the authoritative closure artifact for Phase D: Selective Postmortem Migration. It traces every insight considered during Phase D from source file through disposition decision to canonical destination section. It supersedes any intermediate notes in `current-state/migration-intake.md` for closure-evidence purposes.

---

## Summary Counts

| Disposition | Count |
|---|---|
| Migrated (verbatim or near-verbatim) | 5 |
| Adapted (v4-reframed) | 6 |
| Rejected (stack-specific or irrelevant) | 3 |
| Archive-only (retained without promotion) | 1 |
| **Total intake items** | **15** |

Section-level closure maps: **10** (SLM-001 through SLM-010), all `Closed (2026-03-22)`.

---

## Phase D Migration Source Index

> Sources are all relative to `docs/`.

| Source Bucket | Files Mined |
|---|---|
| `prototype-postmortem/06-lessons/` | `phase-06-lessons-learned.md`, `phase-06-non-negotiables-rust-constraints.md` |
| `prototype-postmortem/04-contracts/` | `phase-04-workflow-types-contract.md` |
| `prototype-postmortem/05-gaps-and-limitations/` | `phase-05-failure-modes-catalog.md` |
| `prototype-postmortem/07-rust-handoff/` | `phase-07-rust-port-primer.md` |
| `prototype-postmortem/09-ops-and-drift/` | `phase-09-ops-telemetry-lessons.md` |
| Retired v3 Phase C merge-target set | `CLI_REFERENCE.md`, `CODEBASE_OVERVIEW.md`, `CORE_API.md`, `GETTING_STARTED.md`, `LOCATORS.md`, `MODULES.md`, `TRANSCRIPT_SYSTEM.md`, `WORKFLOWS.md` |

---

## Detailed Migration Records

### M-001 — Deterministic Transcript Ordering and Normalization Constraints

| Field | Value |
|---|---|
| **Item ID** | M-001 |
| **Source** | `prototype-postmortem/06-lessons/phase-06-lessons-learned.md` |
| **Migration Type** | Migrated |
| **Destination Section** | `DEV_STANDARDS_v4_2.md` → `## Phase D Imported Constraints` / `### Determinism Guarantees` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Transcript ordering and normalization must be fully deterministic, with identical inputs producing byte-identical outputs. Span sort order (y, x), coordinate normalization, and hash guarantees must all be enforced at the IR boundary.

**Rationale:** Correctness constraint — implementation-agnostic. Applies directly to the v4 IR crate validation layer.

---

### M-002 — Workflow State and Ordering Contracts

| Field | Value |
|---|---|
| **Item ID** | M-002 |
| **Source** | `prototype-postmortem/04-contracts/phase-04-workflow-types-contract.md` |
| **Migration Type** | Adapted |
| **Destination Section** | `ARCHITECTURE_v4_2.md` → `## Phase D Imported Architecture Constraints` / `### Workflow State Invariants` and `### MergePlan Ordering Contract` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Workflow execution must follow the canonical `analyze → applyCorrections → execute` gate sequence. MergePlan ordering contracts enforce span-level sequencing invariants.

**Rationale:** Adapted to remove TypeScript-specific typing; the state-ordering invariant is directly applicable to v4 pipeline design.

---

### M-003 — Session Lifecycle Logging and Gate Semantics

| Field | Value |
|---|---|
| **Item ID** | M-003 |
| **Source** | `prototype-postmortem/09-ops-and-drift/phase-09-ops-telemetry-lessons.md` |
| **Migration Type** | Migrated |
| **Destination Sections** | `DEV_STANDARDS_v4_2.md` → `### Session and Operation Logging`; `DOC_GOVERNANCE.md` → `## Deprecation and Feature Gates` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Sessions must log start/end events with operation counts; deprecation gates must be explicit and never silent. Feature disablement must log a reason code.

**Rationale:** High-value operational guardrails for audit trail completeness in the v4 audit crate.

---

### M-004 — TypeScript Implementation Details

| Field | Value |
|---|---|
| **Item ID** | M-004 |
| **Source** | `prototype-live/core/` |
| **Migration Type** | Rejected |
| **Destination** | N/A |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** None — TypeScript runtime, package topology, and npm/Jest implementation details.

**Rationale:** Stack-specific to the deprecated v3 runtime. Not applicable to Rust-first v4 architecture.

---

### M-005 — Rust Handoff Primers and Mapping Notes

| Field | Value |
|---|---|
| **Item ID** | M-005 |
| **Source** | `prototype-postmortem/07-rust-handoff/phase-07-rust-port-primer.md` |
| **Migration Type** | Archive-only |
| **Destination** | `prototype-postmortem/` (retained as historical context) |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Transition narrative mapping TypeScript concepts to Rust idioms.

**Rationale:** Useful historical context for understanding original design intent, but no durable architectural constraint extractable without re-derivation from code.

---

### M-006 — Known Workflow Gaps for Footer Mapping and Schedule Edge Cases

| Field | Value |
|---|---|
| **Item ID** | M-006 |
| **Source** | `prototype-postmortem/05-gaps-and-limitations/phase-05-failure-modes-catalog.md` |
| **Migration Type** | Adapted |
| **Destination Section** | `MASTER_PLAN_v4.md` → `## Known Workflow Gaps (Imported From Postmortem)` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Footer mapping and multi-column schedule edge cases were unresolved in v3 and remain open planning items.

**Rationale:** Important planning constraints carried forward until closed by implementation evidence in v4.

---

### M-007 — Medium-Specific Chrome Detection and Threshold Policy

| Field | Value |
|---|---|
| **Item ID** | M-007 |
| **Source** | `prototype-postmortem/06-lessons/phase-06-non-negotiables-rust-constraints.md` |
| **Migration Type** | Adapted |
| **Destination Sections** | `ARCHITECTURE_v4_2.md` → `### Medium-Specific Chrome Detectors`; `MASTER_PLAN_v4.md` → `### Chrome Detection Threshold Policy` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Chrome detection thresholds must be per-medium and runtime-configurable. Hardcoded prototype constants must not survive into v4.

**Rationale:** Preserves durable architecture intent while removing implementation-specific threshold values.

---

### M-008 — Operation Families and Analyze-First Usage Pattern

| Field | Value |
|---|---|
| **Item ID** | M-008 |
| **Source** | Retired v3 CLI/onboarding bundle (Phase C merge-target set) |
| **Migration Type** | Migrated |
| **Destination Sections** | `ARCHITECTURE_v4_2.md` → `### Baseline Operation Surface (Imported)`; `SETUP.md` → `## Quick Verification Commands (Imported)` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Six durable operation families (analyze, extract, validate, apply-corrections, export, audit) and the analyze-first workflow obligation.

**Rationale:** Preserved operation intent without carrying legacy CLI flag names or stack details.

---

### M-009 — Backend Quality-Tier and Fallback Guardrail Semantics

| Field | Value |
|---|---|
| **Item ID** | M-009 |
| **Source** | Retired v3 transcript bundle (Phase C merge-target set) |
| **Migration Type** | Migrated |
| **Destination Section** | `TRANSCRIPT_ARCHITECTURE_v4_2.md` → `## Backend Quality Tiers (Imported)` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Four backend quality tiers with explicit fallback and reject semantics.

**Rationale:** Converted from v3 narrative into a canonical quality-tier policy applicable to the v4 extraction subsystem.

---

### M-010 — v3 Module Layout Details

| Field | Value |
|---|---|
| **Item ID** | M-010 |
| **Source** | Retired v3 overview/module bundle (Phase C merge-target set) |
| **Migration Type** | Rejected |
| **Destination** | N/A |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** None — v3 TypeScript package topology (`packages/core`, `packages/cli`).

**Rationale:** Prototype package topology is not authoritative for Rust crate architecture.

---

### M-011 — v3 TypeScript API Signatures

| Field | Value |
|---|---|
| **Item ID** | M-011 |
| **Source** | Retired v3 API surface bundle (Phase C merge-target set) |
| **Migration Type** | Rejected |
| **Destination** | N/A |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** None — TypeScript method and interface signatures.

**Rationale:** Stack-specific. Not a valid canonical contract for the Rust v4 API surface.

---

### M-012 — Three-Phase Workflow Pattern Details

| Field | Value |
|---|---|
| **Item ID** | M-012 |
| **Source** | Retired v3 workflow bundle (Phase C merge-target set) |
| **Migration Type** | Adapted |
| **Destination Section** | `ARCHITECTURE_v4_2.md` → `## Phase D Imported Architecture Constraints` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Three-phase execution invariant: the parse phase must complete and be validated before the transform phase begins; both must complete before the emit phase.

**Rationale:** Invariant is implementation-agnostic. Preserved semantics, removed TypeScript-specific typing details.

---

## Section-Level Closure Map (Summary)

| Map ID | Source Insight | Canonical Destination Section | Status |
|---|---|---|---|
| SLM-001 | Determinism ordering, normalization, and hash constraints | `DEV_STANDARDS_v4_2.md` → `### Determinism Guarantees` | Closed (2026-03-22) |
| SLM-002 | Session lifecycle logging and deprecation gate semantics | `DEV_STANDARDS_v4_2.md` → `### Session and Operation Logging`; `DOC_GOVERNANCE.md` → `## Deprecation and Feature Gates` | Closed (2026-03-22) |
| SLM-003 | Workflow state invariants and plan ordering contract | `ARCHITECTURE_v4_2.md` → `### Workflow State Invariants` and `### MergePlan Ordering Contract` | Closed (2026-03-22) |
| SLM-004 | Medium-specific chrome detector policy | `ARCHITECTURE_v4_2.md` → `### Medium-Specific Chrome Detectors` | Closed (2026-03-22) |
| SLM-005 | Transcript backend quality-tier fallback rules | `TRANSCRIPT_ARCHITECTURE_v4_2.md` → `## Backend Quality Tiers (Imported)` | Closed (2026-03-22) |
| SLM-006 | Known workflow gaps and threshold tunability constraints | `MASTER_PLAN_v4.md` → `## Known Workflow Gaps (Imported From Postmortem)` and `### Chrome Detection Threshold Policy` | Closed (2026-03-22) |
| SLM-007 | Operation-family expectations and analyze-first usage | `ARCHITECTURE_v4_2.md` → `### Baseline Operation Surface (Imported)`; `SETUP.md` → `## Quick Verification Commands (Imported)` | Closed (2026-03-22) |

---

## Phase D Extension Records (d-ext)

The following items were added during the Phase D Extension cycle after mining `03-adrs/` and `02-algorithms/` in full.

### M-013 — Transcript-First Extraction Contract

| Field | Value |
|---|---|
| **Item ID** | M-013 |
| **Source** | `prototype-postmortem/03-adrs/phase-03-adr-002-transcript-first.md` |
| **Migration Type** | Migrated |
| **Destination Section** | `ARCHITECTURE_v4_2.md` → `## Phase D Imported Architecture Constraints` / `### Transcript-First Extraction Contract` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Extract once per document invocation, canonicalize at the extraction boundary immediately, cache and serve downstream. Transcript contract must be stable across backend changes. Backend failure triggers explicit controlled fallback, not silent degradation. Backend geometric fidelity differences are first-class architectural properties.

**Rationale:** Core extraction-model invariant — backend-agnostic and directly applicable to the v4 engine pipeline.

---

### M-014 — Profile-Driven Detection Architecture

| Field | Value |
|---|---|
| **Item ID** | M-014 |
| **Source** | `prototype-postmortem/03-adrs/phase-03-adr-006-profile-driven-detection.md` |
| **Migration Type** | Adapted |
| **Destination Section** | `ARCHITECTURE_v4_2.md` → `### Profile-Driven Detection Architecture` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Profile-first detection strategy with ordered ROI fallbacks, strict profile validation at load time, per-ROI failure codes, and feature-flagged heuristic fallback.

**Rationale:** Detection architecture invariant; adapted to remove prototype threshold constants while preserving the profile-first strategy and explicit diagnostic requirements.

---

### M-015 — Transcript Quality Gate Thresholds

| Field | Value |
|---|---|
| **Item ID** | M-015 |
| **Source** | `prototype-postmortem/02-algorithms/phase-02-quality-scoring.md` |
| **Migration Type** | Migrated |
| **Destination Section** | `TRANSCRIPT_ARCHITECTURE_v4_2.md` → `## Quality Gate Thresholds (Imported)` |
| **Closure Date** | 2026-03-22 |

**Insight Extracted:** Four acceptance gates: text presence (>= 50 chars/page), encoding integrity (U+FFFD ratio <= 0.05), ordering sanity (>= 0.80 agreement), and aggregate confidence (>= 0.85). Per-gate diagnostic reporting required. Quality-driven fallback is an architectural requirement.

**Rationale:** Hard quality gates are architectural constraints — directly constrain the v4 extraction quality scoring system.

---

## Section-Level Closure Map (d-ext Additions)

| Map ID | Source Insight | Canonical Destination Section | Status |
|---|---|---|---|
| SLM-008 | Extract-once, canonicalize-immediately contract | `ARCHITECTURE_v4_2.md` → `### Transcript-First Extraction Contract` | Closed (2026-03-22) |
| SLM-009 | Profile-first detection, ordered ROI fallbacks, feature-flagged fallback | `ARCHITECTURE_v4_2.md` → `### Profile-Driven Detection Architecture` | Closed (2026-03-22) |
| SLM-010 | Quality gate thresholds (text presence, encoding, ordering, confidence) | `TRANSCRIPT_ARCHITECTURE_v4_2.md` → `## Quality Gate Thresholds (Imported)` | Closed (2026-03-22) |

---

## Unexhausted Sources for Phase D Extension

The following postmortem sections were identified but not mined in this Phase D pass. These are candidates for a Phase D Extension cycle (d-ext):

| Source Path | Content Type | Recommended Action |
|---|---|---|
| `prototype-postmortem/02-algorithms/` | Transcript canonicalization, ROI detection, merge planning algorithms | Partially mined (M-015). Remaining: ID parsing confidence thresholds, merge planning modes, narrative parser rules, ROI detection algorithm, schedule extraction, specs chrome removal, standards normalization. Promote to an `AEC_ALGORITHMS.md` canonical doc when volume warrants. |
| `prototype-postmortem/03-adrs/ADR-001` | Backend boundary isolation (sidecar pattern) | Durable: backends must accept serializable I/O and fail loudly. Consider adding to `ARCHITECTURE_v4_2.md` as `### Backend Adapter Boundary Contract`. |
| `prototype-postmortem/03-adrs/ADR-005` | Determinism enforcement ADR | Already substantially covered by SLM-001 and M-013; review for residual precision/quantization detail not yet captured. |
| `prototype-postmortem/03-adrs/ADR-007` | Privacy-preserving ML abstraction (TokenVault) | Future-phase relevance; deferred until ML-assisted profile generation is on the roadmap. |

---

## Phase D Completion Statement

Phase D is **complete** for the current intake scope (M-001 through M-015, SLM-001 through SLM-010, including the d-ext cycle). All migrated and adapted items are reflected in the canonical docs listed above. Rejected items are documented with rationale. The unexhausted sources above constitute the scope for a future Phase D continuation cycle.

---

## Revision History

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-03-22 | Initial Phase D closure report |
| 1.1.0 | 2026-03-22 | Added Phase D Extension records (M-013 – M-015, SLM-008 – SLM-010) from ADR-002, ADR-006, and quality-scoring algorithm mining |
