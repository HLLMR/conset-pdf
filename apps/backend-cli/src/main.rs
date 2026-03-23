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
    OperationCounts, WorkflowOperation, WorkflowOptions, WorkflowRequest, CONTRACTS_VERSION,
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

/// Available operations.  Argument validation is active; execution is stubbed
/// until Phase 1 wires the full pipeline.
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
    /// Segment an extracted transcript into document sections (not implemented).
    Segment {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    /// Parse a segmented transcript into a structured document AST (not implemented).
    Parse {
        #[arg(short, long)]
        input: String,
        #[arg(short, long)]
        output: Option<String>,
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
    let (operation, input_path, output_path, dry_run) = match &cli.command {
        Commands::Extract { input, output, dry_run } => {
            (WorkflowOperation::Extract, input.clone(), output.clone(), *dry_run)
        }
        Commands::Segment { input, output, dry_run } => {
            (WorkflowOperation::Segment, input.clone(), output.clone(), *dry_run)
        }
        Commands::Parse { input, output, dry_run } => {
            (WorkflowOperation::Parse, input.clone(), output.clone(), *dry_run)
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
        options: WorkflowOptions { dry_run, ..Default::default() },
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
