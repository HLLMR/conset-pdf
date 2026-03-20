# Phase 02 - ID Parsing and Normalization

## Scope

Capture the deterministic parsing and normalization behavior for drawing sheet IDs and spec section IDs.

## Source Evidence

- `packages/core/src/parser/drawingsSheetId.ts`
- `packages/core/src/parser/specsSectionId.ts`
- `packages/core/src/parser/normalize.ts`

## Drawings ID Parsing

### Primary pattern

- `DEFAULT_DRAWINGS_PATTERN`:
  - `\b([A-Z]{1,3}\s*\d{0,2}\s*[-._ ]\s*\d{1,3}(?:\.\d+)?[A-Z]{0,3})\b`

### False-positive suppression

- Explicit penalties for measurement/reference-like prefixes:
  - `OR`, `AND`, `LB`, `KG`, `CFR`, `EPA`, `PSI`, `GPM`, `CFM`, `BTU`, `TON`, `HP`, `KW`, `VOLT`, `AMP`
- Penalty for `MIN`/`MAX` prefixed numerics.

### Confidence mechanics

`calculateEnhancedConfidence()` base score is `0.5` and then adjusted:

- `+0.3` inside title block ROI
- `+0.3` if near anchor keyword (< 50 pt)
- `+0.2` known discipline prefix
- `-0.4` if same normalized ID appears > 3 times on page
- `+0.2` if appears exactly twice in ROI
- `+0.15` bottom-right strong signal (>70% x and y)
- `+0.1` near bottom-right (>60% x and y)
- `-0.2` not in typical title-block region
- `-0.5` if matching false-positive pattern

Final score is clamped to `[0.0, 1.0]`.

## Specs Section Parsing

### Patterns

- Default pattern:
  - `\b(?:SECTION\s+)?(\d{2}\s+\d{2}\s+\d{2})\b`
- Strict section-line pattern:
  - `^SECTION\s+(\d{2}\s+\d{2}\s+\d{2})\b` (case-insensitive)

### Behavioral rules

- Optional strict mode (`strictSection=true`) prefers `SECTION` line anchors first.
- Division 01 suppression can be enabled (`rejectDivision01=true`, default true).
- Multiple distinct matches reduce confidence.

### Confidence mechanics

`calculateConfidence()` starts at `0.5`:

- `+0.2` keyword proximity (`SECTION`, `DIVISION`, `PART`, `ARTICLE`, `SECTION NO`)
- `+0.2` duplicate consistent occurrence of same normalized ID
- `-0.3` if multiple distinct normalized candidates found on page text

## Normalization Rules

### Drawings

`normalizeDrawingsSheetId()`:

- uppercase
- remove spacing around `.` and `-`
- collapse whitespace
- trim
- remove any remaining spaces

Examples:

- `G 0.01` -> `G0.01`
- `m6 - 03a` -> `M6-03A`

### Specs

`normalizeSpecsSectionId()`:

- strip to digits
- if <6 digits, right-pad with `0`
- take first 6 digits and format `DD SS SS`

Examples:

- `230200` -> `23 02 00`
- `23 2` -> `23 20 00`

## Inputs and Outputs

- Inputs:
  - page text and/or positioned text items
  - parsing options (`strictSection`, `rejectDivision01`)
  - optional custom regex
- Outputs:
  - normalized IDs with confidence scores
  - sorted best candidates

## Invariants

- Normalized drawing IDs are uppercase and whitespace-free.
- Normalized spec IDs are always formatted `DD SS SS`.
- Confidence is always in `[0,1]`.

## Failure Modes

- Ambiguous pages with many candidate IDs reduce confidence.
- Division references can be mistaken as section IDs unless strict/filters applied.
- Aggressive padding in spec normalization can produce syntactically valid but semantically wrong IDs from malformed input.
