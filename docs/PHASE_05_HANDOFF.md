# Phase 0.5 Downstream Handoff

**Version:** 1.0.0
**Date:** April 5, 2026
**Owner:** HLLMR LLC
**Status:** ACTIVE
**Doc Status Tag:** Implemented
**Alignment:** MASTER_PLAN + ARCHITECTURE + DEV_STANDARDS

---

## Purpose

This document is the handoff contract from Phase 0.5 to later phases.
It answers the question: *What did Phase 0.5 actually produce, which surfaces are safe to build on, and what must later phases implement before each capability is production-ready?*

This is a canonical derived document under `MASTER_PLAN.md` per `DOC_GOVERNANCE.md`.

**Audience:** Agents and developers implementing Band 1–4 runtime capabilities.

---

## Table of Contents

1. [What Phase 0.5 Delivered](#1-what-phase-05-delivered)
2. [Developer Workflow: pattern-dev](#2-developer-workflow-pattern-dev)
3. [Overlay and Sidecar Schema](#3-overlay-and-sidecar-schema)
4. [Evidence Payloads: Title-Block, ROI, Spec Heading](#4-evidence-payloads-title-block-roi-spec-heading)
5. [Downstream Contract Boundaries](#5-downstream-contract-boundaries)
6. [Runtime-Ready vs. Contract-Only Surface Map](#6-runtime-ready-vs-contract-only-surface-map)
7. [Priority Band Assignments](#7-priority-band-assignments)
8. [Open Gaps Still Relevant to Downstream Work](#8-open-gaps-still-relevant-to-downstream-work)

---

## 1. What Phase 0.5 Delivered

Phase 0.5 is complete. Its deliverables fall into three categories:

### 1.1 Runtime-ready developer tooling

- `pattern-dev inspect <PDF>` — dumps per-page span geometry, text quality score, and raster/vector flag.
- `pattern-dev test-pattern --family <FAMILY> --output-dir <DIR> <PDF>` — runs one family's detection loop over a single PDF; writes overlay PNGs and sidecar JSONs.
- `pattern-dev validate-corpus --tier 1 [--tier 2] --output-dir <DIR>` — batch validation over explicit tiers; writes per-fixture sidecar tree, `validation-manifest.json`, and `corpus-report.json`.
- All commands support `--dry-run` to validate arguments and fixture inventory without writing files.

### 1.2 Runtime-ready detection families

Three of the six families have working detection that runs today:

| Family | Detection method | Pass rate on Tier 1 corpus (2,892 pages) |
|---|---|---|
| `header-band` | Non-empty span presence in top 15% of page | 98.8% |
| `footer-section-id` | Regex `\b\d{2}\s+\d{2}(?:\s+\d{2})?\b` in bottom 15% | 30.3% |
| `page-counter` | Regex `(?i)\bpage\s+(\d+)\s+of\s+(\d+)\b` in bottom 15% | 12.8% |

The low pass rates for `footer-section-id` and `page-counter` are expected and documented: AEC drawings do not use CSI section IDs or `Page N of M` counters in footers — that is by design. The families are correct for spec sheets; the corpus mix is ~55% drawings.

`title-block-anchor` has a partial runtime detector: it scores all four corner bands
by keyword-label density and selects a winning corner. Pass rate: 33.4% on Tier 1.
This is a best-effort Phase G bonus and the full table-structure detector (G-018, G-019)
is still deferred.

### 1.3 Locked contract drafts

Six contract modules in `crates/contracts/src/` define the type shapes later phases implement against. These are schema-complete structs annotated as contract-only; no runtime execution is wired. See [Section 5](#5-downstream-contract-boundaries).

---

## 2. Developer Workflow: pattern-dev

### 2.1 Getting started

```powershell
# Build (release recommended for corpus runs)
cargo build --release --bin pattern-dev

# Inspect a PDF
$env:PDFIUM_LIB_PATH = "f:\Projects\conset-pdf"
.\target\release\pattern-dev.exe inspect tests/corpus/tier1/SPEC_RWB_LHHS_ALL_ORG.pdf

# Test one family on one PDF
.\target\release\pattern-dev.exe test-pattern `
    --family footer-section-id `
    --output-dir audit_output/my-test `
    tests/corpus/tier1/SPEC_RWB_LHHS_ALL_ORG.pdf

# Batch validation (Tier 1)
.\target\release\pattern-dev.exe validate-corpus `
    --tier 1 `
    --output-dir audit_output/run-1

# Dry-run (no files written)
.\target\release\pattern-dev.exe validate-corpus `
    --tier 1 `
    --output-dir audit_output/run-1 `
    --dry-run
```

### 2.2 Output artifact layout

```
<output-dir>/
  <fixture-stem>/
    footer-section-id/
      page-0000-footer-section-id.json    ← sidecar (test-pattern also writes .png overlay)
      page-0001-footer-section-id.json
      ...
    page-counter/
      ...
    header-band/
      ...
    title-block-anchor/
      ...
    roi-candidate/
      ...
    spec-heading/
      ...
  validation-manifest.json    ← validate-corpus only
  corpus-report.json          ← validate-corpus only
```

**test-pattern** also writes `page-NNNN-<family>.png` overlays alongside each sidecar.
**validate-corpus** is sidecar-only (no PNGs) for volume efficiency.

### 2.3 PDFium bootstrap

PDFium discovery order (same as `classify-pdf`):

1. `$PDFIUM_LIB_PATH` environment variable
2. `$CARGO_WORKSPACE_DIR` (set by `cargo run`)
3. Current working directory

`pdfium.dll` / `libpdfium.so` must exist in the resolved directory.

### 2.4 Corpus tiering policy

| Tier | Directory | Purpose | validate-corpus permitted? |
|---|---|---|---|
| 1 | `tests/corpus/tier1/` | Primary known-structure fixtures | **Yes** |
| 2 | `tests/corpus/tier2/` | Extended / edge-case fixtures | **Yes** |
| 3 | `tests/corpus/tier3/` | Reserved | No |
| holdout | `tests/corpus/holdout/` | Blind evaluation set | **Never** |

The holdout prohibition is enforced in code: specifying `--tier 0`, `--tier 3`, or anything out of range causes an immediate error exit.

---

## 3. Overlay and Sidecar Schema

### 3.1 Sidecar JSON schema (version `0.5.0`)

All sidecar files follow this locked base schema:

```json
{
  "schema_version": "0.5.0",
  "pdf_path": "tests/corpus/tier1/SPEC_RWB_LHHS_ALL_ORG.pdf",
  "page_index": 4,
  "family": "footer-section-id",
  "matched_spans": [
    {
      "text": "23 82 16",
      "bbox": { "x": 0.05, "y": 0.94, "width": 0.15, "height": 0.02 },
      "span_confidence": 0.97
    }
  ],
  "confidence": 0.97,
  "failure_reason": null,
  "branch_reason": "best match in target band",
  "source": "vector",
  "engine_version": "0.1.0",
  "pattern_version": "0.5.0"
}
```

**Field invariants:**

| Field | Notes |
|---|---|
| `schema_version` | Always `"0.5.0"` in Phase 0.5. Increment to `"0.6.0"` on first additive change. |
| `page_index` | Zero-based. |
| `family` | Kebab-case family name (see Section 2.2 for all six). |
| `matched_spans` | Empty array on failure; never null. |
| `confidence` | `null` for schema-only families. Float in `[0.0, 1.0]` when detection ran. |
| `failure_reason` | One of `NO_MATCH`, `LOW_CONFIDENCE`, `REGION_MISS`, `AMBIGUOUS_TIE`, or `null`. |
| `source` | `"vector"` (runtime detection), `"ocr"` (deferred), `"schema-only"` (no detection). |
| `bbox` | Normalized top-left origin: `x`, `y`, `width`, `height` all in `[0.0, 1.0]`. |

**Confidence thresholds (locked, DEV_STANDARDS):**

| Range | Behaviour | Overlay color |
|---|---|---|
| `< 0.80` | Hard fail — emit `failure_reason`, escalate | Red |
| `0.80–0.95` | Pass with warning flag | Amber |
| `≥ 0.95` | Proceed normally | Green |

**Schema evolution policy:** This schema is additive-only. New optional fields may be added with a minor version bump. Removing or renaming a field requires a major version bump and an explicit breaking-change decision recorded in `docs/current-state/decision-log.md`.

### 3.2 Extended sidecar schemas (Phase G)

Three families emit extended schemas with family-specific nested objects:

**`title-block-anchor`** — `TitleBlockSidecar` (`tools/src/pattern_model.rs`):
- Base `MatchEvidence` fields (flattened at JSON top level)
- `"title_block"` object: `corner_candidates` (4 entries, BR/BL/TR/TL), `winning_candidate`, `field_candidates`, `template_lifecycle`

**`roi-candidate`** — `RoiCandidateSidecar`:
- Base fields
- `"roi_evidence"` object: `candidates` (3 pre-seeded zones: footer/body/header), `selected_roi`, `ranking_policy_version`

**`spec-heading`** — `SpecHeadingSidecar`:
- Base fields
- `"heading_diagnostics"` object: `candidates` (empty in schema-only mode), `detection_policy_version`

All three are schema-only in Phase 0.5: `source = "schema-only"`, `confidence = null`. Runtime detection populates the null/empty fields in Band 1–2.

### 3.3 Overlay PNG naming

```
page-{4-digit-zero-padded-index}-{family-kebab-case}.png
```

Examples: `page-0000-footer-section-id.png`, `page-0042-header-band.png`.

Overlay resolution: 1400 px wide (aspect-ratio-preserved). Palette fixed from Phase F:
- Green `rgba(0,200,0,255)`: PASS (confidence ≥ 0.95)
- Amber `rgba(200,200,0,255)`: WARN (0.80–0.95)
- Red `rgba(200,0,0,255)`: FAIL
- Grey `rgba(128,128,128,255)`: detection band outline (always rendered)

### 3.4 validation-manifest.json and corpus-report.json

**`validation-manifest.json`** — per-fixture breakdown:
```json
{
  "schema_version": "0.5.0",
  "generated_at_utc": "...",
  "tiers": [1],
  "families_tested": ["footer-section-id", "page-counter", "header-band",
                       "title-block-anchor", "roi-candidate", "spec-heading"],
  "fixture_count": 27,
  "errored_fixtures": 0,
  "total_pages": 2892,
  "determinism_regressions": 0,
  "fixtures": [
    {
      "stem": "SPEC_RWB_LHHS_ALL_ORG",
      "tier": 1,
      "pdf_path": "tests/corpus/tier1/SPEC_RWB_LHHS_ALL_ORG.pdf",
      "page_count": 571,
      "families": {
        "footer-section-id": {
          "pass": 556, "warn": 10, "fail": 5, "skip": 0,
          "total_pages": 571,
          "sidecar_fingerprint": 12345678901234567890,
          "artifact_dir": "audit_output/.../footer-section-id"
        }
      },
      "determinism_ok": true,
      "determinism_note": null
    }
  ]
}
```

**`corpus-report.json`** — aggregate by family:
```json
{
  "schema_version": "0.5.0",
  "fixture_count": 27,
  "total_pages": 2892,
  "determinism_regressions": 0,
  "by_family": {
    "footer-section-id": {
      "pass": 877, "warn": 58, "fail": 1957, "skip": 0,
      "total_pages": 2892, "pass_rate": 0.3033
    }
  }
}
```

**Determinism fingerprints:** `sidecar_fingerprint` is a FNV-1a 64-bit rolling hash over all sidecar JSON bytes for that family×fixture combination, accumulated in page order. Two runs on the same machine with the same binary must produce identical values. Any change in a detection result, schema field, or serialization order will change the fingerprint — this is intentional and is the regression gate.

---

## 4. Evidence Payloads: Title-Block, ROI, Spec Heading

These three families are the primary targets for Band 1 runtime implementation. The schema placeholders are already in place; later phases fill in the null/empty fields.

### 4.1 `title-block-anchor` — what Band 1 must add

The Phase G schema-only sidecar pre-seeds:
- Four `CornerBandCandidate` entries (BR, BL, TR, TL) with standard region bboxes.
- `axis_line_count: null`, `cell_density: null`, `score: null` — all to be populated by the vector geometry detector.
- `winning_candidate: null` — to be set after scoring.
- `field_candidates: []` — sheet number, sheet title, project, firm, drawn-by fields to be extracted.
- `template_lifecycle` with all null drift metadata.

Phase G bonus work already wired: `detect_title_block_anchor()` in `tools/pattern_dev.rs` scores corner bands by keyword-label density. Gaps G-018 (frame/corner detector), G-019 (table-structure scoring), G-020 (sheet field extraction), and G-021 (template store) remain open.

### 4.2 `roi-candidate` — what Band 1 must add

Three `RoiZoneCandidate` entries are pre-seeded:
- Candidate 0: footer zone (y ∈ [0.85, 1.0])
- Candidate 1: body zone (y ∈ [0.15, 0.85])
- Candidate 2: header zone (y ∈ [0.0, 0.15])

`rank_score: null` on all three; `selected_roi: null`. Band 1 must:
1. Implement the scoring policy (G-011: autonomous deterministic ROI ranking).
2. Populate `rank_score` on each candidate.
3. Set `selected_roi` to the winning zone.

`ranking_policy_version` is pinned to `"0.5.0"` — bump when the scoring policy changes.

### 4.3 `spec-heading` — what Band 1/2 must add

`SpecHeadingDiagnostics.candidates` is empty in Phase 0.5. Band 1 must:
1. Implement the heading detector (G-022): line clustering, font-delta, caps-ratio, indent, leading-gap features.
2. Populate `candidates` with `SpecHeadingCandidate` entries including `bbox`, `text`, and all diagnostic floats.

`detection_policy_version` is pinned to `"0.5.0"` — bump when policy changes.

---

## 5. Downstream Contract Boundaries

All six modules in `crates/contracts/src/` are contract-only in Phase 0.5. No runtime execution is wired to these types. Later phases implement against them.

### 5.1 `intake` — `crates/contracts/src/intake.rs`

| Type | Purpose |
|---|---|
| `NormalizedIntakeBundle` | Top-level intake: primary PDF + addenda list + provenance metadata |
| `IntakeIssue` | Per-file or per-page issue with severity (`Info < Warning < Error < Fatal`) |
| `AddendaOrdering` | Enum: `DeclaredSequence`, `FilenameInferred`, `ManualOverride` |
| `OrderedAddendumSet` | Assembled addenda list with ordering policy and conflict flag |
| `AdvisoryClassificationManifest` | Advisory-only classification result (document type, medium) |
| `AdvisoryEntry` | Per-document classification data: `DocumentClass`, confidence, `TriageConfidence` |

**What Band 1 must implement against this contract:** Intake Triage stage (G-013, G-014, G-015, G-016). `NormalizedIntakeBundle` is the output of the Triage stage and the input to the Lexer pipeline.

**Invariant:** The `AdvisoryClassificationManifest` is advisory only. No destructive page split or medium assignment may be performed without explicit user confirmation (MASTER_PLAN Non-Negotiable #18).

### 5.2 `ocr_routing` — `crates/contracts/src/ocr_routing.rs`

| Type | Purpose |
|---|---|
| `OcrRoutingManifest` | Per-document decision: which pages need OCR |
| `OcrPageDecision` | Per-page routing: `PageTextSource` (vector / ocr / hybrid / unknown) |
| `OcrRoutingSummary` | Aggregate counts over the manifest |

**What Band 1 must implement:** G-027 — raster/low-text page detection gate and OCR extraction path with source tagging. The `PageTextSource` enum defines the allowed source values; no new values may be added without a schema version bump.

### 5.3 `schedule` — `crates/contracts/src/schedule.rs`

| Type | Purpose |
|---|---|
| `CanonicalSchedule` | Extracted schedule table with columns, rows, and export mapping |
| `ScheduleColumn` | Column definition: name, data type, unit |
| `ScheduleRow` | Row data as `Vec<Option<String>>` aligned to column order |
| `ScheduleDataType` | `Text`, `Integer`, `Decimal`, `Boolean`, `Date` |
| `ScheduleExportMapping` | Field-name → export-column mapping for JSON/CSV/XML adapters |

**What Band 2 must implement:** G-028 (schedule/table parser) + G-029 (export adapters). `CanonicalSchedule` is the canonical output of the schedule parser and the input to export adapters.

### 5.4 `assisted_intelligence` — `crates/contracts/src/assisted_intelligence.rs`

| Type | Purpose |
|---|---|
| `MicroMlDecisionRecord` | Decision from a local micro-ML model: feature vector, model version, confidence |
| `MlFeature` | Named feature scalar used as model input |
| `LlmValidationRequest` | Power-user LLM validation request: cropped region + extraction context |
| `LlmValidationResponse` | LLM response with advisory status and promotion intent |
| `LlmInstructionRequest` | Power-user LLM instruction request |
| `IntelligenceSource` | `LocalMicroMl`, `LlmAdvisory`, `LlmInstructionApproved` |

**What Band 2–3 must implement:** G-023 (AI fallback gate), G-024 (micro-ML runtime), G-025 (confidence fusion), G-026 (LLM API contracts). The `IntelligenceSource` enum enforces the boundary: `LlmAdvisory` results must never be written to the primary output without promotion to `LlmInstructionApproved` via an explicit user authorization step.

### 5.5 `operational_trust` — `crates/contracts/src/operational_trust.rs`

| Type | Purpose |
|---|---|
| `CorrectionManifest` + `CorrectionEntry` | Replayable correction record: field, old/new value, scope guards, source |
| `CorrectionSource` | `HumanReview`, `AutoCorrection`, `LlmApproved` |
| `InstructionDslManifest` + `InstructionBlock` + `InstructionOp` | Typed instruction DSL: tagged `InstructionOp` enum (Assign, Normalize, Redact, Merge, Split, Flag, Defer) |
| `DiffPayload` + `DiffEntry` | Field-level diff between two document states |
| `ExceptionRecord` + `ExceptionStatus` | Exception triage record: status (`Open`, `Acknowledged`, `Resolved`, `Suppressed`) |
| `ReviewObject` + `ReviewDecision` | Provenance-first review: field evidence, branch reason, correction history |
| `ResumeJobState` + `JobStatus` | Resumable batch job state: `Pending`, `Running`, `Paused`, `Complete`, `Failed` |
| `ConfidenceProfile` + `ConfidenceThresholdRule` | Named confidence policy profiles |
| `RedactionPolicy` + `RedactionRule` + `RedactionMode` | Outbound redaction control for external API paths |

**What Band 2–3 must implement:** G-030 (correction manifest + reapply engine), G-031 (diff/compare), G-032 (review UI contract), G-033 (privacy/redaction), G-034 (batch orchestration/job-state), G-035 (instruction DSL parser + executor), G-036 (confidence policy profiles).

**Critical invariant:** `CorrectionSource::LlmApproved` must never be set without an explicit user authorization step. Auto corrections (`AutoCorrection`) must be scoped with `scope_guards` populated. No correction may silently bypass the exception triage queue.

### 5.6 `knowledge` — `crates/contracts/src/knowledge.rs`

| Type | Purpose |
|---|---|
| `NormalizationRecord` | Raw-to-canonical mapping with policy version and confidence |
| `EntityRecord` | Resolved entity (firm, equipment tag, revision) with cross-document references |
| `IndexRecord` | Project knowledge index entry: what was found, where, and when |

**What Band 3 must implement:** G-037 (standards normalization wiring), G-038 (cross-document entity resolution), G-039 (project knowledge index and exception triage queue).

---

## 6. Runtime-Ready vs. Contract-Only Surface Map

| Surface | Location | Status |
|---|---|---|
| `pattern-dev inspect` | `tools/pattern_dev.rs` | **Runtime-ready** |
| `pattern-dev test-pattern` | `tools/pattern_dev.rs` | **Runtime-ready** |
| `pattern-dev validate-corpus` | `tools/pattern_dev.rs` | **Runtime-ready** |
| `header-band` detection | `tools/pattern_dev.rs` | **Runtime-ready** |
| `footer-section-id` detection | `tools/pattern_dev.rs` | **Runtime-ready** |
| `page-counter` detection | `tools/pattern_dev.rs` | **Runtime-ready** |
| `title-block-anchor` corner scoring | `tools/pattern_dev.rs` | **Runtime-ready (partial)** — corner selected; table-structure scoring deferred (G-018, G-019) |
| `roi-candidate` detection | `tools/pattern_dev.rs` → `pattern_model.rs` | **Schema-only** — runtime deferred to Band 1 (G-011) |
| `spec-heading` detection | `tools/pattern_dev.rs` → `pattern_model.rs` | **Schema-only** — runtime deferred to Band 1 (G-022) |
| Overlay PNGs | `tools/pattern_dev.rs` | **Runtime-ready** (test-pattern only; validate-corpus is sidecar-only) |
| Sidecar JSON schema v0.5.0 | `tools/src/pattern_model.rs` | **Locked — additive-only evolution** |
| `NormalizedIntakeBundle` + triage contracts | `crates/contracts/src/intake.rs` | **Contract-only** — runtime Band 1 (G-013–G-016) |
| `OcrRoutingManifest` | `crates/contracts/src/ocr_routing.rs` | **Contract-only** — runtime Band 1 (G-027) |
| `CanonicalSchedule` + export mappings | `crates/contracts/src/schedule.rs` | **Contract-only** — runtime Band 2 (G-028, G-029) |
| Micro-ML + LLM contracts | `crates/contracts/src/assisted_intelligence.rs` | **Contract-only** — runtime Band 2–3 (G-023–G-026) |
| Correction manifest + DSL + review + privacy | `crates/contracts/src/operational_trust.rs` | **Contract-only** — runtime Band 2–3 (G-030–G-036) |
| Normalization + entity + index | `crates/contracts/src/knowledge.rs` | **Contract-only** — runtime Band 3 (G-037–G-039) |
| Shared engine pipeline | `crates/engine/` | **Stub** — G-001–G-003, G-005 remain open Critical/High severity |
| IR span conversion | `crates/ir/` | **Stub** — G-005, G-007, G-008 remain open |
| Audit crate integration | `crates/audit/` | **Stub** — G-006, G-010 remain open |

---

## 7. Priority Band Assignments

Later phases implement the open gaps in priority order. This is the canonical band assignment from `MASTER_PLAN.md`:

| Band | Label | Gaps | Key deliverables |
|---|---|---|---|
| **Band 0** | Core pipeline | G-001, G-002, G-003, G-005, G-009 | Shared engine: extractor, processor, CLI, IR wiring, E2E test |
| **Band 1** | Intake + Drawing + OCR + Detection | G-011, G-013, G-014, G-015, G-016, G-018, G-019, G-020, G-022, G-027 | ROI ranking, Intake Triage, title-block table scoring, sheet fields, spec heading, OCR gate |
| **Band 2** | Schedules + Assisted + Corrections + Review | G-021, G-023, G-024, G-025, G-026, G-028, G-029, G-030, G-031, G-032, G-033, G-034, G-035, G-036 | Schedule parser, export adapters, micro-ML, AI fallback, correction manifest, diff, review UI, privacy, batch orchestration, DSL |
| **Band 3** | Knowledge + Entity + Normalization | G-037, G-038, G-039 | Standards normalization, entity resolution, project knowledge index |
| **Band 4** | GUI + Production workflows | G-012 | Admin tooling, production furniture detection, production cloud inference |

**Band 0 is the critical path.** Nothing in Bands 1–4 can reach production without a working shared engine pipeline (G-001 → G-005 → G-009).

**Band 1 is the next priority** once Band 0 is closed. `validate-corpus` output (sidecar trees + manifest) is the primary input to Band 1 detection work — specifically for developing ROI ranking (G-011), title-block table scoring (G-018, G-019), and spec heading (G-022).

---

## 8. Open Gaps Still Relevant to Downstream Work

The following gaps are most directly unblocked by Phase 0.5 output and should be tracked by the implementing agent:

| Gap | Severity | What Phase 0.5 provides to help close it |
|---|---|---|
| G-018: Drawing frame + corner detector | HIGH | `TitleBlockExtension.corner_candidates` schema; partial `detect_title_block_anchor()` implementation; 27-fixture DWG sidecar tree for iteration |
| G-019: Title-block table-structure scoring | HIGH | `CornerBandCandidate.axis_line_count / cell_density / score` fields ready to receive values; audit corpus for regression testing |
| G-020: Sheet field extraction | HIGH | `TitleBlockField` schema; field_candidates list ready to populate |
| G-021: Template store | MEDIUM | `TemplateLifecycle` schema with drift metadata placeholders |
| G-011: Autonomous ROI ranking | HIGH | `RoiEvidence.candidates[].rank_score` and `selected_roi` fields; 27-fixture sidecar tree |
| G-022: Spec heading detector | MEDIUM | `SpecHeadingDiagnostics.candidates` schema; 571-page SPEC sidecar tree for iteration |
| G-013: IntakeBundle contract | HIGH | `NormalizedIntakeBundle` type defined |
| G-027: OCR routing | HIGH | `OcrRoutingManifest` contract defined; `source: "ocr"` already in the sidecar schema |
| G-028: Schedule parser | HIGH | `CanonicalSchedule` + `ScheduleColumn` + `ScheduleRow` contract defined |
| G-001 + G-005: Shared engine wiring | CRITICAL | `PdfiumExtractor::extract_page()` already implemented (G-004 closed) |

---

*This document is authored as the Phase K deliverable of Phase 0.5 and supersedes any informal handoff notes. Later phases should update `docs/current-state/state-summary.md` as gaps are closed.*

---

## Appendix A: Implementation History

This appendix captures the step sequence, locked decisions, critical technical discoveries, test inventory, and corpus results from Phase 0.5 execution. The primary implementation plan (`phase05Implementation.md`) was retired on April 5, 2026 after its content was integrated here. All execution-relevant facts are preserved below.

### A.1 Phase Step Sequence

| Phase | Description | Completed |
|---|---|---|
| **A** | Lock scope, command surface, artifact families, and priority-band assignments before any restructuring | April 4, 2026 (pre-work) |
| **B** | Rework `tools` crate into a stable multi-binary home; add `pattern-dev` alongside `classify-pdf`; introduce `tools/src/` for shared helpers | April 4, 2026 |
| **C** | Build local read-only PDF inspection primitive; implement `PdfiumExtractor::extract_page()` as a bounded fix to return real `PageData` (closing G-004); expose `pattern-dev inspect` | April 4, 2026 |
| **D** | Define deterministic pattern and heuristic model primitives in `tools/src/pattern_model.rs`; lock `PatternSpec`, `HeuristicFamily`, `MatchEvidence`, `FailureCode`, `SourceTag`; lock confidence thresholds | April 4, 2026 |
| **E** | Implement single-PDF developer loop; wire `test-pattern` detection with `page.text().chars()` (Form XObject fix, see A.3); validate on SPEC_RWB_LHHS_ALL_ORG (571 pages): PASS=556 WARN=10 FAIL=5; dry-run stub wired | April 4, 2026 |
| **F** | Add deterministic overlays and sidecar JSON; implement `render_overlay_png()` at 1400 px; lock sidecar schema v0.5.0; lock overlay palette; stable artifact naming from this phase forward | April 4, 2026 |
| **G** | Add vector-first detection scaffolding: `TitleBlockSidecar`, `RoiCandidateSidecar`, `SpecHeadingSidecar` schema types; `serialize_sidecar_for_family()` dispatch; **bonus runtime**: `detect_title_block_anchor()` keyword-label density scorer; validated on DWG fixtures | April 4–5, 2026 |
| **H** | Write all downstream contract drafts to `crates/contracts/src/`; 6 modules; 24/24 unit tests; all marked `CONTRACT-ONLY` | April 5, 2026 |
| **I** | Implement `validate-corpus` batch command; FNV-1a 64-bit determinism fingerprinting; two confirmed runs over 27 Tier 1 fixtures (2,892 pages); `det_regressions=0` | April 5, 2026 |
| **J** | Add 32 unit tests + 5 integration tests; dry-run semantics; debug logging; all tests pass | April 5, 2026 |
| **K** | Publish downstream handoff documentation (`docs/PHASE_05_HANDOFF.md`) | April 5, 2026 |
| **L** | Final verification and closeout; all definition-of-done checks passed | April 5, 2026 |

Critical path: `B → C → D → E → F → G → I → J → K → L`. Phase H can overlap with E–G once Phase D data model is locked.

### A.2 Locked Command Surface

The following subcommand signatures are stable from Phase B and **must not be renamed or restructured** in later phases without an explicit breaking-change decision recorded in `docs/current-state/decision-log.md`.

```
pattern-dev inspect <PDF> [--page <N>] [--json]
```
Dumps per-page geometry summary: page count, dimensions, span count, text quality score, raster-vs-vector flag. `--json` emits machine-readable output.

```
pattern-dev test-pattern <PDF> --family <FAMILY> --output-dir <DIR> [--dry-run]
```
Applies one heuristic family to one PDF. Prints per-page match/no-match with confidence. Writes overlay PNGs and sidecar JSON to `<DIR>/<pdf-stem>/`.  
`--dry-run` exits 0, prints `[dry-run]`, writes nothing.

```
pattern-dev validate-corpus --tier 1 [--tier 2] --output-dir <DIR> [--dry-run]
```
Batch validation over explicit fixture tiers (never holdout). Emits per-fixture sidecar JSON (no PNGs for volume efficiency) plus `validation-manifest.json` and `corpus-report.json`.  
`--tier 3` and `--tier 0` (holdout-adjacent) cause immediate error exit — enforced in code.

**Valid `--family` values:** `footer-section-id`, `page-counter`, `header-band`, `title-block-anchor`, `roi-candidate`, `spec-heading`

### A.3 Critical Technical Discovery — Form XObject (D-028)

AEC spec templates (e.g., `SPEC_RWB_LHHS_ALL_ORG.pdf`) place running headers and footers inside **PDF Form XObjects** — self-contained content streams embedded inside the page. This caused a critical failure in Phase C:

- `page.objects().iter()` is **opaque to Form XObjects**: it yields only objects in the page's direct content stream, not inside embedded Form XObjects. On 499 of 571 pages, no footer text objects were found, producing 499 false failures.
- Resolution: use `page.text().chars()` (PDFium `FPDFText_LoadPage` pipeline), which **descends transparently** into all nested content streams including Form XObjects. After this switch: PASS=556, WARN=10, FAIL=5 — all 5 remaining failures verified as genuine blank/raster insert pages.
- This is recorded as decision D-028 in `docs/current-state/decision-log.md`.
- **Rule:** All future text access in `tools/pattern_dev.rs` and any engine crate that consumes AEC spec PDFs must use `page.text().chars()` (or equivalent PDFium text-layer API), not `page.objects().iter()`, to ensure Form XObject content is not silently invisible.

### A.4 Test Inventory

**Unit tests (32 total, in `tools/pattern_dev.rs` `#[cfg(test)] mod tests`)**

| Test | What it covers |
|---|---|
| `fnv1a_64_empty_returns_offset_basis` | FNV-1a 64-bit offset basis for empty input |
| `fnv1a_64_is_deterministic` | Same bytes → same hash every call |
| `fnv1a_64_distinct_inputs_produce_distinct_hashes` | Different sidecars → different hashes |
| `family_page_counts_accumulates_all_statuses` | PASS/WARN/FAIL/SKIP bucketing |
| `family_page_counts_total_pages_is_sum_of_all_buckets` | `total_pages()` = sum of all four counters |
| `family_page_counts_fingerprint_nonzero_for_nonempty_input` | Fingerprint changes on real content |
| `family_page_counts_fingerprint_matches_for_identical_sequences` | Same sequence → same fingerprint (determinism) |
| `family_page_counts_fingerprint_differs_for_different_content` | Different content → different fingerprint |
| `score_matches_empty_spans_returns_no_match_fail` | Empty → `(0.0, NoMatch, "no spans…")` |
| `score_matches_single_high_confidence_span_passes` | Single span at 0.97 → `(0.97, None, "best match")` |
| `score_matches_geometric_family_single_span_no_regex` | `header-band` (no regex) single span passes |
| `score_matches_two_spans_regex_applies_ambiguity_penalty` | Two spans at 1.0 → conf=0.85, passes |
| `score_matches_two_spans_low_confidence_is_ambiguous_tie` | Two spans at 0.5 → conf=0.425, `AmbiguousTie` |
| `sidecar_filename_is_zero_padded_4_digit_index` | Locks `page-{:04}-{family}.json` format |
| `corpus_report_has_required_top_level_fields` | Smoke corpus-report.json schema integrity |
| `validation_manifest_has_required_fields_and_fingerprints` | Smoke manifest.json per-fixture fingerprints |

(16 additional unit tests in `tools/src/pattern_model.rs` cover model type serialization and pattern spec construction.)

**Integration tests (5 total, in `tools/tests/pattern_dev_integration.rs`)**

| Test | `#[ignore]`? | What it verifies |
|---|---|---|
| `dry_run_validate_corpus_exits_zero_and_writes_no_files` | No | `--dry-run` exits 0, prints `[dry-run]`, writes nothing |
| `holdout_adjacent_tier_is_rejected` | No | `--tier 3` → non-zero exit + tier error in stderr |
| `validate_corpus_with_no_tier_is_rejected` | No | No `--tier` → non-zero exit |
| `golden_path_sidecar_has_required_schema_fields` | **Yes** (needs PDFium + corpus) | Sidecar JSON contains all schema v0.5.0 required fields |
| `determinism_two_test_pattern_runs_produce_byte_identical_sidecars` | **Yes** (needs PDFium + corpus) | Two `test-pattern` runs on same fixture → byte-identical sidecars |

The two `#[ignore]` tests require `PDFIUM_LIB_PATH` to be set and a PDFium library present. All 5 pass when run with `--include-ignored`.

### A.5 Corpus Validation Results (Phase I)

Two confirmed determinism runs over all 27 Tier 1 fixtures, 2,892 total pages:

| Family | Pass | Warn | Fail | Skip | Pass rate |
|---|---|---|---|---|---|
| `footer-section-id` | 877 | 58 | 1957 | 0 | 0.3033 |
| `page-counter` | 370 | 0 | 2522 | 0 | 0.1279 |
| `header-band` | 2857 | 0 | 35 | 0 | 0.9879 |
| `title-block-anchor` | 965 | 139 | 1788 | 0 | 0.3337 |
| `roi-candidate` | 0 | 0 | 0 | 2892 | 0.0000 (schema-only) |
| `spec-heading` | 0 | 0 | 0 | 2892 | 0.0000 (schema-only) |

`det_regressions=0` confirmed both runs. FNV-1a 64-bit fingerprints for all 27×6 family×fixture combinations matched exactly between Run 1 and Run 2. Artifacts at `audit_output/phase-i-smoke/` (run 1) and `audit_output/phase-i-smoke2/` (run 2).

**Footer pass rate context:** 30.3% overall pass rate is expected — most drawing pages have no spec footer. On SPEC-only pages the effective pass rate is ~97%. The low overall rate reflects corpus mix (DWG fixtures dominate).

### A.6 Explicit Non-Goals (Phase 0.5 Scope Boundary)

The following were explicitly excluded from Phase 0.5. Later phases must not assume Phase 0.5 did any of this:

1. Completing the shared engine extraction chain (G-001, G-002, G-005 remain open).
2. Shipping full Intake Triage execution in runtime workflows.
3. Shipping production OCR runtime integration.
4. Shipping production micro-ML or cloud LLM execution paths.
5. Shipping complete enterprise queueing, search UI, or knowledge-browser UX.
6. Reintroducing manual ROI/profile authoring as a primary end-user workflow.
7. Rewriting or extending the canonical standards docs (AEC_STANDARDS, MASTERFORMAT_REFERENCE).
8. GUI work of any kind.

### A.7 Integrated Decisions (Locked Policy)

The following decisions were locked during Phase 0.5 execution and must not be reversed without an explicit breaking-change decision in `docs/current-state/decision-log.md`:

- `page.text().chars()` is the required text access API for all future AEC PDF work (see A.3 — Form XObject D-028).
- Sidecar schema version `0.5.0` is locked from Phase F; field additions are additive (allowed), field renames/removals are breaking (require version bump and migration note).
- Heuristic families `title-block-anchor`, `roi-candidate`, and `spec-heading` must always emit `"source": "schema-only"` until Band 1 runtime is wired; they must never emit synthetic confidence values.
- `image`, `imageproc`, `regex` are dev-tool-only dependencies; they must not be added as workspace-level dependencies or propagate into engine crates.
- Locked command subcommand signatures (`inspect`, `test-pattern`, `validate-corpus`) are stable from Phase B; renaming requires an explicit breaking-change decision.
- `crates/contracts/` modules are `CONTRACT-ONLY`; no runtime execution code may be added without closing the corresponding gap in `docs/current-state/gap-register.md`.
- Holdout prohibition (`tests/corpus/holdout/`) is enforced in code; any future change to allow holdout access requires a decision log entry explaining the rationale.
