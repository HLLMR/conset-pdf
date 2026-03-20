# Legacy Specs Support Architecture

## Overview

Phase 4 of the Standards Refactor adds comprehensive support for **pre-2004 MasterFormat 5-digit section codes** (e.g., `23050`, `15100`, `26010`). This document describes the architecture, detection pipeline, and integration points.

## Background: MasterFormat Formats

### Modern Format (Post-2004)
- **Pattern**: 6-digit with spaces: `XX YY ZZ`
- **Example**: `23 09 00`, `26 05 00`, `22 11 13`
- **Division**: First 2 digits (e.g., `23` for HVAC)

### Legacy Format (Pre-2004)
- **Pattern**: 5-digit with no spaces: `XXYYY`
- **Example**: `23050`, `15100`, `26010`
- **Division**: First 2 digits (e.g., `23` for mechanical)
- **Mapping**: Maps to modern divisions via UDS.xlsx legacy table

## Architecture Components

```
┌─────────────────────────────────────────────────────────┐
│                   Detection Layer                       │
│  (Specs transcript extraction & validation)             │
├─────────────────────────────────────────────────────────┤
│  extractSpecSectionId()                                 │
│  ├─ Regex: /\d{2}\s+\d{2}\s+\d{2}/ (modern) ✓          │
│  ├─ Regex: /\d{5}/ (legacy fallback) ✓                 │
│  └─ Returns: { sectionId, format: 'MODERN'|'LEGACY' }  │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│              Normalization Layer                        │
│  (Standards registry & metadata resolution)             │
├─────────────────────────────────────────────────────────┤
│  normalizeSpecsMasterformat()                           │
│  ├─ Modern lookup: MASTERFORMAT_DIVISIONS map          │
│  ├─ Legacy lookup: standardsRegistry.resolveLegacy()   │
│  └─ Returns: DivisionMeta { divisionID, division,      │
│              order, confidence, basis }                 │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│             splitSet Integration                        │
│  (File grouping & splitting logic)                     │
├─────────────────────────────────────────────────────────┤
│  splitSet() - Specs Mode                                │
│  ├─ Detects modern 6-digit patterns (priority)         │
│  ├─ Fallback: detects legacy 5-digit patterns          │
│  ├─ Normalizes via normalizeSpecsMasterformat()        │
│  ├─ Groups by sectionIdNormalized (section mode)       │
│  └─ Groups by division (division mode, auto-migrated)  │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│             Settings UI Layer                           │
│  (View-only legacy sections table)                     │
├─────────────────────────────────────────────────────────┤
│  conset-pdf-gui/src/app.html                            │
│  ├─ Read-only table: #settings-legacy-section-table    │
│  └─ Columns: Legacy Div | Section Range | Title |      │
│               Modern Div | Modern Code                  │
│                                                         │
│  conset-pdf-gui/src/settings-view.js                    │
│  ├─ renderLegacySections(payload.legacySections)       │
│  └─ Sorts by legacyDivID then sectionRange             │
└─────────────────────────────────────────────────────────┘
```

---

## Implementation Details

### 1. Standards Registry Enhancement

**File**: `conset-pdf/packages/core/src/standards/registry.ts`

Added method to retrieve all legacy section mappings:

```typescript
/**
 * Get all legacy sections (pre-2004) for UI display
 * Returns flat array sorted by legacy division ID
 */
public getAllLegacySections(): LegacySectionEntry[] {
  return Array.from(this.legacySections.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .flatMap(([, sections]) => sections);
}
```

**Data Structure**:
```typescript
interface LegacySectionEntry {
  legacyDivID: string;      // "15", "16", "23"
  sectionRange: string;     // "15100-15300"
  sectionTitle: string;     // "Basic Mechanical Materials and Methods"
  sectionNotes?: string;    // Keywords for fuzzy matching
  divisionID: string;       // Modern equivalent: "23"
  divisionCODE: string;     // Modern code: "HVAC"
}
```

---

### 2. Settings IPC Extension

**File**: `conset-pdf-gui/src/main/ipc/settings.ts`

Extended payload to include legacy sections:

```typescript
export interface StandardsSettingsPayload {
  defaults: {
    legacySections: ReturnType<typeof standardsRegistry.getAllLegacySections>;
    // ... other fields
  };
  legacySections: ReturnType<typeof standardsRegistry.getAllLegacySections>;
  // ... other fields
}

function getStandardsPayload(): StandardsSettingsPayload {
  return {
    defaults: {
      legacySections: standardsRegistry.getAllLegacySections(),
      // ...
    },
    legacySections: standardsRegistry.getAllLegacySections(),
    // ...
  };
}
```

