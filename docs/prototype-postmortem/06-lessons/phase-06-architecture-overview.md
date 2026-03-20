# Phase 06 - Architecture Overview

**Document Type**: Post-Mortem Architecture Narrative  
**Audience**: Rust implementation team (new developers, no prior TypeScript context assumed)  
**Date**: 2026-03-19

---

## System Purpose in One Sentence

Conset PDF is a deterministic, audit-first pipeline for merging, splitting, extracting, and bookmarking AEC construction document PDFs (drawings and specifications), with reliable identity detection based on explicit layout profiles rather than heuristics.

---

## Top-Level Decomposition

The prototype is split into two top-level packages:

```
conset-pdf/               TypeScript monorepo
  packages/core/          @conset-pdf/core  — pure library, no UI
  packages/cli/           @conset-pdf/cli   — CLI entry point, imports core

conset-pdf-gui/           Electron desktop application
  src/main/               Main process (Node.js): IPC handlers, file I/O, core API calls
  src/                    Renderer process (browser): Wizard UIs, profile management
  src/shared/             Cross-process contracts (IPC envelope, types)
```

The GUI is an Electron wrapper that talks to `@conset-pdf/core` exclusively through IPC (main process). The renderer never touches disk or core API directly.

---

## Core Library: Module Map

```
packages/core/src/
│
├── analyze/              PDF loading and caching
│   ├── documentContext.ts    Document lifecycle, transcript caching, PDF.js bookmark compat shim
│   ├── pageContext.ts        Per-page caching (dimensions, rotation, text items, ROI views)
│   └── readingOrder.ts       Visual reading-order reconstruction for wrapped text
│
├── transcript/           V3 extraction system (primary text pipeline)
│   ├── factory.ts            Extractor factory: PyMuPDF primary, PDF.js fallback
│   ├── canonicalize.ts       Normalization: rotation fix, stable sort, content hash
│   ├── quality.ts            Per-page and aggregate quality scoring; acceptance gate
│   ├── candidates.ts         Structural candidate generation (chrome band detection inputs)
│   ├── extractors/
│   │   ├── pymupdfExtractor.ts    PyMuPDF-backed extractor (spawns Python sidecar)
│   │   └── pdfjsExtractor.ts     PDF.js-backed extractor (fallback)
│   ├── sidecar/
│   │   └── extract-transcript.py  Python script for PyMuPDF extraction
│   ├── abstraction/          Privacy-preserving pattern abstraction
│   │   ├── tokenVault.ts         Sensitive text → placeholder token mapping
│   │   └── sanitize.ts           Abstract transcript generation with privacy modes
│   ├── ml/                   ML Ruleset Compiler for profile generation
│   │   ├── rulesetCompiler.ts    Compiler interface
│   │   └── apiCompiler.ts        LLM API backend
│   ├── profiles/             Extended extraction profile system
│   │   ├── types.ts              SpecProfile, SheetTemplateProfile types
│   │   ├── registry.ts           Profile registry with versioning
│   │   └── validation.ts         Profile validation
│   └── schedules/            Equipment schedule extraction
│       ├── extractor.ts          Geometry-first schedule table extractor
│       └── tableBuilder.ts       Column/row assembly from coordinate clusters
│
├── locators/             Pluggable drawing/spec page identity detectors
│   ├── compositeLocator.ts       ROI-first with legacy fallback (drawings only)
│   ├── roiSheetLocator.ts        Profile-driven ROI region extraction (primary)
│   ├── roiSpecsSectionLocator.ts ROI-driven specs section detection
│   ├── sheetLocator.ts           Abstract locator interface
│   ├── specsSectionLocator.ts    Text-based specs section detector (no profile needed)
│   └── legacyTitleblockLocator.ts Heuristic title block detector (fallback, deprecated)
│
├── parser/               Pure ID parsing and normalization (no I/O)
│   ├── drawingsSheetId.ts        Regex patterns for drawing sheet IDs (A-101, M1-01, etc.)
│   ├── specsSectionId.ts         Regex patterns for spec section IDs (XX YY ZZ, XXYYY)
│   └── normalize.ts              Canonical ID format converters
│
├── layout/               JSON layout profile system
│   ├── types.ts                  LayoutProfile, NormalizedROI, profile type enums
│   └── load.ts                   Profile loader with validation and defaults
│
├── standards/            UDS discipline and CSI MasterFormat classification (pure)
│   ├── normalizeDrawingsDiscipline.ts  Prefix → discipline metadata
│   ├── normalizeSpecsMasterformat.ts   Section ID → MasterFormat division metadata
│   ├── compare.ts                Deterministic row comparators for sorting
│   ├── datasets/                 Static lookup tables
│   └── types.ts                  DrawingsDisciplineMeta, SpecsMasterformatMeta
│
├── specs/                Specification PDF extraction, patching, rendering
│   ├── ast/types.ts              SpecDoc AST types, BookmarkAnchorTree
│   ├── extract/                  Specs extraction pipeline
│   │   ├── chromeRemoval.ts          Header/footer band suppression
│   │   ├── sectionDetector.ts        Section heading detection
│   │   ├── anchorDetector.ts         Hierarchical anchor detection
│   │   ├── listDetector.ts           List item detection
│   │   ├── paragraphNormalizer.ts    Wrap-join and hyphen repair
│   │   ├── hierarchyBuilder.ts       Indentation-based hierarchy
│   │   ├── textExtractor.ts          Node creation from raw spans
│   │   └── bookmarkTreeGenerator.ts  SpecDoc AST → BookmarkAnchorTree
│   ├── patch/                    Spec AST patch operations
│   │   ├── types.ts                  SpecPatch, SpecPatchOperation types
│   │   ├── validator.ts              Patch validation
│   │   └── apply.ts                  Patch application (insert/delete/move/renumber/replace)
│   ├── render/                   HTML/CSS → PDF rendering
│   │   ├── htmlGenerator.ts          AST → HTML
│   │   └── pdfRenderer.ts            HTML → PDF via Playwright
│   ├── footerSectionIdParser.ts  Footer section ID pattern parser (Div-23 only, partial)
│   ├── footerSectionMap.ts       Footer-first section boundary map (stub, not yet complete)
│   └── footerValidation.ts       Footer consistency validation
│
├── bookmarks/            Bookmark pipeline (read, validate, build, write)
│   ├── types.ts                  BookmarkNode, BookmarkDestination, BookmarkTree, BookmarkAnchorTree
│   ├── reader.ts                 Read existing bookmarks from PDF
│   ├── validator.ts              Validate bookmark destinations against PDF pages
│   ├── treeBuilder.ts            Build BookmarkTree from BookmarkAnchorTree or inventory
│   ├── corrections.ts            Apply correction overlays (rename/reorder/delete/retarget/rebuild)
│   ├── headingResolver.ts        Layout-aware heading-based section destination resolution
│   ├── pikepdfBookmarkWriter.ts  Sidecar-backed bookmark writer (primary production path)
│   ├── profiles/                 Bookmark style profiles (raw, specs-v1, specs-v2-detailed)
│   └── sidecar/bookmark-writer.py  Python script for QPDF/pikepdf bookmark writing
│
├── narrative/            Addendum narrative PDF processing
│   ├── text-extract.ts           Page-aware text extraction via DocumentContext
│   ├── parse-algorithmic.ts      Pattern-matching parser: drawing/spec revision instructions
│   ├── normalize.ts              ID normalization from narrative text
│   └── validate.ts               Narrative instructions vs. detected inventory validation
│
├── core/                 Business logic: merge, split, assemble
│   ├── mergeAddenda.ts           Legacy direct merge API (wraps planner + applyPlan)
│   ├── splitSet.ts               Split PDF into named subsets
│   ├── assembleSet.ts            Assemble PDFs (deprecated)
│   ├── planner.ts                Merge plan construction (mode semantics, conflict resolution)
│   ├── applyPlan.ts              Plan execution: pdf-lib page copy + pikepdf passthrough write
│   └── report.ts                 Run report generation
│
├── workflows/            Workflow engine (canonical user-facing orchestration)
│   ├── types.ts                  InventoryResult, CorrectionOverlay, ExecuteResult, Issue enums
│   ├── engine.ts                 WorkflowRunner factory and WorkflowImpl interface
│   ├── merge/mergeWorkflow.ts    Merge workflow (analyze/applyCorrections/execute)
│   ├── specs-patch/specsPatchWorkflow.ts  Specs patch workflow (currently abandoned per ROADMAP)
│   ├── bookmarks/bookmarksWorkflow.ts     Bookmarks workflow
│   └── mappers/merge.ts          ParseResult → InventoryRowBase mapping + standards enrichment
│
├── submittals/           Submittal parsing (parser only; no workflow orchestration)
│   └── extract/submittalParser.ts  Experimental submittal parser stub
│
├── config/               Feature flags
│   └── featureFlags.ts           ENABLE_LEGACY_LOCATOR toggle (sparsely exercised)
│
└── utils/                Utilities
    ├── bookmarks.ts              Legacy bookmark generation helpers (deprecated)
    ├── bookmarkWriter.ts         Abstract bookmark writer interface
    ├── pdfLibBookmarkWriter.ts   pdf-lib bookmark writer (development/testing only)
    ├── pikepdfWriter.ts          pikepdf passthrough writer
    ├── pdf.ts                    PDF utility helpers
    ├── fs.ts                     File system helpers
    └── sort.ts                   Deterministic sort utilities
```

