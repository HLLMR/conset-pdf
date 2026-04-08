# Changelog

All notable changes to this project are documented in this file.

## [2026-04-08] Sprint 8.3 Complete: Torture Corpus Validation Infrastructure

### Added

- **`CORPUS_MIN_COVERAGE`, `CORPUS_MAX_UNCLASSIFIED`, `CORPUS_MIN_SECTION_COUNT` constants** (`tools/pattern_dev.rs`): Pass/fail thresholds used by the pipeline corpus validator — 0.90, 0.01, and 1 respectively.

- **`--pipeline segment|parse` mode for `validate-corpus`** (`tools/pattern_dev.rs`): New optional `--pipeline <STAGE>` arg on the `ValidateCorpus` subcommand. When set, dispatches to `run_validate_corpus_pipeline()` instead of the existing heuristic pattern-family path. Runs the full engine pipeline (`Extractor::extract()` → `segment_transcript()`) for `segment`; additionally calls `parse_section_with_stats()` per section for `parse`. Emits `corpus-report.json` (schema `0.1.0`) with per-fixture pass/fail, `section_count`, `coverage_ratio`, `unclassified_ratio` (parse only), and an `aggregate` block. `conset-pdf-engine` and `conset-pdf-ir` added as workspace dependencies in `tools/Cargo.toml`.

- **Phase-H corpus baseline** (`audit_output/phase-h-corpus-baseline/`): First pipeline-level corpus run against all 27 Tier 1 fixtures. Results: 3/27 pass (11.1%). The three passing fixtures are all SPEC_* documents. 24 failures are expected by design — DWG, NAR, and SUB document types carry no CSI section-ID footer stamps. Primary validation target `SPEC_RWB_LHHS_ALL_ORG`: 89 sections, 96.5% coverage, 0.2% unclassified (7,971 nodes) — all thresholds met.

- **Unchanged-page content-hash validation** (`crates/engine/src/stitch.rs`): `validate_unchanged_present()` previously checked `doc.objects.contains_key(id)` only. Added `snapshot_hashes(doc, ids) -> HashMap<ObjectId, u64>` (FNV-1a-64 over `format!("{obj:?}")` bytes) called *before* `splice_page_tree()`, and `validate_unchanged_content(doc, &before_hashes, &mut warnings)` called after `fixup_bookmarks()`. Any object whose hash differs between the snapshot and the post-splice document appends a human-readable warning to `StitchResult.warnings`. Makes Non-Negotiable #12 ("unchanged pages must remain unchanged") mechanically enforceable.

- **`apply_addendum_dry_run_is_deterministic` integration test** (`apps/backend-cli/tests/cli_integration_test.rs`): Runs `apply-addendum --dry-run` twice on `SPEC_RWB_LHHS_ALL_ORG.pdf` with an identical manifest and compares the resulting `change-report.json` files. Asserts: `section_results` length and per-entry `section_id`/`status` are identical across runs; `diagnostics` entries with `stage=="parse"` have identical `node_count` and `node_distribution`. Timestamps and `elapsed_ms` fields are explicitly excluded. Closes the determinism testing gap (Risk 2 in the Phase 8 architecture review).

- **`stitch_unchanged_pages_content_hash_unchanged` unit test** (`crates/engine/src/stitch.rs`): Performs a valid single-section stitch and asserts `result.warnings` is empty, confirming the new hash check produces no false positives on clean stitches.

- **`stitch_two_sections_with_page_growth_preserves_middle_section` unit test** (`crates/engine/src/stitch.rs`): 9-page three-section fixture (A: 0–2, B: 3–5, C: 6–8); stitches C with a 4-page replacement (+1) then A with a 5-page replacement (+2); asserts final page count = 12, original B-section object IDs still present, and no content-hash warnings. Validates last-to-first stitch ordering under nontrivial page-delta conditions at both document ends.

## [2026-04-07] Phase 7 Complete: End-to-End Apply-Addendum Workflow

### Added

