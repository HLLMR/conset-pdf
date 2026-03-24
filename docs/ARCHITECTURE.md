# Conset PDF: Architecture

**Version:** 4.2.0  
**Date:** January 23, 2026  
**Owner:** HLLMR LLC  
**Status:** ✅ ACTIVE  
**Doc Status Tag:** Implemented
**Alignment:** MASTER_PLAN + DEV_STANDARDS + AEC_STANDARDS

---

## Overview

This document describes the **system architecture** of Conset PDF, a deterministic-first, compiler-model system for extracting, parsing, and reconstructing structured content from AEC PDFs.

This is a canonical derived document under `MASTER_PLAN.md` per `DOC_GOVERNANCE.md`.

**Scope:**
- Architectural principles (SSOT, determinism, medium-specificity)
- System components and responsibilities
- Data flow through the compiler pipeline
- Type system and core interfaces
- Standards integration (UDS, MasterFormat)
- Quality enforcement mechanisms

**Audience:** Architects and senior developers designing and implementing the system

**Philosophy:** Architecture serves standards, not the reverse. Every component enforces determinism, correctness, and auditability.

---

## Table of Contents

1. [Architectural Principles](#architectural-principles)
2. [System Components](#system-components)
3. [Compiler Pipeline (Stages)](#compiler-pipeline-stages)
4. [Data Flow](#data-flow)
5. [Type System](#type-system)
6. [Standards Integration](#standards-integration)
7. [Quality Enforcement](#quality-enforcement)
8. [Extension Points](#extension-points)

---

## Architectural Principles

### Principle 1: Single Source of Truth (SSOT)

**Statement:** Every resource has exactly one canonical location. Never duplicate or cache stale state.

**Why:** Stale caches cause silent failures. Canonical sources enable auditability.

**Application:**
- **PDFs:** Load once via `DocumentContext`, never via direct file access
- **Transcripts:** Cache in `DocumentContext`, invalidate on reload
- **Patterns:** Single pattern database, versioned and immutable
- **Audit trails:** Append-only JSONL, never rewrite

**Implication:** If state is needed in two places, it comes from SSOT, never duplicated.

---

### Principle 2: Determinism First

**Statement:** Same input + same profile + same engine version = identical output, always.

**Why:** AEC users need reproducible results for audit trails. Determinism is the moat.

**Application:**
- No runtime randomness (no shuffles, random selection)
- Deterministic sorting (explicit comparators, not alphabetic)
- Deterministic tie-breaking (lower order value wins)
- Fixed-point geometry (use integers where precision matters)
- No timing-dependent behavior

**Quality Gate:** Nightly regression: same input → bit-identical output.

---

### Principle 3: Medium-Specificity

**Statement:** Specs, drawings, and submittals are processed by **separate, medium-specific components**.

**Why:** Over-generalization breaks extraction. A spec parser ≠ drawing parser.

**Application:**
- Separate processors per medium (SpecProcessor, DrawingProcessor, SubmittalProcessor)
- Shared foundation only (LayoutIR, audit framework, geometry utilities)
- Medium-specific ASTs (SpecAST, DrawingAST, SubmittalAST)
- Profile system validates medium compatibility

**Implication:** No shared processing pipeline between mediums.

---

### Principle 4: Compiler Model

**Statement:** Process PDFs like a compiler: Lexer → Parser → Optimizer → Code Generator.

**Why:** Clear separation of concerns, testable stages, explainable transformations.

**Stages:**
1. **Lexer (Layout Extraction):** Raw PDF → LayoutTranscript (geometry + text)
2. **Parser (Semantic Analysis):** LayoutTranscript → DocumentAST (hierarchical structure)
3. **Optimizer (Editing):** DocumentAST → EditableDocModel (with applied edits)
4. **Code Generator (Rendering):** EditableDocModel → OutputPDF (regenerated content)

**Implication:** Each stage has clear inputs/outputs, unit testable.

---

### Principle 5: Audit Trail as First-Class Output

**Statement:** Every operation produces **structured audit trail** (JSONL events) alongside results.

**Why:** Users must understand *why* extraction succeeded or failed. Audit trails are the moat.

**Application:**
- Append-only JSONL event log
- Every decision logged (timestamp, stage, confidence, basis)
- Visual overlays (annotated page images)
- Conflict tracking (ambiguous cases with all candidates)

**Implication:** Audit bundle is mandatory output, not optional.

---

### Principle 6: Chrome Metadata Preservation

**Statement:** Chrome (headers/footers/title blocks) contains critical metadata that must be preserved and reused.

**Why:** Regenerated content must look like official reissues, not Word exports.

**Application:**
- Extract chrome metadata during furniture detection
- Store project info, firm branding, section identifiers
- Reapply chrome to regenerated content with updated dates/page numbers
- Preserve visual continuity with original documents

**Implication:** Furniture detection is not just exclusion—it's metadata extraction.

---

## System Components

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        apps/backend-cli                      │
│  - Command routing                                          │
│  - contracts <-> engine translation boundary                │
│  - Audit session lifecycle                                  │
└─────────────────────────────────────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────────────────┐
│                      crates/engine                          │
│  - Pipeline stage orchestration (extraction/furniture/...)  │
│  - Extractor / Processor public API                         │
└─────────────────────────────────────────────────────────────┘
                  │
                  ↓
┌─────────────────────────────────────────────────────────────┐
│                     Shared Foundation                        │
│  ┌────────────────┬──────────────┬─────────────────────┐   │
│  │ crates/ir      │ crates/audit │ crates/standards-   │   │
│  │ (geometry)     │ (events)     │ data (datasets)     │   │
│  └────────────────┴──────────────┴─────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Current Implementation Snapshot (March 2026)

- `apps/backend-cli` is implemented as the primary executable surface and writes typed audit bundles.
- `apps/desktop-gui` exists with command stubs and stable contracts-shaped handlers.
- `crates/engine` exposes pipeline stages, but stages remain mostly pass-through/scaffold logic.
- `crates/pdf-extraction` has working document/page loading and text extraction primitives, but page-structured extraction remains incomplete.
- `crates/ir` types and tests are present; full validator enforcement remains incomplete.
- `crates/audit` models and persistence are implemented; pipeline hook integration remains open.

Detailed evidence and open gaps are tracked in `docs/current-state/capability-matrix.md` and `docs/current-state/gap-register.md`.

---

### Crate Structure

```
conset-pdf/
├── Cargo.toml
├── apps/
│   ├── backend-cli/          # CLI/API host and contracts boundary
│   └── desktop-gui/          # Desktop GUI backend stubs
├── crates/
│   ├── engine/               # Pipeline orchestration and runtime API
│   │   └── src/
│   │       ├── main.rs       # Engine binary stub
│   │       ├── lib.rs        # Public API
│   │       ├── extractor.rs  # Extractor API
│   │       ├── processor.rs  # Processor API
│   │       └── pipeline/     # extraction/furniture/parsing/optimization
│   │
│   ├── ir/                   # Layout IR (shared)
│   │   └── src/
│   │       ├── types.rs
│   │       ├── transcript.rs
│   │       ├── geometry.rs
│   │       └── validation.rs
│   │
│   ├── audit/                # Audit framework (shared)
│   │   └── src/
│   │       ├── events.rs
│   │       ├── bundle.rs
│   │       ├── writer.rs
│   │       └── lib.rs
│   │
│   ├── pdf-extraction/       # PDF library wrapper (shared)
│   │   └── src/
│   │       ├── extractor.rs
│   │       ├── traits.rs
│   │       └── error.rs
│   │
│   ├── contracts/            # Shared request/response/event schemas
│   ├── workflows/            # Workflow contracts/orchestration scaffolds
│   └── standards-data/       # Standards datasets and lookup support
│   │
├── tools/
│   ├── Cargo.toml
│   └── classify_pdf.rs       # Utility binary for PDF corpus classification
│
└── tests/
  ├── integration/          # Cross-crate integration tests
  └── corpus/               # Tiered PDF corpus fixtures
```

---

### Component Responsibilities

#### apps/backend-cli

**Responsibility:** CLI interface, workflow request construction, engine dispatch, and audit bundle lifecycle.

**Current state:** Implemented and functional for request/response and audit session wiring; operation handlers are still scaffold-level for several workflows.

---

#### apps/desktop-gui

**Responsibility:** Desktop GUI backend command surface aligned to `contracts` types.

**Current state:** Stub command handlers with stable signatures; frontend/runtime wiring is deferred.

---

#### engine

**Responsibility:** Extraction/processing API and pipeline stage orchestration.

**Key modules:**
- `extractor.rs` — Extractor stage entrypoint (`extract`)
- `processor.rs` — Processor stage entrypoint (`process`)
- `pipeline/*` — extraction, furniture_detection, parsing, optimization stages

**Current state:** Pipeline boundaries are present; stage internals are largely scaffold/pass-through pending gap closure.

---

#### ir (Layout IR)

**Responsibility:** Normalized geometric representation of PDF.

**Key types:**
- `LayoutTranscript` — Complete document layout
- `Page` — Per-page geometry + spans
- `Span` — Text span with bbox, font, flags
- `BBox` — Normalized bounding box [0.0, 1.0]

**Invariants:**
- Coordinates normalized (top-left origin, 0.0–1.0 range)
- No impossible bboxes (negative sizes, out-of-bounds)
- All text UTF-8 valid

---

#### audit (Audit Framework)

**Responsibility:** Structured event logging, visual overlays, audit bundle generation.

**Key types:**
- `AuditEvent` — JSONL event with timestamp, stage, confidence, basis
- `AuditTrail` — Append-only event log
- `AuditBundle` — Complete artifact (events + overlays + metrics)

**Output format:** JSONL (one event per line, machine-parseable)

---

#### pdf-extraction

**Responsibility:** PDF library wrapper (PDFium), deterministic extraction.

**Key functions (current):**
- `load_document(path)`
- `get_page_count(document)`
- `extract_text(document, page_index)`
- `extract_page(...)` (currently scaffold-level)

**Critical:** Crash containment, memory caps, safe failure modes.

---

#### contracts

**Responsibility:** Shared schema authority for workflow requests/responses and typed audit payloads.

---

#### workflows

**Responsibility:** Workflow trait/context/result contracts and operation-routing surface.

---

#### standards-data

**Responsibility:** AEC-specific classification logic (UDS, MasterFormat).

**Current state:** Dataset crate is present; incremental implementation remains in progress.

**Source of Truth:** Implements AEC_STANDARDS

---

#### tools

**Responsibility:** Utility binaries for repo-level operations.

**Current state:** `classify_pdf.rs` utility is present.

---

## Compiler Pipeline (Stages)

The stage contracts below define the target behavior. Current implementation status for each stage is tracked in `docs/current-state/capability-matrix.md` and `docs/current-state/gap-register.md`.

### Stage 1: Layout Extraction (Lexer)

**Input:** PDF file (raw bytes)

**Process:**
1. Load PDF with PDFium
2. Extract text + geometry per page (spans with bboxes)
3. Normalize coordinates (bottom-left → top-left, page-relative → normalized [0, 1])
4. Validate invariants (no impossible bboxes, UTF-8 valid)

**Output:** `LayoutTranscript` (geometry + text, no semantics)

**Invariants:**
- Rotation/crop normalized
- Coordinates in [0.0, 1.0] range
- Spans sorted deterministically (page, y, x, span_order)

---

### Stage 2: Furniture Detection (Chrome Analysis)

**Input:** `LayoutTranscript`

**Process:**
1. Detect header/footer regions (top 15%, bottom 15%)
2. Extract text from chrome regions
3. Match against known patterns (footer formats, header formats)
4. Extract metadata (project ID, section ID, dates, firm info)
5. Compute confidence scores

**Output:** 
- `FurnitureRegions` (excluded from content parsing)
- `ChromeMetadata` (preserved for reuse)

**Medium-specific:** Different patterns for specs vs drawings vs submittals.

---

### Stage 3: Semantic Parsing (Parser)

**Input:** `LayoutTranscript` + `FurnitureRegions`

**Process (Specs):**
1. Group spans into lines (baseline clustering)
2. Group lines into paragraphs (gap detection)
3. Parse outline markers (A., 1., 1.1, etc.)
4. Build hierarchical AST (Section → Part → Article → Paragraph)
5. Classify sections (MasterFormat)

**Process (Drawings):**
1. Detect sheets (title block extraction)
2. Extract sheet IDs, titles
3. Classify disciplines (UDS)
4. Build inventory (sheet list with metadata)
5. Extract schedules (table detection)

**Output:** `DocumentAST` (medium-specific: SpecAST, DrawingAST, SubmittalAST)

---

### Stage 4: Editing (Optimizer)

**Input:** `DocumentAST` + edit operations

**Process:**
1. Apply edits to AST (insert, delete, replace)
2. Renumber automatically (cascade changes)
3. Validate structural integrity
4. Update provenance tracking

**Output:** `EditableDocModel` (modified AST)

**Rules:**
- Preserve hierarchy
- Maintain outline numbering consistency
- Track all changes in audit trail

---

### Stage 5: Rendering (Code Generator)

**Input:** `EditableDocModel` + `ChromeMetadata`

**Process (Specs):**
1. Render AST to HTML/CSS
2. Apply chrome template (headers/footers with metadata)
3. Convert HTML to PDF (headless Chrome)
4. Validate output (page count, formatting)

**Process (Drawings):**
1. Replace sheets (verbatim page swap)
2. Update bookmarks
3. Preserve unchanged pages (byte-identical)

**Output:** Regenerated PDF

**Chrome Reapplication:**
- Headers: Preserve project info, firm branding
- Footers: Update date, recalculate page numbers, preserve section ID

---

## Data Flow

### Spec Addendum Workflow (End-to-End)

```
Input: spec-rev0.pdf, addendum-edits.json
  ↓
[Stage 1: Layout Extraction]
  Load PDF → Extract spans → Normalize coords → Validate
  ↓ LayoutTranscript
  ↓
[Stage 2: Furniture Detection]
  Detect chrome → Extract metadata → Confidence scoring
  ↓ FurnitureRegions + ChromeMetadata
  ↓
[Stage 3: Semantic Parsing]
  Group lines → Parse outlines → Build AST → Classify sections
  ↓ SpecAST
  ↓
[Stage 4: Editing]
  Apply edits → Renumber → Validate → Track provenance
  ↓ EditableDocModel
  ↓
[Stage 5: Rendering]
  Render HTML → Apply chrome → Generate PDF → Validate
  ↓ spec-rev1.pdf
  ↓
[Audit Trail]
  JSONL events + overlays + metrics
  ↓ audit-bundle/
  
Output: spec-rev1.pdf + audit-bundle/
```

---

### Drawing Sheet Replacement Workflow

```
Input: drawings-rev0.pdf, addendum-drawings.pdf
  ↓
[Stage 1: Layout Extraction]
  Load both PDFs → Extract spans → Normalize
  ↓ LayoutTranscript (original + addendum)
  ↓
[Stage 2: Furniture Detection]
  Detect title blocks → Extract sheet IDs → Extract metadata
  ↓ SheetInventory (both sets)
  ↓
[Stage 3: Semantic Parsing]
  Build inventory → Classify disciplines (UDS) → Sort by order
  ↓ DrawingAST (sheet list with metadata)
  ↓
[Stage 4: Sheet Matching]
  Match sheets by ID → Detect renaming → Build replacement plan
  ↓ ReplacementPlan
  ↓
[Stage 5: PDF Merging]
  Replace matching sheets → Preserve unchanged → Update bookmarks
  ↓ drawings-rev1.pdf
  ↓
[Audit Trail]
  JSONL events (which sheets replaced, renamed)
  ↓ audit-bundle/
  
Output: drawings-rev1.pdf + audit-bundle/
```

---

## Type System

### Core Types

```rust
// LayoutIR (Stage 1 output)
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
    pub x: f32,      // [0.0, 1.0]
    pub y: f32,      // [0.0, 1.0], top-left origin
    pub width: f32,
    pub height: f32,
}
```

---

### Furniture & Chrome

```rust
// Stage 2 output
pub struct FurnitureRegions {
    pub header_regions: Vec<Region>,
    pub footer_regions: Vec<Region>,
    pub chrome_metadata: ChromeMetadata,
}

pub struct Region {
    pub page_index: usize,
    pub bbox: BBox,
    pub confidence: f32,
}

// Chrome metadata (preserved for reuse)
pub struct ChromeMetadata {
    pub project_id: String,
    pub project_name: String,
    pub date: String,
    pub firm_name: String,
    pub firm_logo: Option<ImageRef>,
    pub medium_specific: MediumChrome,
}

// Medium-specific chrome
pub enum MediumChrome {
    Spec(SpecChrome),
    Drawing(DrawingChrome),
    Submittal(SubmittalChrome),
}

pub struct SpecChrome {
    pub section_id: String,
    pub section_title: String,
}

pub struct DrawingChrome {
    pub sheet_id: String,
    pub sheet_title: String,
    pub discipline: String,
    pub revision: String,
}
```

---

### Document ASTs (Stage 3 output)

```rust
// Spec AST
pub struct SpecAST {
    pub sections: Vec<Section>,
}

pub struct Section {
    pub section_id: String,
    pub title: String,
    pub parts: Vec<Part>,
    pub page_range: (usize, usize),
    pub masterformat: MasterFormatMeta,
}

pub struct Part {
    pub number: usize,
    pub title: String,
    pub articles: Vec<Article>,
}

pub struct Article {
    pub number: String,  // "1.1", "2.3"
    pub title: String,
    pub paragraphs: Vec<Paragraph>,
}

pub struct Paragraph {
    pub marker: String,  // "A.", "1.", "a."
    pub text: String,
    pub sub_paragraphs: Vec<Paragraph>,
    pub provenance: Provenance,
}

// Drawing AST
pub struct DrawingAST {
    pub sheets: Vec<Sheet>,
}

pub struct Sheet {
    pub sheet_id: String,
    pub title: String,
    pub discipline: DisciplineMeta,
    pub page_index: usize,
    pub schedules: Vec<Table>,
}

// Submittal AST
pub struct SubmittalAST {
    pub units: Vec<EquipmentUnit>,
}

pub struct EquipmentUnit {
    pub tag: String,
    pub equipment_type: String,
    pub data: HashMap<String, Value>,
}
```

---

### Audit Types

```rust
pub struct AuditEvent {
    pub timestamp: String,     // ISO 8601 UTC
    pub stage: String,         // "layout_extraction", "furniture_detection", etc.
    pub event: String,         // "section_detected", "discipline_classified", etc.
    pub confidence: Option<f32>,
    pub basis: Option<String>, // "UDS", "MASTERFORMAT", "HEURISTIC", etc.
    pub details: serde_json::Value,
}

pub struct AuditTrail {
    events: Vec<AuditEvent>,
}

pub struct AuditBundle {
    pub manifest: Manifest,
    pub events: AuditTrail,
    pub overlays: Vec<Overlay>,
    pub metrics: Metrics,
}
```

---

## Standards Integration

### UDS Integration (Drawings)

**Component:** `drawing-processor` uses `standards::uds`

**Flow:**
```rust
// 1. Extract sheet ID from title block
let sheet_id = extract_sheet_id(&page)?;

// 2. Normalize ID format
let normalized_id = normalize_id(&sheet_id);

// 3. Classify discipline (UDS)
let discipline = standards::uds::classify(
    &normalized_id,
    &sheet_title,
)?;

// 4. Quality gate
if discipline.confidence < 0.80 {
    escalate_to_review(...);
    continue;
}

// 5. Add to inventory
inventory.push(Sheet {
    sheet_id: normalized_id,
    discipline,
    ...
});

// 6. Log audit event
audit.log(AuditEvent {
    event: "discipline_classified",
    confidence: Some(discipline.confidence),
    basis: Some(discipline.basis.clone()),
    ...
});
```

**Standards alignment:**
- Confidence thresholds from AEC_STANDARDS
- Basis types: UDS, ALIAS, HEURISTIC, UNKNOWN
- Deterministic sort by `discipline.order`

---

### MasterFormat Integration (Specs)

**Component:** `spec-processor` uses `standards::masterformat`

**Flow:**
```rust
// 1. Extract section ID from footer
let section_id = extract_section_id_from_footer(&footer_text)?;

// 2. Classify MasterFormat
let masterformat = standards::masterformat::classify(&section_id)?;

// 3. Quality gate
if masterformat.confidence <= 0.2 {
    escalate_to_review(...);
    continue;
}

// 4. Add to section list
sections.push(Section {
    section_id,
    masterformat,
    ...
});

// 5. Log audit event
audit.log(AuditEvent {
    event: "section_classified",
    confidence: Some(masterformat.confidence),
    basis: Some(masterformat.basis.clone()),
    ...
});
```

**Standards alignment:**
- Confidence: 1.0 for known divisions, 0.7 for unknown (valid format), 0.2 for invalid
- Basis types: MASTERFORMAT, UNKNOWN
- Deterministic sort by `division` number

---

## Quality Enforcement

### Determinism Enforcement

**Mechanisms:**

1. **No randomness:** Banned in production code
2. **Deterministic sorting:** Explicit comparators
3. **Deterministic tie-breaking:** Lower order value wins
4. **Fixed-point geometry:** Use integers where precision matters
5. **Stable IDs:** Derived from input, never computed

**Regression Test (Nightly):**
```bash
# Run engine on same input
cargo run --release -- extract test.pdf > output.json

# Compare to golden file
diff output.json tests/golden/test.golden.json

# Exit code 0 = identical, non-zero = FAIL
```

---

### Quality Gates (Per Stage)

| Stage | Metric | Threshold | Action |
|-------|--------|-----------|--------|
| **Layout Extraction** | Invariants | 100% pass | auto-apply |
| | | any fail | hard fail |
| **Furniture Detection** | confidence | ≥0.95 | auto-apply |
| | | 0.80–0.95 | apply + flag |
| | | <0.80 | escalate |
| **Discipline Classification** | confidence | ≥0.95 | auto-apply |
| | | 0.80–0.95 | apply + flag |
| | | <0.80 | escalate |
| **Section Classification** | confidence | 1.0 | auto-apply |
| | | 0.7 | apply + audit |
| | | ≤0.2 | escalate |

---

### Escalation Workflow

```
Low-confidence detection
  ↓
Log event with all candidates + basis
  ↓
Emit "Needs Review" flag
  ↓
User sees candidates + reasoning in UI
  ↓
User provides correction (ignore, override, etc.)
  ↓
Re-run with correction applied
```

---

### Audit Trail Requirements

**Every operation must log:**
- Timestamp (ISO 8601 UTC)
- Stage (layout_extraction, furniture_detection, etc.)
- Event type (section_detected, discipline_classified, etc.)
- Confidence (0.0–1.0)
- Basis (UDS, MASTERFORMAT, HEURISTIC, UNKNOWN)
- Details (event-specific data)

**JSONL format (one event per line):**
```jsonl
{"timestamp":"2026-01-23T10:50:00Z","stage":"furniture_detection","event":"footer_detected","page":0,"confidence":0.98,"basis":"PATTERN"}
{"timestamp":"2026-01-23T10:50:01Z","stage":"section_classification","event":"section_classified","section_id":"23 82 16","division":"23","confidence":1.0,"basis":"MASTERFORMAT"}
```

---

## Extension Points

### Adding a New Medium

**Example: Change Orders (COs)**

**Steps:**
1. Create `co-processor` crate
2. Define `COAST` (ChangeOrderAST)
3. Implement furniture detection (CO-specific chrome patterns)
4. Implement semantic parsing (CO structure)
5. Add standards module (if needed)
6. Add integration tests
7. Register with engine

**No changes to:**
- `ir` (LayoutIR is medium-neutral)
- `audit` (audit framework is cross-medium)
- `pdf-extraction` (extraction is universal)

---

### Adding a New Workflow

**Example: Automated Sequencing**

**Steps:**
1. Create workflow module in `engine/workflows/sequencing/`
2. Define input/output types
3. Implement analyze/execute pattern
4. Add CLI command
5. Add integration tests

**Architecture ensures:**
- Reuse existing processors
- Reuse audit framework
- Maintain determinism

### Baseline Operation Surface (Imported)

The following operation families are retained as durable external surface expectations for ongoing evolution:

1. `merge-addenda` style workflows: analyze sheet/section mapping, apply corrections, then execute deterministic replacement/insertion.
2. `split-set` style workflows: partition by normalized keys (discipline/section/division) with deterministic grouping.
3. `assemble-set` style workflows: deterministic assembly order with explicit order source.
4. `fix-bookmarks` style workflows: destination validation, repair, and deterministic rebuild.
5. `detect` style workflows: read-only detection reports for IDs and confidence.
6. `specs-patch` style workflows: section-scoped changes, never implicit whole-document rewrite.

Operation names may evolve, but the workflow invariants and deterministic behavior contracts remain mandatory.

---

### Adding a New Pattern

**Using Pattern Dev Tool (Phase 0.5):**

**Steps:**
1. Write pattern in JSON format
2. Test against sample PDFs with pattern-dev tool:
   ```bash
   cargo run --bin pattern-dev -- test-pattern \
     --pdf test.pdf \
     --pattern-type footer_section_id \
     --regex "^(\d{2}\s\d{2}\s\d{2})\s+–" \
     --output-overlays debug/
   ```
3. Review visual overlays (matched regions highlighted)
4. Validate confidence scores
5. Add pattern to database
6. Run regression tests
7. Commit pattern with tests

**Tool ensures:**
- Visual validation (see what matched)
- Confidence calibration (see scores)
- Test coverage (patterns validated before integration)

---

## Appendix: Alignment Matrix

| Component | DEV_STANDARDS | AEC_STANDARDS | Master Plan |
|-----------|---------------|---------------|-------------|
| **LayoutIR** | Determinism (1.1), Correctness (1.2) | — | Stage 1 |
| **Furniture Detection** | Auditability (1.3), Fail Explicitly (1.4) | Medium-specificity (1.1) | Stage 2 |
| **Spec Processor** | Medium-specificity (1.5) | MasterFormat (Div 00-49) | Phase 2-3 |
| **Drawing Processor** | Medium-specificity (1.5) | UDS (G-V), Sort order | Phase 9 |
| **Standards Module** | Confidence scoring (3.3) | UDS + MasterFormat | Shared |
| **Audit Framework** | Auditability (1.3) | — | All phases |
| **Pattern Dev Tool** | — | — | Phase 0.5 |

---

## Phase D Imported Architecture Constraints

### Workflow State Invariants

The workflow engine must preserve this state model:

1. `analyze` is read-only and creates an explicit analyzed state artifact.
2. `applyCorrections` mutates analyzed state overlays without re-running detection by default.
3. `execute` consumes analyzed state + corrections and performs file mutations.

This protects auditability, preserves user corrections, and prevents hidden re-analysis drift.

### MergePlan Ordering Contract

Plan array ordering is semantic, not presentation. Ordering in plan fields must be preserved across serialization, correction, and execution.

### Medium-Specific Chrome Detectors

Drawings, specs, and submittals require separate chrome detector implementations behind shared interfaces. Shared utility code is allowed, but detection logic must remain medium-specific.

### Transcript Coordinate Contract

Canonical transcript coordinate semantics must remain centralized and explicit:

1. One normalized visual coordinate convention across stages.
2. Rotation normalization applied before persistence/hashing.
3. Precision policy applied consistently across extraction paths.

### Transcript-First Extraction Contract

Extraction must follow a single-pass, cache-and-serve model:

1. Extract once per document invocation; do not re-extract per consumer.
2. Canonicalize extracted data immediately at the extraction boundary before any downstream consumer accesses it.
3. The transcript contract must remain stable and independent of the extraction backend implementation, so that backends can be replaced without rewriting dependent logic.
4. Backend failure must trigger explicit controlled fallback, not silent quality degradation.
5. Backend geometric fidelity differences are first-class architectural properties, not implementation details — they directly affect downstream analysis capability.

### Profile-Driven Detection Architecture

Layout detection for drawings, specs, and submittals must follow a profile-first strategy:

1. High-confidence detection requires explicit user-controlled profiles with deterministic ROI coordinates, not probabilistic auto-inference.
2. Layout profiles must specify ordered ROI fallbacks (ROI-1, ROI-2, ...) for bounded resilience without unconstrained page-wide searching.
3. Profile schemas must include strict validation gates at load time (required fields, ROI bounds, shape checks) before any execution.
4. Explicit diagnostic failure codes per ROI (e.g., `ROI_EMPTY`, `ROI_LOW_DENSITY`, `ROI_NO_PATTERN_MATCH`) are required to enable profile debugging.
5. Heuristic fallback must be feature-flagged to signal architectural direction while preserving recovery capability.
6. ROI results with low-confidence IDs must be accepted with warning rather than silently routing to fallback.

### Phase D Monorepo Integration Outcomes (M-001 to M-003)

These integration outcomes are canonical architectural constraints derived from Phase D and applied to the current monorepo boundary design.

- M-001 Transcript-first contract stabilization:
  - `crates/contracts` is the shared schema authority for workflow request/response and audit payload surface types.
  - Engine internals remain `LayoutTranscript`-typed to avoid transport coupling.
  - CLI/GUI transport translation occurs at application boundaries (for example `apps/backend-cli` handlers), not inside engine internals.
  - Result: Stable CLI/GUI IPC contracts without forcing transport schema types into low-level engine stages.
- M-002 Workflow ordering and gate semantics:
  - `crates/workflows` defines workflow trait/context/result contracts and operation routing surface.
  - Operation sequencing remains explicit and deterministic: `analyze -> applyCorrections -> execute`.
  - Operation dispatch routes by workflow operation contract instead of ad hoc command branching.
  - Result: Dedicated workflow crate boundary with deterministic execution contract.
- M-003 Audit event schema and session accounting:
  - `crates/audit` event model aligns to `contracts::AuditEventData` for transport-safe typed events.
  - Session lifecycle events (`SessionStarted`, `SessionEnded`, operation events) are mandatory runtime emissions.
  - Per-run audit bundles and manifest metadata are required traceability artifacts.
  - Result: Type-safe, consistent audit lifecycle across runtime surfaces.

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 4.0.0 | 2026-01-21 | Initial architecture document |
| 4.2.0 | 2026-01-23 | **Aligned with MASTER_PLAN.** Added: (1) Chrome metadata preservation in furniture detection stage, (2) Pattern Development Tool architecture (Phase 0.5 critical infrastructure), (3) Updated component diagram and data flow to include chrome handling, (4) Added medium-specific chrome types, (5) Clarified compiler pipeline stages with chrome reapplication in rendering, (6) Updated type system with ChromeMetadata structures. Simplified: Reduced code examples (moved to DEV_STANDARDS), focused on "what" not "how", cleaner separation between architecture and implementation. |
| 4.2.1 | 2026-03-23 | Added canonical Phase D monorepo integration outcomes (M-001 to M-003): contracts boundary stabilization, workflow gate ordering contract, and typed audit/session lifecycle constraints. |

---

**Status:** ✅ ACTIVE  
**Owner:** HLLMR LLC  
**Last Updated:** March 23, 2026  
**Version:** 4.2.1

---

**End of ARCHITECTURE Document**
