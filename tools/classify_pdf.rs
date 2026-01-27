use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, NaiveDate};
use clap::{ArgAction, Parser};
use colored::*;
use pdfium_render::prelude::*;
use serde::Serialize;
use serde_json::json;

#[derive(Parser, Debug)]
#[command(name = "classify-pdf", about = "Classify PDFs into torture corpus tiers", version)]
struct Cli {
    /// Path to a PDF (ignored when --batch)
    pdf_path: Option<PathBuf>,

    /// Output JSON instead of human-readable text
    #[arg(long, action = ArgAction::SetTrue)]
    json: bool,

    /// Read PDF paths from stdin (one per line)
    #[arg(long, action = ArgAction::SetTrue)]
    batch: bool,

    /// Output directory for auto-sorting PDFs into tier subdirs
    #[arg(long, value_name = "DIR")]
    output_dir: Option<PathBuf>,

    /// Verbose scoring breakdown
    #[arg(long, action = ArgAction::SetTrue)]
    verbose: bool,

    /// Emit metadata template JSON to stdout (ignore classification)
    #[arg(long, action = ArgAction::SetTrue)]
    metadata: bool,

    /// Validate existing metadata.json classification against recommendation
    #[arg(long, action = ArgAction::SetTrue)]
    check: bool,
}

#[derive(Debug, Serialize, Clone)]
struct Indicators {
    producer: Option<String>,
    creation_date: Option<String>,
    page_count: Option<usize>,
    file_size_bytes: Option<u64>,
    bytes_per_page: Option<u64>,
    text_extractable: Option<bool>,
    page_size: Option<String>,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct Classification {
    tier: u8,
    score: i32,
    confidence: f64,
    reasoning: Vec<String>,
    indicators: Indicators,
}

fn main() -> Result<()> {
    let args = Cli::parse();

    if args.metadata {
        print_metadata_template();
        return Ok(());
    }

    if args.batch {
        for line in io::stdin().lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            let path = PathBuf::from(line.trim());
            match classify_one(&path, args.verbose) {
                Ok(result) => {
                    emit(&path, &result, args.json)?;
                    if let Some(dir) = args.output_dir.as_ref() {
                        autosort(&path, dir, result.tier, &result)?;
                    }
                    if args.check {
                        check_metadata(&path, &result)?;
                    }
                }
                Err(e) => {
                    eprintln!("{} {}: {e}", "[error]".red(), path.display());
                }
            }
        }
        return Ok(());
    }

    let path = args
        .pdf_path
        .as_ref()
        .context("Provide <PDF_PATH> or use --batch")?;

    let result = classify_one(path, args.verbose)?;
    emit(path, &result, args.json)?;

    if let Some(dir) = args.output_dir.as_ref() {
        autosort(path, dir, result.tier, &result)?;
    }

    if args.check {
        check_metadata(path, &result)?;
    }

    Ok(())
}

fn print_metadata_template() {
    let template = json!({
        "filename": "example.pdf",
        "tier": "tier1",
        "description": "[FILL IN]",
        "expected_pages": 0,
        "special_features": ["tables"],
        "known_issues": [],
        "source": "[FILL IN]"
    });
    println!("{}", serde_json::to_string_pretty(&template).unwrap());
}

fn load_pdfium() -> Result<Pdfium> {
    if let Ok(dir) = env::var("PDFIUM_LIB_PATH") {
        if let Ok(bindings) =
            Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path(&dir))
        {
            return Ok(Pdfium::new(bindings));
        }
    }

    if let Ok(bindings) = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./")) {
        return Ok(Pdfium::new(bindings));
    }

    if let Ok(bindings) = Pdfium::bind_to_system_library() {
        return Ok(Pdfium::new(bindings));
    }

    Err(anyhow!(
        "Pdfium library not found. Download pdfium.dll (e.g., from bblanchon/pdfium-binaries) and place it next to classify-pdf.exe or set PDFIUM_LIB_PATH to its directory."
    ))
}

