# Development Guide

**Last verified**: 2026-03-01

## How to Work Without Breaking It

### Before You Commit

**Safe Change Checklist**:
- [ ] No IPC channel renames (channels are contracts)
- [ ] No renderer API surface changes without explicit instruction (additive OK)
- [ ] One focused change per commit
- [ ] Build passes (`npm run build`)
- [ ] Tests pass (`npm test`)
- [ ] Targeted manual sanity check (see Smoke Test Critical Path below)
- [ ] If adding new IPC channel: document in `docs/IPC_CONTRACTS.md`

### Smoke Test Critical Path

Before committing, verify the critical path works:
1. Load a profile
2. Load reference PDF
3. Draw ROI
4. Save profile
5. Run merge operation

**Merge Wizard Full Flow**:
1. Step 1: Select profile, choose scope (drawings/specs/both), optionally select narrative PDF
2. Step 2: Select files per lane (base, addenda, output)
3. Step 3: Review inventory, apply filters, optionally ignore rows/override IDs, click "Recompute"
4. Step 4: Execute merge, verify results, test "Next Addendum" stacking flow

**Note**: Full smoke test matrix documentation is planned but not yet created.

### Debugging Tips

**Main Process Logs**:
- Check terminal/console where Electron was launched
- All `console.log()` from `src/main/**` appear here
- IPC handler logs: `logger.logIpcRequest/Success/Failure()`

**Renderer Process Logs**:
- Press `F12` to toggle DevTools (keyboard shortcut configured in `main.ts`)
- Console tab shows all renderer logs
- Network tab shows IPC calls (if enabled)

**IPC Debugging**:
- Preload unwraps responses automatically
- If handler returns `{ success: false }`, preload throws Error
- Check handler return value in main process logs

### Known Sharp Edges

**pdf.js workerSrc Warning**:
- Harmless warning about worker source (relative path in packaged apps)
- Warning logged in `wizard-utils.js` when workerSrc is relative
- Can be ignored or suppressed in main.ts
- Cache-related console messages are automatically suppressed in `main.ts`

**Viewport Reset**:
- When viewport is reset (cleanup), ROI overlay must rebind
- `RoiOverlayController.updateROIOverlays()` is no-op until nodes available
- Single debug log on first skip (not spam)

**Profile Storage**:
- Profiles stored in `userData/profiles/`
- Old structure auto-migrated on read
- Snapshot PDFs stored in profile folder

**IPC Response Envelope**:
- All handlers MUST return `IpcResponse<T>`
- Never throw - always return error response
- Preload unwraps automatically

## Current Capabilities & Limitations

**Implemented**:
- Merge wizard 4-step flow (Configure, Select Files, Inventory & Corrections, Execute)
- Lane-aware file selection (drawings/specs/both)
- Inventory review with filters (search, confidence threshold, status)
- Minimal corrections: ignore rows, override normalizedId/sheetId
- Corrections recompute via `merge:analyze` with `CorrectionOverlay`
- Sequential lane execution (drawings then specs when scope='both')
- "Next Addendum" stacking flow (output becomes new base, clears addenda)
- CLI dry-run inventory output (`--dry-run`, `--json-output`)
- Core workflow engine: `createMergeWorkflowRunner()` with `analyze()`, `applyCorrections()`, `execute()`

**Not Yet Implemented**:
- Narrative parsing (narrative PDF path stored in wizard state but not parsed/used)
- Action overrides in corrections (only ID overrides supported)
- Multi-lane merge in core (GUI supports 'both' scope but core executes lanes separately)
- Narrative vs detection conflicts (conflicts array empty in `InventoryResult`)

**Recently Completed**:
- Specifications profile type support with dynamic ROI keys (sheetId/sheetTitle for Drawings, sectionId/sectionTitle for Specifications)
- Specs inventory display labels now dynamically update based on profile type
- ROI overlay properly handles profile type switching

**Known Follow-ups**:
- Narrative parsing integration with detection system
- Additional profile types beyond Drawings and Specifications

### Build Commands

```bash
npm run build      # Compile TypeScript + copy assets
npm run dev        # Build + launch Electron
npm test           # Run tests
```

### Test Structure

- `tests/gui/` - GUI unit tests
  - `wizard-merge.test.ts` - Wizard logic, page sampling, ROI conversion
  - `ipc-handlers.test.ts` - IPC handler tests
- `tests/integration/` - Integration tests
  - `ipc-handlers.test.ts` - IPC handler integration tests
  - `pdf-render.test.ts` - PDF rendering tests
  - `profile-save-load.test.ts` - Profile persistence tests
  - `roi-coordinates.test.ts` - ROI coordinate tests

### Common Issues

**Build fails**:
- Check `@conset-pdf/core` is available at `../conset-pdf/packages/core`
- Run `npm install` in both repos

**IPC not working**:
- Check handler registered in `src/main/ipc/index.ts`
- Check preload exposes method in `src/preload.ts`
- Check renderer uses `window.api.*` (not direct IPC)

**Profile not loading**:
- Check `userData/profiles/` directory exists
- Check profile JSON is valid
- Check migration logic if old structure
