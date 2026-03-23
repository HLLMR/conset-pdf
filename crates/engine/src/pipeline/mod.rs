//! Pipeline stage modules for the Conset PDF engine.
//!
//! The engine processes documents through four sequential stages that mirror the
//! compiler model described in the V4 architecture:
//!
//! 1. [`extraction`]         — Lexer: load PDF bytes → raw `LayoutTranscript`
//! 2. [`furniture_detection`] — Identify non-content structural elements (headers,
//!    footers, stamp blocks) before semantic parsing
//! 3. [`parsing`]            — Parser: validate and normalise the transcript
//! 4. [`optimization`]       — Code-gen: apply deterministic edits to produce the
//!    final output transcript
//!
//! # Type contract
//!
//! Every stage accepts and returns [`conset_pdf_ir::LayoutTranscript`].  No
//! `contracts` crate types appear here — translation to/from
//! `contracts::WorkflowRequest/WorkflowResponse` happens only in the
//! `apps/backend-cli` handler layer.

pub mod extraction;
pub mod furniture_detection;
pub mod optimization;
pub mod parsing;
