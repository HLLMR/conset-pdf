# Phase 06 - Edge Case Catalog

**Document Type**: Post-Mortem Edge Case Catalog  
**Date**: 2026-03-19

---

## Overview

This catalog documents known failure modes, edge cases, and behavioral boundaries encountered during prototype development and testing. Cases are organized by module. Where test evidence exists, it is cited. Rust migration must account for each category.

---

## Module: Drawing Sheet ID Detection

### EC-001: Blank or Missing Sheet ID

**Description**: A drawing sheet has no printed sheet ID, or the sheet ID is outside the ROI region.

**Symptom**: `NO_ID` issue emitted; row action defaults to `'keep'` or `'unmatched'` in merge.

**Known triggers**:
- General notes pages (may have no sheet number)
- Cover/index pages
- Continuation pages that share an ID with the primary sheet
- Title block with non-standard position (far corner, non-standard size)

**Current behavior**: Page is preserved in output (not dropped) with `status: 'error'`, `confidence: 0`. User must manually override or ignore.

**Rust note**: Soft-fail behavior is correct. Never drop pages with undetected IDs without user confirmation.

---

### EC-002: Duplicate Sheet IDs Within a Set

**Description**: Two or more pages share the same normalized sheet ID.

**Symptom**: `DUPLICATE` issue; highest-confidence detection wins for merge planning; lower-confidence entries get `action: 'conflict'`.

**Known triggers**:
- Addendum includes a revised sheet AND an original was not removed from the addendum PDF
- Multiple addenda both include the same sheet (latest-wins expected but may be misordered)
- Data entry error: same sheet number used for two distinct sheets

**Current behavior**: Conflict flagged; highest confidence wins. No automatic resolution.

**Rust note**: The "highest confidence wins" heuristic should be documented as a tie-break, not a correctness guarantee. Explicitly model addendum sequence ordering to prefer later-in-sequence for duplicate resolution.

---

### EC-003: Non-Standard Sheet ID Formats

**Description**: Sheet ID format not covered by `drawingsSheetId.ts` pattern battery.

**Known formats that are covered**: `A-101`, `A101`, `M1-01`, `T-1.4A`, `FP-001`, `DDC-101`

**Known formats that fail**:
- All-numeric IDs (e.g., `101`, `1001`) — no discipline prefix
- Hybrid formats (e.g., `A1/01`, `A.1.01`) — atypical separators
- Non-Western character sets in IDs
- IDs embedded in complex multi-line title block text with no clean extraction boundary

**Current behavior**: Parser returns null match → `NO_ID` or `LOW_CONFIDENCE`.

**Rust note**: Extend pattern battery iteratively as new formats are encountered. Never remove a pattern that was added for a real project.

---

### EC-004: Very Thin Title Blocks

**Description**: Title block occupies < 2% of page height/width; ROI bounding box excludes legitimate spans.

**Symptom**: `ROI_DETECTION_FAILURE` or `NO_ID` for pages that have readable IDs.

**Current behavior**: Falls back to `LegacyTitleblockLocator` if enabled.

**Rust note**: Consider allowing minor ROI expansion (configurable tolerance) before falling back.

---

### EC-005: Rotated Pages

**Description**: Pages with non-zero rotation (90°, 180°, 270°) have coordinate systems that differ from portrait convention.

**Current behavior**: `transcript/canonicalize.ts` normalizes all spans to rotation=0 basis before transcript is cached. ROI profiles operate on the normalized coordinate space. This should be transparent.

**Known risk**: Pages where rotation is embedded in the page stream (not the `/Rotate` key) may not be normalized. This is a known gap; the canonicalize step uses `/Rotate` metadata.

**Rust note**: Test with PDFs where rotation is in the page stream vs. `/Rotate` key. Handle both paths.

---

## Module: Specs Section Detection

### EC-006: Unnumbered Cover Sections

**Description**: A spec section begins with a non-numbered cover page (project title, table of contents, etc.) that has no section ID in the footer or heading.

