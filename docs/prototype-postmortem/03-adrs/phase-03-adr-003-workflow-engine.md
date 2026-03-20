# Phase 03 - ADR-003 Workflow Engine Pattern

## Status

Accepted for the prototype codebase. This ADR documents the canonical workflow shape used to separate read-only analysis from destructive execution.

## Scope

Capture why the system standardized on `analyze -> applyCorrections -> execute`, how the pattern appears in concrete workflows, what it enables for CLI/GUI integration, and where current implementation still falls short of the full design intent.

## Source Evidence

- `packages/core/src/workflows/engine.ts`
- `packages/core/src/workflows/types.ts`
- `packages/core/src/workflows/merge/mergeWorkflow.ts`
- `packages/core/src/workflows/merge/types.ts`
- `packages/core/src/workflows/specs-patch/specsPatchWorkflow.ts`
- `packages/core/src/workflows/bookmarks/bookmarksWorkflow.ts`
- `packages/core/src/workflows/merge/index.ts`
- `packages/core/src/workflows/bookmarks/index.ts`
- `docs/WORKFLOWS.md`
- `docs/ARCHITECTURE.md`

## Context

The prototype serves both CLI and GUI use cases.

That creates a structural problem if each operation is written as a single monolithic command:

- the UI needs a read-only preview/inventory before making destructive changes
- the user may want to apply manual corrections before execution
- multiple surfaces need the same core logic without duplicating orchestration rules
- long-running or destructive operations need a stable contract for outputs, summaries, and warnings

The workflow engine pattern answers that by treating each domain operation as a three-stage pipeline:

- `analyze`: produce inventory and issues without writing outputs
- `applyCorrections`: apply user edits/overlays to analysis state
- `execute`: perform the actual file-producing operation

## Decision

Adopt a standardized workflow runner contract across major operations.

The canonical interface is defined in `WorkflowImpl` and wrapped by `createWorkflowRunner()`.

Each workflow exposes the same high-level surface:

- `analyze(input) -> InventoryResult`
- `applyCorrections(input, inventory, corrections) -> InventoryResult`
- `execute(input) -> ExecuteResult`

This pattern is the architectural contract for merge, specs patch, bookmarks, and future workflows.

## Why This Was Chosen

### Separate destructive work from read-only analysis

The primary driver is safety and inspectability.

`analyze()` is explicitly documented as a dry-run operation that must not produce output files. That gives the UI and CLI a place to surface:

- detected rows
- issues and warnings
- conflicts
- summary statistics
- optional advisory artifacts such as narrative validation

Only `execute()` is allowed to produce output files.

That separation is central to the product promise of determinism, auditability, and user review before mutation.

### Give GUI and CLI the same orchestration model

The workflow runner gives both surfaces the same mental model and integration contract. A wizard or CLI command can:

1. analyze
2. show inventory/issues
3. accept user corrections
4. execute

without each surface inventing its own control flow around underlying core APIs.

### Normalize data exchange across domains

`InventoryResult`, `CorrectionOverlay`, and `ExecuteResult` provide shared shapes for workflows that operate on very different document structures.

That matters because merge, specs patch, and bookmark repair are domain-distinct, but the user-facing orchestration problem is similar.

### Make later Rust migration easier

The workflow shape isolates orchestration semantics from specific implementation details. Rust can reimplement the internals of each workflow while keeping the same analyze/correct/execute staging contract.

## Current Implementation Shape

### Engine contract

`engine.ts` defines the generic workflow interface and `createWorkflowRunner()` simply exposes the implementation's three methods through a standardized runner.

This is intentionally thin. The engine is primarily a contract boundary, not a heavy framework.

### Shared workflow types

`types.ts` provides the shared language of the workflow layer:

- `WorkflowId`
- `InventoryRowBase`
- `Issue`
- `Conflict`
- `InventoryResult`
- `CorrectionOverlay`
- `ExecuteResult`

This gives the codebase a common vocabulary for reviewable state and execution outcomes.

### Concrete workflows using the pattern

#### Merge workflow