---

### 3. Settings UI (Read-Only View)

**File**: `conset-pdf-gui/src/app.html`

Added read-only table for legacy sections:

```html
<div class="settings-panel" id="settings-panel-standards">
  <!-- ... existing tables ... -->
  
  <!-- Legacy Sections (Pre-2004) -->
  <h3>Legacy Sections (Pre-2004)</h3>
  <p class="help-text">
    Reference table showing how pre-2004 5-digit section codes map to modern MasterFormat divisions.
  </p>
  <table id="settings-legacy-section-table">
    <thead>
      <tr>
        <th>Legacy Div</th>
        <th>Section Range</th>
        <th>Section Title</th>
        <th>Modern Division</th>
        <th>Modern Code</th>
      </tr>
    </thead>
    <tbody>
      <!-- Populated by renderLegacySections() -->
    </tbody>
  </table>
</div>
```

**File**: `conset-pdf-gui/src/settings-view.js`

Renderer function for legacy sections:

```javascript
function renderLegacySections(sections) {
  const tbody = document.querySelector('#settings-legacy-section-table tbody');
  tbody.innerHTML = '';

  // Sort by legacyDivID then sectionRange
  const sorted = [...sections].sort((a, b) => {
    if (a.legacyDivID !== b.legacyDivID) {
      return a.legacyDivID.localeCompare(b.legacyDivID);
    }
    return a.sectionRange.localeCompare(b.sectionRange);
  });

  for (const section of sorted) {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td>${escapeHtml(section.legacyDivID)}</td>
      <td>${escapeHtml(section.sectionRange)}</td>
      <td>${escapeHtml(section.sectionTitle)}</td>
      <td>${escapeHtml(section.divisionID)}</td>
      <td>${escapeHtml(section.divisionCODE)}</td>
    `;
    tbody.appendChild(tr);
  }
}
```

---

### 4. splitSet Legacy Detection

**File**: `conset-pdf/packages/core/src/core/splitSet.ts`

Enhanced specs detection with legacy fallback:

```typescript
// Modern pattern (priority)
const modernMatches = pageText.match(/\b(?:SECTION\s+)?(\d{2}\s+\d{2}\s+\d{2})\b/gi);

if (!modernMatches || modernMatches.length === 0 || confidence < 0.6) {
  // Fallback: Try legacy 5-digit pattern
  const legacyMatches = pageText.match(/\b(?:SECTION\s+)?(\d{5})\b/gi);
  
  if (legacyMatches && legacyMatches.length > 0) {
    for (const match of legacyMatches) {
      const candidate = match.replace(/^SECTION\s+/i, '').trim();
      
      // Validate via standards normalization
      const normalized = normalizeSpecsMasterformat({ normalizedId: candidate });
      
      if (normalized.basis === 'MASTERFORMAT_LEGACY' && normalized.confidence >= 0.6) {
        // Found valid legacy code
        sectionInfo = {
          page: i + 1,
          sectionIdNormalized: candidate,
          sectionTitle: normalized.division || candidate,
          divisionID: normalized.divisionID,
          division: normalized.division
        };
        break;
      }
    }
  }
}
```

**Key Behaviors:**
1. **Modern detection first**: Always tries 6-digit pattern first
2. **Legacy fallback**: Only attempts 5-digit if modern fails
3. **Standards validation**: Validates via `normalizeSpecsMasterformat()` before accepting
4. **Confidence threshold**: Requires `confidence >= 0.6` for legacy matches
5. **Auto-migration**: Legacy codes automatically map to modern `division` for grouping

---

### 5. Test Coverage

#### Unit Tests: `tests/standards/specsMasterformat.test.ts`

```typescript
describe('extractSpecSectionId', () => {
  test('extracts legacy pre-2004 5-digit section ID', () => {
    expect(extractSpecSectionId('23050')).toEqual({ 
      sectionId: '23050', 
      format: 'LEGACY' 
    });
  });
});

