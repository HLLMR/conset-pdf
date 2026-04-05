## Plan: Phase 0.5 Pattern Development and Contract-Shaping Toolchain

Build Phase 0.5 as a developer-first tooling and contract-shaping phase. The core deliverable remains a local pattern-development CLI inside the existing `tools` crate, but the phase now also defines the contract surfaces that later phases will consume for intake normalization, autonomous ROI detection, vector-first title-block/spec heuristics, assisted intelligence, OCR routing, structured schedule outputs, replayable corrections, instruction manifests, provenance-first review, privacy controls, orchestration, standards normalization, entity resolution, and searchable knowledge surfaces.

Phase 0.5 must stay deliberately bounded:
- implement the pattern-dev tool and the artifacts needed to make later detection work concrete and testable;
- define future-facing contracts where downstream phases would otherwise incur breaking changes;
- avoid silently pulling Phase 1+ runtime implementation into the tool just because the contracts are now known.

Treat G-001/G-002/G-004/G-005 as explicit out-of-scope shared-engine dependencies unless a tiny helper can be reused without widening Phase 0.5 scope.

## Integrated Objective

By the end of Phase 0.5, the repo should have one coherent developer toolchain that can inspect raw PDFs, apply deterministic pattern and ROI logic, emit overlays and rich sidecars, validate behavior against the corpus, and publish the typed artifacts/contracts that later phases will use for runtime execution.

This phase is successful only if it does both:
1. shortens the feedback loop for pattern and heuristic development right now;
2. prevents downstream architectural churn by locking the contract shapes for the features now committed in planning.

## In-Scope Outcomes

### Runtime Tooling Outcomes
1. Local PDF inspection path for raw vector PDFs.
2. Deterministic single-PDF pattern test command.
3. Deterministic overlays and per-page sidecars.
4. Corpus validation command with stable fixture selection and stable artifact naming.
5. Rich decision evidence for ROI, title-block, heading, and schedule-oriented pattern work.

### Contract-Shaping Outcomes
1. ROI decision sidecar schema.
2. Intake Triage contract draft (`NormalizedIntakeBundle`, `IntakeIssue`, addenda ordering, advisory classification manifest).
3. Vector-first drawing/spec detection schemas (title-block localization, field scoring, template lifecycle).
4. Assisted intelligence contracts (local micro-ML decision record, power-user LLM validation/instruction schemas, OCR routing/source tagging).
5. Structured schedule schema draft and export mapping requirements (JSON/CSV/XML).
6. Operational trust contracts (replayable corrections, typed instruction DSL, provenance-first review objects, diff/exception payloads, redaction policy, resumable job state, confidence policy profiles, normalization/entity/index schemas).

## Explicit Non-Goals

1. Completing the shared engine extraction chain.
2. Shipping full Intake Triage execution in runtime workflows.
3. Shipping production OCR runtime integration.
4. Shipping production micro-ML or cloud LLM execution paths.
5. Shipping complete enterprise queueing, search UI, or knowledge-browser UX.
6. Reintroducing manual ROI/profile authoring as a primary end-user workflow.
7. Rewriting the canonical standards docs; Phase 0.5 only defines how implementation will consume them.

## Deliverables

### Tool Deliverables
1. `pattern-dev` binary hosted safely beside `classify-pdf`.
2. Single-PDF test command for deterministic pattern/ROI/title-block heuristics.
3. Batch validation command against explicit Tier 1/Tier 2 fixtures.
4. Stable overlay and sidecar artifact set.

### Artifact Deliverables
1. Per-page pattern result sidecar.
2. Per-page ROI decision sidecar with ranking evidence.
3. Deterministic failure reason codes for no-match and low-confidence cases.
4. Validation summary manifest and corpus aggregate report.

### Contract Deliverables
1. Intake contract draft: `NormalizedIntakeBundle`, `IntakeIssue`, addenda ordering, advisory classification.
2. Vector-first detection drafts: title-block candidate score schema, field score schema, template lifecycle schema.
3. Assisted intelligence drafts: micro-ML fusion record, LLM validation request/response, OCR routing/source tags.
4. Structured data drafts: canonical schedule schema plus export mapping expectations.
5. Operational trust drafts: correction manifest, instruction DSL, review object, diff/exception schema, redaction policy, job state, confidence profile, normalization/entity/index records.

## Implementation Priority Stack

The feature surface is now intentionally larger than Phase 0.5 runtime scope. To prevent sprawl, every committed feature belongs to one of the priority bands below.

