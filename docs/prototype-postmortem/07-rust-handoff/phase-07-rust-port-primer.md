# Phase 07 - Rust Port Primer

**Document Type**: Rust Handoff — Primary Entry Point  
**Audience**: Rust implementation team with no prior TypeScript codebase context  
**Date**: 2026-03-19

---

## Purpose

This document is the entry point for the V4 Rust reimplementation. It translates the TypeScript prototype into concrete Rust implementation guidance: crate structure, module ownership, binary entry points, and external library choices. Read this before touching any other Phase 7 document.

For the complete module-level mapping, see `phase-07-ts-to-rust-module-map.md`. For Python sidecar replacement details, see `phase-07-sidecar-replacement-plan.md`. For dataset portability decisions, see `phase-07-dataset-portability-matrix.md`.

---

## System in One Sentence

Conset PDF is a deterministic, audit-first pipeline for merging, splitting, extracting, and bookmarking AEC construction documents (drawings and specifications), driven by explicit JSON layout profiles rather than heuristics.

---

## V4 Repository Structure (from MASTER_PLAN_v4.md)

```
repo-root/
├── apps/
│   ├── backend-cli/         # CLI binary entry point (conset-pdf)
│   └── desktop-gui/         # Tauri desktop app
├── crates/
│   ├── core-engine/         # Deterministic extraction and transforms
│   ├── workflows/           # Merge/split/bookmarks/specs-patch orchestration
│   ├── contracts/           # Shared request/response/event schemas (serde types)
│   └── standards-data/      # Embedded standards datasets (static lookup tables)
├── docs/
│   ├── prototype-postmortem/
│   └── v4/
└── tests/
    ├── corpus/              # Real PDF test fixtures
    └── integration/         # Cross-crate integration tests
```

**Boundary rules**:
- GUI depends on backend only through `crates/contracts`
- No GUI imports from `core-engine` internals
- Every `contracts` version bump requires integration test updates

---

## Crate Ownership

### `crates/core-engine`

The computation kernel. Pure business logic; no I/O except PDF reading. All modules are deterministic pure functions or stateless structs.

**Submodules** (direct mapping from `packages/core/src/`):

| Submodule | Purpose | TS Source |
|---|---|---|
| `transcript` | PDF text extraction, canonicalization, quality scoring | `transcript/` |
| `locators` | Drawing sheet ID and spec section page identity detection | `locators/` |
| `parser` | Pure ID string parsing and normalization | `parser/` |
| `layout` | JSON layout profile loading and validation | `layout/` |
| `standards` | UDS discipline and MasterFormat classification | `standards/` |
| `specs` | Specs PDF extraction pipeline, AST, and patch operations | `specs/` |
| `bookmarks` | Bookmark reading, building, writing, validation | `bookmarks/` |
| `narrative` | Addendum narrative PDF parsing and validation | `narrative/` |
| `analyze` | Document context: transcript caching, page lifecycle | `analyze/` |
| `chrome` | Header/footer band detection per medium | `specs/extract/chromeRemoval`, future drawings/submittals |
| `submittals` | Submittal parsing (stub) | `submittals/` |
| `config` | Runtime configuration and feature toggles | `config/` |

### `crates/workflows`

Workflow orchestration layer. Implements the analyze → applyCorrections → execute engine for each workflow. Depends on `core-engine` and `contracts`.

**Workflows**:
- `merge::MergeWorkflow`
- `split::SplitWorkflow`
- `bookmarks::BookmarksWorkflow`
- `specs_patch::SpecsPatchWorkflow`

Each workflow implements a common `Workflow` trait:

```rust
pub trait Workflow {
    type AnalyzeInput;
    type AnalyzeOutput;      // InventoryResult
    type CorrectionsInput;
    type ExecuteInput;
    type ExecuteOutput;      // ExecuteResult

    fn analyze(&self, input: Self::AnalyzeInput) -> Result<Self::AnalyzeOutput>;
    fn apply_corrections(
        &self,
        inventory: Self::AnalyzeOutput,
        corrections: Self::CorrectionsInput,
    ) -> Result<Self::AnalyzeOutput>;
    fn execute(&self, input: Self::ExecuteInput) -> Result<Self::ExecuteOutput>;
}
```

