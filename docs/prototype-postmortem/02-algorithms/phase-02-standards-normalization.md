# Phase 02 - Standards Normalization

## Scope

Document drawings discipline and specs MasterFormat normalization contracts.

## Source Evidence

- `packages/core/src/standards/normalizeDrawingsDiscipline.ts`
- `packages/core/src/standards/normalizeSpecsMasterformat.ts`
- `packages/core/src/standards/registry.ts`
- `packages/core/src/standards/datasets/*`

## Drawings Discipline Normalization

### Prefix extraction

`extractDrawingsPrefix()`:

- extracts leading alpha token from normalized drawing IDs
- returns `null` for spec-like IDs (`DD SS SS`)

### Resolution order

`normalizeDrawingsDiscipline()` resolves in this order:

1. alias lookup (includes multi-letter aliases)
2. single-letter UDS discipline lookup
3. two-letter modifier interpretation (base discipline + modifier)
4. unknown fallback

### Alias map (default examples)

- `FP`, `FA`
- `DDC`, `ATC`
- `SEC`, `AV`, `IT`

### Ambiguous `C` handling

Heuristic split using title keywords:

- controls keywords -> controls code path (MTEC)
- civil keywords -> civil code path (CIVL)
- no signal -> default civil with reduced confidence

## Specs MasterFormat Normalization

### Accepted formats

- modern: `DD SS SS`
- legacy: `DDDDD`

### Resolution behavior

- modern: division = first two digits, lookup in registry
- legacy: lookup legacy section entry; attempt mapping to modern division
- unknowns still emit structured meta with reduced confidence

## Canonical Sort Intent

- Drawings sorting order derives from registry discipline order values.
- Specs sorting order derives from numeric division order.

## Inputs and Outputs

- Inputs:
  - normalized ID and optional title (drawings)
  - normalized ID (specs)
- Outputs:
  - metadata objects with discipline/division IDs, names, order, confidence, basis

## Invariants

- normalization functions are pure (no IO side effects)
- output always includes explicit `basis` and `confidence`
- unknown classifications return stable sentinel values/order

## Failure Modes

- alias and heuristic quality depends on registry completeness.
- legacy 5-digit mappings are partial and can degrade to lower-confidence fallbacks.
- title-less `C` prefixed sheets bias to civil default.
