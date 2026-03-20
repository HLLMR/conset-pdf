# Field Naming Conventions Guide

## Overview

This document defines the **context-specific field naming conventions** adopted in Phase 3 of the Standards Refactor. These conventions ensure type safety, code clarity, and proper domain modeling across drawings and specs contexts.

## Core Principles

### 1. Context-Specific Naming
- **Never use generic field names** like `normalizedId` when the context is known
- **Drawings context**: Use `discipline` and `sheet` in all field names
- **Specs context**: Use `division` and `section` in all field names

### 2. Consistency Across Layers
- Field names must match between:
  - Type definitions
  - Runtime data structures
  - Function parameters
  - API payloads
  - UI components

### 3. Semantic Clarity
- Field names should be self-documenting
- Avoid ambiguous terms that apply to multiple contexts
- Use suffixes to distinguish variants (e.g., `Normalized`, `Raw`, `Display`)

---

## Field Naming Tables

### Drawings Context Fields

| Field Name | Type | Description | Example |
|------------|------|-------------|---------|
| `sheetId` | `string \| null` | Raw extracted sheet identifier | `"A-101"`, `"M1-01"` |
| `sheetIdNormalized` | `string \| null` | Canonicalized sheet ID (uppercase, trimmed) | `"A-101"`, `"M1-01"` |
| `sheetTitle` | `string \| null` | Sheet title/description | `"First Floor Plan"` |
| `discipline` | `string \| null` | Full level 1 discipline name | `"General"`, `"Mechanical"`, `"Electrical"` |
| `disciplineID` | `string` | Single-letter discipline identifier (UDS standard) | `"G"`, `"A"`, `"M"` |
| `disciplineEid` | `string` | Full 2-character level 1 + level 2 UDS identifier | `"AD"`, `"MP"`, `"EL"` |
| `disciplineCODE` | `string` | 4-character discipline code | `"GENL"`, `"ARCH"`, `"MECH"` |
| `disciplineFull` | `string` | Full discipline name with level 1 & 2 | `"Architectural Demolition"` |
| `disciplineMeta` | `object` | Complete discipline metadata object | `{ disciplineID: "A", ... }` |

**Example Usage:**
```typescript
interface DrawingsInventoryRow {
  pageIndex: number;
  sheetId: string | null;
  sheetIdNormalized: string | null;
  sheetTitle: string | null;
  disciplineID: string | null;
  disciplineEid: string | null;
  discipline: string | null;
}

// In splitSet.ts
interface PageInfo {
  page: number;
  sheetIdNormalized: string | null;
  sheetTitle: string | null;
}
```

---

### Specs Context Fields

| Field Name | Type | Description | Example |
|------------|------|-------------|---------|
| `sectionId` | `string \| null` | Raw extracted section identifier | `"23 09 00"`, `"23050"` (legacy) |
| `sectionIdNormalized` | `string \| null` | Canonicalized section ID | `"23 09 00"`, `"23050"` |
| `sectionTitle` | `string \| null` | Section title/description | `"HVAC Controls"` |
| `division` | `string \| null` | Full division name | `"Heating, Ventilating, and Air Conditioning (HVAC)"` |
| `divisionID` | `string` | 2-digit division identifier (MasterFormat 2018) | `"22"`, `"23"`, `"25"` |
| `divisionCODE` | `string` | 4-character division code | `"PLUM"`, `"HVAC"`, `"ELEC"` |
| `divisionMeta` | `object` | Complete division metadata object | `{ divisionID: "23", ... }` |

**Example Usage:**
```typescript
interface SpecsInventoryRow {
  pageIndex: number;
  sectionId: string | null;
  sectionIdNormalized: string | null;
  sectionTitle: string | null;
  divisionID: string | null;
  division: string | null;
}

// In splitSet.ts
interface SectionInfo {
  page: number;
  sectionIdNormalized: string | null;
  sectionTitle: string | null;
  divisionID: string | null;
  division: string | null;
}
```

---

## Legacy Support Field Naming

### Pre-2004 Specs (5-Digit Section Codes)

Legacy specs use the **same field names** as modern specs:

```typescript
// Legacy section row (5-digit format)
const legacyRow: SpecsInventoryRow = {
  pageIndex: 0,
  sectionId: '23050',              // Legacy 5-digit format
  sectionIdNormalized: '23050',    // Same field name as modern
  sectionTitle: 'Basic Mechanical Materials and Methods',
  divisionID: '23',
  division: 'Heating, Ventilating, and Air Conditioning (HVAC)'
};

// Modern section row (6-digit format)
const modernRow: SpecsInventoryRow = {
  pageIndex: 0,
  sectionId: '23 09 00',           // Modern 6-digit format
  sectionIdNormalized: '23 09 00', // Same field name as legacy
  sectionTitle: 'HVAC Controls',
  divisionID: '23',
  division: 'Heating, Ventilating, and Air Conditioning (HVAC)'
};
```

