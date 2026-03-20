# Phase 02 - Merge Planning

## Scope

Capture merge planning semantics (replace/insert/append), unmatched handling, cover-page behavior, and application invariants.

## Source Evidence

- `packages/core/src/core/planner.ts`
- `packages/core/src/core/applyPlan.ts`

## Planning Inputs

- original PDF path
- one or more addendum PDF paths
- doc type (`drawings` or `specs`)
- mode: `replace+insert`, `replace-only`, `append-only`
- strict mode flag
- locator
- optional replacement overrides

## ID Parsing for Planning

`parsePdfIds()` parses every page and emits normalized IDs with confidence.

Threshold gates:

- drawings accepted at `>= 0.60`
- specs accepted at `>= 0.50`

## Cover Page Behavior

Page 1 with no reliable ID may be normalized as synthetic ID `COVER`:

- assigned confidence `1.0`
- retained in inventory/page mapping for replacement matching

## Working Set Algorithm

Planner builds ordered working set from original pages, then processes addenda in sequence.

For each addendum ID group:

- if ID exists in working set and mode allows replace:
  - remove all matching entries
  - insert addendum pages at first matched position
- else if mode allows insertion:
  - natural-sort insertion point by ID
  - fallback: insert before first null-ID page
- else mark unmatched

Unprocessed addendum pages become `no-id` unmatched (or are appended in append-only mode).

## Duplicate Handling

- ID maps keep arrays of occurrences.
- Replacement removes all existing matching pages for target ID and inserts addendum set.
- Diagnostic logging captures duplicate groups and chosen retention behavior.

## Output Contract

`MergePlan` includes:

- `pages` ordered output instructions
- `replaced`
- `inserted`
- `unmatched`
- `parseWarnings`
- optional `parseNotices`

## Plan Application

`applyMergePlan()` behavior:

- creates new output PDF
- copies pages from source files according to ordered plan
- optional bookmark regeneration from planned page IDs/titles
- output write through pikepdf sidecar (`writePdfWithPikepdf`)

## Invariants

- planner is deterministic for same parsed IDs and mode options
- output page sequence equals planned working-set order
- source pages are copied, not re-rendered

## Failure Modes

- strict mode fails merge when addendum pages lack IDs.
- low-confidence IDs can suppress intended replacements.
- duplicate IDs can over-remove pages if source data quality is poor.