### Priority Band 0 — Must Be Built in Phase 0.5 Runtime

These are the runtime outputs that justify Phase 0.5 existing at all:
1. `pattern-dev` binary and multi-binary `tools` crate structure.
2. Local raw-PDF inspection path for vector PDFs.
3. Deterministic single-PDF pattern and heuristic test loop.
4. Deterministic overlays, sidecars, and failure diagnostics.
5. Corpus validation command and repeated-run determinism checks.

### Priority Band 0.5 — Must Be Defined in Phase 0.5, But Not Fully Implemented

These features are allowed to consume design effort in Phase 0.5 only at the contract/artifact layer:
1. Intake Triage contracts (`NormalizedIntakeBundle`, `IntakeIssue`, addenda ordering, advisory classification).
2. Autonomous ROI sidecar and ranking evidence schemas.
3. Vector-first drawing/spec detection schemas (title-block candidates, field score, template lifecycle, heading diagnostics).
4. Assisted intelligence schemas (micro-ML decision record, power-user LLM validation/instruction contracts, OCR routing manifest).
5. Structured schedule schema and JSON/CSV/XML export mapping drafts.
6. Operational trust schemas (correction manifest, instruction DSL skeleton, review/diff/exception payloads, privacy/redaction policy, resumable job-state contract, confidence profiles, normalization/entity/index drafts).

### Priority Band 1 — First Runtime Layer After Phase 0.5

These are the immediate post-0.5 implementation priorities because later features depend on them:
1. Shared engine extraction chain: G-004 → G-005 → G-001 → G-002 → G-003 → G-009.
2. Stage 0 Intake Triage runtime: G-013, G-014, G-016.
3. Autonomous ROI runtime path: G-011, with G-012 retained as admin-only fallback.
4. Vector-first drawing/spec detection runtime: G-018 through G-022.

### Priority Band 2 — Structured Runtime Expansion

These should land only after Band 1 is producing stable runtime outputs:
1. OCR runtime path: G-027.
2. Schedule parser canonical schema and exports: G-028, G-029.
3. Local micro-ML assist path and deterministic confidence fusion: G-024, G-025.
4. Standards normalization wiring on the existing scaffold: G-037.

### Priority Band 3 — Operational Trust Layer

These are the next moat features once runtime extraction and structured outputs are stable:
1. Replayable corrections: G-030.
2. Native diff/compare workflows: G-031.
3. Provenance-first review objects: G-032.
4. Privacy/redaction controls: G-033.
5. Batch orchestration/resume: G-034.
6. Typed instruction DSL and dry-run execution bridge: G-035.
7. Confidence policy profiles: G-036.

### Priority Band 4 — Intelligence and Project-System Layer

These are strategic but should not be allowed to destabilize core runtime layers:
1. Power-user LLM validation/instruction API runtime: G-026.
2. Cross-document entity resolution: G-038.
3. Project knowledge index, triage queue, and drift dashboard contracts/runtime: G-039.

### Priority Rules

1. If a task does not directly support Priority Band 0 runtime or Priority Band 0.5 contract-shaping, it does not belong inside Phase 0.5 execution.
2. No Priority Band 2-4 runtime work may be smuggled into Phase 0.5 under the label of “just a helper.”
3. Any Phase 0.5 artifact must explicitly say whether it is runtime-ready or contract-only.
4. When in doubt, lock the schema now and defer the runtime implementation.

## Integrated Step Sequence

1. Phase A — Lock scope, command surface, and artifact families.
Define the exact Phase 0.5 command set, artifact directories, sidecar families, contract categories, and priority-band assignment before any restructuring. Output of this step is a single accepted inventory of what Phase 0.5 will implement versus only define.

**Locked command surface (Phase A output):**
- `pattern-dev inspect <PDF> [--page <N>]` — dump per-page geometry summary: page count, dimensions, span count, text quality score, raster-vs-vector flag.
- `pattern-dev test-pattern <PDF> --family <FAMILY> --output-dir <DIR> [--dry-run]` — apply one heuristic family to one PDF, print per-page match/no-match with confidence, write overlays and sidecars.
- `pattern-dev validate-corpus --tier 1 [--tier 2] --output-dir <DIR> [--dry-run]` — batch validation over explicit fixture tiers (never holdout), emits per-fixture artifacts plus aggregate validation manifest.