describe('normalizeSpecsMasterformat', () => {
  test('legacy 5-digit section IDs resolve via legacy mapping', () => {
    const result = normalizeSpecsMasterformat({ normalizedId: '23050' });
    expect(result.sectionId).toBe('23050');
    expect(result.basis).toBe('MASTERFORMAT_LEGACY');
    expect(result.confidence).toBeGreaterThanOrEqual(0.6);
  });
});
```

#### Integration Tests: `tests/workflows/split-legacy.test.ts`

**Test 1: Section Mode Grouping**
```typescript
test('groups legacy 5-digit section IDs in section mode', async () => {
  // Creates PDF with:
  // - Page 1: "SECTION 23050" (legacy)
  // - Page 2: "SECTION 23 09 00" (modern)
  
  const result = await splitSet(pdfPath, outputDir, {
    groupBy: 'section',
    format: 'specs'
  });
  
  // Verifies separate files:
  // - 23050.pdf (legacy section as-is)
  // - 23 09 00.pdf (modern section)
});
```

**Test 2: Division Mode Auto-Migration**
```typescript
test('maps legacy 5-digit section IDs to modern division in division mode', async () => {
  // Creates PDF with:
  // - Page 1: "SECTION 23050" (legacy → division 23)
  // - Page 2: "SECTION 26010" (legacy → division 26)
  
  const result = await splitSet(pdfPath, outputDir, {
    groupBy: 'division',
    format: 'specs'
  });
  
  // Verifies modern division files:
  // - 23.pdf (contains page 1, auto-migrated)
  // - 26.pdf (contains page 2, auto-migrated)
});
```

#### Field Naming Tests: `tests/standards/field-naming-validation.test.ts`

```typescript
test('should use sectionIdNormalized for legacy 5-digit section codes', () => {
  const legacyRow: TestSpecsInventoryRow = {
    id: 'spec-legacy-1',
    page: 1,
    sectionId: '23050',
    sectionIdNormalized: '23050', // Same field name as modern
    title: 'Basic Mechanical Materials and Methods',
    division: '23'
  };
  
  expect(legacyRow.sectionIdNormalized).toBe('23050');
  expect(legacyRow).toHaveProperty('sectionIdNormalized');
});
```

---

## Detection Pipeline Details

### Step 1: Format Detection

```typescript
// In extractSpecSectionId()
const modernRegex = /^(\d{2})\s+(\d{2})\s+(\d{2})$/;
const legacyRegex = /^(\d{5})$/;

if (modernRegex.test(input)) {
  return { sectionId: input, format: 'MODERN' };
}

if (legacyRegex.test(input)) {
  return { sectionId: input, format: 'LEGACY' };
}

return null; // Invalid format
```

### Step 2: Standards Lookup

```typescript
// In normalizeSpecsMasterformat()
const legacy = standardsRegistry.resolveLegacy(sectionId);

