# conset-pdf-v3-gui

Electron-based desktop application for building "latest-and-greatest" construction document sets. Provides wizard interfaces for merging addenda, splitting sets, and managing layout profiles.

**Repo Scope**: Presentation and orchestration only. All PDF processing is delegated to `@conset-pdf/core`. The GUI provides visual ROI selection, preview detection, profile management, and wizard-based workflows.

## Current Status (2026-03-01)

**✅ Fully Implemented:**
- Update Documents wizard (4-step workflow with inventory review)
  - Single addendum workflow (one addendum per wizard run)
  - Narrative validation UI with issue display and suggested corrections
  - Set Order sorting with grouping headers (standards-based)
  - Analysis caching with dirty checking
- Profile management with ROI selection and reference PDF viewer
- IPC architecture with standardized response envelope
- Quick-start drawers (QS-2 workflow, QS-3 profiles) with global header slot positioning
- Multi-lane workflow support (drawings + specs)
- Extract Documents (Split) wizard
- Fix Bookmarks wizard

**🔗 Integration:**
- Uses `@conset-pdf/core` workflow engine for merge operations
- `merge:analyze` IPC channel for inventory analysis
- `runMerge` IPC channel (canonical) routes through core workflow engine

## Dev Setup

### Prerequisites
- Node.js 18+
- `@conset-pdf/core` available locally (see dependency structure below)

### Install
```bash
# Clone repos at same level
git clone <conset-pdf-v3-repo>
git clone <conset-pdf-v3-gui-repo>

# Install GUI dependencies
cd conset-pdf-gui
npm install
```

**Dependency Structure:**
```
workspace-root/
├── conset-pdf/           # Core library (part of conset-pdf-v3)
│   └── packages/core/    # Required by GUI
└── conset-pdf-gui/       # This repo (part of conset-pdf-v3-gui)
```

GUI uses `"@conset-pdf/core": "file:../conset-pdf/packages/core"` for development.

### Build
```bash
npm run build          # Compile TypeScript + copy assets
```

### Run
```bash
npm run dev            # Build + launch Electron (development mode)
npm start              # Same as dev
```

### Package
```bash
npm run build:app      # Build + package with electron-builder
npm run build:win      # Build Windows installer (NSIS + portable)
```

### Test
```bash
npm test               # Run all tests
npm run test:gui       # GUI tests only
npm run test:integration  # Integration tests
```

## App Navigation Structure

The app uses a view-based navigation system with the following views:

- **Dashboard** (`view: 'dashboard'`): Main landing page with workflow cards
- **Wizard** (`view: 'wizard'`): Workflow-specific wizards
  - `workflow: 'merge'`, `mode: 'drawings'` → Update Documents (Drawings)
  - `workflow: 'merge'`, `mode: 'specs'` → Update Documents (Specs)
  - `workflow: 'split'`, `mode: 'drawings'` → Extract Documents (Drawings)
  - `workflow: 'split'`, `mode: 'specs'` → Extract Documents (Specs)
  - `workflow: 'bookmark'` → Fix Bookmarks
- **Profiles** (`view: 'profiles'`): Layout profile management
- **History** (`view: 'history'`): Run history and results
- **Settings** (`view: 'settings'`): Application settings

Navigation is handled via `window.appNavigate(view, options)`.

## Wizard UX Notes

### Update Documents Wizard (4-Step Flow)

1. **Step 1: Configure** - Select profile, document type (drawings/specs), optional narrative PDF
2. **Step 2: Select Files** - Choose original PDF, single addendum PDF, output path
   - Analysis runs automatically on forward navigation (Next button)
   - Analysis cached with dirty checking (only re-runs if inputs change)
3. **Step 3: Inventory & Corrections** - Review detected sheets, ignore rows, override IDs
   - Uses cached analysis from Step 2 (does not trigger new analysis)
   - Narrative validation UI displays issues and suggested corrections
   - Set Order sorting with grouping headers (by discipline for drawings, by division for specs)
4. **Step 4: Execute** - Run merge and view results

### Quick-Start Drawers

- **QS-2 (Workflow)**: Per-workflow drawer mounted in header slot for Update, Extract, and Bookmarks workflows
- **QS-3 (Profiles)**: Profiles view drawer with ROI setup tips
- **Global Positioning**: All drawers use unified header slot system (1100px width, centered, overlays content)
- **Persistence**: Dismiss state stored per view/workflow in `localStorage` (keys: `conset.qs2.{workflowId}.dismissed`)
- **Collapsible**: Drawers can collapse to a tab button, "Don't show again" checkbox persists dismissal

### Per-Workflow Persistence

- Wizard state persisted per workflow (e.g., selected profile, file paths)
- State cleared when navigating away from wizard
- Return navigation from Profiles view preserves workflow context

## Repo Structure