**Locked heuristic families:**
- `footer-section-id` — regex-based spec footer section-ID detection (runtime, Phase E)
- `page-counter` — footer page-in-section counter (`Page N of M`) detection (runtime, Phase E)
- `header-band` — top-band text/logo presence heuristic (runtime, Phase E)
- `title-block-anchor` — drawing corner-band candidate generation (schema-complete in Phase G; runtime in Phase 1)
- `roi-candidate` — autonomous ROI ranking evidence (schema-complete in Phase G; runtime in Phase 1)
- `spec-heading` — spec heading line-feature diagnostics (schema-complete in Phase G; runtime in Phase 1)

**Locked artifact naming (stable from Phase F forward):**
- Overlay PNG: `<output-dir>/<pdf-stem>/page-<NNNN>-<family>.png` (zero-padded 4-digit page index)
- Sidecar JSON: `<output-dir>/<pdf-stem>/page-<NNNN>-<family>.json`
- Validation manifest: `<output-dir>/validation-manifest.json`
- Corpus aggregate: `<output-dir>/corpus-report.json`

2. Phase B — Rework the `tools` crate into a stable multi-binary home. Depends on 1.
Convert the current single-binary package into a shape that hosts both `classify-pdf` and `pattern-dev` without breaking existing usage. Reuse bootstrap conventions from `classify_pdf.rs`, and introduce shared helpers only where they are already clearly stable.

**File targets:**
- `tools/Cargo.toml` — add `[[bin]] name = "pattern-dev" path = "pattern_dev.rs"`; add deps `image`, `imageproc`, `regex` (dev-tool-only; must not propagate to engine crates).
- `tools/pattern_dev.rs` — new entry point using clap derive with `inspect`, `test-pattern`, and `validate-corpus` subcommands; same pdfium bootstrap pattern as `classify_pdf.rs`.
- Shared helpers (if any) go into a `tools/src/` directory and are gated behind `pub(crate)` visibility.

**Build gate:** `cargo build -p classify-pdf` and `cargo build -p pattern-dev` must both succeed, and `cargo test -p classify-pdf` must stay green before this phase closes.

**Status: COMPLETE** — April 4, 2026. Both binaries build; shared `tools/src/` module directory in place; 9/9 unit tests passing; `cargo test -p classify-pdf` unaffected.

3. Phase C — Build the local read-only PDF inspection primitive. Depends on 2.
Implement page loading, rasterization, geometry sampling, and the smallest useful text-access layer needed for pattern work. Support raw vector-PDF inspection first. Add enough hooks to label raster/low-text pages for future OCR routing, but do not implement production OCR runtime in this phase.

**Implementation note — targeted pdfium-render fix:** The inspection primitive calls `PdfiumExtractor::extract_page()` from `crates/pdf-extraction`. That method currently returns empty `PageData` (G-004 open). Phase C includes implementing real pdfium-render API calls in `crates/pdf-extraction/src/extractor.rs` to return actual bboxes and text runs. This is a bounded fix to unblock the pattern-dev tool as its primary consumer; it does not close G-001/G-002/G-005 and must not widen into engine refactoring. The repair is complete when `pattern-dev inspect` can dump real span counts and page dimensions from a Tier 1 fixture.

**Status: COMPLETE** — April 4, 2026. `PdfiumExtractor::extract_page()` implemented in `crates/pdf-extraction/src/extractor.rs`; returns `PageData` with `Vec<SpanData>`, real bboxes, font name/size, and page dimensions. G-004 closed. **Form XObject note:** AEC spec templates place running headers/footers inside PDF Form XObjects. `page.objects().iter()` is opaque to Form XObjects; see Phase E note and D-028 for the `page.text().chars()` resolution.

4. Phase D — Define deterministic pattern and heuristic model primitives. Depends on 3.
Add the internal policy model for regex patterns, region targeting, confidence thresholds, deterministic ordering, failure codes, and tie-break rules. This phase now also defines the score inputs needed for:
- autonomous ROI ranking,
- title-block candidate selection,
- sheet-number/title extraction,
- spec heading detection,
- future schedule/table heuristic work.

