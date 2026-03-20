# Prototype Post-Mortem Library

# Prototype Post-Mortem Library — Navigation Index

**Generated**: 2026-03-19  
**Status**: Phases 1-9 complete. Phase 4 and Phase 7 addenda applied after Phase 8 verification so the library reflects the final covered state.  
**Source plan**: `docs/postMortemDocExtraction.md`

---

## Purpose

Capture architecture, workflow, algorithmic, and operational lessons from the v3 TypeScript prototype and translate them into actionable guidance for the V4 Rust reimplementation. This library is the definitive engineering brief for the Rust implementation team.

**Start here if you are a new V4 developer**:  
→ `07-rust-handoff/phase-07-rust-port-primer.md`

---

## Admin Files

| File | Purpose |
|---|---|
| `00-admin/phase-manifest.md` | Phase status tracker (owner, dates, completion) |
| `00-admin/open-questions-and-risks.md` | Rolling backlog of residual product/code risks plus resolved/addendum tracking (R-001 through R-019) |
| `00-admin/source-of-truth.md` | Doc conflict resolutions (SOT-001 through SOT-012) |

---

## Phase Index

### Phase 1 — Module & Feature Inventory [COMPLETE]

Directory: `01-inventory/`

| Artifact | Description |
|---|---|
| `phase-01-module-inventory.md` | All core library modules with implementation status |
| `phase-01-cli-command-inventory.md` | All CLI commands with status |
| `phase-01-gui-module-inventory.md` | All GUI modules with status |
| `phase-01-summary.md` | Phase summary |

---

### Phase 2 — Algorithmic IP Capture [COMPLETE]

Directory: `02-algorithms/`

| Artifact | Description |
|---|---|
| `phase-02-id-parsing-and-normalization.md` | Drawing sheet ID and spec section ID regex patterns |
| `phase-02-roi-detection.md` | ROI-based detection algorithm, confidence scoring, fallback chain |
| `phase-02-specs-extraction-and-chrome-removal.md` | Chrome band detection, paragraph normalization, section detection |
| `phase-02-transcript-canonicalization.md` | Rotation normalization, stable sort, content hashing |
| `phase-02-merge-planning.md` | Merge plan construction: modes, conflict resolution, page semantics |
| `phase-02-standards-normalization.md` | UDS discipline table, MasterFormat division table, disambiguation |
| `phase-02-narrative-parser.md` | Addendum narrative parsing algorithm and near-match suggestion |
| `phase-02-quality-scoring.md` | Per-page and aggregate quality metrics; extractor acceptance gate |
| `phase-02-schedule-extraction.md` | Geometry-first equipment schedule table extraction |
| `phase-02-summary.md` | Phase summary |

---

### Phase 3 — Architecture Decision Records [COMPLETE]

Directory: `03-adrs/`

| Artifact | Decision |
|---|---|
| `phase-03-adr-001-python-sidecar.md` | Why Python sidecars were used; V4 replacement direction |
| `phase-03-adr-002-transcript-first.md` | PyMuPDF promoted over PDF.js; trade-offs |
| `phase-03-adr-003-workflow-engine.md` | Analyze → applyCorrections → execute pattern rationale |
| `phase-03-adr-004-disk-streaming-merge.md` | Large-file OOM; disk-streaming target architecture |
| `phase-03-adr-005-determinism.md` | All non-determinism sources eliminated |
| `phase-03-adr-006-profile-driven-detection.md` | Explicit ROI profiles vs. auto-detection |
| `phase-03-adr-007-privacy-abstraction.md` | TokenVault and three privacy modes |
| `phase-03-adr-008-abandoned-choices.md` | pdf-lib, PDF.js primary, in-memory merge, generic PDF AST |
| `phase-03-summary.md` | Phase summary |

---

### Phase 4 — Data Schemas & Type Contracts [COMPLETE]

Directory: `04-contracts/`

| Artifact | Schema Coverage |
|---|---|
| `phase-04-layout-profile-schema.md` | `LayoutProfile`, `NormalizedROI`; coordinate system |
| `phase-04-layout-transcript-contract.md` | `LayoutTranscript`, `LayoutPage`, `LayoutSpan`, `QualityMetrics` |
| `phase-04-workflow-types-contract.md` | `InventoryResult`, `CorrectionOverlay`, `ExecuteResult`, `MergePlan` |
| `phase-04-bookmark-types-contract.md` | `BookmarkNode`, `BookmarkTree`, `BookmarkAnchorTree` |
| `phase-04-ipc-envelope-contract.md` | `IpcResponse<T>` and structured error shape |
| `phase-04-output-formats-contract.md` | Merge report, inventory, detection result JSON formats |
| `phase-04-audit-bundle-schema.md` | Canonical V4 `AuditBundle` wire contract for explainable artifact bundles |
| `phase-04-standards-types-contract.md` | `DrawingsDisciplineMeta`, `SpecsMasterformatMeta`, `StandardsBasis` |
| `phase-04-summary.md` | Phase summary |

---