**Critical**: `apply_corrections()` must mutate the prior analyzed state — it must NOT re-run `analyze()`. This was the primary design shortcoming of the prototype (R-007). The corrections overlay is purely additive to the previous `InventoryResult`.

### `crates/contracts`

All shared `serde`-annotated types used across the CLI, GUI, and workflow boundary. No business logic.

**Contents**:
- `LayoutTranscript`, `LayoutPage`, `LayoutSpan` — canonical extraction types
- `InventoryResult`, `CorrectionOverlay`, `ExecuteResult` — workflow contract types
- `MergePlan`, `MergeAction` — merge planning types
- `LayoutProfile`, `NormalizedROI` — profile schema types
- `BookmarkNode`, `BookmarkTree`, `BookmarkAnchorTree` — bookmark types
- `IpcResponse<T>` — GUI/backend IPC envelope
- `AuditBundle` — per-run audit artifact type
- `ScheduleTable` — schedule extraction result type for future GUI/API surfacing

All types must derive `serde::Serialize` and `serde::Deserialize`. All types must include a `schema_version` field or be wrapped in a versioned envelope for forward compatibility.
`AuditBundle` must follow the explicit schema in `04-contracts/phase-04-audit-bundle-schema.md`; do not infer the shape from prototype `MergeReport` output alone.

### `crates/standards-data`

Static lookup tables embedded in the binary. No I/O. Must be a `no_std`-compatible crate.

**Contents**:
- UDS discipline designator table (all UDS single-letter designators + canonical 4-letter codes)
- Multi-letter alias map (FP, FA, DDC, ATC, SEC, AV, IT, SV, EX, DM → canonical codes)
- Disambiguation keyword lists (CONTROLS_KEYWORDS, CIVIL_KEYWORDS for ambiguous `C` prefix)
- CSI MasterFormat division table (all 50 divisions: modern 6-digit format)
- Legacy MasterFormat → modern mapping (5-digit → 6-digit)
- Canonical discipline sort order (determines output file ordering)

See `phase-07-dataset-portability-matrix.md` for portability classification of each dataset.

### `apps/backend-cli`

The CLI binary. Implements all 8 CLI commands by calling `crates/workflows`:

| Command | Workflow | Status in Prototype |
|---|---|---|
| `merge-addenda` | `MergeWorkflow` | Complete |
| `split-set` | `SplitWorkflow` | Complete |
| `fix-bookmarks` | `BookmarksWorkflow` | Complete |
| `specs-patch` | `SpecsPatchWorkflow` | Abandoned (Playwright dep); restore in V4 |
| `specs-inventory` | `MergeWorkflow` (analyze-only mode) | Complete |
| `detect` | `MergeWorkflow` (analyze-only, no execute) | Complete |
| `debug-walkthrough` | diagnostic / dev tooling | Complete |
| `assemble-set` | `AssembleWorkflow` | Deprecated |

New command required in V4:
- `pattern-dev` (or `dev pattern`) — Pattern Development Tool (NN-20; required by master plan as Phase 0.5 infrastructure)

### `apps/desktop-gui`

Tauri application. The frontend communicates with the Rust backend through Tauri's `invoke` IPC mechanism. All responses use the `IpcResponse<T>` envelope from `crates/contracts`.

**Explicit deferred GUI scope carried forward from Phase 5 / Phase 8**:
- Report Viewer: defer UI implementation to V4 Phase 3. The backing data contract is `AuditBundle`; only the viewer surface is deferred.
- Automated ROI detection and profile generation UI: defer to V4 Phase 3+ as an ML-assisted pipeline. The prototype contains only design material, not validated behavior.
- Schedule extraction UI: defer to V4 Phase 3. The extraction core ports in Phase 1, but the GUI/table-inspection surface comes later. `ScheduleTable` must exist in `crates/contracts` from Phase 1 so the later UI is unblocked.

---

## Key External Dependencies

### PDF Library Stack (REQUIRED CHANGES from prototype)

The prototype used:
- `pdfjs-dist` (PDF.js) — text extraction (demoted to fallback)
- `pdf-lib` — page copy and assembly
- Python: `pymupdf` (PyMuPDF) — primary text extraction sidecar
- Python: `pikepdf`/`QPDF` — bookmark writing and PDF passthrough sidecar

