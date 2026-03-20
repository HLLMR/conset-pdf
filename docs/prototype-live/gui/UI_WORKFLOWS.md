# UI Workflows Documentation

**Last verified**: 2026-03-01

**Recent Updates**:
- Single addendum workflow (changed from multi-select)
- Analysis runs on Step 2 forward navigation with caching
- Narrative validation UI in Step 3
- Set Order sorting with grouping headers

This document describes the user interface workflows in the conset-pdf-gui application, including wizard steps, inventory review, and UX behaviors.

## Update Documents Wizard

The Update Documents wizard is a 4-step workflow for merging addenda into original construction document sets.

### Step 1: Configure

**Purpose**: Select profile, document scope, and optional narrative PDF.

**UI Elements**:
- **Active Profile** dropdown: Select layout profile (required)
  - Shows profile name
  - Displays ROI configuration status based on profile type:
    - **Drawings**: Sheet ID ROI and Sheet Title ROI status
    - **Specifications**: Section ID ROI and Section Title ROI status
  - "Edit in Profiles →" button navigates to Profiles view
  - Validation: Profile must have the appropriate ROI fields configured (ID and Title) based on its type

- **Scope** toggle grid: Select document types to update (required)
  - **Drawings**: Sheeted plan pages
  - **Specs**: Divisions / sections
  - Can select both (multi-lane workflow)
  - Validation: At least one scope must be selected

- **Narrative PDF** (optional): Path to addendum narrative PDF
  - Browse button to select file
  - Clear button to remove selection
  - Helper text: "Providing the official addendum narrative greatly improves detection and placement accuracy"

**Validation**: Profile and scope must be selected before proceeding.

**Navigation**:
- **Next**: Always goes to Step 2 (Select Files), never skips
- **Back**: Returns to Dashboard (if on step 0)

### Step 2: Select Files

**Purpose**: Choose original PDF, single addendum PDF, and output path for each selected scope.

**UI Elements** (per scope lane):
- **Original PDF**: File picker for base document set
- **Addendum PDF**: Single file picker for addendum PDF (one addendum per wizard run)
- **Output PDF**: Save dialog for merged output file

**Multi-Lane Behavior**:
- If scope is `'both'` (drawings + specs), shows separate file selection for each:
  - Drawings lane: `drawings.basePath`, `drawings.addendaPaths[0]`, `drawings.outputPath`
  - Specs lane: `specs.basePath`, `specs.addendaPaths[0]`, `specs.outputPath`
- If scope is `'drawings'` or `'specs'`, shows single lane

**Validation**: All required files must be selected before proceeding (exactly one addendum per lane).

**Analysis Behavior**:
- **Automatic Analysis**: When clicking Next (forward navigation), analysis runs automatically
- **Caching**: Analysis results are cached with dirty checking
  - Analysis key computed from: docType, basePath, addendumPath, outputPath, narrativePath, profile
  - Only re-analyzes if key changed or no cached analysis exists
  - Prevents unnecessary re-analysis when navigating back/forward
- **Narrative Integration**: Narrative PDF (if provided) is passed to analysis

**Navigation**:
- **Next**: Triggers analysis (if needed), then goes to Step 3 (Inventory & Corrections)
- **Back**: Returns to Step 1 (does not trigger analysis)

### Step 3: Inventory & Corrections

**Purpose**: Review detected sheets, ignore rows, override IDs before execution.

**What Happens**:
1. **Uses Cached Analysis**: Step 3 displays cached analysis from Step 2 (does not trigger new analysis)
   - If no cached analysis exists, shows message directing user to complete Step 2
   - Analysis must be run from Step 2 forward navigation

2. **Narrative Validation Section** (if narrative PDF provided):
   - Displays narrative validation report at top of Step 3
   - Shows issue summary (total issues, by severity, by type)
   - Issues table with codes, severity, messages, and references
   - Near-match suggestions displayed for typo candidates
   - Suggested corrections with "Apply Suggestion" buttons
   - Clicking "Apply Suggestion" applies correction and triggers recompute

3. **Inventory Table** with:
   - **Rows**: One row per page showing:
     - Page number
     - Detected sheet ID (`normalizedId`)
     - Status (ok, warning, error)
     - Confidence level
     - Source (roi, legacy, etc.)
     - Planned action (replace, insert, keep)
   - **Sort Toggle**: "Set Order" (standards-based with grouping) or "Source Order"
   - **Grouping Headers**: When Set Order selected, groups by:
     - Drawings: Discipline (e.g., "MECH (M)")
     - Specs: Division (e.g., "23 — Heating, Ventilating, and Air Conditioning (HVAC)")
   - **Issues**: List of detection problems
   - **Summary**: Statistics (total rows, rows with IDs, replaced, inserted, unmatched)