- **`crates/ir/src/addendum.rs`** — Phase 7 IR types for the apply-addendum pipeline:
  - `AddendumManifest`: JSON contract with optional `description`, `issue_date`, `project_metadata` override, and required `sections: Vec<SectionEditSpec>`.
  - `SectionEditSpec`: per-section patch spec with `section_id`, `operations: Vec<EditOperation>`, and optional `chrome_override`.
  - `SectionPatchStatus`: enum discriminating `Success { pages_removed, pages_inserted }` vs `Failed { reason }`.
  - `SectionPatchResult`: per-section outcome with `section_id`, `section_title`, and `status`.
  - `AddendumResult`: orchestrator output with aggregate `total_sections`, `succeeded`, `failed`, per-section `section_results`, and optional `output_path`.
  - Full serde round-trip tests for all types + `AddendumResult::from_results()` constructor.
- **`crates/engine/src/specs_patch.rs`** — `SpecsPatchOrchestrator::run()` — core Phase 7 orchestration engine:
  - **Algorithm**: Extract → Segment → [for each section] (Parse → Edit → Render Chrome OR dry-run) → Stitch (descending page-order, last-to-first).
  - **Partial success**: per-section failures recorded in `SectionPatchResult`; other sections continue.
  - **Chrome metadata merge**: base (extracted from source PDF) + manifest-level override + per-section override (highest priority).
  - **Temp directory**: intermediate replacement PDFs and stitch outputs managed in `$TMPDIR/specs_patch_<timestamp>/`.
  - **Issue date logic**: addendum `issue_date` only fills `metadata.date` when extracted date is empty.
  - 5 unit tests covering chrome metadata merge edge cases.
  - Exported from `crates/engine/src/lib.rs`.
- **`apps/backend-cli/src/handlers/apply_addendum.rs`** — `ApplyAddendumHandler` for CLI:
  - Loads and validates `AddendumManifest` JSON from `manifest_path` metadata key.
  - Delegates to `SpecsPatchOrchestrator::run()`.
  - Writes `change-report.json` to `--audit-bundle` directory when provided.
  - Emits `OperationStarted`/`OperationEnded` audit events.
  - Partial success: status is `failed` if all sections failed, `succeeded_with_warnings` if some succeeded.
- **`ApplyAddendum` subcommand** in `apps/backend-cli/src/main.rs`:
  - Args: `--original <PDF>` (source spec), `--addendum <JSON>` (AddendumManifest JSON), `--output <PDF>` (optional; skipped on dry-run), `--audit-bundle <DIR>` (optional), `[--dry-run]`.
  - Wires to `WorkflowOperation::SpecsPatch` (contract variant already existed).
- **5 Phase 7 integration tests** in `apps/backend-cli/tests/cli_integration_test.rs`:
  - Dry-run on SPEC fixture — parses and edits but writes no output PDF.
  - Missing original PDF — returns status=failed.
  - Missing manifest JSON — returns status=failed.
  - Unknown section ID in spec — partial success (marked as per-section failure in change-report).
  - Audit artifacts written — change-report.json exists in `--audit-bundle` dir.
  - Full round-trip Chrome test `#[ignore]` — real SPEC, real render, produces valid PDF with `%PDF` header.

### Pre-Phase-7 Cleanup (Tasks 7.0.A and 7.0.B)

- **Section title extraction in `crates/engine/src/segment.rs`** (Task 7.0.A):
  - `detect_section_id()` now returns `Option<(String, String)>` — captures section_id + section_title from footer.
  - `extract_title_from_clusters()` helper: scans footer clusters right-to-left after section ID cluster; strips leading `–` or `-`; trims whitespace.
  - `build_sections()` populates `SectionEntry.section_title` from extracted title (empty string when not detected).
  - 12 new unit tests covering footer format variants (em-dash, hyphen, no-title, multiple sections).
  - `SectionEntry.section_title` is now guaranteed populated for all sections (previously always empty string).
