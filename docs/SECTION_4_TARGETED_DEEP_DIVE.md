# Section 4 — Targeted Deep Dive Pack

## 4.1 IPC plumbing (paths + pattern)

### Core Pattern
Renderer calls `window.api.methodName(args)` → Preload wraps with `ipcRenderer.invoke()` → Main process handles with `ipcMain.handle()` → Response envelope unwrapped by `unwrapResponse()`.

### IPC Response Envelope
All handlers return `IpcResponse<T>` with fields: `{ success: boolean; data?: T; error?: string }`.

### Helper Functions & Locations
- **unwrapResponse<T>()** — [preload.ts#L19-L25](preload.ts#L19-L25). Throws if `success=false`, else returns data.
- **createSuccessResponse()** — [shared/ipc-response.js](shared/ipc-response.js). Wraps successful result.
- **createErrorResponse()** — [shared/ipc-response.js](shared/ipc-response.js). Wraps error message from thrown exception.
- **IPC channel naming**: Kebab-case, e.g., `runMerge`, `pdf:getPageCount`, `dialog:selectFile`.

### Error End-to-End
1. Handler throws `Error` → caught by try-catch
2. `createErrorResponse(error)` serializes `error.message` into `{ success: false, error: "message" }`
3. Preload's `unwrapResponse()` sees `success=false` → re-throws `Error(response.error)`
4. Renderer catches thrown error normally

---

## 4.2 operations.ts decomposition proposal

File: [conset-pdf-gui/src/main/ipc/operations.ts](conset-pdf-gui/src/main/ipc/operations.ts) (1096 lines) contains ~10 major responsibilities:

| Bucket | Current Location | Proposed Module | Exports |
|--------|------------------|-----------------|---------|
| **Merge handler** (parse, adapt args, route) | operations.ts:80–110 | `ipc/handlers/merge.ts` | `registerMergeHandler()` |
| **Split handler** (detection, filename gen, PDF splitting) | operations.ts:427–835 | `ipc/handlers/split.ts` | `registerSplitHandler()` |
| **Preview naming** (detection + template rendering) | operations.ts:110–425 | `ipc/handlers/naming.ts` | `registerPreviewNamingHandler()` |
| **Standards lookups** (disciplines, divisions) | operations.ts:843–865 | `ipc/handlers/standards.ts` | `registerStandardsHandlers()` |
| **Session cache** (save/load/clear, MD5 hashing) | operations.ts:865–970 | `ipc/handlers/cache.ts` | `registerCacheHandlers()` |
| **Bookmarks generation** (workflow runner execution) | operations.ts:968–1096 | `ipc/handlers/bookmarks.ts` | `registerBookmarksHandler()` |
| **Discipline resolution** (alias lookup, fallback logic) | operations.ts:48–73 | `utils/discipline-resolver.ts` (or @conset-pdf/core) | `resolveDrawingsDiscipline()` |
| **Layout profile creation** | utils/layout-profile.ts | Keep as-is | `createLayoutProfileFromRois()` |
| **Standards registry loading** | operations.ts (lines 33–39) | utils/standards.ts (or stay in each handler) | `loadStandardsIntoRegistry()` |
| **Detection orchestration** | utils/detection-orchestration.ts | Keep as-is | `runDetection()` |

### Move to @conset-pdf/core:
- `resolveDrawingsDiscipline()` — Discipline alias resolution is domain logic, not GUI-specific.
- Standards registry management — Already in core; GUI can just load it.

---

## 4.3 Long-running work & blocking risk

| Operation | Typical Duration | Execution Model | Progress Events |
|-----------|------------------|-----------------|-----------------|
| **runMerge** | 1–5s (PDF size dependent) | Main thread, synchronous | None |
| **previewNaming** (detection) | 2–10s (large PDF, many pages) | Main thread, synchronous | None |
| **runSplit** (file writing) | 0.5–3s (page count, output folder I/O) | Main thread, synchronous | None |
| **sampleDetect** | 1–5s (sampled pages) | Main thread, synchronous | None |
| **generateBookmarks** | 0.5–2s (bookmark tree complexity) | Main thread, synchronous | None |

### Blocking Risk Assessment
All major operations block the main thread. No worker threads or child processes used (except Python sidecar in core for transcript extraction, which is not currently invoked from GUI).

### Progress Reporting
**None.** Handlers return only final result; no partial updates or cancellation tokens.

---

## 4.4 Python integration details (if present)

### Location & Invocation
PyMuPDFExtractor in [conset-pdf/packages/core/src/transcript/extractors/pymupdfExtractor.ts](conset-pdf/packages/core/src/transcript/extractors/pymupdfExtractor.ts). Uses `child_process.execFile()` via promisified `execFileAsync()`.

### Argument Passing
Command-line arguments: `--input <pdfPath> --output <outputPath> [--pages pageList] [--include-lines]`. No stdin or temp JSON files.

### Stdout/Stderr Parsing
- Checks `stderr` for `'PyMuPDF'` substring to detect missing library → throws "PyMuPDF not installed. Run: pip install pymupdf>=1.24.0"
- General error message: `Extraction failed: ${error.message}`

### Timeout & Kill Behavior
`execFile()` has default Node.js child process limits; no explicit timeout set in code. Process will be killed if parent exits.

### Fallback & Selection
- Primary: PyMuPDF (via sidecar script `extract-transcript.py`)
- Fallback: PDF.js-based extractor selected automatically if PyMuPDF unavailable
- Selection determined by `createTranscriptExtractor()` factory in core

### Tempdir & Cleanup
- Creates: `mkdtemp(path.join(os.tmpdir(), 'conset-transcript-'))`
- Cleanup: `fs.rm(tempDir, { recursive: true, force: true })` in finally block

---

## 4.5 File IO safety

### Output PDF Writing Strategy
PDFs written directly via `fs.writeFile()` (no atomic temp+rename pattern observed in GUI). Example: [operations.ts#L496](operations.ts#L496-L497).

### Collision Avoidance (Split Operation)
Function `ensureUniqueFilename()` checks if file exists via `fs.access()`. If exists, appends `_1`, `_2`, etc. until unique name found (polling, not atomic).

### Temp Directories Used
- PyMuPDF sidecar: `os.tmpdir()/conset-transcript-*`
- Bookmarks generation (core): `os.tmpdir()/conset-pdf-bookmarks-*`
- All cleaned up with `fs.rm(tempDir, { recursive: true, force: true })`

### Cleanup Strategy
Try-finally blocks in core extractors; all temp dirs immediately removed after use.

### Windows EACCES Handling
**None observed.** No retry loops or backoff strategies for EACCES errors. Files assumed to be unlocked.

### Profile Snapshots (Special Case)
Snapshots initially created with temp ID (e.g., `temp_xyz.pdf`), then renamed to real profile ID via `fs.rename()` on profile save. If rename fails, reference is cleared and warning logged ([profiles.ts#L227](profiles.ts#L227-L236)).

---

**Document Status**: Complete. All sections derived from source inspection. Where behavior is unclear or unimplemented, marked "None" or "Unknown".