4. User can make corrections:
   - **Ignore rows**: Checkbox to exclude pages from merge
     - Rows remain visible but excluded from summary counts
     - Tagged with `'ignored'` status
   - **Override IDs**: Edit detected sheet ID
     - Updates `normalizedId` field (stable `row.id` unchanged)
     - Keyed by stable `row.id`

5. When corrections are made:
   - Calls `merge:analyze` again with `corrections` overlay (via Recompute button)
   - Recomputes inventory with corrections applied
   - Updates summary counts (excludes ignored rows)
   - Invalidates analysis cache (forces re-analysis on next Step 2→Step 3 navigation)

**Multi-Lane Behavior**:
- If scope is `'both'`:
  - Step 3a: Inventory & Corrections for Drawings
  - Step 3b: Inventory & Corrections for Specs (hidden from stepper)
  - After Step 3a, automatically navigates to Step 3b
  - Both lanes must be reviewed before proceeding to Step 4

**Navigation**:
- **Next**: Goes to Step 4 (Execute) after all lanes reviewed
- **Back**: Returns to Step 2

### Step 4: Execute

**Purpose**: Run merge operation and display results.

**What Happens**:
1. Calls `runMerge` IPC channel (canonical merge handler)
2. Shows progress indicator
3. Displays results:
   - Output file path
   - Statistics (replaced, inserted, unmatched, final pages)
   - Warnings (if any)
4. Option to open output folder or view history

**Multi-Lane Behavior**:
- If scope is `'both'`, executes both lanes sequentially:
  - First drawings merge
  - Then specs merge
  - Shows results for both

**Navigation**:
- **Done**: Returns to Dashboard
- **Back**: Returns to Step 3 (allows re-review before re-executing)

## Inventory Review Details

### Inventory Table Structure

Each row in the inventory table represents one page from the input PDFs (original + addenda).

**Row Fields**:
- `id`: Stable identifier (format: `${source}:${pageIndex}:${idPart}`)
  - Never changes when corrections applied
  - Used as key for corrections overlay
- `normalizedId`: Detected/overridden sheet ID (e.g., "A-101")
  - Updated when user overrides ID
- `page`: Page number (1-based)
- `status`: `'ok'`, `'warning'`, `'error'`, or `'conflict'`
- `confidence`: Detection confidence (0.0 to 1.0)
- `source`: Detection source (`'roi'`, `'legacy'`, etc.)
- `action`: Planned action (`'replace'`, `'insert'`, `'keep'`, etc.)

### Corrections UI

**Ignore Rows**:
- Checkbox per row
- When checked: Row tagged with `'ignored'`, excluded from summary counts
- Rows remain visible in table (not filtered out)

**Override IDs**:
- Editable field for `normalizedId`
- When edited: Updates `row.normalizedId`, stable `row.id` unchanged
- Corrections keyed by stable `row.id`

**Re-compute on Correction**:
- When corrections are made, calls `merge:analyze` with `corrections` overlay
- `applyCorrections()` re-runs `analyze()` first for fresh state
- Then applies corrections overlay
- Updates inventory table and summary

## Single Addendum Workflow

**Important**: Each wizard run processes exactly **one addendum PDF** at a time.

- Single addendum file picker in Step 2 (not multi-select)
- One addendum PDF passed to `merge:analyze` and `runMerge`
- The core workflow engine analyzes the single addendum against the original
- All replacements and insertions are planned in one pass
- A single merged output PDF is produced

**Example**:
- Original: `Drawings_Base.pdf`
- Addendum: `Drawings_A1.pdf`
- Result: Single merged PDF with changes from addendum applied

**Processing Multiple Addenda**: Use the "Next Addendum" button after execution to process additional addenda sequentially. Each addendum is processed in a separate wizard run, with the previous output becoming the new base.

## "Next Addendum" Button Behavior

After executing a merge operation (Step 4), a **"Next Addendum"** button appears in the results view. This button allows you to process additional addenda in a separate operation:

1. **Output becomes new base**: The merged output PDF becomes the new base document
2. **Addenda cleared**: The addenda list is cleared (ready for next addendum)
3. **Output path auto-incremented**: Output path suffix is incremented (e.g., `-A1.pdf` → `-A2.pdf`)
4. **Navigates to Step 2**: Returns to file selection to choose the next addendum
5. **State cleared**: Analysis and corrections are cleared (fresh start)