fn classify_one(path: &Path, verbose: bool) -> Result<Classification> {
    let mut reasoning = Vec::new();
    let mut errors = Vec::new();

    let meta = fs::metadata(path)
        .with_context(|| format!("Could not read file metadata for {}", path.display()))?;
    let file_size_bytes = meta.len();

    // Load PDF with pdfium
    let pdfium = load_pdfium()?;
    let doc = match pdfium.load_pdf_from_file(path, None) {
        Ok(doc) => doc,
        Err(e) => {
            errors.push(format!("failed_to_load: {e}"));
            // Hard-classify as Tier 3 when unreadable
            return Ok(Classification {
                tier: 3,
                score: 0,
                confidence: 0.3,
                reasoning: vec!["Failed to load PDF (treating as Tier 3)".to_string()],
                indicators: Indicators {
                    producer: None,
                    creation_date: None,
                    page_count: None,
                    file_size_bytes: Some(file_size_bytes),
                    bytes_per_page: None,
                    text_extractable: Some(false),
                    page_size: None,
                    errors,
                },
            });
        }
    };

    // Metadata (prefer pdfium; fallback to pdfinfo if not available)
    let (producer, creation_date, mut meta_errors) = extract_metadata(path, &doc);

    // Page metrics
    let page_count: usize = doc.pages().len().into();
    let bytes_per_page = if page_count > 0 {
        Some((file_size_bytes as f64 / page_count as f64) as u64)
    } else {
        None
    };

    // Page size assessment (all pages)
    let page_sizes = assess_page_sizes(&doc, verbose);
    let page_size = page_sizes.as_ref().map(|p| p.summary.clone());

    // Text extractability: attempt first 3 pages
    let mut text_extractable = None;
    let mut extracted_chars = 0usize;
    for idx in 0..page_count.min(3) {
        if let Ok(page) = doc.pages().get(idx as u16) {
            if let Ok(text) = page.text() {
                let s = text.all();
                extracted_chars += s.len();
            }
        }
    }
    if page_count > 0 {
        text_extractable = Some(extracted_chars > 0);
    }

    let indicators = Indicators {
        producer,
        creation_date,
        page_count: Some(page_count),
        file_size_bytes: Some(file_size_bytes),
        bytes_per_page,
        text_extractable,
        page_size,
        errors: {
            let mut combined = errors;
            combined.append(&mut meta_errors);
            combined
        },
    };

    let (score, tier, mut reasoning_updates, measured) = score(&indicators, page_sizes.as_ref(), verbose);
    reasoning.append(&mut reasoning_updates);

    // Confidence: fraction of measured signals plus how strong the score is
    let confidence = compute_confidence(score, tier, measured);

    if verbose {
        eprintln!("Analyzed {}: score={}, tier={}, confidence={:.2}", path.display(), score, tier, confidence);
    }

    Ok(Classification { tier, score, confidence, reasoning, indicators })
}

fn score(ind: &Indicators, page_sizes: Option<&PageSizeEvaluation>, verbose: bool) -> (i32, u8, Vec<String>, usize) {
    let mut score = 0i32;
    let mut reasoning = Vec::new();
    let mut measured = 0usize;

    // Tier 1 indicators (+20)
    if let Some(prod) = ind.producer.as_ref() {
        measured += 1;
        let prod_l = prod.to_lowercase();
        if prod_l.contains("adobe pdf library")
            || prod_l.contains("microsoft word")
            || prod_l.contains("microsoft excel")
            || prod_l.contains("bluebeam")
            || prod_l.contains("autodesk")
        {
            score += 20;
            reasoning.push(format!("Modern producer ({prod}) [+20]"));
        }
    }

    if let Some(date_str) = ind.creation_date.as_ref() {
        measured += 1;
        if let Some(year) = parse_year(date_str) {
            if year >= 2020 {
                score += 20;
                reasoning.push(format!("Recent creation date ({year}) [+20]"));
                log_verbose(verbose, &format!("Creation year: {year} [+20]"));
            } else if (2010..=2019).contains(&year) {
                score += 10;
                reasoning.push(format!("Mid-era creation date ({year}) [+10]"));
                log_verbose(verbose, &format!("Creation year: {year} [+10]"));
            } else {
                score += 5;
                reasoning.push(format!("Old creation date ({year}) [+5]"));
                log_verbose(verbose, &format!("Creation year: {year} [+5]"));
            }
        }
    }

    if let Some(bpp) = ind.bytes_per_page {
        measured += 1;
        if bpp < 300_000 {
            score += 20;
            reasoning.push(format!("Bytes/page {bpp} (<300k) [+20]"));
        } else if bpp <= 500_000 {
            score += 10;
            reasoning.push(format!("Bytes/page {bpp} (300-500k) [+10]"));
        } else {
            score += 5;
            reasoning.push(format!("Bytes/page {bpp} (>500k) [+5]"));
        }
    }

    if let Some(te) = ind.text_extractable {
        measured += 1;
        if te {
            score += 20;
            reasoning.push("Text extractable [+20]".to_string());
        } else {
            score += 5;
            reasoning.push("Text not extractable [+5]".to_string());
        }
    }

    if let Some(status) = page_sizes {
        measured += 1;
        match &status.status {
            PageSizeStatus::AllStandard { name } => {
                score += 20;
                reasoning.push(format!("Standard page sizes ({name}) [+20]"));
                log_verbose(verbose, "All pages standard sizes [+20]");
            }
            PageSizeStatus::MixedStandard { names } => {
                score += 10;
                let joined = names.join(", ");
                reasoning.push(format!("Mixed standard page sizes ({joined}) [+10]"));
                log_verbose(verbose, "Mixed standard page sizes [+10]");
            }
            PageSizeStatus::NonStandard => {
                score += 10;
                reasoning.push("Non-standard page sizes [+10]".to_string());
                log_verbose(verbose, "Non-standard page sizes [+10]");
            }
        }
    }

    // Errors -> treat as chaos signals
    if !ind.errors.is_empty() {
        measured += 1;
        score += 5;
        reasoning.push("PDF load errors detected [+5]".to_string());
    }

    let tier = if score >= 40 {
        1
    } else if score >= 20 {
        2
    } else {
        3
    };

    (score, tier, reasoning, measured)
}