**Locked data model (Phase D output, in `tools/src/pattern_model.rs`):**
```rust
pub struct PatternSpec {
    pub family: HeuristicFamily,
    pub regex: Option<Regex>,
    pub region_band: Option<RegionBand>,  // y_min..y_max in [0.0, 1.0]
    pub confidence_threshold: f32,
    pub version: &'static str,
}

pub enum HeuristicFamily {
    FooterSectionId,
    PageCounter,
    HeaderBand,
    TitleBlockAnchor,  // schema-complete Phase G; runtime Phase 1
    RoiCandidate,      // schema-complete Phase G; runtime Phase 1
    SpecHeading,       // schema-complete Phase G; runtime Phase 1
}

pub struct MatchEvidence {
    pub page_index: usize,
    pub matched_text: Option<String>,
    pub bbox: Option<NormalizedBBox>,
    pub confidence: f32,
    pub failure_reason: Option<FailureCode>,
    pub branch_reason: Option<String>,
    pub source: SourceTag,  // Vector | Ocr (Ocr deferred to Phase 1)
}

pub enum FailureCode {
    NoMatch,
    LowConfidence,
    RegionMiss,
    AmbiguousTie,
}
```

Confidence thresholds (from DEV_STANDARDS): `< 0.80` = hard fail/escalate, `0.80–0.95` = output + warning flag, `≥ 0.95` = proceed normally. These map directly to `FailureCode` selection.

**Status: COMPLETE** — April 4, 2026. `tools/src/pattern_model.rs` locked with all required types; serde serialization; 9/9 unit tests; clippy clean with `-D warnings`.

5. Phase E — Implement the single-PDF developer loop. Depends on 4.
Wire a command that loads one PDF, applies one pattern or heuristic family, prints matched/unmatched results, and writes stable artifacts to an output directory. The first milestone must support the kinds of work later phases depend on most: footer section IDs, page counters, header bands, title-block anchors, and ROI candidate evidence.

**Status: COMPLETE** — April 4, 2026. `pattern-dev test-pattern` real detection loop implemented in `tools/pattern_dev.rs`. Detection uses `page.text().chars()` (PDFium `FPDFText_LoadPage` pipeline) which descends transparently into PDF Form XObjects — critical for AEC spec templates where running footers live in Form XObjects invisible to `page.objects().iter()` (see D-028). Per-page sidecar JSON written at schema version `0.5.0`. `validate-corpus --dry-run` fixture inventory working. Validated on `SPEC_RWB_LHHS_ALL_ORG.pdf` (571 pages): **PASS=556, WARN=10, FAIL=5**. All 5 failures confirmed genuine blank/raster insert pages (pages 0, 11, 12, 14, 15). Effective pass rate on text-bearing pages: 97.1%.

6. Phase F — Add deterministic overlays, sidecars, and provenance payloads. Depends on 5.
Render overlay images with fixed naming, scale, palette, sorting, and reason semantics. Emit sidecars that capture page index, matched text, bboxes, confidence, failure reason, branch reason, and source provenance. This is also the step where the review object shape should become concrete enough to feed later provenance-first review UIs.

**Overlay palette (deterministic, not configurable at runtime):**
- Green bbox = matched, confidence ≥ 0.95
- Yellow bbox = matched, confidence 0.80–0.95 (flag)
- Red bbox = failure (NoMatch, LowConfidence, RegionMiss, AmbiguousTie)

**Locked sidecar JSON schema (version `0.5.0`, additive-only evolution from here):**
```json
{
  "schema_version": "0.5.0",
  "pdf_path": "tests/corpus/tier1/SPEC_RWB_LHHS_ALL_ORG.pdf",
  "page_index": 4,
  "family": "footer-section-id",
  "matched_spans": [{"text": "23 82 16", "bbox": {"x": 0.05, "y": 0.94, "width": 0.15, "height": 0.02}}],
  "confidence": 0.97,
  "failure_reason": null,
  "branch_reason": "pattern matched footer band at y=0.94",
  "source": "vector",
  "engine_version": "0.1.0",
  "pattern_version": "0.5.0"
}
```

7. Phase G — Add vector-first detection scaffolding artifacts. Depends on 6.
Extend the tool outputs so they can exercise and document the future runtime heuristics:
- title-block localization candidate sets and winning corner-band score,
- sheet number/title candidate ranking,
- spec heading line-feature diagnostics,
- auto-learned template candidate payloads with drift metadata placeholders.
The goal here is artifact and schema maturity, not full runtime automation.

8. Phase H — Add Intake Triage and downstream contract drafts. Depends on 6.
Define the non-runtime contract surfaces that later phases need to avoid breaking changes. This step is contract-shaping only and must not broaden into full Band 1-4 runtime implementation:
- `NormalizedIntakeBundle`
- `IntakeIssue`
- addenda ordering schema
- advisory classification manifest
- OCR routing manifest
- schedule schema draft
- micro-ML decision record
- LLM validation/instruction request-response contracts
- correction manifest and typed instruction DSL skeleton
- diff/exception/review object payloads
- job-state and confidence-profile payloads
- normalization/entity/index record drafts