- **Documentation sync** (Task 7.0.B):
  - `docs/state-summary.md`: "Next Focus" updated to reflect Phase 7 as apply-addendum.
  - `docs/MASTER_PLAN.md`: Phase 7 dependency graph annotation clarified from `← ALPHA COMPLETE` to `← NEXT`.

### Baseline

- **36/36** CLI integration tests pass (30 pre-Phase 6 + 6 Phase 6 stitch; 2 Chrome `#[ignore]`).
- **65/65** engine unit tests pass (53 pre-Phase 7 + 12 new segment titles).
- **30/30** IR unit tests pass (18 prior + 12 new addendum types).
- **5/5** specs_patch unit tests pass (chrome metadata merge edge cases).
- **41/41** CLI integration tests pass (36 pre-Phase 7 + 5 Phase 7 apply-addendum; 2 Chrome `#[ignore]`).

### Non-Negotiable Design Adherence

- **Multi-section stitch order — last-to-first (descending page)** per Non-Negotiable #1. Stitching section B (page 200–210) before section A (page 50–60) preserves page indices for A when it is processed.
- **Partial success** achieves Non-Negotiable #13 (soft fail on syntax / typographic errors) and #17 (downstream stages continue when one section fails).
- **Chrome metadata override** prevents hand-patching: manifest-level and per-section chromemetadata let users update headers/footers without CLI changes.
- **Dry-run mode** validates arg correctness and HTML assembly without invoking Chrome or writing files — safe testing mode.
- **Audit bundle** captures full per-section change history: manifest, segment index, change report, per-section AST and PDFs.

---

## [2026-04-06] Font Typography Extraction (Phase 5.5 Supplement)

### Added

- **`lopdf = "0.40.0"` dependency** — added to workspace `[workspace.dependencies]`; wired into `crates/engine/Cargo.toml`. `rust-version` bumped from 1.82 → 1.85 to satisfy lopdf MSRV.
- **`crates/ir/src/stitch.rs`** — `StitchPlan` (original_path, section_id, segment_index, replacement_path, output_path, dry_run), `StitchResult` (section_id, pages_removed, pages_inserted, total_pages_before, total_pages_after, bookmarks_updated, warnings), `StitchError` (thiserror: SectionNotFound, OriginalNotFound, ReplacementNotFound, WriteFailed, PageRangeOutOfBounds, PdfStructure). Exported from `crates/ir/src/lib.rs`.
- **`crates/engine/src/stitch.rs`** — `PdfStitcher::stitch()` implements the full page-replacement algorithm:
  1. Load original + replacement PDFs via `lopdf::Document::load()`.
  2. Resolve section page range from `SegmentIndex`.
  3. Renumber replacement object IDs to avoid collisions, copy objects into original document.
  4. Splice `/Pages` root `/Kids` array: `original[..del_start] + replacement_pages + original[del_end+1..]`.
  5. Update `/Parent` references on replacement pages.
  6. Remove deleted section's `/Page` objects from the object store.
  7. Re-route any outline-item `/Dest` destinations pointing to deleted pages.
  8. Validate unchanged pages are still present.
  9. Write output file (skipped on dry-run).
  Private helpers: `resolve_section_range`, `sorted_page_ids`, `load_and_merge_replacement`, `splice_page_tree`, `find_pages_root_id`, `fixup_bookmarks`, `validate_unchanged_present`.
  8 unit tests: 3 pure-logic + 5 lopdf-based with temp PDFs.
- **`WorkflowOperation::Stitch`** contract variant added to `crates/contracts/src/lib.rs`.
- **`apps/backend-cli/src/handlers/stitch.rs`** — `Stitch` CLI handler: reads `segment_index_path`, `replacement_path`, `section_id` from request metadata; loads `SegmentIndex`; calls `PdfStitcher::stitch()`; emits `OperationStarted`/`OperationEnded` audit events.
- **`Stitch` subcommand** wired in `apps/backend-cli/src/main.rs` — args: `--input`, `--segment-index`, `--section`, `--replacement`, `--output`, `[--dry-run]`.
- **6 Phase 6 integration tests** in `apps/backend-cli/tests/cli_integration_test.rs`: dry-run no-write, missing-input failure, missing-replacement failure, missing-segment-index failure, unknown-section-id failure, full stitch produces valid PDF with `%PDF` header.