fn log_verbose(verbose: bool, message: &str) {
    if verbose {
        eprintln!("[debug] {message}");
    }
}

fn parse_year(s: &str) -> Option<i32> {
    // pdfium date strings often look like D:20240101120000Z
    if let Some(pos) = s.find(':') {
        let year_str = &s[pos + 1..pos + 5.min(s.len())];
        return year_str.parse().ok();
    }
    // Try ISO
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(date.year());
    }
    // pdfinfo: "Mon Feb  3 18:59:22 2025 Central Standard Time"
    for part in s.split_whitespace() {
        if part.len() == 4 && part.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(year) = part.parse() {
                return Some(year);
            }
        }
    }
    None
}

#[derive(Debug)]
enum PageSizeStatus {
    AllStandard { name: String },
    MixedStandard { names: Vec<String> },
    NonStandard,
}

#[derive(Debug)]
struct PageSizeEvaluation {
    status: PageSizeStatus,
    summary: String,
}

fn assess_page_sizes(doc: &PdfDocument, verbose: bool) -> Option<PageSizeEvaluation> {
    let page_count: usize = doc.pages().len().into();
    if page_count == 0 {
        return None;
    }

    let mut names = BTreeSet::new();
    let mut any_non_standard = false;

    for idx in 0..page_count {
        if let Ok(page) = doc.pages().get(idx as u16) {
            let w = page.width().value;
            let h = page.height().value;
            let (norm_w, norm_h) = if w <= h { (w, h) } else { (h, w) };
            let orientation = if w <= h { "portrait" } else { "landscape" };
            let standard = classify_standard_size(norm_w, norm_h);

            match standard {
                Some(name) => {
                    names.insert(name.to_string());
                    log_verbose(verbose, &format!("Page {idx}: {:.0}x{:.0} -> {name} ({orientation})", w, h));
                }
                None => {
                    any_non_standard = true;
                    log_verbose(verbose, &format!("Page {idx}: {:.0}x{:.0} -> Non-standard ({orientation})", w, h));
                }
            }
        }
    }

    let status = if any_non_standard {
        PageSizeStatus::NonStandard
    } else {
        let unique: Vec<String> = names.into_iter().collect();
        if unique.len() == 1 {
            PageSizeStatus::AllStandard {
                name: unique[0].clone(),
            }
        } else {
            PageSizeStatus::MixedStandard { names: unique }
        }
    };

    let summary = match &status {
        PageSizeStatus::AllStandard { name } => name.clone(),
        PageSizeStatus::MixedStandard { names } => format!("Mixed standard ({})", names.join(", ")),
        PageSizeStatus::NonStandard => "Non-standard".to_string(),
    };

    Some(PageSizeEvaluation { status, summary })
}

fn classify_standard_size(w: f32, h: f32) -> Option<&'static str> {
    // Normalize to portrait dimensions: w <= h
    let standard_sizes = [
        (612.0f32, 792.0f32, "Letter"),
        (612.0, 1008.0, "Legal"),
        (792.0, 1224.0, "Tabloid"),
        (595.0, 842.0, "A4"),
    ];
    let tolerance = 2.0;

    for (std_w, std_h, name) in standard_sizes {
        if (w - std_w).abs() <= tolerance && (h - std_h).abs() <= tolerance {
            return Some(name);
        }
    }
    None
}