9. Phase I — Add corpus validation and determinism reporting. Depends on 7 and 8.
Implement a batch validation command over explicit Tier 1/Tier 2 fixture selections, never holdout. Emit aggregate metrics, stable reports, and repeated-run determinism checks. Validation reports should already be shaped so later drift dashboards and exception queues can consume them without another contract redesign.

10. Phase J — Add tests, dry-run semantics, and debug logging. Depends on 9.
Add unit tests around parsing, score ordering, manifest serialization, and artifact naming. Add at least one golden-path integration test for overlay/sidecar generation and one determinism test proving repeated runs produce byte-stable reports. Validate dry-run surfaces for future instruction/correction manifests even if their runtime execution is deferred.

11. Phase K — Publish downstream handoff documentation. Depends on 10.
Document how later phases should consume the artifacts and contracts from this phase:
- pattern-dev workflow
- overlay/sidecar schema
- ROI/title-block/template evidence payloads
- intake contract draft boundaries
- assisted intelligence/OCR/schedule schema boundaries
- correction/DSL/review/privacy/orchestration/normalization/entity/index draft contracts
This must state clearly which surfaces are runtime-ready versus contract-only, and which priority band each deferred runtime feature belongs to.

12. Phase L — Final verification and closeout. Depends on 11.
Run focused checks and tests for the `tools` crate, verify `classify-pdf` still works, smoke-test representative fixtures, and confirm the Phase 0.5 definition of done: local inspection works, deterministic artifacts exist, validation is repeatable, and downstream contracts are clear enough that later phases can implement against them without reopening Phase 0.5 design questions.

## Phase Dependency Graph

```
B (tools multi-binary restructure)
└── C (PDF inspection primitive — targeted extract_page fix)
    └── D (pattern/heuristic model, data types, failure codes)
        ├── E (single-PDF test-pattern loop — footer-section-id, page-counter, header-band)
        │   └── F (overlays + per-page sidecars + provenance payloads)
        │       ├── G (vector-first scaffolding: title-block, roi-candidate, spec-heading schemas)
        │       │   └── I (corpus validate-corpus command + determinism reporting)
        │       │       └── J (tests, dry-run semantics, debug logging)
        │       │           └── K (downstream handoff docs)
        │       │               └── L (final verification + closeout)
        │       └── H (contract drafts — can overlap with G once F sidecars are stable)
        │           └── I
        └── H (can begin design work once D data model is locked)
```

Critical path: **B → C → D → E → F → G → I → J → K → L**

H (contract drafts) can begin in parallel with E–G once the Phase D data model is locked. H must complete before I because the validation manifest schema must be compatible with the downstream contract shapes.

## Agent Handoff Sequence

1. Agent 1 owns Phases A-B and exits only when the command/artifact inventory is locked and the `tools` crate safely hosts `pattern-dev` without regressing `classify-pdf`.
2. Agent 2 owns Phases C-D and exits with a working local inspection path and a documented deterministic score/failure-code model.
3. Agent 3 owns Phases E-F and exits with the usable single-PDF workflow plus overlays, sidecars, and provenance payloads.
4. Agent 4 owns Phase G and exits with artifact support for ROI/title-block/spec-heading/template development.
5. Agent 5 owns Phase H and exits with the major downstream contract drafts written and accepted.
6. Agent 6 owns Phases I-J and exits with corpus validation, determinism evidence, tests, dry-run semantics, and debug logging.
7. Agent 7 owns Phases K-L and exits with downstream handoff docs, verification evidence, and any final discoverability/index updates.

## Relevant Files

- `f:/Projects/conset-pdf/docs/MASTER_PLAN.md` — authoritative Phase 0.5 goal, deliverables, and policy context
- `f:/Projects/conset-pdf/docs/ARCHITECTURE.md` — determinism, auditability, Stage 0, review/privacy/entity constraints
- `f:/Projects/conset-pdf/docs/DEV_STANDARDS.md` — fail-loud behavior, debug logging requirements, and AI micro-task workflow
- `f:/Projects/conset-pdf/docs/AEC_STANDARDS.md` — canonical AEC standards scaffold to consume, not rewrite
- `f:/Projects/conset-pdf/docs/MASTERFORMAT_REFERENCE.md` — canonical MasterFormat normalization scaffold
- `f:/Projects/conset-pdf/docs/REPO_STRUCTURE.md` — repo boundaries and package layering rules
- `f:/Projects/conset-pdf/docs/current-state/gap-register.md` — explicit shared-engine and contract gaps to keep visible
- `f:/Projects/conset-pdf/tools/Cargo.toml` — developer utility package to extend with `pattern-dev`
- `f:/Projects/conset-pdf/tools/classify_pdf.rs` — reference implementation for clap, PDFium bootstrap, and binary conventions
- `f:/Projects/conset-pdf/tests/corpus/README.md` — fixture tiering rules and holdout prohibition
- `f:/Projects/conset-pdf/crates/contracts/src/lib.rs` — optional destination for future contract stubs if any become concrete in Phase 0.5