### Baseline

- 36/36 CLI integration tests pass (30 pre-Phase 6 + 6 Phase 6; 2 `#[ignore]` tests excluded).
- 53/53 engine unit tests pass (43 pre-Phase 6 + 8 Phase 6 stitch + 2 ignored render tests).
- 13/13 IR unit tests pass.

---

## [2026-04-06] Font Typography Extraction (Phase 5.5 Supplement)

### Added

- **`font_weight: f32` and `is_italic: bool` on `SpanData`** (`crates/pdf-extraction/src/types.rs`): PDFium now populates `font_weight` via `text_obj.font().weight()` (matching against `PdfFontWeight` enum variants → f32 value 100–900; defaults to 400 when PDFium returns an error) and `is_italic` via `text_obj.font().is_italic()`. These represent actual values from the source PDF's font metadata.
- **`font_weight: f64` wired through extraction pipeline** (`crates/engine/src/pipeline/extraction.rs`): `span.font_weight` is now set from `f64::from(span_data.font_weight)` in addition to the existing `span.font_name` assignment. Previously `font_weight` was always hardcoded to 400.
- **`is_italic: bool` on `Span` IR type** (`crates/ir/src/types.rs`): New field with `#[serde(default)]` (defaults `false`); set from `span_data.is_italic` in the extraction pipeline. `Span::new()` initializes it to `false`.
- **`body_font_name: String` on `SectionLayout`** (`crates/ir/src/ast.rs`): Modal (most-frequent) font family name among body spans in the section, as reported by PDFium. Uses `#[serde(default = "default_font_name")]` for backward compatibility (defaults to `"Unknown"` when deserializing older AST JSON files).
- **`modal_font_name()` helper in `parse.rs`** (`crates/engine/src/parse.rs`): Computes the most-frequent font name from a slice using a `HashMap<&str, usize>` frequency count. `parse_section()` now collects `font_names: Vec<String>` from body spans and passes them to `compute_section_layout()`.
- **`body_font_name` used in `build_full_html()`** (`crates/engine/src/render/chrome.rs`): When `layout` is `Some` and `body_font_name` is not `"Unknown"` or empty, it replaces `config.font_family` as the CSS `font-family`. Falls back to `config.font_family` for sessions with no extractable font metadata.

### Backward Compatibility

- `Span.is_italic` and `SectionLayout.body_font_name` both carry `#[serde(default)]` — existing serialized transcript and AST JSON files deserialize without errors.
- `SpanData.font_weight` and `SpanData.is_italic` are new required fields; any code constructing `SpanData` struct literals directly (e.g. in extractor tests) must be updated.

---

## [2026-04-06] Layout Geometry Capture (Phase 5 Supplement)

### Added