**Rationale**: Using the same field name (`sectionIdNormalized`) for both formats enables:
1. Type-safe polymorphism (same interface for both)
2. Transparent auto-migration in processing pipelines  
3. No conditional logic based on format throughout the codebase

---

## Migration Patterns

### From Generic to Context-Specific

#### ❌ Before (Generic Pattern)
```typescript
// Ambiguous - applies to both drawings and specs
interface InventoryRow {
  normalizedId: string | null;  // What kind of ID?
  displayName: string | null;   // Display name of what?
}

// splitSet.ts
interface PageInfo {
  normalizedId: string | null;  // Drawing sheet ID or specs section ID?
}
```

#### ✅ After (Context-Specific Pattern)
```typescript
// Drawings inventory
interface DrawingsInventoryRow {
  sheetIdNormalized: string | null;  // Clear: it's a sheet ID
  sheetTitle: string | null;         // Clear: it's a sheet title
}

// Specs inventory
interface SpecsInventoryRow {
  sectionIdNormalized: string | null;  // Clear: it's a section ID
  sectionTitle: string | null;         // Clear: it's a section title
}

// splitSet.ts - Drawings
interface PageInfo {
  sheetIdNormalized: string | null;  // Clear context
}

// splitSet.ts - Specs
interface SectionInfo {
  sectionIdNormalized: string | null;  // Clear context
}
```

---

## Validation Rules

### Required Conventions

1. **No `normalizedId` in context-specific code**
   - ✅ Use `sheetIdNormalized` in drawings code
   - ✅ Use `sectionIdNormalized` in specs code
   - ❌ Never use `normalizedId` when context is known

2. **Consistent Raw vs. Normalized Naming**
   - Raw fields: `sheetId`, `sectionId` (original extracted value)
   - Normalized fields: `sheetIdNormalized`, `sectionIdNormalized` (canonicalized)

3. **Title Fields Match Context**
   - Drawings: `sheetTitle` (not `title`, not `sectionTitle`)
   - Specs: `sectionTitle` (not `title`, not `sheetTitle`)

4. **Metadata Objects Use Full Context**
   - Drawings: `disciplineMeta` (not `meta`, not `divisionMeta`)
   - Specs: `divisionMeta` (not `meta`, not `disciplineMeta`)

### Testing Validation

See `tests/standards/field-naming-validation.test.ts` for comprehensive validation tests that ensure:
- Context-specific fields are used correctly
- No generic `normalizedId` exists in typed structures
- bookmarks tree building uses correct field names
- Legacy codes use same field names as modern codes

---

## Implementation Status

### ✅ Complete (Phase 3)
- `packages/core/src/bookmarks/treeBuilder.ts` - Updated `buildTreeFromInventory` to use `sheetIdNormalized`/`sectionIdNormalized`
- `packages/core/src/core/splitSet.ts` - Updated `PageInfo` to use `sheetIdNormalized`, `SectionInfo` to use `sectionIdNormalized`

### ✅ Complete (Phase 4)
- Legacy support integrated with same field naming (`sectionIdNormalized` for both modern and legacy)
- Standards registry methods return context-specific metadata objects

---

## Future Extensions

### Potential Additions (Not Yet Implemented)

If the full StandardsRegistry system (Phases 1-2) is implemented:

1. **Alias Fields**
```typescript
interface DisciplineEntry {
  userAlias?: string;     // User-defined alternative for disciplineID
  userOverride?: boolean; // User modified this entry
}
```

2. **Extended Metadata**
```typescript
interface DisciplineEntry {
  disciplineEid: string;   // Extended 2-char: "AD", "FA", "EP"
  disciplineDesc?: string; // Keywords for fuzzy matching
  order: number;           // Permanent sort order
}
```

3. **Filename Format Variables**
```typescript
// For file renaming in splitSet
// Drawings
{disciplineID, disciplineCODE, discipline, sheetId, sheetTitle}

// Specs  
{divisionID, divisionCODE, division, sectionId, sectionTitle}
```

---

## References

- **Standards Refactor Proposal**: `STANDARDS_REFACTOR_PROPOSAL.md`
- **Phase 3 Implementation**: Strict cleanup of generic `normalizedId` usage
- **Phase 4 Implementation**: Legacy pre-2004 specs support
- **Test Coverage**: `tests/standards/field-naming-validation.test.ts`

---

## Questions?

For clarification on field naming conventions:
1. Check this guide first
2. Review `tests/standards/field-naming-validation.test.ts` for examples
3. Refer to type definitions in `packages/core/src/bookmarks/treeBuilder.ts` and `packages/core/src/core/splitSet.ts`