## Verification

1. Verify both `classify-pdf` and `pattern-dev` build after the crate restructure.
2. Run the single-PDF pattern workflow on representative spec and drawing fixtures and confirm stable overlays plus sidecars are produced.
3. Confirm sidecars include explicit confidence, failure reason, branch reason, and evidence fields rather than only presentation-oriented output.
4. Run corpus validation against explicit Tier 1 and Tier 2 fixture sets and confirm the holdout set is excluded.
5. Repeat the same commands twice and compare artifact names and report bytes to confirm determinism.
6. Validate that contract-only outputs are clearly marked as draft surfaces and do not imply runtime execution support that does not yet exist.
7. Confirm the downstream handoff docs are sufficient for later agents to implement furniture detection, Intake Triage, assisted intelligence, OCR routing, schedule schema consumers, correction manifests, and review/privacy/entity layers without re-opening Phase 0.5 design assumptions.

## Integrated Decisions

- Included runtime scope: developer CLI, local PDF inspection, deterministic overlays, sidecars, validation suite, logging, and artifact support for ROI/title-block/spec-heading/template work.
- Included design scope: typed contract drafts for Intake Triage, assisted intelligence, OCR, structured schedule outputs, replayable corrections, instruction DSL, provenance review, privacy, orchestration, normalization, entity resolution, and knowledge/triage surfaces.
- Excluded runtime scope: GUI work, production furniture-detection implementation, production OCR runtime, production micro-ML/cloud inference, standards-data implementation completion, and the shared engine extraction critical path.
- Recommended architecture: keep Phase 0.5 primarily local to the `tools` package; allow contract shaping where necessary, but do not smuggle full runtime implementation into the phase.
- Recommended artifact strategy: raster overlays plus machine-readable sidecars first, because they are inspectable, stable for testing, and extensible into future review/diff/queue workflows.
- Recommended rollout strategy: ship the single-PDF developer loop first, then harden validation and contract drafts, then publish handoff docs.
- Priority policy: Phase 0.5 ships Priority Band 0 runtime and Priority Band 0.5 contract-shaping only; all later-band runtime work is explicitly deferred.
- Manual ROI/profile handling policy: admin-only refinement and controlled fallback; autonomous deterministic detection remains the default operating mode.
- Standards normalization policy: consume the existing canonical UDS/NCS/MasterFormat scaffold; do not create parallel mappings in this phase.
- Locked command surfaces: `inspect`, `test-pattern`, `validate-corpus` subcommand signatures are stable from Phase B and must not be renamed or restructured in later phases without an explicit breaking-change decision.
- Phase C pdfium exception: `PdfiumExtractor::extract_page()` in `crates/pdf-extraction` will be implemented as a targeted fix bounded to what `pattern-dev inspect` needs; G-001, G-002, and G-005 (engine integration chain) remain explicitly out of scope and stay open in the gap register.
- Additional `tools/` dependencies: `image`, `imageproc`, `regex` are dev-tool-only and must not be added as workspace-level dependencies or propagate into engine crates.
- Sidecar schema version `0.5.0` is locked from Phase F; field additions are allowed (additive), field renames and removals are breaking changes requiring a version bump and migration note.
- Heuristic families `title-block-anchor`, `roi-candidate`, and `spec-heading` emit schema-complete sidecars in Phase G but have no runtime detection logic until Phase 1; any Phase G sidecar for these families must set `"source": "schema-only"` to signal contract-only status.

## Further Considerations

1. Keep pattern and artifact versioning explicit from the beginning. If multiple heuristic families begin sharing the same output directory, version drift will become painful quickly.
2. Favor stable field names and additive schema evolution in all Phase 0.5 sidecars. Later phases should be able to extend them without breaking the developer loop.
3. If local PDF geometry or text access is weaker than expected, stop and explicitly re-scope rather than hiding Phase 1 engine work inside Phase 0.5.
