# Phase 02 - Schedule Extraction

## Scope

Document geometry-first table extraction behavior and current limitations.

## Source Evidence

- `packages/core/src/transcript/schedules/extractor.ts`
- `packages/core/src/transcript/schedules/tableBuilder.ts`
- `packages/core/src/transcript/candidates.ts`

## Extraction Pipeline

`extractSchedules(transcript, options)`:

1. compute transcript candidates
2. run geometry table build over table candidates
3. additionally scan high line-density pages (`page.lines.length > 10`)
4. return extracted tables

Fallbacks for pdfplumber/camelot are marked TODO and not currently active.

## Geometry Table Builder

`buildTableFromGeometry(page, bbox?)`:

- optional bbox crop
- requires minimum candidate span/cardinality checks
- detects columns by x-center clustering
- groups rows by y clustering
- assigns each span to nearest column interval
- marks first row as header by default
- computes confidence from table structure consistency

## Output Contract

`ScheduleTable` includes:

- `tableId`
- `pageIndex`
- table `bbox`
- `rows` and `columns`
- `confidence`
- extraction `method`

Exports:

- CSV via `exportScheduleToCSV()`
- JSON via `exportScheduleToJSON()`

## Inputs and Outputs

- Input: canonical transcript pages with spans (and optional vector lines)
- Output: zero or more schedule tables

## Invariants

- deterministic clustering from fixed thresholds
- table rows sorted top-to-bottom
- cell assignment reproducible for same geometry

## Failure Modes

- merged cells are not explicitly modeled as spans across multiple columns/rows.
- rotated header text has no dedicated orientation normalization inside schedule module.
- line-only table structures with sparse text can evade current candidate thresholds.
