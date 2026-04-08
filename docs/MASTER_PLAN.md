# Conset PDF: Master Plan
**Version:** 4.9.0 (Phase 8 — Production Hardening)  
**Date:** April 7, 2026  
**Owner:** HLLMR LLC  
**Status:** ✅ Ready for Implementation  
**Doc Status Tag:** Implemented

---

## Executive Overview

Conset PDF is a **deterministic-first, compiler-model** system for extracting, parsing, and reconstructing structured content from AEC PDFs with production-grade reliability, auditability, and privacy preservation.

**Core Promise:** AEC users get *one-button workflows*, *same results every time*, *outputs that can be trusted*, and *provable audit trails*—not AI magic.

**Guiding Principle:** Do it once, and do it right the first time. Spare no expense. We don't care about hard, we don't care about fast. We care about RIGHT.

---

## Table of Contents

1. [The North Star](#the-north-star)
2. [Non-Negotiables](#non-negotiables)
3. [Document Families & Mediums](#document-families--mediums)
4. [Architecture: Open Engine + Paid GUI](#architecture-open-engine--paid-gui)
5. [The Compiler Model](#the-compiler-model)
6. [Technology Stack](#technology-stack)
7. [Deterministic Shared Pipeline](#deterministic-shared-pipeline)
8. [Medium-Specific Processing](#medium-specific-processing)
9. [Pattern Database System](#pattern-database-system)
10. [Audit Trail & Quality Framework](#audit-trail--quality-framework)
11. [Implementation Roadmap](#implementation-roadmap)
12. [Phase Definitions](#phase-definitions)
13. [Development Workflow & AI Coding Agent Strategy](#development-workflow--ai-coding-agent-strategy)
14. [Hard Constraints & Governance](#hard-constraints--governance)

---

## Documentation Authority

This master plan is the top-level documentation authority for Conset PDF.

Authority order:

1. Master plan (`MASTER_PLAN.md`)
2. Canonical derived docs (architecture, standards, workflow, active implementation plan)
3. Code and tests as implementation evidence
4. Historical records as informative only

Decision rule:

If a claim is not in this plan or a canonical doc derived from it, that claim is non-authoritative.

Status tag policy:

All maintained docs must include a `Doc Status Tag` with one of: `Implemented`, `Planned`, `Deferred`, `Deprecated`, `Archived`.

---

## The North Star

### Product Thesis

**AEC does not want "AI." AEC wants:**
- **One button** workflows
- **Same result every time**
- **Outputs that can be trusted**
- **Receipts:** provable audit trails showing what happened

### The Moat

**Our moat is reliability + auditability + AEC-specific structure.**

General PDF tooling aims to be universal and ends up being unreliable. We aim to be *correct for business-critical AEC documents*. This is how we win:

1. **Determinism** (every run identical)
2. **Auditability** (every decision logged with evidence)
3. **Spec section regeneration** (the moat feature—AEC's most painful workflow solved)
4. **High-fidelity extraction** (drawings and submittals treated as serious data, not guesses)
5. **Transparency** (overlays, audit bundles, no silent failures)

---

## Non-Negotiables

These are **hard commitments** that guide every architectural and implementation decision:

1. **Determinism is sacred.** No runtime randomness. Same input + same detection policy version + same engine version = **identical output**.

2. **Do it right the first time.** Architecture decisions optimize for long-term correctness and maintainability, not short-term demos.

3. **Reflow & reconstruction to editable structure is a real AEC need** (spec addenda edits) and is a **core moat feature**.

4. **Chrome/furniture is not content.** Must be detected, modeled, and excluded explicitly.

5. **Chrome/furniture differs by medium** (drawings ≠ specs ≠ submittals). Separate detectors/handlers per type.

6. **Chrome metadata must be preserved and reused.** Headers/footers contain critical project information (project ID, section numbers, dates) that must be extracted, stored, and reapplied to regenerated content to maintain professional appearance.

7. **Audit trail is first-class.** Every run emits an explainable artifact bundle with overlays + decisions.

8. **Specs:** section-only regeneration. Never regenerate whole books unless forced.

9. **Drawings/Submittals:** extraction and organization, not re-typesetting.

10. **No Python runtime** may be shipped to end users. Single Rust binary only.

11. **Licensing hygiene:** engine stays permissive (Apache-2.0 or MIT); no GPL/AGPL in core dependency graph. PDFium is Apache-2.0 compliant.

12. **Unchanged pages must remain unchanged.** For addenda/edits, pages outside the impacted section/sheet must be **verbatim copies** (byte-for-byte when possible).

13. **PDF as hostile input.** Crash containment, memory caps, safe failure modes. **Soft fails preferred:** "Do what you can, notify on failures for direction" rather than hard failures that discard all work.

14. **No silent failures.** Low confidence → emit "Needs Review" with visual evidence, never guess.

15. **Partial truth over null.** When full certainty is impossible, return grounded partial structure.

16. **Tests first.** Every function has a test. Every phase has integration tests against torture corpus.

17. **Partial success is success.** If 80/100 sections process correctly, output those 80 and ask user how to handle the 20 failures. Never discard working results because some operations failed.

18. **Medium detection is user-driven for processing workflows.** GUI enforces context through workflow-based file pickers. CLI requires explicit operation flags. When a user explicitly invokes a classify, split, or normalize intake operation, autonomous classification executes as direct fulfillment of that intent—this is not auto-detection. Silent auto-selection into a wrong processing workflow remains prohibited.

19. **Accuracy over visual fidelity.** For spec regeneration: textual accuracy 100% required, visual fidelity best-effort. Readable and correct beats pixel-perfect.

20. **Pattern Development Tool is infrastructure, not polish.** The Pattern Dev Tool must be built early (Phase 0.5) as it's a critical development dependency for all pattern-based work (Phases 2-4).

21. **ROI detection is autonomous-first and deterministic.** Production extraction must not require manual per-project profile authoring in normal operation. Manual ROI/profile management is retained as an **admin-only** refinement, debugging, and fallback surface.

22. **Intake Triage is a mandatory pre-Lexer stage.** All multi-file ingestion, bundle assembly, page auditing (rotation, blank pages, corrupt pages), and optional document-type classification happen at Stage 0 before any extraction pipeline stage executes. The Lexer operates only on a `NormalizedIntakeBundle`, never on raw user inputs directly.

23. **`lopdf` is the canonical PDF write backend.** Page-level operations (insert, delete, swap, rotation normalization, bookmark update) use `lopdf` (MIT-licensed, pure Rust, no runtime dependency). PDFium handles read-only extraction. Headless Chrome handles content generation. These three libraries have non-overlapping responsibilities and must not be substituted for each other's roles.

24. **Vector-first deterministic adaptive detection is default.** For vector PDFs, the baseline path uses geometry + text heuristics, deterministic scoring, and explainable tie-breaks. ML and cloud AI are never baseline requirements.

25. **Firm templates are auto-learned, not manually authored as primary UX.** The system should auto-learn relative field ROIs (for example sheet number/name) from deterministic detection and persist as unconfirmed templates; user correction is a lightweight refinement flow, not profile authoring.

26. **Any AI-enhanced path is explicit opt-in and minimized-scope.** AI fallback may run only after low-confidence deterministic failure, must be user-invoked, and should transmit only minimal cropped regions required for recovery, never full sheets by default.

27. **Local micro-ML is assistive, deterministic-bounded, and version-locked.** On-device micro-models may boost confidence on hard edge cases, but baseline deterministic heuristics remain primary. Model version, threshold policy, and fusion math must be audit-logged for every assisted decision.

28. **Power-user LLM integration is validation-only unless explicitly authorized.** API-driven LLM flows may validate ambiguous detections and generate explicit instruction manifests, but cannot silently mutate outputs. All LLM prompts/responses must be captured as redacted audit artifacts with provider/model/version metadata.

29. **Raster PDFs require a first-class OCR path.** If text layer quality is insufficient, pipeline must route through OCR with explicit confidence/provenance tagging and visible review state; OCR text must never be mixed with vector text without source attribution.

30. **Schedule parsing must emit schema-versioned structured outputs.** Table extraction outputs must support machine-consumable export formats (at minimum JSON/CSV/XML) with stable schema contracts and field-level provenance.

31. **Replayable correction manifests are first-class artifacts.** Human corrections must be persistable, versioned, and re-applicable to subsequent runs with deterministic scope checks and audit traces.

32. **Review is provenance-first, not screenshot-first.** Every field, decision, export row, and correction must resolve back to source page, region, method, confidence, and branch reason.

33. **External API paths require redaction and outbound payload control.** Any LLM or third-party service integration must support region-level redaction, payload manifests, and policy modes that can prohibit full-page or full-document transmission.

34. **Batch orchestration and resumability are first-class production capabilities.** Multi-file and project-scale runs must support queueing, partial completion, retry, resume, and explicit per-file/per-page state accounting.

35. **Instruction DSL is the canonical automation contract.** Explicit automation requests, user-authored recipes, and AI-suggested instructions must converge on one typed manifest language with validation before execution.

36. **Confidence is exposed through policy profiles, not raw internals.** User-facing operational modes should tune review strictness and escalation behavior without exposing unstable low-level heuristics directly.

37. **Standards normalization builds on the existing canonical standards scaffold.** UDS/NCS, MasterFormat, and related AEC standards references already recovered from the prototype remain the source scaffold for normalization work; implementation must wire to these canonical docs/data rather than invent parallel mappings.

38. **Native diff and exception triage are core workflows.** The system must be able to emit structured spec, sheet, and packet change reports plus a focused exception queue so operators review only ambiguous or divergent work.

39. **Cross-document entity resolution is strategic infrastructure.** Sheet IDs, section IDs, equipment tags, firms, and related project entities must be linkable across drawings, specs, submittals, and revisions.

40. **Project knowledge indexing is a supported downstream surface.** Once entities and normalized outputs exist, the platform should support searchable project-level lookup over extracted records, links, and provenance.

---

## Document Families & Mediums

### Supported Document Types

Conset PDF operates on three primary **mediums**:

1. **Specifications** (Specs)
2. **Drawings** (Construction Documents)
3. **Submittals** (Product Data)

Each medium has unique structure, chrome patterns, and processing requirements.

### Specification Structure

**Sections** follow MasterFormat (CSI):
- Division-Section-Subsection hierarchy (e.g., "23 82 16")
- Three-part format: General, Products, Execution
- Outline-style numbering (1.1.A, 1.1.B, etc.)

**Chrome (Furniture):**
- Headers: Project name, firm logo, project number
- Footers: Date, Section ID, Section Title, Page-in-section counter

**Example footer:**
```
2025-10-01    23 82 16 – HEATING WATER COILS - Page 2 of 3
```

### Drawing Structure

**Sheets** organized by discipline:
- General (G), Architectural (A), Structural (S)
- Mechanical (M), Electrical (E), Plumbing (P)
- Fire Protection (FP), Civil (C)

**Chrome (Furniture):**
- Title blocks (lower-right corner typically)
- Revision blocks (upper-right or triangular corner)
- Sheet ID in footer and/or title block
- Drawing legends, general notes

**Equipment Schedules:**
- Dense tabular data (10-20+ columns)
- Multiple schedules per sheet common
- Rotated text in headers
- Merged cells for titles

### Submittal Structure

**Units** represent individual equipment:
- Cover sheet with submittal metadata
- Per-unit pages with tag, model, specs
- Performance tables, dimension diagrams

**Chrome (Furniture):**
- Repeated form headers/footers
- Project information bands
- Template dividers

---

## Architecture: Open Engine + Paid GUI

### Two-Tier Model

```
┌─────────────────────────────────────────┐
│   Paid GUI (Desktop/Web)                │
│   - One-button workflows                │
│   - Visual overlay review                │
│   - Pattern database management UI       │
│   - Team collaboration                   │
│   - Licensing & billing                  │
└─────────────────────────────────────────┘
                 │
                 │ CLI/API

                 ↓
┌─────────────────────────────────────────┐
│   Open-Source Engine (Apache-2.0)       │
│   - Deterministic parsing                │
│   - PDF extraction (PDFium)              │
│   - Section reconstruction               │
│   - Audit trail generation               │
└─────────────────────────────────────────┘
```

**Monetization:**
- **Engine:** Free, open-source (Apache-2.0), command-line only
- **GUI:** Paid desktop application (one-time or annual license)
- **Web (future):** SaaS subscription model

**Why Open Engine:**
- Builds trust through transparency
- Allows technical validation
- Community contributions improve quality
- Defensive moat against competitors

---

## The Compiler Model

### Analogy

Conset PDF operates like a compiler:

0. **Intake Triage:** Normalize raw inputs → `NormalizedIntakeBundle`
1. **Lexer:** Extract raw layout (spans, bboxes) → `LayoutTranscript`
2. **Parser:** Build semantic tree → `DocumentAST`
3. **Optimizer:** Apply edits, validate → `EditableDocModel`
4. **Code Generator:** Render to PDF → `OutputPDF`

### Why This Works

**Determinism:** Same input + same rules = identical output (no randomness)

**Auditability:** Every transformation logged with provenance

**Testability:** Each stage has clear inputs/outputs, unit testable

**Composability:** Stages can be developed and validated independently

---

## Technology Stack

### Core Dependencies

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| **Language** | Rust | Memory safety, determinism, single-binary deployment |
| **PDF Extraction** | PDFium (via `pdfium-render`) | Industry-standard, Apache-2.0, proven reliability |
| **PDF Generation** | Headless Chrome (via `headless_chrome`) | High-fidelity HTML→PDF, CSS support |
| **PDF Write (page ops)** | `lopdf` | Pure-Rust, MIT-licensed; page-level insert/delete/swap, rotation normalization, bookmark mutation; canonical backend for `PdfStitcher` (Phase 6 — **COMPLETE**) and Intake Triage page normalization (Phase 7) |
| **Local Micro-ML Runtime** | ONNX Runtime (on-device) | Optional confidence boost path for hard edge cases; deterministic-bounded via fixed model/version + logged fusion policy |
| **OCR (raster path)** | On-device OCR engine (pluggable) | Required fallback for raster/low-text PDFs with confidence + provenance tagging |
| **Pattern Matching** | `regex` crate | Deterministic, fast, well-tested |
| **Power-User LLM Validation API** | Provider adapters via contracts | Optional validation/instruction generation path for ambiguous inputs; explicit opt-in and full audit capture |
| **Serialization** | `serde` + `serde_json` | Standard Rust serialization |
| **GUI (desktop)** | Tauri | Rust-native desktop shell with strong backend integration |
| **Testing** | Built-in `cargo test` + golden files | Regression testing |

### Repository Strategy (Monorepo with Hard Boundaries)

Use a **single monorepo** with strict package/crate boundaries.

**Rationale:**
- Backend and GUI contracts evolve rapidly during active implementation
- Atomic cross-boundary refactors are required for reliability
- One CI surface avoids integration drift between repos

**Required structure:**

```
repo-root/
├── apps/
│   ├── backend-cli/         # Rust binary entrypoints (CLI/API host)
│   └── desktop-gui/         # Tauri app + frontend UI
├── crates/
│   ├── engine/              # Deterministic pipeline wrappers and stage orchestration
│   ├── ir/                  # Transcript/IR types and validation
│   ├── pdf-extraction/      # PDF loading and extraction backends
│   ├── audit/               # Audit models and persistence
│   ├── workflows/           # Merge/split/bookmarks/spec patch orchestration
│   ├── contracts/           # Shared request/response/event schemas
│   └── standards-data/      # Embedded standards datasets
├── docs/
│   ├── MASTER_PLAN.md
│   ├── ARCHITECTURE.md
│   ├── DEV_STANDARDS.md
│   ├── AEC_STANDARDS.md
│   └── TRANSCRIPT_ARCHITECTURE.md
└── tests/
  ├── corpus/
  └── integration/
```

**Boundary rules:**
- GUI must depend on backend only through `crates/contracts`
- No GUI imports from `crates/engine` internals
- CLI/API and GUI integration tests required for every contract version bump
- Repo split is optional later, only after contract churn stabilizes

### No Python Runtime

- Python may be used for internal dev tooling (oracle baselines, corpus analysis)
- Python **must never** be a required runtime dependency for end users
- All shipped functionality: pure Rust, single binary

---

## Deterministic Shared Pipeline

### Overview

All mediums (specs, drawings, submittals) share the same extraction pipeline, with medium-specific handlers at appropriate stages.

### Stage 1: Layout Transcript Extraction (Universal)

**Goal:** Extract normalized geometric layout from PDF.

**Implementation:**
```rust
pub struct LayoutTranscript {
    pub pages: Vec<Page>,
    pub metadata: DocumentMetadata,
}

pub struct Page {
    pub index: usize,
    pub width: f32,
    pub height: f32,
    pub spans: Vec<Span>,
}

pub struct Span {
    pub text: String,
    pub bbox: BBox,
    pub font: FontInfo,
    pub flags: SpanFlags,
}

pub struct BBox {
    pub x: f32,      // Normalized [0.0, 1.0]
    pub y: f32,      // Normalized [0.0, 1.0], origin top-left
    pub width: f32,
    pub height: f32,
}
```

**Coordinate System (Critical):**
- **Input:** PDFium uses bottom-left origin, Y increases upward
- **Normalization:** Immediately convert to top-left origin, Y increases downward
- **All coordinates normalized to page dimensions** (0.0 = left/top, 1.0 = right/bottom)

**Invariants (hard gates):**
- Rotation/crop normalization verified
- No impossible bboxes (negative sizes, out-of-page)
- Text layer consistency checks
- All text UTF-8 valid
- **No pass → no build**

**Output:**
- Normalized spans (text, bbox, font, flags, color, rotation)
- Page metadata (dimensions, text layer presence, quality score)
- Quality metrics (char count, whitespace ratio, replacement char count)

### Stage 2: Furniture/Chrome Detection (Medium-Specific)

**Goal:** Identify and exclude project reference matter, not content.

**Chrome types vary by medium:**

| Medium | Chrome | Signal |
|--------|--------|--------|
| **Specs** | Headers, footers, section tags, page counters, addendum marks | Font/position patterns, footer text hashes |
| **Drawings** | Title blocks, revision stamps, legends, sheet IDs | Geometry + known label patterns |
| **Submittals** | Repeated form headers, project bands, dividers | Structure repetition + template patterns |

**Chrome Metadata Extraction and Preservation:**

Chrome is not just noise—it contains critical metadata that must be preserved and reused when regenerating content.

**Metadata to extract:**

**Specs (from headers/footers):**
```rust
pub struct SpecChromeMetadata {
    pub project_id: String,          // "RWB Project No. 25063.00"
    pub project_name: String,        // "Lake Highlands High School"
    pub client: String,              // "Richardson ISD"
    pub date: String,                // "2025-10-01"
    pub section_id: String,          // "23 82 16"
    pub section_title: String,       // "HEATING WATER COILS"
    pub firm_name: String,           // "RWB Consulting Engineers"
    pub firm_logo: Option<ImageRef>, // Logo reference if present
}
```

**Drawings (from title blocks):**
```rust
pub struct DrawingChromeMetadata {
    pub project_id: String,
    pub project_name: String,
    pub sheet_id: String,            // "M6.02"
    pub sheet_title: String,         // "SCHEDULES - MECHANICAL"
    pub discipline: String,          // "Mechanical"
    pub revision: String,            // "Rev 3"
    pub date: String,
    pub firm_name: String,
    pub stamps: Vec<Stamp>,          // Professional seals/stamps
}
```

**Submittals (from form headers):**
```rust
pub struct SubmittalChromeMetadata {
    pub project_id: String,
    pub submittal_number: String,    // "Submittal 23.1"
    pub equipment_type: String,      // "Unit Ventilators"
    pub date: String,
    pub contractor: String,
    pub manufacturer: String,
}
```

**Chrome Reuse Pipeline:**

```rust
// 1. Extract chrome during furniture detection
let chrome = detect_chrome(&transcript)?;
let metadata = extract_chrome_metadata(&chrome)?;

// 2. Process content (exclude chrome regions)
let content_ast = parse_content(&transcript, &chrome.exclusion_regions)?;

// 3. Apply edits to AST
let edited_ast = apply_edits(&content_ast, &edits)?;

// 4. Regenerate content (body only, no chrome)
let regenerated_pages = render_content(&edited_ast)?;

// 5. Reapply chrome with updated metadata
let final_pdf = apply_chrome_template(
    &regenerated_pages,
    &metadata,
    &ChromeUpdateRules {
        update_date: true,          // New date: 2025-10-17
        update_page_numbers: true,  // Recalculate: "Page 2 of 5"
        preserve_project_info: true,// Keep project ID, name, section
        preserve_firm_branding: true,// Keep logo, firm name
    }
)?;
```

**Why This Matters:**

When you regenerate a spec section, the output must look like an **official reissue**, not a Word export. The chrome (headers/footers with project info, firm branding, section identifiers) makes it look professional and maintains visual continuity with the original document.

**Example:**

**Original footer:**
```
2025-10-01    23 82 16 – Heating Water Coils - Page 2 of 3
```

**Regenerated footer (after adding content):**
```
2025-10-17    23 82 16 – Heating Water Coils - Page 2 of 5
     ↑             ↑                                  ↑
  Updated      Preserved                        Recalculated
```

**Algorithm (Deterministic):**
```rust
pub struct FurnitureDetector {
    config: FurnitureConfig,
}

pub struct FurnitureConfig {
    pub header_band: (f32, f32),  // (0.0, 0.15) = top 15%
    pub footer_band: (f32, f32),  // (0.85, 1.0) = bottom 15%
    pub repetition_threshold: f32, // 0.8 = 80% of pages
}

impl FurnitureDetector {
    pub fn detect(&self, transcript: &LayoutTranscript) -> FurnitureRegions {
        // 1. Extract header/footer band spans
        // 2. Hash footer text per page
        // 3. Find repeating pattern (≥80% pages)
        // 4. Extract metadata from pattern
        // 5. Compute confidence
    }
}
```

**Output:** Furniture regions marked; content region extracted; metadata stored.

### Stage 3: Zone Detection

**Goal:** Identify reading regions (columns, whitespace rivers, content breaks).

**Algorithm:**
1. Detect vertical whitespace bands (column separators)
2. Detect horizontal zone breaks (major content boundaries)
3. Order zones left-to-right, top-to-bottom
4. Compute confidence based on consistency

**Output:** Ordered zones per page with confidence scores.

### Stage 4: Line Grouping

**Goal:** Cluster spans into logical lines by baseline proximity.

**Algorithm:**
1. Cluster spans by Y-coordinate (tied to font size, leading)
2. Infer spaces by x-gaps relative to glyph width
3. Assign reading order (left-to-right primary, top-to-bottom secondary)

**Output:** Lines with text, bboxes, font signatures, reading order.

### Stage 5: Block Identification (Medium-Aware)

**Goal:** Group lines into semantic blocks (paragraphs, lists, key-values, tables).

#### 5a. Paragraphs
- Compute line height, paragraph gap, indent clusters
- Deterministic wrap/join + dehyphenation
- Preserve structure cues (not semantic meaning)

#### 5b. Lists / Outlines (Specs)
- Marker grammar: `A.` `1.` `1.1` `(a)` roman, bullets
- Hanging indent ownership
- Indent stack nesting validation

#### 5c. Key-Value Forms (Submittals)
- Label/value pairing by geometry + colon patterns + alignment
- Preserve numeric/unit extraction
- Keep provenance always

#### 5d. Tables (Submittals + Specs + **Drawings**)
- Two modes: **ruled-line tables** (when primitives exist) + **alignment-inferred tables** (text-aligned)
- Grid building: X/Y clustering for cell boundaries
- Multi-page tables (repeated headers, carryover)
- Header detection and validation
- **Multi-table per page detection:** Schedule sheets often contain 2-4 distinct tables
- **Table boundary detection:** Identify where one table ends and another begins

**Drawing-specific table challenges:**
- **Equipment schedules:** Dense tabular data with many columns (10-20+)
- **Rotated text in headers:** Column headers sometimes vertical/rotated 90°
- **Merged cells:** Schedule titles often span multiple columns
- **Nested tables:** Remarks/notes sections within larger schedule
- **Mixed orientation:** Some schedules horizontal, some vertical on same page

#### 5e. Sequence/Schedule Extraction (Specs)
- Detect section order lists (TOC, Division 00)
- Extract section numbers, titles, page numbers
- Parse "NOT USED" exclusions
- Build canonical spec book order

**Output:** Structured blocks with confidence, provenance, and overlays.

---

## Medium-Specific Processing

### Specifications: Section Regeneration

#### Goal
"Make spec addenda painless." Apply surgical edits to individual sections, regenerate only those sections, stitch back into original PDF.

#### Processing Pipeline

**S1: Section Segmentation (Footer-First Oracle)**
- Detect section boundaries using footer section ID
- Validate with page-in-section counters ("Page 2 of 3")
- Build section index (section ID → page range)
- Handle edge cases (missing footers, scanned pages)

**S2: Section Parsing**
- Parse section into hierarchical AST
- Detect Parts, Articles, Paragraphs
- Parse outline numbering (1.1.A, 1.1.B, etc.)
- Infer nesting by indent

**S2a: Deterministic spec heading detection (vector-first)**
- Group spans into lines by y-clustering and stable baseline thresholds.
- Compute line features (font delta vs body, caps ratio, indent, leading whitespace gap).
- Detect section starts using CSI patterns (for example `SECTION 23 82 16`) plus typography/spacing rules.
- Segment section blocks until next heading boundary and emit heading confidence evidence.

**S3: Edit Operations**
- Insert, delete, replace paragraphs
- Renumber automatically when needed
- Validate edits (target exists, no conflicts)

**S4: Section Regeneration**
- Render AST to HTML with CSS formatting
- Convert HTML to PDF via headless Chrome
- Apply chrome template with metadata
- Preserve formatting (fonts, spacing, indentation)

**S5: PDF Stitching**
- Delete old section pages
- Insert new section pages
- Preserve unchanged pages (byte-identical when possible)
- Update bookmarks
- Generate structured section diff/change report and exception list

**Key Insight: Footer-First is Oracle**

Spec footers are the ground truth for section boundaries. Headers can be wrong (delayed updates, template errors), but footers are authoritative because they're programmatically generated during original production.

**Canonical Output Format:**

Regenerated sections must look indistinguishable from original sections. The chrome (headers/footers) makes this possible.

---

### Drawings: Sheet Inventory & Replacement

#### Goal
"Stop manually merging addenda sheets." Automated sheet replacement with audit trail.

#### Processing Pipeline

**D1: Sheet Inventory Extraction**
- Extract sheet IDs from title blocks and/or footers
- Parse sheet names
- Detect discipline prefixes (G, M, E, etc.)
- Build canonical sheet list

**D1a: Deterministic title-block localization (vector-first)**
- Compute drawing frame by finding the largest thin-line rectangle near page margins.
- Define four corner candidate bands inside the frame (policy-tunable percentage bands).
- Score each corner by table-structure density (axis-aligned lines, intersections, rectangular cells).
- Select highest-scoring corner as title block ROI, then tighten bbox by connected components or bbox merge.
- Emit explicit diagnostics (`TB_FRAME_MISSING`, `TB_GRID_WEAK`, `TB_MULTI_CORNER_CONFLICT`) with ranking evidence.

**D1b: Deterministic sheet field extraction inside title block**
- Extract text objects intersecting title-block ROI.
- Normalize features: relative x/y inside block, font size, caps ratio, bold/regular.
- Score sheet number and sheet name candidates by pattern + typography + relative position.
- Persist selected relative ROIs as auto-learned template candidate for firm/layout signature.

**D2: Sheet Matching**
- Parse sheet IDs from original set (by footer/title block)
- Parse sheet IDs from addendum
- Match by ID
- Handle sheet renaming ("Formerly named DG1.1" → "G1.11")

**D3: Sheet Replacement**
- Replace matching sheets (verbatim page swap)
- Preserve unchanged sheets (byte-identical)
- Update bookmarks
- Generate replacement report
- Generate sheet delta report with matched/added/removed/renamed sheets and unresolved exceptions

**D4: Schedule Extraction (Optional)**
- Extract tabular data from equipment schedules
- Handle multi-table pages
- Handle rotated text, merged cells
- Export to schema-versioned JSON/CSV/XML with field-level provenance

**No regeneration.** Drawings are extracted and indexed, not re-typeset.

---

### Submittals: Data Extraction

#### Goal
"Kill two days of copy/paste." Extract accurate, normalized data that AEC professionals can trust.

#### Processing Pipeline

**U1: Unit Boundary Detection**
- Identify cover/unit report/performance report sections
- Segment by tag name or equipment type
- Assign page ranges to each unit

**U2: Per-Unit Header Extraction**
- Extract tag, model, project, date, quantity
- Record confidence + provenance

**U3: Per-Unit Data Parsing**
- Parse unit dimensions, performance specs, sound data
- Normalize units and values
- Aggregate into per-unit record set

#### Canonical Export Format (Tidy)

| Column | Meaning |
|--------|---------|
| `packet_name` | Submittal packet ID |
| `revision_id` | Revision number/date |
| `item_tag` | Equipment tag |
| `equipment_type` | HVAC, plumbing, etc. |
| `section` / `category` | Spec division + spec section |
| `field` | Field name (e.g., "Cooling Airflow CFM") |
| `value_raw` | Raw extracted text |
| `value_num` | Numeric value (optional) |
| `unit` | Unit (optional) |
| `page` | Source page number |
| `bbox` | Bounding box (optional) |
| `confidence` | 0.0–1.0 |
| `source` | "table" or "keyvalue" |
| `conflict_flags` | List of conflicts if any |

#### Outputs
- **EquipmentDataset:** per-unit records in tidy format
- **PerformanceMetrics:** per-unit performance summaries
- **QualityReport:** parsing confidence by unit
- **AuditTrail:** event log with field-level provenance

---

## Pattern Database System

### Philosophy

We scale to new document families **without fragility** by using versioned, validated pattern databases.

**Pattern Database Structure:**

```json
{
  "version": "1.0.0",
  "medium": "specs",
  "firm_profile": "rwb_consulting_engineers",
  "patterns": {
    "footer_section_id": {
      "regex": "^(\\d{2}\\s\\d{2}\\s\\d{2})\\s+–",
      "confidence_threshold": 0.95,
      "examples": [
        "23 82 16 – Heating Water Coils",
        "00 01 10 – Table of Contents"
      ]
    },
    "page_counter": {
      "regex": "Page\\s+(\\d+)\\s+of\\s+(\\d+)$",
      "confidence_threshold": 0.98,
      "examples": [
        "Page 2 of 3",
        "Page 1 of 15"
      ]
    }
  }
}
```

**Versioning:**
- Pattern database versions tracked in Git
- Engine version locked to pattern DB version
- Audit bundle includes pattern DB version used

**Validation:**
- Every pattern includes test cases
- Torture corpus validates pattern coverage
- CI/CD fails if pattern changes break corpus

---

## Audit Trail & Quality Framework

### Audit Bundle

Every run generates an **audit bundle** containing:

1. **Input Metadata:** PDF hash, engine version, pattern DB version, timestamp
2. **Layout Transcript:** Complete extracted layout (JSON)
3. **Visual Overlays:** Annotated page images showing:
   - Furniture regions (headers/footers highlighted)
   - Section boundaries (colored boxes)
   - Detected blocks (paragraphs, lists, tables)
4. **Decision Log:** Every decision with:
   - What was decided (e.g., "Section 23 82 16 spans pages 45-58")
   - Why (e.g., "Footer section ID matched on 98% of pages")
   - Confidence score (0.0–1.0)
   - Evidence (span IDs, bbox coordinates)
5. **Conflicts:** Any ambiguities or failures
6. **Output Metadata:** Pages replaced, operations applied, timing

**Audit Bundle Format:**
```
audit-bundle-2025-10-17-143022/
├── manifest.json
├── transcript.json
├── overlays/
│   ├── page-000-furniture.png
│   ├── page-000-sections.png
│   ├── page-001-furniture.png
│   └── ...
├── decisions.log
├── conflicts.json
└── metrics.json
```

### Quality Framework

**Confidence Scoring:**

Every extraction/decision includes a confidence score (0.0–1.0):

- **1.0:** Perfect certainty (e.g., footer matches known pattern 100%)
- **0.9:** High confidence (e.g., 95% of pages have consistent pattern)
- **0.7:** Moderate confidence (e.g., pattern matches but with minor inconsistencies)
- **0.5:** Low confidence (e.g., ambiguous structure, multiple interpretations)
- **<0.5:** Needs Review (manual intervention required)

**Escalation Rules:**

- Confidence ≥0.9: Auto-apply
- 0.7 ≤ Confidence <0.9: Flag for review, provide suggestion
- Confidence <0.7: Needs Review, show alternatives

**No silent failures.** If confidence is low, emit "Needs Review" state with visual overlays showing the ambiguity.

---

## Implementation Roadmap

### Timeline Overview

**Alpha (Weeks 0-12):** End-to-end spec addenda workflow works, supervised use

**Beta (Weeks 13-18):** Torture corpus ≥95% pass, first paying customer

**V1.0 (Weeks 19-40):** Production hardening, GUI, team features

### Dependency Graph

```
Phase 0: Scaffolding (Week 1) ← COMPLETE
    ↓
Phase 0.5: Pattern Dev Tool (Weeks 2-3) ← COMPLETE
    ↓
Phase 1: Layout Extraction (Weeks 4-5) ← COMPLETE
    ↓
Phase 2: Furniture/Sections (Weeks 6-7) ← COMPLETE
    ↓
Phase 3: Paragraph Parsing (Weeks 8-9) ← COMPLETE + HARDENED
    ↓
Phase 4: Edit Operations (Week 10) ← COMPLETE
    ↓
Phase 5: Regeneration (Weeks 11-12) ← COMPLETE
    ↓
Phase 5.5: Layout Geometry + Font Typography (supplement) ← COMPLETE
    ↓
Phase 6: PDF Stitching (Week 13) ← COMPLETE
    ↓
Phase 7: End-to-End (Week 14) ← COMPLETE
    ↓
Phase 8: Production Hardening (Weeks 15-16) ← ✅ ALL SPRINTS COMPLETE (8.0–8.7)
    ↓
Phase 9: Drawing Sheets (Weeks 17-18)
    ↓
Phase 10: Submittals (Weeks 19-20) ← BETA COMPLETE
    ↓
Phase 11+: GUI & Polish (Weeks 21+) ← V1.0
```

---

### Phase 0 — Foundation & Tooling (Week 1)

**Goal:** Set up project structure, build system, and testing framework.

**Deliverables:**
- ☐ Rust project scaffolding (Cargo workspace)
- ☐ Monorepo scaffolding with `apps/` and `crates/` boundaries
- ☐ `crates/contracts` initialized as the canonical backend/frontend contract package
- ☐ PDFium integration (`pdfium-render` crate)
- ☐ CI/CD pipeline (GitHub Actions)
- ☐ Torture corpus repository structure
- ☐ Test harness (integration tests, golden files)
- ☐ Documentation structure (inline docs + external guides)

**Output:** `cargo test` runs successfully, PDFium can load a PDF.

---

### Phase 0.5 — Pattern Development Tool (Weeks 2-3)

**Goal:** Build the primary development tool for pattern creation and validation.

**Why Early:**
- You need this tool to **build patterns** in Phases 2-4
- Without it, you're flying blind (no visual feedback on pattern matching)
- It's infrastructure, not polish—it's a **developer tool**, not customer-facing
- Building it early prevents rework later

**Deliverables:**
- ✅ CLI tool for pattern development
- ✅ Visual overlay system (show matched regions on PDF pages)
- ✅ Pattern testing framework (regex → PDF → visual confirmation)
- ✅ Pattern validation suite (test patterns against sample PDFs)
- ✅ Debug output (show confidence scores, matched text, bboxes)

**Status: COMPLETE — April 5, 2026.** All deliverables shipped. See `docs/PHASE_05_HANDOFF.md` for the downstream handoff and Appendix A for implementation history.

**Core Features:**

```bash
# Inspect a PDF (page geometry, span counts, text quality)
cargo run --bin pattern-dev -- inspect test.pdf

# Test a heuristic family against a single PDF
cargo run --bin pattern-dev -- test-pattern \
  test.pdf \
  --family footer-section-id \
  --output-dir debug/

# Batch validate against corpus tier 1
cargo run --bin pattern-dev -- validate-corpus \
  --tier 1 \
  --output-dir audit_output/run1/

# Output (test-pattern):
# ✓ Page 0: Matched "23 82 16" (confidence: 0.97)
# ✓ Page 1: Matched "23 82 16" (confidence: 0.97)
# ✗ Page 2: No match — failure_reason: NO_MATCH
#
# Overlays: debug/<pdf-stem>/page-0000-footer-section-id.png
# Sidecars: debug/<pdf-stem>/page-0000-footer-section-id.json
```

**Note:** `--family` accepts: `footer-section-id`, `page-counter`, `header-band`, `title-block-anchor`, `roi-candidate`, `spec-heading`. The old `--pattern-type` and `--output-overlays` flags were superseded during Phase A. See `docs/PHASE_05_HANDOFF.md` Section 2.1 for the canonical command reference.

**Pattern Development Workflow:**

1. **Extract Sample:** Load a sample PDF
2. **Annotate Regions:** Visually mark expected chrome regions (header/footer bands)
3. **Test Pattern:** Apply regex, see visual overlay
4. **Iterate:** Adjust regex, re-test, compare overlays
5. **Validate:** Test pattern against 5-10 sample PDFs from torture corpus
6. **Save Pattern:** Add validated pattern to pattern database

**Visual Output Example:**

```
Page 0:
┌────────────────────────────────────┐
│ [GREEN BOX] Header detected        │  ← Matched header pattern
│                                    │
│ Content region (excluded from      │
│ pattern matching)                  │
│                                    │
│ [GREEN BOX] Footer: "23 82 16 – "  │  ← Matched footer pattern
└────────────────────────────────────┘

Page 2:
┌────────────────────────────────────┐
│ [YELLOW BOX] Header detected       │  ← Low confidence
│                                    │
│ Content region                     │
│                                    │
│ [RED BOX] Footer: No match         │  ← Pattern failed
└────────────────────────────────────┘
```

**Why This Tool Is Critical:**

- **Speeds up pattern development:** Visual feedback loop is 10x faster than code-test-debug
- **Validates patterns before integration:** Catch pattern bugs early
- **Documents pattern coverage:** Shows which PDFs pass/fail for each pattern
- **Enables non-programmer contributions:** Architects can help refine patterns visually

**Definition of Done:**
- Pattern dev tool can load PDFs, apply patterns, generate overlays
- Visual overlays clearly show matched/unmatched regions
- Confidence scores computed and displayed
- Pattern validation suite runs against sample PDFs
- Documentation includes pattern development guide

---

### Phase 1 — Layout Transcript Extraction (Weeks 4-5)

**Goal:** Get normalized layout extraction working end-to-end.

**Deliverables:**
- ✅ LayoutTranscript types defined
- ✅ PDFium text extraction with bbox coordinates
- ✅ Coordinate normalization (PDF bottom-left → display top-left)
- ✅ Invariant validation (no negative sizes, out-of-bounds, etc.)
- ✅ Debug visualization (draw bboxes on page images)
- ✅ Integration tests on 5 sample PDFs

**Test:**
```bash
cargo run -- extract test.pdf -o transcript.json
cargo run -- visualize transcript.json -o debug/
```

**Output:** `debug/page-000.png` with bboxes drawn, coordinate system verified.

**Definition of Done:** 
- ✅ Transcript JSON output is clean and normalized
- ✅ All 5 test PDFs extract successfully
- ✅ No coordinate inversions (headers at top, footers at bottom)
- ✅ All invariants pass

---

### Phase 2 — Furniture Detection & Section Segmentation (Weeks 6-7)

**Goal:** Build the index. Map section IDs to page ranges.

**Deliverables:**
- ✅ FurnitureDetector implementation (uses patterns from Phase 0.5)
- ✅ Chrome metadata extraction (project ID, dates, section info)
- ✅ Pattern database integration
- ✅ Section segmentation algorithm (footer-first oracle)
- ✅ Page-in-section counter detection and validation
- ✅ Coverage validation
- ✅ Debug overlays (furniture regions marked, metadata displayed)

**Test:**
```bash
cargo run -- segment test.pdf -o index.json
cargo run -- visualize-segments test.pdf index.json -o debug/
```

**Output:**
```json
{
  "chrome_metadata": {
    "project_id": "RWB Project No. 25063.00",
    "project_name": "Lake Highlands High School",
    "firm": "RWB Consulting Engineers",
    "date": "2025-10-01"
  },
  "sections": [
    {
      "section_id": "23 00 00",
      "section_title": "HEATING, VENTILATING, AND AIR CONDITIONING (HVAC)",
      "start_page": 0,
      "end_page": 15,
      "page_count": 16,
      "page_counter_detected": true,
      "confidence": 0.98,
      "chrome_metadata": {
        "section_id": "23 00 00",
        "section_title": "HVAC"
      }
    }
  ],
  "coverage": {
    "pages_total": 283,
    "pages_tagged": 283,
    "pages_missing_footer": 0,
    "coverage_ratio": 1.0
  }
}
```

**Definition of Done:**
- ✅ Index JSON shows all sections correctly segmented
- ✅ Chrome metadata extracted and stored
- ✅ Coverage ≥95% on torture corpus
- ✅ Footer patterns match expected format
- ✅ Page-in-section counters validate boundaries
- ✅ No section boundary conflicts

---

### Phase 3 — Paragraph Parsing & AST Construction (Weeks 8-9)

**Goal:** Parse sections into hierarchical AST.

**Status: COMPLETE — April 5, 2026.** See bug-fix sprint note below.

**Deliverables:**
- ✅ Line grouping (baseline clustering)
- ✅ Paragraph detection
- ✅ Outline marker parsing (A., 1., a., i.)
- ✅ Nesting inference (indent-based)
- ✅ AST construction (Section → Part → Article → Paragraph)
- ✅ Debug output (AST visualization)

**Test:**
```bash
cargo run -- parse test.pdf --section "23 82 16" -o ast.json
cargo run -- visualize-ast ast.json -o debug/ast.html
```

**Output:** Full hierarchical AST of Section 23 82 16 with correct nesting.

**Definition of Done:**
- ✅ AST accurately represents section structure
- ✅ Outline numbering parsed correctly (2.7.A, 2.7.B, etc.)
- ✅ Nesting levels inferred correctly
- ✅ Part/Article/Paragraph boundaries detected (all 5 nesting levels)
- ✅ Works on 80%+ of torture corpus sections (0.2% unclassified node rate on full 571-page corpus)
- ✅ 19/19 CLI integration tests pass

**Bug-fix sprint (post-Phase 3, April 5, 2026):**

A second pass hardened the parser after real-corpus inspection revealed:

1. **Span x-sort** — PDFium returns spans in content-stream order; sorted by (y, x) before line clustering.
2. **LINE_Y_EPSILON** raised from 0.005 → 0.012 — dashes on the same visual line had y-delta up to 0.006.
3. **Cluster-based section ID detection** in segment engine — footer IDs are split across 2–3 spans; merged before regex match. Also raised FOOTER_Y threshold from 0.85 → 0.90 to exclude body cross-references.
4. **Noise-only line skipping** — lone punctuation spans (dashes, bullets) discarded before they could anchor all subsequent content as continuation text.
5. **Article regex tightened** — major number must be ≥ 1 (kills `0.x` decimal matches); title must begin with uppercase letter (kills measurement strings).
6. **`inject_missing_parts` recovery pass** — synthetic PART nodes injected when article major number jumps to a part that was never explicitly opened (recovers from kerning-broken PART headers and segmenter page-boundary errors).

Post-hardening results on `SPEC_RWB_LHHS_ALL_ORG.pdf` (571 pages, 89 sections):
- 89 sections detected
- 7,971 total nodes; 0.2% unclassified rate
- 0/70 structured sections with wrong-PART nesting (was 13/70)
- Node distribution: 185 part, 779 article, 2983 paragraph, 2698 sub_paragraph, 1052 sub_sub_paragraph, 258 sub_sub_sub_paragraph, 16 unclassified (all front-matter/forms, expected)

---

### Phase 4 — Edit Operations (Week 10)

**Goal:** Apply surgical edits to AST.

**Status: COMPLETE — April 5, 2026.**

**Deliverables:**
- ✅ SectionEditor implementation
- ✅ Insert operation (insert_after with renumbering)
- ✅ Delete operation
- ✅ Replace operation
- ✅ Paragraph renumbering logic (all 6 CSI levels, canonical scheme locked)
- ✅ Validation (target exists, no conflicts — pre-flight before any mutation)

**Test:**
```bash
# Parse a spec section first, then apply edits to the AST:
cargo run --bin backend-cli -- parse spec.pdf -o ast.json
cargo run --bin backend-cli -- edit \
  --input ast.json \
  --operations ops.json \
  --output edited-ast.json
```

`ops.json` example:
```json
{
  "description": "Insert paragraph after 2.7.B",
  "operations": [{
    "op": "insert_after",
    "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.7", "B."] },
    "new_node": { "tag": "paragraph", "marker": "X.", "text": "Provide return air damper.", "page_index": 0, "level": 2, "children": [] }
  }]
}
```

**Output:** Edited `ParsedDocument` JSON with renumbered siblings (B.→B., inserted→C., old C.→D., …).

**Definition of Done:**
- ✅ Insert/delete/replace operations work correctly
- ✅ Renumbering cascades properly (C→D, D→E) at all 6 CSI nesting levels
- ✅ Validation catches invalid targets (SectionNotFound, PathNotFound, LevelMismatch pre-flight)
- ✅ Edited AST passes structural validation
- ✅ `OperationStarted`/`OperationEnded` audit events emitted (G-006 closed)
- ✅ 7/7 Phase 4 integration tests pass; 26/26 total integration tests pass

---

### Phase 5 — Section Regeneration (HTML → PDF) (Weeks 11-12)

**Goal:** Render AST to PDF via HTML/CSS, with chrome reapplication.

**Status: COMPLETE — April 6, 2026.**

**Deliverables:**
- ✅ SectionRenderer implementation (`crates/engine/src/render/mod.rs`)
- ✅ HTML body template with CSS class mapping (`crates/engine/src/render/body.rs`)
- ✅ Chrome subprocess integration via `std::process::Command` (not `headless_chrome` crate — lighter, more stable)
- ✅ Chrome template system — CSS Paged Media `@page` margin boxes for running headers/footers (`crates/engine/src/render/chrome.rs`)
- ✅ Formatting rules (font family/size, page size, indentation per OutlineTag level)
- ✅ `SpecChromeMetadata` IR type + `RenderConfig`, `PageSize`, `RenderResult`, `RenderError` (`crates/ir/src/render.rs`)
- ✅ `regenerate` CLI subcommand with `--ast`, `--chrome-metadata`, `--output`, `--section`, `--dry-run`, `--font`, `--font-size`
- ✅ `WorkflowOperation::Regenerate` contract variant
- ✅ Dry-run path — assembles full HTML, skips Chrome subprocess entirely
- ✅ G-008 accepted-closed (Document/Element stubs, no consumer defined)
- ✅ G-010 closed (4 behavioral audit tests: JSON round-trip, event ordering, count/clear, iter)
- ⚠️ Visual comparison tests deferred — Chrome not available in this environment; `cli_regenerate_produces_pdf` test is `#[ignore]` pending local Chrome install

**Test:**
```bash
cargo run --bin backend-cli -- regenerate \
  --ast edited-ast.json \
  --chrome-metadata chrome.json \
  --output section-new.pdf
```

**Chrome Metadata Input:**
```json
{
  "project_id": "RWB Project No. 25063.00",
  "project_name": "Lake Highlands High School",
  "section_id": "23 82 16",
  "section_title": "Heating Water Coils",
  "date": "2025-10-17",
  "firm": "RWB Consulting Engineers"
}
```

**Output:** PDF of regenerated section with:
- Headers showing project info (`@top-center`: firm | project name)
- Footers showing date, section ID + title, and `Page N of M` (`@bottom-left`/`@bottom-right`)
- Content formatted per OutlineTag CSS classes (`csi-part`, `csi-article`, `csi-para`, `csi-sub1–3`, `csi-body`)

**Definition of Done:**
- ✅ AST → HTML body fragment (all 7 OutlineTag variants mapped to CSS classes)
- ✅ HTML → full document with CSS Paged Media `@page` rules (firm/project/section/date chrome)
- ✅ All chrome metadata fields HTML-escaped before injection
- ✅ `SectionRenderer::dry_run()` path builds full HTML, no Chrome invocation — used by CI and `--dry-run`
- ✅ Chrome binary discovery: `CHROME_PATH` env var, then common Windows/Linux/macOS system paths (including Brave)
- ✅ `WorkflowOperation::Regenerate` emits `OperationStarted`/`OperationEnded` audit events
- ✅ 4/4 non-Chrome integration tests pass (dry-run, missing-AST, missing-section, invalid-chrome-metadata)
- ✅ Full Chromium round-trip test (`cli_regenerate_produces_pdf`) passes using Brave browser (42 s, real SPEC PDF rendered to `%PDF`-valid output)
- ✅ Visual inspection of output PDF: headers/footers with CSS `@page` margin boxes confirmed present

---

### Phase 5.5 — Layout Geometry Capture (Supplement) (April 6, 2026)

**Goal:** Thread per-line x-position, per-section typography measurements, and full font metadata (weight, italic flag, dominant typeface) from extraction and parse stages through the render pipeline, replacing hardcoded CSS values with measured ones.

**Status: COMPLETE — April 6, 2026.**

**Motivation:** The original Phase 5 renderer used hardcoded indentation levels and a `config.font_size_pt` scalar. Additionally, `Span.font_weight` was silently hardcoded to 400 even though PDFium exposes actual weight values, and italic/typeface information was discarded at extraction time. Capturing all available typographic metadata makes rendered output faithful to the original's indentation, typography, and typeface.

**Deliverables:**
- ✅ `SectionLayout` struct added to `crates/ir/src/ast.rs` — `body_left: f64`, `body_right: f64`, `font_size_pt: f64`, `line_gap_norm: f64`; derived as median-aggregated measurements of body spans within a section
- ✅ `x_indent: f64` field added to `AstNode` — leftmost span x-coordinate for the visual line; `#[serde(default)]` for backward compatibility
- ✅ `layout: Option<SectionLayout>` field added to `SectionAst` — `#[serde(default)]` for backward compatibility; populated during parsing
- ✅ `cluster_lines()` in `crates/engine/src/parse.rs` extended to return `(text, page_index, x_min)` — `x_min` is the leftmost bbox.x of all spans on the line
- ✅ `compute_section_layout()` helper added to `parse.rs` — derives `SectionLayout` from body spans using median font size, min/max x-range, median normalized y-gap, and modal font name via `modal_font_name()`
- ✅ `modal_font_name()` helper added to `parse.rs` — frequency-counts font names via `HashMap`, returns most-frequent; used to populate `SectionLayout.body_font_name`
- ✅ `font_weight: f32` and `is_italic: bool` added to `SpanData` (`crates/pdf-extraction/src/types.rs`) — `font_weight` from `text_obj.font().weight()` (`PdfFontWeight` enum → f32 100–900; default 400 on error); `is_italic` from `text_obj.font().is_italic()`; previously neither was extracted
- ✅ `is_italic: bool` added to `Span` IR type (`crates/ir/src/types.rs`) — `#[serde(default)]`; `Span::new()` initializes to `false`; populated by `span.is_italic = span_data.is_italic` in extraction pipeline
- ✅ `Span.font_weight` wired — was hardcoded to 400 in `Span::new()` and never overwritten; now set from `f64::from(span_data.font_weight)` in `crates/engine/src/pipeline/extraction.rs`
- ✅ `body_font_name: String` added to `SectionLayout` — modal font family name among body spans; `#[serde(default = "default_font_name")]` defaults to `"Unknown"` for backward compatibility
- ✅ `build_full_html()` uses `SectionLayout.body_font_name` for CSS `font-family` when it is a real name (not `"Unknown"`); falls back to `config.font_family`
- ✅ `build_body_html()` in `crates/engine/src/render/body.rs` accepts `layout: Option<&SectionLayout>` — emits `style="margin-left: Xin"` inline CSS per node using `(x_indent - body_left) * 8.5` mapping
- ✅ `build_full_html()` in `crates/engine/src/render/chrome.rs` accepts `layout: Option<&SectionLayout>` — uses `layout.font_size_pt` for CSS `font-size` and derives `line-height` as `1.0 + line_gap_norm * 6.0`; falls back to `config.font_size_pt` / 1.4 when no layout present
- ✅ `SectionLayout` exported from `crates/ir/src/lib.rs`
- ✅ D-032 + D-033 recorded in decision-log: measured geometry and font typography extraction; serde defaults maintain backward compatibility with existing JSON artifacts

**Test:**
All existing 30/30 integration tests and 43+13 unit tests pass unchanged — backward-compatible serde defaults mean no fixture updates were required.

**Definition of Done:**
- ✅ `SectionAst.layout` populated for every section parsed from a real PDF, including `body_font_name`
- ✅ `AstNode.x_indent` populated for every node
- ✅ `Span.font_weight` reflects actual PDFium-reported weight (no longer hardcoded 400)
- ✅ `Span.is_italic` reflects actual PDFium-reported italic flag
- ✅ `build_body_html()` emits measured `margin-left` per node
- ✅ `build_full_html()` uses measured `font-size`, `line-height`, and `font-family` when layout is present
- ✅ All call sites updated (`render/mod.rs`, test fixtures in `edit.rs` for both engine and IR crates)
- ✅ Backward-compatible: all new fields use `#[serde(default)]`; existing JSON files deserialize without error
- ✅ 30/30 CLI integration tests pass; 43/43 engine unit tests pass; 13/13 IR unit tests pass

---

### Phase 6 — PDF Stitching & Writeback (Week 13)

**Goal:** Replace section pages in original PDF using lopdf write path.

**Status: COMPLETE — April 7, 2026.**

**Deliverables:**
- ✅ `PdfStitcher` implementation (`crates/engine/src/stitch.rs`) — stateless struct with full `stitch(plan: &StitchPlan) -> Result<StitchResult, StitchError>` algorithm
- ✅ Page replacement logic: load original + replacement PDFs via `lopdf::Document::load()`; renumber replacement objects with `renumber_objects_with()` to avoid ID collisions; copy objects into original with `doc.objects.extend()`
- ✅ `/Pages` root `/Kids` splice: `sorted_page_ids()` extracts ordered `Vec<ObjectId>`; `splice_page_tree()` rebuilds the flat `/Kids` array as `original[..del_start] + replacement_pages + original[del_end+1..]`; `/Parent` updated on all replacement pages
- ✅ Verbatim copy of unchanged pages: removed section's `/Page` objects deleted from `doc.objects`; pages outside the section range are never touched
- ✅ Bookmark preservation: `fixup_bookmarks()` two-pass scan — first pass collects object IDs with deleted-page `/Dest` array references; second pass rewrites those to `[first_repl_page_id, /Fit]`
- ✅ Validation: `validate_unchanged_present()` checks all non-section page object IDs remain in `doc.objects`; emits warnings in `StitchResult.warnings` if any are missing
- ✅ `StitchPlan`, `StitchResult`, `StitchError` IR types in `crates/ir/src/stitch.rs`; exported from `crates/ir/src/lib.rs`
- ✅ `WorkflowOperation::Stitch` contract variant added to `crates/contracts/src/lib.rs`
- ✅ `stitch` CLI subcommand in `apps/backend-cli/src/main.rs` — `--input`, `--segment-index`, `--section`, `--replacement`, `--output`, `--dry-run`
- ✅ `apps/backend-cli/src/handlers/stitch.rs` — reads metadata keys; loads `SegmentIndex` from JSON; builds `StitchPlan`; calls `PdfStitcher::stitch()`; emits `OperationStarted`/`OperationEnded` audit events; all 9 CLI handlers now have audit wiring
- ✅ `workspace: rust-version` bumped from `1.82` → `1.85` to satisfy lopdf MSRV; `lopdf = "0.40.0"` added to `[workspace.dependencies]`
- ✅ 8 unit tests in `crates/engine/src/stitch.rs` (3 pure-logic `resolve_section_range_*` tests + 5 lopdf-based with temp PDFs: dry-run, page count, last-section replace, section-not-found, missing-original, missing-replacement)
- ✅ 6 Phase 6 integration tests appended to `apps/backend-cli/tests/cli_integration_test.rs`

**Test:**
```bash
cargo run --bin backend-cli -- stitch \
  --input original.pdf \
  --segment-index segment-index.json \
  --section "23 82 16" \
  --replacement section-new.pdf \
  --output output.pdf
```

**Output:** Final PDF with target section pages replaced; all other pages unchanged.

**Definition of Done:**
- ✅ Section pages replaced correctly (del_start..=del_end removed; replacement pages spliced in)
- ✅ Unchanged pages are verbatim (untouched object entries)
- ✅ Bookmarks re-routed when they referenced deleted pages
- ✅ Output passes `%PDF` header validation
- ✅ Dry-run writes nothing (skips `doc.save()`)
- ✅ `StitchResult` reports `pages_removed`, `pages_inserted`, `total_pages_before`, `total_pages_after`, `bookmarks_updated`, `warnings`
- ✅ `OperationStarted`/`OperationEnded` audit events emitted by handler
- ✅ 36/36 CLI integration + 53/53 engine unit + 13/13 IR unit tests pass

---

### Phase 7 — End-to-End Workflow (Week 14)

**Goal:** Single command applies addendum edits.

**Deliverables:**
- ✅ Unified CLI command (`apply-addendum` subcommand in `backend-cli`)
- ✅ JSON addendum format (`AddendumManifest` IR types in `crates/ir/src/addendum.rs`)
- ✅ End-to-end processing pipeline (`SpecsPatchOrchestrator::run()` in `crates/engine/src/specs_patch.rs`)
- ✅ Comprehensive error handling (partial-success semantics; per-section failures tracked; error rollup in `AddendumResult`)
- ✅ Audit bundle generation (change-report.json written to `--audit-bundle` directory; `OperationStarted`/`OperationEnded` events emitted)

**Test:**
```bash
cargo run -- apply-addendum \
  --original specs-rev0.pdf \
  --addendum addendum-3.json \
  -o specs-rev1.pdf \
  --audit-bundle audit/
```

**Output:** 
- Updated PDF with edits applied (stitch order: last-to-first to preserve page indices)
- Audit bundle with change report, logs, metrics, per-section outputs
- Change report (which sections modified, which pages replaced, succeeded/failed counts)

**Definition of Done:**
- ✅ End-to-end workflow works on structured addendum
- ✅ All edits applied correctly (edit operations mapped through parse/edit/render pipeline)
- ✅ Section boundaries preserved (extract from original, identify target section by ID, apply edits, regenerate, stitch)
- ✅ Unchanged sections verbatim copied (stitcher preserves out-of-range pages unchanged)
- ✅ Audit bundle is complete and readable (per-section results in change-report.json)
- ✅ **Ready for internal testing (ALPHA COMPLETE)**

---

### Phase 8 — Production Hardening (Weeks 15-16)

**Goal:** Production-ready engine.

**Deliverables:**
- ✅ Comprehensive error handling (8.1.A/B/D)
- ✅ Memory safety — crash containment, caps (8.1.A/C/H)
- ✅ Performance benchmark (8.4.A — `apply_addendum_benchmark_large_spec`: <10 s on 571-page spec)
- ✅ Torture corpus validation ≥95% on SPEC tier (8.3.C — DWG/NAR/SUB excluded by design)
- ✅ Structured diagnostics audit bundle (`diagnostics.jsonl`, 8.1.E–G)
- ✅ Intake triage Stage 0 — rotation normalization + `IntakeBundle` contract (8.2)
- ✅ Architecture constraint documented — full-extraction O(n\_pages), Phase 9 mitigation path (8.4.B)
- ✅ Metrics dashboard roll-up (`metrics.json` in every audit bundle, schema `metrics/v1` — 8.5)
- ✅ Pattern DB as versioned JSON (`crates/engine/src/patterns/`, `default.json` v1.0.0, `pattern_db_version` locked in `change-report.json` — 8.6)
- ✅ User documentation — `docs/CLI_REFERENCE.md` (11 subcommands + error codes) + `docs/WORKFLOW_APPLYADDENDUM.md` (end-to-end tutorial + manifest JSON reference — 8.7)

**Definition of Done:**
- ✅ Torture corpus passes ≥95% (SPEC tier)
- ✅ No crashes on malformed PDFs
- ✅ Performance acceptable (<10 sec for typical doc — benchmark test added, run with `--ignored`)
- ✅ Error messages are actionable
- ✅ Ready for stress testing (Alpha → Beta) — **Phase 8 COMPLETE**

---

### Phase 9 — Drawing Sheet Management (Weeks 17-18)

**Goal:** Automated sheet replacement in drawing sets.

**Deliverables:**
- ☐ Sheet ID extraction (title blocks + footers)
- ☐ Sheet matching (by ID)
- ☐ PDF merge/replace logic
- ☐ Sheet renaming detection
- ☐ Bookmark generation
- ☐ Schedule extraction (basic tables)

**Definition of Done:**
- Sheet replacement works on real addenda
- Sheet IDs correctly extracted
- Sheet renaming tracked and reported
- Bookmarks generated accurately
- Audit trail shows which sheets replaced/renamed
- Ready for customer testing

---

### Phase 10 — Submittal Data Extraction (Weeks 19-20)

**Goal:** Extract structured data from equipment submittals.

**Deliverables:**
- ☐ Unit boundary detection
- ☐ Table extraction (performance specs)
- ☐ Key-value extraction (tags, models)
- ☐ CSV/JSON export (tidy format)
- ☐ Integration tests

**Definition of Done:**
- Submittal extraction works on 5 real submittals
- Data exported in tidy format
- Confidence scores accurate
- **Ready for customer validation (BETA COMPLETE)**

---

### Phase 11+ — GUI & Polish (Weeks 21+)

**Goal:** Ship monetizable product.

**Deliverables:**
- ☐ Desktop GUI on Tauri (canonical standard)
- ☐ New workflow-first UI architecture (fresh implementation, not incremental patching of prototype wizard stack)
- ☐ One-button workflows
- ☐ Overlay visualization
- ☐ Audit bundle review UI
- ☐ Pattern database management UI
- ☐ Billing + licensing

**GUI migration guardrails:**
- Freeze prototype GUI to bugfix-only while canonical GUI is built
- Use contract-first integration (`crates/contracts`) between Tauri UI and Rust backend
- Migrate workflows lane-by-lane with explicit parity checks
- Remove legacy screens once parity + soak testing pass

**GUI agent execution protocol (required):**
- Use a two-track model.
- **Track A (pre-Phase 11 prep):** contract shaping, UI state models, mock adapters, and integration scaffolding.
- **Track B (Phase 11+ runtime):** real backend integration lane-by-lane after dependency gates pass.
- Require dependency gates before enabling production lane execution.
- **Gate 0:** contract boundary and integration tests are stable.
- **Gate 1:** baseline extraction chain closed (G-004 -> G-005 -> G-001 -> G-002 -> G-003 -> G-009).
- **Gate 2:** review inputs available (warnings/failures consumable, audit hooks wired).
- **Gate 3:** export inputs stable (deterministic artifact selection, partial-success validated).
- Use canonical lane order.
- **Lane 1 (MVP):** Add files -> Start -> Review -> Export.
- **Lane 2:** Advanced audit/provenance detail behind explicit toggle.
- **Lane 3:** Higher-order workflows (compare/exception and additional lanes).
- Promote lanes only when integration tests pass, determinism checks pass, and silent-failure regressions are absent.

**Decision Point: Desktop vs Web**

**Start with desktop only:**
- Single binary, easy distribution
- No server infrastructure
- Works offline
- Simpler architecture

**Add web later (Phase 11+) if customers demand it:**
- Team collaboration
- Enterprise deployment
- Mobile access
- SaaS model

---

## Phase Definitions

### Phase 0-0.5 (Weeks 1-3): Definition of Done
**Pattern Dev Tool built and functional. ✅ ACHIEVED — April 5, 2026.**
- ✅ Can test patterns against PDFs visually
- ✅ Overlay system works
- ✅ Pattern validation suite runs (27 Tier 1 fixtures, 2,892 pages, det_regressions=0)
- ✅ Documentation includes pattern dev guide (`docs/PHASE_05_HANDOFF.md`)
- ✅ Contract surfaces defined for Band 1–4 downstream work (`crates/contracts/`)

### Phase 1 (Weeks 4-5): Definition of Done
**Clean JSON output + document AST.**
- ✅ Section segmentation works on 10 torture PDFs
- ✅ Coordinate system normalized correctly
- ✅ All invariants pass
- ✅ Debug visualization shows correct regions

**Success Metric:** Can extract and visualize layout from any spec PDF. ✅ COMPLETE

---

### Alpha (Weeks 0-14): Definition of Done
**Workflow happy paths work >50% of the time.**
- Can apply simple addendum (insert paragraph, renumber)
- End-to-end workflow executes without crashes
- Audit bundle is generated
- Output quality is "good enough" (Bondo doesn't show)
- Chrome metadata preserved and reapplied

**Success Metric:** Your team can use it for real work (with supervision).

---

### Beta (Weeks 15-20): Definition of Done
**Repeatable results. Torture corpus passes ≥95%. Polish.**
- Error handling is comprehensive
- Performance is acceptable (<10 sec typical)
- Audit trail is complete and actionable
- Documentation is ready
- Quality gates enforce standards

**Success Metric:** Ready to charge for it. First paying customer onboarded.

---

## Development Workflow & AI Coding Agent Strategy

### The Problem with AI Coding Agents

**Common failure mode:**
> "After a big update, the agent touches thousands of lines of code all over the repo and I get completely lost."

This happens because:
1. Tasks are too large (entire phases, not micro-tasks)
2. No incremental validation (can't tell what broke when)
3. No clear checkpoints (can't easily rollback)
4. Changes are opaque (don't understand what was generated)

### Solution: Micro-Task Development Strategy

**Core Principle:** Break every phase into tiny, testable, understandable increments (50-100 lines per task).

**Recommended Workflow with AI Coding Agent (Copilot/Cursor):**

```
1. Write a micro-task spec (1 paragraph, super specific)
2. Ask agent to write the test first (test-driven development)
3. Review the test (does it make sense? does it test the right thing?)
4. Ask agent to implement (just enough to pass the test)
5. Run the test (red → green)
6. Ask agent to explain the code (rubber duck review)
7. Add debug logging (so you can trace execution later)
8. Commit with clear message (document what was built)
9. Ask agent for next task suggestion
```

**Time per task:** 15-30 minutes (not 3 hours of mystery code)

### GUI Workstream Protocol (Phase 11+)

For GUI work, use the same micro-task discipline and enforce lane and gate order.

**Required GUI execution sequence:**
1. Start with **Track A** tasks only (contracts, state, and test scaffolding) until backend gates are satisfied.
2. Enable **Track B** runtime integration only for lanes with green gates.
3. Migrate one lane at a time; do not run broad multi-lane rewrites.
4. Keep the default UX simple-first in all lanes: Add files, Start processing, Review flagged items, Export.
5. Keep advanced diagnostics hidden by default behind an explicit toggle.

**Agent packet requirements for GUI tasks:**
1. Explicit entry gate (0-3).
2. Scope for the current lane only.
3. Acceptance criteria with deterministic tests.
4. Handoff note listing blockers and next dependency owner.

**Do not claim production readiness for any GUI lane unless:**
1. Gate conditions are met.
2. Integration and determinism tests pass twice consecutively.
3. Parity checks for that lane are documented.

---

### Micro-Task Example: Phase 1 (Layout Extraction)

**DON'T do this:**
```
You: "Build Phase 1: Layout Extraction"
Agent: [Generates 3000 lines across 15 files]
You: "WTF just happened?"
```

**DO this instead:**

```
Task 1.1: "Create LayoutTranscript data structure"
→ Agent generates: layout.rs (50 lines)
→ You review: "Does this struct make sense?"
→ Test: cargo build (does it compile?)
→ Commit: git commit -m "Add LayoutTranscript struct"

Task 1.2: "Add PDFium text extraction (just one page)"
→ Agent generates: extract.rs (80 lines)
→ You review: "Does it extract text from page 1?"
→ Test: cargo test test_extract_page_1
→ Commit: git commit -m "Add basic text extraction"

Task 1.3: "Add bbox extraction for spans"
→ Agent generates: extract.rs (40 lines modified)
→ You review: "Are bboxes reasonable?"
→ Test: cargo test test_bbox_extraction
→ Commit: git commit -m "Add bbox extraction"

Task 1.4: "Add coordinate normalization"
→ Agent generates: normalize.rs (60 lines)
→ You review: "Does Y flip correctly?"
→ Test: cargo test test_coordinate_normalization
→ Commit: git commit -m "Add coordinate normalization"
```

**Benefits:**
- Each task = 50-100 lines (manageable)
- You can actually review the diff
- Tests verify each piece works
- Git history shows progression
- Easy to rollback if something breaks

---

### Test-First Development (TDD)

**Write the test BEFORE the code.**

```
You: "Write a test that verifies we can extract text from page 1 of test.pdf"

Agent generates test:
```rust
#[test]
fn test_extract_text_page_1() {
    let pdf = load_pdf("tests/corpus/tier1/simple.pdf");
    let page1 = extract_page(&pdf, 0);
    assert!(page1.text.contains("RWB Project"));
    assert!(page1.spans.len() > 10);
}
```

You: "Now implement the function to make this test pass"

Agent generates implementation:
// Just enough code to pass the test
```

**Benefits:**
- Test defines exactly what you want
- You validate the test first (does it make sense?)
- Implementation is minimal (only what's needed)
- Red → Green → Refactor cycle

---

### Rubber Duck Reviews

**After agent generates code, ask it to explain:**

```
You: "Explain what this normalize_coordinates function does, line by line"

Agent:
"Line 5: We get the page height
Line 7: We flip the Y coordinate by subtracting from page height
Line 9: We clamp the result to [0, 1] range
..."

You: "Why do we clamp to [0, 1]?"

Agent:
"To normalize coordinates so 0=top, 1=bottom, regardless of page size"

You: "What happens if a span extends off the page?"

Agent:
"It would be clamped to 1.0, which could lose precision. We should add a warning for out-of-bounds spans."

You: "Add that warning"
```

**Benefits:**
- You understand what the code does
- Exposes edge cases agent missed
- Forces agent to justify decisions
- You learn by asking questions

---

### Debug Logging (Add Explicit Hooks Everywhere)

**This is CRITICAL for understanding what's happening:**

```rust
// In every function:
pub fn extract_footer(page: &Page) -> Option<Footer> {
    log::debug!("Extracting footer from page {}", page.index);
    
    let footer_region = detect_footer_region(page);
    log::debug!("Footer region: {:?}", footer_region);
    
    let text = extract_text_from_region(page, footer_region);
    log::debug!("Footer text: {}", text);
    
    let section_id = parse_section_id(&text);
    log::debug!("Parsed section ID: {:?}", section_id);
    
    section_id
}
```

**Then run with debug logging:**
```bash
RUST_LOG=debug cargo run -- extract test.pdf
```

**You'll see:**
```
DEBUG: Extracting footer from page 0
DEBUG: Footer region: BBox { y: 0.93, height: 0.05 }
DEBUG: Footer text: "2025-10-01 23 82 16 – Heating Water Coils"
DEBUG: Parsed section ID: Some("23 82 16")
```

**If something breaks, you can see exactly where.**

This is gold. Every function should have debug logging at key decision points. This makes debugging 10x faster and helps you understand the agent's code.

---

### Incremental Commits with Descriptions

**After each micro-task:**

```bash
git add .
git commit -m "
Add coordinate normalization

- Flips Y axis from PDF (bottom-left) to display (top-left)
- Normalizes coords to [0, 1] range
- Adds warning for out-of-bounds spans
- Tests verify header at top, footer at bottom
"
```

**Then:**
```
You to Agent: "Summarize what we just built and what's next"

Agent:
"We just added coordinate normalization. Next task: 
Add furniture detection to identify header/footer regions.
This will use the normalized coordinates we just added."
```

**Benefits:**
- Clear history of what was built
- You can review commit messages later
- Easy to see progression
- Natural checkpoints

---

### Staying in Control: Key Principles

1. **Micro-tasks only:** 50-100 lines per task, not 3000
2. **Test first:** Write test, then implement
3. **Review everything:** Never merge without understanding
4. **Debug hooks everywhere:** Log at every decision point
5. **Commit frequently:** After every micro-task (15-30 min)
6. **Rubber duck:** Ask agent to explain code line-by-line
7. **Validate incrementally:** Run tests after each task
8. **Ask "what's next":** Let agent suggest next micro-task

**If you follow this workflow, you'll never be lost in a sea of generated code again.**

---

## Torture Corpus Management

### Purpose

A **torture corpus** is a curated collection of real-world nightmare PDFs used to validate the system and prevent regression.

### Corpus Structure

**Tier System:**

**Tier 1 (Baseline):** 30-40 PDFs
- Representative samples from known "good" sources
- Common patterns, no edge cases
- 100% pass rate expected

**Tier 2 (Variations):** 50-60 PDFs
- Edge cases: scanned pages, missing footers, irregular formatting
- Different firms, templates, years
- ≥90% pass rate expected

**Tier 3 (Chaos):** 30-40 PDFs
- Truly broken PDFs: corrupted, hand-edited, malformed
- Acceptable failure rate: <50% pass
- Documents known failure modes

**Holdout Set (10%):** 12-15 PDFs
- Never used during development
- Only tested during final Phase 8 validation
- Prevents overfitting to corpus

### Curation Process

1. **Initial Collection:** Gather 120-150 PDFs from 20 years of project archives
2. **Categorization:** Sort into tiers
3. **Baseline Testing:** Run Phase 1 extraction on all tier1 PDFs
4. **Pattern Development:** Use tier1 failures to refine patterns
5. **Validation:** Re-test tier1 → should achieve 100% pass
6. **Expansion:** Move to tier2, refine patterns, achieve ≥90% pass
7. **Edge Case Handling:** Test tier3, document expected failures

### Anti-Overfitting Strategy

**Monthly Refresh:**
- Rotate in new real-world PDFs from current projects
- Rotate out oldest PDFs from corpus
- Keeps corpus representative of current AEC practices

**User-Submitted Failures:**
- Production failures automatically added to tier2 or tier3
- Investigated, fixed, regression tested
- Pattern DB updated if systemic issue found

### Success Metrics

**Phase 1-3:** Tier1 baseline = 100% pass  
**Phase 4-7:** Tier2 variations = ≥90% pass  
**Phase 8:** Holdout set = ≥85% pass  
**Production:** User-submitted failures <5% of total documents processed

---

## Glossary

| Term | Definition |
|------|-----------|
| **Layout Transcript** | Geometric representation of PDF (spans + coordinates, normalized) |
| **LayoutIR** | Intermediate representation (geometry + text, top-left origin, normalized coords) |
| **Pattern Database** | Versioned collection of regex patterns + bbox regions for extraction |
| **Pattern Dev Tool** | Developer tool for creating, testing, and validating patterns visually |
| **DocumentAST** | Semantic tree (SpecAST, DrawingAST, or SubmittalAST) |
| **EditableDocModel** | AST with normalized paragraphs, ready for reconstruction |
| **Chrome/Furniture** | Project reference matter (headers, footers, title blocks), not content |
| **Chrome Metadata** | Structured data extracted from chrome (project ID, dates, section info, branding) |
| **Provenance** | Source tracking (page, bbox, span IDs) for every element |
| **Deterministic Parsing** | Rule-driven extraction (no stochastic components at runtime) |
| **SheetInventory** | Canonical sheet list from drawing set |
| **AuditBundle** | Complete run record (transcript, overlays, metrics, decisions) |
| **Confidence** | 0.0–1.0 score indicating decision quality (numeric, measurable) |
| **Escalation** | Manual review required (FAIL status or explicit conflict) |
| **Verbatim Copy** | Unchanged page preserved byte-for-byte (or visually-identical minimum) |
| **Torture Corpus** | Collection of real-world nightmare PDFs used for validation |
| **Golden File** | Reference output used for regression testing (snapshot comparison) |
| **Micro-Task** | Development task scoped to 50-100 lines of code, completable in 15-30 minutes |

---

## Code Review Checklist

Before merging any code:
- [ ] All tests pass (unit + integration)
- [ ] No new clippy warnings
- [ ] Documentation updated (inline + external)
- [ ] Torture corpus still passes (≥target pass rate)
- [ ] Performance acceptable (no regressions)
- [ ] Error messages are actionable
- [ ] Debug logging added at key decision points
- [ ] Audit trail includes new decisions
- [ ] Micro-task commit message is clear and descriptive

---

## Release Process

**Version numbering:** Semantic versioning (MAJOR.MINOR.PATCH)
- MAJOR: Breaking API changes
- MINOR: New features (backward compatible)
- PATCH: Bug fixes

**Release checklist:**
- [ ] All tests pass on CI
- [ ] Torture corpus ≥95% pass rate
- [ ] Documentation updated
- [ ] Changelog written
- [ ] Binary built for all platforms
- [ ] Signed and checksummed
- [ ] Tagged in Git
- [ ] Published to GitHub releases

---

## Success Metrics

### Phase 0-0.5 Success
- Pattern Dev Tool functional
- Can test patterns visually
- Pattern validation suite works
- Documentation includes pattern dev guide

### Phase 1 Success
- ✅ Can extract layout from 100% of torture corpus
- ✅ Coordinate system normalized correctly (no inversions)
- ✅ All invariants pass
- ✅ Debug visualization shows correct regions

### Alpha Success (Phase 7)
- Your team uses it for real work (supervised)
- Happy path works >50% of the time
- Audit bundle helps debug failures
- Output quality acceptable ("Bondo doesn't show")
- Chrome metadata preserved and reapplied

### Beta Success (Phase 10)
- Torture corpus passes ≥95%
- First paying customer onboarded
- Error messages actionable
- Performance acceptable (<10 sec typical)
- Quality gates enforce standards

### V1.0 Success (Phase 11+)
- 10 paying customers
- <5% support ticket rate
- Feature parity with manual workflow
- Documentation complete
- Team collaboration features shipped

---

## Next Steps

1. **Approve Master Plan** (this document)
2. **Review Phase Definitions** (timeline, deliverables, dependencies)
3. **Confirm Technology Stack** (Rust, PDFium, headless Chrome)
4. **Build Supporting Documentation:**
   - DEV_STANDARDS (coding practices, testing requirements, debug logging rules)
   - AEC_STANDARDS (MasterFormat, UDS/NCS compliance)
   - ARCHITECTURE (detailed system design)
   - EXTERNAL_SURFACE (CLI/API interface definitions)
   - ROADMAP (micro-task breakdown per phase)
5. **Begin Phase 0** (project scaffolding)

---

## Known Workflow Gaps (Imported From Postmortem)

These are retained as active planning constraints until closed by implementation evidence.

1. Footer-driven section boundary mapping is not yet generalized across all divisions and layouts.
2. Complex schedule extraction edge cases (merged cells, rotated headers) need explicit deterministic handling strategy.
3. Narrative validation behavior must remain advisory-only and explicit about hard-fail versus soft-warn semantics.

### Chrome Detection Threshold Policy

Chrome/furniture and ROI scoring thresholds must be treated as policy-tunable parameters, not permanently hardcoded constants. Defaults may exist, but admin-only overrides and validation messaging are required.

### ROI Detection Strategy Policy

ROI detection follows an autonomous deterministic strategy by default:

1. Detection must execute from deterministic heuristics and ordered scoring rules, not manual profile prerequisites.
2. Candidate ROI generation, ranking, and tie-breaks must be deterministic and auditable.
3. Manual ROI/profile edits are admin-only and intended for refinement, diagnostics, and controlled fallback.
4. Low-confidence detection must emit explicit warnings and evidence, never silent coercion.
5. Any manual override path must preserve replayability by persisting effective ROI decisions in run artifacts.

### Deterministic Adaptive Detection Policy

Vector-PDF detection and extraction should follow a deterministic adaptive strategy:

1. Baseline detection must use deterministic geometry + text features first (frame/corner priors, table-structure score, typed pattern rules).
2. Auto-learned firm/layout templates are generated from successful detections and applied forward as accelerators, not authoritative black boxes.
3. Template application must remain auditable: include template ID, version, confidence delta, and divergence checks against fresh detection.
4. If deterministic path is low-confidence, system emits explicit review state and evidence bundle; no silent downgrade.
5. AI-enhanced fallback is optional, explicit opt-in, and bounded to cropped regions needed for recovery.

### Local Micro-ML Assist Policy

On-device micro-model assist should remain bounded and auditable:

1. Micro-ML runs only after deterministic baseline scoring and only when confidence is in configured gray-zone ranges.
2. Model inference must be version-locked and reproducible; confidence fusion must be explicit, deterministic, and policy-configured.
3. Decision artifacts must include baseline score, model score, fused score, threshold basis, and final branch reason.

### Power-User LLM Validation Policy

LLM-backed power-user workflows are optional and contract-bound:

1. LLM validation/instruction generation requires explicit user invocation and cannot run silently.
2. LLM outputs must be treated as advisory unless user promotes them to executable instruction manifests.
3. Prompt/response traceability must be preserved in audit artifacts with sensitive content redaction policy.

### Raster OCR Policy

Raster or low-text-layer pages must follow explicit OCR routing:

1. Stage 0 detects low text-layer quality and marks pages for OCR path.
2. OCR text extraction must capture per-line/per-block confidence and source tag (`ocr` vs `vector`).
3. OCR-derived decisions with low confidence require explicit review warnings and overlays.

### Structured Schedule Data Policy

Schedule extraction outputs must be contract-first:

1. Canonical internal schedule schema must be versioned before broad export.
2. Exports must include JSON/CSV/XML support with stable field mappings.
3. Every exported record requires page/region provenance and parser confidence metadata.

### Replayable Corrections and Automation Policy

1. Corrections must compile into typed manifests with explicit scope, target IDs, and replay guards.
2. Manual review actions, power-user instructions, and approved AI suggestions must converge on the same executable manifest surface.
3. Manifest application must be deterministic, dry-runnable, and auditable before any file mutation occurs.

### Operational Trust and Review Policy

1. Native diff/change reports are required outputs for revision workflows where applicable.
2. Exception queues must focus operator attention on ambiguous, conflicting, drifted, or low-confidence cases.
3. Drift metrics, fallback rates, and review burden must be measurable against the corpus over time.

### Standards Normalization Policy

1. Normalization work must reuse canonical AEC standards references already maintained in `AEC_STANDARDS.md`, `MASTERFORMAT_REFERENCE.md`, and related standards docs.
2. Normalized IDs, aliases, and mapped classifications must preserve both canonical form and source-observed raw value.
3. Changes to normalization mappings must be versioned and backward-auditable.

### Cross-Document Knowledge Policy

1. Entity resolution links must retain evidence basis and confidence.
2. Search/index surfaces must operate on canonical normalized records plus provenance, not ad hoc text caches.
3. Privacy policy controls must propagate to any persisted searchable index.

---

## Governance & Revision

This document is **constitutional**. Changes require explicit approval.

**Revision Process:**
1. Identify conflict with non-negotiable
2. Document trade-off (what's gained, what's lost)
3. Explicit approval (stakeholder consensus)
4. Update document + revision history + rationale

**Current Status:** ✅ READY FOR IMPLEMENTATION

**Owner:** HLLMR LLC  
**Last Updated:** March 23, 2026  
**Version:** 4.2.6 (Operational Trust, Automation, and Knowledge Layer Policy Update)

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 4.0.0 | 2026-01-21 | Initial Master Plan |
| 4.1.0 | 2026-01-22 | Production-ready architecture. Removed Python. Clarified PDFium. Simplified profile system. Defined phase definitions. Locked tech stack. |
| **4.2.0** | **2026-01-23** | **Senior Architect Review - Phase Reorganization.** Key changes: (1) **Moved Pattern Dev Tool from Phase 12 to Phase 0.5** - recognized as critical development infrastructure, not polish. (2) **Added Chrome Metadata Preservation** - explicit extraction, storage, and reuse of headers/footers/branding for professional output. (3) **Added Development Workflow section** - micro-tasking strategy for AI coding agents, test-driven development, rubber duck reviews, debug logging standards. (4) **Added Non-Negotiable #6** - chrome metadata must be preserved. (5) **Added Non-Negotiable #20** - Pattern Dev Tool is infrastructure. (6) **Clarified Desktop-First GUI strategy** - defer web to Phase 11+ unless customer demand. (7) **Updated Glossary** - added Pattern Dev Tool, Chrome Metadata, Micro-Task. (8) **Updated Code Review Checklist** - added debug logging and micro-task commit requirements. |
| **4.2.1** | **2026-03-19** | **Architecture Execution Update - Monorepo + Tauri GUI Direction.** Key changes: (1) Added monorepo repository strategy with hard package boundaries. (2) Standardized desktop direction on Tauri. (3) Added contract-first backend/frontend integration guardrails. (4) Restored documentation authority block and imported workflow-gap constraints from the prior constitutional plan during merge into `MASTER_PLAN.md`. |
| **4.2.2** | **2026-03-23** | **Autonomous ROI Detection Policy Update.** Key changes: (1) Promoted autonomous deterministic ROI detection to first-class strategy, (2) retained manual ROI/profile management as admin-only refinement/fallback, (3) replaced profile-tunable wording with policy/admin-override wording for thresholds, (4) added explicit ROI strategy policy block in Known Workflow Gaps constraints. |
| **4.2.3** | **2026-03-23** | **Multi-file Ingestion & Normalization Policy Update.** Key changes: (1) Added Intake Triage as mandatory pre-Lexer Stage 0 with `NormalizedIntakeBundle` output, (2) revised Non-Negotiable #18 to permit autonomous classification on explicit user-invoked classify/split/normalize operations, (3) added Non-Negotiable #22 (Intake Triage) and #23 (`lopdf` as canonical PDF write backend), (4) added `lopdf` to tech stack table with explicit role delineation (PDFium = read-only extraction, lopdf = page write ops, headless Chrome = content generation). |
| **4.2.4** | **2026-03-23** | **Deterministic Adaptive Detection & Template Policy Update.** Key changes: (1) Added vector-first deterministic adaptive detection policy and auto-learned firm-template policy, (2) added optional AI fallback guardrails (explicit opt-in, cropped region scope), (3) extended drawings/spec processing sections with deterministic title-block and heading extraction steps, (4) aligned governance language with explainable template-assisted workflows. |
| **4.2.5** | **2026-03-23** | **Assisted Intelligence, OCR, and Schedule Data Policy Update.** Key changes: (1) Added local micro-ML assist policy (on-device, deterministic-bounded, version-locked), (2) added power-user LLM validation/instruction API policy (explicit opt-in, advisory-by-default, auditable), (3) added first-class raster OCR policy with confidence/provenance requirements, (4) upgraded schedule extraction exports to schema-versioned JSON/CSV/XML contracts. |
| **4.2.6** | **2026-03-23** | **Operational Trust, Automation, and Knowledge Layer Policy Update.** Key changes: (1) Added replayable correction manifests, provenance-first review, privacy/redaction, batch orchestration, and instruction DSL policies, (2) promoted native diff/exception triage workflows, (3) anchored standards normalization to existing canonical UDS/NCS/MasterFormat scaffold, (4) added cross-document entity resolution and project knowledge indexing as strategic capabilities. |
| **4.2.7** | **2026-04-05** | **Phase 0.5 Completion.** Marked Phase 0.5 deliverables complete (✅). Corrected command examples to match locked implementation (`--family`, `--output-dir`). Updated Phase 0-0.5 Definition of Done with actual results (27 fixtures, 2892 pages validated, det_regressions=0, `crates/contracts/` complete). See `docs/PHASE_05_HANDOFF.md` for full downstream handoff and Appendix A for implementation history. |
| **4.3.0** | **2026-04-05** | **Phase 1–3 Complete + Parser Hardening.** Marked Phases 1, 2, and 3 deliverables complete (✅). Updated Phase 3 Definition of Done with parser hardening results (89 sections, 7,971 nodes, 0.2% unclassified, 0/70 wrong-PART). Updated dependency graph: Phase 4 ← NEXT. 19/19 integration tests passing. |
| **4.5.0** | **2026-04-06** | **Phase 5 Complete: Section Regeneration.** Marked all Phase 5 deliverables complete (✅). New files: `crates/ir/src/render.rs` (SpecChromeMetadata, RenderConfig, PageSize, RenderResult, RenderError), `crates/engine/src/render/` (body.rs, chrome.rs, chrome_pdf.rs, mod.rs). `WorkflowOperation::Regenerate` contract variant added. `regenerate` CLI subcommand implemented (8 args). `SectionRenderer::dry_run()` path for CI/dry-run. CSS Paged Media `@page` margin-box rules for running headers/footers. Chrome/Brave binary discovery added (Brave confirmed working — full round-trip test passes in ~42 s on real SPEC PDF). G-008 accepted-closed; G-010 closed (4 behavioral audit tests). All 31 integration tests pass (30 non-Chrome + 1 full Brave round-trip). Updated dependency graph: Phase 5 ← COMPLETE, Phase 6 ← NEXT. |

---

**End of Master Plan**