---

## Key Architectural Principles

### 1. Analyze → ApplyCorrections → Execute (Workflow Engine)

Every user-facing operation follows a three-phase pattern:

```
analyze()           → Returns InventoryResult (no file writes, pure read)
applyCorrections()  → Returns revised InventoryResult (corrections applied; currently re-runs analyze)
execute()           → Writes output files, returns ExecuteResult
```

The invariant: the user can review `InventoryResult` before any write occurs. The GUI presents the inventory table before proceeding to execute.

**Current limitation**: `applyCorrections()` re-runs `analyze()` internally rather than mutating prior state. Plan reuse in `execute()` is also only partial for merge. See risk R-007.

### 2. Transcript-First Extraction

`DocumentContext` runs a single transcript extraction at initialization:

```
DocumentContext.initialize()
  → spawns extract-transcript.py (PyMuPDF sidecar)
  → caches LayoutTranscript
  → all downstream consumers read from cached transcript
```

The fallback path (PDF.js) is invoked only when PyMuPDF sidecar is unavailable or throws. Quality gates score per-page transcript quality but do not automatically trigger extractor switching on quality grounds (only on error/availability grounds). See R-004.

### 3. Locator Fallback Chain (Drawings)

```
CompositeLocator
  1. RoiSheetLocator    — requires LayoutProfile with explicit ROI bounding boxes
  2. LegacyTitleblockLocator — heuristic fallback (deprecated, ENABLE_LEGACY_LOCATOR flag)
```

