# Migration Intake

**Version:** 1.0.0  
**Date:** March 22, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

## Scope

Candidate insights from postmortem archives with disposition and destination.

| Item ID | Source | Insight Candidate | Disposition | Destination | Rationale |
|---|---|---|---|---|---|
| M-001 | ../prototype-postmortem/06-lessons/phase-06-lessons-learned.md | Deterministic transcript ordering and normalization constraints | Migrated (2026-03-22) | ../DEV_STANDARDS_v4_2.md (Phase D Imported Constraints) | Durable correctness constraints, implementation-agnostic. |
| M-002 | ../prototype-postmortem/04-contracts/phase-04-workflow-types-contract.md | Workflow state and ordering contracts (`analyze` -> `applyCorrections` -> `execute`) | Adapted (2026-03-22) | ../ARCHITECTURE_v4_2.md (Phase D Imported Architecture Constraints) | Needed v4 framing around state invariants and plan ordering. |
| M-003 | ../prototype-postmortem/09-ops-and-drift/phase-09-ops-telemetry-lessons.md | Session lifecycle logging and explicit gate semantics | Migrated (2026-03-22) | ../DEV_STANDARDS_v4_2.md and ../DOC_GOVERNANCE.md | High value operational guardrails for deterministic workflows. |
| M-004 | ../prototype-live/core/ | TypeScript implementation details tied to deprecated runtime stack | Rejected (2026-03-22) | N/A | Not applicable to Rust-first v4 architecture. |
| M-005 | ../prototype-postmortem/07-rust-handoff/phase-07-rust-port-primer.md | Rust handoff primers and mapping notes | Archive-only (2026-03-22) | ../prototype-postmortem/ | Retained as historical context without canonical promotion. |
| M-006 | ../prototype-postmortem/05-gaps-and-limitations/phase-05-failure-modes-catalog.md | Known workflow gap list for footer mapping and schedule edge cases | Adapted (2026-03-22) | ../MASTER_PLAN_v4.md (Known Workflow Gaps) | Important planning constraints until closed by implementation evidence. |
| M-007 | ../prototype-postmortem/06-lessons/phase-06-non-negotiables-rust-constraints.md | Medium-specific chrome detection and threshold policy | Adapted (2026-03-22) | ../ARCHITECTURE_v4_2.md and ../MASTER_PLAN_v4.md | Preserves durable architecture intent, avoids hardcoded prototype constants. |
| M-008 | Retired v3 CLI/onboarding bundle (Phase C merge-target set) | Durable operation families and analyze-first usage pattern | Migrated (2026-03-22) | ../ARCHITECTURE_v4_2.md and ../SETUP.md | Preserved operation intent without carrying legacy stack details. |
| M-009 | Retired v3 transcript bundle (Phase C merge-target set) | Backend quality-tier and fallback guardrail semantics | Migrated (2026-03-22) | ../TRANSCRIPT_ARCHITECTURE_v4_2.md | Converted to canonical quality-tier policy. |
| M-010 | Retired v3 overview/module bundle (Phase C merge-target set) | v3 module layout details | Rejected (2026-03-22) | N/A | Prototype package topology is not authoritative for Rust crate architecture. |
| M-011 | Retired v3 API surface bundle (Phase C merge-target set) | v3 TypeScript API signatures | Rejected (2026-03-22) | N/A | API signatures are stack-specific and not valid canonical Rust interface contract. |
| M-012 | Retired v3 workflow bundle (Phase C merge-target set) | Three-phase workflow pattern details | Adapted (2026-03-22) | ../ARCHITECTURE_v4_2.md | Preserved invariant, removed implementation-specific typing details. |
| M-013 | ../prototype-postmortem/03-adrs/phase-03-adr-002-transcript-first.md | Extract-once, canonicalize-immediately, serve-downstream contract | Migrated (2026-03-22) | ../ARCHITECTURE_v4_2.md (Phase D Ext — Transcript-First Extraction Contract) | Core extraction-model invariant; backend-agnostic and directly applicable to v4 engine. |
| M-014 | ../prototype-postmortem/03-adrs/phase-03-adr-006-profile-driven-detection.md | Profile-first detection with ordered ROI fallbacks and feature-flagged heuristic fallback | Adapted (2026-03-22) | ../ARCHITECTURE_v4_2.md (Phase D Ext — Profile-Driven Detection Architecture) | Detection architecture invariant; removes prototype threshold constants but preserves profile-first strategy. |
| M-015 | ../prototype-postmortem/02-algorithms/phase-02-quality-scoring.md | Transcript acceptance quality gates with explicit thresholds | Migrated (2026-03-22) | ../TRANSCRIPT_ARCHITECTURE_v4_2.md (Quality Gate Thresholds) | Hard quality gates are architectural — directly constrain the v4 extraction quality scoring system. |

## Intake Rules

- Only migrate durable lessons that improve correctness, determinism, auditability, or operational safety.
- Reject prototype-specific implementation details that do not apply to v4 architecture.
- Every migrated item must include destination and rationale.
- Track migrated items by linking both source file and updated canonical destination section.

## Section-Level Migration Map

| Map ID | Source Insight | Canonical Destination Section | Status |
|---|---|---|---|
| SLM-001 | Determinism ordering, normalization, and hash constraints | `DEV_STANDARDS_v4_2.md` -> `## Phase D Imported Constraints` / `### Determinism Guarantees` | Closed (2026-03-22) |
| SLM-002 | Session lifecycle logging and deprecation gate semantics | `DEV_STANDARDS_v4_2.md` -> `### Session and Operation Logging`; `DOC_GOVERNANCE.md` -> `## Deprecation and Feature Gates` | Closed (2026-03-22) |
| SLM-003 | Workflow state invariants and plan ordering contract | `ARCHITECTURE_v4_2.md` -> `## Phase D Imported Architecture Constraints` / `### Workflow State Invariants` and `### MergePlan Ordering Contract` | Closed (2026-03-22) |
| SLM-004 | Medium-specific chrome detector policy | `ARCHITECTURE_v4_2.md` -> `### Medium-Specific Chrome Detectors` | Closed (2026-03-22) |
| SLM-005 | Transcript backend quality-tier fallback rules | `TRANSCRIPT_ARCHITECTURE_v4_2.md` -> `## Backend Quality Tiers (Imported)` | Closed (2026-03-22) |
| SLM-006 | Known workflow gaps and threshold tunability constraints | `MASTER_PLAN_v4.md` -> `## Known Workflow Gaps (Imported From Postmortem)` and `### Chrome Detection Threshold Policy` | Closed (2026-03-22) |
| SLM-007 | Operation-family expectations and analyze-first usage | `ARCHITECTURE_v4_2.md` -> `### Baseline Operation Surface (Imported)`; `SETUP.md` -> `## Quick Verification Commands (Imported)` | Closed (2026-03-22) |
| SLM-008 | Extract-once, canonicalize-immediately, cache-and-serve contract | `ARCHITECTURE_v4_2.md` -> `### Transcript-First Extraction Contract` | Closed (2026-03-22) |
| SLM-009 | Profile-first detection, ordered ROI fallbacks, feature-flagged heuristic fallback | `ARCHITECTURE_v4_2.md` -> `### Profile-Driven Detection Architecture` | Closed (2026-03-22) |
| SLM-010 | Transcript quality gate thresholds (text presence, encoding integrity, ordering sanity, aggregate confidence) | `TRANSCRIPT_ARCHITECTURE_v4_2.md` -> `## Quality Gate Thresholds (Imported)` | Closed (2026-03-22) |
