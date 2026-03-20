# Architecture

**Last verified**: 2026-03-01

## Main Process

### Window Lifecycle (`src/main.ts`)

```
app.whenReady()
  → createWindow()
    → loadWindowState()
    → new BrowserWindow()
    → setMainWindow() (for dialogs)
    → registerAllHandlers()
    → loadFile('app.html')
```

**Window State**: Persisted to `userData/window-state.json` (size, position, maximized state)

**Security**: `nodeIntegration: false`, `contextIsolation: true`, `preload: preload.js`

**Responsibility Boundaries**: `main.ts` owns lifecycle, window creation, and handler registration only. All business logic must live in `src/main/utils/*` or equivalent modules. This is a hard invariant post-Stage 2 refactoring.

### IPC Registration (`src/main/ipc/index.ts`)

All handlers registered via `registerAllHandlers()`:

```
registerAllHandlers()
  ├── registerDialogHandlers()      # dialogs.ts
  ├── registerPdfHandlers()          # pdf.ts
  ├── registerProfileHandlers()     # profiles.ts
  ├── registerDetectionHandlers()   # detection.ts
  ├── registerOperationsHandlers()  # operations.ts
  ├── registerHistoryHandlers()     # history.ts
  ├── registerSystemHandlers()      # system.ts
  ├── registerMergeHandlers()       # merge.ts
  └── registerDebugHandlers()        # debug.ts
```

**Response Format**: All handlers return `IpcResponse<T>` envelope:
```typescript
{ success: boolean, data?: T, error?: string }
```

### Stores

**Profile Store** (`src/main/profiles/store.ts`):
- Profile CRUD operations
- Active profile management
- Snapshot PDF management
- Migration from old structure
- GUI-specific (not in core)

**History Store** (`src/main/history/store.ts`):
- Run history persistence
- Index management
- GUI-specific (not in core)

## Renderer Process

### App Navigation (`src/app.js`)

**State**:
```javascript
appState = {
  view: 'dashboard' | 'wizard' | 'profiles' | 'history' | 'settings',
  workflow: 'merge' | 'split' | 'bookmark' | null,
  mode: 'drawings' | 'specs' | null,
  returnTo: string | null,
  wizardStep: number | null  // Preserved when navigating away/back
}
```

**Navigation Flow**:
```
navigate(view, options)
  → updateUI()          # Update nav highlights
  → loadView()          # Load view HTML/JS
    → initializeView()  # View-specific init
```

**Views**:
- `dashboard` - Main menu
- `wizard` - Wizard container (merge/split/bookmark)
- `profiles` - Profile management
- `history` - Run history and results
- `settings` - Application settings

### Views/Wizards Layout

**Wizard Shell** (`src/wizard-shell.js`):
- Container for wizard steps
- Step navigation
- Progress tracking

**Merge Wizard** (`src/merge-wizard.js`):
- **Step 1: Configure** - Active profile selection, scope (drawings/specs/both), optional narrative PDF (stored only, not parsed)
- **Step 2: Select Files** - Lane-aware file selection (base PDF, addenda PDFs, output path) per scope
- **Step 3: Inventory & Corrections** - Inventory review with filters (search, confidence, status), minimal corrections UI (ignore rows, override IDs), recompute via `merge:analyze`
- **Step 3b: Inventory & Corrections (Specs)** - Conditional step for specs lane when scope='both'
- **Step 4: Execute** - Sequential lane execution, results display, "Next Addendum" stacking flow

**Split Wizard** (`src/split-drawings-wizard.js`):
- Mode selection
- File selection
- ROI marking
- Filename format
- Split execution

**Profiles View** (`src/profiles-view.js`):
- Profile list
- Profile editor
- Reference PDF viewer
- ROI editor

## Shared Libraries

### PdfViewportEngine (`src/modules/pdf/pdfViewportEngine.js`)

**Purpose**: Deterministic PDF rendering with race safety

**Key Methods**:
- `loadDocument(path)` - Load PDF
- `setPage(n)` - Navigate to page
- `fitToViewport()` - Auto-fit scale
- `setZoomPercent(n)` - Manual zoom
- `getCanvas()` - Get canvas element
- `getPageWrapper()` - Get page wrapper element
- `getCurrentViewport()` - Get PDF.js viewport

**Render Flow**:
```
setPage() / fitToViewport() / setZoomPercent()
  → _scheduleRender()
    → _renderPage()
      → PDF.js render task
      → onRenderComplete callback
```

**Race Safety**: Uses `renderSeq` token to ignore stale renders

### RoiOverlay + Controller

**RoiOverlay** (`src/roi-overlay.js`):
- DOM manipulation for ROI overlays
- Coordinate conversion (screen ↔ normalized)
- Drawing mode handling
- Dynamic ROI type key system

**RoiOverlayController** (`src/modules/roi/roiOverlayController.js`):
- Wraps RoiOverlay
- Manages viewport binding
- Text items overlay
- ROI change callbacks
- ROI type key management

**ROI Type Keys**:
Profiles support two ROI types: Drawings and Specifications. Each type uses different key names for ROI storage:

| Profile Type | ID Key | Title Key |
|---|---|---|
| Drawings | `sheetId` | `sheetTitle` |
| Specifications | `sectionId` | `sectionTitle` |

