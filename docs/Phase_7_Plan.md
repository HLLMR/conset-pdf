## Project State: Post-Phase 6

The core pipeline is complete and working. Every individual stage exists, is tested, and passes:

| Phase | Deliverable | Status |
|---|---|---|
| 1 | Transcript extraction (PDFium) | ✅ |
| 2 | Section segmentation + chrome metadata | ✅ |
| 3 | CSI outline parser (hardened, 0.2% unclassified) | ✅ |
| 4 | AST edit operations | ✅ |
| 5/5.5 | Section regeneration + layout geometry + font typography | ✅ |
| 6 | PDF stitching via lopdf | ✅ |

Baseline: **36/36 CLI integration** · **53/53 engine unit** · **13/13 IR unit**

---

## Pre-Phase-7 Work Not Yet Completed

There is exactly **one genuine functional gap** from pre-Phase-7 work, plus two documentation hygiene items.

### 1. Section title extraction (Phase 2 gap) — segment.rs

segment.rs

In `build_sections()`, `SectionEntry.section_title` is hardcoded to `String::new()`. The footer format carries the title immediately after the section ID:

```
2025-10-01    23 82 16 – HEATING WATER COILS - Page 2 of 3
                       ─────────────────────
                         this is never extracted
```

The `detect_section_id()` function identifies the section ID cluster but discards the title suffix. The Phase 2 definition of done lists "Chrome metadata extracted and stored" as ✅, and the project-level chrome metadata (project_id, firm, date) is genuinely implemented — but per-section titles were left empty-string throughout.

**Why it matters for Phase 7:** The `apply-addendum` change report must name each modified section. An empty `section_title` means the audit output says `"23 82 16"` changed but not that it was `"HEATING WATER COILS"`. The `SpecChromeMetadata` fields used by `regenerate` also include `section_title` — without this, the regenerated PDF footer is wrong unless the user provides it manually.

**Fix scope (~60 lines):**
- Extend `detect_section_id()` to return `(section_id, Option<section_title>)` — capture the `– TITLE` suffix cluster
- Pass the title through `build_sections()` into `SectionEntry.section_title`
- One new unit test verifying title extraction on the known corpus footer format

### 2. state-summary.md "Next Focus" is stale

state-summary.md

The doc says "Phase 7 candidates: intake triage, cross-medium addenda merge, or drawing title-block localization." This predates the decision to treat Phase 7 as the `apply-addendum` end-to-end workflow per MASTER_PLAN authority. It needs one paragraph update.

### 3. MASTER_PLAN dependency graph annotation is ambiguous

MASTER_PLAN.md — the dependency graph shows:
```
Phase 7: End-to-End (Week 14) ← ALPHA COMPLETE
```
That annotation is the *goal outcome* of Phase 7, not a status marker, but it reads as if Phase 7 is already done. A `← NEXT` annotation would be precise.

---

## Phase 7 Plan: End-to-End / `apply-addendum`

Phase 7 per MASTER_PLAN is a single orchestrating command that drives all existing stages:

```
cargo run -- apply-addendum \
  --original specs-rev0.pdf \
  --addendum addendum-3.json \
  -o specs-rev1.pdf \
  --audit-bundle audit/
```

All building blocks are ready. Phase 7 is pure **orchestration + error handling + audit collection**.

### Key Design Decisions

**Multi-section stitch ordering — last-to-first.** When multiple sections are patched, sections must be stitched in descending page order. Stitching section B (page 200-210) before section A (page 50-60) means section A's original SegmentIndex page indices are still correct when it's processed — nothing earlier in the document has shifted. This avoids any page-offset recalculation. A spurious `validate_unchanged_present` warning may appear for the already-replaced section; this is informational, non-blocking.

**Soft-fail / partial success.** If one section fails to parse, edit, or regenerate, it is recorded as `Failed { reason }` in the result; all other sections continue. The output PDF still contains the successful replacements. This satisfies Non-Negotiables #13 and #17.

**Chrome metadata merge.** The orchestrator sources chrome metadata from the SegmentIndex (already extracted from the source PDF headers/footers). If the addendum manifest includes a `project_metadata` override, it merges over the extracted metadata (manifest keys take precedence). This makes the workflow one-button for standard cases while allowing override for non-standard headers.

**AddendumManifest as the canonical automation contract.** The manifest is a JSON file validated before execution (pre-flight), with a dry-run mode that validates all section paths without invoking Chrome or writing any files.

---

### Micro-Task Checklist

#### Sprint 7.0 — Pre-Phase-7 cleanup (1 day)

- [ ] **7.0.A — Section title extraction in segment.rs** (~60 lines + tests)  
  Extend title capture from the footer cluster. Populate `SectionEntry.section_title`.  
  _Files:_ segment.rs, tests

- [ ] **7.0.B — Documentation sync** (~15 min)  
  Update state-summary "Next Focus", clarify MASTER_PLAN dependency graph annotation.

---