For specs, `SpecsSectionLocator` uses pure text pattern matching; no profile required.

### 4. Python Sidecar Boundary

Two Python sidecar scripts handle operations that require LGPL-licensed libraries (not bundleable under Apache-2.0):

| Sidecar | Purpose | Library |
|---|---|---|
| `extract-transcript.py` | Text extraction | PyMuPDF |
| `bookmark-writer.py` | Bookmark writing | pikepdf/QPDF |

The `pikepdfWriter.ts` also calls pikepdf for final PDF passthrough writing during merge. These are the only cross-language process boundaries in the prototype. The Rust implementation replaces all three with native PDFium/lopdf equivalents.

### 5. Determinism as Design Invariant

All output must be byte-identical across identical (input, profile, engine version) tuples. Mechanisms:

- Timestamps excluded from hashes
- Stable sort with full tie-break on y-position, x-position, content hash
- Coordinate normalization: page dimensions normalized to rotation=0 basis before any comparison
- Merge planner ordering: cover pages first, then lexicographic by source and page index
- Standards normalization: canonical sort order defined in static lookup tables

Determinism is conditional in current prototype: strong on PyMuPDF path, weakened on PDF.js fallback path (coordinate precision variance tolerated in tests). See R-008 and SOT-006.

### 6. Profile-Driven Detection (No Auto-Detection)

Layout profiles are explicit JSON files that define named ROI bounding boxes relative to page dimensions. There is no auto-detection of title block location in production paths. The `--auto-layout` CLI flag and profile registry matching exist as infrastructure, but are not wired into merge locator selection (SOT-007).

### 7. Unchanged Pages are Byte-Exact Copies

Merge operations preserve all non-replaced pages as verbatim copies. The planner tracks which pages require no modification; `applyPlan.ts` copies those pages without re-encoding. This is non-negotiable for audit integrity and round-trip safety.

---

## Module Dependency Rules

The architecture enforces strict layering:

```
utils/            ← no internal deps
parser/           ← no internal deps (pure)
standards/        ← no internal deps (pure)
layout/           ← no internal deps
locators/         ← parser, layout, analyze (pageContext only), transcript (types)
analyze/          ← transcript, utils, layout
transcript/       ← utils
specs/            ← transcript, analyze, locators, parser, standards
narrative/        ← analyze, parser
bookmarks/        ← analyze, utils
core/             ← analyze, locators, parser, bookmarks, narrative, utils, standards
workflows/        ← core, specs, bookmarks, narrative, standards, types
```

The GUI and CLI consume only `workflows/` and a narrow set of `layout/` + `bookmarks/types.ts` imports. They do not reach into internal pipeline modules.

