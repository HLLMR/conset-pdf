## Master-Plan-Aligned Agent Execution Plan: Simple First AEC GUI

This version supersedes the original draft and is explicitly aligned to roadmap timing, phase definitions, and current gap status.

Primary product goal remains unchanged:
1. Add files
2. Start processing
3. Review flagged items
4. Export

Default UX constraints remain unchanged:
1. One-window simple flow
2. Technical details hidden behind Advanced by default
3. Plain-language statuses only

## 0) Master Plan Fit (Non-Negotiable)

This plan is constrained by canonical policy:
1. GUI is a Phase 11+ delivery lane in the roadmap timeline.
2. Integration is contract-first via shared contracts.
3. GUI must not depend on engine internals.
4. One-button workflows are required in paid GUI.
5. Partial success must export successfully.
6. No silent failures; unresolved confidence must surface as Needs review.
7. Audit/provenance must exist, but advanced details remain optional.

Execution consequence:
1. Current work must be split into two tracks:
2. Track A (Now): contract-shaping, state model, test scaffolding, and lane migration prep.
3. Track B (Phase 11+): production GUI runtime integration lane-by-lane after dependency gates pass.

## 1) Phase-Gated Scope

### Track A: Allowed Now
1. Session state and reducer logic for the simple 4-step UX.
2. Operation-chain policy contracts and adapter interfaces.
3. Review queue and inline edit semantics in session state.
4. Export eligibility rules and deterministic selection policy.
5. Mock-driven integration tests and parity harness scaffolding.

### Track A: Explicitly Not Allowed
1. Claiming full production Start-chain behavior over unresolved backend gaps.
2. Coupling GUI behavior directly to unfinished engine internals.
3. Treating mocks as production-ready evidence.

### Track B: Phase 11+ Runtime Enablement
1. Enable real lane execution in desktop GUI against real backend responses.
2. Migrate workflows lane-by-lane with parity checks.
3. Keep prototype behavior bugfix-only during canonical migration.

## 2) Dependency Gates

Gate 0: Contract Boundary Ready
1. Shared request/response/event schema stable for target lane.
2. GUI integration tests cover current contract shape.

Gate 1: Baseline Pipeline Ready
1. Critical extraction chain closed: G-004 -> G-005 -> G-001 -> G-002 -> G-003 -> G-009.
2. Real artifacts exist for baseline extraction flow.

Gate 2: Review Inputs Ready
1. Warning/failure payloads are consumable for review queue generation.
2. Audit hooks are wired enough for advanced evidence surfaces (G-006).

Gate 3: Export Inputs Ready
1. Artifact outputs are stable and deterministically selectable.
2. Partial-success export behavior is validated.

If a gate is not met:
1. Continue Track A with mocks/adapters.
2. Preserve interfaces.
3. Do not mark lane production-ready.

## 3) Agent Packets (Track A Now, Track B Later)

### Agent A: Session State and Transition Engine
Track A:
1. Define deterministic session/file/review state models.
2. Implement pure transition logic and repeatable tests.
Track B:
1. Bind transitions to real backend responses.
Entry gate: Gate 0

### Agent B: Start Button Chain Policy
Track A:
1. Define fixed precedence and capability-driven adapter interface.
2. Define plain-language status mapping for default UI.
Track B:
1. Execute real operation chain when Gate 1 passes.
Entry gates: Track A Gate 0, Track B Gate 1

### Agent C: Review Queue and Inline Edits
Track A:
1. Define review item schema and one-at-a-time queue behavior.
2. Define Confirm/Edit/Skip decision semantics.
Track B:
1. Bind queue to real warnings/audit data once Gate 2 passes.
Entry gates: Track A Gate 0, Track B Gate 2

### Agent D: Export Policy and Packaging
Track A:
1. Define readiness rules and deterministic artifact ordering policy.
2. Define plain-language export summary contract.
Track B:
1. Implement real packaging with optional advanced audit bundle export.
Entry gates: Track A Gate 0, Track B Gate 3

### Agent E: Contract Stewardship
Track A:
1. Add optional fields only when required by proven gaps.
2. Preserve backward compatibility when fields are absent.
Track B:
1. Coordinate lane-specific contract bumps with parity tests.
Entry gate: Gate 0

### Agent F: Integration and Parity Testing
Track A:
1. Build mock-driven tests for the 4-step UX.
2. Add deterministic rerun checks.
Track B:
1. Replace mocks lane-by-lane with real integration tests.
Entry gates: Track A Gate 0, Track B Gates 1-3 per lane

## 4) Lane Migration Order (Phase 11+)

Lane 1 (MVP): Add files -> Start -> Review -> Export
1. Must satisfy simple-first UX.
2. Must preserve partial-success export policy.

Lane 2: Advanced review details
1. Surface confidence/audit/provenance only behind Advanced.

Lane 3: Higher-order workflows
1. Add compare/exception and additional workflow lanes after Lane 1 soak stability.

Lane promotion criteria:
1. Integration tests pass.
2. Determinism checks pass.
3. No silent-failure regressions.

## 5) Compliance Checklist

This plan is compliant only if all remain true:
1. Desktop-first scope.
2. Contract-first boundary.
3. No GUI import from engine internals.
4. One-button workflow behavior.
5. Partial-success exports remain available.
6. No silent failures.
7. Advanced details hidden by default.

Canonical anchors:
1. [docs/MASTER_PLAN.md](docs/MASTER_PLAN.md)
2. [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
3. [docs/current-state/gap-register.md](docs/current-state/gap-register.md)

## 6) DoD for This Plan Revision

1. Agent packets are phase-gated and master-plan-timed.
2. Track A vs Track B boundaries are explicit.
3. Dependency gates block premature production claims.
4. Lane migration order and parity requirements are explicit.
