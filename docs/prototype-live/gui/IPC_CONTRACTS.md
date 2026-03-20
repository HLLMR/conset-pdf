# IPC Contracts Documentation

**Last verified**: 2026-03-01

This document describes the IPC (Inter-Process Communication) contracts between the renderer and main processes.

## Response Envelope

All IPC handlers return responses in a standardized envelope format:

```typescript
interface IpcResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
}
```

- **success**: `true` if the operation succeeded, `false` if it failed
- **data**: The result data (only present when `success === true`)
- **error**: Error message (only present when `success === false`)

**Invariant**: Main process always returns `{ success, data?, error? }`. Preload unwraps this so renderer code never sees the envelope. This exists specifically to prevent renderer churn when error handling changes. The preload script automatically unwraps this envelope, so renderer code receives the data directly or throws an error.

## IPC Channels

### Dialog Handlers

#### `dialog:selectFile`
- **Request**: `{ title: string; filters?: Array<{ name: string; extensions: string[] }> }`
- **Response**: `IpcResponse<string | null>`
- **Success Data**: Selected file path, or `null` if canceled
- **Error Conditions**: Dialog initialization failure

#### `dialog:selectFiles`
- **Request**: `{ title: string; filters?: Array<{ name: string; extensions: string[] }> }`
- **Response**: `IpcResponse<string[]>`
- **Success Data**: Array of selected file paths (empty if canceled)
- **Error Conditions**: Dialog initialization failure

#### `dialog:selectFolder`
- **Request**: `{ title: string }`
- **Response**: `IpcResponse<string | null>`
- **Success Data**: Selected folder path, or `null` if canceled
- **Error Conditions**: Dialog initialization failure

#### `dialog:saveFile`
- **Request**: `{ title: string; defaultPath?: string; filters?: Array<{ name: string; extensions: string[] }> }`
- **Response**: `IpcResponse<string | null>`
- **Success Data**: Selected save path, or `null` if canceled
- **Error Conditions**: Dialog initialization failure

### PDF Handlers

#### `pdf:getPageCount`
- **Request**: `string` (PDF file path)
- **Response**: `IpcResponse<number>`
- **Success Data**: Number of pages in the PDF
- **Error Conditions**:
  - Invalid file path
  - File is not a PDF
  - PDF cannot be read or parsed

#### `pdf:getPageInfo`
- **Request**: `string` (PDF file path), `number` (page index, 0-based)
- **Response**: `IpcResponse<{ width: number; height: number; rotation: number }>`
- **Success Data**: Page dimensions and rotation
- **Error Conditions**:
  - Invalid file path
  - File is not a PDF
  - Page index out of range
  - PDF cannot be read or parsed

#### `pdf:getFileData`
- **Request**: `string` (PDF file path)
- **Response**: `IpcResponse<Uint8Array>`
- **Success Data**: PDF file contents as Uint8Array
- **Error Conditions**:
  - File not found
  - File cannot be read

### Detection Handlers

#### `sampleDetect`
- **Request**: `{ basePdfPath: string; rois: { sheetId: {...} | null; sheetTitle: {...} | null }; pages: number[]; mode: ConsetDocType }`
- **Response**: `IpcResponse<PageDetectionResult[]>`
- **Success Data**: Array of detection results for each page
- **Error Conditions**:
  - Invalid PDF path
  - Invalid page numbers
  - PDF cannot be read or parsed
  - Detection failure

#### `detect:run`
- **Request**: `{ pdfPath: string; pageNumbers: number[]; layoutProfile?: LayoutProfile; docType: ConsetDocType }`
- **Response**: `IpcResponse<PageDetectionResult[]>`
- **Success Data**: Array of detection results for each page
- **Error Conditions**:
  - Invalid PDF path
  - Invalid page numbers
  - PDF cannot be read or parsed
  - Detection failure

### Operations Handlers

#### `runMerge` (Canonical)
- **Request**: `{ mode: ConsetDocType; basePdfPath: string; addendaPaths: string[]; rois: {...}; outputPath: string }`
- **Response**: `IpcResponse<MergeReport>`
- **Success Data**: Merge operation report with statistics
- **Error Conditions**:
  - Invalid PDF paths
  - PDFs cannot be read or parsed
  - Merge operation failure
  - Output path cannot be written