The ROI overlay uses dynamic keys to determine:
- Which ROI to save when drawing is complete
- Which ROI overlay to display when loading profile
- Which color to use (blue for ID ROI, green for Title ROI)
- Button labels in the UI ("Sheet" or "Section")

**Binding Flow**:
```
setViewport(viewport)
  → Create/update RoiOverlay
attachToCanvas(canvas, container)
  → roiOverlay.attachToCanvas()
  → roiOverlay.setPageWrapper()
setRoiTypeKeys(profileType)
  → Sets dynamic roiKeys based on 'drawings' | 'specs' type
  → Updates drawing mode buttons and overlay display
updateROIOverlays()
  → Check viewport has valid nodes
  → Uses roiKeys to determine which ROI overlays to create
```

**Profile Type Workflow**:
```
Load PDF with Profile
  → roiOverlay.setRoiTypeKeys(profile.type)     # 'drawings' | 'specs'
  → roiController.attachToCanvas()
  → roiController.setRois(profile.rois)         # Keys match profile type
  → roiOverlay.updateROIOverlays()
```

**Drawing Flow**:
```
User clicks "Draw Sheet ID / Section ID" button
  → roiController.enableDraw(roiKeys.idKey)     # 'sheetId' | 'sectionId'
  → RoiOverlay enters draw mode
    → drawingMode = roiKeys.idKey
    → Drawing color is blue
  → User draws ROI
    → Saves to rois[roiKeys.idKey]              # Blue overlay created
    → Button label shows "✓"
```

**Safety**: `updateROIOverlays()` is no-op if `pageWrapper` or `canvas` missing

## IPC Flow

### Request Path
```
Renderer: window.api.profiles.readProfile(id)
  → Preload: ipcRenderer.invoke('profiles:read', id)
    → Main: ipcMain.handle('profiles:read', ...)
      → Handler: createSuccessResponse(data) | createErrorResponse(error)
        → Preload: unwrapResponse(response)
          → Renderer: data | throw Error
```

### Response Envelope
All handlers return:
```typescript
interface IpcResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}
```

Preload automatically unwraps:
- `success: true` → return `data`
- `success: false` → throw `Error(error)`

## Data Flow Examples

### Profile Load
```
User clicks profile
  → profiles-view.js: loadProfile(id)
    → window.api.profiles.readProfile(id)
      → IPC: profiles:read
        → profiles.ts handler
          → store.readProfile(id)
            → Return profile
              → Renderer: Display profile
```

### Merge Operation
```
User clicks Merge (Step 4 Execute)
  → wizard: runMerge(options)
    → window.api.runMerge(options)
      → IPC: runMerge
        → operations.ts handler
          → merge-internal.ts: runMergeInternal()
            → adaptRunMergeArgs() (normalize input)
            → createMergeWorkflowRunner() (core)
            → runner.execute()
            → executeResultToMergeReport() (adapter)
            → Return MergeReport
              → Renderer: Display results + "Next Addendum" button
```

### Merge Analyze (Step 3 Inventory)
```
User navigates to Step 3 or clicks "Recompute"
  → wizard: mergeAnalyze(options)
    → window.api.mergeAnalyze(options)
      → IPC: merge:analyze
        → merge.ts handler
          → createMergeWorkflowRunner() (core)
          → runner.analyze() (or runner.applyCorrections() if corrections provided)
          → Return InventoryResult
            → Renderer: Display inventory table with filters
```

## File Structure

```
src/
├── main.ts                    # Entry point
├── preload.ts                 # IPC bridge
├── app.js                     # Navigation
├── app.html                   # Main app shell
├── profiles-view.js           # Profiles management view
├── wizard-shell.js            # Wizard container
├── merge-wizard.js            # Merge wizard (4-step flow)
├── split-drawings-wizard.js  # Split wizard
├── placeholder-wizard.js      # Placeholder for unimplemented workflows
├── roi-overlay.js             # ROI overlay DOM manipulation
├── wizard-utils.js            # Wizard utilities
├── main/
│   ├── ipc/                   # IPC handlers
│   │   ├── index.ts           # Handler registration
│   │   ├── dialogs.ts         # File dialogs
│   │   ├── pdf.ts             # PDF operations
│   │   ├── profiles.ts        # Profile management
│   │   ├── detection.ts       # Detection operations
│   │   ├── operations.ts      # Merge/split operations (runMerge)
│   │   ├── merge.ts           # Advanced merge (merge:run, merge:analyze)
│   │   ├── merge-internal.ts  # Consolidated merge logic
│   │   ├── system.ts           # File system
│   │   ├── history.ts         # Run history
│   │   └── debug.ts           # Debug tools
│   ├── profiles/
│   │   └── store.ts           # Profile storage (GUI-specific)
│   ├── history/
│   │   └── store.ts           # History storage (GUI-specific)
│   └── utils/                 # Business logic utilities
│       ├── layout-profile.ts
│       ├── detection-orchestration.ts
│       └── filename-generation.ts
├── modules/
│   ├── pdf/
│   │   └── pdfViewportEngine.js  # PDF rendering engine
│   ├── profiles/
│   │   └── profilesStore.js      # Profile state management
│   └── roi/
│       └── roiOverlayController.js  # ROI overlay controller
└── shared/
    ├── ipc-response.ts        # Response envelope
    ├── logger.js              # Logging
    └── validate.js            # Validation
```
