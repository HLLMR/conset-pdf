# Phase 06 - Non-Negotiables as Rust Design Constraints

**Document Type**: Post-Mortem Rust Design Constraints  
**Source**: V4 Master Plan Non-Negotiables (20 items), reframed as concrete Rust implementation requirements  
**Date**: 2026-03-19

---

## Purpose

This document translates each of the 20 V4 Master Plan non-negotiables into a concrete Rust design constraint or implementation requirement. For each constraint, this document also records:

- **Prototype proof**: Did the prototype implement this, and was it validated at scale?
- **Prototype gap**: Where did the prototype fall short?
- **Rust requirement**: What must the Rust implementation do differently or better?

---

## NN-01: Determinism is Sacred

> Same input + same profile + same engine version = identical output. No runtime randomness.

**Prototype proof**: Strong. Transcript canonicalization enforces stable sort, rotation normalization, and hash exclusion of timestamps. Determinism test suite (`tests/transcript/determinism.test.ts`) validates hash stability across repeated runs on the PyMuPDF path.

**Prototype gap**: Determinism is **path-conditional**: PDF.js fallback path explicitly tolerates coordinate variance (tests use relaxed equality). This means the determinism guarantee is not uniform across all extraction paths. See SOT-006 and R-008.

**Rust requirement**:
- All extraction paths (PDFium primary; any future fallback) must produce identical transcripts for identical inputs
- No `HashMap` with non-deterministic iteration order in any pipeline
- No `f64` coordinate comparisons without explicit normalization to fixed precision (e.g., round to 2 decimal places, in pts)
- Canonical sort must use a total ordering with no undefined tie-break cases
- Content hash must include the same fields on all extraction paths (no path-conditional hash contracts)
- Test: run identical input through every extraction path and assert byte-equal transcript hashes

---

## NN-02: Do It Right the First Time

> Architecture decisions optimize for long-term correctness and maintainability, not short-term demos.

**Prototype proof**: Partially demonstrated. The three-phase workflow engine, profile-driven detection, and transcript system are all "right first time" designs that held up throughout development.

**Prototype gap**: Several corners were cut that required later redesign: in-memory merge (required disk-streaming redesign), pdf-lib bookmarks (required pikepdf sidecar), PDF.js as primary extractor (required PyMuPDF migration). Each shortcut added weeks of remediation.

**Rust requirement**:
- Disk-streaming merge from day one (NN-12)
- Native PDF library producing indirect-object bookmarks from day one
- PDFium extraction from day one (no fallback that reduces determinism)
- Complete type system: no `any` equivalents, no untyped JSON blobs as inter-module contracts; use `serde`-annotated structs with explicit versioning for all inter-process data
- No `unwrap()` in production paths — all error cases must be handled or surfaced with context

---

## NN-03: Spec Section Reflow & Reconstruction

> Reflow & reconstruction to editable structure is a core moat feature.

**Prototype proof**: Partially demonstrated. The specs-patch workflow extracted `SpecDoc` AST and applied patch operations. Section-only regeneration (not whole-book) was the explicit design. The AST extraction pipeline was validated on real spec PDFs.

**Prototype gap**: The Playwright rendering dependency made deployment impractical. The workflow was subsequently abandoned. The Rust reimplementation must restore this capability with a native rendering path.

**Rust requirement**:
- Implement `SpecDoc` AST extraction as a production-grade Rust pipeline (not deferred)
- Replace Playwright with a native PDF generation path (options: wkhtmltopdf shim, headless Chromium via headless_chrome, or purpose-built CSI spec formatter using printpdf/lopdf)
- Section-only regeneration invariant: patched sections produce a PDF of that section only; whole-book regeneration is a separate, opt-in operation
- `BookmarkAnchorTree` must be generated as a first-class output of all spec extraction/rendering paths
- Test: round-trip test — extract AST → apply no-op patch → render → extract again → assert structural equivalence

---

## NN-04: Chrome/Furniture is Explicitly Modeled

> Chrome must be detected, modeled, and excluded explicitly.