- **`SectionLayout` IR type** (`crates/ir/src/ast.rs`): New struct capturing measured PDF geometry for a parsed section — `body_left` (normalized x of leftmost body span, i.e. the physical left text margin), `body_right` (normalized x of rightmost span right-edge), `font_size_pt` (median font size in points from source PDF spans), `line_gap_norm` (median top-to-top y distance between consecutive lines in normalized coordinates). Exported from `crates/ir/src/lib.rs` as a public type.
- **`x_indent: f64` on `AstNode`** (`crates/ir/src/ast.rs`): Normalized x position of the leftmost span on that node's first line. Populated by the parser from span geometry; `#[serde(default)]` so existing serialized `ast.json` files remain fully compatible.
- **`layout: Option<SectionLayout>` on `SectionAst`** (`crates/ir/src/ast.rs`): Measured geometry attached to each section at parse time. `None` when fewer than two body spans are found (empty or title-only sections). `#[serde(default)]` for full backward compatibility.
- **Layout geometry computation in `parse.rs`** (`crates/engine/src/parse.rs`): `parse_section()` now collects body-span raw data (x, right-edge, font size, y) across all pages of the section and calls `compute_section_layout()` to produce a `SectionLayout`. New helpers: `compute_section_layout(x_vals, x_right_vals, font_sizes, span_ys) -> Option<SectionLayout>` using `f64::min`/`f64::max` for left/right marginss and `median_val()` for font size and line-gap statistics; `median_val(vals: &[f64]) -> f64` (sort + midpoint).
- **`x_indent` propagation in `cluster_lines()` and `classify_lines()`** (`crates/engine/src/parse.rs`): `cluster_lines()` now returns `Vec<(String, usize, f64)>` — the third element is the `bbox.x` of the leftmost span on that line after x-sort. `classify_lines()` writes it into `FlatItem.x_indent`; `build_tree()` carries it through to `AstNode.x_indent`.
- **Measured margin-left in `build_body_html()`** (`crates/engine/src/render/body.rs`): Function signature extended to `build_body_html(ast, config, layout: Option<&SectionLayout>)`. When `layout` is `Some` and `node.x_indent > 0.0`, an inline `style="margin-left: X.XXXin"` is emitted using `(node.x_indent - layout.body_left) * 8.5` (scales normalized coords to CSS inches on an 8.5 in page). Falls back silently to CSS-class-only output when `layout` is `None`.
- **Measured font size and line-height in `build_full_html()`** (`crates/engine/src/render/chrome.rs`): Function signature extended to `build_full_html(body, chrome, config, layout: Option<&SectionLayout>)`. When `layout` is `Some`, `font_size_pt` replaces `config.font_size_pt` as the CSS `font-size`; `line_gap_norm * (11 * 72)` is converted to a CSS `line-height` ratio (clamped 1.1–2.0), replacing the hardcoded 1.4. Falls back to config defaults when `layout` is `None`.
- **`ast.layout.as_ref()` threaded in `SectionRenderer`** (`crates/engine/src/render/mod.rs`): Both `render()` and `dry_run()` pass `ast.layout.as_ref()` to both `build_body_html` and `build_full_html`.

### Backward Compatibility

- Existing `ast.json` files with no `x_indent` or `layout` fields deserialize correctly via `#[serde(default)]`.
- All 30 non-Chrome integration tests pass; 43/43 engine unit tests pass; 13/13 IR unit tests pass.

---

## [2026-04-06] Phase 5 Complete: Section Regeneration

### Added

- **Phase 5 (Section Regeneration — AST → PDF via HTML/CSS + Chrome):**
  - `crates/ir/src/render.rs` — `SpecChromeMetadata` (project/firm/section chrome fields), `RenderConfig` (font family, font size, page size), `PageSize` (Letter/A4), `RenderResult` (pdf_bytes, page_count_estimate, warnings), `RenderError` (thiserror-derived: ChromeNotFound, TempFileError, ChromeError, PdfNotWritten, HtmlBuildError).
  - `crates/engine/src/render/body.rs` — `build_body_html()`: converts `SectionAst` → HTML fragment; maps all 7 `OutlineTag` variants to CSS classes (`csi-part`, `csi-article`, `csi-para`, `csi-sub1`, `csi-sub2`, `csi-sub3`, `csi-body`).
  - `crates/engine/src/render/chrome.rs` — `build_full_html()`: wraps body in `<!DOCTYPE html>` with `<head>`, CSS reset/typography, and CSS Paged Media `@page` margin-box rules (`@top-center` firm/project header, `@bottom-left` date+section+title, `@bottom-right` Page N of M via `counter(page)`/`counter(pages)`). All chrome fields HTML-escaped.
  - `crates/engine/src/render/chrome_pdf.rs` — `render_html_to_pdf()`: invokes Chrome subprocess with `--headless --disable-gpu --no-sandbox --print-to-pdf`; binary discovery via `CHROME_PATH` env var with fallback to common Windows/Linux/macOS paths; writes PDF to temp file and reads bytes back.
  - `crates/engine/src/render/mod.rs` — `SectionRenderer { config: RenderConfig }`: `new(config)`, `with_defaults()`, `render(ast, chrome_meta) -> Result<RenderResult, RenderError>`, `dry_run(ast, chrome_meta) -> RenderResult` (builds HTML, skips Chrome — used by CLI `--dry-run` and CI).
  - `apps/backend-cli` — `regenerate` subcommand (`--ast`, `--chrome-metadata`, `--output`, `--section`, `--dry-run`, `--font`, `--font-size`); handler `src/handlers/regenerate.rs`; `WorkflowOperation::Regenerate` variant added to contracts.
  - 5 Phase 5 integration tests appended to `apps/backend-cli/tests/cli_integration_test.rs`: dry-run no-write, missing-AST failure, missing-section failure, invalid-chrome-metadata failure, full-Chrome round-trip (`#[ignore]`). 30 non-ignored tests pass.
