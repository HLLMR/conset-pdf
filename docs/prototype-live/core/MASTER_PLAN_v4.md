# Conset PDF: Master Plan V4.2
**Version:** 4.2.1 (Architecture Execution Update - Monorepo + Tauri GUI Direction)  
**Date:** March 19, 2026  
**Owner:** HLLMR LLC  
**Status:** ✅ Ready for Implementation  

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

1. **Determinism is sacred.** No runtime randomness. Same input + same profile + same engine version = **identical output**.

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

18. **Medium detection is user-driven.** GUI enforces context through workflow-based file pickers. CLI requires explicit operation flags. No auto-detection—explicit is better than implicit.

19. **Accuracy over visual fidelity.** For spec regeneration: textual accuracy 100% required, visual fidelity best-effort. Readable and correct beats pixel-perfect.

20. **Pattern Development Tool is infrastructure, not polish.** The Pattern Dev Tool must be built early (Phase 0.5) as it's a critical development dependency for all pattern-based work (Phases 2-4).

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
| **Pattern Matching** | `regex` crate | Deterministic, fast, well-tested |
| **Serialization** | `serde` + `serde_json` | Standard Rust serialization |
| **GUI (V4 desktop)** | Tauri | Rust-native desktop shell with strong backend integration |
| **Testing** | Built-in `cargo test` + golden files | Regression testing |

### Repository Strategy (Monorepo with Hard Boundaries)

Use a **single monorepo** for V4 development with strict package/crate boundaries.

**Rationale:**
- Backend and GUI contracts will evolve rapidly during V4
- Atomic cross-boundary refactors are required for reliability
- One CI surface avoids integration drift between repos

**Required structure:**

```
repo-root/
├── apps/
│   ├── backend-cli/         # Rust binary entrypoints (CLI/API host)
│   └── desktop-gui/         # Tauri app + frontend UI
├── crates/
│   ├── core-engine/         # Deterministic extraction and transforms
│   ├── workflows/           # Merge/split/bookmarks/spec patch orchestration
│   ├── contracts/           # Shared request/response/event schemas
│   └── standards-data/      # Embedded standards datasets
├── docs/
│   ├── prototype-postmortem/
│   └── v4/
└── tests/
  ├── corpus/
  └── integration/
```

**Boundary rules:**
- GUI must depend on backend only through `crates/contracts`
- No GUI imports from `core-engine` internals
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

**D4: Schedule Extraction (Optional)**
- Extract tabular data from equipment schedules
- Handle multi-table pages
- Handle rotated text, merged cells
- Export to CSV/JSON

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
Phase 0: Scaffolding (Week 1)
    ↓
Phase 0.5: Pattern Dev Tool (Weeks 2-3) ← CRITICAL EARLY BUILD
    ↓
Phase 1: Layout Extraction (Weeks 4-5)
    ↓
Phase 2: Furniture/Sections (Weeks 6-7) ← Uses Pattern Dev Tool
    ↓
Phase 3: Paragraph Parsing (Weeks 8-9)
    ↓
Phase 4: Edit Operations (Week 10)
    ↓
Phase 5: Regeneration (Weeks 11-12)
    ↓
Phase 6: PDF Stitching (Week 13)
    ↓
Phase 7: End-to-End (Week 14) ← ALPHA COMPLETE
    ↓
Phase 8: Production Hardening (Weeks 15-16)
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
- ☐ CLI tool for pattern development
- ☐ Visual overlay system (show matched regions on PDF pages)
- ☐ Pattern testing framework (regex → PDF → visual confirmation)
- ☐ Pattern validation suite (test patterns against sample PDFs)
- ☐ Debug output (show confidence scores, matched text, bboxes)

**Core Features:**

```bash
# Test a footer pattern against a PDF
cargo run --bin pattern-dev -- test-pattern \
  --pdf test.pdf \
  --pattern-type footer_section_id \
  --regex "^(\d{2}\s\d{2}\s\d{2})\s+–" \
  --output-overlays debug/

# Output:
# ✓ Page 0: Matched "23 82 16 – " (confidence: 0.98)
# ✓ Page 1: Matched "23 82 16 – " (confidence: 0.98)
# ✗ Page 2: No match (confidence: 0.0)
#
# Overlay images saved to debug/page-*.png
```

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
- ☐ LayoutTranscript types defined
- ☐ PDFium text extraction with bbox coordinates
- ☐ Coordinate normalization (PDF bottom-left → display top-left)
- ☐ Invariant validation (no negative sizes, out-of-bounds, etc.)
- ☐ Debug visualization (draw bboxes on page images)
- ☐ Integration tests on 5 sample PDFs

**Test:**
```bash
cargo run -- extract test.pdf -o transcript.json
cargo run -- visualize transcript.json -o debug/
```

