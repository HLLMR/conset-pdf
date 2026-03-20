# Phase 06 - Lessons Learned

**Document Type**: Post-Mortem Lessons Learned  
**Date**: 2026-03-19

---

## Overview

This document captures what worked well, what failed or was abandoned, and what was deferred with rationale, in the conset-pdf prototype. It is organized for the Rust implementation team: what to replicate, what to replace, and what to reconsider.

---

## Section 1: What Worked

### 1.1 ROI Profile-Driven Detection

**What**: Explicit JSON layout profiles that define named bounding box regions (ROIs) relative to page dimensions, used to extract text for drawing sheet ID detection.

**Why it worked**:
- AEC title blocks are highly variable across firms, project types, and eras. Attempting to auto-detect title block layout produced too many false positives and near-misses across real project PDFs.
- ROI profiles allow field users to define exactly where the sheet ID appears in their specific title block, once, and never re-tune.
- ROI profiles compose naturally: a single profile covers an entire project set because all sheets in a project use the same title block template.
- Coordinate normalization to rotation=0 basis means profiles work across portrait/landscape/rotated sheets without per-sheet tuning.
- Profile registry and validation gates (required fields, coordinate range checks) surfaced misconfigured profiles early.

**Lessons**:
- Profile-driven detection is correct for AEC, where layout diversity is high but within-project layout is uniform.
- The profile format should be preserved verbatim in Rust.
- The one missing piece: profile auto-selection from PDF content, which was designed but not wired. Rust should implement this from the start.

---

### 1.2 Workflow Engine Pattern (analyze → applyCorrections → execute)

**What**: All user-facing operations follow a three-phase pattern: read-only analysis → user correction overlay → destructive execute.

**Why it worked**:
- AEC users need to review and approve detected results before any files are modified. The analyze phase gives them this review opportunity at zero risk.
- The correction overlay model is clean: stable row IDs survive corrections round-trips, and corrections are additive (ignore, override, rename)—never destructive to the inventory data.
- The pattern enabled consistent GUI design: every wizard follows the same stepflow regardless of which workflow it hosts.
- Test coverage is natural: analyze tests verify detection quality; execute tests verify PDF output correctness; corrections tests verify overlay application.

**Lessons**:
- This pattern must be preserved in Rust. It is not a UX sugar layer — it is a core audit-safety invariant.
- The prototype implementation fell short on plan reuse: `applyCorrections()` re-runs `analyze()` internally (R-007). Rust must implement true cursor state: corrections mutate prior analyzed state without re-detection.
- The separation between analyzed state and corrections state is the primary evolution driver; do not collapse these into a single phase.

---

### 1.3 Transcript System (V3 Extraction Architecture)

**What**: PyMuPDF-based extraction producing `LayoutTranscript` — a canonicalized, rotation-normalized, stably-sorted, hashable representation of all text spans in a PDF, cached per document.

**Why it worked**:
- PDF.js `getTextContent()` provides text content but unreliable bounding boxes. PyMuPDF's `page.get_text("rawdict")` provides accurate bounding boxes, span-level detail, font metrics, and color — all needed for geometric analysis (ROI filtering, column detection, chrome removal).
- The canonical hash (per-span content hash, per-page content hash) enables change detection and determinism verification.
- Stable sort (y-position primary, x-position secondary, content hash tertiary) eliminates extraction-order non-determinism.
- The `quality.ts` scoring layer provides per-page metrics (char count, whitespace ratio, replacement character ratio) that quantify extraction reliability.
- Caching at `DocumentContext` level means each PDF is extracted once; all downstream consumers work from the same cached transcript.

**Lessons**:
- The `LayoutTranscript` type contract should be preserved verbatim. It is the single most important shared type in the codebase.
- The rotation normalization step (applying rotation matrix to raw span coordinates before caching) is critical and must be reproduced in Rust.
- Quality scoring thresholds (per-page char counts, whitespace ratios) should be tunable parameters, not hardcoded constants.
- The quality-driven fallback (use lower-quality extractor only when primary fails on error, not on quality grounds) is a known limitation; Rust should implement quality-gate-driven switching.

---

### 1.4 Python Sidecar Decoupling

**What**: Isolating PyMuPDF (LGPL) and pikepdf/QPDF (LGPL) operations behind subprocess boundaries.

**Why it worked**:
- Licensing boundary is clean: Apache-2.0 library never touches LGPL code at link time.
- Python version isolation: sidecar scripts can pin their own library versions without constraining the Node.js runtime.
- The protocol (stdin/stdout JSON messages or file-path arguments) is simple and testable independently.
- Performance cost (spawn overhead ≈ 0.5–1.5s per operation) was acceptable for batch PDF workflows.

**Lessons**:
- The sidecar pattern proved the narrow boundary concept: only two operations needed Python (extraction + bookmark writing), and both have clear, replaceable Rust alternatives.
- Do not replicate this pattern in Rust — it only existed as an Apache-2.0 boundary workaround. Replace with native PDFium for extraction and lopdf/pdf-rs for writing.

