# Phase 05 - Unimplemented Features

## Scope

Step 29 and Step 30 capture features that are still incomplete, planned-only, or partially implemented.

## Unimplemented or Partial GUI Workflows

### Report Viewer (GUI)

- Status: Partial placeholder.
- Evidence: history action still uses TODO + alert placeholder in `conset-pdf-gui/src/modules/app/historyUI.js`.
- Gap:
  - no modal/view component for report JSON
  - no rich report export UX from history screen

### Placeholder workflow shell

- Status: Exists and is still used for unsupported workflow states.
- Evidence: `conset-pdf-gui/src/placeholder-wizard.js`, fallback branch in `conset-pdf-gui/src/modules/app/viewManager.js`.
- Gap:
  - unsupported workflow/mode combinations route to generic "coming soon" shell

### Specs Patch GUI wizard

- Status: Not wired in GUI, while engine + CLI exist.
- Evidence:
  - specs patch workflow code exists in `conset-pdf/packages/core/src/workflows/specs-patch/specsPatchWorkflow.ts`
  - CLI command exists in `conset-pdf/packages/cli/src/commands/specsPatch.ts`
  - operations router in GUI only registers merge/split/bookmarks stacks in `conset-pdf-gui/src/main/ipc/operations.ts`
- Gap:
  - no GUI navigation entry, no wizard flow, no IPC handler path for specs patch execution

### Submittal workflow (end-to-end)

- Status: Parser-level APIs exist, but no workflow engine/CLI/GUI path.
- Evidence:
  - parser exported from core index in `conset-pdf/packages/core/src/index.ts`
  - parser marked early development / experimental in `conset-pdf/packages/core/src/submittals/extract/submittalParser.ts`
  - no submittal command under `conset-pdf/packages/cli/src/commands/`
  - no submittal workflow module under `conset-pdf/packages/core/src/workflows/`
- Gap:
  - no analyze/applyCorrections/execute runner
  - no production CLI route
  - no GUI wiring

## Planned But Not Started (Prototype)

### Automated ROI detection and profile generation

- Status: Plan document only.
- Evidence: `conset-pdf/docs/automatedRoiRefactorPlan.md` defines phases but implementation wiring is absent in merge/split execution paths.

### Equipment schedule extraction UI

- Status: Not present in GUI.
- Evidence:
  - extraction engine exists in core (`conset-pdf/packages/core/src/transcript/schedules/extractor.ts`)
  - no dedicated schedule workflow surfaces in `conset-pdf-gui/src/app.html` or operations router.

### Web/SaaS mode

- Status: Future concept only.
- Evidence: marked future in `conset-pdf/docs/MASTER_PLAN_v4.md`.

### Pattern Development Tool

- Status: Planned in V4 roadmap, not implemented in prototype repos.
- Evidence: listed as critical Phase 0.5 dependency in `conset-pdf/docs/MASTER_PLAN_v4.md`.

### Audit bundle and visual overlay export package

- Status: Target architecture documented, not delivered as a complete prototype output contract.
- Evidence: aspirational audit bundle spec in `conset-pdf/docs/MASTER_PLAN_v4.md`; no corresponding end-to-end artifact writer under current workflow outputs.

### LLM-assisted narrative integration

- Status: Deferred.
- Evidence: explicitly marked optional/not implemented in `conset-pdf/ROADMAP.md` while algorithmic path is active.

## Drift Notes Against Legacy Planning Claims

- Bookmarks GUI is implemented in current code (`conset-pdf-gui/src/bookmark-wizard.js`), so older "fully TBD" claims are stale.
- Split GUI is implemented in current code (`conset-pdf-gui/src/split-drawings-wizard.js`), so older "placeholder" claims are stale.
- Settings view is implemented (`conset-pdf-gui/src/settings-view.js`), while older roadmap text still labels it placeholder.

## Migration Implications

- Rust planning should treat Report Viewer, submittal workflow orchestration, and specs-patch GUI wiring as true delivery gaps.
- Rust planning should treat old roadmap status labels for split/bookmark/settings as documentation drift, not executable truth.