`mergeWorkflowImpl`:

- `analyze()` builds detection inventory and merge summary via planner logic
- `applyCorrections()` re-runs analysis, then applies ignored rows and ID overrides
- `execute()` performs merge output generation and can incorporate correction overlays into locator behavior

#### Specs patch workflow

`specsPatchWorkflowImpl`:

- `analyze()` extracts spec structure into rows/issues plus AST metadata
- `applyCorrections()` re-runs analysis, validates and applies patch operations, then regenerates inventory and bookmark tree
- `execute()` renders corrected output PDF

#### Bookmarks workflow

`bookmarksWorkflowImpl`:

- `analyze()` builds or reads bookmark tree and inventories bookmark issues
- `applyCorrections()` re-runs analysis and applies bookmark corrections to the tree
- `execute()` writes corrected bookmarks through the sidecar writer

## Benefits Achieved

### Strong review boundary

The system now has a first-class place to surface inventory and issues before mutating files.

### Corrections as explicit data

User edits are modeled as overlays rather than ad hoc imperative patching inside UI code. That is important for auditability and future persistence of correction state.

### Reuse across multiple product surfaces

The same workflow implementations can back CLI, GUI, and future automation surfaces.

### Shared summary/result model

Every workflow can return machine-usable outputs plus human-usable summaries and warnings in a consistent structure.

## Trade-Offs and Current Limitations

### The design intent is ahead of the implementation in some places

The architectural story is "analyze once, let the user correct, then execute from the reviewed state." Current code only partially realizes that.

In all three implemented workflows, `applyCorrections()` currently re-runs `analyze()` to obtain fresh state instead of mutating previously cached analysis state in place.

That means the pattern provides a review boundary, but not yet a full no-reparse cursor-state pipeline.

### Plan caching is only partially realized

The merge execute types expose optional analyzed plan input, but current merge execution still re-analyzes when corrections are involved and does not meaningfully use analyzed plan state as the primary execution cursor.

So the interface supports future plan/state reuse more strongly than the current implementation does.

### Engine layer is deliberately thin

This is mostly a positive, but it means lifecycle guarantees like cache reuse, persisted overlays, or execution graphing are not provided by the engine itself. Each workflow still owns those details.

## Alternatives Rejected

### Monolithic command-per-operation design

Rejected because it makes preview/correction UI awkward and pushes orchestration logic into every caller.

### UI-owned correction logic without core workflow contract

Rejected because it would duplicate domain logic across clients and weaken auditability.

### Fully generic workflow framework with heavy runtime abstraction

Rejected in favor of a thin contract layer. The prototype needed consistency, not framework complexity.

## Consequences for Architecture

This pattern makes the workflow layer the canonical integration surface above low-level parsing and PDF operations.

That means:

- low-level modules remain focused on domain logic
- user-facing orchestration happens through workflow runners
- correction overlays become durable architectural objects instead of UI-only state
- audit/report surfaces can standardize around inventory and execute results

## Rust Preservation Requirements

Rust should preserve the workflow staging model even if implementation details change.

The must-keep behavior is:

- reviewable read-only analysis before mutation
- explicit correction overlay application step
- execute-only destructive/file-writing phase
- common inventory/result contracts across workflows

Rust should improve on the prototype by making cached analyzed state and correction application more directly reusable between phases instead of re-running analysis whenever corrections are applied.

## Prototype Invariants Worth Preserving

- `analyze()` must remain non-destructive
- `execute()` is the only phase allowed to produce final output files
- corrections should be data overlays, not UI-local mutations
- workflow outputs should use consistent inventory/issue/summary/result shapes
- multiple product surfaces should share the same workflow orchestration contract

## Source-of-Truth Notes

For this ADR, code was treated as canonical over comments and idealized workflow prose.

Important current-state clarifications:

- the workflow engine pattern is real and implemented
- the no-reparse ideal is only partially achieved because concrete `applyCorrections()` implementations re-run `analyze()`
- merge `analyzed.plan` support is presently more aspirational than foundational in execution flow