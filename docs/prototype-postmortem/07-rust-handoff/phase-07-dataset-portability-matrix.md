# Phase 07 - Dataset Portability Matrix

**Document Type**: Rust Handoff — Dataset and Regex Portability  
**Date**: 2026-03-19

---

## Purpose

Classify every dataset, regex pattern, and static schema in the codebase as one of:

- **Port verbatim**: No algorithmic re-expression needed. Copy the data as-is into Rust (static arrays, `phf` compile-time maps, or embedded JSON).
- **Port with adaptation**: The data is correct, but the Rust representation requires format changes (type adjustments, serde annotations, etc.).
- **Algorithmic re-expression required**: The logic cannot be expressed as static data; it must be re-implemented as a Rust function.
- **Do not port**: Deprecated, test-only, superseded, or out-of-scope for V4.

---

## Portability Classification Key

| Symbol | Meaning |
|---|---|
| V | Port verbatim |
| A | Port with adaptation |
| R | Algorithmic re-expression required |
| X | Do not port |

---

## Standards Datasets

### `standards/datasets/drawingsDesignators.ts`

**Content**: UDS single-letter designator table, multi-letter alias map, disambiguation keyword lists.

| Field | Classification | Notes |
|---|---|---|
| `UDS_DESIGNATORS` record (12 entries) | V | Port as `const` array or `phf::Map`. Keys are `&str`, values are `{ canonical4, display_name }`. No changes to key–value mappings. |
| `ALIAS_MAPPINGS` array (10 entries) | V | Port as `const` slice. All fields: alias, canonical4, displayName, designator, confidence, basis. |
| `CONTROLS_KEYWORDS` array (8 entries) | V | Port as `const` slice of `&str`. |
| `CIVIL_KEYWORDS` array (varies) | V | Port as `const` slice of `&str`. |

**Recommendation**: Use `phf` crate (`phf::Map`) for O(1) compile-time lookup of `UDS_DESIGNATORS` and `ALIAS_MAPPINGS` by key. This eliminates the runtime `HashMap` and satisfies the determinism requirement (no `HashMap` iteration order).

```rust
// In crates/standards-data
use phf::phf_map;

pub static UDS_DESIGNATORS: phf::Map<&'static str, DisciplineEntry> = phf_map! {
    "G" => DisciplineEntry { canonical4: "GENR", display_name: "General" },
    "C" => DisciplineEntry { canonical4: "CIVL", display_name: "Civil" },
    // ... (12 entries total)
};
```

**Scope note**: Any change to designator canonical codes or sort orders must be treated as a breaking change — it affects output file names, which downstream teams use for routing and filing.

---

### `standards/datasets/disciplines.generated.ts`

**Content**: Generated discipline table (canonical 4-letter codes, display names, sort order integers).

| Classification | V |
|---|---|
| Notes | Port as `const` slice ordered by `sort_order`. Sort order integers are the canonical output ordering and must not change between versions without a migration notice. |

---

### `standards/datasets/divisions.generated.ts`

**Content**: Generated CSI MasterFormat division table. Maps division numbers (01–50) to division names and sort order.

| Classification | V |
|---|---|
| Notes | Port as `const` slice. All 50 entries. Sort order determines specs output file naming. Must be byte-identical with prototype behavior. |

---

### `standards/datasets/masterformatDivisions.ts`

**Content**: Primary MasterFormat division definitions used by `normalizeSpecsMasterformat.ts`. Includes division prefixes and canonical names for modern (6-digit) format.

| Classification | V |
|---|---|
| Notes | Port verbatim. This is the authoritative CSI MasterFormat division table. Any V4 additions (new divisions introduced in MasterFormat updates) must go through a schema versioning process. |

---

### `standards/datasets/drawingsOrderHeuristic.ts`

**Content**: Canonical discipline sort order used to order output files for split and merge operations.

| Classification | V |
|---|---|
| Notes | Port verbatim. This table was hand-tuned for AEC project filing conventions. It is domain knowledge, not a derivable computation. Changes affect all existing file naming workflows. |

---

### `standards/datasets/legacySections.generated.ts`

**Content**: Legacy 5-digit MasterFormat section IDs mapped to their modern 6-digit equivalents.

