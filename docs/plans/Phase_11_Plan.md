# Phase 11 Plan: Desktop GUI (V1.0)

**Version:** 1.0.0  
**Date:** April 11, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE — Ready for implementation  
**Doc Status Tag:** Planned

---

## Table of Contents

1. [Project State Assessment](#1-project-state-assessment)
2. [Pre-Phase-11 Cleanup](#2-pre-phase-11-cleanup)
3. [GUI Gate Verification](#3-gui-gate-verification)
4. [Phase 11 Architecture](#4-phase-11-architecture)
5. [Sprint Plan](#5-sprint-plan)
6. [Definition of Done](#6-definition-of-done)
7. [Open Gaps Relevant to Phase 11](#7-open-gaps-relevant-to-phase-11)

---

## 1. Project State Assessment

### Baseline (Phase 10 Complete — April 11, 2026)

| Suite | Passing | Ignored |
|---|---|---|
| CLI integration | **67** | 5 (2 Chrome + 1 spec benchmark + 1 drawing benchmark + 1 SUB extraction profile) |
| Engine unit | **167** | 3 (Chrome) |
| IR unit | **50** | — |
| standards-data unit | **33** | — |

- `cargo check --workspace` exits 0 (confirmed Sprint 10.5).
- All three production workflows are backend-complete and CLI-tested:
  - **Specs:** `apply-addendum` (extract → segment → parse → edit → render → stitch), full audit bundle.
  - **Drawings:** `apply-sheet-addendum` (extract → index-drawing → stitch), sheet rename detection, bookmark generation.
  - **Submittals:** `extract-submittal` (extract → index-submittal → KV + table extraction → tidy export JSON/CSV).
- All three workflows have determinism tests that pass.
- `docs/CLI_REFERENCE.md` is complete for all 15+ subcommands.
- `docs/WORKFLOW_APPLYADDENDUM.md`, `docs/WORKFLOW_APPLYSHEETADDENDUM.md`, `docs/WORKFLOW_EXTRACTSUBMITTAL.md` all published.

### Desktop GUI Current State

`apps/desktop-gui/` contains:
- `src/lib.rs` — stub command handlers returning `NOT_IMPLEMENTED` for `cmd_extract`, `cmd_segment`, `cmd_parse`. Three unit tests pass (stub contract shape only).
- `src-tauri/tauri.conf.json` — scaffold config: product name, 1200×800 window, `frontendDist: "../dist"`. No `src-tauri/` Rust binary, no frontend HTML/JS/CSS present.
- `Cargo.toml` — depends on `conset-pdf-contracts` + `serde_json` + `tauri = "2.0"`. Intentionally does **not** depend on `conset-pdf-engine`.

The desktop-gui crate compiles and its 3 unit tests pass. No UI exists. This is the correct starting state for a fresh canonical implementation.

---

## 2. Pre-Phase-11 Cleanup

~~Two documentation inconsistencies from Phase 10 close require one-time reconciliation before Phase 11 implementation begins.~~

**Status: COMPLETE (April 11, 2026)** — both items resolved during Phase 11 plan authoring.

### Pre-11-A — Update `state-summary.md` Next Focus section ✅

~~The `## Next Focus` section still reads:~~

~~> **Phase 10 (Submittal Data Extraction)** can begin — all Phase 9 DoD rows are satisfied.~~

Updated to Phase 11 forward-pointer. Done.

### Pre-11-B — Bump `capability-matrix.md` version and date ✅

~~The header reads `Version: 2.4.0 / Date: April 8, 2026`. Sprint 10.5 changelog records that 4 new submittal rows were added, but the version was not bumped in the header.~~

Bumped to `Version: 2.5.0 / Date: April 11, 2026`. Done.

### Pre-Phase-11 Cleanup Effort

**Total: ~1 hour. No code changes required. Documentation only.**

---

## 3. GUI Gate Verification

The MASTER_PLAN requires four dependency gates to be verified before enabling Track B (production GUI runtime). All four gates are confirmed satisfied as of Phase 10 close.

### Gate 0 — Contract Boundary Ready ✅

- `crates/contracts/src/lib.rs` defines all shared request/response/event types: `WorkflowRequest`, `WorkflowResponse`, `WorkflowOperation`, `OperationResult`, `OperationStatus`, `WorkflowOptions`.
- `crates/contracts/src/intake.rs` defines `IntakeBundle`, `IntakeFile`, `IntakeRole`, `NormalizedIntakeBundle`, `IntakeIssue`.
- 24/24 contract unit tests pass.
- `apps/desktop-gui/Cargo.toml` already depends on `conset-pdf-contracts` only — boundary enforced.

### Gate 1 — Baseline Extraction Chain Closed ✅

Critical path closed: G-004 → G-005 → G-001 → G-002 → G-003 → G-009.

- Real PDFium extraction wired (`crates/engine/src/pipeline/extraction.rs`).
- `validate_transcript()` called in pipeline.
- `backend-cli` covers all runtime paths with 67 active integration tests.
- All three end-to-end orchestrators (`SpecsPatchOrchestrator`, `DrawingsPatchOrchestrator`, + submittal pipeline) proven on real corpus.

### Gate 2 — Review Inputs Ready ✅

- `DiagnosticEvent` IR types with 6 stage variants in `crates/ir/src/diagnostics.rs`.
- `diagnostics.jsonl` written to every `--audit-bundle` directory (Sprint 8.1).
- Per-section `SectionPatchResult` in `AddendumResult` carries `status`, `warnings`, `failure_reason`.
- Per-sheet `SheetPatchResult` in `DrawingPatchResult` carries `status`.
- Per-unit `UnitSummary` in `EquipmentDataset` carries `avg_confidence`, `warnings`.
- All audit hooks wired in all 10 CLI handlers (G-006 closed).

### Gate 3 — Export Inputs Ready ✅

- `AddendumResult` final PDF path confirmed in `change-report.json` for specs.
- `DrawingPatchResult` final PDF path confirmed in `change-report.json` for drawings.
- `EquipmentDataset` JSON + CSV output with `schema_version = "1.0.0"` for submittals (G-028 closed).
- Determinism tests for all three workflows pass (specs: `apply_addendum_dry_run_is_deterministic`, drawings: `apply_sheet_addendum_dry_run_is_deterministic`, submittals: `extract_submittal_dry_run_is_deterministic`).
- Partial-success export validated: `SpecsPatchOrchestrator` continues and writes partial output when one or more sections fail.

**All four gates pass. Track B (production lane execution) is authorized for Lane 1.**

---

## 4. Phase 11 Architecture

### 4.1 Technology Stack

| Layer | Technology | Rationale |
|---|---|---|
| **Desktop shell** | Tauri v2 | Rust-native, cross-platform, ships a single installer; canonical per MASTER_PLAN |
| **Frontend** | React 18 + TypeScript + Vite | Standard, small bundle, strong typing; Vite aligns with Tauri v2 scaffold defaults |
| **IPC bridge** | Tauri `invoke()` API → `#[tauri::command]` handlers | Typed TypeScript ↔ Rust boundary; contracts types serialized as JSON |
| **Backend invocation** | `backend-cli` subprocess spawned by Tauri Rust side | Enforces `crates/contracts` boundary; Tauri Rust never imports `crates/engine` |
| **State management** | React context + `useReducer` | Deterministic reducer logic, easy to snapshot-test |
| **Styling** | Tailwind CSS v3 | Utility-first, no runtime overhead, fast iteration |

### 4.2 IPC Contract Design

The MASTER_PLAN non-negotiable: **GUI must depend on backend only through `crates/contracts`**. The `desktop-gui` crate `Cargo.toml` already enforces this — it depends on `conset-pdf-contracts` and NOT on `conset-pdf-engine`.

**Call path:**

```
TypeScript (React)
   → tauri.invoke("run_workflow", { request: WorkflowRequest })
   → #[tauri::command] fn run_workflow(request: WorkflowRequest)
       in src-tauri/src/commands.rs
   → spawn backend-cli subprocess with --request <JSON> flag
   → parse stdout as WorkflowResponse JSON
   → return WorkflowResponse to TypeScript
```

**Why subprocess, not direct engine link:**

- The Tauri Rust backend (src-tauri) must NOT import `crates/engine` internals per the architectural boundary.
- `backend-cli` is the authoritative engine host. Spawning it as a subprocess means the GUI literally uses the same code path as the CLI, making parity trivial to verify.
- PDFium requires `PDFIUM_LIB_PATH` to be set. The subprocess inherits env from the Tauri process, so the bundled PDFium path can be injected at launch time.
- `backend-cli` is shipped alongside the desktop app bundle (bundled in `resources/`).

**WorkflowRequest → subprocess:**

The Tauri commands module serializes `WorkflowRequest` to JSON and invokes:

```
backend-cli <subcommand> --request <base64-encoded-json>
```

or passes arguments as per the existing CLI flag conventions (preferred — avoids a new --request flag injection). The Tauri command layer maps the GUI session state to the appropriate CLI subcommand and arguments.

**Progress streaming (Option B — see Section 4.7):**

For operations that take time (e.g., `apply-addendum` on a 571-page PDF), `backend-cli` emits structured JSONL progress events to stdout line-by-line when `--progress-events` is set. The Tauri backend reads stdout with `BufReader::lines()` in a spawned thread and re-emits each line as a Tauri window event. The frontend subscribes via `listen("pipeline_progress", handler)` to update the stage label and progress indicator in real time.

### 4.3 Four-Step UX Model

Follows the canonical MASTER_PLAN/guiPlan.md simple-first 4-step flow:

```
Step 1: Add Files      →  Step 2: Start          →  Step 3: Review         →  Step 4: Export
─────────────────────────────────────────────────────────────────────────────────────────────
File picker (native)      Workflow selection         Confidence-flagged         Save to disk
PDF drag-and-drop         One-button start           items queue               Format selection
File validation           Progress indicator         Confirm / Skip / Edit     Partial-success
Workflow auto-hint        Partial success display    Provenance visible         Export summary
                          Per-item status            Advanced behind toggle
```

**Workflow auto-hint (non-enforcement):** When a user adds files, the GUI analyzes filenames for hints (`SPEC_`, `DWG_`, `SUB_` prefixes or naming conventions) and pre-selects the probable workflow. This is advisory only; the user confirms. Users are never locked into a workflow selected by filename — they can override.

**Manifest requirements by workflow:**
- **Drawings:** GUI synthesizes `DrawingAddendumManifest` from the two file picker inputs (original PDF + addendum PDF). No JSON authoring required.
- **Submittals:** No manifest. User picks one PDF; output format is selected at export time.
- **Specs:** Requires an `AddendumManifest` specifying per-section edit operations. The **Manifest Assistant** (Section 4.8) guides the user: analyze the spec → view detected sections → generate a pre-populated template → author offline → load and validate. No blank-JSON authoring against an unknown section list.

**One-window, single flow.** No tabbed multi-workflow views in v1.0. Open a new window for each independent job.

### 4.4 Session State Machine

```
Idle
  ↓ ADD_FILES
FilesAdded (files: Vec<FileEntry>, workflow_type: Option<WorkflowType>)

  [Drawings / Submittals — manifest synthesized internally by GUI]
  ↓ CONFIRM_WORKFLOW
WorkflowReady (files, workflow_type, manifest: Option<ManifestRef>)

  [Specs addendum — Manifest Assistant path]
  ↓ AUTO_SEGMENT (background: cmd_extract + cmd_segment)
ManifestDraft (files, workflow_type: SpecsAddendum, sections: Vec<DetectedSection>)
  ↓ LOAD_MANIFEST (user loads and validates JSON manifest)
WorkflowReady (files, workflow_type: SpecsAddendum, manifest: ManifestRef)

  ↓ START
Processing (operation: ActiveOperation, progress: ProgressState)
  ↓ COMPLETE / PARTIAL_SUCCESS / FAILED
ReviewReady (result: WorkflowResult, items: Vec<ReviewItem>)
  ↓ EXPORT
Exported (export_path: PathBuf, summary: ExportSummary)
  ↓ RESET
Idle
```

All transitions are pure functions (deterministic, no side effects) suitable for snapshot testing. Side effects (subprocess spawn, file I/O) happen only in Tauri command handlers, not in the session reducer.

### 4.5 Review Item Model

Each `ReviewItem` maps to a flagged diagnostic from the backend:

```typescript
interface ReviewItem {
  id: string;
  source: "section" | "sheet" | "unit";
  label: string;            // "Section 23 82 16" / "Sheet A-101" / "Unit AHU-1"
  confidence: number;       // 0.0–1.0
  status: "needs_review" | "confirmed" | "skipped" | "edited";
  page: number;
  warning_text: string;     // plain-language description
  // Advanced (behind toggle):
  bbox?: BBox;
  source_field?: string;
  audit_evidence?: string;
}
```

Confidence thresholds for review queue gating:
- `≥ 0.9` — auto-accept, do not queue for review
- `0.7–0.9` — flag for review, show suggestion
- `< 0.7` — flag as Needs Review, require explicit confirm or skip

---

### 4.6 Binary Bundling and Environment Setup

`backend-cli` and PDFium must ship inside the Tauri installer — not assumed to exist on the user's PATH.

**Tauri `bundle.resources` (`tauri.conf.json`):**
```json
"bundle": {
  "resources": {
    "../../target/release/backend-cli": "backend-cli",
    "../../target/release/backend-cli.exe": "backend-cli.exe",
    "pdfium/pdfium.dll": "pdfium.dll",
    "pdfium/libpdfium.dylib": "libpdfium.dylib"
  }
}
```

**Runtime resolution (`src-tauri/src/backend_process.rs`):**
```rust
let res_dir = app_handle.path().resource_dir()?;
let binary = if cfg!(windows) { "backend-cli.exe" } else { "backend-cli" };
let cli_path = res_dir.join(binary);
// PDFIUM_LIB_PATH env var set to res_dir before spawning subprocess
```

**Build prerequisite:** Run `cargo build --release -p backend-cli` before `cargo tauri build`. Document in `docs/SETUP.md` under a new "Desktop GUI Build" section. The Tauri CI/build script must express this dependency explicitly.

**First-run probe:** On every app startup, before the main UI is interactive, `probe_environment(cli_path)` calls `Command::new(cli_path).arg("--help").status()`. On failure a blocking modal shows the specific error (binary not found / PDFium not found) and resolution steps. The app does not proceed to the main UI until the probe passes or the user quits.

---

### 4.7 Progress Streaming Protocol (Option B)

`backend-cli` emits structured JSONL progress events to stdout line-by-line when `--progress-events` is set, enabling real-time stage labels in the GUI.

**Stdout protocol:**
```
# Each line is valid JSON; emitted immediately before each stage begins.
{"type":"progress","stage":"extracting","label":"Extracting layout…","pct":null}
{"type":"progress","stage":"segmenting","label":"Segmenting sections…","pct":null}
{"type":"progress","stage":"editing","label":"Applying edits (3/7)…","pct":43}
{"type":"progress","stage":"rendering","label":"Rendering sections…","pct":70}
{"type":"progress","stage":"stitching","label":"Stitching pages…","pct":90}
{"type":"result", ...WorkflowResponse fields... }
```

**Contract:**
- Without `--progress-events` (default): only the final `WorkflowResponse` JSON line is emitted. Fully backward-compatible.
- With `--progress-events`: N progress lines followed by exactly one `{"type":"result", ...}` final line.
- `pct` is `null` for indeterminate stages; integer 0–100 for stages with known item counts.
- The Tauri backend reads subprocess stdout with `BufReader::lines()` in a spawned thread, calling `app_handle.emit("pipeline_progress", line)` per progress line and resolving the command future on the `{"type":"result"}` line.
- The frontend subscribes via `listen("pipeline_progress", handler)` to update the stage label and progress bar.

**Backend-cli changes (Sprint 11.3.G):**
- `--progress-events` boolean flag added to global args in `apps/backend-cli/src/main.rs`
- `emit_progress(stage, label, pct)` helper in `apps/backend-cli/src/handlers/mod.rs` — writes one JSONL line to stdout only when the flag is active
- Wired at stage boundaries: `apply_addendum.rs` (5 stages: extract, segment, edit, render, stitch), `apply_sheet_addendum.rs` (3 stages: extract, index, stitch), `extract_submittal.rs` (3 stages: extract, index, export)

---

### 4.8 Manifest Assistant (Specs Workflow)

The spec addendum workflow requires an `AddendumManifest` encoding per-section edit operations — domain knowledge that cannot be auto-generated. The Manifest Assistant eliminates blank-JSON authoring friction while keeping domain authority with the user.

**Mini-flow (embedded in Step 1 of the four-step UX):**

1. **Select:** User picks the spec PDF and selects "Apply Spec Addendum" workflow.
2. **Analyze:** GUI calls `cmd_extract` + `cmd_segment` in background. Shows inline spinner "Analyzing document…". Session state → `ManifestDraft`.
3. **Show sections:** Detected section IDs, titles, and page ranges displayed in a collapsible panel — the user sees exactly what the engine found, with real section IDs.
4. **Generate template:** "Generate Manifest Template" button → save dialog → writes a JSON file with all detected section IDs pre-populated and empty `ops: []` arrays, ready to fill in.
5. **Author offline:** User edits the template in any text editor, adding `EditOperation` entries for the sections they want to change. Reference: `docs/WORKFLOW_APPLYADDENDUM.md` (`AddendumManifest` schema).
6. **Load + validate:** "Load Manifest" file picker reads the filled-in JSON and calls `cmd_validate_manifest(manifest_json, segment_index_json)`:
   - Checks all `section_id` values exist in the detected `SegmentIndex`
   - Validates operation types and required fields
   - Shows inline error list (e.g., `"Section '23 99 99' not found in document"`) or `"✓ Manifest valid — N sections targeted"` badge
7. **Advance:** "Next →" enables only when a manifest is loaded and passes validation. Session state → `WorkflowReady`.

**Drawings:** GUI synthesizes `DrawingAddendumManifest` from the two file picker inputs. User never writes JSON.

**Submittals:** No manifest required.

---

## 5. Sprint Plan

### Pre-work — Cleanup (~0.25 day)

**Tasks:**
- Pre-11-A: Update `state-summary.md` Next Focus section (Phase 10 → Phase 11 forward pointer)
- Pre-11-B: Bump `capability-matrix.md` version to `2.5.0`, date to `April 11, 2026`
- Verify `cargo check --workspace` exits 0 (baseline check before any GUI changes)

---

### Sprint 11.0 — Tauri Shell + Frontend Scaffold (~2 days)

**Entry gate:** Gate 0 (contract boundary ready). Track B authorized.

**Purpose:** Get a real Tauri v2 application opening a window with a working IPC round-trip; establish all tooling, plugin registrations, binary bundling, and the first-run probe so all subsequent sprints build on a complete foundation.

**11.0.A — `src-tauri/` Rust binary setup**
- Add `src-tauri/src/main.rs` and `src-tauri/src/lib.rs` (Tauri v2 pattern)
- Add `src-tauri/Cargo.toml` depending on `tauri`, `conset-pdf-contracts`, `serde`, `serde_json`; NOT on `conset-pdf-engine`
- Add `src-tauri/src/commands.rs` — `#[tauri::command] fn run_workflow(request: WorkflowRequest) -> WorkflowResponse`; stubs for now (returns `NOT_IMPLEMENTED`)
- `tauri::Builder::default().invoke_handler(tauri::generate_handler![run_workflow]).run()`
- Update `tauri.conf.json` to reference `src-tauri` binary
- `cargo check` on src-tauri → EXIT 0

**11.0.B — Frontend scaffold**
- `npm create tauri-app@latest` for React + TypeScript + Vite in `apps/desktop-gui/`
- Skeleton `src/App.tsx` renders `<h1>Conset PDF</h1>` + a test button that calls `invoke("run_workflow", ...)` and surfaces the `NOT_IMPLEMENTED` response in a `<pre>` tag
- `npm run dev` opens window, button works, response visible

**11.0.C — Smoke test**
- `#[test] fn tauri_run_workflow_stub_returns_not_implemented()` in `src-tauri/src/commands.rs` — directly calls the command function (not through Tauri runtime), asserts `status == Failed`, `error_code == Some("NOT_IMPLEMENTED")`
- Existing `apps/desktop-gui/src/lib.rs` tests keep passing

**11.0.D — Tauri v2 plugin setup**

Tauri v2 requires each plugin registered in `Cargo.toml`, `package.json`, `lib.rs`, AND `capabilities/default.json`. Register all three plugins Phase 11 needs now, before any feature sprint depends on them:

- `tauri-plugin-dialog` — native file/folder picker (Sprint 11.3 file picker, Sprint 11.6 save dialog)
- `tauri-plugin-shell` — "Open in Explorer/Finder" (Sprint 11.6 export summary)
- `tauri-plugin-fs` — local file reads for audit bundle content (Sprint 11.7 page viewer)

For each plugin: add crate to `src-tauri/Cargo.toml`, add JS package to `package.json` devDependencies, call `.plugin(tauri_plugin_<name>::init())` in `src-tauri/src/lib.rs`, add capability permission in `src-tauri/capabilities/default.json`. Update CSP in `tauri.conf.json` to allow `asset://localhost` for local PNG loading in the page viewer.

Verify: `cargo check --workspace` exits 0 with all three plugins registered.

**11.0.E — Binary + PDFium bundling (Section 4.6)**

- Configure `tauri.conf.json` `bundle.resources` to include `backend-cli` binary (platform-specific name) and PDFium library for all target platforms.
- Update `src-tauri/src/backend_process.rs` `backend_cli_path()` to resolve via `app_handle.path().resource_dir()`.
- Set `PDFIUM_LIB_PATH` to the resource directory before each subprocess spawn.
- Add "Desktop GUI Build" section to `docs/SETUP.md` documenting the `cargo build --release -p backend-cli` prerequisite.
- Smoke-test: `cargo tauri build --debug` places the binary at the resolved resource path.

**11.0.F — First-run environment probe (Section 4.6)**

Implement `probe_environment(cli_path: &Path) -> Result<(), SetupError>` in `src-tauri/src/setup.rs`:
- Runs `Command::new(cli_path).arg("--help").status()` (no PDFium needed)
- On failure: shows a blocking modal ("backend binary not found" / "PDFium library not found") with specific resolution steps before the main window becomes interactive
- Wire into app startup via Tauri `setup` hook

1 unit test: `probe_environment_fails_gracefully_when_binary_missing`.

**11.0.G — Frontend test infrastructure**

Before any component sprint can ship tests, configure:
- `vitest.config.ts`: `environment: 'jsdom'`, `setupFiles: ['src/test-setup.ts']`
- `src/test-setup.ts`: global mock for `@tauri-apps/api/core` `invoke`; exposes `setMockResponse(command: string, response: unknown)` for per-test stub configuration
- Install: `@testing-library/react`, `@testing-library/user-event`, `vitest`, `@vitest/coverage-v8`, `jsdom`
- `npm test` runs Vitest in `--run` mode; verify it exits 0 on the empty test suite

This is a hard prerequisite for every frontend component sprint (11.3–11.7).

**Definition of Done:**
- Tauri window opens: `cargo tauri dev` shows the app with the test button; IPC round-trip returns `NOT_IMPLEMENTED`
- All three Tauri v2 plugins registered; `cargo check --workspace` exits 0
- First-run probe: missing binary shows setup dialog, not silent undefined behavior at first operation
- `backend-cli` + PDFium bundled in resources; `backend_cli_path()` resolves correctly
- `npm test` exits 0 (empty suite passes)
- 1 new Rust unit test passes

---

### Sprint 11.1 — Session State Model (~1.5 days)

**Entry gate:** Gate 0. Track A.

**Purpose:** Define the pure Rust session state types and all state transitions. No UI yet — these types become the single source of truth for the frontend reducer.

**11.1.A — `crates/ir/src/session.rs`**

New types (fully serializable, schema-versioned):

```rust
pub enum WorkflowType { SpecsAddendum, DrawingAddendum, SubmittalExtract }

pub struct FileEntry {
    pub path: PathBuf,
    pub file_stem: String,
    pub workflow_hint: Option<WorkflowType>,
    pub valid: bool,
    pub error: Option<String>,
}

pub struct ManifestRef {
    pub path: PathBuf,
    pub manifest_type: WorkflowType,
}

pub struct ProgressState {
    pub stage: String,          // "Extracting...", "Segmenting...", etc.
    pub pct: Option<u8>,        // None = indeterminate
    pub processed: usize,
    pub total: usize,
}

pub enum ReviewItemStatus { NeedsReview, Confirmed, Skipped }

pub struct ReviewItem {
    pub id: String,
    pub source: String,         // "section" | "sheet" | "unit"
    pub label: String,
    pub confidence: f64,
    pub status: ReviewItemStatus,
    pub page: usize,
    pub warning_text: String,
    pub bbox: Option<crate::types::BBox>,
    pub audit_evidence: Option<String>,
}

pub struct WorkflowResult {
    pub workflow_type: WorkflowType,
    pub output_path: Option<PathBuf>,
    pub audit_bundle_dir: Option<PathBuf>,
    pub succeeded: usize,
    pub failed: usize,
    pub warnings: usize,
    pub items: Vec<ReviewItem>,
}

pub struct ExportSummary {
    pub path: PathBuf,
    pub format: String,
    pub items_exported: usize,
    pub items_skipped: usize,
}

pub struct DetectedSection {
    pub id: String,          // e.g., "23 82 16"
    pub title: String,
    pub page_range: (usize, usize),
}

pub enum SessionState {
    Idle,
    FilesAdded { files: Vec<FileEntry>, workflow_type: Option<WorkflowType> },
    ManifestDraft {                          // Specs workflow only
        files: Vec<FileEntry>,
        sections: Vec<DetectedSection>,      // from auto-segment analysis
        manifest: Option<ManifestRef>,       // set after user loads+validates
    },
    WorkflowReady { files: Vec<FileEntry>, workflow_type: WorkflowType, manifest: Option<ManifestRef> },
    Processing { progress: ProgressState },
    ReviewReady { result: WorkflowResult },
    Exported { result: WorkflowResult, summary: ExportSummary },
}
```

**11.1.B — State transition functions**

Pure functions with no side effects:

```rust
pub fn add_files(state: &SessionState, files: Vec<FileEntry>) -> SessionState
pub fn confirm_workflow(state: &SessionState, workflow_type: WorkflowType, manifest: Option<ManifestRef>) -> Result<SessionState, String>
pub fn begin_segment_analysis(state: &SessionState) -> Result<SessionState, String>  // FilesAdded (Specs) → ManifestDraft
pub fn segment_analysis_complete(state: &SessionState, sections: Vec<DetectedSection>) -> SessionState  // populates ManifestDraft.sections
pub fn load_manifest(state: &SessionState, manifest: ManifestRef) -> Result<SessionState, String>  // ManifestDraft → WorkflowReady
pub fn start_processing(state: &SessionState) -> Result<SessionState, String>
pub fn update_progress(state: &SessionState, progress: ProgressState) -> SessionState
pub fn complete_with_result(state: &SessionState, result: WorkflowResult) -> SessionState
pub fn confirm_review_item(state: &SessionState, item_id: &str) -> SessionState
pub fn skip_review_item(state: &SessionState, item_id: &str) -> SessionState
pub fn export_complete(state: &SessionState, summary: ExportSummary) -> SessionState
pub fn reset(state: &SessionState) -> SessionState
```

**11.1.C — `crates/ir/src/lib.rs`** — add `pub mod session;` + re-exports

**11.1.D — Unit tests** (13 minimum)
- `idle_to_files_added_transition`
- `files_added_drawings_to_workflow_ready`
- `files_added_specs_triggers_manifest_draft`
- `manifest_draft_sections_populated_on_analysis_complete`
- `manifest_draft_to_workflow_ready_on_valid_manifest_load`
- `manifest_draft_load_invalid_manifest_returns_error`
- `workflow_ready_to_processing`
- `processing_progress_update`
- `processing_to_review_ready_on_success`
- `processing_to_review_ready_on_partial_success`
- `review_confirm_item_updates_status`
- `review_skip_item_updates_status`
- `reset_from_any_state_returns_idle`

**11.1.E — TypeScript type binding via `specta`**

Add `specta = "2"` and `tauri-specta = { version = "2", features = ["derive", "typescript"] }` to `src-tauri/Cargo.toml`. Annotate all session types exported to TypeScript with `#[derive(specta::Type)]`. Add a `bindings.rs` export call in `src-tauri/build.rs` (dev builds only) that generates `src/bindings.ts` at build time.

`src/bindings.ts` becomes the canonical TypeScript type source for all session + contract types. Manual TypeScript definitions for these types are **prohibited** — only the generated bindings file is authoritative. Add `src/bindings.ts` to `.gitignore` and regenerate at build time.

**Test delta:** IR unit ~50 → 63 (+13 session state tests)

---

### Sprint 11.2 — Tauri Command Layer + Backend Process Integration (~2 days)

**Entry gate:** Gate 0 (Track A — command shapes) + Gate 1 (Track B — real execution).

**Purpose:** Replace the stub `run_workflow` command with real command handlers that spawn `backend-cli` as a subprocess and parse its JSON output. Define the complete set of Tauri commands needed for Lane 1.

**11.2.A — `src-tauri/src/backend_process.rs`**

```rust
/// Spawns the bundled backend-cli binary with given arguments.
/// Returns stdout as a parsed serde_json::Value.
pub fn run_backend(args: &[&str], pdfium_lib_path: &str) -> Result<Value, String>

/// Path to bundled backend-cli binary (resolved relative to app resource dir).
pub fn backend_cli_path(app_handle: &tauri::AppHandle) -> PathBuf
```

- Uses `std::process::Command` to spawn `backend-cli`
- Sets `PDFIUM_LIB_PATH` env var from app resource directory
- Captures stdout + stderr; parses stdout as JSON
- If exit code != 0, returns `Err(stderr_string)`
- 3 unit tests: `backend_path_resolves`, `successful_exit_parses_json`, `nonzero_exit_returns_error`

**11.2.B — `src-tauri/src/commands.rs`** — Full command set

```rust
#[tauri::command]
fn cmd_extract(app: AppHandle, input: String, output: String) -> Result<WorkflowResponse, String>

#[tauri::command]
fn cmd_segment(app: AppHandle, input: String, output: String) -> Result<WorkflowResponse, String>

#[tauri::command]
fn cmd_index_drawing(app: AppHandle, input: String, output: String) -> Result<WorkflowResponse, String>

#[tauri::command]
fn cmd_index_submittal(app: AppHandle, input: String, output: String) -> Result<WorkflowResponse, String>

#[tauri::command]
fn cmd_apply_addendum(app: AppHandle, original: String, addendum: String, output: String, audit_bundle: String, dry_run: bool) -> Result<WorkflowResponse, String>

#[tauri::command]
fn cmd_apply_sheet_addendum(app: AppHandle, manifest: String, output: String, audit_bundle: String, dry_run: bool) -> Result<WorkflowResponse, String>

#[tauri::command]
fn cmd_extract_submittal(app: AppHandle, input: String, index: String, output: String, format: String, audit_bundle: String, dry_run: bool) -> Result<WorkflowResponse, String>

#[tauri::command]
fn cmd_open_file_dialog(app: AppHandle, title: String, filters: Vec<FileFilter>) -> Result<Option<Vec<String>>, String>

#[tauri::command]
fn cmd_save_file_dialog(app: AppHandle, title: String, default_name: String) -> Result<Option<String>, String>

#[tauri::command]
fn cmd_validate_manifest(manifest_json: String, segment_index_json: String) -> Result<ManifestValidationResult, String>
// ManifestValidationResult { valid: bool, errors: Vec<String>, sections_targeted: usize }
// Validates section IDs exist in SegmentIndex + operation schema. Lives in crates/contracts.

#[tauri::command]
fn cmd_visualize(app: AppHandle, input: String, output_dir: String) -> Result<WorkflowResponse, String>
// Runs backend-cli visualize to generate per-page span-detection PNG overlays (Sprint 11.7).
```

The `WorkflowResponse` type comes from `conset-pdf-contracts`. No engine types imported.

**11.2.C — `src-tauri/src/main.rs`** — register all commands in `generate_handler![]`

**11.2.D — Integration tests for command layer**

- `cmd_extract_dry_run_returns_success` — spawns real backend-cli with a test fixture PDF, dry-run mode; asserts response status is not `Failed`, output path present
- `cmd_apply_addendum_missing_file_returns_error` — bogus path; asserts `error_code` present

**11.2.E — Subprocess lifecycle management**

Add `AppState { active_child: Mutex<Option<std::process::Child>> }` to Tauri managed state. When a command spawns a subprocess, store the handle. Register a `window.on_window_event` handler for `WindowEvent::CloseRequested`:

- If `active_child` is `Some`: show confirmation dialog ("An operation is in progress. Cancel and exit?").
  - Confirmed: `child.kill()` + `child.wait()` + `app.exit(0)`
  - Declined: cancel the close event (operation continues)
- If `active_child` is `None`: `app.exit(0)` directly

On operation completion (success or error): clear `active_child` from state.

2 unit tests: `close_while_idle_exits_without_confirmation`, `active_child_cleared_on_operation_completion`.

**Test delta:** src-tauri tests: +5 unit + 2 integration + 2 lifecycle

---

### Sprint 11.3 — Lane 1A: Add Files + Workflow Selection + Manifest Assistant (~3 days)

**Entry gate:** Gate 0. Lane 1 scope.

**Purpose:** Build the first step of the four-step UX, including the Manifest Assistant flow for specs. User adds files, the GUI detects workflow type, specs users walk through the section-analysis/template/load flow, drawings users pick two PDFs, submittals users pick one PDF. Session state updates for all three paths.

**11.3.A — File picker UI (`src/components/FilePicker.tsx`)**
- Drag-and-drop zone + "Browse files" button
- "Browse files" calls `cmd_open_file_dialog` (Tauri dialog, PDF filter)
- Drag-and-drop uses HTML drag events + Tauri path validation
- Per-file validation: checks `.pdf` extension, warns if file size > 50 MB
- Workflow type hint shown per file based on stem prefix (`SPEC_`, `DWG_`, `SUB_`) — advisory label only, not binding
- File list displays: filename, size, workflow hint badge, remove button

**11.3.B — Workflow selection panel (`src/components/WorkflowSelector.tsx`)**
- Three buttons: "Apply Spec Addendum", "Apply Drawing Addendum", "Extract Submittal Data"
- **Drawings:** selecting this shows a second file picker for the "Addendum PDF" (the GUI synthesizes `DrawingAddendumManifest` from the two paths — no JSON authoring required)
- **Submittals:** no additional input required (one PDF, output format chosen at export time)
- **Specs:** selecting this triggers the Manifest Assistant flow (11.3.C) inline
- Selected workflow highlighted; clear visual affordance for the active path

**11.3.C — Manifest Assistant (`src/components/ManifestAssistant.tsx`, Specs workflow only)**

Implements the Section 4.8 mini-flow:

- On specs workflow selection: immediately call `cmd_extract` + `cmd_segment` in background. Show inline spinner "Analyzing document…". Session → `ManifestDraft`.
- On completion: display a collapsible section list (ID + title + page count). User can browse the document structure before authoring any manifest.
- "Generate Manifest Template" button → `cmd_save_file_dialog` → write JSON template with all detected section IDs pre-populated, `ops: []` arrays, and inline comments pointing to `WORKFLOW_APPLYADDENDUM.md` schema reference.
- "Load Manifest" button → `cmd_open_file_dialog` (`.json` filter) → read file → call `cmd_validate_manifest(manifest_json, segment_index_json)` → show inline error list OR `"✓ Manifest valid — N sections targeted"` badge. Session → `WorkflowReady` on success.

Component tests:
- `manifest_assistant_shows_spinner_while_analyzing`
- `manifest_assistant_shows_section_list_after_analysis`
- `manifest_assistant_generate_template_opens_save_dialog`
- `manifest_assistant_load_invalid_manifest_shows_errors`
- `manifest_assistant_load_valid_manifest_shows_success_badge`

**11.3.D — Session reducer integration**
- `useReducer` with session state machine from Sprint 11.1 (via `src/bindings.ts` — generated types)
- `ADD_FILES` → `filesAdded`; `SET_WORKFLOW` → triggers Manifest Assistant for specs or moves directly to `workflowReady` for drawings/submittals
- `AUTO_SEGMENT_COMPLETE` → `manifestDraft` (sections populated)
- `LOAD_MANIFEST` → `workflowReady`
- State preserved in `sessionStorage` (survives page reload; cleared on window close)

**11.3.E — Next button (conditional)**
- For specs: enabled when `ManifestDraft` has a valid manifest loaded
- For drawings: enabled when two valid PDFs selected
- For submittals: enabled when one valid PDF selected
- Disabled state shows tooltip explaining exactly what is missing

**11.3.F — Component tests (Vitest + Testing Library)**
- `file_picker_renders_empty_state`
- `file_picker_accepts_pdf_and_shows_workflow_hint`
- `file_picker_rejects_non_pdf_with_warning`
- `workflow_selector_drawings_shows_second_file_picker`
- `workflow_selector_submittals_requires_no_manifest`
- `next_button_disabled_for_specs_without_manifest`
- `next_button_enabled_for_drawings_with_two_pdfs`
- `next_button_enabled_for_submittals_with_one_pdf`

**11.3.G — `--progress-events` protocol for `backend-cli`** *(backend prerequisite for Sprint 11.4)*

Implements the Option B streaming protocol (Section 4.7). Pure backend-cli/handlers change — no engine modifications:

- `--progress-events` boolean flag added to `apps/backend-cli/src/main.rs` global args (default: off, fully backward-compatible)
- `fn emit_progress(stage: &str, label: &str, pct: Option<u8>)` added to `apps/backend-cli/src/handlers/mod.rs`; writes `{"type":"progress",...}\n` to stdout only when the flag is active
- Wired in `apply_addendum.rs` (5 calls: before extract, segment, edit, render, stitch)
- Wired in `apply_sheet_addendum.rs` (3 calls: before extract, index-drawing, stitch)
- Wired in `extract_submittal.rs` (3 calls: before extract, index-submittal, export)

3 integration tests: `progress_events_specs_emits_five_stages`, `progress_events_drawings_emits_three_stages`, `progress_events_disabled_by_default_no_extra_output`.

**Test delta:** CLI active +3 (progress events); frontend component tests +13 (11.3.C + 11.3.F)

---

### Sprint 11.4 — Lane 1B: Start Processing (~2 days)

**Entry gate:** Gate 1 (baseline pipeline ready). Track B.

**Purpose:** The "one button" experience. Clicking Start triggers the full operation chain for the selected workflow. Progress is shown. Results arrive. Partial success is handled gracefully.

**11.4.A — Start button + progress display (`src/components/ProcessingView.tsx`)**
- "Start" button visible when in `WorkflowReady` state
- On click: session transitions to `Processing`, backend command invoked
- Progress display: spinner + stage label + optional percentage
- Stage labels mapped from `DiagnosticEvent` stage field ("Extracting layout...", "Segmenting sections...", "Applying edits...", "Rendering...", "Stitching...")
- Cancel button (sends SIGTERM to backend subprocess and resets to `WorkflowReady`)

**11.4.B — Operation chain wiring per workflow**

`apply-addendum` and `apply-sheet-addendum` each run their own full extraction + indexing pipeline internally. Calling `cmd_extract` before them would double-extract the source PDF for no benefit. Do **not** pre-run `cmd_extract` for Specs or Drawings workflows.

*Specs workflow (single command — full pipeline internal):*
```
cmd_apply_addendum(original, manifest, output, audit_bundle, progress_events=true)
  → runs extract + segment + parse + edit + render + stitch internally
  → emits progress events via --progress-events (Section 4.7)
  → WorkflowResult
```

*Drawings workflow (single command — full pipeline internal):*
```
cmd_apply_sheet_addendum(manifest, output, audit_bundle, progress_events=true)
  → runs extract + index-drawing + stitch internally
  → emits progress events via --progress-events
  → WorkflowResult
```

*Submittals workflow (three commands — explicit extract):*
```
cmd_extract(input, transcript_path, progress_events=true)      → transcript.json
cmd_index_submittal(transcript_path, index_path)               → submittal-index.json
cmd_extract_submittal(transcript_path, index_path, output, format, audit_bundle)
  → WorkflowResult
```

Progress events are consumed from subprocess stdout line-by-line by a `BufReader::lines()` loop on a spawned thread and re-emitted as Tauri window events. The frontend subscribes via `listen("pipeline_progress", handler)` to update the stage label and progress bar.

**11.4.C — Result parsing**

The `WorkflowResult` type (from `crates/ir/src/session.rs`, serialized as JSON, parsed on frontend) is built from:
- Specs: `change-report.json` → per-section results → `ReviewItem` list (failed/warned sections become review queue)
- Drawings: `change-report.json` → per-sheet results → `ReviewItem` list
- Submittals: `unit-report.json` from audit bundle → per-unit summaries → `ReviewItem` list (low-confidence units flagged)

**11.4.D — Partial success display**
- If `result.failed > 0` or `result.warnings > 0`: banner visible ("X sections could not be processed — review below")
- If `result.succeeded == 0`: error state with actionable message + option to restart
- If `result.succeeded > 0` (even with failures): "Continue to Review" button enabled

**11.4.E — Tests**
- `processing_view_shows_spinner_on_start`
- `processing_view_shows_stage_label_on_progress_event`
- `processing_view_shows_partial_success_banner_when_some_failed`
- `processing_view_shows_error_state_when_all_failed`

---

### Sprint 11.5 — Lane 1C: Review Queue (~2 days)

**Entry gate:** Gate 2 (review inputs ready). Track B.

**Purpose:** Show the user what needs attention. Confidence-flagged items from the backend are presented one at a time (or as a list). User can confirm, skip, or note an edit for manual follow-up.

**11.5.A — Review queue list (`src/components/ReviewQueue.tsx`)**
- Items sorted: `needs_review` first (lowest confidence first), then `confirmed`, then `skipped`
- Each item shows:
  - Label (section ID / sheet ID / unit tag)
  - Confidence indicator pill (green ≥ 0.9, yellow 0.7–0.9, red < 0.7)
  - Warning text (plain language)
  - Page number
  - Action buttons: "Confirm" / "Skip" / "Note edit needed"
- Item count badge: "3 need review / 12 total"
- If 0 items need review: "All items accepted — ready to export" message

**11.5.B — Item detail panel (`src/components/ReviewItemDetail.tsx`)**
- Click any item → side panel opens with:
  - Page number and bounding box location (text description: "Upper-right quarter of page 45")
  - The warning text (expanded)
  - "Advanced" toggle (hidden by default):
    - Raw confidence score
    - `audit_evidence` field (diagnostic text from `diagnostics.jsonl`)
    - Extracted value with bbox coordinates
- No embedded PDF renderer in v1.0 (image overlays from Sprint 11.7 add this)

**11.5.C — "Note edit needed" action**
- Opens a text input field: "Describe what needs manual attention"
- Free-text note saved to a local `review-notes.json` alongside the audit bundle
- These notes do NOT mutate the output — they are tracked as "manual action required" items
- Exported in the export summary as pending manual actions

**11.5.D — Session state updates**
- `CONFIRM_REVIEW_ITEM` + `SKIP_REVIEW_ITEM` + `NOTE_REVIEW_ITEM` actions
- "Export" button enabled when: no `needs_review` items remain (all are `confirmed`, `skipped`, or `noted`)

**11.5.E — Tests**
- `review_queue_shows_needs_review_items_first`
- `review_queue_confirm_item_moves_it_to_confirmed`
- `review_queue_skip_item_moves_it_to_skipped`
- `review_queue_export_button_disabled_while_items_need_review`
- `review_queue_export_button_enabled_when_all_items_resolved`

---

### Sprint 11.6 — Lane 1D: Export (~1 day)

**Entry gate:** Gate 3 (export inputs stable). Track B.

**Purpose:** Save the results to disk. User picks a save location. Export summary shows what was written, what was skipped, and what needs manual attention.

**11.6.A — Export action (`src/components/ExportView.tsx`)**
- "Export Results" button → `cmd_save_file_dialog` → user picks save location
- Format by workflow:
  - **Specs:** Output PDF already written by `apply-addendum`; "Save As…" copies it to chosen location
  - **Drawings:** Output PDF already written by `apply-sheet-addendum`; "Save As…"
  - **Submittals:** "JSON" / "CSV" radio selector → `cmd_extract_submittal --format <json|csv>` to chosen output path (or copy if already generated)
- "Also export audit bundle" checkbox (copies `audit_bundle_dir` contents to chosen folder)

**11.6.B — Export summary display**
- After export: summary panel shows:
  - Path where output was saved
  - Item counts: exported, skipped, pending manual action (from review notes)
  - "Open in Explorer/Finder" button (uses Tauri `open` shell command)
  - "Start New Job" button → resets to `Idle`
- Pending manual actions listed: section/sheet/unit label + note text

**11.6.C — Partial-success export**
- If any items were failed (not skipped), they appear in "What was not exported" section with reason
- Export still proceeds with available results — partial success is real success per MASTER_PLAN

**11.6.D — Tests**
- `export_view_shows_save_dialog_on_click`
- `export_view_shows_summary_after_export`
- `export_view_shows_pending_manual_actions`
- `export_view_start_new_job_resets_session`

---

### Sprint 11.7 — Overlay Visualization (~2 days)

**Purpose:** Surface the audit trail visually. When a user clicks a review item, they should see the relevant page with the detected region highlighted. This closes the "trust gap" — users can verify the engine's work.

**Entry gate:** Gate 2. Lane 2 (advanced audit visualization).

**11.7.A — Generate overlay PNGs via post-processing `visualize` call**

Span-detection PNG overlays are **not** generated automatically during `apply-addendum` or other workflows — they require an explicit `backend-cli visualize` invocation. After a job completes and the audit bundle directory is available:

- The Tauri backend enqueues a background `cmd_visualize(transcript_path, overlays_dir)` call (non-blocking—does not delay the review queue appearing)
- A "Generating review overlays…" status label is shown while PNGs are written
- When complete, a "View page" button appears on each `ReviewItem` that has a page number
- PNGs are cached in `<audit_bundle_dir>/overlays/page-NNN.png`; not regenerated on subsequent sessions if the directory exists
- `cmd_visualize` was added to the Tauri command set in Sprint 11.2.B

**11.7.B — Page viewer component (`src/components/PageViewer.tsx`)**
- Displays a PNG image sourced from the audit bundle directory via Tauri `asset://` protocol
- Navigation: previous/next page buttons
- Zoom: CSS transform scale, two preset levels (fit-to-width, 1:1)
- Loaded on demand (not all pages pre-loaded)

**11.7.C — Review item → page link**
- When a `ReviewItem` has a page number, its detail panel shows a "View page" button
- Clicking opens `PageViewer` scrolled to the correct page image
- If bbox is available, a CSS overlay (`<div>` with `position: absolute`, `border: 2px solid orange`) is drawn on top of the image at the normalized bbox coordinates

**11.7.D — Overlay image sourcing**
The Tauri security policy must be updated in `tauri.conf.json` to allow reading from the audit bundle directory via `asset://` or using the `fs` Tauri plugin for local file loading.

**11.7.E — Tests**
- `page_viewer_renders_image_for_given_path`
- `page_viewer_navigates_to_next_page`
- `review_item_view_page_button_opens_page_viewer`
- `page_viewer_shows_bbox_overlay_when_item_has_bbox`

---

### Sprint 11.8 — Determinism + Integration Tests + Parity Checks (~1 day)

**Purpose:** Verify that the GUI-driven workflow produces the same output as the CLI. Confirm Lane 1 promotion criteria.

**11.8.A — GUI determinism test**

A Tauri integration test (or Rust unit test in src-tauri that calls command functions directly):

- `gui_apply_addendum_twice_produces_identical_change_report` — invokes `cmd_apply_addendum` twice with the same inputs; asserts `change-report.json` outputs are byte-equivalent (excluding timestamps) — same test discipline as the CLI determinism tests.
- `gui_extract_submittal_twice_produces_identical_dataset` — same two-run comparison for submittals.
- `gui_apply_sheet_addendum_twice_produces_identical_change_report` — same for drawings.

**11.8.B — Parity check**

Document evidence that the GUI produces the same outputs as the CLI for all three workflows:

```
GUI flow (apply-addendum):
  cmd_extract → cmd_apply_addendum
  Yields: output.pdf + audit_bundle/change-report.json

CLI flow (equivalent):
  backend-cli extract -i spec.pdf -o transcript.json
  backend-cli apply-addendum --original spec.pdf --addendum manifest.json -o output.pdf --audit-bundle audit/
  
Assert: change-report.json content is byte-equivalent (excluding timestamps).
```

This parity check is documented in the Phase_11_Plan — it does NOT need to be an automated test, but it MUST be manually verified with evidence before Lane 1 is declared production-ready.

**11.8.C — Lane 1 promotion checklist**

Before Phase 11 is marked BETA COMPLETE, verify:

| Gate | Criterion | Verified |
|---|---|---|
| Gate 0 | Contracts stable | ✅ (Phase 10) |
| Gate 1 | Baseline pipeline closed | ✅ (Phase 10) |
| Gate 2 | Review inputs consumable | ✅ (Phase 10) |
| Gate 3 | Export inputs stable | ✅ (Phase 10) |
| Lane 1 | Integration tests pass twice consecutively | ☐ (Sprint 11.8) |
| Lane 1 | Determinism checks pass twice consecutively | ☐ (Sprint 11.8) |
| Lane 1 | Parity with CLI documented and verified | ☐ (Sprint 11.8) |
| Lane 1 | No silent-failure regressions | ☐ (Sprint 11.8) |

---

### Sprint 11.9 — Documentation + Phase Close (~0.5 day)

**Purpose:** Close Phase 11. Update all canonical docs. Publish user guide.

**11.9.A — `docs/GUI_GUIDE.md`** (new)
- Prerequisites: Tauri runtime, PDFium library path setup
- Installation: downloading the installer, first launch
- Four-step workflow walkthrough with screenshots (or ASCII placeholders until screenshots available)
- Specs workflow: step-by-step with addendum manifest JSON example
- Drawings workflow: step-by-step with `DrawingAddendumManifest` JSON example
- Submittals workflow: step-by-step, output format selection
- Confidence indicator guide (what green/yellow/red means)
- Review queue usage: when to confirm, skip, or note
- Overlay visualization: how to use the page viewer
- Export summary interpretation
- Troubleshooting: PDFium path not found, Chrome not found, backend-cli not found

**11.9.B — `docs/MASTER_PLAN.md`** — Phase 11 deliverables marked ✅

**11.9.C — `docs/current-state/state-summary.md`** — Phase 11 COMPLETE section added

**11.9.D — `docs/current-state/capability-matrix.md`** — GUI capability rows added:
- Desktop GUI shell (Tauri v2 + React) → Implemented
- Session state model → Implemented
- Lane 1: Add files + workflow selection → Implemented
- Lane 1: Start processing (one-button) → Implemented
- Lane 1: Review queue → Implemented
- Lane 1: Export → Implemented
- Overlay visualization (Lane 2) → Implemented

**11.9.E — `CHANGELOG.md`** — Sprint 11.9 entry added

**11.9.F — Repo memory update** — Update `/memories/repo/conset-pdf-architecture.md` with Phase 11 test baselines and new crate/structure notes.

---

## 6. Definition of Done

### Phase 11 DoD

| Row | Criterion | Sprint | Status |
|---|---|---|---|
| 1 | Tauri window opens; test IPC round-trip works (NOT_IMPLEMENTED → stub response) | 11.0 | ☐ |
| 1b | Tauri v2 plugins (dialog, shell, fs) registered; `cargo check --workspace` clean | 11.0 | ☐ |
| 1c | First-run probe: missing binary shows setup dialog before any operation | 11.0 | ☐ |
| 1d | `backend-cli` + PDFium bundled in resources; `backend_cli_path()` resolves at runtime | 11.0 | ☐ |
| 1e | Vitest + invoke mock: `npm test` exits 0 (empty suite passes) | 11.0 | ☐ |
| 2 | Session state machine: all 13 transitions tested, all pass (incl. ManifestDraft path) | 11.1 | ☐ |
| 2b | `specta` bindings auto-generated; `src/bindings.ts` is canonical TS type source | 11.1 | ☐ |
| 3 | All 9 Tauri commands wired to subprocess backend; cmd_extract dry-run test passes | 11.2 | ☐ |
| 3b | Subprocess lifecycle: close-while-processing shows confirmation; zombie-free exit | 11.2 | ☐ |
| 4 | File picker opens native dialog; PDF files shown with hints; non-PDF rejected | 11.3 | ☐ |
| 5 | Workflow type selection works for all 3 workflows; Next button gate enforced | 11.3 | ☐ |
| 5b | Manifest Assistant: spec PDF segmented, section list shown, template JSON downloadable | 11.3 | ☐ |
| 5c | Manifest validation errors shown inline; Start unreachable until manifest validates | 11.3 | ☐ |
| 5d | `--progress-events`: all 3 workflows emit correct stage sequence; disabled by default | 11.3 | ☐ |
| 6 | Start button triggers full operation chain for specs workflow on real addendum | 11.4 | ☐ |
| 7 | Start button triggers full operation chain for drawings workflow on real addendum | 11.4 | ☐ |
| 8 | Start button triggers full operation chain for submittals workflow on real submittal | 11.4 | ☐ |
| 9 | Partial success displayed correctly (some sections fail, rest export) | 11.4 | ☐ |
| 10 | Review queue shows confidence-flagged items; Confirm/Skip/Note actions work | 11.5 | ☐ |
| 11 | Export saves correct output to user-chosen path; summary shows pending manual actions | 11.6 | ☐ |
| 12 | Overlay PNG viewer shows page with bbox annotations linked from review items | 11.7 | ☐ |
| 13 | GUI determinism tests pass twice consecutively for all 3 workflows | 11.8 | ☐ |
| 14 | CLI parity verified and documented for all 3 workflows | 11.8 | ☐ |
| 15 | `docs/GUI_GUIDE.md` published; MASTER_PLAN Phase 11 deliverables marked ✅ | 11.9 | ☐ |

### V1.0 Success Criteria

Per `MASTER_PLAN.md` Beta/V1.0 definition:

> "Ready to charge for it. First paying customer onboarded."

Phase 11 completion means:
- An AEC professional can add a spec PDF + addendum manifest, click Start, see section-level results, confirm/skip flagged items, and export the updated PDF — without touching the CLI.
- Same one-button experience for drawing addenda and submittal data extraction.
- All outputs are byte-identical to the CLI output (parity verified).
- No silent failures — all flags surface in the review queue.
- Audit trail accessible via overlay visualization from within the GUI.

---

## 7. Open Gaps Relevant to Phase 11

These open gaps from the gap register affect Phase 11. They are categorized by impact.

### Directly Addressed in Phase 11

| Gap | Description | Sprint |
|---|---|---|
| G-032 | Provenance-first review object model and UI contract | Sprint 11.5 (review item model + detail panel) |
| G-036 | Confidence policy profiles exposed to users | Sprint 11.5 (confidence threshold constants surfaced in review queue UI) |

### Deferred to Phase 12+ (Not Blocking Lane 1)

| Gap | Description | Deferral Rationale |
|---|---|---|
| G-011 | Autonomous deterministic ROI candidate generation | Hard technical problem; Lane 1 works with existing title-block detection |
| G-014 | Batch addenda assembly + deterministic ordering | Multi-addenda workflows are Phase 12 scope |
| G-015 | Autonomous document-type classification manifest | Advisory classification for GUI file picker is Sprint 11.3 hint only |
| G-017 | Cross-medium addenda merge workflow | Phase 12+ when both spec and drawing addenda present in one bundle |
| G-018, G-019, G-020 | Full vector-first title-block detection and scoring | Current detection works on corpus fixtures; G-018/19/20 improve accuracy |
| G-021 | Auto-learned firm/layout template store | Pattern database management UI is Phase 12 |
| G-023–G-026 | AI fallback, micro-ML, LLM validation paths | Explicit opt-in paths; not needed for baseline one-button workflows |
| G-027 | Raster OCR extraction path | `SUB_LHHS_MCMJ_CARRIER-UV` (raster risk) already handled by graceful degradation |
| G-029 | XML export adapter | JSON + CSV sufficient for v1.0; XML is Phase 12 |
| G-030 | Replayable correction manifests | Phase 12 — review notes from Sprint 11.5 are a precursor |
| G-031 | Native diff/compare workflows | Phase 12+ higher-order workflows (Lane 3) |
| G-033 | Redaction/privacy outbound controls | Phase 12 — needed for enterprise / LLM integration paths |
| G-034 | Batch orchestration and resumability | Phase 12 — Lane 3 multi-file batch jobs |
| G-035 | Instruction DSL / manifest language | Phase 12 — advanced automation surface |
| G-037 | Standards normalization wiring | Phase 12 — beneficial for submittal classification |
| G-038 | Cross-document entity resolution | Phase 13+ strategic infrastructure |
| G-039 | Project knowledge index and triage queue | Phase 13+ strategic infrastructure |

### Gaps Partially Addressed (Lane 2 consideration)

| Gap | Partial Work in Phase 11 | Full Closure Sprint |
|---|---|---|
| G-032 | Review item model + detail panel built in Sprint 11.5; full provenance resolver deferred | Phase 12 (Lane 2 advanced provenance) |

---

## Sprint Sequence and Estimated Sizing

```
Pre-work  (0.25d): state-summary.md + capability-matrix.md cleanup                         ✅
Sprint 11.0 (2d):  Tauri shell + scaffold + IPC + plugins + probe + bundling + test infra   ☐
Sprint 11.1 (1.5d): Session state model (IR types + ManifestDraft + specta bindings)        ☐
Sprint 11.2 (2d):  Tauri command layer + subprocess integration + lifecycle management       ☐
Sprint 11.3 (3d):  Lane 1A: File picker + workflow selection + Manifest Assistant + progress events  ☐
Sprint 11.4 (2d):  Lane 1B: Start processing + operation chains (corrected) + progress      ☐
Sprint 11.5 (2d):  Lane 1C: Review queue + confidence display + notes                       ☐
Sprint 11.6 (1d):  Lane 1D: Export + summary + partial-success display                      ☐
Sprint 11.7 (2d):  Overlay visualization (explicit visualize call + page viewer)             ☐
Sprint 11.8 (1d):  Determinism tests + integration tests + parity checks                    ☐
Sprint 11.9 (0.5d): GUI_GUIDE.md + MASTER_PLAN update + docs close                         ☐
─────────────────────────────────────────────────────────
Total: ~17 working days (~3.5 weeks)
```

---

## Appendix A — Non-Negotiables Governing This Plan

The following MASTER_PLAN non-negotiables are most relevant to Phase 11 execution. Any implementation decision that conflicts with these must be escalated:

1. **NN-1 (Determinism):** GUI-driven operations must produce byte-identical outputs to CLI operations given the same inputs.
2. **NN-7 (Audit trail first-class):** Every GUI-initiated operation must emit the same audit bundle as the CLI path.
3. **NN-10 (No Python runtime):** All GUI binaries ship as pure Rust + bundled backend-cli. No Python interpreter bundled.
4. **NN-14 (No silent failures):** Every failed section/sheet/unit must appear in the review queue. The "all succeeded" path must not be reachable if any item has unresolved confidence < 0.7.
5. **NN-17 (Partial success is success):** If 80/100 sections succeeded, the export button must be available. The GUI must NEVER discard working results because some operations failed.
6. **NN-18 (Medium detection is user-driven):** Workflow type must be explicitly selected or confirmed by the user. The auto-hint from filename is advisory only. Silent auto-selection into a wrong workflow is prohibited.

---

*End of Phase 11 Plan*