---

### 1.5 pikepdf for Bookmark Writing

**What**: Using pikepdf/QPDF via Python sidecar to write PDF bookmarks as indirect objects.

**Why it worked**:
- `pdf-lib` (TypeScript) does not produce viewer-compatible bookmark structures out of the box. PDF viewers (especially PDF-XChange, dominant in AEC) require outline items to be indirect objects with proper `/Parent`-`/First`-`/Last`-`/Next`-`/Prev` chain.
- pikepdf's `make_indirect()` produces correct indirect objects; the resulting PDFs render correctly in all tested viewers.
- Post-write validation (re-read and traverse outline) provided a safety gate against silent write failures.

**Lessons**:
- The outline object structure requirements (indirect objects, bidirectional link chains, proper `/Count` values) are PDF spec requirements, not pikepdf-specific. Any Rust PDF library must satisfy these invariants.
- Post-write validation should be a built-in behavior of the bookmark writer, not an optional step. Rust should enforce this.
- The `bookmark-writer.py` sidecar script is the most brittle component: Python version drift, pikepdf version drift, and QPDF library updates have all caused compatibility issues. Native replacement is high priority.

---

### 1.6 Standards Normalization as Pure Functions

**What**: UDS discipline identification and CSI MasterFormat division classification implemented as pure functions over static lookup tables.

**Why it worked**:
- Stateless, side-effect-free functions are trivially testable (comprehensive test tables) and portable.
- Static lookup tables can be embedded in the binary (no external database).
- Deterministic sort order (defined in table, not computed) guarantees consistent output file ordering.
- Disambiguation heuristics (e.g., `C` = Civil vs. Controls) are explicit, documented rules — not implicit model behavior.

**Lessons**:
- These tables should be ported verbatim to Rust. They represent accumulated AEC domain knowledge.
- Any changes to discipline order or MasterFormat division assignments must go through a formal change process (they affect sort order and therefore output file names).

---

### 1.7 Narrative PDF Advisory Validation

**What**: Algorithmic parsing of addendum narrative PDFs to extract references to sheet/section revisions, validated against the detected inventory.

**Why it worked**:
- Real addendum packages include a narrative description letter that lists which sheets/sections were updated. Cross-referencing detected inventory against the narrative catches cases where the narrative says "A-101 updated" but no A-101 was found in the addendum PDFs.
- Advisory-only pattern (narrative results never modify detection results) was the right design: the narrative is often written informally and can contain typos. It is a human-language document, not a machine-readable manifest.
- Near-match suggestions (Levenshtein distance on normalized IDs) provide actionable correction hints without auto-applying corrections.

**Lessons**:
- The advisory-only constraint is correct and must be preserved. Narrative text is not authoritative — it is informational.
- The `NARR_INVENTORY_NOT_MENTIONED` code (sheet detected but not mentioned in narrative) is uniquely valuable: it catches sheets that may have been accidentally included in an addendum PDF.

---

## Section 2: What Failed

### 2.1 In-Memory PDF Assembly for Large Files

**What**: Using `pdf-lib`'s in-process page copy API to assemble merged PDFs.

**Why it failed**:
- `pdf-lib` loads full PDF page content into Node.js heap for assembly. Large drawing sets (>500MB) caused OOM crashes.
- The fix (disk-based streaming merge via pikepdf sidecar) was designed (ADR-004, `largeFileRefactorPlan.md`) but only partially implemented: pikepdf is used for passthrough writing, but page assembly still occurs in-memory.
- Performance for smaller PDFs was acceptable but the memory ceiling was unacceptable for production AEC use.

**Rust implication**: Implement streaming page-copy merge from day one. Never load full input PDFs into memory simultaneously. Process pages as an ordered stream.

---

### 2.2 pdf-lib Bookmark Writing

**What**: Initial attempt to use `pdf-lib` (TypeScript/WASM) to write PDF bookmarks natively.

**Why it failed**:
- `pdf-lib` wrote bookmark outline items as inline dictionary objects, not indirect objects.
- Several PDF viewers (PDF-XChange, Foxit) failed to display these bookmarks, or displayed only the top level.
- The failure was viewer-specific and not caught until real-world user testing with AEC-standard viewers.
- The fix (pikepdf sidecar) required significant architecture change.

**Rust implication**: Any PDF library used for bookmark writing must support creating indirect objects. Validate against PDF-XChange and Acrobat at the start of Rust development, not after.

---

### 2.3 PDF.js as Primary Extraction Backend

**What**: Initial extraction used PDF.js `getTextContent()` as the primary text source.

**Why it failed**:
- PDF.js `getTextContent()` provides text items with bounding boxes, but the bounding boxes are often imprecise (off by several points), and span grouping decisions are made by PDF.js's internal heuristics which are not designed for geometric analysis.
- ROI filtering with imprecise bounding boxes produced detection failures on sheets where the title block text was close to (but outside) the ROI boundary.
- Multi-column and rotated text were handled inconsistently.
- The accuracy was sufficient for basic text extraction but insufficient for the precision required by ROI-based detection.