if (legacy) {
  return {
    sectionId,
    divisionID: legacy.divisionID,      // Modern division ID (e.g., "23")
    division: legacy.sectionTitle,      // Legacy title/division name
    divisionCODE: legacy.divisionCODE,  // Modern code (e.g., "HVAC")
    order: parseInt(legacy.divisionID, 10),
    confidence: 0.8,                    // Lower than modern (1.0)
    basis: 'MASTERFORMAT_LEGACY'
  };
}
```

### Step 3: splitSet Integration

**Section Mode**:
```typescript
// Groups by exact sectionIdNormalized
// Legacy: "23050" → 23050.pdf
// Modern: "23 09 00" → 23 09 00.pdf
const groupKey = sectionInfo.sectionIdNormalized;
```

**Division Mode**:
```typescript
// Groups by modern division (auto-migrated)
// Legacy: "23050" → division "23" → 23.pdf
// Modern: "23 09 00" → division "23" → 23.pdf
const groupKey = sectionInfo.division;
```

---

## Data Flow Example

### Scenario: PDF with Legacy Code "15100"

1. **Page Text Extraction**
   ```
   Page 5: "SECTION 15100 - Basic Mechanical Materials and Methods"
   ```

2. **Detection** (splitSet.ts)
   ```typescript
   const legacyMatches = pageText.match(/\b(?:SECTION\s+)?(\d{5})\b/gi);
   // Result: ["SECTION 15100"]
   
   const candidate = "15100";
   ```

3. **Normalization** (normalizeSpecsMasterformat)
   ```typescript
   const normalized = normalizeSpecsMasterformat({ normalizedId: "15100" });
   // Result: {
   //   sectionId: "15100",
  //   divisionID: "23",
  //   division: "Basic Mechanical Materials and Methods",
   //   divisionCODE: "HVAC",
   //   order: 23,
   //   confidence: 0.8,
   //   basis: 'MASTERFORMAT_LEGACY'
   // }
   ```

4. **Section Info Creation** (splitSet.ts)
   ```typescript
   sectionInfo = {
     page: 5,
     sectionIdNormalized: "15100",  // Preserves legacy format
     sectionTitle: "Basic Mechanical Materials and Methods",
     division: "23"                  // Maps to modern division
   };
   ```

5. **File Grouping**
   - **Section Mode**: Creates `15100.pdf` (legacy code as filename)
   - **Division Mode**: Assigns to `23.pdf` (modern division grouping)

---

## Design Decisions

### 1. Read-Only UI
**Rationale**: Legacy sections are reference data from UDS.xlsx, not user-editable. Users need visibility for troubleshooting but shouldn't modify these mappings.

### 2. Same Field Names
**Decision**: Legacy and modern specs use identical field names (`sectionIdNormalized`, `sectionTitle`, `division`)

**Rationale**:
- Type-safe polymorphism (same interface)
- No conditional logic needed throughout codebase
- Transparent auto-migration in pipelines

### 3. Auto-Migration in Division Mode
**Decision**: Legacy codes automatically map to modern divisions when `groupBy: 'division'`

**Rationale**:
- Users expect division-based grouping to use modern divisions
- File output should be consistent (all `23.pdf`, not mix of `23.pdf` and `15.pdf`)
- Legacy format is internal detail, not exposed in output structure

### 4. Confidence Scoring
**Decision**: Legacy matches have lower confidence (0.7-0.8) than modern matches (1.0)

**Rationale**:
- Reflects higher uncertainty in legacy detection
- Allows downstream code to make quality decisions
- Maintains backward compatibility with existing thresholds

---

## Limitations & Future Work

### Current Limitations

1. **No Legacy Drawings Support**: Only specs have legacy format (pre-2004 MasterFormat). Drawings use UDS designators which haven't changed.

2. **Fixed Legacy Mappings**: Legacy-to-modern mappings are hardcoded from UDS.xlsx. Users cannot customize these (by design).

3. **Single-Page Detection**: splitSet detects one format per page. Mixed modern/legacy on same page defaults to modern.

### Future Enhancements (Not Yet Implemented)

1. **User Aliases for Legacy Codes**: Allow users to define custom aliases for legacy codes (requires Phase 2 GUI integration).

2. **Migration Warnings**: Optionally warn users when legacy codes are auto-migrated to modern divisions.

3. **Legacy Format in Filenames**: Option to preserve legacy format in filenames even in division mode (e.g., `23-legacy.pdf`).

---

## Troubleshooting

### Legacy Code Not Detected

**Symptom**: splitSet doesn't recognize a legacy 5-digit code

**Diagnosis**:
1. Check if modern 6-digit pattern detected first (modern has priority)
2. Verify code exists in standards registry legacy table
3. Check confidence threshold (`>= 0.6` required)

**Solution**:
- Add code to UDS.xlsx legacy table if missing
- Regenerate standards datasets
- Verify with unit test in `specsMasterformat.test.ts`

### Legacy Code Grouped Incorrectly

**Symptom**: Legacy code ends up in wrong division file

**Diagnosis**:
1. Check legacy-to-modern division mapping in settings UI
2. Verify `normalizeSpecsMasterformat()` returns correct `division`
3. Confirm `groupBy` mode (section vs. division)

**Solution**:
- Correct mapping in UDS.xlsx if wrong
- Regenerate standards datasets
- Test with integration test in `split-legacy.test.ts`

---

## References

- **Standards Refactor Proposal**: `STANDARDS_REFACTOR_PROPOSAL.md`
- **Phase 4 Implementation**: Legacy pre-2004 specs support
- **Field Naming Guide**: `FIELD_NAMING_GUIDE.md`
- **Test Coverage**:
  - Unit: `tests/standards/specsMasterformat.test.ts`
  - Integration: `tests/workflows/split-legacy.test.ts`
  - Validation: `tests/standards/field-naming-validation.test.ts`

---

## Questions?

For clarification on legacy support:
1. Check this guide first
2. Review test cases in `tests/workflows/split-legacy.test.ts`
3. Inspect settings UI: Run GUI → Settings → Standards → Legacy Sections table
4. Refer to standards registry: `packages/core/src/standards/registry.ts`
