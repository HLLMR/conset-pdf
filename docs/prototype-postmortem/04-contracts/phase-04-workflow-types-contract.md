# Phase 04 - Workflow Types Contract

## Scope

Step 24 captures shared workflow-engine contracts and merge planning shapes that form the analyze/applyCorrections/execute boundary.

Primary contract files:

- `packages/core/src/workflows/types.ts`
- `packages/core/src/core/planner.ts` (for `MergePlan`)
- `packages/core/src/workflows/merge/types.ts` (merge workflow input contracts)

## Canonical Type Set

Core shared types (`packages/core/src/workflows/types.ts`):

- `WorkflowId` (`:16`)
- `Severity` (`:21`)
- `RowStatus` (`:26`)
- `Confidence` (`:32`)
- `InventoryRowBase`
- `Issue`
- `Conflict`
- `InventoryResult` (`:104`)
- `CorrectionOverlay` (`:141`)
- `ExecuteResult` (`:170`)

Merge planning types (`packages/core/src/core/planner.ts`):

- `MergePlan` (`:18`)

## Enum and Scalar Contracts

### Workflow identity

`WorkflowId = 'merge' | 'split' | 'assemble' | 'bookmark' | 'specs-patch' | 'fix-bookmarks'`

### Severity model

`Severity = 'error' | 'warning' | 'info'`

### Row status model

`RowStatus = 'ok' | 'warning' | 'error' | 'conflict'`

### Confidence

`Confidence` is numeric `0..1` by convention (not runtime-enforced as a branded type).

## Inventory Contract

`InventoryResult` requires:

- workflow identity (`workflowId`)
- row list (`rows`)
- issue list (`issues`)
- conflict list (`conflicts`)
- normalized summary object with baseline counters:
  - `totalRows`, `rowsWithIds`, `rowsWithoutIds`
  - `rowsOk`, `rowsWarning`, `rowsError`, `rowsConflict`
  - `issuesCount`, `conflictsCount`

Extension rule:

- `summary` allows workflow-specific numeric/string additions through index signature.

Optional additions used by merge workflow:

- `meta`
- `narrative`
- `narrativeValidation`
- `inventoryPath`

## Corrections Contract

`CorrectionOverlay` supports:

- ignore lists (`ignoredRowIds`, `ignoredPages`)
- row-level overrides (`overrides[rowId]`)
- conflict choice (`resolvedConflictChoice`)
- patch payloads (`patches` or `patchPath`)

Known limitation carried from implementation:

- merge execute accepts `corrections` but Phase 1 merge driver does not fully apply all overlay pathways.

## Execute Contract

`ExecuteResult` requires:

- `outputs: Record<string, string>`
- `summary: { success: boolean; ... }`

Optional:

- `warnings?: string[]`
- `errors?: string[]`

## MergePlan Contract

`MergePlan` includes:

- planned output sequence (`pages[]` with source/sourceIndex/sourceFile/id/title)
- replacement records (`replaced[]`)
- insertion records (`inserted[]`)
- unmatched records (`unmatched[]` with `reason` enum)
- parser diagnostics (`parseWarnings`, optional `parseNotices`)

This plan shape is the deterministic handoff boundary between analysis and mutation (`applyPlan`).

## Rust Mapping Notes

- Keep all union literals as tagged enum variants.
- Keep optional collections optional; do not silently default to empty arrays at serialization boundary unless explicitly documented.
- Preserve open-ended `summary` map behavior for workflow-specific counters.
- Keep `MergePlan` arrays ordered; ordering is behavior, not presentation.

## Evidence

- `packages/core/src/workflows/types.ts`
- `packages/core/src/workflows/merge/types.ts`
- `packages/core/src/workflows/mappers/merge.ts`
- `packages/core/src/core/planner.ts`
- `packages/core/src/core/mergeAddenda.ts`
- `03-adrs/phase-03-adr-003-workflow-engine.md`
