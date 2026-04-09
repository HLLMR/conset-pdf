// pattern-dev — Phase 0.5 pattern development and inspection tool.
//
// Subcommands (Phase B skeleton — implementation filled in Phases C-I):
//
//   inspect         Load a PDF and dump per-page geometry summary.
//   test-pattern    Apply one heuristic family to a single PDF and emit
//                   deterministic overlays + sidecars.
//   validate-corpus Batch-run test-pattern against explicit Tier 1/Tier 2
//                   fixtures and emit an aggregate validation manifest.
//
// Command surface is locked per Phase A. Subcommand signatures must not be
// renamed or restructured without an explicit breaking-change decision.

// Phase D: shared pattern model (pattern_model.rs lives in tools/src/).
#[path = "src/pattern_model.rs"]
mod pattern_model;
use pattern_model::{
    CornerBandCandidate, FailureCode, HeuristicFamily, MatchEvidence, MatchedSpan,
    NormalizedBBox, PatternSpec, RoiCandidateSidecar, RoiEvidence, SelectedTitleBlock, SourceTag,
    SpecHeadingDiagnostics, SpecHeadingSidecar, TemplateLifecycle, TitleBlockExtension,
    TitleBlockField, TitleBlockSidecar, CONFIDENCE_PASS,
};

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use conset_pdf_engine::{
    build_equipment_dataset,
    extractor::Extractor,
    extract_kv_pairs,
    extract_unit_tables,
    parse::parse_section_with_stats,
    segment::segment_transcript,
    DrawingSegmentEngine,
    ExtractedTable,
    SubmittalSegmentEngine,
};
use conset_pdf_ir::{AstNode, KvPair, OutlineTag};
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use pdfium_render::prelude::*;
use regex::Regex;

// ── CLI definition ────────────────────────────────────────────────────────────

/// Pattern development and corpus validation tool.
///
/// Inspect raw PDF geometry, test deterministic heuristic families against
/// single PDFs, and validate behavior across the fixture corpus.
#[derive(Debug, Parser)]
#[command(name = "pattern-dev", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Dump per-page geometry summary for a PDF.
    ///
    /// Loads the PDF via PDFium, prints page count, dimensions, span count,
    /// text quality score, and raster-vs-vector flag for each page.
    Inspect {
        /// Path to the PDF to inspect.
        pdf_path: PathBuf,

        /// Restrict output to a single zero-based page index.
        #[arg(long, value_name = "N")]
        page: Option<usize>,

        /// Output JSON instead of human-readable text.
        #[arg(long)]
        json: bool,

        /// Dump per-span text, normalized bbox, and mid_y for each page.
        /// Useful for diagnosing band thresholds on failing pages.
        #[arg(long)]
        spans: bool,
    },

    /// Apply one heuristic family to a single PDF.
    ///
    /// Prints per-page match / no-match results with confidence on stdout and
    /// writes deterministic overlays (PNG) and sidecars (JSON) to the output
    /// directory. Nothing is written when --dry-run is set.
    TestPattern {
        /// Path to the PDF to analyse.
        pdf_path: PathBuf,

        /// Heuristic family to apply.
        #[arg(long)]
        family: HeuristicFamily,

        /// Directory for overlay PNGs and sidecar JSONs.
        #[arg(long, value_name = "DIR")]
        output_dir: PathBuf,

        /// Validate arguments only; skip all processing and write no files.
        #[arg(long)]
        dry_run: bool,
    },

    /// Batch-validate heuristic families against explicit corpus tiers.
    ///
    /// Runs the full test-pattern loop over all PDFs in the requested tiers.
    /// Never touches the holdout set. Emits per-fixture artifacts and an
    /// aggregate validation-manifest.json plus corpus-report.json.
    ValidateCorpus {
        /// Corpus tiers to include (repeatable: --tier 1 --tier 2).
        ///
        /// Only values 1 and 2 are accepted; tier 3 and holdout are prohibited.
        #[arg(long = "tier", value_name = "N")]
        tiers: Vec<u8>,

        /// Root directory of the test corpus (defaults to tests/corpus).
        #[arg(long, value_name = "DIR", default_value = "tests/corpus")]
        corpus_dir: PathBuf,

        /// Directory for all output artifacts and manifests.
        #[arg(long, value_name = "DIR")]
        output_dir: PathBuf,

        /// Validate arguments and fixture inventory only; skip all processing.
        #[arg(long)]
        dry_run: bool,

        /// Engine pipeline stage to validate: "segment" or "parse".
        ///
        /// When absent, runs heuristic pattern-family validation.
        /// When set to "segment", extracts and segments every fixture and checks
        /// that `coverage_ratio >= 0.90` and `section_count >= 1`.
        /// When set to "parse", also runs the paragraph parser on every section
        /// and checks that the unclassified-node rate is `<= 0.01`.
        /// Emits `corpus-report.json` in the output directory.
        #[arg(long, value_name = "STAGE")]
        pipeline: Option<String>,
    },
}

// ── PDFium bootstrap (same discovery chain as classify_pdf.rs) ────────────────

fn load_pdfium() -> std::result::Result<Pdfium, String> {
    if let Ok(dir) = env::var("PDFIUM_LIB_PATH") {
        if let Ok(bindings) =
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir))
        {
            return Ok(Pdfium::new(bindings));
        }
    }

    if let Ok(workspace_root) = env::var("CARGO_WORKSPACE_DIR") {
        if let Ok(bindings) = Pdfium::bind_to_library(
            Pdfium::pdfium_platform_library_name_at_path(&workspace_root),
        ) {
            return Ok(Pdfium::new(bindings));
        }
    }

    if let Ok(cwd) = env::current_dir() {
        if let Ok(bindings) = Pdfium::bind_to_library(
            Pdfium::pdfium_platform_library_name_at_path(cwd.to_str().unwrap_or(".")),
        ) {
            return Ok(Pdfium::new(bindings));
        }
    }

    // Walk up from CARGO_MANIFEST_DIR to repo root (tools/ → repo root)
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let root = PathBuf::from(manifest_dir)
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    if let Ok(bindings) = Pdfium::bind_to_library(
        Pdfium::pdfium_platform_library_name_at_path(root.to_str().unwrap_or(".")),
    ) {
        return Ok(Pdfium::new(bindings));
    }

    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Ok(Pdfium::new(bindings));
    }

    Err(
        "PDFium library not found. Download pdfium.dll/dylib/so and place it in the project root \
         or set PDFIUM_LIB_PATH."
            .to_string(),
    )
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { pdf_path, page, json, spans } => {
            run_inspect(&pdf_path, page, json, spans)
        }
        Commands::TestPattern { pdf_path, family, output_dir, dry_run } => {
            run_test_pattern(&pdf_path, &family, &output_dir, dry_run)
        }
        Commands::ValidateCorpus { tiers, corpus_dir, output_dir, dry_run, pipeline } => {
            if let Some(stage) = pipeline {
                run_validate_corpus_pipeline(&tiers, &corpus_dir, &output_dir, dry_run, &stage)
            } else {
                run_validate_corpus(&tiers, &corpus_dir, &output_dir, dry_run)
            }
        }
    }
}

// ── inspect ───────────────────────────────────────────────────────────────────