```
conset-pdf-gui/
├── src/
│   ├── main.ts                    # Electron main process entry
│   ├── preload.ts                 # IPC bridge (contextBridge)
│   ├── app.js                     # App navigation/router
│   ├── app.html                   # Main app shell
│   ├── profiles-view.js           # Profiles management view
│   ├── wizard-shell.js           # Wizard container
│   ├── merge-wizard.js  # Merge wizard (4-step flow: Configure, Select Files, Inventory & Corrections, Execute)
│   ├── split-drawings-wizard.js  # Split wizard
│   ├── main/
│   │   ├── ipc/                  # IPC handlers (main process)
│   │   │   ├── index.ts          # Handler registration
│   │   │   ├── dialogs.ts        # File dialogs
│   │   │   ├── pdf.ts            # PDF operations
│   │   │   ├── profiles.ts       # Profile management
│   │   │   ├── detection.ts     # Detection operations
│   │   │   ├── operations.ts     # Merge/split operations (runMerge handler)
│   │   │   ├── merge.ts         # Advanced merge (merge:run, merge:analyze handlers)
│   │   │   ├── merge-internal.ts # Consolidated merge logic (shared by runMerge and merge:run)
│   │   │   ├── system.ts         # File system
│   │   │   ├── history.ts        # Run history
│   │   │   └── debug.ts          # Debug tools
│   │   ├── profiles/
│   │   │   └── store.ts          # Profile storage (GUI-specific)
│   │   ├── history/
│   │   │   └── store.ts          # History storage
│   │   └── utils/                # Business logic utilities
│   │       ├── layout-profile.ts
│   │       ├── detection-orchestration.ts
│   │       └── filename-generation.ts
│   ├── modules/
│   │   ├── pdf/
│   │   │   └── pdfViewportEngine.js  # PDF rendering engine
│   │   ├── profiles/
│   │   │   └── profilesStore.js       # Profile state management
│   │   └── roi/
│   │       └── roiOverlayController.js  # ROI overlay controller
│   ├── shared/
│   │   ├── ipc-response.ts       # IPC response envelope
│   │   ├── logger.js             # Logging utilities
│   │   └── validate.js           # Validation utilities
│   └── roi-overlay.js            # ROI overlay DOM manipulation
├── dist/                         # Compiled output
├── docs/                         # Documentation
└── tests/                        # Test files
```

## IPC Architecture

### Location
- **Handlers**: `src/main/ipc/*.ts` (main process)
- **Preload**: `src/preload.ts` (exposes `window.api`)
- **Renderer**: Uses `window.api.*` (no direct IPC access)

### Adding a New IPC Channel

1. **Create handler** in `src/main/ipc/`:
   ```typescript
   // src/main/ipc/myfeature.ts
   import { ipcMain } from 'electron';
   import { createSuccessResponse, createErrorResponse } from '../../shared/ipc-response.js';
   
   export function registerMyFeatureHandlers() {
     ipcMain.handle('myfeature:doSomething', async (_event, options) => {
       try {
         const result = await doSomething(options);
         return createSuccessResponse(result);
       } catch (error) {
         return createErrorResponse(error);
       }
     });
   }
   ```

2. **Register** in `src/main/ipc/index.ts`:
   ```typescript
   import { registerMyFeatureHandlers } from './myfeature.js';
   
   export function registerAllHandlers() {
     // ... existing handlers
     registerMyFeatureHandlers();
   }
   ```

3. **Expose** in `src/preload.ts`:
   ```typescript
   contextBridge.exposeInMainWorld('api', {
     // ... existing API
     myFeature: {
       doSomething: async (options) => {
         const response = await ipcRenderer.invoke('myfeature:doSomething', options);
         return unwrapResponse(response);
       },
     },
   });
   ```

4. **Document** in `docs/IPC_CONTRACTS.md`:
   - Request shape
   - Response shape
   - Error conditions

## Quick Onboarding for New Developers

1. **Start Here**: Read this README for setup and navigation structure
2. **IPC Architecture**: Read [IPC_CONTRACTS.md](docs/IPC_CONTRACTS.md) for all IPC channels and payload shapes
3. **UI Workflows**: Read [UI_WORKFLOWS.md](docs/UI_WORKFLOWS.md) for wizard behavior and UX patterns
4. **Architecture**: Read [ARCHITECTURE.md](docs/ARCHITECTURE.md) for main/renderer separation and IPC flow
5. **Development**: Read [DEVELOPMENT.md](docs/DEVELOPMENT.md) for debugging and testing

**Key Concepts:**
- **IPC Pattern**: All handlers return `IpcResponse<T>` envelope, preload unwraps automatically
- **Workflow Engine**: GUI routes through `@conset-pdf/core` workflow engine for consistency
- **View-Based Navigation**: Dashboard, Wizard, Profiles, History, Settings views
- **Wizard State**: Persisted per workflow, cleared on navigation away
- **Multi-Lane**: Update Documents supports drawings + specs in single wizard flow

## Documentation

- **[Architecture](docs/ARCHITECTURE.md)** - Main/renderer architecture, IPC flow
- **[IPC Contracts](docs/IPC_CONTRACTS.md)** - All IPC channels, request/response shapes
- **[UI Workflows](docs/UI_WORKFLOWS.md)** - Wizard steps, inventory review, UX behaviors
- **[Development](docs/DEVELOPMENT.md)** - Debugging, testing, known issues
- **[Changelog](CHANGELOG.md)** - Version history

## License

Proprietary - See [LICENSE](LICENSE)
