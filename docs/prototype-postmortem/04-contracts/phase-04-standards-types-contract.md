# Phase 04 - Standards Types Contract

## Scope

Step 28 captures standards metadata contracts for drawings/specs normalization and context-specific naming conventions.

Primary sources:

- `packages/core/src/standards/types.ts`
- `docs/FIELD_NAMING_GUIDE.md`

## Canonical Type Families

### New canonical dataset types

From `packages/core/src/standards/types.ts`:

- `DisciplineEntry` (`:17`)
- `DivisionEntry` (`:44`)
- `LegacySectionEntry` (`:69`)
- `DisciplineAlias` (`:89`)
- `UserCustomizations`

These map directly to UDS/MasterFormat-style dataset payloads and user override structure.

### Legacy compatibility types retained

- `StandardsBasis` (`:128`)
- `SpecsBasis` (`:130`)
- `DisciplineCanonical4`
- `DrawingsDisciplineMeta` (`:153`)
- `SpecsMasterformatMeta` (`:180`)

These remain active compatibility contracts in normalization flows.

## Field Naming Contract

Per `docs/FIELD_NAMING_GUIDE.md`:

- drawings context uses `sheet*` and `discipline*`
- specs context uses `section*` and `division*`
- avoid generic `normalizedId` in context-specific contracts

Examples:

- drawings: `sheetId`, `sheetIdNormalized`, `sheetTitle`, `disciplineID`
- specs: `sectionId`, `sectionIdNormalized`, `sectionTitle`, `divisionID`

Legacy 5-digit specs and modern 6-digit specs share the same section field names for polymorphic handling.

## Confidence/Basis Contract

`DrawingsDisciplineMeta` and `SpecsMasterformatMeta` include:

- confidence numeric score (`0..1` by convention)
- basis enum (`StandardsBasis` or `SpecsBasis`)
- deterministic sort order field (`order`)

Sort order is semantically meaningful (grouping, naming, and output ordering), not just presentation.

## User Customization Contract

`UserCustomizations` supports:

- discipline/division overrides
- additions
- deletions
- discipline alias extensions

This contract is a migration boundary for user-managed standards mappings and must remain backward-compatible or explicitly versioned.

## Rust Mapping Notes

- Preserve dataset field names exactly where they intentionally mirror source schema.
- Preserve basis enums and confidence fields for explainability and auditability.
- Preserve `order` semantics as behavior-critical sorting metadata.
- Keep legacy type support until full deprecation plan is executed.

## Evidence

- `packages/core/src/standards/types.ts`
- `packages/core/src/standards/index.ts`
- `docs/FIELD_NAMING_GUIDE.md`
- `docs/STANDARDS.md`