// The serde_json::json! macro calls .unwrap() internally on infallible operations.
// This is a known false positive with the disallowed_methods lint.
#[allow(clippy::disallowed_methods)]
fn run_inspect(pdf_path: &Path, page_filter: Option<usize>, json: bool, spans: bool) -> Result<()> {
    if !pdf_path.exists() {
        bail!("PDF not found: {}", pdf_path.display());
    }

    let pdfium =
        load_pdfium().map_err(|e| anyhow::anyhow!("Failed to initialise PDFium: {e}"))?;

    let doc = pdfium
        .load_pdf_from_file(pdf_path, None)
        .with_context(|| format!("Failed to open PDF: {}", pdf_path.display()))?;

    let page_count: usize = doc.pages().len().into();

    let mut pages: Vec<serde_json::Value> = Vec::new();

    for idx in 0..page_count {
        if let Some(filter) = page_filter {
            if idx != filter {
                continue;
            }
        }

        let Ok(idx_u16) = u16::try_from(idx) else { continue };
        let Ok(page) = doc.pages().get(idx_u16) else { continue };

        let width_pts = page.width().value;
        let height_pts = page.height().value;

        // Count text objects (spans) via the objects API — consistent with
        // extract_page() in crates/pdf-extraction.
        let mut span_count: usize = 0;
        let mut char_count: usize = 0;
        let mut span_records: Vec<(String, f32, f32, f32, f32, f32)> = Vec::new(); // text, raw_x, raw_y, raw_w, raw_h, mid_y_norm
        let mut bounds_failures: usize = 0;
        for object in page.objects().iter() {
            if let Some(text_obj) = object.as_text_object() {
                let text = text_obj.text();
                if !text.trim().is_empty() {
                    span_count += 1;
                    char_count += text.chars().count();
                    if spans {
                        if let Ok(b) = object.bounds() {
                            let rx = b.left().value;
                            let ry = b.bottom().value;
                            let rw = (b.right().value - b.left().value).abs();
                            let rh = (b.top().value - b.bottom().value).abs();
                            let mid_y_norm = if height_pts > 0.0 {
                                let y_norm = 1.0 - (ry + rh) / height_pts;
                                (y_norm + (rh / height_pts) * 0.5).clamp(0.0, 1.0)
                            } else {
                                0.0
                            };
                            span_records.push((text, rx, ry, rw, rh, mid_y_norm));
                        } else {
                            bounds_failures += 1;
                        }
                    }
                }
            }
        }

        let text_extractable = span_count > 0;

        // Raster heuristic: low text density relative to page area.
        // Threshold will be refined with the full heuristic model in Phase D.
        let area_pts = width_pts * height_pts;
        let text_density = if area_pts > 0.0 { char_count as f32 / area_pts } else { 0.0 };
        let likely_raster = !text_extractable || text_density < 0.001;

        if spans {
            // Sort by mid_y_norm descending (bottom of page first) for easy reading.
            span_records.sort_by(|a, b| b.5.partial_cmp(&a.5).unwrap_or(std::cmp::Ordering::Equal));
            println!("  --- page {idx:04}  {width_pts:.0}x{height_pts:.0}pt  spans={span_count}  bounds_ok={}  bounds_fail={bounds_failures} ---",
                span_records.len());
            println!("  {:>8}  {:>8}  {:>8}  {:>8}  {:>8}  text", "raw_x", "raw_y", "raw_w", "raw_h", "mid_y");
            for (t, rx, ry, rw, rh, my) in &span_records {
                println!(
                    "  {:>8.1}  {:>8.1}  {:>8.1}  {:>8.1}  {:>8.4}  {:?}",
                    rx, ry, rw, rh, my,
                    if t.len() > 60 { &t[..60] } else { t.as_str() }
                );
            }
        }

        let entry = serde_json::json!({
            "page_index": idx,
            "width_pts": width_pts,
            "height_pts": height_pts,
            "span_count": span_count,
            "char_count": char_count,
            "text_extractable": text_extractable,
            "likely_raster": likely_raster
        });
        pages.push(entry);
    }

    if json {
        let out = serde_json::json!({
            "pdf_path": pdf_path.display().to_string(),
            "page_count": page_count,
            "pages": pages
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("PDF:        {}", pdf_path.display());
        println!("Pages:      {page_count}");
        for p in &pages {
            let idx = p["page_index"].as_u64().unwrap_or(0);
            let w = p["width_pts"].as_f64().unwrap_or(0.0);
            let h = p["height_pts"].as_f64().unwrap_or(0.0);
            let spans = p["span_count"].as_u64().unwrap_or(0);
            let chars = p["char_count"].as_u64().unwrap_or(0);
            let raster = p["likely_raster"].as_bool().unwrap_or(false);
            println!(
                "  page {:04}  {w:.1}x{h:.1}pt  spans={spans}  chars={chars}  raster={}",
                idx,
                if raster { "yes" } else { "no" }
            );
        }
    }

    Ok(())
}

// ── test-pattern ──────────────────────────────────────────────────────────────

fn run_test_pattern(
    pdf_path: &Path,
    family: &HeuristicFamily,
    output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    if !pdf_path.exists() {
        bail!("PDF not found: {}", pdf_path.display());
    }

    if dry_run {
        println!(
            "[dry-run] test-pattern: pdf={} family={} output_dir={}",
            pdf_path.display(),
            family.as_str(),
            output_dir.display()
        );
        if !family.is_runtime_ready() {
            println!(
                "[dry-run] NOTE: family '{}' is schema-only — sidecars will set \
                 source=schema-only; no detection logic runs until Phase 1.",
                family.as_str()
            );
        }
        return Ok(());
    }

    let pdf_stem = pdf_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let artifact_dir = output_dir.join(pdf_stem);
    std::fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("Failed to create output dir: {}", artifact_dir.display()))?;

    let pdfium =
        load_pdfium().map_err(|e| anyhow::anyhow!("Failed to initialise PDFium: {e}"))?;

    let doc = pdfium
        .load_pdf_from_file(pdf_path, None)
        .with_context(|| format!("Failed to open PDF: {}", pdf_path.display()))?;

    let page_count: usize = doc.pages().len().into();
    let family_str = family.as_str();
    let spec = PatternSpec::for_family(family);

    // Compile the regex once for the whole document.  None for presence-only families.
    let compiled_regex: Option<Regex> = spec
        .regex_pattern
        .as_deref()
        .map(Regex::new)
        .transpose()
        .with_context(|| format!("Invalid regex in PatternSpec for '{family_str}'"))?;

    let mut results: Vec<MatchEvidence> = Vec::with_capacity(page_count);

    for idx in 0..page_count {
        let Ok(idx_u16) = u16::try_from(idx) else { continue };
        let Ok(page) = doc.pages().get(idx_u16) else { continue };

        let (evidence, tba_ext_opt): (MatchEvidence, Option<TitleBlockExtension>) =
            if matches!(family, HeuristicFamily::TitleBlockAnchor) {
                let (ev, ext) =
                    detect_title_block_anchor(&page, &pdf_path.display().to_string(), idx);
                (ev, Some(ext))
            } else if !family.is_runtime_ready() {
                (MatchEvidence::placeholder(pdf_path.display().to_string(), idx, family), None)
            } else {
                let matched_spans = collect_page_matches(&page, &spec, compiled_regex.as_ref());
                let (confidence, failure_reason, branch_reason) =
                    score_matches(&matched_spans, compiled_regex.is_some());
                (
                    MatchEvidence {
                        schema_version: MatchEvidence::SCHEMA_VERSION,
                        pdf_path: pdf_path.display().to_string(),
                        page_index: idx,
                        family: family_str.to_owned(),
                        matched_spans,
                        confidence: Some(confidence),
                        failure_reason,
                        branch_reason,
                        source: SourceTag::Vector,
                        engine_version: env!("CARGO_PKG_VERSION"),
                        pattern_version: MatchEvidence::PATTERN_VERSION,
                    },
                    None,
                )
            };

        let status = if matches!(evidence.source, SourceTag::SchemaOnly) {
            "SKIP"
        } else if evidence.failure_reason.is_some() {
            "FAIL"
        } else if evidence.is_flagged() {
            "WARN"
        } else {
            "PASS"
        };
        let conf_str = evidence
            .confidence
            .map_or_else(|| "n/a".to_owned(), |c| format!("{c:.2}"));
        println!(
            "  [{status}] page {:04}  conf={conf_str}  matches={}",
            idx,
            evidence.matched_spans.len()
        );

        let sidecar_name = format!("page-{idx:04}-{family_str}.json");
        let sidecar_path = artifact_dir.join(&sidecar_name);
        let sidecar_json =
            serialize_sidecar_for_family(family, &evidence, tba_ext_opt.as_ref())
                .with_context(|| format!("Failed to serialise sidecar for page {idx}"))?;
        std::fs::write(&sidecar_path, sidecar_json)
            .with_context(|| format!("Failed to write sidecar: {}", sidecar_path.display()))?;

        // Phase F — render overlay PNG with match evidence annotated.
        let overlay_name = format!("page-{idx:04}-{family_str}.png");
        let overlay_path = artifact_dir.join(&overlay_name);
        render_overlay_png(&page, &evidence, &spec, &overlay_path)
            .with_context(|| format!("Overlay render failed for page {idx}"))?;

        results.push(evidence);
    }

    let n_skip = results
        .iter()
        .filter(|e| matches!(e.source, SourceTag::SchemaOnly))
        .count();
    let n_fail = results
        .iter()
        .filter(|e| !matches!(e.source, SourceTag::SchemaOnly) && e.is_failure())
        .count();
    let n_warn = results.iter().filter(|e| e.is_flagged()).count();
    let n_pass = results.iter().filter(|e| e.is_high_confidence()).count();

    println!(
        "\ntest-pattern complete: {page_count} pages  \
         PASS={n_pass} WARN={n_warn} FAIL={n_fail} SKIP={n_skip}"
    );
    println!(
        "  family={family_str}  overlays+sidecars={}",
        artifact_dir.display()
    );

    Ok(())
}

// ── Detection helpers ─────────────────────────────────────────────────────────

/// Y-axis tolerance (raw PDF points) for grouping characters onto the same baseline.
///
/// 6 pt covers sub-baseline-shift variation within a single line.
/// Typical inter-line leading on AEC spec documents is ≥ 12 pt, so this
/// threshold captures same-line chars without merging adjacent lines.
const CHAR_LINE_TOLERANCE_PTS: f32 = 6.0;

// ── Phase F overlay constants ─────────────────────────────────────────────────

/// Target pixel width for rasterizing pages in overlay PNG output.
/// Aspect ratio is maintained; height is calculated automatically from page dimensions.
const OVERLAY_TARGET_WIDTH_PX: i32 = 1400;

/// Overlay box line thickness in pixels (drawn as concentric inset hollow rects).
const OVERLAY_RECT_THICKNESS: i32 = 3;

/// PASS overlay colour — green: matched, confidence ≥ 0.95.
const COLOR_PASS: Rgba<u8> = Rgba([0u8, 192, 0, 230]);
/// WARN overlay colour — amber: matched, confidence 0.80–0.95 (flagged).
const COLOR_WARN: Rgba<u8> = Rgba([255u8, 160, 0, 230]);
/// FAIL overlay colour — red: any `FailureCode`.
const COLOR_FAIL: Rgba<u8> = Rgba([220u8, 0, 0, 230]);
/// Detection-band outline colour — mid-grey: always rendered, 1 px.
const COLOR_BAND: Rgba<u8> = Rgba([140u8, 140, 140, 160]);

/// Iterate all characters on `page` (including those inside Form XObjects),
/// filter by region band and optional regex, and return matched logical lines
/// as [`MatchedSpan`] values.
///
/// ## Why character-level, not object-level?
///
/// `page.objects().iter()` yields only **top-level** page objects.
/// PDFium's Form XObjects (`FPDF_PAGEOBJ_FORM`) — used by many PDF generators
/// for running headers/footers — appear as a single opaque node; their
/// interior text objects are never yielded, so footer text is silently missed.
///
/// `page.text().chars()` uses PDFium's `FPDFText_LoadPage` / `FPDFText_GetCharBox`
/// pipeline, which **descends into Form XObjects transparently** and returns
/// page-coordinate bounding boxes for every character.
///
/// ## Algorithm
///
/// 1. Collect all non-control, non-null characters whose bounding-box mid-Y
///    falls within `spec.region_band`.
/// 2. Sort by raw_y ascending then raw_x ascending (bottom → top, left → right
///    in PDF bottom-left-origin coords).
/// 3. Group into logical lines (characters within [`CHAR_LINE_TOLERANCE_PTS`]).
/// 4. Within each line, insert a space when the X-gap between adjacent characters
///    exceeds one-third of the average character width.
/// 5. For regex families: test the reconstructed line text; skip when no match.
///    For presence-only families (HeaderBand): all non-empty in-band lines qualify.
fn collect_page_matches<'d>(
    page: &PdfPage<'d>,
    spec: &PatternSpec,
    regex: Option<&Regex>,
) -> Vec<MatchedSpan> {
    let width_pts = page.width().value;
    let height_pts = page.height().value;

    let text_page = match page.text() {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };

    // Collect in-band characters with raw PDF coordinates.
    // raw_y = bottom of char bbox (PDF bottom-left origin).
    let mut band: Vec<(char, f32, f32, f32, f32)> = Vec::new(); // (char, x, y, w, h)
    for ch in text_page.chars().iter() {
        let Some(c) = ch.unicode_char() else { continue };
        // Skip null, BOM, and control characters (but keep printable ASCII and Unicode).
        if c == '\0' || c == '\u{FEFF}' || (c.is_control() && c != ' ') {
            continue;
        }
        let Ok(rect) = ch.loose_bounds() else { continue };
        let raw_x = rect.left().value;
        let raw_y = rect.bottom().value;
        let raw_w = (rect.right().value - rect.left().value).abs().max(0.001);
        let raw_h = (rect.top().value - rect.bottom().value).abs().max(0.001);
        let Some(nbbox) =
            NormalizedBBox::from_raw(raw_x, raw_y, raw_w, raw_h, width_pts, height_pts)
        else {
            continue;
        };
        if !spec.region_band.contains(nbbox.mid_y()) {
            continue;
        }
        band.push((c, raw_x, raw_y, raw_w, raw_h));
    }

    if band.is_empty() {
        return Vec::new();
    }

    // Sort by raw_y ascending (lowest y = nearest page bottom first),
    // then by raw_x ascending (left to right within a line).
    band.sort_by(|a, b| {
        a.2.partial_cmp(&b.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Group into logical lines.
    #[allow(clippy::type_complexity)]
    let mut char_lines: Vec<Vec<(char, f32, f32, f32, f32)>> = Vec::new();
    let mut cur: Vec<(char, f32, f32, f32, f32)> = Vec::new();
    let mut line_y = band[0].2;
    for item in band {
        if (item.2 - line_y).abs() <= CHAR_LINE_TOLERANCE_PTS {
            cur.push(item);
        } else {
            if !cur.is_empty() {
                char_lines.push(cur);
            }
            line_y = item.2;
            cur = vec![item];
        }
    }
    if !cur.is_empty() {
        char_lines.push(cur);
    }

    // Reconstruct line text and bboxes; apply pattern gate.
    let mut matched: Vec<MatchedSpan> = Vec::new();
    for line in &char_lines {
        // Re-sort each line by x (global sort is by y-then-x but chars on the
        // same baseline from different columns can interleave across the sort).
        let mut ls = line.clone();
        ls.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Build text: insert a space when the X-gap > ⅓ of the average char width.
        let mut line_text = String::new();
        for (i, (c, raw_x, _, raw_w, _)) in ls.iter().enumerate() {
            if i > 0 {
                let prev = &ls[i - 1];
                let gap = raw_x - (prev.1 + prev.3);
                let avg_w = (raw_w + prev.3) * 0.5;
                if gap > avg_w * 0.33 && !line_text.ends_with(' ') {
                    line_text.push(' ');
                }
            }
            line_text.push(*c);
        }
        let line_text = line_text.trim().to_owned();
        if line_text.is_empty() {
            continue;
        }

        // Union bbox for the line.
        let min_x = ls.iter().map(|(_, x, _, _, _)| *x).fold(f32::INFINITY, f32::min);
        let min_y = ls.iter().map(|(_, _, y, _, _)| *y).fold(f32::INFINITY, f32::min);
        let max_r = ls.iter().map(|(_, x, _, w, _)| x + w).fold(f32::NEG_INFINITY, f32::max);
        let max_t = ls.iter().map(|(_, _, y, _, h)| y + h).fold(f32::NEG_INFINITY, f32::max);
        let Some(nbbox) = NormalizedBBox::from_raw(
            min_x, min_y, (max_r - min_x).max(0.001), (max_t - min_y).max(0.001),
            width_pts, height_pts,
        ) else {
            continue;
        };

        let span_conf = if let Some(re) = regex {
            if !re.is_match(&line_text) {
                continue;
            }
            0.97_f32
        } else {
            0.95_f32
        };
        matched.push(MatchedSpan { text: line_text, bbox: nbbox, span_confidence: span_conf });
    }

    matched
}

/// Aggregate per-span matches into a page-level confidence score.
///
/// - **0 matches** → `NoMatch` hard fail, confidence 0.0.
/// - **2+ matches from a regex family** → penalise for ambiguity; fail as
///   `AmbiguousTie` when the penalised score drops below [`CONFIDENCE_PASS`].
/// - **1 match, or presence-only family** → return the best span confidence.
fn score_matches(
    matched: &[MatchedSpan],
    has_regex: bool,
) -> (f32, Option<FailureCode>, String) {
    if matched.is_empty() {
        return (
            0.0,
            Some(FailureCode::NoMatch),
            "no spans matched pattern in target band".to_owned(),
        );
    }
    if matched.len() >= 2 && has_regex {
        let n = matched.len() as f32;
        let avg: f32 = matched.iter().map(|s| s.span_confidence).sum::<f32>() / n;
        let conf = (avg * 0.85).clamp(0.0, 1.0);
        let reason = format!(
            "{} regex matches in band — ambiguity penalty applied",
            matched.len()
        );
        if conf < CONFIDENCE_PASS {
            return (conf, Some(FailureCode::AmbiguousTie), reason);
        }
        return (conf, None, reason);
    }
    let conf = matched
        .iter()
        .map(|s| s.span_confidence)
        .fold(0.0_f32, f32::max);
    (conf, None, "best match in target band".to_owned())
}

// ── Phase F overlay rendering ─────────────────────────────────────────────────

/// Draw a thick hollow rectangle on `img` by rendering `thickness` concentric inset
/// hollow rects. Coordinates are derived from `bbox` in normalised [0.0, 1.0] space.
///
/// Inset approach: each successive rect is 1 px smaller on all sides, giving a
/// solid-stroke appearance without alpha-blending artefacts from overlapping fills.
fn draw_thick_hollow_rect(
    img: &mut RgbaImage,
    bbox: &NormalizedBBox,
    img_w: u32,
    img_h: u32,
    color: Rgba<u8>,
    thickness: i32,
) {
    let x = (bbox.x * img_w as f32) as i32;
    let y = (bbox.y * img_h as f32) as i32;
    let w = ((bbox.width * img_w as f32) as u32).max(2);
    let h = ((bbox.height * img_h as f32) as u32).max(2);
    for t in 0..thickness {
        let rx = (x + t).max(0);
        let ry = (y + t).max(0);
        let rw = (w as i32 - 2 * t).max(1) as u32;
        let rh = (h as i32 - 2 * t).max(1) as u32;
        let rect = Rect::at(rx, ry).of_size(rw, rh);
        draw_hollow_rect_mut(img, rect, color);
    }
}

/// Render a per-page overlay PNG with match evidence annotated.
///
/// ## Rendering pipeline
///
/// 1. Rasterise the page at [`OVERLAY_TARGET_WIDTH_PX`] using PDFium (form data
///    included so Form XObject headers/footers appear visually).
/// 2. Draw a 1 px grey outline of the detection band so the searched region is
///    always visible regardless of outcome.
/// 3. For runtime-ready families: draw coloured bounding boxes using the locked
///    Phase F palette:
///    - **Green**  (`COLOR_PASS`) — matched, confidence ≥ 0.95
///    - **Amber**  (`COLOR_WARN`) — matched, confidence 0.80–0.95 (flagged)
///    - **Red**    (`COLOR_FAIL`) — failure (`NoMatch`, `LowConfidence`, etc.)
///      When no spans were matched (failure), a red box is drawn around the interior
///      of the detection band to signal "searched but found nothing."
/// 4. For schema-only families: emit the plain rendered page without match boxes.
/// 5. Save to `output_path` as PNG.
fn render_overlay_png(
    page: &PdfPage<'_>,
    evidence: &MatchEvidence,
    spec: &PatternSpec,
    output_path: &Path,
) -> Result<()> {
    let render_config = PdfRenderConfig::new()
        .set_target_width(OVERLAY_TARGET_WIDTH_PX)
        .render_form_data(true)
        .render_annotations(false);

    let bitmap = page
        .render_with_config(&render_config)
        .map_err(|e| anyhow::anyhow!("Page rasterisation failed: {e}"))?;

    let mut img = bitmap.as_image().into_rgba8();

    // Delegate all drawing to the shared helper (also used in validate-corpus
    // when overlay generation is enabled for individual fixtures).
    draw_evidence_on_img(&mut img, evidence, spec);

    img.save(output_path)
        .with_context(|| format!("Failed to save overlay: {}", output_path.display()))?;
    Ok(())
}

// ── Title-block anchor detection ────────────────────────────────────────────

/// Minimum width-to-height aspect ratio for a span to be treated as horizontal text.
///
/// Revit plot stamps ("FILE PATH:", "DATE/TIME:", "PLOT SCALE:") are rotated
/// 90° along the page margin.  Their raw bboxes have width ≪ height (ratio ≈ 0.08).
/// Filtering these out prevents them from being mistaken for title-block labels.
const MIN_SPAN_ASPECT_RATIO: f32 = 0.4;

/// Phrase keywords (case-insensitive substring) that signal a title-block cell label.
///
/// Phrases are used where a shorter prefix would also match value/role names:
///   - \"CHECKED BY\" not \"CHECK\"  → avoids matching the Revit role name \"Checker\"
///   - \"APPROVED BY\" not \"APPROV\" → avoids matching the Revit role name \"Approver\"
///
/// Rotated-text false positives (\"DATE/TIME:\", \"PLOT SCALE:\") are handled by
/// `MIN_SPAN_ASPECT_RATIO` before keywords are consulted.
const TITLE_BLOCK_KEYWORDS: &[&str] = &[
    "SHEET NO", "SHEET",
    "DRAWN BY", "DRAWN",
    "CHECKED BY", "CHECKED",
    "APPROVED BY", "APPROVED",
    "DATE", "SCALE", "REVISI",
    "PROJECT", "JOB NO",
    "ENGINEER", "ARCHITEC",
    "DESIGNED BY", "DESIGN",
    "SEAL", "TITLE",
];

/// Returns `true` when the normalised point (`x_norm`, `y_top_norm`) lies
/// inside `bbox`.  Both axes use the top-left origin convention.
fn point_in_bbox(bbox: &NormalizedBBox, x_norm: f32, y_top_norm: f32) -> bool {
    x_norm >= bbox.x
        && x_norm <= bbox.x + bbox.width
        && y_top_norm >= bbox.y
        && y_top_norm <= bbox.y + bbox.height
}

/// Runs the title-block-anchor corner-scoring heuristic on `page`.
///
/// Scores each of the four pre-seeded corner regions by the fraction of text
/// spans whose text contains a [`TITLE_BLOCK_KEYWORDS`] entry.
/// Text is collected via `page.objects()` (direct page objects, not Form
/// XObjects) — appropriate for DWG-style PDFs where title-block text lives
/// in regular page objects.
///
/// Returns scored [`MatchEvidence`] and a populated [`TitleBlockExtension`].
fn detect_title_block_anchor(
    page: &PdfPage<'_>,
    pdf_path_str: &str,
    page_index: usize,
) -> (MatchEvidence, TitleBlockExtension) {
    let page_w = page.width().value;
    let page_h = page.height().value;

    // Collect normalized span data: (x_left_norm, y_top_norm, w_norm, h_norm, text).
    let mut spans: Vec<(f32, f32, f32, f32, String)> = Vec::new();
    if page_w > 0.0 && page_h > 0.0 {
        for obj in page.objects().iter() {
            if let Some(text_obj) = obj.as_text_object() {
                let text = text_obj.text();
                if text.trim().is_empty() {
                    continue;
                }
                if let Ok(b) = obj.bounds() {
                    let rx = b.left().value;
                    let ry = b.bottom().value;
                    let rw = (b.right().value - b.left().value).abs().max(0.001);
                    let rh = (b.top().value - b.bottom().value).abs().max(0.001);
                    // Skip rotated text objects (Revit plot stamps, margin annotations).
                    if rw / rh < MIN_SPAN_ASPECT_RATIO {
                        continue;
                    }
                    let x_norm = (rx / page_w).clamp(0.0, 1.0);
                    let y_top = (1.0 - (ry + rh) / page_h).clamp(0.0, 1.0);
                    let w_norm = (rw / page_w).clamp(0.0, 1.0);
                    let h_norm = (rh / page_h).clamp(0.0, 1.0);
                    spans.push((x_norm, y_top, w_norm, h_norm, text));
                }
            }
        }
    }

    // Score each of the four pre-seeded corner regions.
    let scored: Vec<CornerBandCandidate> =
        TitleBlockExtension::schema_placeholder()
            .corner_candidates
            .into_iter()
            .map(|mut c| {
                let in_corner: Vec<_> = spans
                    .iter()
                    .filter(|(xn, yn, _, _, _)| point_in_bbox(&c.bbox, *xn, *yn))
                    .collect();
                let n_spans = in_corner.len();
                let n_kw = in_corner
                    .iter()
                    .filter(|(_, _, _, _, t)| {
                        let up = t.to_uppercase();
                        TITLE_BLOCK_KEYWORDS.iter().any(|k| up.contains(k))
                    })
                    .count();
                let area = (c.bbox.width * c.bbox.height).max(f32::EPSILON);
                c.cell_density = Some(n_spans as f32 / area);
                // Require at least 2 keyword hits to score non-zero.
                c.score = Some(if n_kw >= 2 && n_spans > 0 {
                    n_kw as f32 / n_spans as f32
                } else {
                    0.0
                });
                c
            })
            .collect();

    // Select the corner with the highest keyword ratio.
    let (best_idx, best_ratio) = scored
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.score.unwrap_or(0.0)))
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((0, 0.0));

    // Confidence: a keyword ratio of 0.15 maps to 1.0 (PASS).
    // 3 hits out of 20 spans ≈ 0.15 → confident enough.
    let confidence = (best_ratio / 0.15_f32).clamp(0.0, 1.0);

    let winning = if best_ratio > 0.0 {
        let c = &scored[best_idx];
        Some(SelectedTitleBlock {
            corner: c.corner.clone(),
            bbox: c.bbox.clone(),
            score: best_ratio,
        })
    } else {
        None
    };

    let (failure_reason, branch_reason) = if winning.is_none() {
        (
            Some(FailureCode::NoMatch),
            "no corner region contains title-block keywords".to_owned(),
        )
    } else if confidence < CONFIDENCE_PASS {
        (
            Some(FailureCode::LowConfidence),
            format!(
                "best corner {:?} kw_ratio={:.3} below threshold",
                scored[best_idx].corner, best_ratio,
            ),
        )
    } else {
        (
            None,
            format!(
                "title block in {:?} corner, kw_ratio={:.3}",
                scored[best_idx].corner, best_ratio,
            ),
        )
    };

    // Matched spans = keyword-matched spans in the winning corner (capped at 20).
    let matched_spans: Vec<MatchedSpan> = if winning.is_some() {
        spans
            .iter()
            .filter(|(xn, yn, _, _, t)| {
                point_in_bbox(&scored[best_idx].bbox, *xn, *yn) && {
                    let up = t.to_uppercase();
                    TITLE_BLOCK_KEYWORDS.iter().any(|k| up.contains(k))
                }
            })
            .take(20)
            .map(|(xn, yn, wn, hn, t)| MatchedSpan {
                text: t.clone(),
                bbox: NormalizedBBox { x: *xn, y: *yn, width: *wn, height: *hn },
                span_confidence: confidence,
            })
            .collect()
    } else {
        Vec::new()
    };

    let evidence = MatchEvidence {
        schema_version: MatchEvidence::SCHEMA_VERSION,
        pdf_path: pdf_path_str.to_owned(),
        page_index,
        family: HeuristicFamily::TitleBlockAnchor.as_str().to_owned(),
        matched_spans,
        confidence: Some(confidence),
        failure_reason,
        branch_reason,
        source: SourceTag::Vector,
        engine_version: env!("CARGO_PKG_VERSION"),
        pattern_version: MatchEvidence::PATTERN_VERSION,
    };

    // Extract label→value field pairs from the winning corner.
    // Use (f32,f32,f32,f32,&str) which is Copy so iterator chains stay simple.
    let field_candidates: Vec<TitleBlockField> = if winning.is_some() {
        let win_bbox = &scored[best_idx].bbox;

        let corner: Vec<(f32, f32, f32, f32, &str)> = spans
            .iter()
            .filter(|(xn, yn, _, _, _)| point_in_bbox(win_bbox, *xn, *yn))
            .map(|(xn, yn, wn, hn, t)| (*xn, *yn, *wn, *hn, t.as_str()))
            .collect();

        let labels: Vec<(f32, f32, f32, f32, &str)> = corner
            .iter()
            .copied()
            .filter(|(_, _, _, _, t)| {
                let up = t.to_uppercase();
                TITLE_BLOCK_KEYWORDS.iter().any(|k| up.contains(k))
            })
            .collect();

        let values: Vec<(f32, f32, f32, f32, &str)> = corner
            .iter()
            .copied()
            .filter(|(_, _, _, _, t)| {
                let up = t.to_uppercase();
                !TITLE_BLOCK_KEYWORDS.iter().any(|k| up.contains(k))
                    && !t.trim().is_empty()
            })
            .collect();

        labels
            .iter()
            .filter_map(|&(lx, ly, lw, lh, label_text)| {
                let lcx = lx + lw * 0.5;
                let lcy = ly + lh * 0.5;
                // Nearest value span within 0.12 normalised units of the label centre.
                values
                    .iter()
                    .map(|&(vx, vy, vw, vh, vt)| {
                        let dx = vx + vw * 0.5 - lcx;
                        let dy = vy + vh * 0.5 - lcy;
                        (dx * dx + dy * dy, vx, vy, vw, vh, vt)
                    })
                    .filter(|(d2, _, _, _, _, _)| *d2 < 0.12_f32 * 0.12_f32)
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|(_, vx, vy, vw, vh, vt)| TitleBlockField {
                        label: Some(label_text.trim().trim_end_matches(':').to_owned()),
                        label_bbox: Some(NormalizedBBox {
                            x: lx, y: ly, width: lw, height: lh,
                        }),
                        value_bbox: Some(NormalizedBBox {
                            x: vx, y: vy, width: vw, height: vh,
                        }),
                        extracted_value: Some(vt.trim().to_owned()),
                        field_score: None,
                    })
            })
            .collect()
    } else {
        Vec::new()
    };

    let ext = TitleBlockExtension {
        corner_candidates: scored,
        winning_candidate: winning,
        field_candidates,
        template_lifecycle: TemplateLifecycle::schema_placeholder(),
    };

    (evidence, ext)
}