**Prototype proof**: Demonstrated for specs (chromeRemoval.ts with repetition analysis + Y-band clustering) and drawings (ROI-based title block isolation). Chrome detection is per-page and logged.

**Prototype gap**: Chrome detection thresholds are hardcoded (top 12%, bottom 12%, repetition ≥50%). These values were tuned on available test PDFs but not validated against a broader corpus.

**Rust requirement**:
- All chrome detection thresholds must be configurable parameters (as profile fields or engine options), not hardcoded constants
- Chrome classification must produce a structured output (list of classified spans with reasons) that appears in the audit trail
- False positive detection capability: allow users to whitelist specific recurring content as "not chrome"
- Test: run chrome detection on the MECH test corpus and verify no content spans are removed from extracted sections

---

## NN-05: Per-Medium Chrome Detectors

> Drawings ≠ Specs ≠ Submittals. Separate detectors/handlers per type.

**Prototype proof**: Demonstrated. Drawings use ROI-based profile filtering; specs use Y-band + repetition analysis; submittals have a separate parser stub with its own structural model.

**Prototype gap**: Submittals chrome detector is not implemented; only parser stub exists.

**Rust requirement**:
- `drawings::chrome`, `specs::chrome`, `submittals::chrome` are separate Rust modules with no shared implementation
- Each medium's chrome detector exposes the same trait interface for pipeline composition, but has distinct algorithm internals
- Submittal chrome detection must be built before the submittal workflow is considered complete

---

## NN-06: Chrome Metadata Preserved and Reused

> Headers/footers contain critical project information that must be extracted, stored, and reapplied to regenerated content.

**Prototype proof**: Partially demonstrated. Spec footer parsing (`footerSectionIdParser.ts`) extracts section IDs from footer text. Chrome removal preserves the extracted metadata before suppression.

**Prototype gap**: Full chrome metadata extraction (project name, firm, date, project number from headers/footers) is not implemented. The footer section ID parser is Division-23 specific. Metadata reapplication to regenerated content (HTML → PDF rendering) is not implemented.

**Rust requirement**:
- Chrome metadata schema: define a `ChromeMetadata` struct with typed fields for section ID, page number, project number, date, section title
- Footer parser must cover all MasterFormat divisions (not only Division 23)
- `ChromeMetadata` must be includeded in audit output per page
- When rendering regenerated specs, `ChromeMetadata` provides the footer/header template data
- Test: extract footer metadata from a real spec PDF footer; assert section ID, page counter, and date are correctly parsed

---

## NN-07: Audit Trail is First-Class

> Every run emits an explainable artifact bundle with overlays + decisions.

**Prototype proof**: Partially demonstrated. JSON audit reports (detection confidence, action taken, issues) are emitted by all workflow execute phases. The report schema is documented in Phase 4 output formats contract.

**Prototype gap**: Visual overlays (highlighting detected spans, ROI bounding boxes, band classifications on rendered page images) are not implemented. Audit bundles are flat JSON, not structured artifact bundles. The `reportPath` option and report content are under-specified.

**Rust requirement**:
- Define a formal `AuditBundle` struct with: detected rows (with confidence, source, coordinates), issues (with evidence spans), decisions (which plan action, why), timing metrics, engine version, input checksums
- Audit bundle format must be versioned (include `schema_version` field)
- Implement visual overlay generation: for each page, render a PNG/SVG of detected spans, ROI regions, band boundaries, and issue highlights
- Overlay images must be included in the audit bundle (as base64 or file refs)
- Test: after every merge/split/bookmark operation, assert the audit bundle contains at least one entry per modified page

---

## NN-08: Specs Section-Only Regeneration

> Never regenerate whole books unless forced.

**Prototype proof**: Demonstrated as design intent in specs-patch workflow. Section granularity is preserved through the AST model.

**Prototype gap**: Workflow was abandoned before production use. Section-only scoping was never tested at scale on full spec books (50+ sections).

**Rust requirement**:
- `spec_patch::execute()` must accept a list of target section IDs; processing is bounded to those sections
- Whole-book mode is a separate, opt-in operation that requires an explicit flag
- Performance target: single-section patch on a 300-page spec book completes in <10s
- Output PDF is a single section (not the full book), unless whole-book mode is selected