**Symptom**: Cover page gets `NO_ID`; may be split into Division-00 group or dropped from output.

**Known triggers**: Division 00 (procurement and contracting requirements) frequently begins with unnumbered administrative pages.

**Current behavior**: Pages before first detected section are assigned to an implicit leading group.

**Rust note**: Division 00 should be a first-class group; always produce a Division 00 output even if it only contains unnumbered pages.

---

### EC-007: Legacy 5-Digit Section IDs in Modern Books

**Description**: A specification book contains a mix of legacy 5-digit IDs (e.g., `16000`) and modern 6-digit IDs (e.g., `26 00 00`).

**Symptom**: Grouping may place sections in wrong divisions if normalization is inconsistent.

**Evidence**: `tests/workflows/split-legacy.test.ts` (explicit regression test).

**Current behavior**: Dual-format normalization in `parser/specsSectionId.ts` handles both. Grouping by division is stable when IDs are normalized to canonical form before grouping.

**Rust note**: Port both normalization paths verbatim. The dual-format test fixture must pass.

---

### EC-008: Section ID in Header vs. Footer

**Description**: Section ID appears only in the page header, not the footer (or vice versa).

**Section anchoring impact**: Footer-First Section Anchoring algorithm looks in the footer band first. If the section ID is in the header only, the algorithm falls back to heading-based resolution, which may be less accurate.

**Known triggers**: Some spec templates print section ID in header and page title in footer (reversed from typical CSI convention).

**Rust note**: Footer-first is the preferred pattern; heading-only fallback is needed but should be clearly logged in the audit output so users can identify affected sections.

---

### EC-009: Multi-Column Footers

**Description**: Footer band contains two or more columns of information (e.g., left: section ID, center: project name, right: page number).

**Symptom**: Footer text extraction may merge columns if multi-span assembly is overly aggressive.

**Current behavior**: Transcript extraction returns spans with individual bounding boxes; footer band filtering by Y position should keep all spans. Multi-column reassembly is a text concatenation risk in `readingOrder.ts`.

**Rust note**: Preserve header/footer span-level bounding boxes in the transcript. Do not pre-concatenate footer spans across column boundaries.

---

## Module: Specs Extraction (Chrome Removal)

### EC-010: Revision Clouds and Stamps Misclassified as Chrome

**Description**: Revision markup (clouds, delta triangles, stamp text) appears at consistent Y positions across pages and gets misclassified as repeating chrome.

**Symptom**: Revision markup removed from extracted AST; patches applied to wrong nodes.

**Current behavior**: Content repetition threshold (≥50% of pages) should exclude single-page stamps, but revision clouds that appear on all addendum revision pages can trigger false classification.

**Rust note**: Chrome detection threshold should be configurable. Consider a whitelist approach: only classify chrome what matches known patterns (section ID, date, project name) rather than any repeating content.

---

### EC-011: Watermarks at Mid-Page

**Description**: PDF contains a diagonal watermark (e.g., "DRAFT", "ISSUED FOR BID") at a consistent mid-page position.

**Symptom**: Watermark may be misclassified as chrome if it appears at a consistent Y position; body text spans may be excluded from extraction.

**Current behavior**: Y-band-based chrome removal (top 12%, bottom 12%) should protect mid-page body content. Watermarks outside those bands should not be suppressed. If watermark appears at consistent > 50% frequency and within the band range, false positive can occur.

**Rust note**: Mid-page watermark handling requires a separate detection pass (diagonal text pattern, low opacity, high font size, centered layout) rather than relying solely on repetition+Y-band heuristics.

---

## Module: Merge Planning

### EC-012: Cover Pages Without IDs

**Description**: Original PDF begins with a cover page (title page, TOC, etc.) that has no detectable sheet ID.

**Symptom**: Cover page may get `NO_ID`; merge planner must not treat it as an unmatched addendum page.