// ── Phase G sidecar serialisation ────────────────────────────────────────────

/// Serialise one page's sidecar JSON, embedding family-specific extensions for
/// schema-only and runtime-detected families (Phase G+) alongside the locked
/// base schema.
///
/// `tba_ext` carries the runtime-scored [`TitleBlockExtension`] when called
/// from the `title-block-anchor` detection branch; `None` falls back to the
/// schema-only placeholder (useful for standalone serialisation tests).
fn serialize_sidecar_for_family(
    family: &HeuristicFamily,
    evidence: &MatchEvidence,
    tba_ext: Option<&TitleBlockExtension>,
) -> Result<String> {
    match family {
        HeuristicFamily::TitleBlockAnchor => {
            let ext =
                tba_ext.cloned().unwrap_or_else(TitleBlockExtension::schema_placeholder);
            let sidecar = TitleBlockSidecar { base: evidence.clone(), title_block: ext };
            Ok(serde_json::to_string_pretty(&sidecar)?)
        }
        HeuristicFamily::RoiCandidate => {
            let sidecar = RoiCandidateSidecar {
                base: evidence.clone(),
                roi_evidence: RoiEvidence::schema_placeholder(),
            };
            Ok(serde_json::to_string_pretty(&sidecar)?)
        }
        HeuristicFamily::SpecHeading => {
            let sidecar = SpecHeadingSidecar {
                base: evidence.clone(),
                heading_diagnostics: SpecHeadingDiagnostics::schema_placeholder(),
            };
            Ok(serde_json::to_string_pretty(&sidecar)?)
        }
        _ => Ok(serde_json::to_string_pretty(evidence)?),
    }
}

