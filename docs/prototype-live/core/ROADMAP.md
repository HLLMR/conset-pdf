# Roadmap

**Last updated**: 2026-03-19

This document provides a concise view of what's complete and what remains to be implemented in the conset-pdf project.

## Overview

**Project Philosophy**: Boring, explicit, construction-grade features with incremental commits and CLI parity. Core-first development. No architectural "improvements" or refactors for cleanliness.

**Workflow Pattern**: All workflows follow the **analyze → applyCorrections → execute** pattern for consistency.

---

## ✅ Fully Complete

### V3 PDF Extraction Architecture

**Status**: ✅ **100% Complete**

Complete transcript-based extraction system with:
- Backend-agnostic transcript extraction (PyMuPDF primary, PDF.js fallback)
- Deterministic canonicalization (rotation, coordinates, stable-sort, hashes)
- Quality scoring and validation gates
- Privacy-preserving pattern abstraction (TokenVault, sanitization)
- ML Ruleset Compiler for profile generation
- Enhanced parsers (specs with chrome removal, schedules, submittals)
- Comprehensive test coverage

**Files**: `packages/core/src/transcript/` (complete module)

**Documentation**: `docs/TRANSCRIPT_ARCHITECTURE.md`, `docs/MIGRATION_V3.md`, `docs/ML_RULESET_COMPILER.md`

### Update Documents (Merge) Workflow

**Status**: ✅ **100% Complete** (Engine + CLI + GUI)

- Complete workflow engine implementation
- CLI command: `merge-addenda` with full options
- GUI wizard: 4-step Update Documents workflow
- Drawings support: ROI-based detection with layout profiles + legacy fallback
- Specs support: Text-based section ID detection
- Inventory analysis with corrections support
- All merge modes: `replace+insert`, `replace-only`, `append-only`
- Narrative validation integration

**Files**: `packages/core/src/workflows/merge/`, `packages/cli/src/commands/mergeAddenda.ts`, `conset-pdf-gui/src/merge-wizard.js`

### Fix Bookmarks Workflow

**Status**: ✅ **100% Complete** (Engine + CLI)

- Read, validate, and repair PDF bookmarks
- Rebuild bookmarks from `BookmarkAnchorTree` (Specs Pipeline) or sheet/section inventory
- Footer-First Section Anchoring: Deterministic section destination resolution
- Deterministic sidecar writer (QPDF/pikepdf) for cross-viewer compatibility
- Bookmark correction support (rename, reorder, delete, retarget, rebuild)
- CLI command: `fix-bookmarks`

**Files**: `packages/core/src/workflows/bookmarks/`, `packages/core/src/bookmarks/`, `packages/cli/src/commands/fixBookmarks.ts`

### Standards Module (UCS/CSI)

**Status**: ✅ **100% Complete**

- Drawings: UDS-style discipline identification and sorting
- Specs: CSI MasterFormat classification and sorting
- Integrated into merge workflow inventory mapping

**Files**: `packages/core/src/standards/`

**Documentation**: `docs/STANDARDS.md`

### Narrative PDF Processing

**Status**: ✅ **100% Complete**

- Text extraction from narrative PDFs
- Algorithmic parsing
- Deterministic validation against inventory
- Issue detection with near-match suggestions
- Suggested corrections (advisory only)
- Integration into merge workflow `analyze()`

**Files**: `packages/core/src/narrative/`

**Note**: LLM-assisted extraction is optional and not implemented. Algorithmic parsing works for most cases.

---

## ❌ Abandoned

### Specs Patch Workflow

**Status**: ❌ **Abandoned** - Extracted spec functionality integrated into Extract Documents workflow

**Superseded By**: Extract Documents (split) workflow with expanded capabilities

**Files**: `packages/core/src/workflows/specs-patch/` (deprecated), `packages/cli/src/commands/specsPatch.ts` (deprecated)

### Assemble Set Workflow  

**Status**: ❌ **Abandoned** - Replaced by modular composition in Extract Documents workflow

**Superseded By**: Extract Documents (split) workflow output composition

**Files**: `packages/cli/src/commands/assembleSet.ts` (deprecated), `packages/core/src/core/assembleSet.ts` (deprecated)

---

## ❌ Not Started

### Optional Enhancements

- **LLM-Assisted Narrative Extraction**: Optional LLM provider for better narrative parsing (algorithmic parsing works for most cases)
- **Schedule Extraction Fallbacks**: pdfplumber/camelot Python sidecar scripts (geometry-based extraction is fully implemented)
- **TokenVault Reconstruction**: Full transcript reconstruction from abstract transcript (tokenization works correctly)

---

## GUI Work (conset-pdf-gui)

### ✅ Complete

- Update Documents wizard (4-step workflow)
- Profile management with ROI selection
- IPC architecture
- Quick-start drawers
- Narrative validation UI integration

### ⚠️ Placeholder UI Exists

- Extract Documents (Split) wizard
- Build Package (Assemble) wizard
- Fix Bookmarks wizard

**Files**: `conset-pdf-gui/src/placeholder-wizard.js`, `conset-pdf-gui/src/split-drawings-wizard.js`

**Blocked By**: GUI wiring and productization work (core engines are implemented)

---

## Summary Table

| Feature | Core Engine | CLI | GUI | Status |
|---------|-------------|-----|-----|--------|
| **Update Documents (Merge)** | ✅ | ✅ | ✅ | Complete |
| **Extract Documents (Split)** | ✅ | ✅ | ⚠️ | Core+CLI complete; GUI placeholder |
| **Fix Bookmarks** | ✅ | ✅ | ⚠️ | Core+CLI complete; GUI placeholder |
| **Specs Patch** | ❌ | ❌ | ❌ | Abandoned |
| **Assemble Set** | ❌ | ❌ | ❌ | Abandoned |
| **Standards (UCS/CSI)** | ✅ | ✅ | ✅ | Complete |
| **Narrative Extraction** | ✅ | ✅ | ✅ | Complete |
| **Narrative Validation** | ✅ | ✅ | ✅ | Complete |

**Legend**:
- ✅ Complete
- ⚠️ Partial/Placeholder
- ❌ Not started

---

## Implementation Principles

### Guardrails

1. **No changing the stable workflow pattern**: All workflows follow analyze → applyCorrections → execute
2. **Narrative is additive**: Narrative validation adds to analyze results, doesn't replace detection
3. **No smart auto-corrections**: Suggestions only, never auto-applied
4. **Stable IDs**: All outputs explicitly keyed and versionable (stable codes, stable IDs)
5. **Incremental commits**: One concern per commit, green build/tests
6. **CLI parity**: All workflows must have CLI commands
7. **Core-first**: Implement in core library first, then CLI, then GUI

### Architecture Invariants

- Only `DocumentContext` loads PDFs
- Locators are pure (no IO)
- Parsers are pure functions
- Workflow engine is canonical (CLI and GUI route through it)

---

## Next Steps (Recommended Order)

1. **GUI Integration** - Complete Split and Fix Bookmarks wizard implementations
2. **Doc Accuracy Cleanup** - Keep README/ROADMAP/CLI command status aligned with code truth
3. **Deferred Feature Planning** - Decide V4 path for auto-layout and report-viewer surfaces

---

**Last Updated**: 2026-03-19