V4 Rust replaces all of these with:

| Operation | Prototype | V4 Rust | License |
|---|---|---|---|
| Text/layout extraction | PyMuPDF (Python sidecar) | `pdfium-render` | Apache-2.0 |
| Page stream copy (merge) | `pdf-lib` + pikepdf passthrough | native streaming via `lopdf` | MIT |
| Bookmark writing | pikepdf/QPDF (Python sidecar) | `lopdf` (indirect objects) | MIT |
| PDF structure parsing | `pdf-lib` | `lopdf` or `pdf-rs` | MIT |
| Spec section rendering | Playwright (headless Chrome) | `headless_chrome` crate or `wkhtmltopdf` shim | MIT/LGPL-note |

**Note on spec rendering**: `wkhtmltopdf` uses Qt WebKit which is LGPL. If this conflicts with licensing requirements (NN-11), use `headless_chrome` (MIT wrapper around system Chromium) or a purpose-built spec formatter using `printpdf` + `ab_glyph`. The system Chromium approach avoids LGPL-at-link-time since it shells out to an external process.

### Full Rust Dependency Table

| Crate | Version | Purpose | License |
|---|---|---|---|
| `pdfium-render` | latest | PDF text and geometry extraction | Apache-2.0 |
| `lopdf` | latest | PDF structure manipulation, bookmark writing, page copy | MIT |
| `serde` + `serde_json` | 1.x | Serialization for all contracts | MIT/Apache |
| `regex` | 1.x | ID pattern matching (verbatim port of TS patterns) | MIT/Apache |
| `tokio` | 1.x | Async runtime for CLI and IPC | MIT |
| `clap` | 4.x | CLI argument parsing | MIT/Apache |
| `tauri` | 2.x | Desktop GUI shell | MIT/Apache |
| `proptest` | 1.x | Property-based testing for ID parsing | MIT/Apache |
| `tracing` + `tracing-subscriber` | latest | Structured logging, audit trail emission | MIT |
| `uuid` | 1.x | Workflow run IDs, span IDs | MIT/Apache |
| `sha2` | 0.10.x | Content hashing for determinism verification | MIT/Apache |
| `cargo-license` | dev/CI | License audit gate (NN-11) | MIT |

---

## Three-Phase Workflow Pattern (Preserve Exactly)

Every workflow follows this pattern. This is an audit-safety invariant, not a UX preference:

```
analyze(input: AnalyzeInput) → InventoryResult
  ↓ (user reviews, optionally adjusts corrections)
apply_corrections(inventory: InventoryResult, corrections: CorrectionOverlay) → InventoryResult
  ↓ (user confirms)
execute(input: ExecuteInput) → ExecuteResult
```

**Invariants**:
1. `analyze()` makes zero file writes; it is a pure read operation
2. `apply_corrections()` mutates the existing `InventoryResult` cursor — it does NOT re-parse any PDF
3. `execute()` is the only phase that writes output files
4. All output PDFs are written to a temp path first; renamed atomically on success (write-then-rename)
5. Unchanged pages bypass all processing — page content streams are passed through byte-identical

---

## Coordinate System (Critical for Rust Implementation)

**Protocol for all spatial values in `LayoutTranscript` and `LayoutProfile`**:

- Origin: **top-left corner** of page at `(0.0, 0.0)`
- Y-axis: increases **downward** (toward bottom of page)
- X-axis: increases **rightward**
- All coordinates are **normalized to page dimensions**: `0.0` = left/top edge, `1.0` = right/bottom edge

**PDFium's native coordinate system is bottom-left origin, Y increases upward.** The extraction layer must immediately apply the transformation:

```rust
// PDFium → Conset normalized
fn pdfium_to_normalized(pdfium_bbox: PdfiumRect, page_width: f32, page_height: f32) -> BBox {
    let x0 = pdfium_bbox.left / page_width;
    let y0 = 1.0 - (pdfium_bbox.top / page_height);    // flip Y, then normalize
    let x1 = pdfium_bbox.right / page_width;
    let y1 = 1.0 - (pdfium_bbox.bottom / page_height); // flip Y, then normalize
    BBox { x0, y0, x1, y1 }
}
```

