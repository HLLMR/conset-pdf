# Gap Register

**Version:** 2.2.0
**Date:** April 6, 2026
**Owner:** HLLMR LLC
**Status:** ACTIVE
**Doc Status Tag:** Implemented

## Scope

Enumeration of all known code-level gaps: stubs, no-ops, missing wiring, and trivial tests. Each entry is compared against its canonical doc claim. This register is the primary output of Phase E and the input to Phase F reconciliation.

Gap status:
- `Open` — known gap, not yet closed by implementation evidence
- `Accepted` — deliberate scope deferral with documented rationale
- `Closed` — closed by implementation evidence (reference to test or commit)

---

## Gap Inventory

| Gap ID | Crate | File | Item | Gap Type | Severity | Canonical Claim | Status |
|---|---|---|---|---|---|---|---|
| G-001 | engine | `crates/engine/src/pipeline/extraction.rs` | `Extractor::extract()` | PARTIAL | CRITICAL | `ARCHITECTURE.md` — Compiler pipeline: Lexer stage converts raw PDF input to LayoutTranscript | **Closed** — Band 0 rewrote `crates/engine/src/pipeline/extraction.rs` with real PDFium extraction loop; `PdfiumExtractor::try_new()` added; per-page `SpanData`→`BoundingBox`→`normalize_bbox()`→`BBox`→`Span` conversion wired. Evidence: `crates/engine/src/pipeline/extraction.rs`. |
| G-002 | engine | `crates/engine/src/pipeline/parsing.rs` | `Processor::process()` | NOOP | CRITICAL | `ARCHITECTURE.md` — Parser/Optimizer stages normalize, validate, and enrich transcript | **Closed** — Band 0 wired `validate_transcript()` call into `crates/engine/src/pipeline/parsing.rs`; returns `EngineError::validation` on failure. Evidence: `crates/engine/src/pipeline/parsing.rs`. |
| G-003 | engine | `crates/engine/src/main.rs` | `fn main()` | STUB | HIGH | `ARCHITECTURE.md` — CLI orchestrates full extraction/processing pipeline | **Accepted** — `backend-cli` in `apps/backend-cli/` covers all runtime paths (extract, visualize, segment, visualize-segments, parse, visualize-ast, edit) with full integration test coverage. `crates/engine/src/main.rs` printing version-only is deliberate scope deferral; no behaviour depends on it. No action required. |
| G-004 | pdf-extraction | `crates/pdf-extraction/src/extractor.rs` | `PdfiumExtractor::extract_page()` | STUB | CRITICAL | `TRANSCRIPT_ARCHITECTURE.md` — Extraction backend produces structured PageData with bboxes and text runs | **Closed** — Phase C implemented real pdfium-render calls; returns `PageData` with `Vec<SpanData>`, `RawBBox`, `width_pts`, `height_pts`. Evidence: `crates/pdf-extraction/src/extractor.rs`, `crates/pdf-extraction/src/types.rs`. G-005 (PDF→IR conversion wiring) also closed in Band 0. |
| G-005 | pdf-extraction / engine | `crates/engine/src/pipeline/extraction.rs` | PDF text → IR span conversion | MISSING-WIRING | CRITICAL | `ARCHITECTURE.md` — Pipeline connects pdf-extraction output to IR crate types | **Closed** — `RawBBox`→`BoundingBox`→`normalize_bbox()`→`BBox` and `BoundingBox::new()`→`Span::new()` conversion chain wired in `crates/engine/src/pipeline/extraction.rs`. Same commit as G-001 closure. |
| G-006 | audit | `crates/audit/src/` | Audit hook integration | MISSING-WIRING | MEDIUM | `ARCHITECTURE.md` — Audit events generated during extraction and processing | **Closed** — Phase 4 Task 4.8 confirmed all 7 CLI handlers (`extract`, `visualize`, `segment`, `visualize-segments`, `parse`, `visualize-ast`, `edit`) emit `AuditEventData::OperationStarted` and `AuditEventData::OperationEnded` via `AuditBundle`. Evidence: `apps/backend-cli/src/handlers/edit.rs` and all sibling handlers in `apps/backend-cli/src/handlers/`. |
| G-007 | ir | `crates/ir/src/validation.rs` | `Validator::validate()` | NOOP | MEDIUM | `DEV_STANDARDS.md` — Validation layer enforces all IR invariants at the crate boundary | **Closed** — `Validator::validate()` now delegates to `validate_transcript()` and maps the error to `String`. Evidence: `crates/ir/src/validation.rs`. |
| G-008 | ir | `crates/ir/src/types.rs` | `Document`, `Element` structs | STUB | LOW | `ARCHITECTURE.md` — Document type models structured document content; purpose currently unclear | **Accepted** — `Document` and `Element` stubs have no defined downstream consumer and carry no canonical claim that is currently being violated. Closing as deliberate scope deferral pending a concrete Phase 6+ consumer. No action required. |
| G-009 | engine | `crates/engine/tests/end_to_end_test.rs` | E2E test pipeline invocation | TRIVIAL-TEST | HIGH | `MASTER_PLAN.md` — End-to-end pipeline test validates real PDF → LayoutTranscript path | **Closed** — `#[ignore]` removed from `test_e2e_loads_pdf_successfully` and `test_e2e_extracts_text_from_pdf`; `engine_api_test.rs` rewritten to use real `simple.pdf` fixture. All 6 E2E + 3 API tests pass. Evidence: `crates/engine/tests/end_to_end_test.rs`, `crates/engine/tests/engine_api_test.rs`. |
| G-010 | audit | `crates/audit/src/bundle.rs` | Audit unit tests | TRIVIAL-TEST | LOW | `DEV_STANDARDS.md` — Tests validate behavior including JSON persistence and event ordering | **Closed** — Phase 5 pre-work added 4 behavioral tests to `crates/audit/src/bundle.rs`: `test_json_round_trip_serde` (serializes/deserializes bundle, verifies event count and timestamp), `test_event_ordering_preserved` (3 events added in order, iter yields same order), `test_event_count_tracks_add_and_clear` (count increments and clear resets to 0), `test_iter_yields_all_events` (iter yields events matching added count). All 6 audit unit tests pass. Evidence: `crates/audit/src/bundle.rs`. |
| G-011 | engine / workflows | (detection policy boundary) | Autonomous deterministic ROI candidate generation, ranking, and tie-break implementation | MISSING-WIRING | HIGH | `MASTER_PLAN.md` + `ARCHITECTURE.md` — ROI detection is autonomous-first and deterministic | Open |
| G-012 | apps/desktop-gui + backend-cli | (admin tooling boundary) | Admin-only manual ROI/profile refinement and override manifest flow | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` + `ARCHITECTURE.md` — manual ROI/profile management retained as admin-only refinement/fallback | Open |
| G-013 | contracts + workflows | (intake boundary) | Multi-file intake bundle contract: `IntakeBundle` type, multi-input support in `WorkflowRequest`, `IntakeIssue` artifact type | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #22 — Intake Triage is mandatory pre-Lexer stage | Open |
| G-014 | workflows | `crates/workflows/src/merge_addenda.rs` | Batch addenda assembly and deterministic ordering logic (declared-sequence primary, filename-inferred fallback, conflict detection) | STUB | HIGH | `MASTER_PLAN.md` — Intake Triage assembles multi-file bundles with deterministic addenda ordering | Open |
| G-015 | engine / workflows | (furniture patterns boundary) | Autonomous document-type classification from furniture patterns: page-range → medium assignment manifest, advisory output only | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiables #18 + #22 — explicit-invoke classification produces advisory manifest before any destructive split; Phase 2-gated | Open |
| G-016 | engine / workflows | (intake boundary) | Page rotation detection and normalization: PDFium read of page-dict metadata, `lopdf` write for `/Rotate` correction, per-page normalization manifest | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #23 — `lopdf` is canonical PDF write backend for page-level ops including rotation normalization | Open |
| G-017 | workflows | `crates/workflows/src/merge_addenda.rs` | Cross-medium addenda merge workflow (single addendum package revises both drawing sheets and spec sections) | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` — integrated manuals and combined addenda may contain mixed mediums; Phase 6–7 gated | Open |
| G-018 | engine / workflows | (drawing detection boundary) | Deterministic drawing-frame and corner-band detector for title-block candidate generation (vector geometry pass). **Phase 0.5 note:** `detect_title_block_anchor()` in `tools/pattern_dev.rs` implements keyword-label density corner scoring (Phase G bonus); `TitleBlockSidecar` schema pre-seeds 4 corner candidates (BR/BL/TR/TL). Full shared-engine frame/corner detector remains Open for Band 1. | MISSING-WIRING | HIGH | `MASTER_PLAN.md` + `ARCHITECTURE.md` — vector-first deterministic title-block localization is baseline | Open |
| G-019 | engine / workflows | (drawing detection boundary) | Title-block table-structure scoring (axis-aligned lines/intersections/rectangles) and deterministic tie-break. **Phase 0.5 note:** `CornerBandCandidate` fields `axis_line_count`, `cell_density`, `score` defined in `tools/src/pattern_model.rs`; runtime population of these fields deferred to Band 1. | MISSING-WIRING | HIGH | `MASTER_PLAN.md` + `ARCHITECTURE.md` — title-block ROI chosen by deterministic geometry score with diagnostics | Open |
| G-020 | engine / workflows | (field extraction boundary) | Deterministic sheet number/title extraction inside title-block ROI using pattern + typography + relative-position scoring | MISSING-WIRING | HIGH | `MASTER_PLAN.md` + `ARCHITECTURE.md` — sheet fields must be extracted from ROI with auditable scoring evidence | Open |
| G-021 | contracts + workflows | (template boundary) | Auto-learned firm/layout template store and schema (`status`, `signature`, relative field bboxes, drift metadata) | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #25 — templates are auto-learned internal accelerators | Open |
| G-022 | engine | `crates/engine/src/parse.rs` | Deterministic vector-text spec heading detector (line clustering, font/caps/indent/spacing heuristics) | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` + `ARCHITECTURE.md` — vector-first deterministic spec heading segmentation | **Closed** — Phase 3 implemented full CSI outline parser in `crates/engine/src/parse.rs`: y-proximity line clustering (LINE_Y_EPSILON=0.012), span x-sort, PART/Article/Paragraph/Sub-paragraph regexes with uppercase-title and major≥1 guards, continuation-text folding, `inject_missing_parts` recovery pass. Post-hardening: 7,971 nodes, 0.2% unclassified rate, 0/70 wrong-PART sections. Evidence: `crates/engine/src/parse.rs`, `apps/backend-cli/tests/cli_integration_test.rs` (19/19 pass). |
| G-023 | workflows + contracts | (fallback boundary) | Explicit opt-in AI fallback contract (low-confidence gate, cropped-region payload manifest, audit markers) | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #26 — AI fallback must be explicit, bounded, and non-baseline | Open |
| G-024 | engine / workflows | (assist boundary) | Local micro-ML runtime integration (on-device inference path, model registry/version pin, deterministic preprocessing) | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #27 — local micro-ML is assistive, deterministic-bounded, version-locked | Open |
| G-025 | engine | (confidence boundary) | Deterministic confidence fusion policy (baseline + micro-ML score fusion with auditable branch reason) | MISSING-WIRING | MEDIUM | `ARCHITECTURE.md` — Local Micro-ML Confidence Assist Architecture requires explicit deterministic fusion and provenance | Open |
| G-026 | contracts + workflows | (power-user API boundary) | Power-user LLM validation/instruction API contracts (request/response schema, advisory status, promotion to executable manifest) | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #28 — power-user LLM integration is validation-only unless explicitly authorized | Open |
| G-027 | pdf-extraction / workflows | (ocr boundary) | Raster/low-text page detection and OCR extraction path with source/confidence tagging (`ocr` vs `vector`) | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #29 — raster PDFs require first-class OCR path | Open |
| G-028 | engine / workflows | (table boundary) | Schedule/table parser to canonical schema with page/region provenance and confidence per extracted record | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #30 — schedule parsing must emit schema-versioned structured outputs | Open |
| G-029 | workflows / contracts | (export boundary) | Schema-versioned schedule export adapters for JSON/CSV/XML with stable field mapping contract | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #30 — machine-consumable export formats JSON/CSV/XML required | Open |
| G-030 | contracts + workflows | (correction boundary) | Replayable correction manifest schema and reapply engine with scope guards and divergence reporting | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #31 — replayable correction manifests are first-class artifacts | Open |
| G-031 | workflows | (compare boundary) | Native diff/compare report workflows for spec sections, drawing sheets, and packet revisions | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #38 — native diff and exception triage are core workflows | Open |
| G-032 | contracts + apps/desktop-gui | (review boundary) | Provenance-first review object model and UI contract (field -> source evidence, branch reason, correction history) | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #32 — review is provenance-first | Open |
| G-033 | contracts + workflows | (privacy boundary) | Redaction/privacy policy model and outbound payload manifest for external API paths | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #33 — external API paths require redaction and outbound payload control | Open |
| G-034 | workflows | (orchestration boundary) | Batch orchestration, resumable job state, retry, and partial completion accounting | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #34 — batch orchestration and resumability are first-class | Open |
| G-035 | contracts + workflows | (dsl boundary) | Typed instruction DSL / manifest language with parser, validator, dry-run, and execution bridge | MISSING-WIRING | HIGH | `MASTER_PLAN.md` Non-Negotiable #35 — instruction DSL is the canonical automation contract | Open |
| G-036 | contracts + apps/desktop-gui | (policy boundary) | Confidence calibration framework and user-facing policy profiles | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #36 — confidence is exposed through policy profiles | Open |
| G-037 | standards-data + workflows | (normalization boundary) | Standards normalization wiring to canonical UDS/NCS/MasterFormat scaffold with raw-to-canonical mapping evidence | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #37 — normalization builds on existing canonical standards scaffold | Open |
| G-038 | engine / workflows | (entity boundary) | Cross-document entity resolution across sheets, sections, equipment tags, firms, and revisions | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiable #39 — cross-document entity resolution is strategic infrastructure | Open |
| G-039 | workflows / apps/desktop-gui | (knowledge boundary) | Project knowledge index, searchable lookup surface, exception triage queue, and drift metrics dashboard contracts | MISSING-WIRING | MEDIUM | `MASTER_PLAN.md` Non-Negotiables #38 + #40 — exception triage and project knowledge indexing are supported surfaces | Open |

---

## Severity Definitions

| Severity | Meaning |
|---|---|
| CRITICAL | Blocks any real end-to-end extraction; the pipeline cannot produce output without this |
| HIGH | Required for Phase 0 completion per `MASTER_PLAN.md` acceptance criteria |
| MEDIUM | Required for correctness or auditability guarantees stated in canonical docs |
| LOW | Maintenance debt or minor capability gap; does not block pipeline |

---

## Gap Type Definitions

| Type | Meaning |
|---|---|
| STUB | Function exists; returns a dummy/empty/hardcoded value |
| NOOP | Function exists; body is an identity function or always-Ok return |
| MISSING-WIRING | Both source and target components exist; integration layer is absent |
| PARTIAL | Function is partially implemented with placeholder comments |
| TRIVIAL-TEST | Test exists but validates only trivial construction, not behavior |

---

## Critical Path

The minimum gap closure sequence for a functional end-to-end pipeline:

```
G-004 (extract_page stub)                       ← CLOSED Phase C
  → G-005 (PDF text → IR conversion wiring)     ← CLOSED Band 0
    → G-001 (Extractor::extract() real impl)    ← CLOSED Band 0
      → G-002 (Processor::process() validation) ← CLOSED Band 0
        → G-003 (main.rs CLI orchestration)     ← Open (backend-cli app covers runtime path)
          → G-009 (E2E test with real PDF)      ← CLOSED Band 0