// ── validate-corpus ───────────────────────────────────────────────────────────

/// Minimum fraction of pages that must have a detected section ID (0.90 = 90%).
const CORPUS_MIN_COVERAGE: f64 = 0.90;

/// Maximum fraction of AST nodes that may be `Unclassified` (0.01 = 1%).
const CORPUS_MAX_UNCLASSIFIED: f64 = 0.01;

/// Minimum number of CSI sections a fixture must contain to pass.
const CORPUS_MIN_SECTION_COUNT: usize = 1;

/// Minimum number of sheets that must be detected in a DWG fixture to pass.
const DRAWING_MIN_SHEET_COUNT: usize = 1;

/// Minimum fraction of pages that must belong to a detected sheet.
/// (sheets_with_id_pages / total_pages)
const DRAWING_MIN_SHEET_COVERAGE: f64 = 0.80;

/// Minimum number of non-cover units a SUB fixture must produce to pass.
const SUBMITTAL_MIN_UNIT_COUNT: usize = 1;

/// Minimum total extraction records (KV + table rows) a SUB fixture must produce.
const SUBMITTAL_MIN_RECORD_COUNT: usize = 1;

/// Stable FNV-1a 64-bit content fingerprint.
///
/// Used for determinism checking between validate-corpus runs: if the same sidecar
/// content produces the same fingerprint, the run is deterministic.  No cryptographic
/// security is required here — stability between runs on the same machine is all
/// that matters.
fn fnv1a_64(data: &[u8]) -> u64 {
    let mut hash: u64 = 14_695_981_039_346_656_037;
    for &byte in data {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash
}

/// Draw match evidence bounding boxes onto an already-rendered image in place.
///
/// Extracted from [`render_overlay_png`] so that `validate-corpus` can render
/// each page *once* and clone it per family rather than re-rasterising.
///
/// Draws the detection band outline (always) and per-span colour boxes (when
/// the family is runtime-ready).
fn draw_evidence_on_img(
    img: &mut image::RgbaImage,
    evidence: &MatchEvidence,
    spec: &PatternSpec,
) {
    let img_w = img.width();
    let img_h = img.height();

    // Detection band outline (grey, 1 px).
    let band_outline = NormalizedBBox {
        x: 0.0,
        y: spec.region_band.y_min,
        width: 1.0,
        height: spec.region_band.y_max - spec.region_band.y_min,
    };
    draw_thick_hollow_rect(img, &band_outline, img_w, img_h, COLOR_BAND, 1);

    if matches!(evidence.source, SourceTag::SchemaOnly) {
        return;
    }

    let box_color = if evidence.failure_reason.is_some() {
        COLOR_FAIL
    } else if evidence.is_flagged() {
        COLOR_WARN
    } else {
        COLOR_PASS
    };

    if evidence.matched_spans.is_empty() {
        let inset = 0.005_f32;
        let band_h = spec.region_band.y_max - spec.region_band.y_min;
        let fail_band = NormalizedBBox {
            x: inset,
            y: spec.region_band.y_min + inset,
            width: (1.0_f32 - 2.0 * inset).max(0.01),
            height: (band_h - 2.0 * inset).max(0.005),
        };
        draw_thick_hollow_rect(img, &fail_band, img_w, img_h, box_color, OVERLAY_RECT_THICKNESS);
    } else {
        for span in &evidence.matched_spans {
            draw_thick_hollow_rect(img, &span.bbox, img_w, img_h, box_color, OVERLAY_RECT_THICKNESS);
        }
    }
}

/// Per-family page-level counters accumulated while processing one fixture.
#[derive(Default)]
struct FamilyPageCounts {
    n_pass: u32,
    n_warn: u32,
    n_fail: u32,
    n_skip: u32,
    /// XOR of FNV-1a fingerprints for all sidecar files (order-dependent).
    sidecar_fingerprint: u64,
}

impl FamilyPageCounts {
    fn record(&mut self, status: &str, sidecar_bytes: &[u8]) {
        match status {
            "PASS" => self.n_pass += 1,
            "WARN" => self.n_warn += 1,
            "FAIL" => self.n_fail += 1,
            _ => self.n_skip += 1,
        }
        // Accumulate fingerprint by folding sequentially (wrapping_add keeps
        // order sensitivity without needing alloc).
        self.sidecar_fingerprint =
            self.sidecar_fingerprint.wrapping_add(fnv1a_64(sidecar_bytes));
    }

    fn total_pages(&self) -> u32 {
        self.n_pass + self.n_warn + self.n_fail + self.n_skip
    }
}

// The serde_json::json! macro calls .unwrap() internally on infallible operations.
#[allow(clippy::disallowed_methods)]
fn run_validate_corpus(
    tiers: &[u8],
    corpus_dir: &Path,
    output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    // Enforce holdout prohibition. Only tiers 1 and 2 are allowed.
    for &tier in tiers {
        if tier == 0 || tier > 2 {
            bail!(
                "Only --tier 1 and --tier 2 are permitted. \
                 Tier 3 and holdout are prohibited by corpus policy."
            );
        }
    }

    if tiers.is_empty() {
        bail!("Specify at least one --tier (1 or 2).");
    }

    // Collect fixture paths from each requested tier.
    let mut fixtures: Vec<(u8, PathBuf)> = Vec::new();
    for &tier in tiers {
        let tier_dir = corpus_dir.join(format!("tier{tier}"));
        if !tier_dir.exists() {
            eprintln!("WARNING: tier directory not found, skipping: {}", tier_dir.display());
            continue;
        }
        let entries = std::fs::read_dir(&tier_dir)
            .with_context(|| format!("Failed to read tier dir: {}", tier_dir.display()))?;
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("pdf") {
                fixtures.push((tier, p));
            }
        }
    }

    // Deterministic iteration order (sort by path within each tier).
    fixtures.sort_by(|a, b| a.1.cmp(&b.1));

    if dry_run {
        println!(
            "[dry-run] validate-corpus: tiers={tiers:?} fixtures={} \
             corpus_dir={} output_dir={}",
            fixtures.len(),
            corpus_dir.display(),
            output_dir.display()
        );
        for (tier, f) in &fixtures {
            println!("  [dry-run] tier{tier}  {}", f.display());
        }
        return Ok(());
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    let pdfium =
        load_pdfium().map_err(|e| anyhow::anyhow!("Failed to initialise PDFium: {e}"))?;

    let all_families = HeuristicFamily::all();

    // Pre-compile regexes for every family once.
    let family_regexes: Vec<Option<regex::Regex>> = all_families
        .iter()
        .map(|f| {
            let spec = PatternSpec::for_family(f);
            spec.regex_pattern
                .as_deref()
                .map(regex::Regex::new)
                .transpose()
                .with_context(|| format!("Invalid regex for family '{}'", f.as_str()))
        })
        .collect::<Result<Vec<_>>>()?;

    let generated_at = Utc::now().to_rfc3339();
    let schema_version = "0.5.0";

    // Load the existing manifest to enable determinism comparison.
    let manifest_path = output_dir.join("validation-manifest.json");
    let prior_manifest: Option<serde_json::Value> = if manifest_path.exists() {
        std::fs::read_to_string(&manifest_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
    } else {
        None
    };

    // ── Per-fixture processing ────────────────────────────────────────────────

    let mut fixture_records: Vec<serde_json::Value> = Vec::new();

    // Aggregate totals across all fixtures: family_str → cumulative counts.
    let mut global_family_counts: HashMap<String, FamilyPageCounts> =
        all_families
            .iter()
            .map(|f| (f.as_str().to_owned(), FamilyPageCounts::default()))
            .collect();
    let mut global_total_pages: u64 = 0;
    let mut global_errored_fixtures: u32 = 0;

    for (fixture_num, (tier, fixture_path)) in fixtures.iter().enumerate() {
        let stem = fixture_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        println!(
            "\n[{}/{}] tier{}  {}",
            fixture_num + 1,
            fixtures.len(),
            tier,
            stem
        );

        let doc = match pdfium.load_pdf_from_file(fixture_path, None) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  ERROR: could not open PDF — skipping: {e}");
                global_errored_fixtures += 1;
                fixture_records.push(serde_json::json!({
                    "stem": stem,
                    "tier": tier,
                    "pdf_path": fixture_path.display().to_string(),
                    "error": e.to_string(),
                }));
                continue;
            }
        };

        let page_count: usize = doc.pages().len().into();
        global_total_pages += page_count as u64;

        // Initialise per-family counters for this fixture.
        let mut fixture_family_counts: HashMap<String, FamilyPageCounts> =
            all_families
                .iter()
                .map(|f| (f.as_str().to_owned(), FamilyPageCounts::default()))
                .collect();

        // Pre-create all family artifact directories.
        for family in all_families {
            let dir = output_dir.join(stem).join(family.as_str());
            std::fs::create_dir_all(&dir).with_context(|| {
                format!("Failed to create artifact dir: {}", dir.display())
            })?;
        }

        for idx in 0..page_count {
            let Ok(idx_u16) = u16::try_from(idx) else { continue };
            let Ok(page) = doc.pages().get(idx_u16) else { continue };

            for (fi, family) in all_families.iter().enumerate() {
                let family_str = family.as_str();
                let spec = PatternSpec::for_family(family);
                let compiled_regex = family_regexes[fi].as_ref();

                let (evidence, tba_ext_opt): (MatchEvidence, Option<TitleBlockExtension>) =
                    if matches!(family, HeuristicFamily::TitleBlockAnchor) {
                        let (ev, ext) = detect_title_block_anchor(
                            &page,
                            &fixture_path.display().to_string(),
                            idx,
                        );
                        (ev, Some(ext))
                    } else if !family.is_runtime_ready() {
                        (
                            MatchEvidence::placeholder(
                                fixture_path.display().to_string(),
                                idx,
                                family,
                            ),
                            None,
                        )
                    } else {
                        let matched_spans =
                            collect_page_matches(&page, &spec, compiled_regex);
                        let (confidence, failure_reason, branch_reason) =
                            score_matches(&matched_spans, compiled_regex.is_some());
                        (
                            MatchEvidence {
                                schema_version: MatchEvidence::SCHEMA_VERSION,
                                pdf_path: fixture_path.display().to_string(),
                                page_index: idx,
                                family: family_str.to_owned(),
                                matched_spans,
                                confidence: Some(confidence),
                                failure_reason,
                                branch_reason,
                                source: SourceTag::Vector,
                                engine_version: env!("CARGO_PKG_VERSION"),
                                pattern_version: MatchEvidence::PATTERN_VERSION,
                            },
                            None,
                        )
                    };

                let status = if matches!(evidence.source, SourceTag::SchemaOnly) {
                    "SKIP"
                } else if evidence.failure_reason.is_some() {
                    "FAIL"
                } else if evidence.is_flagged() {
                    "WARN"
                } else {
                    "PASS"
                };

                // Write sidecar JSON and fingerprint it.
                let sidecar_bytes = {
                    let json = serialize_sidecar_for_family(
                        family,
                        &evidence,
                        tba_ext_opt.as_ref(),
                    )
                    .with_context(|| {
                        format!("sidecar serialise failed page {idx} family {family_str}")
                    })?;
                    json.into_bytes()
                };
                let sidecar_path = output_dir
                    .join(stem)
                    .join(family_str)
                    .join(format!("page-{idx:04}-{family_str}.json"));
                std::fs::write(&sidecar_path, &sidecar_bytes).with_context(|| {
                    format!("Failed to write sidecar: {}", sidecar_path.display())
                })?;

                // Overlays are omitted in validate-corpus (sidecar-only mode).
                // Use test-pattern for per-file visual inspection.

                // Accumulate counters.
                fixture_family_counts
                    .get_mut(family_str)
                    .unwrap()
                    .record(status, &sidecar_bytes);
                global_family_counts
                    .get_mut(family_str)
                    .unwrap()
                    .record(status, &sidecar_bytes);
            }

            // Progress indicator every 50 pages.
            if idx > 0 && idx % 50 == 0 {
                println!("    ... page {idx}/{page_count}");
            }
        }

        // ── Determinism check for this fixture ───────────────────────────────

        let mut determinism_ok = true;
        let mut determinism_note: Option<String> = None;

        if let Some(ref prior) = prior_manifest {
            if let Some(prior_fixtures) = prior["fixtures"].as_array() {
                if let Some(prior_fx) = prior_fixtures
                    .iter()
                    .find(|v| v["stem"].as_str() == Some(stem))
                {
                    for family in all_families {
                        let fs = family.as_str();
                        let cur_fp = fixture_family_counts[fs].sidecar_fingerprint;
                        let prior_fp = prior_fx["families"][fs]["sidecar_fingerprint"]
                            .as_u64()
                            .unwrap_or(u64::MAX);
                        if cur_fp != prior_fp {
                            determinism_ok = false;
                            determinism_note = Some(format!(
                                "family '{fs}' fingerprint changed: prior={prior_fp:#018x} \
                                 current={cur_fp:#018x}"
                            ));
                            eprintln!(
                                "  DETERMINISM DRIFT — {stem} — {fs}: \
                                 prior={prior_fp:#018x} current={cur_fp:#018x}"
                            );
                            break;
                        }
                    }
                }
            }
        }

        // ── Build per-fixture record ─────────────────────────────────────────

        let families_json: serde_json::Value = {
            let mut map = serde_json::Map::new();
            for family in all_families {
                let fs = family.as_str();
                let c = &fixture_family_counts[fs];
                map.insert(
                    fs.to_owned(),
                    serde_json::json!({
                        "pass": c.n_pass,
                        "warn": c.n_warn,
                        "fail": c.n_fail,
                        "skip": c.n_skip,
                        "total_pages": c.total_pages(),
                        "sidecar_fingerprint": c.sidecar_fingerprint,
                        "artifact_dir": output_dir.join(stem).join(fs).display().to_string(),
                    }),
                );
            }
            serde_json::Value::Object(map)
        };

        let total_pass: u32 = all_families
            .iter()
            .map(|f| fixture_family_counts[f.as_str()].n_pass)
            .sum();
        let total_fail: u32 = all_families
            .iter()
            .map(|f| fixture_family_counts[f.as_str()].n_fail)
            .sum();

        println!(
            "  done  pages={page_count}  pass={total_pass}  fail={total_fail}  \
             det_ok={determinism_ok}"
        );

        fixture_records.push(serde_json::json!({
            "stem": stem,
            "tier": tier,
            "pdf_path": fixture_path.display().to_string(),
            "page_count": page_count,
            "families": families_json,
            "determinism_ok": determinism_ok,
            "determinism_note": determinism_note,
        }));
    }

    // ── Aggregate family totals ───────────────────────────────────────────────

    let by_family: serde_json::Value = {
        let mut map = serde_json::Map::new();
        for family in all_families {
            let fs = family.as_str();
            let g = &global_family_counts[fs];
            let total = g.total_pages();
            let pass_rate = if total > 0 { f64::from(g.n_pass) / f64::from(total) } else { 0.0 };
            map.insert(
                fs.to_owned(),
                serde_json::json!({
                    "pass": g.n_pass,
                    "warn": g.n_warn,
                    "fail": g.n_fail,
                    "skip": g.n_skip,
                    "total_pages": total,
                    "pass_rate": (pass_rate * 10_000.0).round() / 10_000.0,
                }),
            );
        }
        serde_json::Value::Object(map)
    };

    let determinism_regressions: u32 = fixture_records
        .iter()
        .filter(|v| v["determinism_ok"].as_bool() == Some(false))
        .count() as u32;

    // ── Write validation-manifest.json ────────────────────────────────────────

    let manifest = serde_json::json!({
        "schema_version": schema_version,
        "generated_at_utc": generated_at,
        "tiers": tiers,
        "families_tested": all_families.iter().map(|f| f.as_str()).collect::<Vec<_>>(),
        "fixture_count": fixtures.len(),
        "errored_fixtures": global_errored_fixtures,
        "total_pages": global_total_pages,
        "determinism_regressions": determinism_regressions,
        "fixtures": fixture_records,
    });

    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    // ── Write corpus-report.json ──────────────────────────────────────────────

    let report = serde_json::json!({
        "schema_version": schema_version,
        "generated_at_utc": generated_at,
        "tiers": tiers,
        "fixture_count": fixtures.len(),
        "errored_fixtures": global_errored_fixtures,
        "total_pages": global_total_pages,
        "determinism_regressions": determinism_regressions,
        "by_family": by_family,
    });

    let report_path = output_dir.join("corpus-report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)
        .with_context(|| format!("Failed to write report: {}", report_path.display()))?;

    // ── Summary ───────────────────────────────────────────────────────────────

    println!(
        "\nvalidate-corpus complete\n  \
         fixtures={} ({} errored)  pages={global_total_pages}  \
         det_regressions={determinism_regressions}",
        fixtures.len(),
        global_errored_fixtures,
    );
    println!("  manifest : {}", manifest_path.display());
    println!("  report   : {}", report_path.display());

    if determinism_regressions > 0 {
        eprintln!(
            "WARNING: {determinism_regressions} fixture(s) produced different output \
             from the previous run — check determinism_note in the manifest."
        );
    }

    Ok(())
}

// ── Pipeline corpus validation (8.3.A/B) ─────────────────────────────────────

/// Recursively walk an `AstNode` tree, returning `(total_nodes, unclassified_nodes)`.
fn count_ast_nodes(nodes: &[AstNode]) -> (usize, usize) {
    let mut total = 0usize;
    let mut unclassified = 0usize;
    for node in nodes {
        total += 1;
        if node.tag == OutlineTag::Unclassified {
            unclassified += 1;
        }
        let (ct, cu) = count_ast_nodes(&node.children);
        total += ct;
        unclassified += cu;
    }
    (total, unclassified)
}

/// Batch-run the engine pipeline (`segment` or `parse`) against all fixtures
/// in the requested tiers and emit a `corpus-report.json`.
///
/// Pass/fail thresholds:
/// - `CORPUS_MIN_COVERAGE` (0.90) — fraction of pages with a footer section ID
/// - `CORPUS_MIN_SECTION_COUNT` (1) — minimum sections detected
/// - `CORPUS_MAX_UNCLASSIFIED` (0.01) — unclassified AST node fraction (parse only)
// The serde_json::json! macro calls .unwrap() internally on infallible operations.
#[allow(clippy::disallowed_methods)]
fn run_validate_corpus_pipeline(
    tiers: &[u8],
    corpus_dir: &Path,
    output_dir: &Path,
    dry_run: bool,
    pipeline_stage: &str,
) -> Result<()> {
    match pipeline_stage {
        "segment" | "parse" => {}
        "drawing-segment" => {
            return run_validate_corpus_drawing(tiers, corpus_dir, output_dir, dry_run);
        }
        "submittal-extract" => {
            return run_validate_corpus_submittal(tiers, corpus_dir, output_dir, dry_run);
        }
        other => bail!("--pipeline must be 'segment', 'parse', 'drawing-segment', or 'submittal-extract', got: '{other}'"),
    }

    for &tier in tiers {
        if tier == 0 || tier > 2 {
            bail!(
                "Only --tier 1 and --tier 2 are permitted. \
                 Tier 3 and holdout are prohibited by corpus policy."
            );
        }
    }
    if tiers.is_empty() {
        bail!("Specify at least one --tier (1 or 2).");
    }

    let mut fixtures: Vec<(u8, PathBuf)> = Vec::new();
    for &tier in tiers {
        let tier_dir = corpus_dir.join(format!("tier{tier}"));
        if !tier_dir.exists() {
            eprintln!("WARNING: tier directory not found, skipping: {}", tier_dir.display());
            continue;
        }
        for entry in std::fs::read_dir(&tier_dir)
            .with_context(|| format!("Failed to read tier dir: {}", tier_dir.display()))?
            .flatten()
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("pdf") {
                fixtures.push((tier, p));
            }
        }
    }
    fixtures.sort_by(|a, b| a.1.cmp(&b.1));

    if dry_run {
        println!(
            "[dry-run] validate-corpus --pipeline {pipeline_stage}: tiers={tiers:?} \
             fixtures={} corpus_dir={} output_dir={}",
            fixtures.len(),
            corpus_dir.display(),
            output_dir.display()
        );
        for (tier, f) in &fixtures {
            println!("  [dry-run] tier{tier}  {}", f.display());
        }
        return Ok(());
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    let generated_at = Utc::now().to_rfc3339();
    let extractor = Extractor::new();

    let mut fixture_records: Vec<serde_json::Value> = Vec::new();
    let mut total_passed = 0u32;
    let mut total_failed = 0u32;
    let mut total_errored = 0u32;

    for (fixture_num, (tier, fixture_path)) in fixtures.iter().enumerate() {
        let stem = fixture_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        println!(
            "\n[{}/{}] tier{}  {}",
            fixture_num + 1,
            fixtures.len(),
            tier,
            stem
        );

        let path_str = match fixture_path.to_str() {
            Some(s) => s,
            None => {
                eprintln!("  ERROR: non-UTF-8 path — skipping");
                total_errored += 1;
                continue;
            }
        };

        // ── Extract ──────────────────────────────────────────────────────────
        let transcript = match extractor.extract(path_str) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("  ERROR: extraction failed — {e}");
                total_errored += 1;
                fixture_records.push(serde_json::json!({
                    "stem": stem,
                    "tier": tier,
                    "pdf_path": path_str,
                    "pass": false,
                    "error": e.to_string(),
                }));
                continue;
            }
        };

        // ── Segment ──────────────────────────────────────────────────────────
        let index = match segment_transcript(&transcript) {
            Ok(i) => i,
            Err(e) => {
                eprintln!("  ERROR: segmentation failed — {e}");
                total_errored += 1;
                fixture_records.push(serde_json::json!({
                    "stem": stem,
                    "tier": tier,
                    "pdf_path": path_str,
                    "pass": false,
                    "error": e.to_string(),
                }));
                continue;
            }
        };

        let section_count = index.sections.len();
        let coverage_ratio = index.coverage.coverage_ratio;
        let mut issues: Vec<String> = Vec::new();

        if section_count < CORPUS_MIN_SECTION_COUNT {
            issues.push(format!(
                "section_count {section_count} < min {CORPUS_MIN_SECTION_COUNT}"
            ));
        }
        if coverage_ratio < CORPUS_MIN_COVERAGE {
            issues.push(format!(
                "coverage_ratio {coverage_ratio:.4} < min {CORPUS_MIN_COVERAGE}"
            ));
        }

        // ── Parse (optional) ─────────────────────────────────────────────────
        let unclassified_ratio: Option<f64> = if pipeline_stage == "parse" {
            let mut total_nodes = 0usize;
            let mut total_unclassified = 0usize;
            for section_entry in &index.sections {
                let (section_ast, _stats) =
                    parse_section_with_stats(&transcript, section_entry);
                let (nodes, uncl) = count_ast_nodes(&section_ast.nodes);
                total_nodes += nodes;
                total_unclassified += uncl;
            }
            let ratio = if total_nodes == 0 {
                0.0
            } else {
                total_unclassified as f64 / total_nodes as f64
            };
            if ratio > CORPUS_MAX_UNCLASSIFIED {
                issues.push(format!(
                    "unclassified_ratio {ratio:.4} > max {CORPUS_MAX_UNCLASSIFIED} \
                     ({total_unclassified}/{total_nodes} nodes)"
                ));
            }
            println!(
                "  sections={section_count}  coverage={coverage_ratio:.3}  \
                 unclassified={ratio:.4}  nodes={total_nodes}  issues={}",
                issues.len()
            );
            Some(ratio)
        } else {
            println!(
                "  sections={section_count}  coverage={coverage_ratio:.3}  issues={}",
                issues.len()
            );
            None
        };

        let pass = issues.is_empty();
        if pass {
            total_passed += 1;
        } else {
            total_failed += 1;
            for issue in &issues {
                eprintln!("  FAIL: {issue}");
            }
        }

        let mut record = serde_json::json!({
            "stem": stem,
            "tier": tier,
            "pdf_path": path_str,
            "pass": pass,
            "section_count": section_count,
            "coverage_ratio": coverage_ratio,
            "issues": issues,
        });
        if let Some(ratio) = unclassified_ratio {
            record["unclassified_ratio"] = serde_json::json!(ratio);
        }
        fixture_records.push(record);
    }

    let total = total_passed + total_failed + total_errored;
    let pass_rate = if total == 0 {
        1.0f64
    } else {
        f64::from(total_passed) / f64::from(total)
    };

    let report = serde_json::json!({
        "schema_version": "0.1.0",
        "generated_at": generated_at,
        "pipeline": pipeline_stage,
        "thresholds": {
            "min_coverage": CORPUS_MIN_COVERAGE,
            "max_unclassified": CORPUS_MAX_UNCLASSIFIED,
            "min_section_count": CORPUS_MIN_SECTION_COUNT,
        },
        "aggregate": {
            "total": total,
            "passed": total_passed,
            "failed": total_failed,
            "errored": total_errored,
            "pass_rate": pass_rate,
        },
        "fixtures": fixture_records,
    });

    let report_path = output_dir.join("corpus-report.json");
    std::fs::write(
        &report_path,
        serde_json::to_string_pretty(&report)? + "\n",
    )
    .with_context(|| format!("Failed to write: {}", report_path.display()))?;

    println!(
        "\n[validate-corpus --pipeline {pipeline_stage}] \
         passed={total_passed} failed={total_failed} errored={total_errored} \
         total={total} pass_rate={:.1}%",
        pass_rate * 100.0
    );
    println!("  report: {}", report_path.display());

    Ok(())
}

