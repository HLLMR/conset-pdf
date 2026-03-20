# Phase 03 - ADR-006 Profile-Driven Detection vs. Auto-Detection

## Status

Accepted. Profile-driven ROI detection is the canonical production path for drawings in the prototype. Auto-profile detection exists as design direction and partial scaffolding, not a completed execution path.

## Scope

Document why explicit layout profiles were chosen over full automatic layout inference, what schema/versioning contracts are active, what matching mechanisms exist, and where current code diverges from docs or CLI surface claims.

## Source Evidence

- `packages/core/src/layout/types.ts`
- `packages/core/src/layout/load.ts`
- `layouts/layout-template.json`
- `packages/core/src/locators/roiSheetLocator.ts`
- `packages/core/src/locators/compositeLocator.ts`
- `packages/core/src/locators/legacyTitleblockLocator.ts`
- `packages/core/src/utils/pdf.ts`
- `packages/core/src/workflows/merge/mergeWorkflow.ts`
- `packages/cli/src/commands/mergeAddenda.ts`
- `packages/core/src/transcript/profiles/types.ts`
- `packages/core/src/transcript/profiles/registry.ts`
- `docs/CLI.md`
- `docs/automatedRoiRefactorPlan.md`

## Context

Sheet ID/title detection quality depends heavily on project-specific title block geometry. Across AEC sets, field positions, annotation density, rotation habits, and drawing template variations are large enough that a generic one-shot layout detector is fragile.

The prototype therefore prioritized deterministic, user-controlled ROI profiles, with legacy auto-detected title-block heuristics as fallback, and a separate planned track for full automatic ROI profile generation.

## Decision

Use explicit profile-driven ROI detection as the default and required high-confidence path for drawings workflows.

Keep automatic detection only as:

- legacy heuristic fallback for bounded recovery behavior
- future roadmap/refactor target for profile suggestion/generation

Do not treat auto-layout profile selection/generation as fully implemented in current merge execution paths.

## Why Explicit Profiles Won

### 1. Deterministic geometry contract

`LayoutProfile` provides explicit ROIs in normalized coordinates and fixed extraction intent (`sheetId.rois`, optional `sheetTitle.rois`, optional anchor keywords and validation rules). This makes detection behavior auditable and reproducible per project template.

### 2. Ordered fallback inside a known profile

Profiles can define multiple ROIs per field, tried in deterministic order. This gives controlled resilience without unconstrained page-wide searching.

### 3. Strong input validation at load time

`loadLayoutProfile()` enforces required fields and ROI bounds, with warnings for suspiciously tiny/large ROIs and strict shape checks before execution.

### 4. Better operational transparency

`RoiSheetLocator` emits ROI-specific failure reasons (`ROI_EMPTY`, low density, no pattern match, prefix rejection) and confidence context. Users can fix profile coordinates directly, rather than reverse-engineering opaque auto-inference failures.

### 5. Controlled fallback boundary

`CompositeLocator` makes ROI-first behavior explicit and constrains legacy fallback behind a feature flag (`ENABLE_LEGACY_LOCATOR`). This preserves recovery capability while signaling architectural direction away from legacy heuristics.

## Active Schema and Versioning Contracts

### Layout profile JSON contract (production path)

From `layout/types.ts` and `layouts/layout-template.json`, active profile contract includes:

- top-level identity/version fields: `name`, `version`
- optional page model: orientation and ROI space (`visual` or `pdf`)
- `sheetId.rois[]` required for drawing detection
- optional `sheetTitle.rois[]`
- optional validation constraints (`allowedPrefixes`, etc.)
- optional metadata/source tagging

This is the live wire format consumed by CLI/core drawing workflows.

### Extended transcript profile system (adjacent, not merge-driving)

`transcript/profiles/types.ts` and `transcript/profiles/registry.ts` define broader typed profiles (`SpecProfile`, `SheetTemplateProfile`, `EquipmentSubmittalProfile`) plus metadata versioning and confidence-based matching (`findMatchingProfile` threshold 0.70).

However, these registry matching APIs are not currently wired into merge workflow locator selection.

## Automatic Profile Selection and Matching: Current Reality

### Implemented

- legacy title-block auto-detection heuristics (`autoDetectTitleBlock`) used by `LegacyTitleblockLocator`
- confidence-based matching routine in transcript profile registry (`findMatchingProfile`) for transcript-side profile objects

### Not implemented in active merge flow

- automatic selection of drawing layout profile from a profile registry during merge analyze/execute
- end-to-end use of CLI `--auto-layout` / `--save-layout` options to actually infer and persist layout profiles

`mergeAddenda.ts` defines those options, but execution currently only builds profile from explicit `--layout` or inline ROI flags. No branch consumes `autoLayout` or `saveLayout` in command action.

`mergeWorkflow.ts` creates locator as:

- specs: `SpecsSectionLocator`
- drawings with profile: `RoiSheetLocator` wrapped by `CompositeLocator`
- drawings without profile: `LegacyTitleblockLocator`

No auto-profile lookup/generation step runs in this path.

## Match Criteria in Existing Systems

### ROI locator candidate matching (drawing IDs)

Within configured ROIs, `RoiSheetLocator` uses pattern matching plus scoring factors (structure, length, suffix patterns, geometry compactness, position, token count, anchor proximity) to select best candidate and emit confidence.

This is ID matching inside a chosen profile, not profile selection across templates.

### Transcript profile registry matching

`ProfileRegistry.findMatchingProfile()` validates all stored profiles against transcript and returns highest-confidence profile only if score >= 0.70.

This is a profile-selection mechanism, but currently isolated from merge workflow execution.

## Alternatives Rejected

### Full auto-detection as default now

Rejected because implementation is incomplete for production merge path and historically too brittle across layout diversity.

### Legacy auto-titleblock only

Rejected as primary strategy because it is less deterministic, lower confidence on heterogeneous sets, and explicitly deprecated in architecture direction.

### No fallback strategy

Rejected because ROI misconfiguration and cover/title-sheet variability require bounded recovery paths during transition.

## Rust Preservation Requirements

Rust implementation should preserve:

- explicit profile-driven ROI as first-class path
- stable profile schema/version contract as user-facing wire format
- deterministic ROI fallback ordering and clear confidence/failure diagnostics
- strict validation gates before execution

Rust should improve by:

- implementing real profile registry selection for drawing workflows when enabled
- making auto-profile suggestion explicit, testable, and quality-gated
- removing stale CLI/documentation claims where features are not wired

## Source-of-Truth Notes

For this ADR, executable command/workflow code was treated as canonical over CLI/docs wording.

Critical clarifications:

- profile-driven ROI detection is the only complete, explicit drawings path
- legacy auto-titleblock detection exists, but is fallback/deprecated
- auto-layout profile suggestion options are exposed but currently not executed in merge command flow
- transcript profile registry matching exists but is not currently integrated into merge locator selection