### Phase 5 — Incomplete Work & Known Limitations [COMPLETE]

Directory: `05-gaps-and-limitations/`

| Artifact | Coverage |
|---|---|
| `phase-05-unimplemented-features.md` | GUI placeholders, planned-but-unstarted features (ROI auto-detect, schedule UI, web/SaaS, etc.) |
| `phase-05-failure-modes-catalog.md` | Known failure modes from test output and fixtures |
| `phase-05-technical-debt-register.md` | TD-001 through TD-007 with P1/P2/P3 prioritization |
| `phase-05-summary.md` | Phase summary |

---

### Phase 6 — Post-Mortem Write-Up [COMPLETE]

Directory: `06-lessons/`

| Artifact | Coverage |
|---|---|
| `phase-06-architecture-overview.md` | Module map, dependency rules, data flow diagrams, implementation status table |
| `phase-06-workflow-breakdown-merge.md` | Merge workflow (drawings + specs lanes, multi-addendum, narrative validation) |
| `phase-06-workflow-breakdown-split.md` | Split workflow (discipline/division grouping, footer boundary detection) |
| `phase-06-workflow-breakdown-bookmarks.md` | Bookmarks workflow (Footer-First Anchoring, QPDF sidecar, correction types) |
| `phase-06-workflow-breakdown-specs-patch.md` | Specs Patch workflow (8-stage AST pipeline, patch operations, BookmarkAnchorTree) |
| `phase-06-lessons-learned.md` | What worked (7), what failed (5), what was deferred (7) |
| `phase-06-edge-case-catalog.md` | 23 edge cases across 7 modules |
| `phase-06-non-negotiables-rust-constraints.md` | All 20 V4 non-negotiables reframed as Rust requirements with P0/P1/P2 priority |
| `phase-06-summary.md` | Phase summary |

---

### Phase 7 — Reference Library Assembly & Rust Handoff [COMPLETE]

Directory: `07-rust-handoff/`

**Entry point for V4 Rust developers**: Start with `phase-07-rust-port-primer.md`.

| Artifact | Coverage |
|---|---|
| `phase-07-rust-port-primer.md` | V4 crate structure, external dependencies, three-phase workflow pattern, coordinate system, determinism requirements, ADR cross-references, explicit GUI deferrals, summary of what to port vs. replace |
| `phase-07-ts-to-rust-module-map.md` | Complete flat mapping of every TS source file to its Rust equivalent (port / replace / do not port), including post-verification addenda |
| `phase-07-sidecar-replacement-plan.md` | How to replace PyMuPDF sidecar with pdfium-render; how to replace pikepdf/QPDF sidecar with lopdf; disk-streaming merge architecture |
| `phase-07-dataset-portability-matrix.md` | Classification of every dataset, regex, and schema as verbatim-port / adapted-port / algorithmic-re-expression / do-not-port |
| `phase-07-summary.md` | Phase summary |

---

### Phase 8 — Verification [COMPLETE]

Directory: `08-verification/`

| Artifact | Description |
|---|---|
| `phase-08-coverage-checklist.md` | Module/accounting matrix, ADR table, Phase 5 resolution table, and gap register |
| `phase-08-validation-report.md` | Validation findings, verdicts, and resolution plan for remaining addenda |
| `phase-08-summary.md` | Phase summary |

---

### Phase 9 — Documentation Drift, Ops & Risk Controls [COMPLETE]

Directory: `09-ops-and-drift/`

| Artifact | Description |
|---|---|
| `phase-09-doc-drift-matrix.md` | "Doc claim vs. code reality" matrix and migration-gate truth table |
| `phase-09-deprecation-gates.md` | Deprecation controls, defaults, failure messaging, and Rust feature-toggle implications |
| `phase-09-ops-telemetry-lessons.md` | Logging, export-bundle, supportability, and telemetry lessons from the GUI prototype |
| `phase-09-ux-critical-path.md` | Workflow-critical manual paths, IPC envelope rules, ROI lifecycle, and acceptance criteria |
| `phase-09-roi-coordinate-spec.md` | Canonical coordinate-space reference and regression guardrails |
| `phase-09-summary.md` | Phase summary |

---

## Residual Open Items

The post-mortem documentation coverage is complete. Remaining open items in `00-admin/open-questions-and-risks.md` are substantive product or implementation risks rather than missing extraction artifacts:

- final V4 GUI runtime/cutover decision
- specs footer map completion and all-division footer parser coverage
- schedule extraction completeness on complex layouts
- external repo-doc cleanup identified by the Phase 9 drift matrix

---

## Naming Rules

- Phase summary: `phase-XX-summary.md`
- Topic docs: `phase-XX-<topic-kebab>.md`
- Data snapshots: `phase-XX-<artifact-name>.json`

## Handoff Rule

Each phase agent must:
1. Read `00-admin/phase-manifest.md` and prior phase summary before starting
2. Write current phase summary before exiting
3. Update `00-admin/phase-manifest.md` with completion status
4. Move any unresolved risks to `00-admin/open-questions-and-risks.md`