- **Notes**: 
  - **Canonical merge handler** used by wizard UI (Step 4 Execute)
  - Routes through `merge-internal.ts` → core workflow runner (`createMergeWorkflowRunner`)
  - Execute mode: Uses `runner.execute()` and adapts `ExecuteResult` to `MergeReport` via `executeResultToMergeReport()` adapter for backward compatibility
  - Always uses `mode: 'replace+insert'`, `regenerateBookmarks: false`, `verbose: true`
  - Creates layout profile from ROIs automatically
  - **Recommendation**: Use this handler for new code (canonical entry point)

#### `previewNaming`
- **Request**: `{ inputPdfPath: string; rois: {...}; pages: number[]; mode: ConsetDocType; filenameFormat: string; splitMode: 'grouped' | 'single' }`
- **Response**: `IpcResponse<PreviewResult[]>`
- **Success Data**: Array of preview results showing generated filenames
- **Error Conditions**:
  - Invalid PDF path
  - Invalid page numbers
  - PDF cannot be read or parsed
  - Detection failure

#### `runSplit`
- **Request**: `{ mode: ConsetDocType; inputPdfPath: string; outputFolder: string; rois: {...}; filenameFormat: string; splitMode: 'grouped' | 'single' }`
- **Response**: `IpcResponse<{ outputFolder: string; entries: SplitEntry[]; warnings: string[] }>`
- **Success Data**: Split operation results with entries and warnings
- **Error Conditions**:
  - Invalid PDF path
  - Invalid output folder
  - PDF cannot be read or parsed
  - Split operation failure
  - File write failures

### Merge Handlers

#### `merge:analyze` (Additive - New)
- **Request**: `{ originalPdfPath: string; addendumPdfPaths: string[]; layoutProfile?: LayoutProfile; docType: ConsetDocType; mode?: 'replace+insert' | 'replace-only' | 'append-only'; corrections?: CorrectionOverlay; narrativePdfPath?: string }`
- **Response**: `IpcResponse<InventoryResult>`
- **Success Data**: Inventory analysis result with rows, issues, conflicts, and summary
- **Error Conditions**:
  - Invalid PDF paths
  - PDFs cannot be read or parsed
  - Analysis failure
- **Notes**:
  - **Additive channel** for Step 3 inventory review (no breaking changes)
  - Uses core workflow runner `createMergeWorkflowRunner().analyze()` method
  - If `corrections` is provided: calls `analyze()` first, then `applyCorrections()` with the corrections overlay
  - If `narrativePdfPath` is provided: passes to core analyze for narrative validation (advisory only)
  - Does not write output files (dry-run mode only)
  - Returns `InventoryResult` for display in wizard inventory table
  - **Corrections behavior**: Ignored rows remain visible but excluded from counts; ID overrides update `row.normalizedId` (stable `row.id` unchanged)
  - **Narrative validation**: If narrative provided, `InventoryResult.narrativeValidation` contains validation issues and optional suggested corrections (advisory only, does not modify detection results)

#### `merge:run` (Legacy Alias)
- **Request**: `{ originalPdfPath: string; addendumPdfPaths: string[]; outputPdfPath: string; layoutProfile?: LayoutProfile; docType: ConsetDocType; mode?: 'replace+insert' | 'replace-only' | 'append-only'; regenerateBookmarks?: boolean }`
- **Response**: `IpcResponse<MergeReport>`
- **Success Data**: Merge operation report with statistics
- **Error Conditions**:
  - Invalid PDF paths
  - PDFs cannot be read or parsed
  - Merge operation failure
  - Output path cannot be written
- **Notes**: 
  - **Legacy alias** - routes to same internal implementation as `runMerge` via `merge-internal.ts`
  - Both handlers use `runMergeInternal()` → core workflow runner (`createMergeWorkflowRunner`)
  - Execute mode: Uses `runner.execute()` and adapts `ExecuteResult` to `MergeReport` via `executeResultToMergeReport()` adapter for backward compatibility
  - Supports optional layout profile (if not provided, uses default locator)
  - Supports configurable merge mode and bookmark regeneration
  - **Recommendation**: Use `runMerge` for new code (canonical handler). This alias is preserved for backward compatibility only.