```

G-006 (audit hooks), G-007 (Validator — **CLOSED Band 0**), G-008 (Document/Element), and G-010 (audit tests) are parallel work, not on the critical path.

Autonomous ROI first-class readiness adds a secondary policy track:

```
G-011 (autonomous deterministic ROI policy implementation)
  → G-012 (admin-only override/refinement pathway)
```

Intake Triage normalization adds a third track (parallel to the critical path, gates on Stage 0 before any extraction):

```
G-013 (IntakeBundle contract + multi-input WorkflowRequest)
  → G-014 (batch addenda assembly + deterministic ordering)
    → G-016 (rotation detection + lopdf normalization)
      → G-015 (explicit-invoke classification manifest — Phase 2-gated)
        → G-017 (cross-medium addenda merge — Phase 6–7 gated)
```

Vector-first deterministic adaptive detection adds a fourth track:

```
G-018 (drawing frame + corner-band candidates)
  → G-019 (table-structure title-block score + tie-break)
    → G-020 (sheet field extraction score model)
      → G-021 (auto-learned firm/layout template store + drift checks)
        → G-023 (explicit AI fallback contract, optional only)

G-022 (spec heading detector) runs in parallel and feeds section segmentation quality.
```

Assisted intelligence, OCR, and schedule-data extraction add a fifth track:

```
G-027 (raster/low-text detection + OCR source tagging)
  → G-028 (schedule parser canonical schema + provenance)
    → G-029 (JSON/CSV/XML export adapters + schema versioning)

