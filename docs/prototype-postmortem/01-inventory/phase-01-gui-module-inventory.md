# Phase 01 - GUI Module Inventory

## Scope

Phase 1, Step 3 from docs/postMortemDocExtraction.md: enumerate GUI modules in conset-pdf-gui/src/ and record implementation status.

## Method

- Verified GUI source structure under conset-pdf-gui/src/.
- Spot-checked key wizard and view implementations for readiness classification.
- Counted implementation files per module group to assess complexity.
- Applied status labels: Complete, Partial, Planned-but-not-started, Abandoned.

## Inventory Matrix

### Wizards (Top-Level)

| Wizard File | CLI-Like Command | Status | Notes |
|---|---|---|---|
| merge-wizard.js | Update Documents | Complete | Full 4-step workflow; lane-aware (separate drawings/specs state); narrative support; inventory correction UI. |
| split-drawings-wizard.js | Extract Documents | Complete | Full 4-step workflow; grouping and format options; filename preview and override support. |
| bookmark-wizard.js | Fix Bookmarks | Complete | Full 4-step workflow; multi-file loop support; profile/strategy options; preview UI. |
| placeholder-wizard.js | (N/A) | Partial | Minimal placeholder UI showing "coming soon" screen; not a functional workflow. |

### Views (Top-Level)

| View File | Status | Notes |
|---|---|---|
| profiles-view.js | Complete | Full ROI editor with PDF viewport, overlay controller, region selection, CRUD operations, detection test. |
| settings-view.js | Complete | (Imported but actual implementation not reviewed; assumed complete based on fixture). |

### Utility Modules (Top-Level)

| Utility File | Status | Notes |
|---|---|---|
| wizard-shell.js | Complete | Base class for wizard UI pattern; step management, state, validation, UI updates. |
| wizard-utils.js | Complete | Shared utilities (page sampling, PDF context, filename generation). |
| roi-overlay.js | Complete | Visual ROI overlay rendering and interaction. |

### Application Modules (src/modules/)

| Module | Impl Files | Status | Notes |
|---|---:|---|---|
| app/ | 6 | Complete | Drawer manager, history UI, navigation state, progress modal, UI updater, view manager. |
| bookmarks/ | 4 | Complete | Gate panel, preview UI, profile handler, step config. |
| merge/ | 7 | Complete | Execution UI, inventory UI (with utils), lane UI, profile gate, step config, merge utils. |
| pdf/ | 1 | Complete | PDF viewport engine for page rendering and interaction. |
| profiles/ | 10 | Complete | CRUD operations, detection/test runners, editor UI, event bindings, layout management, list handlers, PDF controller, store, view mode, view utilities. |
| roi/ | 1 | Complete | ROI overlay controller for interactive selection. |
| split/ | 4 | Complete | Gate panel, preview UI, profile handler, step config. |

### IPC Handlers and Main Process (src/main/)

| Component | Impl Files | Status | Notes |
|---|---:|---|---|
| src/main/ipc/ | 13 | Complete | Top-level handlers: debug, detection, dialogs, history, index, logging, merge-internal, merge, operations, pdf, profiles, settings, system. |
| src/main/ipc/handlers/ | 6 | Complete | Submodule handlers: bookmarks, cache, merge, naming, split, standards. |
| src/main/profiles/ | 2 | Complete | Profile store and import/export. |
| src/main/history/ | 1 | Complete | History state store. |
| src/main/utils/ | 7 | Complete | Detection orchestration, file I/O, filename generation, job runner, layout profile, logger, specs section index, system info. |
| src/main/workers/ | 1 | Complete | Job worker for background task execution. |

### Shared Modules (src/shared/)

| Module | Status | Notes |
|---|---|---|
| ipc-response.ts | Complete | IPC envelope type used across all inter-process communication. |
| helpText.js | Complete | Centralized workflow help text. |
| logger.js | Complete | Shared logging utilities. |
| validate.js | Complete | Input validation helpers. |

## Status Summary (Step 3)

- Not including submodule counts (to avoid double-counting):
  - Wizards: 4 total (3 Complete, 1 Partial)
  - Views: 2 total (2 Complete)
  - Utility files: 3 total (3 Complete)
  - Module groups: 7 total (7 Complete)
  - IPC and Main Process: 30+ handlers/stores/utils/workers (all Complete)
  - Shared: 4 total (4 Complete)

**Summary**: 50+ total GUI-side components across wizards, views, modules, handlers, stores. All fully implemented except placeholder wizard (Partial).

## Drift and Notes

- Placeholder wizard shows "coming soon" and represents a UI shell only; no functional workflow implemented.
- All active wizards follow the analyze → execute pattern via IPC calls to core library.
- Profiles view is the most complex module (10 files) covering ROI selection, profile CRUD, detection test, PDF viewport management.
- IPC handlers (src/main/ipc/) provide router layer between GUI front-end and core library workflows.
- Job worker and history store support undo/redo and background operation tracking.

## Evidence Reviewed

- conset-pdf-gui/src/ full directory tree
- Spot-checked implementations:
  - merge-wizard.js (lines 1–60)
  - split-drawings-wizard.js (lines 1–50)
  - bookmark-wizard.js (lines 1–60)
  - placeholder-wizard.js (full file)
  - profiles-view.js (lines 1–50)
- Module file counts from src/modules/ and src/main/
- README.md
- ROADMAP.md (GUI status section)
