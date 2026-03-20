# Phase 04 - Bookmark Types Contract

## Scope

Step 25 captures the bookmark workflow wire contract, including tree node identity and destination semantics.

Primary contract file: `packages/core/src/bookmarks/types.ts`.

## Canonical Types

Defined in `packages/core/src/bookmarks/types.ts`:

- `BookmarkDestination` (`:10`)
- `BookmarkNode` (`:30`)
- `BookmarkTree` (`:62`)
- `BOOKMARK_ISSUE_CODES` (`:74`)

## Destination Contract

`BookmarkDestination` required fields:

- `pageIndex: number` (0-based)
- `fitType: 'XYZ' | 'Fit' | 'FitH' | null`
- `isValid: boolean`

Optional location payload (for `XYZ`):

- `top?: number`
- `left?: number`
- `zoom?: number`
- `validationError?: string`

## Node Identity Contract

`BookmarkNode.id` is explicitly defined as stable node identity:

- preferred basis: source anchor (`BookmarkAnchorTree` anchor)
- fallback basis: logical path (for non-anchor cases)
- `pageIndex` is metadata only and is intentionally excluded from stable ID

Required node fields:

- `id`, `title`, `level`, `destination`, `page`, `status`

Optional relation and metadata fields:

- `sourceAnchor`, `logicalPath`, `parentId`, `childIds`, `issues`

## Tree Container Contract

`BookmarkTree` carries:

- `roots: BookmarkNode[]`
- `nodes: Map<string, BookmarkNode>`
- `source: string`

Rust migration implication:

- if persisted to JSON, `Map` serialization behavior must be explicitly defined (object map vs entry array) to avoid compatibility drift.

## Issue Code Enumeration

Defined set (9 codes):

- `NO_ID`
- `LOW_CONFIDENCE`
- `BOOKMARK_ORPHAN`
- `BOOKMARK_DEAD_DEST`
- `BOOKMARK_INVALID_FIT`
- `BOOKMARK_MISMATCHED_HIERARCHY`
- `BOOKMARK_DUPLICATE_TITLE`
- `BOOKMARK_ANCHOR_NOT_FOUND`
- `BOOKMARK_PAGE_HINT_MISMATCH`

This enum-like object is part of operational diagnostics contract and should remain stable.

## BookmarkAnchorTree Bridge Note

Specs to bookmark workflow bridge (`BookmarkAnchorTree` and `BookmarkAnchor`) is defined in workflow input types:

- `packages/core/src/workflows/bookmarks/types.ts`

This bridge carries logical outline anchors from specs patch output into bookmark generation.

## Rust Mapping Notes

- Preserve stable ID semantics exactly; never key identity by page index.
- Keep destination fit-mode subset fixed unless versioned.
- Preserve issue code strings exactly for compatibility with diagnostics, reports, and UI filters.

## Evidence

- `packages/core/src/bookmarks/types.ts`
- `packages/core/src/workflows/bookmarks/types.ts`
- `packages/core/src/bookmarks/treeBuilder.ts`
- `03-adrs/phase-03-adr-003-workflow-engine.md`