---

## Data Flow: Merge Operation (Drawings)

```
CLI/GUI
  → MergeWorkflow.analyze(input: MergeAnalyzeInput)
      → DocumentContext.initialize() for each PDF
          → extract-transcript.py spawned via sidecar
          → LayoutTranscript cached
      → planner.planMerge() (includeInventory: true)
          → CompositeLocator.locate() per page
              → RoiSheetLocator: applies LayoutProfile ROI filters
                  → readingOrder.ts assembles multi-span text
                  → drawingsSheetId.ts parses assembled text
              → LegacyTitleblockLocator (fallback, if ENABLE_LEGACY_LOCATOR)
          → standards enrichment (discipline, sort order)
          → collision/duplicate resolution
      → InventoryResult returned (no writes)
  → [User reviews inventory in GUI or CLI table]
  → MergeWorkflow.execute(input: MergeExecuteInput)
      → applyPlan.ts page-copies in-memory (pdf-lib)
      → pikepdfWriter.ts issues passthrough write (pikepdf sidecar)
      → report.ts writes audit JSON
```

---

## Data Flow: Specs Extraction (SpecsPatch Workflow)

Note: The specs-patch workflow was abandoned per current ROADMAP and prototype execution, but its internal pipeline remains partially active (AST extraction, bookmarkTreeGenerator) and feeds the bookmarks workflow. See ADR-008 and R-011.

```
SpecsPatchWorkflow.analyze()
  → DocumentContext.initialize()
  → specs/extract pipeline:
      chromeRemoval → sectionDetector → anchorDetector
      → listDetector → paragraphNormalizer → hierarchyBuilder
      → textExtractor → bookmarkTreeGenerator
  → SpecDoc AST + BookmarkAnchorTree returned

SpecsPatchWorkflow.execute()
  → specs/patch/apply.ts applies patch ops to AST
  → specs/render/htmlGenerator.ts: AST → HTML
  → specs/render/pdfRenderer.ts: HTML → PDF (Playwright)
  → BookmarksWorkflow pipe: BookmarkAnchorTree → final bookmarks
```

---

## GUI–Core IPC Boundary

All GUI↔core calls go through named IPC channels. The response envelope is always:

```typescript
IpcResponse<T> = {
  success: boolean;
  data?: T;
  error?: {
    message: string;
    code?: string;
    stack?: string;
    context?: unknown;
  };
}
```

The renderer receives this via preload's `bridge.invoke()`, which throws a `BridgeError` on `success: false`. See Phase 4 contract doc for full shape.

---

## Implementation Status Summary

| Module/Subsystem | Status | Notes |
|---|---|---|
| Transcript (PyMuPDF path) | Complete | Primary extraction path |
| Transcript (PDF.js fallback) | Complete | Fallback only; reduced determinism |
| ROI-based detection | Complete | Requires explicit profile |
| Legacy title block detection | Deprecated/partial | Gated behind feature flag |
| Merge workflow (drawings + specs) | Complete | Engine + CLI + GUI |
| Split workflow | Complete | Engine + CLI + GUI |
| Fix Bookmarks workflow | Complete | Engine + CLI; GUI placeholder only |
| Specs Patch workflow | Abandoned | Engine + CLI existed; now abandoned per ROADMAP |
| Standards normalization | Complete | Pure, fully tested |
| Narrative validation | Complete | Advisory; not mode-driven |
| Footer-first anchoring | Partial | Parser exists; map builder is stub |
| Schedule extraction | Partial | Geometry-first core; fallback TODOs |
| Submittal parser | Partial | Parser stub only; no workflow/CLI/GUI |
| Privacy abstraction (TokenVault) | Complete | Default strict mode |
| ML Ruleset Compiler | Complete | API-backed; LLM provider optional |

---

## Rust Handoff Summary

The Rust implementation should:

1. Replace both Python sidecars (PyMuPDF + pikepdf) with native PDFium + lopdf/pdf-rs
2. Implement the transcript canonical type as a first-class data structure
3. Implement the three-phase workflow engine pattern natively
4. Port all regex patterns from `parser/` and `specs/extract/` verbatim
5. Port standards lookup tables from `standards/datasets/` verbatim
6. Implement disk-streaming merge from day one (do not implement in-memory assembly)
7. Enforce determinism uniformly across all extraction paths (no path-conditional variance)
8. Maintain the byte-verbatim unchanged-page invariant

See `07-rust-handoff/` for full module-to-crate mapping and dataset portability matrix.
