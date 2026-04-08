//! backend-cli — deterministic PDF processing command-line interface.
//!
//! Constructs a [`WorkflowRequest`] from command-line arguments, initialises an
//! audit session, delegates to the appropriate operation handler, and emits the
//! [`WorkflowResponse`] as pretty-printed JSON on stdout.  The full audit bundle
//! is written to `<audit_dir>/<session_id>.json` on exit.
//!
//! # Layering
//!
//! This binary is the sole point where `contracts` types are translated to and
//! from the engine's `LayoutTranscript`-typed API.  The engine crate itself has
//! no dependency on `contracts`.

mod handlers;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::{Parser, Subcommand};
use conset_pdf_audit::{AuditBundle, AuditEvent, AuditEventData};
use conset_pdf_contracts::{
    KeyValuePair, OperationCounts, WorkflowOperation, WorkflowOptions, WorkflowRequest,
    CONTRACTS_VERSION,
};
use serde_json::{Map, Value};
use std::path::PathBuf;

/// Deterministic PDF processing CLI (Phase 0 skeleton).
#[derive(Debug, Parser)]
#[command(name = "backend-cli", version, about)]
struct Cli {
    /// Directory used for audit bundle output (created if absent).
    #[arg(long, default_value = "audit_output")]
    audit_dir: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

/// Available operations.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Extract a layout transcript from a PDF file.
    Extract {
        /// Path to the input PDF.
        #[arg(short, long)]
        input: String,
        /// Path for the output transcript JSON.
        #[arg(short, long)]
        output: Option<String>,
        /// Validate arguments only; skip all processing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Render per-page PNG overlays from a transcript JSON (layout inspection).
    Visualize {
        /// Path to the input transcript JSON (produced by `extract`).
        #[arg(short, long)]
        input: String,
        /// Output directory for overlay PNGs.
        #[arg(short, long)]
        output: String,
        /// Validate arguments only; skip all rendering.
        #[arg(long)]
        dry_run: bool,
    },
    /// Segment an extracted transcript into document sections.
    Segment {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Render per-page PNG overlays from a SegmentIndex for section boundary inspection.
    VisualizeSegments {
        /// Path to the input SegmentIndex JSON (produced by `segment`).
        #[arg(short, long)]
        input: String,
        /// Output directory for color-coded overlay PNGs.
        #[arg(short, long)]
        output: String,
        /// Validate arguments only; skip all rendering.
        #[arg(long)]
        dry_run: bool,
    },
    /// Parse a PDF into a hierarchical document AST (extract → segment → parse pipeline).
    Parse {
        /// Path to the input PDF.
        #[arg(short, long)]
        input: String,
        /// Path for the output AST JSON.
        #[arg(short, long)]
        output: Option<String>,
        /// Only parse the specified CSI section ID (e.g. "23 82 16"). Parses all sections when omitted.
        #[arg(long)]
        section: Option<String>,
        /// Validate arguments only; skip all processing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Render a ParsedDocument AST JSON as a collapsible HTML tree (outline inspection).
    VisualizeAst {
        /// Path to the input AST JSON (produced by `parse`).
        #[arg(short, long)]
        input: String,
        /// Output path for the rendered HTML file.
        #[arg(short, long)]
        output: String,
        /// Validate arguments only; skip all rendering.
        #[arg(long)]
        dry_run: bool,
    },
    /// Apply surgical edit operations (insert/delete/replace) to a ParsedDocument AST.
    Edit {
        /// Path to the input ParsedDocument JSON (produced by `parse`).
        #[arg(short, long)]
        input: String,
        /// Path to the EditRequest JSON file describing the operations to apply.
        #[arg(long)]
        operations: String,
        /// Path for the output (edited) ParsedDocument JSON.
        #[arg(short, long)]
        output: Option<String>,
        /// Validate arguments only; skip all processing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Render a section from a ParsedDocument AST to a PDF via headless Chrome.
    Regenerate {
        /// Path to the input ParsedDocument JSON (produced by `parse` or `edit`).
        #[arg(short, long)]
        ast: String,
        /// CSI section ID to render (e.g. "23 82 16"). Renders first section when omitted.
        #[arg(long)]
        section: Option<String>,
        /// Path to a SpecChromeMetadata JSON file (headers/footers metadata).
        #[arg(long)]
        chrome_metadata: String,
        /// Path for the output PDF file.
        #[arg(short, long)]
        output: String,
        /// CSS font family override (default: "Arial, sans-serif").
        #[arg(long)]
        font: Option<String>,
        /// Body font size in points (default: 10).
        #[arg(long)]
        font_size: Option<u8>,
        /// Validate arguments and build HTML only; skip Chrome render.
        #[arg(long)]
        dry_run: bool,
    },
    /// Replace a section's pages in the original PDF with a regenerated replacement PDF.
    ///
    /// Requires a SegmentIndex JSON (produced by `segment`) to locate the section's
    /// page range.  Unchanged pages are validated and bookmarks are updated automatically.
    Stitch {
        /// Path to the original (source) PDF.
        #[arg(short, long)]
        input: String,
        /// Path to the SegmentIndex JSON (produced by `segment`).
        #[arg(long)]
        segment_index: String,
        /// CSI section ID to replace (e.g. "23 82 16").
        #[arg(long)]
        section: String,
        /// Path to the regenerated replacement PDF (produced by `regenerate`).
        #[arg(long)]
        replacement: String,
        /// Path for the stitched output PDF.
        #[arg(short, long)]
        output: String,
        /// Validate and compute result without writing the output file.
        #[arg(long)]
        dry_run: bool,
    },
    /// Apply an addendum manifest to a source spec PDF, producing a revised PDF.
    ///
    /// Extracts the transcript, segments, parses, applies edits, regenerates sections
    /// via headless Chrome, and stitches the replacements in last-to-first page order.
    /// Partial success: failed sections are recorded; other sections still proceed.
    ApplyAddendum {
        /// Path to the original (source) spec PDF.
        #[arg(long)]
        original: String,
        /// Path to the AddendumManifest JSON file.
        #[arg(long)]
        addendum: String,
        /// Path for the revised output PDF (skipped on dry-run).
        #[arg(short, long)]
        output: Option<String>,
        /// Directory for audit bundle artifacts (optional; skipped when absent).
        #[arg(long)]
        audit_bundle: Option<String>,
        /// Validate, parse, and edit all sections but skip Chrome render and PDF write.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // ── Audit session initialisation ──────────────────────────────────────────
    let started_at = Utc::now();
    let session_id = format!("session-{}", started_at.timestamp_millis());
    let started_at_str = started_at.to_rfc3339();

    std::fs::create_dir_all(&cli.audit_dir)
        .with_context(|| format!("failed to create audit directory {}", cli.audit_dir.display()))?;

    let mut manifest = Map::new();
    manifest.insert("session_id".to_owned(), Value::String(session_id.clone()));
    manifest.insert("contracts_version".to_owned(), Value::String(CONTRACTS_VERSION.to_owned()));
    manifest.insert("started_at_utc".to_owned(), Value::String(started_at_str.clone()));
    manifest
        .insert("engine_version".to_owned(), Value::String(env!("CARGO_PKG_VERSION").to_owned()));
    std::fs::write(
        cli.audit_dir.join("manifest.json"),
        serde_json::to_string_pretty(&Value::Object(manifest))?,
    )
    .context("failed to write audit manifest")?;

    let mut bundle = AuditBundle::new();
    bundle.add_event(AuditEvent::new(AuditEventData::SessionStarted {
        session_id: session_id.clone(),
        started_at_utc: started_at_str,
        contracts_version: CONTRACTS_VERSION.to_owned(),
        engine_version: Some(env!("CARGO_PKG_VERSION").to_owned()),
    }));

    // ── Build WorkflowRequest from CLI args ───────────────────────────────────
    let (operation, input_path, output_path, dry_run, extra_metadata) = match &cli.command {
        Commands::Extract { input, output, dry_run } => {
            (WorkflowOperation::Extract, input.clone(), output.clone(), *dry_run, vec![])
        }
        Commands::Visualize { input, output, dry_run } => {
            (WorkflowOperation::Visualize, input.clone(), Some(output.clone()), *dry_run, vec![])
        }
        Commands::Segment { input, output, dry_run } => {
            (WorkflowOperation::Segment, input.clone(), output.clone(), *dry_run, vec![])
        }
        Commands::VisualizeSegments { input, output, dry_run } => (
            WorkflowOperation::VisualizeSegments,
            input.clone(),
            Some(output.clone()),
            *dry_run,
            vec![],
        ),
        Commands::Parse { input, output, section, dry_run } => {
            let mut meta: Vec<KeyValuePair> = vec![];
            if let Some(sec) = section {
                meta.push(KeyValuePair { key: "section_filter".to_owned(), value: sec.clone() });
            }
            (WorkflowOperation::Parse, input.clone(), output.clone(), *dry_run, meta)
        }
        Commands::VisualizeAst { input, output, dry_run } => (
            WorkflowOperation::VisualizeAst,
            input.clone(),
            Some(output.clone()),
            *dry_run,
            vec![],
        ),
        Commands::Edit { input, operations, output, dry_run } => {
            let meta = vec![KeyValuePair {
                key: "operations_path".to_owned(),
                value: operations.clone(),
            }];
            (WorkflowOperation::Edit, input.clone(), output.clone(), *dry_run, meta)
        }
        Commands::Regenerate { ast, section, chrome_metadata, output, font, font_size, dry_run } => {
            let mut meta = vec![
                KeyValuePair {
                    key: "chrome_metadata_path".to_owned(),
                    value: chrome_metadata.clone(),
                },
            ];
            if let Some(sec) = section {
                meta.push(KeyValuePair { key: "section_filter".to_owned(), value: sec.clone() });
            }
            if let Some(f) = font {
                meta.push(KeyValuePair { key: "font_family".to_owned(), value: f.clone() });
            }
            if let Some(fs) = font_size {
                meta.push(KeyValuePair {
                    key: "font_size_pt".to_owned(),
                    value: fs.to_string(),
                });
            }
            (WorkflowOperation::Regenerate, ast.clone(), Some(output.clone()), *dry_run, meta)
        }
        Commands::Stitch { input, segment_index, section, replacement, output, dry_run } => {
            let meta = vec![
                KeyValuePair {
                    key: "segment_index_path".to_owned(),
                    value: segment_index.clone(),
                },
                KeyValuePair {
                    key: "replacement_path".to_owned(),
                    value: replacement.clone(),
                },
                KeyValuePair { key: "section_id".to_owned(), value: section.clone() },
            ];
            (WorkflowOperation::Stitch, input.clone(), Some(output.clone()), *dry_run, meta)
        }
        Commands::ApplyAddendum { original, addendum, output, audit_bundle, dry_run } => {
            let mut meta = vec![
                KeyValuePair {
                    key: "manifest_path".to_owned(),
                    value: addendum.clone(),
                },
                KeyValuePair {
                    key: "original_path".to_owned(),
                    value: original.clone(),
                },
            ];
            if let Some(bundle_dir) = audit_bundle {
                meta.push(KeyValuePair {
                    key: "audit_bundle_dir".to_owned(),
                    value: bundle_dir.clone(),
                });
            }
            (WorkflowOperation::SpecsPatch, original.clone(), output.clone(), *dry_run, meta)
        }
    };

    let operation_id = format!("op-1-{}", started_at.timestamp_millis());
    let request = WorkflowRequest {
        request_id: format!("req-1-{}", started_at.timestamp_millis()),
        session_id: session_id.clone(),
        operation_id,
        operation,
        input_path,
        output_path,
        options: WorkflowOptions { dry_run, metadata: extra_metadata, ..Default::default() },
    };

    // ── Dispatch ──────────────────────────────────────────────────────────────
    let response = handlers::dispatch(&request, &mut bundle);

    // ── Emit response JSON to stdout ──────────────────────────────────────────
    println!("{}", serde_json::to_string_pretty(&response)?);

    // ── Close audit session and write bundle ──────────────────────────────────
    let ended_at = Utc::now();
    let elapsed_ms = (ended_at - started_at).num_milliseconds();
    let duration_ms: u64 = u64::try_from(elapsed_ms).unwrap_or(0);
    let succeeded =
        u32::from(response.result.status != conset_pdf_contracts::OperationStatus::Failed);
    bundle.add_event(AuditEvent::new(AuditEventData::SessionEnded {
        session_id: session_id.clone(),
        ended_at_utc: ended_at.to_rfc3339(),
        duration_ms,
        operation_counts: OperationCounts { total: 1, succeeded, failed: 1 - succeeded, warned: 0 },
    }));

    conset_pdf_audit::writer::write_bundle_json(
        &bundle,
        cli.audit_dir.join(format!("{session_id}.json")),
    )
    .context("failed to write audit bundle")?;

    Ok(())
}