// ── drawing-segment corpus validation (9.5.A) ─────────────────────────────────

/// Batch-run `DrawingSegmentEngine::build_index()` against all fixtures in the
/// requested tiers and emit a `corpus-report.json` to the output directory.
///
/// Pass/fail thresholds:
/// - `DRAWING_MIN_SHEET_COUNT` (1) — minimum sheets detected
/// - `DRAWING_MIN_SHEET_COVERAGE` (0.80) — fraction of pages belonging to a sheet
// #[allow(clippy::disallowed_methods)] — serde_json::json! uses .unwrap() internally.
#[allow(clippy::disallowed_methods)]
fn run_validate_corpus_drawing(
    tiers: &[u8],
    corpus_dir: &Path,
    output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    for &tier in tiers {
        if tier == 0 || tier > 2 {
            bail!(
                "Only --tier 1 and --tier 2 are permitted. \
                 Tier 3 and holdout are prohibited by corpus policy."
            );
        }
    }
    if tiers.is_empty() {
        bail!("Specify at least one --tier (1 or 2).");
    }

    let mut fixtures: Vec<(u8, PathBuf)> = Vec::new();
    for &tier in tiers {
        let tier_dir = corpus_dir.join(format!("tier{tier}"));
        if !tier_dir.exists() {
            eprintln!("WARNING: tier directory not found, skipping: {}", tier_dir.display());
            continue;
        }
        for entry in std::fs::read_dir(&tier_dir)
            .with_context(|| format!("Failed to read tier dir: {}", tier_dir.display()))?
            .flatten()
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("pdf") {
                fixtures.push((tier, p));
            }
        }
    }
    fixtures.sort_by(|a, b| a.1.cmp(&b.1));

    if dry_run {
        println!(
            "[dry-run] validate-corpus --pipeline drawing-segment: tiers={tiers:?} \
             fixtures={} corpus_dir={} output_dir={}",
            fixtures.len(),
            corpus_dir.display(),
            output_dir.display()
        );
        for (tier, f) in &fixtures {
            println!("  [dry-run] tier{tier}  {}", f.display());
        }
        return Ok(());
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    let generated_at = Utc::now().to_rfc3339();
    let extractor = Extractor::new();

    let mut fixture_records: Vec<serde_json::Value> = Vec::new();
    let mut total_passed = 0u32;
    let mut total_failed = 0u32;
    let mut total_errored = 0u32;

    for (fixture_num, (tier, fixture_path)) in fixtures.iter().enumerate() {
        let stem = fixture_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        println!(
            "\n[{}/{}] tier{}  {}",
            fixture_num + 1,
            fixtures.len(),
            tier,
            stem
        );

        let path_str = match fixture_path.to_str() {
            Some(s) => s,
            None => {
                eprintln!("  ERROR: non-UTF-8 path — skipping");
                total_errored += 1;
                continue;
            }
        };

        // ── Extract ──────────────────────────────────────────────────────────
        let transcript = match extractor.extract(path_str) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("  ERROR: extraction failed — {e}");
                total_errored += 1;
                fixture_records.push(serde_json::json!({
                    "stem": stem,
                    "tier": tier,
                    "pdf_path": path_str,
                    "pass": false,
                    "error": e.to_string(),
                }));
                continue;
            }
        };

        let total_pages = transcript.pages().len();

        // ── Drawing segment ───────────────────────────────────────────────────
        let drawing_index = DrawingSegmentEngine::build_index(&transcript);
        let sheet_count = drawing_index.sheet_count;

        // Sheet coverage: count pages that belong to at least one detected sheet.
        let covered_pages: usize = drawing_index
            .sheets
            .iter()
            .map(|s| s.page_count)
            .sum();
        let sheet_coverage = if total_pages == 0 {
            0.0f64
        } else {
            covered_pages as f64 / total_pages as f64
        };

        let mut issues: Vec<String> = Vec::new();
        if sheet_count < DRAWING_MIN_SHEET_COUNT {
            issues.push(format!(
                "sheet_count {sheet_count} < min {DRAWING_MIN_SHEET_COUNT}"
            ));
        }
        if sheet_coverage < DRAWING_MIN_SHEET_COVERAGE {
            issues.push(format!(
                "sheet_coverage {sheet_coverage:.4} < min {DRAWING_MIN_SHEET_COVERAGE}"
            ));
        }

        let pass = issues.is_empty();
        if pass {
            total_passed += 1;
        } else {
            total_failed += 1;
            for issue in &issues {
                eprintln!("  FAIL: {issue}");
            }
        }

        println!(
            "  sheets={sheet_count}  coverage={sheet_coverage:.3}  \
             total_pages={total_pages}  issues={}",
            issues.len()
        );

        fixture_records.push(serde_json::json!({
            "stem": stem,
            "tier": tier,
            "pdf_path": path_str,
            "pass": pass,
            "sheet_count": sheet_count,
            "total_pages": total_pages,
            "covered_pages": covered_pages,
            "sheet_coverage": sheet_coverage,
            "issues": issues,
        }));
    }

    let total = total_passed + total_failed + total_errored;
    let pass_rate = if total == 0 {
        1.0f64
    } else {
        f64::from(total_passed) / f64::from(total)
    };

    let report = serde_json::json!({
        "schema_version": "0.1.0",
        "generated_at": generated_at,
        "pipeline": "drawing-segment",
        "thresholds": {
            "min_sheet_count": DRAWING_MIN_SHEET_COUNT,
            "min_sheet_coverage": DRAWING_MIN_SHEET_COVERAGE,
        },
        "aggregate": {
            "total": total,
            "passed": total_passed,
            "failed": total_failed,
            "errored": total_errored,
            "pass_rate": pass_rate,
        },
        "fixtures": fixture_records,
    });

    let report_path = output_dir.join("corpus-report.json");
    std::fs::write(
        &report_path,
        serde_json::to_string_pretty(&report)? + "\n",
    )
    .with_context(|| format!("Failed to write: {}", report_path.display()))?;

    println!(
        "\n[validate-corpus --pipeline drawing-segment] \
         passed={total_passed} failed={total_failed} errored={total_errored} \
         total={total} pass_rate={:.1}%",
        pass_rate * 100.0
    );
    println!("  report: {}", report_path.display());

    Ok(())
}