| Classification | V |
|---|---|
| Notes | Port verbatim. Used by `normalizeSpecsMasterformat.ts` legacy path. The mapping is domain-fixed (it reflects the 2004 MasterFormat renumbering). No algorithm; just data. |

---

## Regex Patterns

### Drawing Sheet ID Patterns (`parser/drawingsSheetId.ts`)

| Pattern | Classification | Notes |
|---|---|---|
| `DEFAULT_DRAWINGS_PATTERN` (`/\b([A-Z]{1,3}\s*\d{0,2}\s*[-._ ]\s*\d{1,3}(?:\.\d+)?[A-Z]{0,3})\b/g`) | V | Port verbatim to Rust `regex` crate. Compile once using `lazy_static!` or `once_cell::sync::Lazy`. Verify match groups are equivalent (Rust `regex` uses named captures with `(?P<name>...)` syntax if needed). |
| `ANCHOR_KEYWORDS` array (`['SHEET', 'SHT', 'DWG NO', ...]`) | V | Port as `const` slice. Used for score-boost when anchor keyword precedes sheet ID text. |
| `DISCIPLINE_PREFIXES` set | V | Port as `phf::Set`. Used for prefix validation during ID parsing. |
| `FALSE_POSITIVE_PATTERNS` array | V | Port verbatim. These are literal regex strings; compile with Rust `regex` crate at startup. Test all patterns against the same positive/negative examples as TS tests. |
| `isConstructionDrawingSize()` logic | R | Pure function that checks page dimensions against ANSI/ARCH size tables. Port as `fn is_construction_drawing_size(width_pts: f32, height_pts: f32) -> bool`. The ANSI/ARCH size constants (as `(width_inches, height_inches)` pairs with 0.5in tolerance) are verbatim data; the size-comparison logic requires re-expression as Rust. |

**Rust regex compilation note**: The Rust `regex` crate uses a slightly different syntax from JavaScript regex in edge cases. Verify the `DEFAULT_DRAWINGS_PATTERN` produces equivalent matches by running the existing TS test cases through the Rust version. Known differences to watch:
- JS `/g` flag (global) = Rust `find_iter()`
- JS `\b` word boundary = Rust `\b` (same semantics, but Rust `regex` uses bytes mode by default; use `RegexBuilder::unicode(true)` if needed)

---

### Spec Section ID Patterns (`parser/specsSectionId.ts`)

| Pattern | Classification | Notes |
|---|---|---|
| Modern 6-digit pattern (`DD SS SS` format, e.g., `23 82 16`) | V | Port verbatim. |
| Legacy 5-digit pattern (e.g., `15600`) | V | Port verbatim. |
| Normalization rules (uppercase, whitespace collapse) | R | Pure string transformation functions; port as Rust functions using `str::to_uppercase()` and `split_whitespace().collect::<Vec<_>>().join(" ")`. |

---

### Narrative Parser Patterns (`narrative/parse-algorithmic.ts`)

| Pattern | Classification | Notes |
|---|---|---|
| Sheet reference extraction regexes | V | Port verbatim to Rust `regex` crate. |
| Section reference extraction regexes | V | Port verbatim. |
| Near-match scoring function | R | Algorithmic: edit distance + partial match scoring. Port as a Rust function. |

---

### Chrome/Footer Detection Patterns

#### `specs/footerSectionIdParser.ts`

| Pattern | Classification | Notes |
|---|---|---|
| Division 23 footer ID pattern | V | Port verbatim. |
| Extension to all 50 divisions (R-003 — not yet implemented) | R | New content required in V4. The existing Division-23 pattern is a subset of what's needed. V4 must implement full coverage using the MasterFormat table from `standards-data`. |

---

## JSON Schemas

### Layout Profile Schema (`layouts/layout-template.json`, `layout/types.ts`)

