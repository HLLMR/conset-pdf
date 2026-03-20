# Phase 02 - Specs Extraction and Chrome Removal

## Scope

Capture section anchoring, chrome suppression, paragraph reconstruction, and heading/anchor detection used by specs extraction.

## Source Evidence

- `packages/core/src/specs/extract/chromeRemoval.ts`
- `packages/core/src/specs/extract/sectionDetector.ts`
- `packages/core/src/specs/extract/paragraphNormalizer.ts`
- `packages/core/src/specs/extract/anchorDetector.ts`
- `packages/core/src/transcript/candidates.ts`
- `packages/core/src/specs/footerSectionIdParser.ts`
- `packages/core/src/specs/footerSectionMap.ts`

## Chrome Removal

`removeChrome()` uses candidate-derived bands plus defaults:

- default header cutoff: top `15%`
- default footer cutoff: bottom `15%`
- detected header/footer bands can adjust those limits
- applies `+/-20pt` padding around detected bands
- keeps items whose center Y lies between adjusted thresholds

## Footer-First Anchoring Status

- `sectionDetector.ts` supports footer-first boundary mode (`boundarySource: footer|auto`).
- Footer map implementation is currently a stub (`buildFooterSectionMap()` returns empty ranges/stats).
- `parseFooterSectionId()` is implemented but constrained to Division 23 patterns:
  - `23 XX XX`, `23-XX-XX`, `23.XX.XX`, `23 - XX - XX`

Current state: footer-first logic exists in architecture, but authoritative page range construction is not fully wired.

## Section Detection Grammar

`detectSections()` three-phase model:

1. discover strict `SECTION NN NN NN` starts
2. scope sections from each start to next start (or EOF)
3. parse internals later in text extraction phase

Additional constraints:

- reject Division 01 references
- reject section markers in top/bottom chrome regions
- reject markers in bottom 25% of usable page area
- optional hard-fail on section count bounds

## Paragraph Normalization

`normalizeParagraph()` heuristics:

- joins wrapped lines when previous line is non-terminal and next starts lowercase
- repairs line-wrap hyphenation when previous line ends with `-` or `--`
- inserts paragraph breaks when signals indicate new paragraph

Helpers:

- `repairHyphens()` removes `word-\nword` splits
- `joinWrappedLines()` joins soft wraps by punctuation/lowercase heuristic

## Anchor Detection

`anchorDetector.ts` checks patterns in priority order:

- full hierarchical anchors
- partial letter-led anchors
- simple numeric anchors

Rules:

- anchors must be followed by additional text
- duplicate anchors are flagged
- nodes without anchors receive `ANCHOR_REQUIRED`

## Inputs and Outputs

- Inputs:
  - transcript/page text items
  - optional footer boundary map
  - section detection options
- Outputs:
  - filtered text (chrome removed)
  - detected sections and ranges
  - node anchors and issues

## Invariants

- Section IDs normalized to `DD SS SS` format.
- Section ranges are monotonic and non-negative.
- Chrome filtering is deterministic given same transcript candidates.

## Failure Modes

- Stubbed footer map limits determinism gains from footer-first segmentation.
- Division-23-only footer parser does not generalize across all spec divisions.
- Over-aggressive chrome filters can remove valid headings near page margins.
