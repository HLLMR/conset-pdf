# Phase 02 - Narrative Parser

## Scope

Document deterministic narrative parsing and inventory-validation behavior for drawing/spec instruction extraction.

## Source Evidence

- `packages/core/src/narrative/parse-algorithmic.ts`
- `packages/core/src/narrative/validate.ts`
- `packages/core/src/narrative/normalize.ts`

## Parser Entry Point

`parseNarrativeAlgorithmic(doc)`:

- pure algorithmic parsing (no LLM)
- identifies drawing and spec revision sections by heading patterns
- emits:
  - drawing instructions
  - spec instructions
  - parse issues

## Drawings Section Parsing

Recognizes section headings and row patterns such as:

- numbered `SHEET` lines
- bare sheet ID plus title forms

Behavior:

- sheet IDs normalized via narrative normalizer
- optional note lines captured (lettered sub-items, `Formerly named`, `Note:`)
- duplicates are detected and reported

## Specs Section Parsing

Recognizes `SECTION` lines with both strict and spacing-tolerant forms:

- handles compact and split digit variants
- normalizes to `NN NN NN`
- captures title and section action lines

Action verb classification:

- `add`, `revise`, `delete`, `replace`, `unknown`

Noise suppression removes common header/footer/admin artifacts.

## Validation Against Inventory

`validate.ts` behavior:

- deterministic validation only (advisory; does not mutate inventory)
- partitions inventory by document sequence (original vs addendum)
- Levenshtein near-match scoring for typo/similarity suggestions
  - default near-match threshold: `0.75`
  - max near matches per issue configurable (default 3)

Also includes line-based set-membership logic for high-confidence rename/replace style suggestions.

## Inputs and Outputs

- Inputs:
  - extracted narrative text document
  - inventory result for validation stage
- Outputs:
  - `NarrativeInstructionSet`
  - validation report and optional correction suggestions

## Invariants

- parser is deterministic for same text input
- normalization applied before cross-checking inventory IDs
- validation emits issues/suggestions only, no destructive changes

## Failure Modes

- highly variable narrative prose can evade strict heading/pattern detectors
- malformed section numbers can be normalized into invalid semantic references
- near-match heuristics can over-suggest in dense ID neighborhoods
