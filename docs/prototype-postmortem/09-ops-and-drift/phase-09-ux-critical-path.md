# Phase 09 — UX Critical Path and Sharp Edges

**Purpose**: Capture workflow-critical manual test paths and known sharp edges from the GUI prototype. Distinguish correctness-critical UX rules from convenience behaviors. Translate these into acceptance criteria for Rust GUI/API front ends.

---

## 1. Workflow-Critical Manual Test Path (Smoke Test)

The minimum path that must pass before any GUI release. From `docs/DEVELOPMENT.md`:

1. Load a profile
2. Load reference PDF
3. Draw ROI (on profile view)
4. Save profile
5. Run merge operation

**Full Merge Wizard Flow** (Step-by-step):

| Step | Action | Success Indicator |
|------|--------|-----------------|
| Step 1 | Select layout profile; choose scope (drawings/specs/both); optionally select narrative PDF | Profile loaded, scope badge shows, narrative path shown if selected |
| Step 2 | Select original PDF; select single addendum PDF; select output path per lane | Forward navigation triggers analysis; analysis caches result |
| Step 3 | Review inventory table; apply filters (search, confidence, status); optionally ignore rows / override IDs; click Recompute | Inventory reflects corrections; summary counts update (ignored rows excluded) |
| Step 4 | Execute merge; verify results page shows replaced/inserted/unmatched/final page counts | Output file created; no error shown |
| Post | Click "Next Addendum" | Output becomes new base, addenda cleared, wizard returns to Step 2 |

This path must succeed for every release. It is the **only fully implemented GUI workflow** in the prototype.

---

## 2. IPC Envelope Constraints (Correctness-Critical)

### 2.1 Envelope Shape

All IPC handlers must return `IpcResponse<T>`:
```typescript
{ success: true, data: T }         // success
{ success: false, error: { message, code?, stack?, context? } }  // failure
```

Source: `src/shared/ipc-response.ts`

### 2.2 Critical Rules

| Rule | Violation consequence |
|------|----------------------|
| **Never throw from an IPC handler** — always return an error response | If a handler throws, preload receives an undefined result; renderer sees an uncaught error with no message |
| **Always return IpcResponse** — never return raw data or undefined | Preload's `unwrapResponse()` expects the envelope; deviation breaks all error handling silently |
| **Preload unwraps automatically** — renderer code catches `BridgeError` | Renderer developers must catch errors at the `window.wizardAPI.*` call site, not deeper |
| **Error properties are preserved** — `code`, `stack`, `context` from the original Error survive the IPC boundary | Enables renderer to display typed error messages and log structured context |

### 2.3 `wrapIpcHandler` Utility

```typescript
export function wrapIpcHandler<TArgs, TReturn>(
  handler: (...args: TArgs) => Promise<TReturn>
): (...args: TArgs) => Promise<IpcResponse<TReturn>>
```

This wraps any async handler and automatically catches exceptions into `IpcResponse` error format. **All new IPC handlers should use this wrapper.** Handlers that do not use it and throw will escape the envelope contract.

### 2.4 Acceptance Criterion for Rust/Tauri

> All IPC handlers (Tauri commands) must return `Result<T, ErrorEnvelope>` where `ErrorEnvelope` carries: message, code (string typed error variant), and optional context. The frontend must never receive an untyped error string. `tauri::command` functions must never panic — use `Result::Err` for all error paths.

---

## 3. Analysis Caching Lifecycle (Correctness-Critical)

### 3.1 Cache Key

Analysis results are cached with a key computed from:
- `docType`
- `basePath`
- `addendumPath`
- `outputPath`
- `narrativePath`
- Profile identifier

### 3.2 Cache Invalidation Triggers

| Trigger | Effect |
|---------|--------|
| Any field in the cache key changes | Cache invalidated; new analysis required |
| User makes corrections (ignore row / override ID) | Cache invalidated; recompute required |
| User clicks "Recompute" | `merge:analyze` called with correction overlay; cache updated |
| User navigates back to Step 2 and changes files | Cache key changes; analysis will re-run on next forward navigation |

### 3.3 Acceptance Criterion

> Analysis results must never persist across file changes. If the user changes any input (original PDF, addendum PDF, output path, profile) the cached result must be discarded and re-run. Showing stale analysis after input changes is a correctness failure, not a performance optimization.

---

## 4. ROI Overlay Lifecycle (Correctness-Critical)

### 4.1 Source: `src/modules/roi/roiOverlayController.js`

The ROI overlay (the canvas element allowing users to draw detection regions) has a critical lifecycle constraint:

| Event | Required Action |
|-------|----------------|
| Viewport set | Create `RoiOverlay`; bind `onRoiChange` callback |
| Viewport reset (cleanup) | `cleanup()` on existing overlay; set `roiOverlay = null` |
| Canvas changes | Detect via `canvasChanged` flag; re-attach overlay to new canvas |
| `updateROIOverlays()` called before canvas available | **No-op** (single debug log on first skip; not repeated) |

### 4.2 Sharp Edge: Viewport Reset Without Re-bind

If the viewport is reset (e.g., navigating between wizard steps or reloading a PDF page), the `RoiOverlay` becomes detached. The controller's `updateROIOverlays()` gracefully skips the update, but this means ROI drawing is silently non-functional until `attachToCanvas(canvas, container)` is called again.

The code guards against repeated log spam using a `_hasLoggedMissingNodes` flag (logged once per disconnection event).

**Acceptance Criterion**: Any Rust/Tauri ROI overlay implementation must handle canvas lifecycle events explicitly. The overlay must re-bind when the canvas element is replaced. A silent no-op during disconnection is acceptable; a crash or corrupted state is not.

### 4.3 Profile Type Switching

