# Changelog

All notable changes to this project are documented in this file.

## [2026-03-23] Phase 0 Closeout + Postmortem Integration

### Added
- Monorepo app/crate boundaries for backend CLI and desktop GUI scaffolds.
- New shared contract and workflow crates for stable backend/frontend integration surfaces.
- Migration closeout documentation for Phase 0 and Phase D integration outcomes.
- Repository structure and migration log docs to codify architecture and boundary rules.
- This changelog for milestone tracking and release-level highlights.

### Changed
- Documentation authority and canonical references aligned to the top-level `docs/` canonical set and `docs/DOCUMENTATION_INDEX.md`.
- README revised for current workspace layout, corpus paths, and active CLI commands.
- Coverage workflow hardened to emit machine-readable reports (`cobertura.xml`, `lcov.info`) and upload them to Codecov with artifact retention.

### Removed
- Non-permanent runtime audit artifacts from source control scope.
- Legacy/debris planning and archival files outside canonical active docs.

### Notes
- This milestone represents the requested Phase 0 closeout and postmortem update checkpoint before Phase 0.5 GUI implementation work.