---

## NN-09: Drawings/Submittals: Extraction and Organization, Not Re-Typesetting

> Extract and organize; do not re-render or reflow drawings/submittals.

**Prototype proof**: Fully demonstrated. Merge and split workflows operate on page streams verbatim; no re-encoding or re-rendering of drawing content.

**Prototype gap**: None significant.

**Rust requirement**:
- Drawing page content streams must be passed through without decoding or re-encoding
- The only allowed mutations to drawing pages: PDF container structure (page tree, cross-references), bookmark writing, and report metadata attachment
- Never use Chromium or any renderer for drawing PDF output

---

## NN-10: No Python Runtime for End Users

> Single Rust binary only.

**Prototype proof**: Failed. The prototype requires Python for PyMuPDF extraction and pikepdf bookmark writing. End users must install Python 3.10+ and the sidecar requirements.

**Prototype gap**: This is the defining architectural shortcoming of the prototype. The Python sidecar pattern was a deliberate workaround for licensing constraints (Apache-2.0 vs. LGPL). Rust eliminates both problems.

**Rust requirement**:
- No subprocess spawning of any script language (Python, Node.js, etc.) in production paths
- Extraction: use `pdfium-render` Rust crate (PDFium is Apache-2.0)
- Bookmark writing: implement native indirect-object outline via `lopdf` or `pdf-rs`
- PDF merge/passthrough: implement via native Rust PDF stream handling (no pikepdf)
- Distribution: a single static Rust binary with PDFium statically linked or provided as a single companion `.so`/`.dll`
- Test: end-to-end workflow must complete on a no-Python system (CI runs on a no-Python Docker image)

---

## NN-11: Licensing Hygiene

> Engine stays Apache-2.0 or MIT. No GPL/AGPL in core dependency graph.

**Prototype proof**: Maintained in prototype for the TypeScript library. Python sidecars are isolated by process boundary (not at link time), which satisfies the LGPL intent.

**Rust requirement**:
- Audit all Rust crate dependencies via `cargo-license` before each release
- Reject any crate with GPL/AGPL license from `crates/core-engine` dependency graph
- PDFium: Apache-2.0 ✅
- `lopdf`: MIT ✅
- `serde`, `regex`, `tokio`: MIT/Apache-2.0 ✅
- License audit must be part of CI (gate on FAIL if any GPL-licensed crate is detected)

---

## NN-12: Unchanged Pages are Verbatim Copies

> Pages outside the impacted section/sheet must be byte-for-byte copies where possible.

**Prototype proof**: Demonstrated in prototype design. The merge planner explicitly marks unchanged pages as `'keep'`; `applyPlan.ts` uses `pdf-lib`'s `copyPages()` which does not re-encode page streams.

**Prototype gap**: The pikepdf passthrough write step may re-normalize PDF containers, defeating strict byte-equality at the file level. The guarantee is strongest for page stream content; container-level byte equality is not proven.

**Rust requirement**:
- Unchanged page streams (content streams) must be passed through byte-identical
- Do not re-encode, re-compress, or apply any transform to unchanged page streams
- PDF container normalization (cross-reference table, object IDs) is acceptable — but page content streams must not be touched
- Implement a test: copy a page from a real PDF using the merge path; assert `sha256(original_page_stream) == sha256(output_page_stream)`
- For specs patch: only sections explicitly targeted by a patch operation may be regenerated; all other sections must use the original page streams

---

## NN-13: PDF as Hostile Input

> Crash containment, memory caps, soft fails.

**Prototype proof**: Partially demonstrated. `DocumentContext` initialization can fail gracefully; workflow analyze returns error issues rather than throwing. Large-file OOM is not contained (crash on heap exhaustion).

**Prototype gap**: No explicit memory caps. Large PDF inputs crash Node.js process. No sandbox/signal-safe handling for malformed PDFs.