| Schema Element | Classification | Notes |
|---|---|---|
| `NormalizedROI` fields (x, y, width, height, all in [0,1]) | V | Port exactly. Use `serde(rename_all = "camelCase")` to match the JSON field names. |
| `LayoutProfile` top-level fields (name, version, description) | V | Port exactly. |
| `page` sub-object (orientation, roiSpace, min/maxWidth/Height) | V | Port exactly. |
| `sheetId.rois` array | V | Port exactly. |
| `sheetId.anchorKeywords` array | V | Port exactly. |
| `sheetTitle` sub-object | V | Port exactly. |
| `validation.allowedPrefixes` array | V | Port exactly. |
| `source` enum ('auto-detected' \| 'manual' \| 'user-defined') | A | Port as Rust enum with `serde(rename_all = "kebab-case")`. |
| Coordinate system note | — | The `roiSpace: "visual" \| "pdf"` field indicates whether ROIs use top-left or bottom-left origin. "visual" means top-left (matches LayoutTranscript convention). "pdf" means PDFium native bottom-left. V4 should normalize all profiles to "visual" on load. |

**Profile forward-compatibility rule**: Any V4 change to the `LayoutProfile` schema that removes or renames a field must be accompanied by a migration path for existing user profile JSON files. Users have existing `.json` profiles they have hand-tuned; breaking them silently is a critical UX failure.

---

### `LayoutTranscript` / `LayoutPage` / `LayoutSpan`

| Schema Element | Classification | Notes |
|---|---|---|
| All `LayoutTranscript` fields | V | Port exactly into `crates/contracts::LayoutTranscript`. Required fields become non-Option Rust fields; optional fields become `Option<T>`. |
| `LayoutSpan.bbox` as `[f32; 4]` (not an object) | A | TypeScript uses tuple `[x0, y0, x1, y1]`. Rust: use `#[serde(deserialize_with = "...")]` or a newtype `BBox([f32; 4])`. The wire format must remain `[x0, y0, x1, y1]`. |
| `LayoutSpan.flags` object | A | Port as a struct `SpanFlags { is_bold: Option<bool>, is_italic: Option<bool>, is_fixed_pitch: Option<bool> }` with `#[serde(rename_all = "camelCase")]`. |
| `extractionDate` exclusion from hash | R | This is behavioral, not data: the serializer excludes `extractionDate` from the hash computation. Port as a documented convention in the `calculate_content_hash()` function. |

---

### `IpcResponse<T>` Envelope

| Schema Element | Classification | Notes |
|---|---|---|
| `success: bool` | V | Port exactly. |
| `data?: T` | V | Port as `Option<T>`. |
| `error?: { message, code?, stack?, context? }` | A | Port as `Option<IpcError>` where `IpcError = { message: String, code: Option<String>, stack: Option<String>, context: Option<serde_json::Value> }`. See R-012: documentation describes plain string but code uses structured shape; use the code shape. |

---

### `InventoryResult`, `CorrectionOverlay`, `ExecuteResult`

| Schema Element | Classification | Notes |
|---|---|---|
| `WorkflowId` enum | V | Port as Rust enum. |
| `Severity` enum | V | Port as Rust enum. |
| `RowStatus` enum | V | Port as Rust enum. |
| `Confidence` (numeric 0..1) | A | TypeScript: plain `number`. Rust: consider `struct Confidence(f32)` newtype for type safety; validate that value is in `[0.0, 1.0]` at construction. Wire format remains a float. |
| `InventoryResult.summary` index signature for workflow-specific fields | A | Port base summary fields as a struct; workflow-specific additions as `extra: HashMap<String, serde_json::Value>`. On serialization, flatten the `extra` fields into the JSON object (use `#[serde(flatten)]`). |
| `CorrectionOverlay.patches` / `patchPath` | V | Port exactly. |
| `ExecuteResult.outputs: Record<string, string>` | A | Port as `HashMap<String, String>` in Rust (field ordering is not significant for outputs record). |

---

### `MergePlan`

| Schema Element | Classification | Notes |
|---|---|---|
| `MergePlan` envelope | V | Port exactly. Must be fully serializable to JSON for cross-process handoff (the architecture intent described in ADR-004). |
| `MergeAction` enum variants | V | Port as a Rust enum with `serde` tagging. |

---

### `BookmarkTree` / `BookmarkNode`

| Schema Element | Classification | Notes |
|---|---|---|
| `BookmarkNode` fields (title, destination, children) | V | Port exactly. |
| `BookmarkDestination` (page index + optional position) | V | Port as struct. |
| `BookmarkAnchorTree` | V | Port exactly. Cross-workflow contract between specs-patch and bookmarks. Must be stable. |

