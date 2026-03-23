# Plan: Monorepo Reorganization for Phase 0→1 Transition

**TL;DR:** Restructure the codebase from a flat `crates/` layout to a master-plan-compliant `apps/` + `crates/` hierarchy before Phase 1 begins. This includes creating hard boundaries between CLI/backend and GUI layers, introducing a `contracts` crate for request/response schemas, reorganizing modules within `engine` to a pipeline structure, and reorganizing docs. Full code refactoring including module renames and import updates.

---

## Steps

1. **Scaffold directory structure** (*parallel with 2-3*)
   - Create `apps/backend-cli/` directory with Cargo.toml skeleton
   - Create `apps/desktop-gui/` directory with Tauri template skeleton
   - Create `crates/contracts/` directory with Cargo.toml skeleton
   - Create `crates/workflows/` directory with Cargo.toml skeleton (stub modules: merge_addenda, split_set, assemble_set, fix_bookmarks)
   - Create `crates/standards-data/` directory with Cargo.toml skeleton (module stubs for MasterFormat/AEC data)
   - Move `tests/fixtures/` → `tests/corpus/` with README explaining torture corpus structure
   - Create `tests/integration/` directory for integration test structure

2. **Create contracts crate** (*depends on step 1; parallel with 3-5*)
   - Define canonical request/response types in `crates/contracts/src/lib.rs`: WorkflowRequest, WorkflowResponse, OperationResult, AuditEventData
   - Align audit event schema types with Phase D migration M-003 (session start/end, operation counts, gate semantics)
   - Implement Serialize/Deserialize on all types
   - Document contract versioning policy (locked to engine version during 0.x, separate versioning post-v1.0)
   - Add version constant that backend-cli and desktop-gui must match

3. **Create workflows crate** (*depends on step 1; parallel with 2, 4-5*)
   - Define Workflow trait in `crates/workflows/src/lib.rs`
   - Create module stubs for each workflow style: merge_addenda, split_set, assemble_set, fix_bookmarks, detect, specs_patch
   - Define WorkflowContext and WorkflowResult types
   - Document workflow ordering contract from Phase D migration M-002 (analyze → applyCorrections → execute gate sequence)

4. **Refactor engine crate modules** (*depends on step 1; workspace registration pulled forward from step 5*)
   - **Pre-requisite [pulled forward from Step 5]:** Register all new crates in root `Cargo.toml` workspace members
     and add workspace-level dependency entries for `conset-pdf-contracts` and `conset-pdf-workflows`, so that
     `cargo check --workspace` validates every crate in the repository.
   - Write engine-specific regression tests covering the public `Extractor::extract` and `Processor::process`
     signatures **before changing any logic** — `crates/engine/tests/engine_api_test.rs`
   - Reorganize `crates/engine/src/` to use pipeline structure:
     - Create `crates/engine/src/pipeline/mod.rs` with public module declarations
     - Create stage stub files: `extraction.rs`, `furniture_detection.rs`, `parsing.rs`, `optimization.rs`
     - `Extractor::extract` delegates to `pipeline::extraction::run` then `pipeline::furniture_detection::run`
     - `Processor::process` delegates to `pipeline::parsing::run` then `pipeline::optimization::run`
     - **All pipeline stages use `LayoutTranscript` at every boundary — no `contracts` types at this layer**
   - Update `engine/src/lib.rs` to declare `pub mod pipeline` and add top-level re-exports for `Extractor`
     and `Processor` (so callers use `conset_pdf_engine::Extractor`, not the full module path)
   - Update all internal imports to use `crate::pipeline::*` module paths
   - **Deferred to Step 6:** Wiring the engine API to `contracts::WorkflowRequest/WorkflowResponse`.
     The engine stays `LayoutTranscript`-typed internally; the translation layer belongs in `backend-cli`
     handlers, not inside the engine crate itself.