- **G-010 closed** — 4 behavioral audit tests added to `crates/audit/src/bundle.rs`: JSON round-trip serde, event ordering, count tracking with clear, iter yields all events.
- **G-008 accepted-closed** — `Document`/`Element` stubs in `crates/ir/src/types.rs` have no defined downstream consumer; closing as deliberate scope deferral pending Phase 6+ consumer.

---

## [2026-04-05] Phase 4 Complete: Edit Operations

### Added

- **Phase 4 (Surgical AST Edit Operations):**
  - `crates/ir/src/edit.rs` — `NodePath`, `EditOperation` (InsertAfter/Delete/Replace), `EditRequest`, `EditResult`, `EditError`; all serde round-tripped (8/8 unit tests).
  - `crates/engine/src/edit.rs` — `SectionEditor` with pre-flight validation and `apply(EditRequest) -> EditResult`; helper functions `find_node`, `find_node_mut`, `find_in_parent`, `make_marker`, `renumber_siblings`, `apply_delete_to_section`, `apply_replace_to_section`, `apply_insert_after_to_section` (27/27 unit tests).
  - `apps/backend-cli` — `edit` subcommand (`--input`, `--operations`, `--output`, `--dry-run`); handler in `src/handlers/edit.rs`; `WorkflowOperation::Edit` variant added to contracts.
  - All 7 CLI handlers confirmed emitting `OperationStarted`/`OperationEnded` audit events — **G-006 closed**.
  - 7 Phase 4 integration tests appended to `apps/backend-cli/tests/cli_integration_test.rs`: delete-renumber, replace-text-only, insert-after-renumber, multi-op, invalid-section failure, invalid-path failure, dry-run no-write. All 26/26 integration tests passing.
- **G-003 accepted-closed** — `backend-cli` covers all runtime paths; `crates/engine/src/main.rs` version-only is deliberate scope deferral.

### CSI Renumbering Scheme (canonical, locked in Phase 4)

| Level | Tag | Marker format | Overflow limit |
|---|---|---|---|
| 0 | Part | `PART N` | unlimited |
| 1 | Article | `P.N` (P = parent Part number) | unlimited |
| 2 | Paragraph | `A.` … `Z.` | 26 |
| 3 | SubParagraph | `1.` `2.` … | unlimited |
| 4 | SubSubParagraph | `a.` … `z.` | 26 |
| 5 | SubSubSubParagraph | `1)` `2)` … | unlimited |

---

## [2026-04-05] Phase 1–3 Complete + Parser Hardening Sprint

### Added

