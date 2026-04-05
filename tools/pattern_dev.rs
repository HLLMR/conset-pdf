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
    FailureCode, HeuristicFamily, MatchEvidence, MatchedSpan, NormalizedBBox, PatternSpec,
    SourceTag, CONFIDENCE_PASS,
};

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
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
        Commands::ValidateCorpus { tiers, corpus_dir, output_dir, dry_run } => {
            run_validate_corpus(&tiers, &corpus_dir, &output_dir, dry_run)
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

        let evidence = if !family.is_runtime_ready() {
            MatchEvidence::placeholder(pdf_path.display().to_string(), idx, family)
        } else {
            let matched_spans = collect_page_matches(&page, &spec, compiled_regex.as_ref());
            let (confidence, failure_reason, branch_reason) =
                score_matches(&matched_spans, compiled_regex.is_some());
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
            }
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
        std::fs::write(&sidecar_path, serde_json::to_string_pretty(&evidence)?)
            .with_context(|| format!("Failed to write sidecar: {}", sidecar_path.display()))?;

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
    println!("  family={family_str}  artifacts={}", artifact_dir.display());

    Ok(())
}

// ── Detection helpers ─────────────────────────────────────────────────────────

/// Y-axis tolerance (raw PDF points) for grouping characters onto the same baseline.
///
/// 6 pt covers sub-baseline-shift variation within a single line.
/// Typical inter-line leading on AEC spec documents is ≥ 12 pt, so this
/// threshold captures same-line chars without merging adjacent lines.
const CHAR_LINE_TOLERANCE_PTS: f32 = 6.0;

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
        let raw_x = rect.left.value;
        let raw_y = rect.bottom.value;
        let raw_w = (rect.right.value - rect.left.value).abs().max(0.001);
        let raw_h = (rect.top.value - rect.bottom.value).abs().max(0.001);
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
                if gap > avg_w * 0.33 && line_text.chars().last() != Some(' ') {
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

// ── validate-corpus ───────────────────────────────────────────────────────────

// The serde_json::json! macro calls .unwrap() internally on infallible operations.
// This stub is replaced with typed serialization in Phase I.
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
    let mut fixtures: Vec<PathBuf> = Vec::new();
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
                fixtures.push(p);
            }
        }
    }

    // Deterministic iteration order (sort by path).
    fixtures.sort();

    if dry_run {
        println!(
            "[dry-run] validate-corpus: tiers={:?} fixtures={} corpus_dir={} output_dir={}",
            tiers,
            fixtures.len(),
            corpus_dir.display(),
            output_dir.display()
        );
        for f in &fixtures {
            println!("  [dry-run] would process: {}", f.display());
        }
        return Ok(());
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("Failed to create output dir: {}", output_dir.display()))?;

    // Phase I will run real detection per fixture here.
    // This skeleton emits a stub validation-manifest.json with correct schema.
    let mut fixture_records: Vec<serde_json::Value> = Vec::new();
    for f in &fixtures {
        let stem = f.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
        fixture_records.push(serde_json::json!({
            "pdf_path": f.display().to_string(),
            "stem": stem,
            "status": "NOT_IMPLEMENTED",
            "note": "detection not yet implemented — Phase I"
        }));
        println!("  [stub] {}", f.display());
    }

    let manifest = serde_json::json!({
        "schema_version": "0.5.0",
        "tiers": tiers,
        "fixture_count": fixtures.len(),
        "fixtures": fixture_records,
        "note": "stub manifest — real per-fixture runs implemented in Phase I"
    });

    let manifest_path = output_dir.join("validation-manifest.json");
    std::fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    let report = serde_json::json!({
        "schema_version": "0.5.0",
        "fixture_count": fixtures.len(),
        "passed": 0,
        "failed": 0,
        "pending": fixtures.len(),
        "note": "stub report — aggregate metrics implemented in Phase I"
    });

    let report_path = output_dir.join("corpus-report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report)?)
        .with_context(|| format!("Failed to write report: {}", report_path.display()))?;

    println!(
        "\nvalidate-corpus complete: {} fixtures, manifest={}, report={}",
        fixtures.len(),
        manifest_path.display(),
        report_path.display()
    );

    Ok(())
}
