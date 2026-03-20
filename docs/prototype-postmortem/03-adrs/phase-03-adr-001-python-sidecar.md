# Phase 03 - ADR-001 Python Sidecar Pattern

## Status

Accepted for the prototype codebase. This ADR documents the pattern actually used in v3 and the constraints that motivated it.

## Scope

Capture why Python sidecars were introduced, where the boundary exists today, what problems it solved, and what the Rust rewrite should replace natively.

## Source Evidence

- `packages/core/src/transcript/extractors/pymupdfExtractor.ts`
- `packages/core/src/transcript/sidecar/extract-transcript.py`
- `packages/core/src/bookmarks/pikepdfBookmarkWriter.ts`
- `packages/core/src/bookmarks/sidecar/bookmark-writer.py`
- `packages/core/src/utils/pikepdfWriter.ts`
- `packages/core/scripts/copy-sidecars.mjs`
- `packages/core/package.json`
- `docs/TRANSCRIPT_ARCHITECTURE.md`
- `docs/ARCHITECTURE.md`
- `docs/MIGRATION_V3.md`
- `docs/largeFileRefactorPlan.md`
- `docs/MASTER_PLAN_v4.md`

## Context

The prototype needed PDF capabilities that the TypeScript-only stack could not deliver with acceptable accuracy or reliability:

- transcript extraction needed PyMuPDF span geometry instead of PDF.js bounding boxes
- bookmark writing needed pikepdf/QPDF output for cross-viewer compatibility
- merge output needed a more reliable writer path than raw `pdf-lib` serialization

At the same time, the V4 direction requires a permissive engine license and ultimately a single Rust binary with no Python runtime dependency for end users. That made direct entanglement of the Node package with Python libraries an architectural dead end.

The prototype therefore isolated Python-dependent PDF operations behind process boundaries:

- TypeScript orchestrates and owns workflow state
- Python scripts perform the specialized PDF operation
- data crosses the boundary as file paths and JSON payloads
- the core package copies sidecar scripts into `dist/` during build so runtime can execute them from packaged output

## Decision

Use Python sidecars, invoked by `execFile`, for PDF operations where the TypeScript stack is insufficient or materially less reliable.

The sidecar boundary is the contract:

- Node/TypeScript remains the system orchestrator
- Python is an implementation detail for selected PDF operations
- sidecars accept explicit file paths and/or JSON inputs
- sidecars return either files on disk or JSON artifacts
- runtime dependency failures surface as explicit user-facing errors with install instructions

## Affected Subsystems

### 1. Transcript Extraction

`PyMuPDFExtractor` finds a Python executable at runtime, invokes `extract-transcript.py`, reads a JSON transcript from a temp directory, and returns standardized `LayoutTranscript` data.

Why this moved behind a sidecar:

- PyMuPDF provides dict/rawdict-first span extraction with the geometry quality needed for deterministic downstream parsing
- the TypeScript layer only needs the transcript contract, not direct bindings to the extractor implementation
- fallback to PDF.js stays in Node, so the system can degrade gracefully when Python is unavailable

### 2. Bookmark Writing

`writeBookmarksViaSidecar()` serializes bookmark entries to JSON and calls `bookmark-writer.py`, which uses pikepdf/QPDF primitives to write outline dictionaries and `/A` GoTo actions, then verifies outline linkage and destination validity.

Why this moved behind a sidecar:

- `pdf-lib` bookmark support was not reliable enough across target viewers
- pikepdf/QPDF gave better structural control over outlines and destination objects
- viewer-compatibility validation fits naturally inside the same PDF-native toolchain

### 3. PDF Passthrough Writing for Merge Output

`writePdfWithPikepdf()` and `writePdfFileThroughPikepdf()` route final PDF output through `bookmark-writer.py --mode passthrough`.

Important implementation note:

- this is a sidecar-based write boundary
- it is not yet the fully disk-streaming merge architecture described in `docs/largeFileRefactorPlan.md`
- current `applyMergePlan()` still assembles the output document in memory with `pdf-lib` before the final pikepdf passthrough write