fn compute_confidence(score: i32, tier: u8, measured: usize) -> f64 {
    if measured == 0 {
        return 0.3;
    }
    let coverage = measured as f64 / 6.0; // rough number of signals
    let tier_margin = match tier {
        1 => (score - 40).max(0) as f64 / 60.0,
        2 => (score - 20).abs() as f64 / 40.0,
        _ => (20 - score).max(0) as f64 / 40.0,
    };
    let mut conf = 0.5 * coverage + 0.5 * tier_margin;
    if conf > 1.0 {
        conf = 1.0;
    }
    conf
}

fn extract_metadata(path: &Path, _doc: &PdfDocument) -> (Option<String>, Option<String>, Vec<String>) {
    // pdfium-render 0.8.x does not currently expose producer / creation_date directly;
    // fall back to pdfinfo when available.
    eprintln!("[debug] pdfium metadata not available; falling back to pdfinfo");

    // Fallback: shell out to pdfinfo if available
    match Command::new("pdfinfo").arg(path).output() {
        Ok(out) => {
            if !out.status.success() {
                return (None, None, vec![format!("pdfinfo failed with status {}", out.status)]);
            }
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut producer = None;
            let mut creation_date = None;
            for line in stdout.lines() {
                let line = line.trim();
                if line.starts_with("Producer:") {
                    producer = line.splitn(2, ':').nth(1).map(|s| s.trim().to_string());
                }
                if line.starts_with("CreationDate:") {
                    creation_date = line.splitn(2, ':').nth(1).map(|s| s.trim().to_string());
                }
            }
            eprintln!("[debug] Extracted metadata via pdfinfo: producer={:?}, date={:?}", producer, creation_date);
            (producer, creation_date, Vec::new())
        }
        Err(e) => {
            (None, None, vec![format!("pdfinfo not available: {e}")])
        }
    }
}

fn emit(path: &Path, result: &Classification, json_mode: bool) -> Result<()> {
    if json_mode {
        let mut value = serde_json::to_value(result)?;
        if let Some(obj) = value.as_object_mut() {
            obj.insert("filename".to_string(), json!(path.file_name().and_then(|s| s.to_str())));
            obj.insert("path".to_string(), json!(path.display().to_string()));
        }
        println!("{}", serde_json::to_string_pretty(&value)?);
        return Ok(());
    }

    let tier_str = match result.tier {
        1 => "Tier 1".green(),
        2 => "Tier 2".yellow(),
        _ => "Tier 3".red(),
    };

    println!("Analyzing: {}", path.display());
    for line in &result.reasoning {
        println!("├─ {line}");
    }
    println!("└─ Total Score: {}", result.score);
    println!("Recommended Tier: {}", tier_str);
    println!("Confidence: {:.2}", result.confidence);
    Ok(())
}

fn autosort(src: &Path, out_dir: &Path, tier: u8, result: &Classification) -> Result<()> {
    let tier_dir = out_dir.join(match tier {
        1 => "tier1",
        2 => "tier2",
        _ => "tier3",
    });
    fs::create_dir_all(&tier_dir)?;
    let filename = src.file_name().context("missing filename")?;
    let dest = tier_dir.join(filename);
    fs::copy(src, &dest).with_context(|| format!("Failed to copy to {}", dest.display()))?;

    let meta_path = dest.with_extension("metadata.json");
    let meta = json!({
        "filename": filename.to_string_lossy(),
        "tier": format!("tier{tier}"),
        "score": result.score,
        "confidence": result.confidence,
        "indicators": result.indicators,
    });
    fs::write(&meta_path, serde_json::to_string_pretty(&meta)?)?;
    Ok(())
}

fn check_metadata(pdf: &Path, result: &Classification) -> Result<()> {
    let meta_path = pdf.with_extension("metadata.json");
    if !meta_path.exists() {
        eprintln!("{} no metadata.json beside {}", "[warn]".yellow(), pdf.display());
        return Ok(());
    }
    let data = fs::read_to_string(&meta_path)?;
    let value: serde_json::Value = serde_json::from_str(&data)?;
    if let Some(expected) = value.get("tier").and_then(|v| v.as_str()) {
        let recommended = format!("tier{}", result.tier);
        if expected != recommended {
            eprintln!(
                "{} tier mismatch: metadata={}, recommended={}",
                "[mismatch]".yellow(),
                expected,
                recommended
            );
        }
    }
    Ok(())
}
