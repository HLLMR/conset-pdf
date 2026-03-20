# Phase 02 - Transcript Canonicalization

## Scope

Define transcript canonicalization guarantees required for deterministic extraction behavior.

## Source Evidence

- `packages/core/src/transcript/canonicalize.ts`

## Canonicalization Pipeline

`canonicalizeTranscript()`:

1. canonicalize each page
2. recompute deterministic `contentHash`
3. recompute deterministic `spanHash`

## Rotation Normalization

`normalizeRotation()` transforms bboxes to rotation `0` basis.

- 90 deg: `(x,y) -> (height - y, x)` mapping form
- 180 deg: `(x,y) -> (width - x, height - y)` mapping form
- 270 deg: `(x,y) -> (y, width - x)` mapping form

For 90/270 pages, width and height are swapped.

## Coordinate Normalization

`normalizeCoordinates()` enforces bbox ordering per span:

- `[min(x0,x1), min(y0,y1), max(x0,x1), max(y0,y1)]`

This normalizes direction/origin assumptions into a top-left, y-down consistent representation.

## Stable Sorting and Span IDs

`stableSortSpans()` sorting key:

- primary: `y`
- secondary: `x`
- tolerance: `0.1 pt`

After sorting, IDs are regenerated deterministically:

- `page{pageIndex}_span{idx}`

## Hash Inputs

### contentHash

Includes:

- page dimensions
- span count
- span text
- span bbox
- font name and size

### spanHash

Includes:

- page index/span count
- spanId
- span bbox
- font name/size
- font flags (bold/italic/fixed)

Excludes run-time extraction timestamp fields.

## Inputs and Outputs

- Input: raw `LayoutTranscript`
- Output: canonical `LayoutTranscript` with normalized pages and hash metadata

## Invariants

- all output pages have `rotation = 0`
- span ordering is deterministic for same geometric input
- hashing is deterministic for same canonicalized transcript

## Failure Modes

- If upstream extractors emit unstable bboxes, deterministic sorting can still shift.
- Hash behavior depends on consistent floating-point serialization in span geometry.
