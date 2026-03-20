## Plan: Prototype Codebase Extraction & Post-Mortem Documentation

Create a structured library of post-mortem documents capturing the architecture, workflows, lessons, and edge cases from the two prototype repos. This library will serve as the definitive knowledge-transfer artifact for the **Rust reimplementation**—the goal is not a historical record for its own sake, but a complete engineering brief that the Rust implementation can be built directly from.

---

**Context**

This prototype is being sunset. Target successor is a **single Rust binary** with no Python runtime dependency for end users (per V4 Non-Negotiable #10). The two prototype repos are:

- `conset-pdf` — TypeScript monorepo: core library (`@conset-pdf/core`) + CLI (`@conset-pdf/cli`)
- `conset-pdf-gui` — Electron desktop GUI consuming `@conset-pdf/core` via IPC

---

**Execution Model (Required for Multi-Agent Runs)**

This plan is designed to be executed with one agent per phase. Each phase must produce concrete artifacts on disk before the next phase begins.

### Output Root and Folder Layout

All post-mortem outputs must be written under:

- `/docs/prototype-postmortem/`

Required structure:

- `/docs/prototype-postmortem/00-admin/`
- `/docs/prototype-postmortem/01-inventory/`
- `/docs/prototype-postmortem/02-algorithms/`
- `/docs/prototype-postmortem/03-adrs/`
- `/docs/prototype-postmortem/04-contracts/`
- `/docs/prototype-postmortem/05-gaps-and-limitations/`
- `/docs/prototype-postmortem/06-lessons/`
- `/docs/prototype-postmortem/07-rust-handoff/`
- `/docs/prototype-postmortem/08-verification/`
- `/docs/prototype-postmortem/09-ops-and-drift/`

### File Naming Convention

Use deterministic, sortable names:

- Phase summary: `phase-XX-summary.md`
- Topic docs: `phase-XX-<topic-kebab>.md`
- Tables/matrices: `phase-XX-<matrix-name>.md`
- JSON exports/data snapshots: `phase-XX-<artifact-name>.json`

Examples:

- `phase-01-module-inventory.md`
- `phase-03-adr-001-python-sidecar.md`
- `phase-04-workflow-types-contract.md`
- `phase-09-doc-drift-matrix.md`

### Write Timing Rules

1. During each phase, write/update artifacts continuously (do not wait until phase end).
2. Before phase completion, finalize `phase-XX-summary.md` with:
    - What was completed
    - Evidence/source files reviewed
    - Open questions and carry-forward risks
3. Next phase may start only after prior phase summary exists and required artifacts are present.

### Agent Handoff Contract

Each agent must read these before executing:

- `/docs/prototype-postmortem/00-admin/phase-manifest.md`
- Prior phase summary: `/docs/prototype-postmortem/<previous-phase>/phase-XX-summary.md`

Each agent must write these before exiting:

- Current phase summary: `/docs/prototype-postmortem/<current-phase>/phase-XX-summary.md`
- Updated manifest entry in `/docs/prototype-postmortem/00-admin/phase-manifest.md`
- Decision/risk updates in `/docs/prototype-postmortem/00-admin/open-questions-and-risks.md`

### Bootstrap Files (create first)

- `/docs/prototype-postmortem/README.md` (index + navigation)
- `/docs/prototype-postmortem/00-admin/phase-manifest.md` (phase status tracker)
- `/docs/prototype-postmortem/00-admin/open-questions-and-risks.md` (rolling backlog)
- `/docs/prototype-postmortem/00-admin/source-of-truth.md` (doc conflict resolutions)

---

**Steps**

### Phase 1: Module & Feature Inventory
1. Enumerate all implemented modules in `conset-pdf/packages/core/src/`:
   - `analyze/` — DocumentContext, PageContext, reading-order
   - `bookmarks/` — reader, validator, treeBuilder, headingResolver, corrections, pikepdf sidecar
   - `config/` — feature flags
   - `core/` — mergeAddenda, splitSet, assembleSet, planner, applyPlan, report
   - `layout/` — layout profile loading and types
   - `locators/` — compositeLocator, roiSheetLocator, roiSpecsSectionLocator, sheetLocator, specsSectionLocator, legacyTitleblockLocator
   - `narrative/` — text-extract, parse-algorithmic, normalize, validate
   - `parser/` — drawingsSheetId, specsSectionId, normalize
   - `specs/` — extract pipeline: chromeRemoval, anchorDetector, bookmarkTreeGenerator, hierarchyBuilder, listDetector, paragraphNormalizer, sectionDetector, tableDetector, textExtractor; footerIndexBuilder, footerSectionIdParser, footerSectionMap, footerValidation; inventory, patch/apply, patch/validator
   - `standards/` — normalizeDrawingsDiscipline, normalizeSpecsMasterformat (modern + legacy v3); registry, datasets, compare
   - `submittals/` — submittalParser
   - `transcript/` — extractors (PyMuPDF + PDF.js), factory, canonicalize, quality, candidates, abstraction (TokenVault, sanitize, lineGrouping, repetitionMetrics, shapeFeatures), ML compiler (rulesetCompiler, apiCompiler, promptBuilder), profiles (registry, validation), schedules (extractor, tableBuilder)
   - `workflows/` — engine, merge, specs-patch, bookmarks, mappers
   - `utils/` — bookmarks (legacy), bookmarkWriter, pdfLibBookmarkWriter, pikepdfBookmarkWriter, pdf, fs, sort
2. Enumerate all CLI commands in `conset-pdf/packages/cli/src/commands/`:
   - `assembleSet`, `debugWalkthrough`, `detect`, `fixBookmarks`, `mergeAddenda`, `specsInventory`, `specsPatch`, `splitSet`
3. Enumerate all GUI modules in `conset-pdf-gui/src/`:
   - Wizards: `merge-wizard.js`, `split-drawings-wizard.js`, `bookmark-wizard.js`, `placeholder-wizard.js`
   - Views: `profiles-view.js`, `settings-view.js`
   - Modules: `app/`, `bookmarks/`, `merge/`, `pdf/`, `profiles/`, `roi/`, `split/`
   - IPC handlers: `dialogs`, `pdf`, `profiles`, `detection`, `operations`, `history`, `system`, `merge`, `debug`, `settings`, `naming`, `standards`, `cache`
   - Profile store, history store, job worker
4. Record implementation status for each (Complete / Partial / Planned-but-not-started / Abandoned).

**Required Artifacts (Phase 1)**

- `/docs/prototype-postmortem/01-inventory/phase-01-module-inventory.md`
- `/docs/prototype-postmortem/01-inventory/phase-01-cli-command-inventory.md`
- `/docs/prototype-postmortem/01-inventory/phase-01-gui-module-inventory.md`
- `/docs/prototype-postmortem/01-inventory/phase-01-summary.md`

### Phase 2: Algorithmic IP Capture
Core algorithms that must survive the migration verbatim or be re-implemented with exact behavioral parity. Capture: inputs, outputs, invariants, and any known failure modes.

5. **ID Parsing & Normalization** (`parser/`)
   - Drawings sheet ID regex patterns for all known formats (e.g., `A-101`, `M1-01`, `T-1.4A`)
   - Specs section ID patterns: 6-digit modern with spaces (`XX YY ZZ`), legacy 5-digit (`XXYYY`)
   - Normalization rules (uppercase, whitespace collapse, format detection)
6. **ROI-Based Detection** (`locators/roiSheetLocator.ts`, `roiSpecsSectionLocator.ts`)
   - How layout profiles (JSON) define detection regions (bounding boxes, relative coordinates)
   - How multi-span text is reassembled from ROI-filtered spans
   - Reading-order assembly from `readingOrder.ts` (visual order reconstruction)
   - Confidence scoring mechanics
   - Fall-back chain: ROI → LegacyTitleblock → SpecsSectionLocator
7. **Chrome Removal & Spec Extraction** (`specs/extract/`)
   - Header/footer band detection via Y-clustering + repetition analysis (candidates system)
   - `chromeRemoval.ts` algorithm: band membership, threshold tuning, known edge cases
   - Paragraph normalization: wrap-join heuristics, hyphen repair
   - Section detection patterns: heading regexes, hierarchy inference
   - Anchor detection, list detection
   - Footer-First Section Anchoring: why page-level footer detection is needed for deterministic bookmark destinations
8. **Transcript Canonicalization** (`transcript/canonicalize.ts`)
   - Rotation normalization: how page coordinates are transformed to rotation=0 basis
   - Coordinate system convention (top-left origin, y=0 at top)
   - Stable sort algorithm (y-position primary, x-position secondary, deterministic tie-break)
   - Content hash and span hash construction (what fields are included/excluded)
9. **Merge Planning** (`core/planner.ts`, `core/applyPlan.ts`)
   - How replace/insert/append-only modes work
   - Duplicate handling (highest confidence wins)
   - Unmatched page behavior
   - Byte-verbatim page copy invariant
10. **Standards Normalization** (`standards/`)
    - UDS discipline designator table and multi-letter alias map
    - Disambiguation heuristics for ambiguous designators (e.g., `C`: Civil vs. Controls)
    - MasterFormat division table (all 50 divisions), legacy→modern mapping
    - Canonical discipline sort order (why this order was chosen)
11. **Narrative PDF Algorithmic Parser** (`narrative/parse-algorithmic.ts`)
    - How addendum narrative text is parsed into sheet/section reference structures
    - Near-match suggestion algorithm
    - Validation against inventory
12. **Quality Scoring** (`transcript/quality.ts`)
    - Per-page metrics and thresholds (char count, whitespace ratio, replacement chars, ordering sanity)
    - Aggregate confidence gate
    - When to accept PyMuPDF vs. fall back to PDF.js
13. **Schedule Table Extraction** (`transcript/schedules/`)
    - Geometry-first approach: column/row detection via coordinate clustering
    - Merged cell handling
    - Rotated header text handling

**Required Artifacts (Phase 2)**

- `/docs/prototype-postmortem/02-algorithms/phase-02-id-parsing-and-normalization.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-roi-detection.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-specs-extraction-and-chrome-removal.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-transcript-canonicalization.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-merge-planning.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-standards-normalization.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-narrative-parser.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-quality-scoring.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-schedule-extraction.md`
- `/docs/prototype-postmortem/02-algorithms/phase-02-summary.md`

### Phase 3: Architecture Decision Records (ADRs)
Document the **why** behind each major architectural choice, especially those that diverge from the obvious approach.

14. **ADR-001: Python Sidecar Pattern**
    - Why: pikepdf (LGPL-licensed) and PyMuPDF cannot be bundled in an Apache-2.0 library; sidecar isolates licensing boundary
    - Affected subsystems: transcript extraction (PyMuPDF), bookmark writing (pikepdf/QPDF), and the prototype's planned/partial pikepdf-backed output path for large-file merge handling
    - Rust alternative: PDFium (Apache-2.0) for extraction, lopdf or pdf-rs for structure manipulation
    - Known issues: process startup overhead; sidecar versioning coupling
15. **ADR-002: Transcript-First Extraction (v3 migration)**
    - Why: PDF.js `getTextContent()` gives unreliable bounding boxes; PyMuPDF's dict/rawdict-first approach gives 95-99% accuracy needed for geometric analysis
    - Trade-off: in the prototype, the preferred path requires Python at runtime and is invoked through Node.js spawn; acceptable for v3 but must be removed in Rust
    - Rust alternative: PDFium text extraction provides equivalent or better accuracy natively
16. **ADR-003: Workflow Engine Pattern (analyze → applyCorrections → execute)**
    - Why: separates read-only analysis from destructive writes; enables corrections UI and staged execution (current implementation still re-runs analyze in applyCorrections)
    - How cursor state moves between phases (plan caching, corrections overlay)
    - Must preserve in Rust
17. **ADR-004: Disk-Based Merge via Pikepdf Sidecar**
    - Why: Large PDFs (>500MB) caused OOM with in-memory pdf-lib assembly
    - Merge plan JSON serialization for cross-language handoff is the intended target architecture; current code does not yet execute merge assembly through that serialized handoff
    - Atomic temp-file-then-rename pattern for safe overwrite
    - Note: current code only partially realizes this direction; final output is routed through pikepdf, but page assembly still occurs in memory
    - Rust: implement as native streaming merge from day one
18. **ADR-005: Determinism as a Design Invariant**
    - Enumerate all known sources of non-determinism eliminated: date fields in hashes, sort tie-breaks, floating-point coordinate rounding, Python random seeding
    - Rust must inherit these constraints
19. **ADR-006: Profile-Driven Detection vs. Auto-Detection**
    - Why explicit profiles are required: auto-detection was prototyped but too error-prone on the diversity of AEC title block layouts
    - JSON profile schema description (ROI bounding boxes, profile type, page model)
    - Profile registry and versioning scheme
    - Match criteria for automatic profile selection
20. **ADR-007: Privacy-Preserving ML Abstraction**
    - TokenVault: how sensitive text is replaced with structural placeholders before sending to LLM
    - Three privacy modes (STRICT_STRUCTURE_ONLY, WHITELIST_ANCHORS, FULL_TEXT_OPT_IN)
    - Why this is necessary: AEC documents contain PII and proprietary project data
21. **ADR-008: Technology Choices Abandoned**
    - `pdf-lib` bookmark writing path was superseded by pikepdf/QPDF direction, but still remains active in parts of merge execution (capture current-state drift)
    - `PDF.js` extraction was demoted from primary to fallback/auxiliary role after PyMuPDF migration (not fully removed)
    - In-memory merge remains partially active while disk-based pikepdf strategy is the target architecture
    - Broad generic PDF AST ambition was abandoned, but scoped specs AST workflows remain active and should not be mislabeled as removed
    - Legacy locator system → deprecated (ROI profiles solved the same problem more reliably)
    - LLM-assisted narrative parsing → deferred (algorithmic parsing covers most cases)

**Required Artifacts (Phase 3)**

- `/docs/prototype-postmortem/03-adrs/phase-03-adr-001-python-sidecar.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-adr-002-transcript-first.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-adr-003-workflow-engine.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-adr-004-disk-streaming-merge.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-adr-005-determinism.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-adr-006-profile-driven-detection.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-adr-007-privacy-abstraction.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-adr-008-abandoned-choices.md`
- `/docs/prototype-postmortem/03-adrs/phase-03-summary.md`

### Phase 4: Data Schemas & Type Contracts
All canonical data shapes that must be preserved or explicitly remapped in Rust.

22. **Layout Profile JSON schema** (`layouts/layout-template.json`, `layout/types.ts`)
    - Full schema with all fields annotated
    - Profile types: drawings vs. specs
    - ROI coordinate system and normalization rules
23. **LayoutTranscript types** (`transcript/types.ts`)
    - `LayoutTranscript`, `LayoutPage`, `LayoutSpan`, bbox representation
    - All metadata fields (font, size, flags, color, block/line/span hierarchy)
24. **Workflow types** (`workflows/types.ts`)
    - `InventoryResult`, `InventoryRowBase`, `CorrectionOverlay`, `ExecuteResult`
    - Issue type enum and severity model
    - `MergePlan` and `MergeAction` types
25. **BookmarkAnchorTree** and related bookmark types (`bookmarks/types.ts`)
    - `BookmarkNode`, `BookmarkDestination`, `BookmarkTree`
    - How `BookmarkAnchorTree` carries spec outline structure from specs-patch workflow to bookmarks workflow
26. **IPC Response Envelope** (`conset-pdf-gui/src/shared/ipc-response.ts`)
    - `IpcResponse<T>` pattern for all inter-process communication
    - Why the envelope shape was chosen (stability under error-handling changes)
27. **Report and Inventory File formats** (`docs/OUTPUT_STRUCTURE.md`, `core/report.ts`)
    - Full JSON shape for merge reports, inventory files, preview/detection results
    - File naming conventions
28. **Standards types** (`standards/types.ts`, `FIELD_NAMING_GUIDE.md`)
    - Context-specific field naming conventions (drawings vs. specs)
    - `DrawingsDisciplineMeta`, `SpecsMasterformatMeta`
    - `StandardsBasis`, `SpecsBasis` confidence/basis enums

**Required Artifacts (Phase 4)**

- `/docs/prototype-postmortem/04-contracts/phase-04-layout-profile-schema.md`
- `/docs/prototype-postmortem/04-contracts/phase-04-layout-transcript-contract.md`
- `/docs/prototype-postmortem/04-contracts/phase-04-workflow-types-contract.md`
- `/docs/prototype-postmortem/04-contracts/phase-04-bookmark-types-contract.md`
- `/docs/prototype-postmortem/04-contracts/phase-04-ipc-envelope-contract.md`
- `/docs/prototype-postmortem/04-contracts/phase-04-output-formats-contract.md`
- `/docs/prototype-postmortem/04-contracts/phase-04-standards-types-contract.md`
- `/docs/prototype-postmortem/04-contracts/phase-04-summary.md`

### Phase 5: Incomplete Work & Known Limitations Inventory
Capture everything that was designed but not finished, so the Rust implementation doesn't re-derive the same conclusions.

29. **Unimplemented GUI workflows:**
    - Report Viewer (placeholder only; see GUI ROADMAP.md)
    - Specs GUI wizard (engine + CLI complete; GUI not wired)
    - Bookmarks GUI wizard (engine + CLI complete; GUI fully TBD)
    - Submittal workflow (type stubs + parser stub only; no workflow engine or CLI command)
    - Placeholder PDF wizard (UI shell only)
30. **Planned but not started in this prototype:**
    - ROI auto-detection from PDFs (automatedRoiRefactorPlan.md — algorithm designed but not built)
    - Equipment schedule extraction UI
    - Web/SaaS mode (V4 Master Plan phase)
    - Pattern Development Tool (V4 Non-Negotiable #20 — no implementation)
    - Audit bundle / overlay visual export
    - LLM-assisted narrative parsing integration
31. **Known failure modes and edge cases** (document from `test-output/`, `tests/fixtures/`, `tests/workflows/`):
    - Footer detection edge cases: multi-column footers, rotated footers, footers with non-standard delimiters
    - Sheet ID parsing failures: sheets with blank IDs, sheets with identical IDs, non-standard alphanumeric formats
    - Specs section detection: legacy 5-digit codes embedded in modern books, unnumbered cover sections
    - Chrome removal false positives/negatives: stamps, revision clouds, watermarks misclassified
    - Bookmark writing failures: destination page mismatch after merge operations (headingResolver limitations)
    - Large file handling: OOM edge cases prior to disk-based streaming fix
32. **Known technical debt:**
    - `DocumentContext` still loads a separate PDF.js document for bookmarks (temporary, pending full migration)
    - `pdfLibBookmarkWriter` retained for development/testing but not production
    - `utils/bookmarks.ts` legacy bookmark generation is deprecated but not removed
    - Specs workflow uses Playwright for HTML→PDF rendering (adds a large dependency)
    - `featureFlags.ts` is sparsely used; feature flag discipline not fully established
    - Workflow corrections overlay is loaded on `execute()` but ignored in Phase 1 of merge driver (doc notes this as known limitation)

**Required Artifacts (Phase 5)**

- `/docs/prototype-postmortem/05-gaps-and-limitations/phase-05-unimplemented-features.md`
- `/docs/prototype-postmortem/05-gaps-and-limitations/phase-05-failure-modes-catalog.md`
- `/docs/prototype-postmortem/05-gaps-and-limitations/phase-05-technical-debt-register.md`
- `/docs/prototype-postmortem/05-gaps-and-limitations/phase-05-summary.md`

### Phase 6: Post-Mortem Write-Up
33. **Architecture Overview doc** — one-page narrative + module dependency graph; entry point for new Rust developers
34. **Workflow Breakdown docs** (one per workflow):
    - Update Documents (Merge): drawings and specs lanes, multi-addendum handling, narrative validation, corrections
    - Extract Documents (Split): section/division grouping, output naming, format configuration
    - Fix Bookmarks: anchor resolution, tree building, correction operations, QPDF sidecar
    - Specs Patch: AST extraction, patch operations, Playwright render, BookmarkAnchorTree output
35. **Lessons Learned doc** — three sections:
    - What worked (ROI profiles, workflow engine pattern, transcript system, Python-sidecar decoupling, pikepdf for merges)
    - What failed (in-memory merge, pdf-lib bookmarks, PDF.js extraction accuracy, PDF AST concept)
    - What was deferred and why (LLM narrative, auto-ROI, schedule UI, submittals)
36. **Edge Case Catalog** — enumerate real failure cases encountered; include sample inputs where available; categorize by module
37. **Non-Negotiables as Rust Design Constraints** — reframe all 20 V4 non-negotiables as concrete Rust implementation requirements; call out which ones had prototype implementations that proved the constraint (e.g., #12: byte-verbatim unchanged pages), and which ones were aspirational but not yet tested at scale

**Required Artifacts (Phase 6)**

- `/docs/prototype-postmortem/06-lessons/phase-06-architecture-overview.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-workflow-breakdown-merge.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-workflow-breakdown-split.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-workflow-breakdown-bookmarks.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-workflow-breakdown-specs-patch.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-lessons-learned.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-edge-case-catalog.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-non-negotiables-rust-constraints.md`
- `/docs/prototype-postmortem/06-lessons/phase-06-summary.md`

### Phase 7: Reference Library Assembly & Rust Handoff
38. Organize all extraction documents into `/docs/prototype-postmortem/` with a clear index
39. Write a **Rust Port Primer** doc that maps:
    - Each TypeScript module → Rust crate/module equivalent
    - Each Python sidecar script → Rust library candidate (PDFium, lopdf, printpdf, etc.)
    - Each JSON schema → Rust struct (with serde annotations)
    - Key external dependencies and their Rust equivalents or alternatives
40. Cross-reference all ADRs and lessons learned back to specific V4 Master Plan phases
41. Flag which datasets (standards tables, regex patterns, profile schemas) can be ported verbatim vs. which require algorithmic re-expression

**Required Artifacts (Phase 7)**

- `/docs/prototype-postmortem/README.md` (updated final index)
- `/docs/prototype-postmortem/07-rust-handoff/phase-07-rust-port-primer.md`
- `/docs/prototype-postmortem/07-rust-handoff/phase-07-ts-to-rust-module-map.md`
- `/docs/prototype-postmortem/07-rust-handoff/phase-07-sidecar-replacement-plan.md`
- `/docs/prototype-postmortem/07-rust-handoff/phase-07-dataset-portability-matrix.md`
- `/docs/prototype-postmortem/07-rust-handoff/phase-07-summary.md`

### Phase 8: Verification
42. Review all docs against actual source files; ensure no major module is missing from coverage
43. Confirm all 8 ADRs address the decisions that had the most implementation impact
44. Validate that every item in Phase 5 (Incomplete Work) either has a Rust design answer in Phase 7 or is explicitly deferred with rationale
45. Do a final pass: anything in the codebase not mentioned in any extraction doc that a Rust developer would need to know?

**Required Artifacts (Phase 8)**

- `/docs/prototype-postmortem/08-verification/phase-08-coverage-checklist.md`
- `/docs/prototype-postmortem/08-verification/phase-08-validation-report.md`
- `/docs/prototype-postmortem/08-verification/phase-08-summary.md`

### Phase 9: Documentation Drift, Ops Learnings, and Risk Controls
46. **Documentation Drift Matrix**
    - Build a "Doc Claim vs Reality" matrix for command/workflow status, especially where docs disagree on what is active vs abandoned
    - Treat this matrix as a migration gate: Rust planning uses code truth + tests, not any single markdown source
    - Required comparison sources: `README.md`, `ROADMAP.md`, `docs/WORKFLOWS.md`, `docs/CLI.md`
47. **Deprecation Gates and Behavioral Flags**
    - Capture all deprecation controls and defaults, including `ENABLE_LEGACY_LOCATOR` behavior and failure messaging
    - Record which behavior is permanently removed vs temporarily gated for compatibility
    - Include migration implications for Rust feature toggles and compatibility shims
48. **Operational Telemetry and Supportability Lessons**
    - Extract logging architecture lessons from GUI implementation (log levels, rotation limits, exported bundles, system metadata capture, unhandled-error capture)
    - Document alpha bug-reporting process as a product requirement input for Rust-era diagnostics
    - Define minimum support telemetry baseline for Rust successor (startup/session markers, operation timing, failure context, reproducibility bundle)
49. **UX Critical Path and Sharp Edges**
    - Capture workflow-critical manual test paths and known sharp edges (IPC envelope constraints, ROI overlay lifecycle, wizard state persistence)
    - Distinguish correctness-critical UX rules from convenience behaviors
    - Translate these into explicit acceptance criteria for Rust GUI/API front ends
50. **ROI Coordinate Space Clarification and Test Guardrails**
    - Reconcile coordinate-space assumptions used in docs, profile schema, and extraction code paths (origin convention, normalization basis, rotation handling)
    - Add migration tests that fail on coordinate-space regressions
    - Require one canonical coordinate-spec reference doc and prohibit ambiguous wording in future docs

**Required Artifacts (Phase 9)**

- `/docs/prototype-postmortem/09-ops-and-drift/phase-09-doc-drift-matrix.md`
- `/docs/prototype-postmortem/09-ops-and-drift/phase-09-deprecation-gates.md`
- `/docs/prototype-postmortem/09-ops-and-drift/phase-09-ops-telemetry-lessons.md`
- `/docs/prototype-postmortem/09-ops-and-drift/phase-09-ux-critical-path.md`
- `/docs/prototype-postmortem/09-ops-and-drift/phase-09-roi-coordinate-spec.md`
- `/docs/prototype-postmortem/09-ops-and-drift/phase-09-summary.md`

---

**Relevant Files — conset-pdf**
- `docs/MASTER_PLAN_v4.md` — canonical product vision and non-negotiables
- `docs/ARCHITECTURE.md` — module overview and dependency rules
- `docs/TRANSCRIPT_ARCHITECTURE.md` — V3 extraction architecture
- `docs/WORKFLOWS.md` — workflow inputs/outputs/behaviors
- `docs/PUBLIC_API.md` — stable API contracts
- `docs/STANDARDS.md` — standards module design
- `docs/FIELD_NAMING_GUIDE.md` — field naming conventions (critical for Rust struct naming)
- `docs/OUTPUT_STRUCTURE.md` — output file formats and naming
- `docs/CLI.md` — CLI contracts, options, and command status claims
- `docs/LEGACY_SUPPORT_ARCHITECTURE.md` — legacy MasterFormat support
- `docs/LEGACY.md` — legacy code boundaries and migration guidance
- `DEPRECATION_CHANGES.md` — deprecation gating and feature-flag behavior
- `docs/ML_RULESET_COMPILER.md` — ML-assisted profile generation
- `docs/MIGRATION_V3.md` — transcript migration guide
- `docs/QUICK_START.md` — ROI-first usage path and coordinate conventions
- `docs/automatedRoiRefactorPlan.md` — ROI automation design (not implemented)
- `docs/largeFileRefactorPlan.md` — disk-based streaming merge design
- `ROADMAP.md` — implementation status for all features
- `CHANGELOG.md` — notable changes and abandoned approaches
- `packages/core/src/` — all source modules (see Phase 1 inventory)
- `packages/core/src/transcript/sidecar/extract-transcript.py` — PyMuPDF extraction script
- `packages/core/src/bookmarks/sidecar/bookmark-writer.py` — pikepdf bookmark writer
- `layouts/layout-template.json` — canonical layout profile schema
- `tests/` — smoke, workflows, standards, transcript, narrative test suites
- `test-output/` — real edge case outputs from test runs
- `scripts/` — developer diagnostic scripts (inspect-uds, inspect-narrative, test-v3-extraction, show-ml-input)

**Relevant Files — conset-pdf-gui**
- `docs/ARCHITECTURE.md` — GUI process architecture, IPC registration, store design
- `docs/IPC_CONTRACTS.md` — full IPC channel documentation
- `docs/UI_WORKFLOWS.md` — all wizard steps, UX behaviors, validation rules
- `docs/LOGGING.md` — logging system design
- `docs/LOGGING_IMPLEMENTATION.md` — concrete logging implementation details and constraints
- `docs/DEVELOPMENT.md` — smoke-test critical path and known sharp edges
- `docs/ALPHA_TESTING_GUIDE.md` — user-facing bug reporting guide (evidence of alpha deployment)
- `docs/PRE_ALPHA_CHECKLIST.md` — release readiness and support workflow checks
- `ROADMAP.md` — GUI completion status
- `CHANGELOG.md` — GUI changes, abandoned features
- `src/main/ipc/` — all IPC handlers
- `src/main/profiles/store.ts` — profile storage with migration
- `src/main/history/store.ts` — run history persistence
- `src/shared/ipc-response.ts` — IPC envelope type
- `src/modules/roi/roiOverlayController.js` — ROI drawing overlay controller
- `src/profiles-view.js` — profile management UI

---

**Verification**
1. Every module in the Phase 1 inventory has at least one extraction document covering it.
2. All 8 ADRs are written with: problem statement, decision made, alternatives considered, known consequences.
3. Every item in the Incomplete Work inventory has a disposition (ported / deferred / dropped) in the Rust Primer.
4. Rust team can bootstrap the core extraction pipeline using only the extraction docs, without reading source code.
5. Documentation Drift Matrix exists and all critical conflicts are resolved or explicitly marked with source-of-truth decisions.

---

**Decisions**
- Scope is both repos: `conset-pdf` (engine + CLI) and `conset-pdf-gui` (Electron GUI).
- Python sidecar scripts (`extract-transcript.py`, `bookmark-writer.py`) are in-scope; their logic must be fully documented even though the Rust port replaces them.
- Rust IP extraction is the primary goal; historical documentation is secondary.
- Do not produce summary docs that merely paraphrase existing `docs/` files—extraction docs must add new synthesis, cross-referencing, and Rust translation guidance.

---

**Further Considerations**
1. The `test-output/` directory and `tests/fixtures/` contain real PDFs and debug outputs that serve as the ground-truth edge case corpus; preserve or migrate these to the Rust test suite.
2. The `scripts/` directory contains developer diagnostics (`inspect-uds.ts`, `inspect-narrative.ts`, `test-v3-extraction.ts`, `show-ml-input.ts`) that are effectively executable documentation of the extraction pipeline—capture their logic in the algorithmic IP docs.
3. The 20 V4 non-negotiables in `MASTER_PLAN_v4.md` are the most concentrated statement of product requirements; they should be the first thing the Rust team reads. Prioritize Phase 6 item 37 early.
4. The layout profile JSON schema is a user-facing contract that must remain stable across the TypeScript → Rust transition; treat it as wire format, not internal state.
5. Standards datasets (discipline tables, MasterFormat divisions, legacy mappings) are pure data with high value—these can be ported to Rust as static tables or embedded JSON with zero algorithmic translation cost.