`setRoiTypeKeys(profileType)` switches the ROI field names between `'drawings'` (sheetId/sheetTitle) and `'specs'` (sectionId/sectionTitle). This affects:
- The label text in the overlay UI
- The field names stored in the profile JSON
- The inventory display column headers in Step 3

**Acceptance Criterion**: When the user switches profile type, the ROI overlay must update its key mapping before any ROI is drawn or saved. Saving an ROI under the wrong key is a data corruption issue.

---

## 5. Wizard State Persistence Rules

### 5.1 What Persists

| State | Persistence | Notes |
|-------|------------|-------|
| Layout profiles | Disk (`userData/profiles/`) | Auto-migrated on read; old structure handled |
| Profile snapshot PDFs | Disk (profile folder) | Used for ROI overlay preview |
| Run history | Disk (`userData/history/`) | Previous merge operations |
| Active wizard state | In-memory only | Cleared when wizard is closed or app restarts |

### 5.2 What Does NOT Persist

| State | Notes |
|-------|-------|
| Inventory analysis results | Must be recomputed on next session |
| Correction overlays | User must re-apply corrections in each session |
| File picker selections | Must be re-selected each run |
| Narrative PDF selection | Must be re-selected each run |

### 5.3 Acceptance Criterion

> The wizard must never silently carry over analysis results from a previous session. Opening the wizard must always start from Step 1 (or Step 2 at most with clean file state). Any UX that implies continuity from a previous run (e.g., "Continue where you left off") is risky without explicit analysis revalidation.

---

## 6. Multi-Lane Behavior (Drawings + Specs)

When scope is `'both'`:
- Step 2 shows two separate file pickers (drawings lane and specs lane)
- Analysis runs independently for each lane
- Step 3 becomes 3a (drawings) and 3b (specs) — stepper hides 3b until 3a is completed
- Step 4 executes both lanes sequentially (drawings first, then specs)

**Sharp Edge**: In the prototype, both lanes use the **same layout profile** for ROI keying. A profile typed as `'drawings'` will use `sheetId`/`sheetTitle` keys. A `'specs'` type profile uses `sectionId`/`sectionTitle`. If the user selects a drawings-typed profile for a specs lane, the ROI keys won't match the expected detection fields.

**Acceptance Criterion**: The UI must validate profile type against the selected scope lane. A drawings profile must not be used for a specs lane and vice versa. This is a silent correctness failure if not enforced.

---

## 7. "Next Addendum" Stacking Flow

**Behavior**: After a successful execute, clicking "Next Addendum" sets:
- Output PDF → new base (original) PDF
- Addenda → cleared
- Wizard returns to Step 2

This enables sequential addendum application: Add1 → Add2 → Add3 without re-selecting the (cumulative) base each time.

**Sharp Edge**: The correction overlay from the previous run is **discarded** when the wizard returns to Step 2. The user must re-apply any ID overrides or row ignores for the next addendum cycle. There is no persistence of corrections across stacking runs.

**Acceptance Criterion**: The stacking flow must clearly indicate to the user that corrections are not carried forward. If corrections are carried forward in a future implementation, the behavior must be explicit and auditable.

---

## 8. Inventory Corrections — Row Stability Invariant

**Rule**: The `row.id` field is stable and never changes when corrections are applied. The `row.normalizedId` field is what changes when a user overrides an ID.

This distinction is critical:
- The correction overlay is keyed by `row.id` (stable, format: `${source}:${pageIndex}:${idPart}`)
- `row.normalizedId` is the mutable display/merge value
- When `applyCorrections()` re-runs analyze, the stable IDs survive the re-analysis and corrections are re-applied

**Acceptance Criterion**: In the Rust successor, the page identifier used for corrections must be *stable across re-analysis*. If re-analysis produces new row IDs for the same physical pages, user corrections become orphaned. The ID scheme must be deterministic from source file path + page index.

---

## 9. Known Sharp Edges (From DEVELOPMENT.md)

| Edge | Description | Workaround |
|------|------------|-----------|
| `pdf.js` workerSrc warning | Harmless warning about relative worker path in packaged apps | Logged at startup; suppress or accept |
| Viewport reset / ROI rebind | When viewport resets, ROI overlay becomes detached | `updateROIOverlays()` is no-op; re-attach on next canvas attach event |
| Profile storage migration | Old profile structure auto-migrated on read | No user action needed; transparent |
| Cache-related console messages | Certain pdf.js cache messages appear as console noise | Suppressed in `main.ts` |
| Narrative path stored but not parsed in all paths | DEVELOPMENT.md notes: "narrative PDF path stored in wizard state but not parsed/used" | Core engine integrates narrative; GUI path wiring is partial |

---

## 10. Translation to Rust/Tauri Acceptance Criteria (Summary)

| # | Criterion | Priority |
|---|-----------|---------|
| AC-1 | All Tauri commands return `Result<T, TypedError>`; no panics, no untyped strings | MUST |
| AC-2 | Analysis is never used from a prior session without explicit revalidation | MUST |
| AC-3 | ROI overlay re-binds when canvas is replaced; no silent stale state | MUST |
| AC-4 | Profile type validated against selected scope lane before ROI keys are written | MUST |
| AC-5 | Correction row IDs are deterministic from (source path, page index) | MUST |
| AC-6 | "Next Addendum" stacking clearly discards corrections; or explicitly carries them with audit trail | MUST |
| AC-7 | Wizard state does not persist between sessions (or if it does, analysis is explicitly revalidated) | MUST |
| AC-8 | `--auto-layout` / `--save-layout` equivalents must not ship unless implemented | MUST |
| AC-9 | Multi-lane scope (drawings + specs) enforces per-lane profile type matching | SHOULD |
| AC-10 | Single addendum per wizard run (simplify mental model); batch via CLI | SHOULD |
