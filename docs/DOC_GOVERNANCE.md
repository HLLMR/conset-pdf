# Conset PDF Documentation Governance

**Version:** 1.0.0  
**Date:** March 22, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

---

## Purpose

This document defines the documentation authority model for Conset PDF and prevents conflicting guidance across planning, architecture, standards, and execution docs.

---

## Authority Order

Documentation authority is ranked in this order:

1. **Master plan first**: `docs/MASTER_PLAN.md` (current constitutional source)
2. **Canonical derived docs second**: architecture, standards, workflow, and active implementation plan docs
3. **Code and tests third**: implementation evidence and behavior verification
4. **Historical archives last**: informative only, never normative

---

## Mandatory Doc Status Tags

Every active or archived documentation file must declare one `Doc Status Tag` in its metadata block.

Allowed values:

- `Implemented`
- `Planned`
- `Deferred`
- `Deprecated`
- `Archived`

Interpretation:

- `Implemented`: reflects delivered behavior or accepted baseline
- `Planned`: approved future intent not yet delivered
- `Deferred`: intentionally postponed work
- `Deprecated`: superseded and pending retirement
- `Archived`: historical record, retained for context only

---

## Documentation Decision Rule

If a claim is not present in the current master plan or a canonical doc derived from it, the claim is **non-authoritative**.

Non-authoritative content may be useful context, but it cannot define roadmap, architecture constraints, implementation requirements, or acceptance criteria.

---

## Canonical Baseline For Phase A

As of Phase A kickoff, the canonical set is:

1. `docs/MASTER_PLAN.md`
2. `docs/ARCHITECTURE.md`
3. `docs/DEV_STANDARDS.md`
4. `docs/DOCUMENTATION_INDEX.md`
5. Active implementation plan in `docs/dev/`

This list will be tightened further during triage and consolidation phases.

---

## Coordinate Space Conventions

Canonical docs that define transcript/layout semantics must use consistent coordinate terminology.

Required conventions:

1. Distinguish PDF-space and normalized visual-space explicitly.
2. State where rotation normalization occurs in the pipeline.
3. Define precision policy for coordinate comparisons and persistence.

If conventions are split across docs, the architecture and transcript architecture docs are the controlling references.

---

## Deprecation and Feature Gates

Every deprecation gate or feature flag documented in active docs must specify:

1. Default state (`on` or `off`)
2. Behavior class (`hard-fail` or `soft-warn`)
3. Intended removal or review milestone

Undocumented gate semantics are non-authoritative and must not drive implementation behavior.

---

## Phase Closure Snapshot (Canonical)

The following closure outcomes are canonical and replace separate phase closure artifact docs.

### Phase C Documentation Triage Closure (2026-03-22)

- Full docs triage completed with merge-target count = 0.
- Deprecated v3 merge-target documentation set was retired after extraction.
- Current documentation set is governed by the authority order in this document and indexed via `docs/DOCUMENTATION_INDEX.md`.

### Phase D Migration Closure (2026-03-22)

- Intake scope completed: M-001 through M-015 and SLM-001 through SLM-010.
- Disposition counts: 5 migrated, 6 adapted, 3 rejected, 1 archive-only (15 total).
- Section-level closure maps SLM-001 through SLM-010 are closed.
- Canonical destination anchors:
	- `docs/ARCHITECTURE.md` (workflow/order contracts, transcript-first, profile-driven detection)
	- `docs/DEV_STANDARDS.md` (determinism guarantees, session/operation logging)
	- `docs/TRANSCRIPT_ARCHITECTURE.md` (backend quality tiers, quality gate thresholds)
	- `docs/MASTER_PLAN.md` (known workflow gaps, chrome detection threshold policy)