**Rust requirement**:
- Set explicit memory limits for any in-memory operation (configurable, default 256MB per document)
- Use `catch_unwind` at the workflow entry point to prevent panics from propagating to the CLI/GUI
- On malformed PDF (corrupt stream, invalid xref): emit structured error with page range context; continue processing remaining pages
- PDFium handles most malformed PDF cases gracefully — rely on it as the first line of defense
- Memory accounting: track per-document allocation, emit warning if approaching cap (80%), abort with soft-fail if at cap (100%)
- Test: run workflows against intentionally malformed PDFs (truncated, invalid xref, circular object references) and assert no panic/crash

---

## NN-14: No Silent Failures

> Low confidence → emit "Needs Review" with visual evidence.

**Prototype proof**: Demonstrated. Detection below confidence threshold emits `LOW_CONFIDENCE` issue; pages with no detection emit `NO_ID`. GUI presents these as yellow/red rows requiring review.

**Prototype gap**: "Visual evidence" is not provided. The audit report names the issue but does not provide a page image or span highlight showing why confidence is low.

**Rust requirement**:
- Every issue must include: `page_index`, `confidence`, `evidence` (list of detected spans with bounding boxes), and `reason` (human-readable explanation)
- Visual audit overlay (NN-07) must highlight low-confidence detections
- No issue may be silently absorbed; if a detection fails, it must produce either a result with explicit low-confidence flag or an explicit `NO_ID` result — never a missing row

---

## NN-15: Partial Truth Over Null

> Return grounded partial structure when full certainty is impossible.

**Prototype proof**: Demonstrated. `InventoryResult` includes all rows including low-confidence and error rows. `SpecDoc` AST includes nodes with `null` anchors. Merge proceeds with partial inventory.

**Rust requirement**:
- No operation may return `null`/`None` for an entire document result when partial extraction succeeded
- `LayoutTranscript` for a page is always emitted, even if quality score is 0 (page with no text returns an empty page transcript, not null)
- `SpecDoc` extraction returns sections even when some anchors are ambiguous (ambiguous sections are flagged, not omitted)
- Test: run extraction on a PDF where page 5 is corrupt; assert pages 1-4 and 6+ still appear in the transcript with valid quality scores

---

## NN-16: Tests First

> Every function has a test. Every phase has integration tests against torture corpus.

**Prototype proof**: Partially demonstrated. Comprehensive unit tests for standards normalization, parser, transcript canonicalization, determinism, and bookmarks. Integration tests for merge/split/narrative workflows.

**Prototype gap**: No torture corpus for extraction edge cases (EC-001 through EC-023 in this catalog are partially covered but not systematically). No property-based tests for ID normalization.

**Rust requirement**:
- All regex patterns in `drawingsSheetId.ts` equivalents must have unit tests with positive and negative examples for each pattern
- Every calibrated threshold (chrome detection percentages, confidence thresholds) must have a test that exercises the threshold boundary
- Build a corpus test harness: given a directory of real PDF inputs + expected output fixtures, run all workflows and assert JSON output matches
- Integration test CI must run on a no-Python system to validate NN-10
- Property-based tests (`proptest` crate) for ID parsing: verify that normalized output of any valid input is in canonical form

---

## NN-17: Partial Success is Full Success

> If 80/100 sections process correctly, output those 80 and ask user about the 20 failures. Never discard working results.

**Prototype proof**: Demonstrated. Merge workflow produces output PDF even when some pages have `NO_ID`. Specs extraction produces partial AST when some sections fail. Issues are reported per-page, not per-document.

**Rust requirement**:
- All workflow execute phases must produce file output files for all successfully processed pages/sections, even when some fail
- Each failure must be captured in the `errors` field of `ExecuteResult` with the specific identity of the failed item (page index, section ID)
- CLI must exit with a non-zero status only when ALL pages/sections fail, not when some fail
- GUI must present partial results and offer a "handle failures" flow, not a binary pass/fail dialog
- Test: run merge with one deliberately un-parseable addendum page; assert that output PDF contains all parseable pages

---

## NN-18: Medium Detection is User-Driven

> GUI enforces context through workflow-based file pickers. CLI requires explicit operation flags. No auto-detection.

**Prototype proof**: Fully demonstrated. GUI uses separate wizards per document type. CLI requires `--doc-type drawings|specs` on all commands. No code path auto-infers document type from PDF content.

