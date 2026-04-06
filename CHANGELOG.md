# Changelog

All notable changes to this project are documented in this file.

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