### Profile Handlers

#### `profiles:list`
- **Request**: None
- **Response**: `IpcResponse<ProfileSummary[]>`
- **Success Data**: Array of profile summaries (id, name, type, updatedAt)
- **Error Conditions**:
  - Profiles directory cannot be accessed
  - Index file corruption

#### `profiles:read`
- **Request**: `string` (profile ID)
- **Response**: `IpcResponse<Profile>`
- **Success Data**: Full profile object
- **Error Conditions**:
  - Profile not found
  - Profile file corruption
  - Invalid profile schema

#### `profiles:save`
- **Request**: `Profile` (partial profile object)
- **Response**: `IpcResponse<Profile>`
- **Success Data**: Saved profile object
- **Error Conditions**:
  - Invalid profile data
  - Profile directory cannot be created
  - File write failure

#### `profiles:delete`
- **Request**: `string` (profile ID)
- **Response**: `IpcResponse<void>`
- **Success Data**: `undefined`
- **Error Conditions**:
  - Profile not found
  - File deletion failure

#### `profiles:setActive`
- **Request**: `string | null` (profile ID)
- **Response**: `IpcResponse<void>`
- **Success Data**: `undefined`

### Settings Handlers

#### `settings:standards:get`
- **Request**: None
- **Response**: `IpcResponse<StandardsSettingsPayload>`
- **Success Data**: Current standards customizations plus effective/default disciplines, divisions, and aliases.

#### `settings:standards:save`
- **Request**: `UserCustomizations`
- **Response**: `IpcResponse<StandardsSettingsPayload>`
- **Success Data**: Saved and reloaded standards payload.

#### `settings:standards:reset`
- **Request**: None
- **Response**: `IpcResponse<StandardsSettingsPayload>`
- **Success Data**: Defaults restored and returned.

#### `settings:standards:import`
- **Request**: `string` (file path)
- **Response**: `IpcResponse<StandardsSettingsPayload>`
- **Success Data**: Imported and reloaded standards payload.
- **Supported file formats**: `.json`, `.csv`, `.xlsx`, `.xls`

#### `settings:standards:export`
- **Request**: `string` (file path), optional `UserCustomizations`
- **Response**: `IpcResponse<boolean>`
- **Success Data**: `true` when export file is written.
- **Error Conditions**:
  - Settings file cannot be written

#### `profiles:getActive`
- **Request**: None
- **Response**: `IpcResponse<Profile | null>`
- **Success Data**: Active profile object, or `null` if none
- **Error Conditions**:
  - Settings file cannot be read
  - Active profile not found (returns null, not error)

#### `profiles:chooseReferencePdf`
- **Request**: None
- **Response**: `IpcResponse<string | null>`
- **Success Data**: Selected PDF path, or `null` if canceled
- **Error Conditions**: Dialog initialization failure

#### `profiles:captureReferenceSnapshot`
- **Request**: `{ profileId: string; sourcePdfPath: string; pageNumbers: number[] }`
- **Response**: `IpcResponse<ReferenceObject>`
- **Success Data**: Reference object with snapshot path
- **Error Conditions**:
  - Source PDF not found
  - Invalid page numbers
  - PDF cannot be read or parsed
  - Snapshot creation failure

#### `profiles:getSnapshotPath`
- **Request**: `string` (relative path)
- **Response**: `IpcResponse<string>`
- **Success Data**: Absolute snapshot path
- **Error Conditions**:
  - Snapshot not found
  - Path resolution failure

### System Handlers

#### `fs:readFile`
- **Request**: `string` (file path)
- **Response**: `IpcResponse<string>`
- **Success Data**: File contents as string
- **Error Conditions**:
  - File not found
  - File cannot be read
  - Permission denied

#### `fs:writeFile`
- **Request**: `string` (file path), `string` (content)
- **Response**: `IpcResponse<boolean>`
- **Success Data**: `true`
- **Error Conditions**:
  - Directory does not exist
  - File cannot be written
  - Permission denied