**Output:** `debug/page-000.png` with bboxes drawn, coordinate system verified.

**Definition of Done:** 
- Transcript JSON output is clean and normalized
- All 5 test PDFs extract successfully
- No coordinate inversions (headers at top, footers at bottom)
- All invariants pass

---

### Phase 2 — Furniture Detection & Section Segmentation (Weeks 6-7)

**Goal:** Build the index. Map section IDs to page ranges.

**Deliverables:**
- ☐ FurnitureDetector implementation (uses patterns from Phase 0.5)
- ☐ Chrome metadata extraction (project ID, dates, section info)
- ☐ Pattern database integration
- ☐ Section segmentation algorithm (footer-first oracle)
- ☐ Page-in-section counter detection and validation
- ☐ Coverage validation
- ☐ Debug overlays (furniture regions marked, metadata displayed)

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
- Index JSON shows all sections correctly segmented
- Chrome metadata extracted and stored
- Coverage ≥95% on torture corpus
- Footer patterns match expected format
- Page-in-section counters validate boundaries
- No section boundary conflicts

---

### Phase 3 — Paragraph Parsing & AST Construction (Weeks 8-9)

**Goal:** Parse sections into hierarchical AST.

**Deliverables:**
- ☐ Line grouping (baseline clustering)
- ☐ Paragraph detection
- ☐ Outline marker parsing (A., 1., a., i.)
- ☐ Nesting inference (indent-based)
- ☐ AST construction (Section → Part → Article → Paragraph)
- ☐ Debug output (AST visualization)

**Test:**
```bash
cargo run -- parse test.pdf --section "23 82 16" -o ast.json
cargo run -- visualize-ast ast.json -o debug/ast.html
```

**Output:** Full hierarchical AST of Section 23 82 16 with correct nesting.

**Definition of Done:**
- AST accurately represents section structure
- Outline numbering parsed correctly (2.7.A, 2.7.B, etc.)
- Nesting levels inferred correctly
- Part/Article boundaries detected
- Works on 80% of torture corpus sections

---

### Phase 4 — Edit Operations (Week 10)

**Goal:** Apply surgical edits to AST.

**Deliverables:**
- ☐ SectionEditor implementation
- ☐ Insert operation (insert_after with renumbering)
- ☐ Delete operation
- ☐ Replace operation
- ☐ Paragraph renumbering logic
- ☐ Validation (target exists, no conflicts)

**Test:**
```bash
cargo run -- edit test.pdf \
  --section "23 82 16" \
  --operation insert_after \
  --target "2.7.B" \
  --content "C. Provide return air damper." \
  --renumber \
  -o edited-ast.json
```

**Output:** Modified AST with new paragraph 2.7.C, subsequent paragraphs renumbered.

**Definition of Done:**
- Insert/delete/replace operations work correctly
- Renumbering cascades properly (C→D, D→E)
- Validation catches invalid targets
- Edited AST passes structural validation

---

### Phase 5 — Section Regeneration (HTML → PDF) (Weeks 11-12)

**Goal:** Render AST to PDF via HTML/CSS, with chrome reapplication.

**Deliverables:**
- ☐ SectionRenderer implementation
- ☐ HTML template with CSS formatting
- ☐ Headless Chrome integration (`headless_chrome` crate)
- ☐ Chrome template system (headers/footers with metadata)
- ☐ Formatting rules (fonts, spacing, indentation, page breaks)
- ☐ Visual comparison tests

**Test:**
```bash
cargo run -- regenerate edited-ast.json \
  --chrome-metadata chrome.json \
  -o section-new.pdf
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
- Headers showing project info
- Footers showing section ID, updated date, recalculated page numbers
- Content formatted consistently with original

**Definition of Done:**
- Regenerated sections look "good enough" (Bondo doesn't show)
- Chrome (headers/footers) applied correctly with updated metadata
- Fonts/spacing approximately match original
- Page breaks handled correctly
- Output passes visual inspection

---

### Phase 6 — PDF Stitching & Writeback (Week 13)

**Goal:** Replace section pages in original PDF.

**Deliverables:**
- ☐ PdfStitcher implementation
- ☐ Page replacement logic (delete old, insert new)
- ☐ Bookmark preservation
- ☐ Verbatim copy of unchanged pages
- ☐ Validation (unchanged pages identical)

**Test:**
```bash
cargo run -- stitch \
  --original test.pdf \
  --section "23 82 16" \
  --replacement section-new.pdf \
  -o output.pdf
