# Conset PDF Documentation Index

**Repository**: conset-pdf  
**Version**: 2.0.0  
**Last Updated**: March 24, 2026  
**Status**: Active canonical index  
**Doc Status Tag:** Implemented

---

## Documentation Governance

This index follows `DOC_GOVERNANCE.md`.

Authority order:

1. `docs/MASTER_PLAN.md`
2. Canonical derived docs (`docs/ARCHITECTURE.md`, `docs/DEV_STANDARDS.md`, `docs/AEC_STANDARDS.md`, `docs/TRANSCRIPT_ARCHITECTURE.md`)
3. Code and tests as implementation evidence
4. Historical archives as informative only

Decision rule:

If a claim is not in the master plan or canonical derived docs, it is non-authoritative.

---

## Canonical Entry Points

Use this set for active planning and implementation guidance:

1. `docs/MASTER_PLAN.md`
2. `docs/ARCHITECTURE.md`
3. `docs/DEV_STANDARDS.md`
4. `docs/AEC_STANDARDS.md`
5. `docs/TRANSCRIPT_ARCHITECTURE.md`
6. `docs/DOC_GOVERNANCE.md`
7. `docs/REPO_STRUCTURE.md`

Standards data annexes (part of the canonical standards suite):

- `docs/UDS_DISCIPLINES.md`
- `docs/MASTERFORMAT_REFERENCE.md`
- `docs/DRAWINGS_CLASSIFICATION.md`

Execution planning:

- `docs/dev/PHASE_0_IMPLEMENTATION_PLAN_v4_2_1.md`

Current-state library:

- `docs/current-state/state-summary.md`
- `docs/current-state/capability-matrix.md`
- `docs/current-state/risk-register.md`
- `docs/current-state/decision-log.md`
- `docs/current-state/gap-register.md`

---

## Start Here

### For Project Direction

Start with [MASTER_PLAN.md](./MASTER_PLAN.md).

### For System Design

Start with [ARCHITECTURE.md](./ARCHITECTURE.md).

### For Implementation Constraints

Start with [DEV_STANDARDS.md](./DEV_STANDARDS.md).

### For Domain Standards

Start with [AEC_STANDARDS.md](./AEC_STANDARDS.md).

### For Extraction Contract

Start with [TRANSCRIPT_ARCHITECTURE.md](./TRANSCRIPT_ARCHITECTURE.md).

### For Repository Layout

Start with [REPO_STRUCTURE.md](./REPO_STRUCTURE.md).

### For Phase D Integration Outcomes

Start with [ARCHITECTURE.md](./ARCHITECTURE.md) under "Phase D Monorepo Integration Outcomes (M-001 to M-003)".

### For Standards Data Implementation

Reference the data recovered from the prototype (now at top-level `docs/` alongside the canonical standards docs):

- [UDS_DISCIPLINES.md](./UDS_DISCIPLINES.md) — Full UDS discipline table (21 discipline groups, ~100 sub-disciplines with 4-char codes, sort order, and descriptions); canonical source for `crates/standards-data`
- [MASTERFORMAT_REFERENCE.md](./MASTERFORMAT_REFERENCE.md) — CSI MasterFormat 2018 divisions (35 entries) and pre-2004 legacy migration table with range-based lookup algorithm
- [DRAWINGS_CLASSIFICATION.md](./DRAWINGS_CLASSIFICATION.md) — Drawing discipline classification algorithm: UDS single-letter designators, multi-letter aliases with confidence scores, `C`-designator disambiguation, and sort-order heuristic table

### For GUI Workstream Execution (Phase 11+)

The GUI agent execution protocol and workstream governance rules live in `MASTER_PLAN.md` in two locations:

- **Phase 11+ guardrails — "GUI agent execution protocol (required)"**: Two-track model (Track A prep / Track B runtime), dependency Gates 0–3, canonical Lane order (Lane 1 MVP → Lane 2 advanced → Lane 3 higher-order), and lane promotion criteria. Agents must read this section before beginning any GUI implementation work.
- **Development Workflow — "GUI Workstream Protocol (Phase 11+)"**: Required execution sequence, agent packet requirements, and production readiness rules governing when Track B runtime integration may begin.

Quick reference: all GUI-facing contract types (`WorkflowRequest`, `WorkflowResponse`, `OperationStatus`, `OperationResult`, `AuditEventData`) live in `crates/contracts/src/lib.rs`. GUI command stubs live in `apps/desktop-gui/src/lib.rs`. Integration tests for IPC contract compliance live in `tests/integration/gui_ipc_test.rs`.