**Use Case**: Process addenda one at a time, with each merge result becoming the base for the next addendum.

**Example Flow**:
1. Original: `Drawings_Base.pdf`, Addendum: `Drawings_A1.pdf` → Output: `Drawings_Merged-A1.pdf`
2. Click "Next Addendum"
3. Base: `Drawings_Merged-A1.pdf` (auto-filled), Addendum: `Drawings_A2.pdf` → Output: `Drawings_Merged-A2.pdf`

## Quick-Start Drawers

### Global Positioning System

All quick-start drawers use a unified header slot system:

**Positioning**:
- Positioned in `#appHeaderDrawerSlot` (absolute positioning at top of main-panel)
- Overlays content (does not push content down)
- Centered with `margin: 0 auto`
- Fixed width: 1100px (matches page content width)
- Max-width: 100% (responsive)
- z-index: 100

**Visual Design**:
- White background with bottom border radius
- No top border radius (seamless with header)
- Consistent spacing and styling across all views

### QS-2 (Workflow Drawers)

**Location**: Header slot (mounted dynamically per workflow)

**Supported Workflows**:
- Update Documents (`merge`)
- Extract Documents (`split`)
- Fix Bookmarks (`bookmark`)

**Behavior**:
- Per-workflow drawer with workflow-specific tips
- Mounted via `window.__app.mountHeaderQuickStartDrawer(workflowId, tipsHtml)`
- Unmounted when leaving workflow via `window.__app.unmountHeaderQuickStartDrawer()`
- Dismiss state stored per workflow: `conset.qs2.{workflowId}.dismissed`
- "Got it" button collapses drawer
- "Don't show again" checkbox persists dismissal
- "Show tips" tab allows re-expansion after dismissal

**Content** (Update Documents workflow):
- Keep base documents and addenda separate
- Separate drawings and specs
- Extract the addendum narrative (recommended)
- Use clean, obvious filenames
- Review before finalizing

### QS-3 (Profiles Drawer)

**Location**: Header slot (mounted when Profiles view loads)

**Behavior**:
- Mounted via `window.__app.mountHeaderQuickStartDrawer('profiles', tipsHtml)` in `loadProfiles()`
- Same persistence and collapse behavior as workflow drawers
- Dismiss state stored in `conset.qs2.profiles.dismissed`
- Appears immediately when navigating to Profiles view

**Content**:
- Set up profiles by engineer/architect
- Set ROIs generously
- Use small files for setting ROIs
- Don't worry about perfection (90% is good enough)

### Persistence Rules

- Dismiss state per view/workflow (different localStorage keys)
- State persists across app sessions
- Can be re-enabled by clearing localStorage
- Each drawer maintains independent dismiss state

## Per-Workflow Persistence

Wizard state is persisted per workflow:

- **Selected profile**: Stored in wizard state
- **File paths**: Stored in wizard state (per lane if multi-lane)
- **Corrections**: Stored in wizard state
- **Current step**: Stored in wizard state

**State Management**:
- State cleared when navigating away from wizard
- Return navigation from Profiles view preserves workflow context
- `returnTo` option in navigation preserves step and workflow

**Example**:
```javascript
// Navigate to Profiles with return context
window.appNavigate('profiles', {
  returnTo: { 
    view: 'wizard', 
    workflow: 'merge', 
    mode: 'drawings', 
    step: 0 
  }
});

// When returning, wizard state is restored
```

## Wizard Shell Features

The wizard uses `WizardShell` class for consistent behavior:

- **Step validation**: Each step can define `validate()` function
- **Step navigation**: Custom `onEnter`, `onExit` handlers
- **State management**: Shared state object across steps
- **Stepper UI**: Visual step indicator (can hide steps with `stepperVisible: false`)
- **Quick-start integration**: Mounts QS-2 drawer in header slot

## Other Workflows

### Extract Documents (Split)

**Status**: ⚠️ Placeholder wizard exists but not fully implemented

**Planned Steps**:
1. Configure (profile, document type)
2. Select Files (input PDF, output directory)
3. Preview (generated filenames)
4. Execute (split operation)

### Fix Bookmarks

**Status**: ⚠️ Placeholder wizard exists but not fully implemented

**Planned Steps**:
1. Configure (profile, document type)
2. Select Files (input PDF, output PDF)
3. Execute (regenerate bookmarks)