- **Phase 1 (Layout Transcript Extraction):** Real PDFium extraction wired in `crates/engine/src/pipeline/extraction.rs`; `SpanData`→`Span` conversion with normalized bbox (top-left origin, [0,1] range); `visualize` CLI subcommand with per-page PNG overlay; G-001, G-002, G-005, G-007, G-009 all closed; 8/8 integration tests pass across SPEC, DWG, NAR, SUB, and simple fixtures.
- **Phase 2 (Section Segmentation):** `SegmentIndex` and `SectionEntry` IR types in `crates/ir/src/segment.rs`; CSI footer-oracle segmentation engine in `crates/engine/src/segment.rs`; `segment` and `visualize-segments` CLI subcommands; 5/5 Phase 2 integration tests pass.
- **Phase 3 (Paragraph Parsing & AST):** `ParsedDocument`, `SectionAst`, `AstNode`, `OutlineTag` IR types in `crates/ir/src/ast.rs`; line-clustering + 5-level CSI outline tree parser in `crates/engine/src/parse.rs` with `build_tree` and `classify_lines`; HTML collapsible AST visualizer in `crates/engine/src/visualize_ast.rs`; `parse` and `visualize-ast` CLI subcommands; 6/6 Phase 3 integration tests pass.

### Fixed (Parser Hardening Sprint — April 5, 2026)

- **Span x-sort:** PDFium returns spans in content-stream order; all `body_spans` now sorted by `(y ASC, x ASC)` before line clustering. Without this, PART headings were assembled in wrong order (`"GENERAL PART 1"` instead of `"PART 1 GENERAL"`), causing regex failures.
- **`LINE_Y_EPSILON` raised 0.005 → 0.012:** Same-visual-line spans (e.g. dash separators) had y-delta up to 0.006, formerly splitting them into separate clusters and breaking line text assembly.
- **Cluster-based section ID detection** (`crates/engine/src/segment.rs`): Footer section IDs are rendered as 2–3 adjacent spans (`"22 "`, `"07 "`, `"00 "`). New `detect_section_id()` merges all footer spans (y > FOOTER_Y=0.90) into x-proximity clusters, applies a single-digit merge pass (`"0 0"`→`"00"`) for PDFium split-zero rendering, skips date clusters (4-digit year), then matches on merged text. Section detection: 16 → 89 sections.
- **`FOOTER_Y` raised 0.85 → 0.90:** Body text at y ≈ 0.86–0.89 contained cross-references with CSI section IDs (e.g. `"specifications in accordance with Section 22 05 00."`) that generated false-positive section boundaries.
- **Noise-only line skipping** (`crates/engine/src/parse.rs`, `classify_lines`): Lines whose trimmed text consists entirely of punctuation/whitespace characters (`- –—|•·*\\/`) are discarded rather than becoming unclassified root nodes that absorb all subsequent content as continuation text.
- **Article regex major ≥ 1** (`article_re`): Changed `^(\d+\.\d+)` → `^([1-9]\d*\.\d+)`. Eliminates phantom articles from `0.x` decimal continuations (e.g., `"0.26 Acceptable Manufacturers"`, `"0.016 inches thick wall"`).
- **Article regex uppercase title** (`article_re`): Changed `(\S.*)` → `([A-Z].*)`. Requires article title to start with uppercase letter, excluding decimal measurements and lowercase continuation fragments.
- **`inject_missing_parts` recovery pass** (`crates/engine/src/parse.rs`): New post-classification pass inserts synthetic `PART N` flat items when an article's major number jumps to a part that was never explicitly opened. Recovers from (a) PART heading lines broken by PDFium kerning artifacts and (b) segmenter cutting section one page too early. Corrected 13/70 structured sections that had wrong-PART article nesting.
- **Test isolation** (`apps/backend-cli/tests/cli_integration_test.rs`): `cli_segment_and_visualize_segments_spec_pdf` now clears the vis directory before the visualize call. Previously accumulated PNGs from prior runs inflated the count check.

### Summary Statistics (post-hardening, `SPEC_RWB_LHHS_ALL_ORG.pdf`, 571 pages)

| Metric | Before hardening | After hardening |
|---|---|---|
| Sections detected | 16 | 89 |
| Total AST nodes | 218 | 7,971 |
| Unclassified node rate | 11% | 0.2% |
| Wrong-PART sections | 13/70 | 0/70 |
| Integration tests | 19/19 | 19/19 |