// ── submittal-extract corpus validation (10.5.A) ──────────────────────────────

/// Batch-run the submittal pipeline against all `SUB_*.pdf` fixtures in the
/// requested tiers and emit a `sub-corpus-report.json`.
///
/// Pass/fail thresholds:
/// - `SUBMITTAL_MIN_UNIT_COUNT` (1) — minimum non-cover units detected
/// - `SUBMITTAL_MIN_RECORD_COUNT` (1) — minimum KV + table rows extracted
// The serde_json::json! macro calls .unwrap() internally on infallible operations.
#[allow(clippy::disallowed_methods)]
fn run_validate_corpus_submittal(
    tiers: &[u8],
    corpus_dir: &Path,
    output_dir: &Path,
    dry_run: bool,
) -> Result<()> {
    for &tier in tiers {
        if tier == 0 || tier > 2 {
            bail!(
                "Only --tier 1 and --tier 2 are permitted. \
                 Tier 3 and holdout are prohibited by corpus policy."
            );
        }
    }
    if tiers.is_empty() {
        bail!("Specify at least one --tier (1 or 2).");
    }

    // Collect SUB_*.pdf fixtures only.
    let mut fixtures: Vec<(u8, PathBuf)> = Vec::new();
    for &tier in tiers {
        let tier_dir = corpus_dir.join(format!("tier{tier}"));
        if !tier_dir.exists() {
            eprintln!("WARNING: tier directory not found, skipping: {}", tier_dir.display());
            continue;
        }
        for entry in std::fs::read_dir(&tier_dir)
            .with_context(|| format!("Failed to read tier dir: {}", tier_dir.display()))?
            .flatten()
        {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) != Some("pdf") {
                continue;
            }
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                if stem.to_ascii_uppercase().starts_with("SUB_") {
                    fixtures.push((tier, p));
                }
            }
        }
    }
    fixtures.sort_by(|a, b| a.1.cmp(&b.1));

    if dry_run {
        println!(
            "[dry-run] validate-corpus --pipeline submittal-extract: tiers={tiers:?} \
             fixtures={} corpus_dir={} output_dir={}",
            fixtures.len(),
            corpus_dir.display(),
            output_dir.display()
        );
        for (tier, f) in &fixtures {
            println!("  [dry-run] tier{tier}  {}", f.display());
        }
        return Ok(());
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    let generated_at = Utc::now().to_rfc3339();
    let extractor = Extractor::new();

    let mut fixture_records: Vec<serde_json::Value> = Vec::new();
    let mut total_passed = 0u32;
    let mut total_failed = 0u32;
    let mut total_errored = 0u32;

    for (fixture_num, (tier, fixture_path)) in fixtures.iter().enumerate() {
        let stem = fixture_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        println!(
            "\n[{}/{}] tier{}  {}",
            fixture_num + 1,
            fixtures.len(),
            tier,
            stem
        );

        let path_str = match fixture_path.to_str() {
            Some(s) => s,
            None => {
                eprintln!("  ERROR: non-UTF-8 path — skipping");
                total_errored += 1;
                continue;
            }
        };

        // ── Extract ──────────────────────────────────────────────────────────
        let transcript = match extractor.extract(path_str) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("  ERROR: extraction failed — {e}");
                total_errored += 1;
                fixture_records.push(serde_json::json!({
                    "stem": stem,
                    "tier": tier,
                    "pdf_path": path_str,
                    "pass": false,
                    "error": e.to_string(),
                }));
                continue;
            }
        };

        let total_pages = transcript.pages().len();

        // ── Submittal segment ─────────────────────────────────────────────────
        let packet_name = fixture_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("UNKNOWN");
        let submittal_index = SubmittalSegmentEngine::build_index(&transcript, packet_name);
        let unit_count = submittal_index.units.iter().filter(|u| !u.is_cover).count();

        // ── Per-unit extraction ───────────────────────────────────────────────
        let pages = transcript.pages();
        let mut kv_by_unit: Vec<(usize, Vec<KvPair>)> = Vec::new();
        let mut tables_by_unit: Vec<(usize, Vec<ExtractedTable>)> = Vec::new();

        for (unit_idx, unit) in submittal_index.units.iter().enumerate() {
            if unit.is_cover {
                continue;
            }
            let start = unit.start_page;
            let end = unit.end_page.min(total_pages.saturating_sub(1));
            let unit_pages: Vec<&_> = pages[start..=end].iter().collect();
            kv_by_unit.push((unit_idx, extract_kv_pairs(&unit_pages)));
            tables_by_unit.push((unit_idx, extract_unit_tables(&unit_pages, unit)));
        }

        // ── Assemble dataset ──────────────────────────────────────────────────
        let dataset = build_equipment_dataset(&submittal_index, &tables_by_unit, &kv_by_unit);
        let record_count = dataset.record_count;

        let mut issues: Vec<String> = Vec::new();
        if unit_count < SUBMITTAL_MIN_UNIT_COUNT {
            issues.push(format!(
                "unit_count {unit_count} < min {SUBMITTAL_MIN_UNIT_COUNT}"
            ));
        }
        if record_count < SUBMITTAL_MIN_RECORD_COUNT {
            issues.push(format!(
                "record_count {record_count} < min {SUBMITTAL_MIN_RECORD_COUNT}"
            ));
        }

        let pass = issues.is_empty();
        if pass {
            total_passed += 1;
        } else {
            total_failed += 1;
            for issue in &issues {
                eprintln!("  FAIL: {issue}");
            }
        }

        println!(
            "  units={unit_count}  records={record_count}  pages={total_pages}  issues={}",
            issues.len()
        );

        fixture_records.push(serde_json::json!({
            "stem": stem,
            "tier": tier,
            "pdf_path": path_str,
            "pass": pass,
            "unit_count": unit_count,
            "record_count": record_count,
            "total_pages": total_pages,
            "issues": issues,
        }));
    }

    let total = total_passed + total_failed + total_errored;
    let pass_rate = if total == 0 {
        1.0f64
    } else {
        f64::from(total_passed) / f64::from(total)
    };

    let report = serde_json::json!({
        "schema_version": "0.1.0",
        "generated_at": generated_at,
        "pipeline": "submittal-extract",
        "thresholds": {
            "min_unit_count": SUBMITTAL_MIN_UNIT_COUNT,
            "min_record_count": SUBMITTAL_MIN_RECORD_COUNT,
        },
        "aggregate": {
            "total": total,
            "passed": total_passed,
            "failed": total_failed,
            "errored": total_errored,
            "pass_rate": pass_rate,
        },
        "fixtures": fixture_records,
    });

    let report_path = output_dir.join("sub-corpus-report.json");
    std::fs::write(
        &report_path,
        serde_json::to_string_pretty(&report)? + "\n",
    )
    .with_context(|| format!("Failed to write: {}", report_path.display()))?;

    println!(
        "\n[validate-corpus --pipeline submittal-extract] \
         passed={total_passed} failed={total_failed} errored={total_errored} \
         total={total} pass_rate={:.1}%",
        pass_rate * 100.0
    );
    println!("  report: {}", report_path.display());

    Ok(())
}



