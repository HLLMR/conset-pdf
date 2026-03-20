# Phase 01 - CLI Command Inventory

## Scope

Phase 1, Step 2 from docs/postMortemDocExtraction.md: enumerate CLI commands in packages/cli/src/commands and record implementation status.

## Method

- Verified command modules present on disk in packages/cli/src/commands.
- Verified command registration in packages/cli/src/cli.ts.
- Read each command file to classify runtime readiness and deprecation state.
- Applied status labels: Complete, Partial, Planned-but-not-started, Abandoned.

## Command Registration

All commands below are wired into the Commander entrypoint in packages/cli/src/cli.ts.

## Inventory Matrix

| Command Module | CLI Command Name | Primary Core Path Used | Status | Notes |
|---|---|---|---|---|
| assembleSet.ts | assemble-set | core/assembleSet | Abandoned | Still wired and callable, but roadmap marks assemble workflow superseded/abandoned. |
| debugWalkthrough.ts | debug-walkthrough | transcript + specs extraction pipeline (debug script path) | Partial | Implemented as deep diagnostic workflow against hardcoded/reference PDF discovery; not a normal production workflow command. |
| detect.ts | detect | DocumentContext + RoiSheetLocator or LegacyTitleblockLocator | Complete | Fully implemented preview/detection command with JSON output and page sampling options. |
| fixBookmarks.ts | fix-bookmarks | workflows/bookmarks | Complete | Fully implemented analyze/execute flow, profile controls, section-start strategy, validation flags. |
| mergeAddenda.ts | merge-addenda | workflows/merge | Complete | Fully implemented analyze dry-run and execute modes with narrative support and report/output options. |
| specsInventory.ts | specs-inventory | specs inventory/footer-first sectionization | Complete | Implemented deterministic inventory generator using region detection and sectionization passes. |
| specsPatch.ts | specs-patch | workflows/specs-patch | Abandoned | Still implemented and wired, but roadmap marks specs-patch superseded/abandoned. |
| splitSet.ts | split-set | core/splitSet | Complete | Implemented split command path with grouping and TOC output options; active command wiring remains. |

## Status Summary (Step 2)

- Complete: 5
- Partial: 1
- Planned-but-not-started: 0
- Abandoned: 2

## Drift and Notes

- Command surface still includes abandoned commands (assemble-set, specs-patch) for compatibility/documentation continuity.
- split-set currently routes through core split function directly (not through workflow runner pattern used by newer commands).
- debug-walkthrough is a specialized debugging pipeline command and should be treated as engineering diagnostics, not primary end-user workflow.

## Evidence Reviewed

- packages/cli/src/cli.ts
- packages/cli/src/commands/assembleSet.ts
- packages/cli/src/commands/debugWalkthrough.ts
- packages/cli/src/commands/detect.ts
- packages/cli/src/commands/fixBookmarks.ts
- packages/cli/src/commands/mergeAddenda.ts
- packages/cli/src/commands/specsInventory.ts
- packages/cli/src/commands/specsPatch.ts
- packages/cli/src/commands/splitSet.ts
- ROADMAP.md
- README.md