**Rust requirement**:
- `docType: drawings | specs | submittals` is a required parameter on all workflow entry points
- No inference of document type from content (title block text, footer patterns, etc.) may substitute for an explicit user declaration
- GUI: separate view/wizard per document type; no shared "detect and handle" flow
- CLI: commands must fail with clear error if `--doc-type` is omitted

---

## NN-19: Accuracy Over Visual Fidelity

> For spec regeneration: textual accuracy 100% required; visual fidelity best-effort.

**Prototype proof**: Partially demonstrated in design intent. The HTML/CSS rendering path prioritized structure preservation over pixel-perfect visual output. In practice, Playwright rendering on real spec PDFs was visually reasonable.

**Prototype gap**: "Textual accuracy 100%" was not formally tested. No regeneration round-trip test was implemented.

**Rust requirement**:
- Implement a round-trip accuracy test: extract spec section AST → render to PDF → extract text from rendered PDF → assert 100% text content match with original
- Visual fidelity test (optional): render section → compare visual layout to reference; flag differences but do not fail on minor visual variations
- Spec template CSS must be maintained as a versioned artifact; visual regressions in rendered output must be logged with each template version change

---

## NN-20: Pattern Development Tool is Infrastructure

> Must be built in Phase 0.5. Critical development dependency for all pattern-based work.

**Prototype proof**: Failed. No Pattern Development Tool was built. All pattern development was done via direct code editing and test output inspection.

**Prototype gap**: Without the tool, every ROI profile field and extraction threshold required full pipeline runs to validate. This made profile iteration slow and error-prone.

**Rust requirement**:
- Build a Pattern Development Tool before any production profile development begins
- Minimum viable tool:
  1. Load a PDF and display a page with rendered page (raster image)
  2. Overlay extracted spans with bounding boxes (colored by type: chrome, content, heading, footer)
  3. Allow user to draw ROI bounding boxes and preview extraction results in real-time
  4. Save/load profile JSON directly from the tool
- Extended tool features (Phase 1):
  1. Show quality score per page
  2. Show chrome removal classification decisions
  3. Show confidence scores per detected ID
  4. Batch validation: run a profile against a directory of PDFs and show hit/miss rates
- The Pattern Development Tool must be part of the open-source CLI distribution (it is developer infrastructure, not a GUI-only premium feature)

---

## Constraint Implementation Summary

| NN | Constraint | Prototype Status | Rust Priority |
|---|---|---|---|
| 01 | Determinism | Partial (path-conditional) | P0 |
| 02 | Do It Right | Partial (shortcuts taken) | P0 |
| 03 | Spec Reflow | Partial (Playwright dep, abandoned) | P0 |
| 04 | Chrome Modeling | Partial (thresholds hardcoded) | P1 |
| 05 | Per-Medium Chrome | Partial (submittals missing) | P1 |
| 06 | Chrome Metadata Reuse | Partial (extraction only, no reapplication) | P1 |
| 07 | Audit Trail | Partial (JSON only, no visual overlay) | P1 |
| 08 | Section-Only Regen | Not validated at scale | P0 |
| 09 | No Drawing Re-Render | Complete | P2 |
| 10 | No Python Runtime | Failed (Python required) | P0 |
| 11 | Licensing Hygiene | Complete (prototype) | P0 |
| 12 | Byte-Verbatim Pages | Partial (pikepdf may normalize) | P0 |
| 13 | Hostile Input | Partial (no memory caps, OOM) | P0 |
| 14 | No Silent Failures | Partial (no visual evidence) | P1 |
| 15 | Partial Truth | Complete | P2 |
| 16 | Tests First | Partial (torture corpus missing) | P1 |
| 17 | Partial Success | Complete | P2 |
| 18 | User-Driven Medium | Complete | P2 |
| 19 | Accuracy Over Fidelity | Not formally tested | P1 |
| 20 | Pattern Dev Tool | Failed (not built) | P0 |

**P0**: Must be implemented in Rust Phase 1 before any production use  
**P1**: Required for production but can follow P0 items  
**P2**: Maintained behavior from prototype; no redesign needed