#### `fs:exists`
- **Request**: `string` (file path)
- **Response**: `IpcResponse<boolean>`
- **Success Data**: `true` if file exists, `false` otherwise
- **Error Conditions**: None (always returns success)

#### `shell:openPath`
- **Request**: `string` (path)
- **Response**: `IpcResponse<void>`
- **Success Data**: `undefined`
- **Error Conditions**:
  - Path does not exist
  - System cannot open path

### History Handlers

#### `history:save`
- **Request**: `{ workflow: string; mode: string; basePdfPath: string; outputPath: string; status: string; timestamp: string; report?: any }`
- **Response**: `IpcResponse<{ id: string }>`
- **Success Data**: Run ID
- **Error Conditions**:
  - History directory cannot be created
  - File write failure

#### `history:get`
- **Request**: None
- **Response**: `IpcResponse<RunData[]>`
- **Success Data**: Array of all run history entries
- **Error Conditions**: None (returns empty array on error)

#### `history:getRecent`
- **Request**: `number` (limit, default 5)
- **Response**: `IpcResponse<RunData[]>`
- **Success Data**: Array of recent run history entries
- **Error Conditions**: None (returns empty array on error)

#### `history:clear`
- **Request**: None
- **Response**: `IpcResponse<boolean>`
- **Success Data**: `true`
- **Error Conditions**:
  - Index file cannot be written

### Debug Handlers

#### `debug:getPageTextItems`
- **Request**: `{ pdfPath: string; pageNumber: number }`
- **Response**: `IpcResponse<TextItemsResult>`
- **Success Data**: Text items extracted from page
- **Error Conditions**:
  - Invalid PDF path
  - Invalid page number
  - PDF cannot be read or parsed
  - Text extraction failure

## Rules

### Envelope Format

All handlers MUST return `IpcResponse<T>`:
- `success: true` → `data` contains result
- `success: false` → `error` contains message string

**Never throw errors** from handlers - always return error response.

### Cancel vs Error Semantics

**Cancel** (user action):
- Dialogs: `canceled: true` → return `null` (not an error)
- Returns `{ success: true, data: null }`

**Error** (system failure):
- File not found, validation failure, etc.
- Returns `{ success: false, error: "message" }`

### Preload Unwrap Behavior

Preload automatically unwraps responses:

```typescript
// Handler returns
{ success: true, data: "result" }
// Preload unwraps to
"result"

// Handler returns
{ success: false, error: "File not found" }
// Preload unwraps to
throw new Error("File not found")
```

**Renderer code** receives data directly or throws - no envelope handling needed.

## Error Handling

All errors are returned in the standardized response envelope. The preload script automatically:
1. Checks the `success` field
2. Returns `data` if successful
3. Throws an `Error` with the `error` message if unsuccessful

This ensures consistent error handling across all IPC calls without requiring changes to renderer code.

## Future IPC Versioning Strategy

### Current Status
- All IPC channels use unversioned names (e.g., `pdf:getPageCount`)
- Response format is standardized but not versioned
- No breaking changes have been introduced

### Future Considerations

If IPC versioning becomes necessary, consider the following approach:

1. **Channel Versioning**: Add version suffix to channel names
   - Example: `pdf:getPageCount:v2`
   - Keep old channels for backward compatibility during transition

2. **Response Envelope Versioning**: Add version field to response envelope
   ```typescript
   interface IpcResponse<T = any> {
     version?: string; // Optional version field
     success: boolean;
     data?: T;
     error?: string;
   }
   ```

3. **Request Versioning**: Include version in request payload
   - Example: `{ version: '2.0', ...otherParams }`
   - Handler can check version and adapt behavior

4. **Migration Strategy**:
   - Support multiple versions simultaneously
   - Deprecate old versions with warnings
   - Remove old versions after sufficient transition period
   - Document version compatibility matrix

5. **When to Version**:
   - Breaking changes to request/response shapes
   - Changes to error handling behavior
   - Changes to channel semantics
   - Major feature additions requiring new contract

### Recommendation

For now, versioning is not needed. The standardized response envelope provides a stable foundation. If versioning becomes necessary:
- Start with response envelope versioning (least disruptive)
- Add channel versioning only if channel semantics change
- Maintain backward compatibility during transitions
- Document all version changes in this file
