# Phase 0 Migration Log

**Version:** 1.0.0  
**Date:** March 23, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

---

## Purpose

Tracks the Phase 0 monorepo reorganization work as an auditable sequence of structural changes.

---

## Summary

Phase 0 established the repository boundaries required before Phase 1 feature implementation.

Completed high-level outcomes:

1. Introduced apps/crates split with explicit workspace membership.
2. Added contracts and workflows crates.
3. Refactored engine into pipeline stage modules without changing public runtime type boundaries.
4. Aligned audit events to typed contracts schema.
5. Created backend-cli and desktop-gui scaffolds with contracts-shaped command interfaces.
6. Reorganized canonical documentation under `docs/v4/`.

---

## Change Log

### 2026-03-23

- Added workspace members:
  - `apps/backend-cli`
  - `apps/desktop-gui`
  - `crates/contracts`
  - `crates/workflows`
  - `crates/standards-data`
- Added workspace dependencies for contracts/workflows/engine and common CLI runtime deps.
- Refactored engine internals to `crates/engine/src/pipeline/` with stage stubs:
  - `extraction`
  - `furniture_detection`
  - `parsing`
  - `optimization`
- Preserved public engine API shape (`Extractor`/`Processor` over `LayoutTranscript`).
- Added engine API regression tests.
- Updated audit crate to use typed `AuditEventData` from contracts.
- Implemented backend-cli request dispatch scaffold and audit bundle output.
- Implemented desktop-gui command handler stubs and placeholder window config.
- Moved canonical v4 docs to `docs/v4/`.
- Moved archived deep-dive artifact to `docs/archived/`.

---

## Verification Artifacts

- `cargo check --workspace` exit code 0 after each major step
- `cargo test -p conset-pdf-engine` exit code 0
- `cargo test -p conset-pdf-audit -p conset-pdf-engine` exit code 0
- `cargo test -p conset-pdf-desktop-gui` exit code 0
- `cargo build --bin backend-cli` exit code 0

---

## Notes

- Prototype historical trees (`prototype-live/`, `prototype-postmortem/`) had already been deleted before this migration and remain referenced only as historical evidence in archive docs.
- Frontend implementation is intentionally deferred; GUI crate currently exposes command and config placeholders only.