#### Task 7.1 — `AddendumManifest` IR types (~80 lines)
  _File:_ `crates/ir/src/addendum.rs` (new)

  ```rust
  pub struct AddendumManifest {
      pub description: Option<String>,
      pub issue_date: Option<String>,
      pub project_metadata: Option<SpecChromeMetadata>, // override extracted metadata
      pub sections: Vec<SectionEditSpec>,
  }
  pub struct SectionEditSpec {
      pub section_id: String,
      pub operations: Vec<EditOperation>, // reuse existing Phase 4 type
      pub chrome_override: Option<SpecChromeMetadata>,
  }
  pub struct SectionPatchResult {
      pub section_id: String,
      pub section_title: String,
      pub status: SectionPatchStatus, // Success { pages_removed, pages_inserted } | Failed { reason }
  }
  pub struct AddendumResult {
      pub manifest_description: Option<String>,
      pub total_sections: usize,
      pub succeeded: usize,
      pub failed: usize,
      pub section_results: Vec<SectionPatchResult>,
      pub output_path: Option<String>,
  }
  ```
  Export from lib.rs. Tests: serde round-trip for each type.

---

#### Task 7.2 — `apply-addendum` CLI subcommand (~60 lines in main.rs)
  _File:_ main.rs

  ```
  apply-addendum
    --original <PDF>         source spec book
    --addendum <JSON>        AddendumManifest JSON
    --output <PDF>           stitched output (skipped on dry-run)
    --audit-bundle <DIR>     directory for audit artifacts [optional]
    --dry-run                validate + parse + edit; skip Chrome + stitch
  ```
  Wire to `WorkflowOperation::SpecsPatch` in the request metadata.

---

#### Task 7.3 — `ApplyAddendumHandler` (~100 lines)
  _File:_ `apps/backend-cli/src/handlers/apply_addendum.rs` (new)

  - Load and validate `AddendumManifest` JSON
  - Emit `OperationStarted` + `OperationEnded` audit events
  - Delegate to `SpecsPatchOrchestrator`
  - Write `AddendumResult` and audit artifacts to `--audit-bundle` dir

---

#### Task 7.4 — `SpecsPatchOrchestrator` (~250 lines)
  _File:_ `crates/engine/src/specs_patch.rs` (new); export from lib.rs

  Core algorithm:
  1. **Extract** transcript from source PDF via `Extractor`
  2. **Segment** transcript → `SegmentIndex`
  3. **Merge** chrome metadata: extracted + manifest override
  4. **Parse + Edit + Regenerate** — for each `SectionEditSpec`:  
     - Parse section AST  
     - Apply `SectionEditor::apply(EditRequest)`  
     - If dry-run: `SectionRenderer::dry_run()`; else `SectionRenderer::render()`  
     - Collect `SectionPatchResult` (success or failure)
  5. **Stitch** — for successful sections, sorted by `start_page` descending (last-to-first):  
     - On first pass: copy source PDF to temp output path  
     - Sequential `PdfStitcher::stitch()` on the accumulating output  
  6. Return `AddendumResult`

  Partial success: if a section regeneration fails, it is skipped from the stitch list; the failure is recorded in `SectionPatchResult`.

---

#### Task 7.5 — `WorkflowOperation::SpecsPatch` wiring in contracts (~5 lines)
  The contract variant already exists. Ensure the handler properly sets metadata keys (`manifest_path`, `original_path`) so the audit bundle records them.

---

#### Task 7.6 — Audit bundle output (~80 lines, within handler)
  Write to `--audit-bundle` directory:
  - `manifest.json` — run summary (session_id, engine_version, input hashes, timestamp)
  - `segment-index.json` — copy of detected SegmentIndex
  - `change-report.json` — `AddendumResult` with all `SectionPatchResult` entries
  - `sections/<section-id>/` — per-section artifacts: edited AST JSON, regenerated PDF (if Chrome ran)

---

#### Task 7.7 — Integration tests (~150 lines)
  _File:_ cli_integration_test.rs

  - `apply_addendum_dry_run_no_write` — dry-run on real SPEC fixture, verifies no output PDF written
  - `apply_addendum_missing_original_fails` — non-existent source PDF returns error
  - `apply_addendum_missing_manifest_fails` — non-existent manifest returns error
  - `apply_addendum_unknown_section_continues` — section ID not in spec book → that section fails, others succeed (partial success)
  - `apply_addendum_invalid_edit_path_continues` — bad node path → section fails, others succeed
  - `apply_addendum_produces_valid_pdf` — `#[ignore]` Chrome test: full round-trip produces `%PDF`-headed output with correct page count

---

#### Task 7.8 — Documentation updates
  - CHANGELOG.md — Phase 7 entry
  - state-summary.md — Phase 7 complete section
  - capability-matrix.md — update specs-patch row to Implemented
  - MASTER_PLAN.md — Phase 7 checkboxes ☐ → ✅, dependency graph annotation
  - repo memory — update phase status, test counts

---

### Why Phase 7 is Tractable

The heavy lifting (PDFium extraction, CSI parser, edit engine, Chrome renderer, lopdf stitcher) is all Phase 1-6 infrastructure. Phase 7 adds roughly **800 lines of orchestration + test code** — no new algorithmic complexity. The hardest design question (multi-section stitch ordering) is solved by the last-to-first rule.

### What Phase 7 Explicitly Defers

- **Intake triage (G-013/G-014/G-016):** Full intake bundle assembly and page rotation normalization are Phase 8+ work. Phase 7's `apply-addendum` takes a single source PDF as input — no multi-file bundle required.
- **Drawing title-block localization (G-018/G-019):** Phase 9 per the roadmap.
- **Cross-medium addenda merge (G-017):** Requires intake triage first.
- **Pattern database as versioned JSON (Phase 8 polish):** Patterns remain hardcoded in Rust for now.

Phase 7 delivers **Alpha complete** for the spec addenda workflow: `apply-addendum --original specs.pdf --addendum addendum.json --output updated-specs.pdf`. That's the exact target.