**Rotation normalization** must be applied before any downstream consumer sees span coordinates. After normalization, all downstream code assumes rotation=0 basis regardless of the physical orientation of the PDF page.

**This transformation must happen once in the extraction layer.** No downstream module should contain rotation logic; if it does, that is a design violation.

---

## Determinism Requirements

Every aspect of the pipeline must produce byte-identical output for identical `(input_files, profile, engine_version)`:

| Requirement | Implementation notes |
|---|---|
| No `HashMap` in pipeline | Use `BTreeMap` everywhere in deterministic paths |
| Stable sort | Total ordering: y-position primary, x-position secondary, content hash tertiary |
| Coordinate precision | Round all extracted coordinates to 2 decimal places (points) before hashing or comparison |
| Hash fields | Content hash excludes `extractionDate`, `filePath`; includes text, bbox, font, flags |
| No random seeding | No `rand` in production paths |
| Canonical output ordering | Standards sort order is determined by `standards-data` tables, not by insertion order |

**Known gap from prototype**: PDF.js fallback path was non-deterministic (coordinate precision differed from PyMuPDF). In V4, PDFium is the only extraction path. There is no fallback that relaxes determinism. If PDFium fails on a page, emit an empty `LayoutPage` for that page (partial truth) rather than switching to a different extractor (NN-15).

---

## Byte-Verbatim Page Invariant (NN-12)

For every merge operation:
- Pages not targeted by the merge plan must be copied as **verbatim byte streams**
- The page content stream (PDF `stream...endstream` object) must be byte-identical to the source
- PDF container normalization (cross-reference table, object IDs) is acceptable collateral
- Test: `sha256(original_page_content_stream) == sha256(output_page_content_stream)` for all unchanged pages

Implementation hint: `lopdf` exposes access to raw page content streams. Reading the bytes and writing them unchanged (without decode/re-encode) satisfies this requirement. Do not call any PDF rendering or font subsetting pass on these streams.

---

## Error Handling and Hostile Input (NN-13)

All workflow entry points must use `catch_unwind` to contain panics:

```rust
use std::panic;

pub fn execute_workflow(input: &WorkflowInput) -> WorkflowResult {
    let result = panic::catch_unwind(|| {
        // ... workflow execution
    });
    match result {
        Ok(r) => r,
        Err(payload) => WorkflowResult::panic_error(payload),
    }
}
```

Memory limits:
- Default per-document memory cap: 256 MB
- At 80% of cap: emit `APPROACHING_MEMORY_CAP` warning in audit output
- At 100% of cap: abort with structured soft-fail; return partial results for pages processed before cap

Malformed PDF handling:
- `pdfium-render` handles most malformed PDFs gracefully
- Any page that produces an error from PDFium should emit an empty `LayoutPage` with a `quality_score: 0.0` and error code, not a panic

---

## ADR Cross-References

Each V4 architectural decision is grounded in prototype ADRs. Refer to Phase 3 for full context:

| V4 Decision | ADR Reference | Phase 7 Section |
|---|---|---|
| Replace PyMuPDF with PDFium | ADR-001 | sidecar-replacement-plan.md |
| Replace pikepdf with lopdf | ADR-001 | sidecar-replacement-plan.md |
| Disk-streaming merge from day one | ADR-004 | Workflow section above |
| Determinism on all paths | ADR-005 | Determinism section above |
| Profile-driven detection | ADR-006 | coordinate system section above |
| Privacy-preserving abstraction | ADR-007 | TokenVault → Rust section below |
| No PDF.js | ADR-002 | Dependency table above |
| No in-memory merge | ADR-004 | Workflow section above |

---

## Privacy-Preserving Abstraction (TokenVault → Rust)

The prototype's `TokenVault` (in `transcript/abstraction/tokenVault.ts`) replaces sensitive text with structural tokens before any LLM call. This must be ported to Rust as a first-class component of `crates/core-engine::transcript::abstraction`.

