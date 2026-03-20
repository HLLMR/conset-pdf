# Roadmap

**Last updated**: 2026-03-01

This document provides a concise view of what's complete and what remains to be implemented in the conset-pdf-gui application.

## Overview

**Repo Scope**: Presentation and orchestration only. All PDF processing is delegated to `@conset-pdf/core`. The GUI provides visual ROI selection, preview detection, profile management, and wizard-based workflows.

**Development Philosophy**: Core-first development. GUI workflows depend on core workflow engine implementations. No architectural "improvements" or refactors for cleanliness.

---

## ✅ Fully Complete

### Update Documents (Merge) Wizard

**Status**: ✅ **100% Complete**

- Complete 4-step wizard workflow (Configure, Select Files, Inventory & Corrections, Execute)
- Multi-lane support (drawings + specs in single wizard)
- Single addendum workflow (one addendum per wizard run)
- Analysis caching with dirty checking
- Narrative validation UI with issue display and suggested corrections
- Set Order sorting with grouping headers (standards-based)
- Corrections UI: ignore rows, override IDs
- Integration with core workflow engine (`merge:analyze`, `runMerge` IPC channels)

**Files**: `src/merge-wizard.js`, `src/main/ipc/merge.ts`, `src/main/ipc/operations.ts`, `src/main/ipc/merge-internal.ts`

### Profile Management

**Status**: ✅ **100% Complete**

- Profile CRUD operations (list, read, save, delete)
- Active profile management
- ROI selection with visual overlay
- Reference PDF viewer with page navigation
- Snapshot PDF capture and management

**Files**: `src/profiles-view.js`, `src/main/ipc/profiles.ts`, `src/main/profiles/store.ts`

### IPC Architecture

**Status**: ✅ **100% Complete**

- Standardized response envelope (`IpcResponse<T>`)
- All handlers return consistent format
- Preload automatic response unwrapping
- Complete IPC contracts documentation

**Files**: `src/shared/ipc-response.ts`, `src/preload.ts`, `src/main/ipc/*.ts`

**Documentation**: `docs/IPC_CONTRACTS.md`

### Quick-Start Drawers

**Status**: ✅ **100% Complete**

- QS-2 (Workflow): Per-workflow drawer for Update, Extract, and Bookmarks workflows
- QS-3 (Profiles): Profiles view drawer with ROI setup tips
- Global header slot positioning system (1100px width, centered, overlays content)
- Persistence: Dismiss state stored in `localStorage` per view/workflow
- Note: Dashboard drawer (QS-1) removed; "Learn more" links hidden pending help menu

**Files**: `src/app.html`, `src/modules/app/drawerManager.js`, `src/modules/app/viewManager.js`

**Documentation**: `docs/UI_WORKFLOWS.md`

### Navigation & App Structure

**Status**: ✅ **100% Complete**

- View-based navigation system (dashboard, wizard, profiles, history, settings)
- Wizard state persistence per workflow
- Window state persistence (size, position, maximized)

**Files**: `src/app.js`, `src/main.ts`, `src/wizard-shell.js`

---

## ⚠️ Partially Complete

### Extract Documents (Split) Wizard

**Status**: ✅ **100% Complete** (Workflow Engine + CLI + GUI)

**What's Done**:
- Complete 3-step wizard workflow (Select Files, Define Format, Run Extract)
- File selection UI, filename format configuration, preview naming
- Full integration with core workflow engine (analyze/execute pattern)
- Inventory analysis step with corrections support
- Split execution via workflow runner

**Files**: `src/split-drawings-wizard.js`, `src/main/ipc/operations.ts`, core workflow runner integration

### Report Viewer

**Status**: ⚠️ **Placeholder (TODO)**

**What's Done**:
- History view displays run history
- History entries show workflow, mode, status, timestamp

**What's Missing**:
- Report modal/viewer (TODO in `src/app.js`)
- Detailed report display
- Report export functionality

**Files**: `src/app.js`

### Settings View

**Status**: ⚠️ **Placeholder**

**What's Done**:
- Settings view exists in navigation

**What's Missing**:
- All settings functionality
- Settings persistence
- Settings UI

**Files**: `src/app.html`

---

## ❌ Not Started

### Fix Bookmarks Wizard

**Status**: ✅ **100% Complete** (Workflow Engine + CLI + GUI)

**What's Done**:
- Complete bookmark workflow UI
- Full integration with core bookmarks workflow engine
- Bookmark repair and writing capabilities
- Rebuild from BookmarkAnchorTree or inventory

**Files**: `src/placeholder-wizard.js` (now active), `src/main/ipc/operations.ts`

### Zoom-to-Point Feature

**Status**: ❌ **Not Implemented (TODO)**

**What Exists**:
- Mouse coordinates preserved in `pdfViewportEngine.js`
- TODO comment: "Implement zoom-to-point using mouseX/mouseY"

**Files**: `src/modules/pdf/pdfViewportEngine.js`

### Help System

**Status**: ❌ **Placeholder**

**What Exists**:
- Help button in UI
- Placeholder message: "Help is coming soon"

**Files**: `src/app.js`

---

## Dependencies on Core

The GUI fully implements all available core workflow engines:

| GUI Workflow | Core Status | GUI Status |
|--------------|-------------|------------|
| **Update Documents Wizard** | ✅ Complete | ✅ Complete |
| **Extract Documents (Split) Wizard** | ✅ Complete | ✅ Complete |
| **Fix Bookmarks Wizard** | ✅ Complete | ✅ Complete |
| **Specs Patch** | ❌ Abandoned | ❌ Abandoned |
| **Build Package (Assemble)** | ❌ Abandoned | ❌ Abandoned |

**See**: `../conset-pdf/ROADMAP.md` for core implementation status

---

## Summary Table

| Feature | Status | Core Dependency | Priority |
|---------|--------|-----------------|----------|
| **Update Documents Wizard** | ✅ Complete | ✅ Core ready | - |
| **Extract Documents Wizard** | ✅ Complete | ✅ Core ready | - |
| **Fix Bookmarks Wizard** | ✅ Complete | ✅ Core ready | - |
| **Profile Management** | ✅ Complete | None | - |
| **IPC Architecture** | ✅ Complete | None | - |
| **Quick-Start Drawers** | ✅ Complete | None | - |
| **Navigation & App Structure** | ✅ Complete | None | - |
| **Narrative Integration** | ✅ Complete | ✅ Core ready | - |
| **Report Viewer** | ⚠️ TODO | None | P2 |
| **Settings View** | ⚠️ Placeholder | None | P2 |
| **Zoom-to-Point** | ❌ TODO | None | P2 |
| **Help System** | ❌ Placeholder | None | P2 |

**Legend**:
- ✅ Complete
- ⚠️ Partial/Placeholder
- ❌ Not started
- P1 = Priority 1 (blocked by core)
- P2 = Priority 2 (can implement now)

---

## Next Steps (Recommended Order)

1. **Bookmarks Wizard** (Priority 2, Core Ready)
   - Core workflow is implemented
   - Can implement GUI wizard following merge wizard pattern

2. **Report Viewer Modal** (Priority 2, No Core Dependency)
   - High user value
   - No blocking dependencies

3. **Settings View** (Priority 2, No Core Dependency)
   - User-requested feature
   - Improves UX

4. **Wait for Core Workflows** (Priority 1, Blocked)
   - Split wizard
   - Depend on core implementations

---

**Last Updated**: 2026-01-17