**Current behavior**: Cover pages in the original PDF are kept verbatim (`action: 'keep'`). Cover pages from addenda are appended only in `append-only` mode.

**Rust note**: Preserve cover page handling as-is. Never inject addendum cover pages into a merged set that already has a cover page, unless `append-only` mode is selected.

---

### EC-013: Addendum-Only Sheets (No Matching Original)

**Description**: An addendum introduces a completely new sheet (ID not present in original). In `replace-only` mode, this sheet is dropped.

**Symptom**: In `replace+insert` mode, the sheet is correctly inserted at the lexicographic position for its discipline. In `replace-only` mode, user sees `UNMATCHED` warning and sheet is not included.

**Current behavior**: Mode semantics are enforced correctly. `UNMATCHED` warning is emitted.

**Rust note**: Log insertion position calculations for debugging. Include the computed insertion page index in the audit output.

---

### EC-014: Large File OOM

**Description**: Input PDFs exceed available Node.js heap (>500MB combined) causing crash during in-memory pdf-lib assembly.

**Symptom**: Process crash or heap allocation failure during execute phase.

**Current behavior**: No soft-fail; crash with Node.js OOM message.

**Evidence**: Referenced in ADR-004 (`largeFileRefactorPlan.md`) as the trigger for disk-streaming merge design.

**Rust note**: This failure mode must not exist in Rust. Implement streaming merge that bounds memory to a fixed page decode buffer (e.g., 50MB), regardless of total document size.

---

## Module: Bookmarks

### EC-015: Bookmark Destination Page Mismatch After Merge

**Description**: Existing bookmarks reference page numbers that shift after merge operations (new pages inserted before them).

**Symptom**: Bookmarks land on wrong pages post-merge; validated bookmarks show `BOOKMARK_PAGE_HINT_MISMATCH`.

**Evidence**: `packages/core/src/bookmarks/tests/bookmarkPageMapping.test.ts` and `bookmarkDestinations.test.ts` are regression guards for this case.

**Current behavior**: `regenerateBookmarks: true` option in merge execute triggers a full bookmark rebuild after merge, replacing old bookmarks.

**Rust note**: Never preserve old bookmarks after a merge that inserts pages. Always regenerate. The test suites must be ported to Rust verbatim.

---

### EC-016: Bookmarks Display Failure in PDF-XChange

**Description**: Bookmarks written as inline PDF dictionary objects (rather than indirect objects) fail to display in PDF-XChange Viewer.

**Evidence**: `packages/core/src/bookmarks/tests/bookmarkViewerCompatibility.test.ts`

**Current behavior**: pikepdf sidecar uses `pdf.make_indirect()` to ensure indirect objects. Post-write validation traverses the outline chain to confirm indirect-object compliance.

**Rust note**: This is a Rust-critical requirement. Native PDF library must support creating indirect objects. Test bookmarks in PDF-XChange from day one.

---

### EC-017: Anchor Not Found for Spec Section Bookmark

**Description**: A `BookmarkAnchorTree` section anchor (e.g., `23 05 53`) cannot be matched to any page in the target PDF via footer-first or heading-based resolution.

**Symptom**: `BOOKMARK_ANCHOR_NOT_FOUND` issue; section is omitted from bookmark output or lands at `pageIndexHint` (which may be wrong).

**Known triggers**:
- Section was removed from the PDF but still exists in the `BookmarkAnchorTree`
- Footer text format changed between extraction and bookmark generation (different spec revision)
- Section ID printed in non-standard format in footer (legacy vs. modern format mismatch)

**Current behavior**: Issue flagged; `--allow-invalid-destinations` flag bypasses validation gate.

**Rust note**: Emit clear diagnostic with matched/unmatched anchor details. Consider fuzzy matching (normalized ID comparison) as a fallback before declaring anchor not found.

---

## Module: Narrative PDF Validation

### EC-018: Narrative File Not Found

**Description**: The `narrativePdfPath` provided to analyze does not exist or is not readable.

**Symptom**: Narrative processing is silently skipped; `InventoryResult.narrative` is undefined.

