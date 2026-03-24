# Conset PDF Repository Structure

**Version:** 1.0.0  
**Date:** March 23, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

---

## Purpose

Defines the monorepo layout, dependency boundaries, and test organization for the `apps/` + `crates/` repository structure.

---

## Top-Level Layout

- `apps/`: executable surfaces and UI frontends
- `crates/`: shared libraries and core pipeline crates
- `tests/`: repository-level integration and corpus assets
- `docs/`: canonical plans, architecture, and governance docs

---

## Responsibility Boundaries

### apps/

- `apps/backend-cli`: workflow entrypoint for local and CI execution
- `apps/desktop-gui`: Tauri desktop surface (frontend wiring deferred)

Rules:

- apps may depend on crates
- apps must not import each other directly
- UI-specific concerns stay in apps, not in core crates

### crates/

- `crates/contracts`: canonical request/response and audit schemas
- `crates/workflows`: workflow orchestration and step sequencing
- `crates/engine`: deterministic pipeline wrappers and stage orchestration
- `crates/pdf-extraction`: PDF loading/text extraction implementation details
- `crates/ir`: intermediate representation and validation semantics
- `crates/audit`: audit bundle models and persistence
- `crates/standards-data`: standards datasets and lookup helpers

Rules:

- `contracts` is a leaf contract crate (no internal crate deps)
- lower-level crates must not depend on app crates
- avoid circular dependencies between crates

---

## Contract Boundary

- Desktop GUI must depend on `crates/contracts` for request/response envelopes
- GUI should not consume engine internals directly
- Backend CLI is the translation boundary between `contracts` and engine runtime types

---

## Test Organization

- Unit tests: colocated within crates (in `src` and crate-local `tests/`)
- Integration tests: repository-level under `tests/integration/`
- Corpus and fixtures: `tests/corpus/` grouped by tier and holdout sets

---

## Dependency Graph (Current Baseline)

- `conset-pdf-contracts` -> (none)
- `conset-pdf-workflows` -> `conset-pdf-contracts`
- `conset-pdf-ir` -> (none)
- `conset-pdf-extraction` -> `conset-pdf-ir`, `conset-pdf-contracts`
- `conset-pdf-audit` -> `conset-pdf-contracts`
- `conset-pdf-engine` -> `conset-pdf-ir` (dev: `conset-pdf-extraction`)
- `conset-pdf-backend-cli` -> `conset-pdf-engine`, `conset-pdf-contracts`, `conset-pdf-audit`
- `conset-pdf-desktop-gui` -> `conset-pdf-contracts`, `tauri`

---

## Build Commands

- `cargo check --workspace`
- `cargo test --workspace`
- `cargo build --bin backend-cli`
- `cargo test -p conset-pdf-desktop-gui`

Use workspace-level dependencies in root `Cargo.toml` wherever possible.