#[cfg(test)]
mod tests {
    use super::*;

    // ── fnv1a_64 ──────────────────────────────────────────────────────────────

    #[test]
    fn fnv1a_64_empty_returns_offset_basis() {
        // FNV-1a 64-bit: no bytes processed → hash is the FNV offset basis.
        assert_eq!(fnv1a_64(&[]), 14_695_981_039_346_656_037u64);
    }

    #[test]
    fn fnv1a_64_is_deterministic() {
        // Same bytes on two separate calls must produce the same hash.
        assert_eq!(fnv1a_64(b"page-0000-footer-section-id.json"), fnv1a_64(b"page-0000-footer-section-id.json"));
    }

    #[test]
    fn fnv1a_64_distinct_inputs_produce_distinct_hashes() {
        // Different sidecar bytes must distinguish themselves.
        assert_ne!(fnv1a_64(b"page-0000-footer-section-id.json"), fnv1a_64(b"page-0001-footer-section-id.json"));
    }

    // ── FamilyPageCounts ──────────────────────────────────────────────────────

    #[test]
    fn family_page_counts_accumulates_all_statuses() {
        let mut c = FamilyPageCounts::default();
        c.record("PASS", b"p");
        c.record("WARN", b"w");
        c.record("FAIL", b"f");
        c.record("SKIP", b"s");
        assert_eq!(c.n_pass, 1);
        assert_eq!(c.n_warn, 1);
        assert_eq!(c.n_fail, 1);
        assert_eq!(c.n_skip, 1);
    }