**Evidence**: `tests/workflows/merge-narrative.test.ts` covers this case.

**Current behavior**: Soft-fail; narrative is omitted from result.

**Rust note**: Distinguish between "no narrative provided" (expected, no warning) and "provided path not found" (warning). Emit a warning in the second case so users notice the skipped validation.

---

### EC-019: Near-Match Narrative References

**Description**: Narrative says "Sheet A-101 was revised" but the addendum PDF contains `A-100` and `A-102` (A-101 absent).

**Symptom**: `NARR_NEAR_MATCH` issue with suggestion: "Did you mean A-100?"

**Evidence**: Near-match algo uses normalized string distance (`parser/normalize.ts`).

**Current behavior**: Advisory; user sees suggestion but must manually confirm. Never auto-applies.

**Rust note**: Preserve near-match suggestion behavior. The threshold for "near match" should be a configurable parameter (currently hardcoded).

---

## Module: Transcript Quality

### EC-020: Zero-Text Pages

**Description**: PDF page contains no extractable text (scanned image, blank page, or pure vector drawings).

**Symptom**: `quality.ts` emits low quality score (0 characters); transcript quality gate fails.

**Current behavior**: Low-quality page is included in transcript with quality metrics. Downstream consumers see empty text items.

**Rust note**: Zero-text page detection should be explicit and logged. These pages should not cause detection pipeline failures — they should emit `NO_ID` cleanly.

---

### EC-021: PDF.js Fallback Coordinate Variance

**Description**: When PyMuPDF sidecar is unavailable, PDF.js fallback is used. Bounding box coordinates from PDF.js are less precise.

**Symptom**: ROI filtering with PDF.js coordinates may include/exclude slightly different spans than PyMuPDF.

**Evidence**: `tests/transcript/determinism.test.ts` explicitly relaxes coordinate equality requirements for PDF.js path.

**Current behavior**: Non-deterministic on fallback path. Known limitation.

**Rust note**: PDFium extraction must provide PyMuPDF-equivalent precision. There should be no fallback-path determinism compromise in Rust.

---

## Module: Standards Normalization

### EC-022: Ambiguous Discipline Designator `C`

**Description**: The designator `C` is used for both Civil and Controls/DDC in different firms.

**Symptom**: `normalizeDrawingsDiscipline` returns confidence < 1.0; disambiguation heuristic applied.

**Current behavior**: Context heuristic uses surrounding sheet IDs to infer likely discipline. Low confidence flagged in `DrawingsDisciplineMeta`.

**Rust note**: Port disambiguation heuristic. Consider making the heuristic configurable per layout profile (a profile could declare "in this project, C = Controls") to eliminate ambiguity.

---

### EC-023: Legacy 5-Digit MasterFormat Codes

**Description**: Pre-2004 CSI MasterFormat used 5-digit section numbers (e.g., `16000` for Electrical). These appear in older project specs.

**Current behavior**: `normalizeSpecsMasterformat` handles legacy codes with explicit mapping table.

**Evidence**: `standards/datasets/masterformatDivisions.ts` contains legacy mapping entries.

**Rust note**: Port legacy mapping table verbatim. Dual-format support is required.

---

## Fixtures and Test Artifacts

**Primary test suite directories**:
- `conset-pdf/tests/workflows/` — merge, split, narrative regression tests
- `conset-pdf/packages/core/src/bookmarks/tests/` — bookmark destination and viewer compatibility
- `conset-pdf/packages/core/src/specs/tests/` — specs extraction unit tests
- `conset-pdf/tests/transcript/` — determinism and quality scoring tests
- `conset-pdf/tests/standards/` — UDS and MasterFormat normalization tests

**Primary run artifact directory**:
- `conset-pdf/test-output/23_MECH_FULL_fresh_run/` — real extraction output from MECH spec book

All Rust migration tests should start from these fixtures. Do not create new synthetic fixtures when real PDF test data is available.
