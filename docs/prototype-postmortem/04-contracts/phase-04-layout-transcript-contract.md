# Phase 04 - Layout Transcript Contract

## Scope

Step 23 captures the canonical transcript wire contract used by extraction, ROI/location logic, quality scoring, and downstream workflows.

Primary code contract: `packages/core/src/transcript/types.ts`.

## Canonical Types

Defined in `packages/core/src/transcript/types.ts`:

- `LayoutTranscript`
- `TranscriptMetadata`
- `LayoutPage`
- `PageMetadata`
- `LayoutSpan`
- `LayoutImage`
- `LineSegment`
- `QualityMetrics`

Symbol anchors:

- `LayoutTranscript`: `packages/core/src/transcript/types.ts:11`
- `TranscriptMetadata`: `packages/core/src/transcript/types.ts:27`
- `LayoutPage`: `packages/core/src/transcript/types.ts:41`
- `LayoutSpan`: `packages/core/src/transcript/types.ts:77`
- `LayoutImage`: `packages/core/src/transcript/types.ts:106`
- `LineSegment`: `packages/core/src/transcript/types.ts:122`
- `QualityMetrics`: `packages/core/src/transcript/types.ts:140`

## Wire-Shape Contract

### `LayoutTranscript`

Required fields:

- `filePath: string`
- `extractionEngine: string`
- `extractionDate: string` (ISO timestamp)
- `pages: LayoutPage[]`
- `metadata: TranscriptMetadata`

Determinism note:

- `extractionDate` is explicitly excluded from deterministic hashes.

### `TranscriptMetadata`

Required fields:

- `totalPages: number`
- `hasTrueTextLayer: boolean`

Optional deterministic fields:

- `contentHash?: string`
- `spanHash?: string`

### `LayoutPage`

Required fields:

- `pageNumber: number` (1-based)
- `pageIndex: number` (0-based)
- `width: number`
- `height: number`
- `rotation: number`
- `spans: LayoutSpan[]`
- `metadata: PageMetadata`

Optional fields:

- `images?: LayoutImage[]`
- `lines?: LineSegment[]`

### `LayoutSpan`

Required fields:

- `text: string`
- `bbox: [x0, y0, x1, y1]`
- `fontName: string`
- `fontSize: number`
- `flags: { isBold?: boolean; isItalic?: boolean; isFixedPitch?: boolean }`
- `spanId: string`
- `pageIndex: number`

Optional fields:

- `color?: string` (hex)

### `LayoutImage`

Required fields:

- `imageId: string`
- `bbox: [x0, y0, x1, y1]`
- `width: number`
- `height: number`
- `pageIndex: number`

### `LineSegment`

Required fields:

- `lineId: string`
- `start: [x, y]`
- `end: [x, y]`
- `pageIndex: number`

Optional fields:

- `width?: number`
- `color?: string`

### `QualityMetrics`

Required fields:

- `extractedCharCount: number`
- `whiteSpaceRatio: number`
- `replacementCharCount: number`
- `orderingSanityScore: number`
- `estimatedOCRNeeded: boolean`
- `confidenceScore: number`

## Coordinate and Rotation Semantics

Type-level statement in `LayoutSpan.bbox` says visual-space points (`[x0, y0, x1, y1]`).

Canonicalization path in `packages/core/src/transcript/canonicalize.ts` enforces:

- rotation normalization to `rotation = 0`
- bbox ordering normalization (`x0 <= x1`, `y0 <= y1`)
- stable span order by `(y, x)` with tolerance

Important migration constraint:

- Rust implementation must preserve post-canonicalization visual-space behavior and stable ordering before hash computation.

## Hash and Stability Contract

Per `packages/core/src/transcript/canonicalize.ts`:

- `contentHash` includes page dimensions, span count, span text, bbox, font name/size.
- `spanHash` includes page index, span count, span ID, bbox, font name/size, and flag bits.
- Both hashes exclude `extractionDate`.

Deterministic ID behavior:

- canonicalization rewrites span IDs to deterministic `page{pageIndex}_span{idx}` sequence after stable sort.

## Rust Mapping Notes

- Keep `pageNumber` and `pageIndex` duality (user-facing vs zero-based internal).
- Preserve tuple ordering and numeric precision semantics for `bbox`, `start`, `end`.
- Maintain optionality exactly (`Option<T>` only where `?` is present).
- Preserve hash inclusion/exclusion list exactly to avoid drift in regression tests.

## Evidence

- `packages/core/src/transcript/types.ts`
- `packages/core/src/transcript/canonicalize.ts`
- `docs/MIGRATION_V3.md`
- `docs/TRANSCRIPT_ARCHITECTURE.md`
- `03-adrs/phase-03-adr-005-determinism.md`