Three privacy modes (preserve exactly):
- `STRICT_STRUCTURE_ONLY` — all text replaced with structural tokens; no content sent to LLM; default for production
- `WHITELIST_ANCHORS` — known-safe anchor tokens (section IDs, discipline prefixes) are sent; PII fields remain tokenized
- `FULL_TEXT_OPT_IN` — user explicitly opts in to full text transmission; must be gated behind CLI flag and user confirmation

**Risk R-010 carry-forward**: Ensure that `FULL_TEXT_OPT_IN` is guarded by an explicit user-facing acknowledgment in both CLI and GUI. The token vault mode must appear in the audit bundle.

---

## Submittal Workflow Decision (R-014)

The prototype has a submittal parser stub with no workflow engine, CLI command, or GUI. Phase 7 flags this as a V4 scope decision:

**Recommended**: Promote submittals to a first-class workflow in V4 Phase 2. The submittal medium is part of the product thesis (three mediums: specs, drawings, submittals). Not implementing it makes the system incomplete for the target AEC use case.

**Minimum V4 Scope**: Define the `SubmittalWorkflow` struct and `SubmittalChromeMetadata` struct in `crates/workflows` and `crates/contracts` respectively, even if the extraction is not yet complete. This prevents the scope creep of further deferral.

---

## V4 Phase–to–ADR Reference Matrix

Cross-reference of V4 Master Plan phases to post-mortem ADRs and lessons (Step 40):

| V4 Phase | Relevant ADRs | Relevant Lessons | Key Risk |
|---|---|---|---|
| Phase 0: Infrastructure | ADR-005 (determinism) | 1.2 (workflow engine) | R-008 (extractor determinism) |
| Phase 0.5: Pattern Dev Tool | ADR-006 (profile detection) | 1.1 (ROI profiles) | NN-20 (not built in prototype) |
| Phase 1: Core Engine | ADR-001, ADR-002 (sidecar replacement) | 1.3 (transcript), 2.1 (in-memory) | R-006 (merge maturity) |
| Phase 1: Merge Workflow | ADR-004 (disk streaming) | 1.2 (workflow engine), 2.2 (pdf-lib) | R-007 (plan state reuse) |
| Phase 2: Specs Patch | ADR-003 (workflow engine), ADR-008 (Playwright) | 2.3 (Playwright), 3.5 (deferred) | NN-03 (spec reflow) |
| Phase 2: Standards | ADR-005 (determinism) | 1.6 (pure functions) | None |
| Phase 3: GUI | ADR-003 (workflow engine) | 1.2 (three-phase pattern) | R-012 (IPC envelope drift) |
| Phase 4: Submittals | ADR-001 (sidecar) | 3.3 (submittal deferred) | R-014 (no workflow engine) |
| Phase 4: ML/LLM | ADR-007 (privacy) | 3.1 (LLM narrative deferred) | R-010 (privacy mode policy) |

---

## Summary of What to Port vs. What to Replace

| Category | Action | Priority |
|---|---|---|
| Three-phase workflow pattern | Port exactly | P0 |
| `LayoutTranscript` type contract | Port exactly (use `contracts` crate) | P0 |
| UDS discipline + MasterFormat tables | Port verbatim (see portability matrix) | P0 |
| Drawing sheet ID regex patterns | Port verbatim (see portability matrix) | P0 |
| Spec section ID regex patterns | Port verbatim | P0 |
| Layout profile JSON schema | Port exactly | P0 |
| `IpcResponse<T>` envelope | Port exactly | P0 |
| Rotation normalization algorithm | Port, retest | P0 |
| Byte-verbatim page copy | Implement natively (lopdf) from day one | P0 |
| Python sidecars (PyMuPDF + pikepdf) | Replace with PDFium + lopdf | P0 |
| In-memory merge assembly | Replace with disk-streaming merge | P0 |
| pdf-lib bookmark writer | Replace with lopdf indirect objects | P0 |
| Playwright spec renderer | Replace with headless_chrome or printpdf | P1 |
| PDF.js fallback extractor | Remove entirely (no V4 fallback path) | P0 |
| Feature flag system | Replace with Rust compile-time features + runtime config | P2 |
| Legacy locator | Do not port (deprecated) | — |
| Legacy bookmark utils | Do not port (deprecated) | — |
| `assembleSet` workflow | Do not port (deprecated) | — |