---

## Static Assets

### CSS Template for Spec Regeneration (`specs/render/htmlGenerator.ts`)

| Asset | Classification | Notes |
|---|---|---|
| HTML/CSS spec template | A | Port the template structure. Store as an embedded `&'static str` or as a Tauri resource. The CSS should be maintained as a versioned file, not inline code. See NN-19. |

---

## Summary Table

| Dataset / Pattern | Classification | Rust Target | Priority |
|---|---|---|---|
| `UDS_DESIGNATORS` table | V | `standards-data::drawings_designators` using `phf::Map` | P0 |
| `ALIAS_MAPPINGS` array | V | `standards-data::drawings_designators` | P0 |
| `CONTROLS_KEYWORDS` / `CIVIL_KEYWORDS` | V | `standards-data::drawings_designators` | P0 |
| `disciplines.generated` table | V | `standards-data::disciplines` | P0 |
| `divisions.generated` table | V | `standards-data::divisions` | P0 |
| `masterformatDivisions` table | V | `standards-data::masterformat_divisions` | P0 |
| `drawingsOrderHeuristic` sort order | V | `standards-data::drawings_order_heuristic` | P0 |
| `legacySections.generated` mapping | V | `standards-data::legacy_sections` | P0 |
| `DEFAULT_DRAWINGS_PATTERN` regex | V | `core-engine::parser::drawings_sheet_id` | P0 |
| `ANCHOR_KEYWORDS` array | V | `core-engine::parser::drawings_sheet_id` | P0 |
| `DISCIPLINE_PREFIXES` set | V | `core-engine::parser::drawings_sheet_id` using `phf::Set` | P0 |
| `FALSE_POSITIVE_PATTERNS` regexes | V | `core-engine::parser::drawings_sheet_id` | P0 |
| Spec section ID patterns (modern + legacy) | V | `core-engine::parser::specs_section_id` | P0 |
| Narrative parser regexes | V | `core-engine::narrative::parse_algorithmic` | P1 |
| Division-23 footer ID pattern | V | `core-engine::specs::footer_section_id_parser` | P1 |
| All-divisions footer ID pattern | R | `core-engine::specs::footer_section_id_parser` (new) | P1 |
| `LayoutProfile` JSON schema | V | `crates/contracts::layout` | P0 |
| `LayoutTranscript` shape | V→A | `crates/contracts::transcript` | P0 |
| `IpcResponse<T>` shape | V→A | `crates/contracts::ipc` | P1 |
| `InventoryResult` + children | V→A | `crates/contracts::workflows` | P0 |
| `MergePlan` + `MergeAction` | V | `crates/contracts::workflows` | P0 |
| `BookmarkTree` + nodes | V | `crates/contracts::bookmarks` | P0 |
| `BookmarkAnchorTree` | V | `crates/contracts::specs` | P0 |
| `isConstructionDrawingSize()` logic | R | `core-engine::parser::drawings_sheet_id` | P1 |
| Near-match scoring function | R | `core-engine::narrative::parse_algorithmic` | P2 |
| Spec HTML/CSS template | A | `core-engine::specs::render` embedded resource | P1 |

---

## Critical Portability Constraints

1. **Standards data is canonical**: Any change to the sort order integers in `disciplines.generated.ts` or `masterformatDivisions.ts` is a breaking change for all users who have generated output files with the current naming scheme. Version the data and provide a migration notice.

2. **Regex patterns must pass all existing test cases**: Before shipping V4, run the full prototype test suite (especially `tests/standards/`, `tests/narrative/`) under the Rust implementation by using the same test inputs and asserting identical outputs.

3. **Profile schema backward compatibility**: Existing user-defined `.json` profiles must load correctly in V4. If a field is added, make it `Option` with a sensible default. Never remove or rename a field without a migration handler.

4. **Floating-point precision in coordinates**: The `LayoutSpan.bbox` values from PyMuPDF and PDFium will differ slightly (different internal geometry engines). For the standards/narrative/parser data, which are pure text operations, this is irrelevant. For the locator/specs-extraction data, accept up to 0.5pt coordinate variance between prototype and V4 as acceptable in first-pass testing.
