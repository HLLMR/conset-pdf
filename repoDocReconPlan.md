## Plan: Doc Reset, Consolidation, and V4 Master Alignment

Treat Phase 0 foundation as complete and aligned, then perform a controlled documentation reset to reduce sprawl, preserve only useful knowledge, and make one updated master plan the source from which all other sticky docs derive.

## Core Assumptions

1. Phase 0 implementation is accepted as complete, tested, and validated for current scope.
2. Any top-level crate reorganization is structural cleanup, not a rewrite of Phase 0 behavior.
3. Prototype post-mortem content is mostly historical and non-normative for Rust V4.
4. Useful post-mortem content should be migrated selectively into V4 docs, then the library archived.

## Target End State

1. Minimal, durable doc set where each file has a clear owner and purpose.
2. One authoritative master plan that drives architecture, standards, and execution docs.
3. A compact living analysis library (small, post-mortem style) for ongoing platform evolution.
4. Deprecated and redundant docs removed from active navigation to avoid agent and team confusion.

## Phase A: Establish the New Doc Governance Model

1. Set authority order:
- Master Plan first.
- Supporting canonical docs second (architecture, standards, workflows).
- Code and tests as implementation evidence.
- Historical archives as informative only.
2. Define mandatory status tags in all surviving docs:
- Implemented, Planned, Deferred, Deprecated, Archived.
3. Add a doc decision rule:
- If a claim is not in the master plan or a derived canonical doc, it is non-authoritative.

## Phase B: Produce a Compact Current-State Analysis Library

Create a new small library under docs/current-state/ with exactly these files:

1. state-summary.md
- Where the project is now, what is complete, what is next.
2. capability-matrix.md
- Implemented vs planned capabilities by crate and workflow.
3. risk-register.md
- Active risks, severity, owner, mitigation, decision date.
4. decision-log.md
- High-value decisions and rationale that influence near-term execution.
5. migration-intake.md
- Candidate insights extracted from post-mortem with disposition:
	migrate, adapt, reject, archive-only.

Rules:
1. Keep this library small and continuously updated.
2. No speculative design prose; every claim maps to evidence or approved plan intent.

## Phase C: Full Repo Doc Triage (Keep, Merge, Delete)

Perform a complete inventory of docs/ and classify each file into one bucket:

1. Sticky canonical (keep and update).
2. Merge target (extract useful content into sticky docs, then delete source).
3. Historical archive (retain but clearly archived and removed from active index).
4. Delete outright (redundant, stale, or superseded with no unique value).

Triage criteria:
1. Unique value.
2. Current relevance to Rust V4.
3. Duplication level.
4. Maintenance cost.
5. Risk if removed.

## Phase D: Selective Post-Mortem Migration

1. Mine only durable insights from post-mortem:
- Determinism lessons.
- Workflow correctness pitfalls.
- Contract and schema pitfalls.
- Operational failure modes worth preventing in Rust.
2. Reject prototype-specific implementation details that do not apply to V4 architecture.
3. Record every imported item in migration-intake.md with rationale and destination doc.
4. After migration, archive the full post-mortem library behind an explicit historical index and remove it from default doc entrypoints.

## Phase E: Master-Plan-Centric Rewrite

1. Rewrite master plan as single strategic source of truth:
- Mission and constraints.
- Current baseline (including accepted Phase 0 completion).
- Next phases and sequencing.
- Quality gates and acceptance criteria.
- Documentation derivation map.
2. Rewrite supporting docs to derive from the new master plan only:
- Architecture doc defines how.
- Standards doc defines constraints and quality rules.
- Workflow doc defines execution mechanics.
- Implementation plan doc defines near-term delivery.
3. Remove or archive any supporting doc that cannot justify a unique role.

## Phase F: Repository Alignment Audit Against the New Docs

Audit code and tests against rewritten canonical docs with these outputs:

1. Alignment report:
- aligned, partially aligned, misaligned.
2. Gap register:
- required code changes, test changes, or doc corrections.
3. Priority queue:
- P0 correctness and determinism gaps.
- P1 workflow and auditability gaps.
- P2 structural or ergonomics gaps.

This phase is where we verify the statement that the current foundation is 100 percent complete for its intended scope and identify only remaining structural segmentation cleanup.

## Suggested Minimal Long-Term Doc Footprint

Active docs:
1. MASTER_PLAN_v4_2.md (or next versioned master plan).
2. ARCHITECTURE_v4_2.md.
3. DEV_STANDARDS_v4_2.md.
4. PHASE_X_IMPLEMENTATION_PLAN.md (single current execution plan).
5. DOCUMENTATION_INDEX.md.
6. docs/current-state/* (5-file compact library).

Archived docs:
1. Prototype post-mortem tree.
2. Legacy v3 and superseded planning docs.

Deleted docs:
1. Redundant or stale files with no unique retained content.

## Execution Sequence

1. Build current-state library skeleton.
2. Run full triage on existing docs.
3. Perform selective post-mortem migration.
4. Archive post-mortem from active navigation.
5. Rewrite master plan and derived canonical docs.
6. Run alignment audit and publish final reconciliation report.

## Acceptance Criteria

1. Doc count is substantially reduced and sustainable.
2. Every active doc has a single, non-overlapping purpose.
3. Master plan is authoritative and all supporting docs visibly derive from it.
4. Post-mortem is archived and no longer confuses active guidance.
5. Current-state library exists and is usable for ongoing incremental updates.
6. Repo alignment report is completed with prioritized follow-up actions.

## Immediate Next Deliverables

1. File-level keep/merge/delete matrix for every doc in docs/.
2. Draft skeleton content for docs/current-state/ (5 files).
3. Master plan rewrite outline with section-by-section deltas.
4. Archive plan for post-mortem entrypoint and routing changes.