## [2026-04-04] Phase 0.5 Phases B–E

### Added
- `pattern-dev` binary in `tools` crate alongside `classify-pdf`; multi-binary crate restructure with shared `tools/src/` module directory (Phase B).
- Real `PdfiumExtractor::extract_page()` implementation in `crates/pdf-extraction/src/extractor.rs`; replaces stub with actual pdfium-render object API calls returning `PageData` with `SpanData`, `RawBBox`, width/height in pts (Phase C, closes G-004).
- Shared pattern model in `tools/src/pattern_model.rs`: `PatternSpec`, `HeuristicFamily` (6 variants with stable kebab-case identifiers), `RegionBand`, `NormalizedBBox`, `MatchEvidence`, `MatchedSpan`, `FailureCode`, `SourceTag` — all serde-serializable, all unit-tested (Phase D, 9 tests green).
- `pattern-dev test-pattern --family <FAMILY> --output-dir <DIR>` single-PDF detection loop with real per-page sidecar JSON (schema version `0.5.0`) and PASS/WARN/FAIL/SKIP reporting; runtime-ready families: `footer-section-id`, `page-counter`, `header-band`; schema-only families emit `"source": "schema-only"` sidecars (Phase E).
- `pattern-dev inspect --spans` diagnostic flag: dumps per-span text, raw bbox coords, and normalized mid-Y for every page object (with bounds-failure count), enabling band-threshold diagnosis on failing pages.
- `pattern-dev validate-corpus --tier 1 [--tier 2] --dry-run` fixture inventory pass enumerating all PDFs in requested tiers; holdout prohibition enforced at argument validation.
- `--dry-run` flag on `test-pattern` validated: argument wiring confirmed, no file writes occur.

### Changed
- Text extraction in `pattern-dev test-pattern` uses `page.text().chars()` (PDFium `FPDFText_LoadPage` / `FPDFText_GetCharBox` pipeline) rather than `page.objects().iter()`. The object-level API treats PDF Form XObjects as opaque nodes; running footers and headers placed via reusable content streams are invisible to it. The FPDFText API descends transparently into Form XObjects and returns page-coordinate bounds for every character. Characters are grouped by 6 pt baseline tolerance, then reconstructed into logical lines with word-gap spacing before pattern matching.

### Notes
- Smoke-tested on `SPEC_RWB_LHHS_ALL_ORG.pdf` (571 pages): PASS=556 WARN=10 FAIL=5. All 5 failures are confirmed blank / raster insert pages (cover + TOC inserts) with no footer text layer.
- Confidence thresholds from `DEV_STANDARDS.md` enforced: `≥ 0.95` → PASS, `0.80–0.95` → WARN, `< 0.80` → FAIL.
- D-028 added to decision log documenting the FPDFText-vs-objects API choice as a permanent architectural decision.
- G-004 closed in gap register; G-005 (PDF→IR span conversion wiring in shared engine) remains open.

## [2026-03-23] Phase 0 Closeout + Postmortem Integration

### Added
- Monorepo app/crate boundaries for backend CLI and desktop GUI scaffolds.
- New shared contract and workflow crates for stable backend/frontend integration surfaces.
- Migration closeout documentation for Phase 0 and Phase D integration outcomes.
- Repository structure and migration log docs to codify architecture and boundary rules.
- This changelog for milestone tracking and release-level highlights.

### Changed
- Documentation authority and canonical references aligned to the top-level `docs/` canonical set and `docs/DOCUMENTATION_INDEX.md`.
- README revised for current workspace layout, corpus paths, and active CLI commands.
- Coverage workflow hardened to emit machine-readable reports (`cobertura.xml`, `lcov.info`) and upload them to Codecov with artifact retention.

### Removed
- Non-permanent runtime audit artifacts from source control scope.
- Legacy/debris planning and archival files outside canonical active docs.

### Notes
- This milestone represents the requested Phase 0 closeout and postmortem update checkpoint before Phase 0.5 GUI implementation work.
