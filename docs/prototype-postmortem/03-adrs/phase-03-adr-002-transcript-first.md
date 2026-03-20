# Phase 03 - ADR-002 Transcript-First Extraction

## Status

Accepted for the prototype codebase. This ADR documents the v3 migration from PDF.js-centered extraction to transcript-first extraction with PyMuPDF primary and PDF.js fallback.

## Scope

Capture why extraction was reorganized around a canonical `LayoutTranscript`, why PyMuPDF became the preferred extractor, how the fallback chain works in current code, and what the Rust rewrite must preserve.

## Source Evidence

- `packages/core/src/analyze/documentContext.ts`
- `packages/core/src/transcript/factory.ts`
- `packages/core/src/transcript/extractors/pymupdfExtractor.ts`
- `packages/core/src/transcript/extractors/pdfjsExtractor.ts`
- `packages/core/src/transcript/sidecar/extract-transcript.py`
- `packages/core/src/transcript/quality.ts`
- `docs/TRANSCRIPT_ARCHITECTURE.md`
- `docs/MIGRATION_V3.md`
- `docs/ARCHITECTURE.md`
- `docs/prototype-postmortem/02-algorithms/phase-02-quality-scoring.md`
- `docs/prototype-postmortem/00-admin/source-of-truth.md`

## Context

The prototype originally depended on PDF.js extraction semantics. That was good enough for basic text access, but not for the geometry-heavy tasks the system evolved toward:

- ROI-based sheet/spec detection
- reading-order reconstruction
- chrome suppression and band analysis
- schedule/table geometry
- deterministic anchor targeting for downstream workflows

The core issue was bounding-box fidelity. PDF.js `getTextContent()` provided usable text but unreliable span geometry for serious spatial analysis. The migration therefore moved extraction to a transcript model where:

- extraction produces a canonical `LayoutTranscript`
- `DocumentContext` initializes by extracting the transcript once per document
- `PageContext` consumes transcript spans instead of directly asking PDF.js for text items
- canonicalization normalizes coordinates, sort order, and transcript-level determinism before the rest of the system sees the data

This changed the architectural center of gravity from "PDF.js document with ad hoc extraction" to "canonical transcript with optional viewer/runtime fallbacks."

## Decision

Adopt transcript-first extraction as the canonical extraction model.

In current executable code, that means:

- `DocumentContext.initialize()` extracts a transcript once and caches it
- default extractor factory order is PyMuPDF first, then PDF.js fallback
- all successful extractor outputs are canonicalized before use
- downstream analyzers and locators consume transcript-derived page text rather than directly depending on extractor-specific APIs
- PDF.js remains only as a fallback extractor and, temporarily, as a bookmark-related surface in `DocumentContext`

## Why This Was Chosen

### Geometry accuracy

PyMuPDF's dict/rawdict-first span extraction provides the accuracy needed for geometric reasoning. The codebase and docs consistently treat PDF.js as materially lower confidence for bounding-box accuracy.

Prototype guidance states the expected difference plainly:

- PyMuPDF: production-quality geometry, approximately 95-99% bbox accuracy for intended parsing tasks
- PDF.js: fallback-only geometry, approximately 15-25% bbox accuracy for the same class of tasks

That difference is large enough to change architectural choices, not just implementation details.

### One extraction, many consumers

By extracting once at `DocumentContext.initialize()` and caching the transcript, the system avoids repeated extractor-specific work and gives every downstream module the same normalized representation.

This was important because the prototype had already accumulated multiple consumers that needed text and geometry:

- locators
- specs extraction
- narrative processing
- transcript quality scoring
- schedule extraction

### Stable, canonical contract

The transcript format creates a boundary between backend extraction mechanics and business logic. That let the codebase:

- plug in PyMuPDF without rewriting all downstream modules around Python-specific structures
- preserve API compatibility for existing callers of `DocumentContext` and `PageContext`
- apply canonicalization once so determinism rules are centralized

### Backward-compatible migration path

