# Standards Module

Authoritative standards module for drawings discipline identification and specs MasterFormat classification.

## Overview

This module provides:
- **Drawings**: UDS-style discipline designator identification, multi-letter alias handling, heuristic-based disambiguation, and discipline-based sorting
- **Specs**: CSI MasterFormat division identification, section classification, and MasterFormat-based sorting

## Design Principles

- **Pure functions**: No IO, no side effects
- **Additive only**: Does not modify existing structures
- **Optional fields**: Adds `discipline` (drawings) or `specs` (specs) fields to rows, does not break existing JSON shapes
- **Workflow integration**: Integrated into merge workflow analyze inventory mapping only
- **Separate lanes**: Drawings and specs use separate metadata fields and comparators

## Drawings Standards

### Types

- `StandardsBasis`: 'UDS' | 'ALIAS' | 'HEURISTIC' | 'UNKNOWN'
- `DisciplineCanonical4`: 4-letter canonical discipline codes (GENR, SURV, DEMO, CIVL, LAND, ARCH, INTR, STRU, MECH, PLUM, FIRE, ELEC, TECH, CTRL, VEND, SPEC, UNKN)
- `DrawingsDisciplineMeta`: Complete discipline metadata including designator, alias, canonical code, order, confidence, and basis

### Functions

#### `normalizeDrawingsDiscipline(input)`

Normalizes a drawings discipline from `normalizedId` and optional `title`.

Returns `DrawingsDisciplineMeta` with:
- `designator`: UDS-style single letter (e.g., 'M')
- `modifier`: Optional modifier (e.g., 'D' in 'AD')
- `alias`: Raw observed prefix (e.g., 'FP', 'DDC')
- `canonical4`: Canonical 4-letter discipline code
- `displayName`: Human-readable name
- `order`: Sort order (lower = earlier)
- `confidence`: Confidence level (0.0 to 1.0)
- `basis`: Classification basis
- `reason`: Optional explanation

#### `compareDrawingsRows(a, b)`

Comparator for sorting drawings rows:
1. Primary: discipline order
2. Secondary: natural sort by normalizedId
3. Tertiary: source, then page, then row.id

### Datasets

#### `drawingsDesignators.ts`

- UDS single-letter designators (G, C, L, A, I, S, M, P, E, F, T)
- Multi-letter aliases (FP, DDC, ATC, SEC, AV, IT, etc.)
- Keyword hints for disambiguating 'C' (Controls vs Civil)

#### `drawingsOrderHeuristic.ts`

Pragmatic ordering table:
1. Cover/General
2. Survey/Existing/Demo
3. Civil
4. Landscape
5. Architectural (+ Interiors)
6. Structural
7. Mechanical
8. Plumbing
9. Fire Protection
10. Electrical
11. Technology / Low voltage
12. Controls
13. Vendor/deferred
14. Unknown (last)

## Specs Standards

### Types

- `SpecsBasis`: 'MASTERFORMAT' | 'UNKNOWN'
- `SpecsMasterformatMeta`: Complete MasterFormat metadata including section ID, divisionID, division name, order, confidence, and basis

### Functions

#### `normalizeSpecsMasterformat(input)`

Normalizes a specs MasterFormat classification from `normalizedId`.

Returns `SpecsMasterformatMeta` with:
- `sectionId`: Full section ID in format "DD SS SS" (e.g., "23 09 00")
- `divisionID`: Division code (e.g., "23")
- `division`: Full division name from dataset if present
- `order`: Numeric division order (fallback 999)
- `confidence`: Confidence level (0.0 to 1.0)
  - 1.0 for known divisions in dataset
  - 0.7 for unknown divisions (valid format but not in dataset)
  - 0.2 for invalid/missing section IDs
- `basis`: Classification basis ('MASTERFORMAT' or 'UNKNOWN')
- `reason`: Optional explanation (e.g., 'unknown-division', 'no-spec-section-id')

#### `extractSpecSectionId(normalizedId)`

Extracts section ID if it matches MasterFormat pattern `^\d{2}\s\d{2}\s\d{2}$`.

#### `compareSpecsRows(a, b)`

Comparator for sorting specs rows:
1. Primary: division order
2. Secondary: section tuple comparison (division, section, subsection) - numeric element-wise
3. Tertiary: source, then page, then row.id

### Datasets

#### `masterformatDivisions.ts`

CSI MasterFormat 2018 divisions dataset (00-49):
- **00**: Procurement and Contracting Requirements
- **01**: General Requirements
- **02**: Existing Conditions
- **03**: Concrete
- **04**: Masonry
- **05**: Metals
- **06**: Wood, Plastics, and Composites
- **07**: Thermal and Moisture Protection
- **08**: Openings
- **09**: Finishes
- **10**: Specialties
- **11**: Equipment
- **12**: Furnishings
- **13**: Special Construction
- **14**: Conveying Equipment
- **21**: Fire Suppression
- **22**: Plumbing
- **23**: Heating, Ventilating, and Air Conditioning (HVAC)
- **25**: Integrated Automation
- **26**: Electrical
- **27**: Communications
- **28**: Electronic Safety and Security
- **31**: Earthwork
- **32**: Exterior Improvements
- **33**: Utilities
- **34**: Transportation
- **35**: Waterway and Marine Construction
- **40**: Process Integration
- **41**: Material Processing and Handling Equipment
- **42**: Process Heating, Cooling, and Drying Equipment
- **43**: Process Gas and Liquid Handling, Purification and Storage Equipment
- **44**: Pollution and Waste Control Equipment
- **45**: Industry-Specific Manufacturing Equipment
- **46**: Water and Wastewater Equipment
- **48**: Electrical Power Generation
- **49**: Water and Wastewater Treatment Equipment

The dataset can be expanded later as needed. Tests should not assume completeness beyond what is included.

## Integration

Integrated into `workflows/mappers/merge.ts`:
- **Drawings**: Applied only for `docType === 'drawings'`, adds optional `discipline` field to inventory rows
- **Specs**: Applied only for `docType === 'specs'`, adds optional `specs` field to inventory rows
- Does not modify existing behavior or JSON shapes

## Usage Examples

### Drawings

```typescript
import { normalizeDrawingsDiscipline, compareDrawingsRows } from '@conset-pdf/core';

// Normalize discipline
const meta = normalizeDrawingsDiscipline({
  normalizedId: 'M1-01',
  title: 'Mechanical Plan'
});
// Returns: { designator: 'M', canonical4: 'MECH', order: 70, ... }

// Sort rows
rows.sort(compareDrawingsRows);
```

### Specs

```typescript
import { normalizeSpecsMasterformat, compareSpecsRows } from '@conset-pdf/core';

// Normalize MasterFormat
const meta = normalizeSpecsMasterformat({
  normalizedId: '23 09 00'
});
// Returns: { sectionId: '23 09 00', divisionID: '23', division: 'Heating, Ventilating, and Air Conditioning (HVAC)', order: 23, ... }

// Sort rows
rows.sort(compareSpecsRows);
```