**Rust implication**: Use PDFium for primary extraction — it provides the same level of span-detail and geometric precision as PyMuPDF. Do not start with a hobbyist-level Rust PDF parser for primary extraction.

---

### 2.4 PDF AST Abstraction Concept

**What**: An early design goal was a generic, universal PDF AST — a complete structural model of the PDF page tree that could serve all extraction needs.

**Why it failed**:
- The concept was too broad. Different document types (specs, drawings, submittals, schedules) require completely different structural models. A single AST abstraction either forces all types into an awkward common shape or becomes so abstract as to provide no value.
- The narrower, per-medium extractions (`SpecDoc` for specs; `LayoutTranscript` for all media; `BookmarkAnchorTree` for navigation) proved far more useful in practice.
- Scope was reduced: only `SpecDoc` for specs-patch and `LayoutTranscript` for all transcript extraction are implemented; generic AST ambition was abandoned.

**Rust implication**: Do not design a universal PDF AST. Design medium-specific extraction types, held together by the `LayoutTranscript` base layer.

---

### 2.5 Workflow State Reuse in applyCorrections

**What**: The intended design was that `applyCorrections()` would take prior analyzed state and apply corrections without re-detecting. Current implementation re-runs `analyze()` internally.

**Why it failed**:
- State management between analyze and applyCorrections was not implemented incrementally. When corrections were added, the simplest path was full re-analysis.
- This means the analyzed state is not truly preserved: changes in source files between phases would be silently re-detected.
- It also adds unnecessary latency for large documents.

**Rust implication**: Implement true cursor state from the start. Analyzed state must be serializable and corrections-applicable without re-analysis.

---

## Section 3: What Was Deferred and Why

### 3.1 LLM-Assisted Narrative Parsing

**Deferred because**: The algorithmic narrative parser (`parse-algorithmic.ts`) handles the vast majority of real addendum narrative formats. LLM integration adds significant complexity (provider management, cost, latency, privacy concerns) for a marginal improvement in coverage.

**Rust plan**: Keep as optional enhancement. Implement only after algorithmic parser is ported and validated. The `TokenVault` privacy abstraction should be ported as infrastructure even if LLM integration is not initially enabled.

---

### 3.2 ROI Auto-Detection from PDFs

**Deferred because**: The algorithm was designed (`automatedRoiRefactorPlan.md`) but never implemented. Manual ROI definition via the GUI profile editor proved sufficient for beta users, and auto-detection complexity was significant.

**Rust plan**: Implement as a Phase 2 feature after core extraction is working. The profile editor (GUI) remains the primary UX; auto-detection is a productivity enhancement, not a correctness improvement.

---

### 3.3 Equipment Schedule Extraction UI

**Deferred because**: The geometry-first schedule extraction engine (`transcript/schedules/`) was implemented but the GUI surface for schedule review/export was not built. The core algorithm handles standard grid tables; complex table variants (merged cells, rotated headers) need more work.

**Rust plan**: Complete the algorithm first (merged cells, rotated headers). Build GUI after algorithm validation.

---

### 3.4 Submittal Workflow

**Deferred because**: The `submittalParser.ts` stub exists, but no workflow engine (`analyze/applyCorrections/execute`), CLI command, or GUI was built. The domain complexity (submittal types, multi-product packages, review stamps) is significant.

**Rust plan**: Requires an explicit V4 scoping decision. The parser stub provides a domain model starting point. Treat as a planned but unframed feature, not a carry-forward of an existing design.

---

### 3.5 Web/SaaS Mode

**Deferred because**: V4 Master Plan Phase 4 scope. The prototype is desktop-only (Electron). Moving to a web architecture requires a multi-tenant auth/billing stack, file upload/download, asynchronous job queuing, and privacy boundary redesign.

**Rust plan**: The Rust binary is the right foundation — it can be called from a web service worker or REST handler. The core logic is already designed for stateless invocation (analyze/execute pattern, no persistent shared state).

---

### 3.6 Pattern Development Tool

**Deferred because**: V4 Non-Negotiable #20 calls for a Pattern Development Tool as Phase 0.5 infrastructure. It was not built in the prototype. All pattern development was done by inspecting test outputs directly.

**Rust plan**: This is high priority for Rust V4. Without a visual pattern development environment, all ROI and extraction profile work requires running full extraction pipelines for each iteration. This should be built before production profile creation work begins.

---

### 3.7 Audit Bundle / Overlay Visual Export

**Deferred because**: V4 Non-Negotiable #7 requires first-class audit trails. The prototype emits JSON audit reports; the planned visual overlay (highlighting detected spans, ROI regions, classification decisions on rendered page images) was designed but not implemented.

**Rust plan**: Design the audit bundle format specification before implementing. The visual overlay is a critical differentiation feature (transparency), not a nice-to-have.
