# Changelog

All notable changes to conset-pdf-v3-gui will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-03-03

### Added
- Profiles quick-start drawer (QS-3) with ROI setup tips
- Global header slot positioning system for all drawers
- Drawer overlay behavior (drawers no longer push content down)
- Import/export functionality for ROI Profiles
- Standardized IPC response envelope (`IpcResponse<T>`)
- Business logic utilities (layout profile creation, detection orchestration, filename generation)
- IPC contracts documentation
- Profile management view with reference PDF viewer
- Comprehensive UI workflow documentation (`docs/UI_WORKFLOWS.md`)
- Quick-start drawers (QS-1 dashboard, QS-2 per-workflow)
- Multi-lane workflow support (drawings + specs in single wizard)
- Profiles: Specifications Support with separate ROI configuration

### Changed
- **IPC Refactor**: All handlers now return standardized response envelope with major improvements to error handling, debugging, and logging
- **IPC Extraction**: Handlers moved from `main.ts` to `src/main/ipc/` modules
- **Error Handling**: Consistent error handling via response envelope
- **Merge Process**: Configured pikepdf as primary PDF handler in merge process
- **Child Processing**: Enabled jobrunner for child processing operations
- **File Operations**: Deployed atomic file writes and retry helpers
- Standardized all drawers to use unified header slot system with consistent positioning
- Drawer width fixed at 1100px (matching page content width) with auto-centering
- Workflow drawers (Update, Extract, Bookmarks) use consistent positioning
- Profiles drawer automatically displays when navigating to Profiles view
- Profile Storage: Migrated to folder-based structure with snapshots
- Merge Workflow: Update Documents wizard now uses workflow engine

### Deprecated
- PDF AST feature (abandoned)
- Legacy locator system

### Removed
- Dashboard quick-start drawer (QS-1) removed
- "Learn more" links hidden in all drawers (pending help menu implementation)
- Obsolete profiles-specific drawer functions from drawerManager

### Fixed
- ROI Overlay Drawing: Fixed issue where drawn ROIs weren't persisting after drawing completion
- Profile Type Switching: Fixed coordinate mismatch when switching between profile types mid-editing
- ROI Keys Consistency: Ensured ROI overlay controller properly reinitializes with correct key names
- Return Button Validation: Fixed "Use and Return" button to check correct ROI key based on profile type
- Test Detection: Fixed validation and result table headers to use correct ROI labels

## [1.0.0] - 2026-01-14

### Added
- Initial release of conset-pdf-gui
- Wizard-style user interface for PDF merge operations
- Interactive ROI (Region of Interest) selection for sheet ID and title detection
- Sample detection preview on randomly selected pages
- Step-by-step validation and progress tracking
- Support for both Drawings and Specs document types
- Windows installer and portable build configurations
- Integration with @conset-pdf/core library

### Security
- Updated pdfjs-dist from 3.11.174 to 5.4.530 (fixes high severity vulnerability)
- Updated electron from 28.0.0 to 39.2.7 (fixes moderate severity vulnerability)

### Technical
- Electron-based desktop application
- Context isolation enabled for security
- TypeScript for type safety
- IPC-based communication between renderer and main process
- Seeded random number generator for deterministic page sampling