5. **Migrate existing crates' dependencies** (*depends on step 1; workspace member registration completed as Step 4 prerequisite*)
   - Update `crates/pdf-extraction/Cargo.toml` to add contracts dependency
   - Update `crates/audit/Cargo.toml` to add contracts dependency and align AuditBundle/AuditEvent types with contracts schema
   - Update `crates/ir/Cargo.toml` (no changes needed; it's a leaf dependency)
   - ~~Update root `Cargo.toml` workspace member list~~ — **completed as Step 4 pre-requisite:**
     - `crates/contracts`, `crates/workflows`, `crates/standards-data`, `apps/backend-cli`, `apps/desktop-gui` already registered
   - Add workspace-level settings: resolver = "2", lints inheritance

6. **Create backend-cli binary** (*depends on steps 2, 4, 5; parallel with 7*)
   - Create `apps/backend-cli/Cargo.toml` depending on engine, contracts, audit crates; declare dependencies on clap (CLI args)
   - Create `apps/backend-cli/src/main.rs` entrypoint
   - Create CLI argument parser skeleton with operations: extract, segment, parse (not implemented, just argument validation)
   - Define main loop that receives WorkflowRequest, calls engine methods, returns WorkflowResponse matching contracts
     - **This is the contracts-engine integration point deferred from Step 4:** translate `WorkflowRequest` →
       engine inputs; translate engine's `LayoutTranscript` outputs → `WorkflowResponse`. Engine internals
       remain `LayoutTranscript`-typed; only the handler layer speaks contracts.
   - Implement audit trail initialization: on startup, create audit bundle directory, write manifest.json with version/timestamp
   - Create `apps/backend-cli/src/handlers/` module with individual operation handlers (stubs)

7. **Scaffold desktop-gui Tauri app** (*depends on step 1; parallel with 6*)
   - Initialize Tauri project in `apps/desktop-gui/` (use `cargo tauri init` or template)
   - Create `apps/desktop-gui/Cargo.toml` depending on contracts, serde_json, tauri crates
   - Create `apps/desktop-gui/src/lib.rs` with Tauri command handler stubs
   - Define command handlers accepting contracts::WorkflowRequest, returning contracts::WorkflowResponse
   - Document that frontend (TypeScript/React) will be added in GUI phase; backend structure only
   - Create placeholder for tauri/TauriWindow configuration

8. **Reorganize documentation** (*no dependencies; parallel with 1-7*)
   - Create `docs/v4/` directory and move/symlink canonical v4 docs:
     - MASTER_PLAN_v4.md, ARCHITECTURE_v4_2.md, DEV_STANDARDS_v4_2.md, AEC_STANDARDS_v4_2.md
   - Create `docs/archived/` and move v3 + prototype artifacts:
     - prototype-postmortem → archived/prototype-postmortem/
     - v3 reference docs → archived/
   - Create `docs/REPO_STRUCTURE.md` describing:
     - apps/ and crates/ responsibilities and layering rules
     - Contract boundary: GUI depends only on `crates/contracts`
     - Test organization (unit in crates/, integration in tests/integration/)
     - Build commands and dependency graph
   - Create `docs/MIGRATION_LOG.md` recording this Phase 0 reorganization (audit trail for future maintainers)
   - Update `docs/DOCUMENTATION_INDEX.md` to reference docs/v4/ as canonical source
   - Create `docs/v4/PHASE_D_INTEGRATION.md` summarizing how Phase D migration insights (M-001 through M-003) were integrated into this new structure

9. **Refactor imports and dependency graph** (*depends on steps 4, 5, 6, 7*)
   - Update engine crate to use pipeline module paths in pub exports
   - Ensure pdf-extraction and audit import contracts types where applicable (AuditEvent, WorkflowRequest, etc.)
   - Update backend-cli to import only from engine + contracts (not pdf-extraction or ir directly);
     backend-cli handlers perform the `LayoutTranscript` ↔ `contracts` translation — engine itself does not
   - Verify no circular dependencies: contracts → (nothing), workflows → contracts, engine → {ir, pdf-extraction [dev]},
     backend-cli → {engine, contracts, audit}, desktop-gui → {contracts, tauri}
   - Run `cargo check --all` to validate linking

10. **Create integration test scaffold** (*no dependencies; can follow steps 6-7*)
    - Create `tests/integration/cli_basic_test.rs` with test harness for backend-cli binary
    - Create `tests/integration/gui_ipc_test.rs` with mock Tauri IPC command tests
    - Both tests should load sample PDFs from `tests/corpus/`, invoke backend operations, and validate response structure (will be stubbed/failing until phase implementations)
    - Document in `tests/integration/README.md` what each test covers and expected pass criteria post-Phase 1

11. **Verify all builds and update CI/CD** (*depends on steps 9-10*)
    - `cargo test --all` should compile (may have stub failures, document as expected)
    - `cargo build --bin backend-cli` produces executable
    - `cargo build --lib` builds all crates cleanly
    - `cargo check --all` with no errors or warnings (use clippy)
    - CI/CD updated to test both CLI and GUI builds
    - Create MIGRATION_COMPLETION_CHECKLIST.md documenting done/verified items

---

## Relevant Files

- `Cargo.toml` (root) — Update workspace members, add resolver, feature flags
- `crates/contracts/Cargo.toml` (create new) — contracts crate definition
- `crates/contracts/src/lib.rs` (create new) — WorkflowRequest, WorkflowResponse, AuditEvent types; align with Phase D M-003
- `crates/contracts/README.md` (create new) — Contract versioning and API stability policy
- `crates/workflows/Cargo.toml` (create new) — workflows crate definition
- `crates/workflows/src/lib.rs` (create new) — Workflow trait, context, module organization; document Phase D M-002 ordering contract
- `crates/standards-data/Cargo.toml` (create new) — standards-data crate definition
- `crates/engine/src/lib.rs` — Refactor public exports to re-export from pipeline submodules
- `crates/engine/src/pipeline/mod.rs` (create new) — Pipeline stage submodule organization
- `crates/engine/src/pipeline/{extraction,furniture_detection,parsing,optimization}.rs` (create new) — Pipeline stage modules
- `crates/engine/Cargo.toml` — Add contracts dependency
- `crates/pdf-extraction/Cargo.toml` — Add contracts dependency
- `crates/audit/Cargo.toml` — Add contracts dependency
- `apps/backend-cli/Cargo.toml` (create new) — CLI binary crate definition
- `apps/backend-cli/src/main.rs` (create new) — CLI entrypoint with operation handlers and request/response packing
- `apps/backend-cli/src/handlers/mod.rs` (create new) — Handler module stubs for extract, segment, parse, etc.
- `apps/desktop-gui/Cargo.toml` (create new) — Tauri application config
- `apps/desktop-gui/src/lib.rs` (create new) — Tauri command handler stubs
- `apps/desktop-gui/src-tauri/tauri.conf.json` (create new) — Tauri window/app config
- `docs/v4/REPO_STRUCTURE.md` (create new) — This repo's organization, boundaries, and build guide
- `docs/v4/PHASE_D_INTEGRATION.md` (create new) — Summary of how Phase D insights were integrated
- `docs/MIGRATION_LOG.md` (create new) — Record of this Phase 0 reorganization
- `docs/archived/` (create new directory) — V3 and prototype artifacts
- `tests/corpus/` (move from fixtures) — Torture corpus reference with README
- `tests/integration/` (create new directory) — Integration test structure
- `tests/integration/cli_basic_test.rs` (create new) — Backend CLI test scaffold
- `tests/integration/gui_ipc_test.rs` (create new) — GUI IPC command test scaffold
- `tests/integration/README.md` (create new) — Integration test documentation

---

## Verification Checklist

- [ ] Directory structure matches master plan (apps/, crates/, docs/{v4,archived}/, tests/{corpus,integration})
- [ ] `cargo check --all` passes with no errors (stubs OK, no missing links)
- [ ] `cargo build --bin backend-cli` produces executable artifact
- [ ] `cargo build --lib` builds contracts, engine, workflows, standards-data cleanly
- [ ] Contracts crate is dependency-free except serde; serves as canonical schema
- [ ] All core imports reference contracts types for request/response/audit structures
- [ ] No circular dependencies in crate graph (verify with `cargo metadata --format-version 1`)
- [ ] Desktop-gui Rust backend compiles (no frontend code required yet)
- [ ] Integration test structure exists with placeholders; CI can execute (tests may fail due to stubs)
- [ ] `docs/v4/REPO_STRUCTURE.md` accurately describes directory layout and contract boundaries
- [ ] No dead imports or unused dependencies; `cargo clippy --all` passes

---

## Decisions Record

**Crate naming:** Keep `crates/engine` (simpler than `core-engine`), but reorganize modules internally to pipeline structure for clarity and phase-by-phase fill-in.

**Contract versioning:** Locked to engine version during 0.x (0.1.0 = Phase 0.5 pattern database, etc.); separate versioning only post-v1.0 if GUI diverges.

**GUI scaffolding:** Create now, implementation deferred to GUI phase; allows parallel backend work and validates IPC contract early.

**Prototype docs integration:** Archive original v4 phase docs but integrate Phase D migration insights (M-001, M-002, M-003, etc.) into canonical v4 docs via explicit callouts.

**Module organization:** Engine uses `pipeline/` with stages (extraction, furniture, parsing, optimization) to match compiler model and enable phase-by-phase implementation.

**Feature flags:** desktop-gui is optional build (`gui` feature); allows CI to test CLI-only on CI runners without Tauri/WebView2 deps.

**Test structure:** Unit tests remain in crates (`/tests/` subdirs); integration tests in `tests/integration/`; torture corpus in `tests/corpus/`.

---

## Further Considerations

### 1. Contract API Stability
Should we stabilize the contracts API (freeze WorkflowRequest/Response signatures) before starting implementations, or iterate as we build?

**Recommendation:** Freeze contracts now (Phase 0 end) at 0.1.0; pre-release tag signals "expect changes," but document that Phase 1–5 implementations assume this contract frozen. This avoids mid-phase refactoring cascades.

### 2. Feature Flag Strategy
Should the Tauri dependency be truly optional, or always present?

**Recommendation:** Make it optional (`gui` feature off by default). CLI-focused CI runners can skip WebView2 build; users installing from source can choose `cargo install --no-default-features` for CLI only.

### 3. Documentation Symlinking
Should v4 canonical docs in `docs/v4/` be symlinked in root for discoverability, or stay nested?

**Recommendation:** Keep in `docs/v4/` but ensure `DOCUMENTATION_INDEX.md` has a prominent "Start here" section pointing to it. Avoids root clutter while making canonical docs discoverable.

---

## Open Questions for Refinement

1. **CI/CD scope:** Should GitHub Actions be updated during this phase, or deferred to Phase 1 when implementations exist?
   - Options: Update now (set up workflows for both CLI and GUI), defer, split (CLI now, GUI later)
   - Current assumption: Update now to catch build/check failures early

2. **Existing test migration:** How should tests currently in `crates/*/tests/` be reorganized if at all?
   - Options: Leave in place (crate unit tests), move to `tests/integration/`, split by type
   - Current assumption: Leave in place; move to `tests/integration/` only if they cross crate boundaries

3. **Fixture/corpus naming:** Is `tests/corpus/` the right name, or prefer `tests/fixtures/` for consistency?
   - Options: Keep `fixtures`, rename to `corpus` (matches master plan language)
   - Current assumption: Rename to `corpus` with clear description that this is the "torture corpus" (all test PDFs)

4. **Backend CLI deployment:** Should `backend-cli` be distributed as:
   - Standalone binary in `apps/backend-cli/` (current plan)
   - Library in `crates/` with binary wrapper (alternative)
   - Both (crate + binary)
   - Current assumption: Standalone binary, designed to be redistributable

5. **Desktop GUI frontend placeholder:** What minimal frontend structure should exist in Phase 0 for `apps/desktop-gui/`?
   - Options: Just Rust backend (current plan), minimal HTML shell, TypeScript project structure
   - Current assumption: Just Rust backend for now; TypeScript scaffolding added in GUI phase

6. **Workspace default features:** Should workspace have a `default` feature set?
   - Options: No defaults (user picks), enable backends (all non-GUI), enable all including gui
   - Current assumption: No default; let users/CI explicitly request what they want

---

## Timeline Estimate

Assuming sequential execution (reality will be much of this in parallel):

- Steps 1-3: ~1-2 hours (file system setup, contracts types)
- Steps 4-5: ~2-3 hours (refactoring engine, updating dependencies)
- Steps 6-7: ~2 hours (CLI and GUI scaffolding)
- Step 8: ~1 hour (docs reorganization)
- Step 9: ~1-2 hours (import refactoring, cargo check loops)
- Steps 10-11: ~1-2 hours (integration test scaffolding, verification)

**Total:** ~11-13 hours for a single developer, likely &lt;8 hours with parallel work.

**Actual estimate for team:** 4-6 hours calendar time if parallelized correctly.