The migration was intentionally staged. Existing document/page APIs were kept stable while their internal extraction source changed. That reduced migration risk while still moving the architecture toward the transcript model.

## Current Implementation Shape

### Factory and fallback

`createTranscriptExtractor()` currently constructs a chain of:

- primary: `PyMuPDFExtractor`
- fallback: `PDFjsExtractor`

Fallback in current code is triggered only for PyMuPDF availability/runtime failures recognized by message pattern:

- PyMuPDF not installed
- Python runtime not found
- transcript extraction runtime unavailable

It is not currently triggered by low quality after a successful extraction.

### Canonicalization

All successful extractor results are passed through `canonicalizeTranscript()` before callers receive them.

This matters because transcript-first extraction is not only about a new backend. It is also about a new invariant:

- one normalized coordinate system
- one stable sort contract
- one deterministic transcript representation regardless of backend source

### `DocumentContext` consequence

`DocumentContext.initialize()` now:

- reads PDF bytes once
- extracts transcript once
- sets page count from transcript metadata
- keeps PDF.js loaded only temporarily for bookmark-related operations

`extractTextForPage()` then maps transcript spans into the legacy `TextItemWithPosition` shape expected by existing callers.

That is the architectural payoff: callers mostly keep their interface while the extraction model underneath becomes transcript-first.

## Trade-Offs

### Positive

- much better geometric fidelity for detection and parsing
- cached extraction at document level instead of repeated page-specific backend work
- unified representation for downstream modules
- smoother migration path because public APIs remained mostly unchanged
- explicit fallback path when Python/PyMuPDF is unavailable

### Negative

- the prototype now depends on an external Python runtime at runtime for the preferred path, not merely at build time
- fallback is availability/error based, not quality driven
- `DocumentContext` still temporarily carries PDF.js for bookmarks, so the migration is not fully complete
- extractor capability differences remain visible in quality scores even after transcript canonicalization

## Alternatives Rejected

### Stay PDF.js-first

Rejected because bounding-box quality was too weak for the geometric analysis the product actually needed.

### Build all logic directly around PyMuPDF-native structures

Rejected because it would hard-wire business logic to one backend and make fallback/replacement harder.

### Delay migration until Rust

Rejected because v3 still needed a production-usable extraction architecture before the Rust rewrite existed.

## Consequences for Downstream Design

Transcript-first extraction enabled or stabilized several later choices:

- ROI-driven locators could depend on more trustworthy span geometry
- quality scoring could reason about extractor output as a first-class artifact
- canonicalization became a foundational determinism step rather than an optional cleanup pass
- the extraction backend became replaceable while keeping the rest of the workflow engine intact

This ADR is therefore upstream of later design decisions. It is not just a parser implementation note.

## Known Limitations

- quality scoring does not yet trigger an automatic retry on alternate extractors after a successful low-quality run
- fallback remains PyMuPDF -> PDF.js; references to PDFium in architecture prose are future-directional, not current implementation
- bookmark extraction still keeps a temporary PDF.js dependency inside `DocumentContext`

## Rust Replacement Direction

Rust should preserve transcript-first extraction as the architectural model while replacing the backend stack.

The replacement target is:

- native PDFium-backed extraction for high-fidelity spans and layout metadata
- canonical transcript generation as a first-class internal contract
- no Python runtime dependency for the preferred or shipped path

The migration target is not "go back to direct backend-specific extraction." The migration target is "keep transcript-first, replace the extractor implementation."

## Prototype Invariants Worth Preserving

- extract once per document, cache, and reuse downstream
- canonicalize before consumers use transcript data
- keep a stable transcript contract independent of extraction backend
- fail over explicitly when the preferred backend is unavailable
- measure extraction quality as a first-class output of the pipeline

## Source-of-Truth Notes

For this ADR, executable code and Phase 2 algorithm capture were treated as canonical over aspirational architecture prose.

Important current-state clarifications:

- active fallback chain is PyMuPDF -> PDF.js
- PDFium is a Rust/V4 direction, not an active v3 extractor
- Python is a runtime dependency of the preferred prototype extraction path