```

**Output:** Final PDF with section replaced, all other pages unchanged.

**Definition of Done:**
- Section pages replaced correctly
- Unchanged pages are verbatim copies (byte-identical when possible)
- Bookmarks updated to reflect new structure
- Output passes validation

---

### Phase 7 — End-to-End Workflow (Week 14)

**Goal:** Single command applies addendum edits.

**Deliverables:**
- ☐ Unified CLI command
- ☐ JSON addendum format
- ☐ End-to-end processing pipeline
- ☐ Comprehensive error handling
- ☐ Audit bundle generation

**Test:**
```bash
cargo run -- apply-addendum \
  --original specs-rev0.pdf \
  --addendum addendum-3.json \
  -o specs-rev1.pdf \
  --audit-bundle audit/
```

**Output:** 
- Updated PDF with edits applied
- Audit bundle with overlays, logs, metrics
- Change report (which sections modified, which pages replaced)

**Definition of Done:**
- End-to-end workflow works on structured addendum
- All edits applied correctly
- Section boundaries preserved
- Unchanged sections verbatim copied
- Audit bundle is complete and readable
- **Ready for internal testing (ALPHA COMPLETE)**

---

### Phase 8 — Production Hardening (Weeks 15-16)

**Goal:** Production-ready engine.

**Deliverables:**
- ☐ Comprehensive error handling
- ☐ Memory safety (crash containment, caps)
- ☐ Performance optimization (large PDFs)
- ☐ Torture corpus validation (≥95% pass rate)
- ☐ Metrics dashboard (confidence, coverage, failures)
- ☐ User documentation

**Definition of Done:**
- Torture corpus passes ≥95%
- No crashes on malformed PDFs
- Performance acceptable (<10 sec for typical doc)
- Error messages are actionable
- Ready for stress testing (Alpha → Beta)

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
- ☐ Desktop GUI on Tauri (V4 standard)
- ☐ New workflow-first UI architecture (fresh implementation, not incremental patching of prototype wizard stack)
- ☐ One-button workflows
- ☐ Overlay visualization
- ☐ Audit bundle review UI
- ☐ Pattern database management UI
- ☐ Billing + licensing

**GUI migration guardrails:**
- Freeze prototype GUI to bugfix-only while V4 GUI is built
- Use contract-first integration (`crates/contracts`) between Tauri UI and Rust backend
- Migrate workflows lane-by-lane with explicit parity checks
- Remove legacy screens once parity + soak testing pass

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
**Pattern Dev Tool built and functional.**
- Can test patterns against PDFs visually
- Overlay system works
- Pattern validation suite runs
- Documentation includes pattern dev guide

### Phase 1 (Weeks 4-5): Definition of Done
**Clean JSON output + document AST.**
- Section segmentation works on 10 torture PDFs
- Coordinate system normalized correctly
- All invariants pass
- Debug visualization shows correct regions

**Success Metric:** Can extract and visualize layout from any spec PDF.

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
    let pdf = load_pdf("tests/fixtures/test.pdf");
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
- Can extract layout from 100% of torture corpus
- Coordinate system normalized correctly (no inversions)
- All invariants pass
- Debug visualization shows correct regions

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

1. **Approve Master Plan V4.2** (this document)
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

## Governance & Revision

This document is **constitutional**. Changes require explicit approval.

**Revision Process:**
1. Identify conflict with non-negotiable
2. Document trade-off (what's gained, what's lost)
3. Explicit approval (stakeholder consensus)
4. Update document + revision history + rationale

**Current Status:** ✅ READY FOR IMPLEMENTATION

**Owner:** HLLMR LLC  
**Last Updated:** January 23, 2026  
**Version:** 4.2.0 (Senior Architect Review - Phase Reorganization)

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 4.0.0 | 2026-01-21 | Initial Master Plan |
| 4.1.0 | 2026-01-22 | Production-ready architecture. Removed Python. Clarified PDFium. Simplified profile system. Defined phase definitions. Locked tech stack. |
| **4.2.0** | **2026-01-23** | **Senior Architect Review - Phase Reorganization.** Key changes: (1) **Moved Pattern Dev Tool from Phase 12 to Phase 0.5** - recognized as critical development infrastructure, not polish. (2) **Added Chrome Metadata Preservation** - explicit extraction, storage, and reuse of headers/footers/branding for professional output. (3) **Added Development Workflow section** - micro-tasking strategy for AI coding agents, test-driven development, rubber duck reviews, debug logging standards. (4) **Added Non-Negotiable #6** - chrome metadata must be preserved. (5) **Added Non-Negotiable #20** - Pattern Dev Tool is infrastructure. (6) **Clarified Desktop-First GUI strategy** - defer web to Phase 11+ unless customer demand. (7) **Updated Glossary** - added Pattern Dev Tool, Chrome Metadata, Micro-Task. (8) **Updated Code Review Checklist** - added debug logging and micro-task commit requirements. |

---

**End of Master Plan V4.2**
