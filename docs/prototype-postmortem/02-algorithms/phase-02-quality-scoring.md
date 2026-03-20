# Phase 02 - Quality Scoring

## Scope

Capture quality metrics, gate thresholds, and extractor fallback relationship.

## Source Evidence

- `packages/core/src/transcript/quality.ts`
- `packages/core/src/transcript/factory.ts`
- `packages/core/src/transcript/extractors/pymupdfExtractor.ts`
- `packages/core/src/transcript/extractors/pdfjsExtractor.ts`

## Per-Page Metrics

`scorePageQuality()` computes:

- `extractedCharCount`
- `whiteSpaceRatio`
- `replacementCharCount` (U+FFFD)
- `orderingSanityScore`
- `estimatedOCRNeeded`
- `confidenceScore`

## Aggregate Gates

`scoreTranscriptQuality()` gates:

- minimum chars per page: `>= 50`
- replacement ratio: `<= 0.05`
- ordering sanity: `>= 0.80`
- aggregate confidence: `>= 0.85`

`passes` is true only when all gates pass.

## Ordering Sanity

Measured by comparing source order against geometric sort order:

- primary y, secondary x
- y tolerance of 5 points for line grouping
- positional tolerance allows small index offsets

## Extractor Selection Relationship

Current extractor fallback in `factory.ts` is availability/error based, not quality-gate based:

- preferred/default chain: PyMuPDF then PDF.js
- fallback triggered on PyMuPDF availability/runtime errors
- canonicalization applied to whichever extractor succeeds

Quality scoring currently reports quality but does not auto-switch extractor after successful extraction.

## Inputs and Outputs

- Input: `LayoutTranscript`
- Output: `QualityReport` with page metrics, aggregate metrics, gate booleans, issues

## Invariants

- deterministic scoring for same transcript
- explicit issue messages for each gate failure
- confidence clamped to `[0,1]`

## Failure Modes

- hard-coded thresholds may not generalize across all document classes.
- low text pages (for example separators) can fail min-char gate even when extraction is valid.
- no automatic quality-driven extractor retry path yet.