So the prototype adopted the sidecar pattern broadly, but only partially realized the planned large-file streaming merge path.

## Decision Drivers

### Licensing hygiene

`@conset-pdf/core` is published as MIT and V4 explicitly requires a permissive engine dependency graph. The sidecar pattern kept Python-only dependencies out of the Node package dependency graph and made the boundary explicit instead of burying it inside the core library API.

### Capability gap in the TypeScript stack

- PDF.js extraction did not provide sufficiently reliable bounding boxes for geometry-heavy analysis
- `pdf-lib` did not provide the bookmark-writing reliability required across Bluebeam, PDF-XChange, Foxit, browser viewers, and Adobe

### Replaceability for Rust

Because the boundary is process-and-contract based rather than deeply embedded in business logic, the Rust rewrite can replace each sidecar with a native subsystem while preserving higher-level workflow semantics.

### Operational containment

The sidecar boundary localizes installation checks, subprocess errors, temp file handling, and packaging concerns to a few adapter files instead of leaking them across the whole codebase.

## Consequences

### Positive

- unlocked PyMuPDF extraction quality without rewriting the whole engine
- unlocked pikepdf/QPDF bookmark output with stronger viewer compatibility
- kept workflow engine, parser logic, and contracts in TypeScript
- made future replacement with Rust-native PDF subsystems straightforward at the interface level
- allowed Electron packaging to treat sidecars as copied runtime assets, including `.asar.unpacked` handling

### Negative

- requires Python discovery at runtime (`python`, `py`, `python3` search)
- creates version-coupling between TypeScript adapters and sidecar scripts
- adds process startup overhead and temp-file churn for each invocation
- adds a second dependency installation surface (`pip install pymupdf`, `pip install pikepdf`)
- complicates packaging and support because failures can come from interpreter lookup, import errors, script path issues, or sidecar output validation

## Known Issues

- process startup overhead is paid on each sidecar call
- dependency installation is manual in the prototype and surfaced via runtime errors rather than bundled distribution
- sidecar assets must be copied into `dist/` and specially resolved when running inside Electron ASAR packaging
- merge-sidecar streaming is incomplete in current code: final output is rewritten through pikepdf, but page assembly still happens in-memory via `pdf-lib`
- the documentation set overstates merge-sidecar maturity in some places; code is the canonical source for current behavior

## Alternatives Considered or Implicitly Rejected

### Stay entirely in TypeScript

Rejected for the affected operations because the available JS stack did not meet extraction accuracy and bookmark reliability requirements.

### Bind Python libraries directly into the Node package surface

Rejected because it would blur licensing, packaging, and runtime boundaries and make later replacement harder.

### Wait for Rust before solving the problem

Rejected because the prototype still needed working extraction and bookmark output in v3.

## Rust Replacement Direction

The Rust rewrite should preserve the architectural intent but remove the Python runtime entirely.

Target replacements called out by the plan:

- transcript extraction: PDFium-based native extraction
- structure manipulation and bookmarks: `lopdf`, `pdf-rs`, or equivalent native PDF object tooling
- large-file merge: true native streaming merge from day one, not in-memory assembly plus post-write passthrough

The behavior to preserve is not "Python sidecars" themselves. The behavior to preserve is:

- accurate extraction
- reliable bookmark/object writing
- explicit failure boundaries
- swappable backend implementations behind stable workflow contracts

## Prototype-Specific Invariants Worth Carrying Forward

- subprocess/native backend boundaries must accept explicit, serializable inputs
- specialized PDF operations should be isolated behind narrow adapters
- missing backend dependencies must fail loudly with actionable diagnostics
- packaging concerns belong at the boundary layer, not inside workflow/business logic

## Source-of-Truth Notes

For this ADR, code was treated as canonical over planning docs where they diverged.

Most important divergence:

- transcript and bookmark sidecars are implemented and active
- large-file merge sidecar streaming is planned and referenced in docs, but current executable merge flow still uses in-memory `pdf-lib` assembly followed by pikepdf passthrough write