G-024 (local micro-ML runtime) + G-025 (deterministic confidence fusion)
  → G-023 (optional AI fallback contract)

G-026 (power-user LLM validation/instruction API contracts) runs parallel and integrates at workflow boundary.
```

Operational trust, automation, and knowledge layer add a sixth track:

```
G-030 (replayable correction manifests)
  → G-035 (instruction DSL / validation / execution bridge)
    → G-031 (native diff/compare reports)
      → G-032 (provenance-first review contract)
        → G-039 (exception triage queue + project knowledge index + drift metrics)

G-033 (redaction/privacy controls), G-034 (batch orchestration/resume), G-036 (confidence policy profiles), G-037 (standards normalization wiring), and G-038 (cross-document entity resolution) run in parallel and feed the same operational layer.
```

---

## Acceptance Criteria (G-030 to G-039)

The gaps below can only move to `Closed` when all listed acceptance criteria are met and linked evidence is recorded (tests, artifacts, and contract docs as applicable).

### G-030 — Replayable correction manifest + reapply engine
1. Typed correction manifest schema exists with target identity, scope, and replay guard fields.
2. Reapply command supports dry-run and emits divergence report when targets changed.
3. At least one deterministic integration test proves: apply correction, replay on unchanged input, same output.
4. Audit artifact records correction source, operator, timestamp, and replay result.

### G-031 — Native diff/compare workflows
1. Structured compare payload exists for specs, drawing sheets, and packet-level revisions.
2. Compare output supports at minimum: added/removed/changed/renamed classification.
3. Golden tests cover representative add/remove/rename edge cases.
4. Compare artifacts are machine-readable and include evidence refs, not only human text.

### G-032 — Provenance-first review contract
1. Review object schema maps each field/decision to page, region, method, confidence, and branch reason.
2. Correction history links are supported (pre-correction and post-correction evidence references).
3. Contract validation rejects review records missing required provenance fields.
4. At least one end-to-end fixture emits review objects consumed by a downstream UI-safe payload.

### G-033 — Redaction/privacy + outbound payload manifest
1. Policy schema defines permitted outbound scopes (none/region/page/document) and redaction modes.
2. Outbound payload manifest is generated for every external API invocation.
3. Tests prove restricted modes block disallowed payload scopes.
4. Audit events capture provider, model/service, payload manifest ID, and policy mode used.

### G-034 — Batch orchestration + resumable state
1. Job state model exists with job/file/page granularity and explicit terminal/non-terminal states.
2. Resume operation can continue from checkpoint without duplicating already-completed mutations.
3. Retry behavior is deterministic and bounded by policy-configured limits.
4. Integration test covers interruption + resume + consistent final state.

### G-035 — Typed instruction DSL + validation bridge
1. DSL grammar/schema is published with strict typing and version identifier.
2. Parser and validator reject ambiguous or unsafe instructions with actionable diagnostics.
3. Dry-run output includes resolved targets, planned actions, and policy checks.
4. Execution bridge only accepts validated instruction manifests.

### G-036 — Confidence calibration + policy profiles
1. Policy profile schema exists (for example conservative/balanced/aggressive review behavior).
2. Runtime outputs include profile ID and thresholds used per decision set.
3. Tests prove profile changes alter escalation/review routing deterministically.
4. Profile defaults and override rules are documented and validated at load time.

### G-037 — Standards normalization wiring on existing scaffold
1. Runtime normalization consumes canonical references from `AEC_STANDARDS.md` and `MASTERFORMAT_REFERENCE.md`.
2. Mapping record stores both raw observed value and canonical normalized value.
3. Mapping/version metadata is emitted in outputs and audit artifacts.
4. Regression fixtures prove no parallel mapping source is used when canonical source exists.

### G-038 — Cross-document entity resolution
1. Entity-link record schema exists with basis, confidence, and merge history.
2. Resolver handles at minimum sheets, sections, equipment tags, firms, and revision variants.
3. Tests cover positive link, ambiguous link, and no-link outcomes.
4. Ambiguous links are routed to exception payloads rather than silently auto-merged.

### G-039 — Project knowledge index + triage queue + drift metrics
1. Search/index record schema exists and references canonical entities plus provenance.
2. Exception queue payload schema exists with reason code, severity, and recommended action.
3. Drift metrics payload includes at minimum confidence drift, fallback rate, and unresolved exception counts.
4. At least one integration fixture proves records are indexable and traceable back to source evidence.

---

## Revision History

| Version | Date | Change |
|---|---|---|
| 1.0.0 | 2026-03-22 | Initial gap enumeration from Phase E code audit |
| 1.1.0 | 2026-03-23 | Added G-011/G-012 for autonomous ROI first-class strategy and admin-only manual override path. |
| 1.2.0 | 2026-03-23 | Added G-013–G-017 for multi-file ingestion, Intake Triage Stage 0, lopdf rotation normalization, autonomous classification, and cross-medium addenda merge. |
| 1.3.0 | 2026-03-23 | Added G-018–G-023 for vector-first deterministic title-block/spec heading detection, auto-learned templates, and explicit optional AI fallback contract. |
| 1.4.0 | 2026-03-23 | Added G-024–G-029 for local micro-ML confidence assist, power-user LLM validation API, raster OCR path, schedule parser canonical schema, and JSON/CSV/XML export adapters. |
| 1.5.0 | 2026-03-23 | Added G-030–G-039 for replayable corrections, native diff/compare, provenance-first review, privacy/redaction, batch orchestration, instruction DSL, confidence policy profiles, standards normalization wiring, cross-document entity resolution, and project knowledge indexing/triage. |
| 1.6.0 | 2026-03-23 | Added explicit acceptance criteria for G-030 through G-039 to define closure evidence and prevent ambiguous completion claims. |
| 1.7.0 | 2026-04-04 | Closed G-004 (`PdfiumExtractor::extract_page()` implemented with real `PageData`). Phase 0.5 Phase C complete. |
| 1.8.0 | 2026-04-05 | Added Phase 0.5 implementation notes to G-018 and G-019 (partial runtime detector in `tools/pattern_dev.rs::detect_title_block_anchor()`; `TitleBlockSidecar` / `CornerBandCandidate` schemas in place). Both gaps remain Open for shared-engine Band 1 implementation. |