    #[test]
    fn family_page_counts_total_pages_is_sum_of_all_buckets() {
        let mut c = FamilyPageCounts::default();
        for _ in 0..3 { c.record("PASS", b"x"); }
        for _ in 0..2 { c.record("WARN", b"x"); }
        for _ in 0..4 { c.record("FAIL", b"x"); }
        for _ in 0..1 { c.record("SKIP", b"x"); }
        assert_eq!(c.total_pages(), 10);
    }

    #[test]
    fn family_page_counts_fingerprint_nonzero_for_nonempty_input() {
        let mut c = FamilyPageCounts::default();
        c.record("PASS", b"some sidecar content");
        assert_ne!(c.sidecar_fingerprint, 0);
    }

    #[test]
    fn family_page_counts_fingerprint_matches_for_identical_sequences() {
        // Two independent runs processing the same bytes must agree on fingerprint.
        let inputs: &[&[u8]] = &[b"sidecar-0", b"sidecar-1", b"sidecar-2"];
        let mut c1 = FamilyPageCounts::default();
        let mut c2 = FamilyPageCounts::default();
        for &bytes in inputs {
            c1.record("PASS", bytes);
            c2.record("PASS", bytes);
        }
        assert_eq!(c1.sidecar_fingerprint, c2.sidecar_fingerprint);
    }

    #[test]
    fn family_page_counts_fingerprint_differs_for_different_content() {
        let mut c1 = FamilyPageCounts::default();
        c1.record("PASS", b"content-A");
        let mut c2 = FamilyPageCounts::default();
        c2.record("PASS", b"content-B");
        assert_ne!(c1.sidecar_fingerprint, c2.sidecar_fingerprint);
    }

    // ── score_matches ─────────────────────────────────────────────────────────

    fn make_span(conf: f32) -> MatchedSpan {
        MatchedSpan {
            text: "23 82 16".to_owned(),
            bbox: NormalizedBBox { x: 0.05, y: 0.90, width: 0.20, height: 0.02 },
            span_confidence: conf,
        }
    }

    #[test]
    fn score_matches_empty_spans_returns_no_match_fail() {
        let (conf, code, branch) = score_matches(&[], true);
        assert_eq!(conf, 0.0);
        assert_eq!(code, Some(FailureCode::NoMatch));
        assert!(branch.contains("no spans"));
    }

    #[test]
    fn score_matches_single_high_confidence_span_passes() {
        let (conf, code, branch) = score_matches(&[make_span(0.97)], true);
        assert!((conf - 0.97).abs() < 1e-5);
        assert!(code.is_none());
        assert!(branch.contains("best match"));
    }

    #[test]
    fn score_matches_geometric_family_single_span_no_regex() {
        // Header-band has no regex; single span in band → best match.
        let (conf, code, _) = score_matches(&[make_span(0.95)], false);
        assert!((conf - 0.95).abs() < 1e-5);
        assert!(code.is_none());
    }

    #[test]
    fn score_matches_two_spans_regex_applies_ambiguity_penalty() {
        // Two spans at 1.0 each → avg=1.0, conf=1.0*0.85=0.85 → passes (≥0.80).
        let spans = vec![make_span(1.0), make_span(1.0)];
        let (conf, code, branch) = score_matches(&spans, true);
        assert!((conf - 0.85).abs() < 1e-5);
        assert!(code.is_none(), "conf 0.85 should pass but got {code:?}");
        assert!(branch.contains("ambiguity penalty"));
    }

    #[test]
    fn score_matches_two_spans_low_confidence_is_ambiguous_tie() {
        // Two spans at 0.5 each → avg=0.5, conf=0.5*0.85=0.425 < 0.80 → AmbiguousTie.
        let spans = vec![make_span(0.5), make_span(0.5)];
        let (conf, code, _) = score_matches(&spans, true);
        assert!(conf < CONFIDENCE_PASS);
        assert_eq!(code, Some(FailureCode::AmbiguousTie));
    }

    // ── Sidecar artifact naming ───────────────────────────────────────────────

    /// Locks the sidecar filename format: `page-{4-digit-zero-padded-index}-{family}.json`.
    ///
    /// This format is used by `run_validate_corpus` and `run_test_pattern` to
    /// write and locate sidecar artifacts. Changing any part of the format is a
    /// breaking change for all consumers of the artifact tree.
    #[test]
    fn sidecar_filename_is_zero_padded_4_digit_index() {
        assert_eq!(format!("page-{:04}-{}.json", 0, "footer-section-id"),
                   "page-0000-footer-section-id.json");
        assert_eq!(format!("page-{:04}-{}.json", 42, "header-band"),
                   "page-0042-header-band.json");
        assert_eq!(format!("page-{:04}-{}.json", 9999, "page-counter"),
                   "page-9999-page-counter.json");
        // Beyond 4 digits: format expands naturally without truncation.
        assert_eq!(format!("page-{:04}-{}.json", 10000, "title-block-anchor"),
                   "page-10000-title-block-anchor.json");
    }

    // ── Corpus-report manifest schema ─────────────────────────────────────────

    /// Verifies the Phase I smoke corpus-report has required top-level fields
    /// and the correct set of families. Skips gracefully when smoke artifacts
    /// are not present (e.g. on a clean clone).
    #[test]
    fn corpus_report_has_required_top_level_fields() {
        let report_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join("audit_output/phase-i-smoke/corpus-report.json");

        if !report_path.exists() {
            return; // Smoke artifacts are not committed; skip silently.
        }

        let content = std::fs::read_to_string(&report_path).expect("read corpus-report.json");
        let v: serde_json::Value = serde_json::from_str(&content).expect("parse corpus-report.json");

        assert_eq!(v["schema_version"], "0.5.0");
        assert!(v["fixture_count"].as_u64().is_some_and(|n| n > 0));
        assert!(v["total_pages"].as_u64().is_some_and(|n| n > 0));
        assert_eq!(v["determinism_regressions"].as_u64(), Some(0));
        assert!(v["by_family"].is_object());

        for family in HeuristicFamily::all() {
            let fs = family.as_str();
            let fv = &v["by_family"][fs];
            assert!(fv.is_object(), "family '{fs}' missing from corpus-report by_family");
            assert!(fv["pass"].is_number(),   "{fs}: pass must be a number");
            assert!(fv["fail"].is_number(),   "{fs}: fail must be a number");
            assert!(fv["total_pages"].is_number(), "{fs}: total_pages must be a number");
            assert!(fv["pass_rate"].is_number(),   "{fs}: pass_rate must be a number");
        }
    }

    /// Verifies the validation-manifest has required top-level fields and that
    /// each fixture entry carries per-family sidecar_fingerprints.
    #[test]
    fn validation_manifest_has_required_fields_and_fingerprints() {
        let manifest_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("workspace root")
            .join("audit_output/phase-i-smoke/validation-manifest.json");

        if !manifest_path.exists() {
            return; // Smoke artifacts not committed; skip silently.
        }

        let content = std::fs::read_to_string(&manifest_path).expect("read validation-manifest.json");
        let v: serde_json::Value = serde_json::from_str(&content).expect("parse validation-manifest.json");

        assert_eq!(v["schema_version"], "0.5.0");
        assert!(v["families_tested"].as_array().is_some_and(|a| a.len() == 6));
        assert_eq!(v["determinism_regressions"].as_u64(), Some(0));

        let fixtures = v["fixtures"].as_array().expect("fixtures must be array");
        assert!(!fixtures.is_empty());

        // Spot-check the first fixture: every family must have a u64 sidecar_fingerprint.
        let first = &fixtures[0];
        assert!(first["stem"].is_string());
        assert!(first["page_count"].as_u64().is_some_and(|n| n > 0));
        assert_eq!(first["determinism_ok"], true);

        for family in HeuristicFamily::all() {
            let fs = family.as_str();
            assert!(
                first["families"][fs]["sidecar_fingerprint"].is_number(),
                "fixture[0] family '{fs}' missing sidecar_fingerprint"
            );
        }
    }
}
