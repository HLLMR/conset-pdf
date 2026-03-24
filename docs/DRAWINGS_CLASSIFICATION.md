# Drawing Discipline Classification Reference

> **Parent document:** [AEC_STANDARDS.md](./AEC_STANDARDS.md) — see §Drawings Standards for the overview table, alias table, and disambiguation algorithm.
> **Purpose:** Algorithm and reference data for classifying AEC drawings by discipline from sheet numbers and titles.
> **Status:** Canonical algorithm ready for implementation in `crates/standards-data` / `crates/engine`.

---

## Overview

Drawing sheets in AEC construction document sets have sheet numbers in the form `X-NNN` or `XX-NNN`, where the leading letter(s) identify the discipline. For example:

- `A-101` → Architectural floor plan
- `S-001` → Structural general notes
- `M-201` → Mechanical equipment plan
- `FP-101` → Fire Protection (non-UDS alias)
- `C-001` → Civil site plan **or** Controls drawing (ambiguous — requires disambiguation)

The classification algorithm takes a **sheet number prefix** and optional **sheet title** as input and produces a **canonical4 discipline code** plus a **confidence score**.

---

## Step 1 — Extract the Designator

Strip trailing digits and separators from the sheet number to isolate the designator:

```
"A-101"   → "A"
"FP-101"  → "FP"
"M-201"   → "M"
"DDC-01"  → "DDC"
"A101"    → "A"   (no separator variant)
```

The designator is everything before the first digit. Normalize to uppercase.

---

## Step 2 — Single-Letter UDS Lookup

If the designator is a single letter, look it up in the UDS designator table. This covers the standard AIA/UDS naming convention.

| Letter | Canonical4 | Display Name     | Notes                                   |
|--------|------------|------------------|-----------------------------------------|
| `G`    | `GENR`     | General          |                                         |
| `V`    | `SURV`     | Survey / Mapping | V is Survey/Mapping in UDS (not Vertical Transportation) |
| `C`    | `CIVL`     | Civil            | **Ambiguous** — may be Controls; see Step 4 |
| `D`    | `PROC`     | Process          | Non-industrial projects often use D for Demolition (`DEMO`); see note |
| `L`    | `LAND`     | Landscape        |                                         |
| `A`    | `ARCH`     | Architectural    |                                         |
| `I`    | `INTR`     | Interiors        |                                         |
| `S`    | `STRU`     | Structural       |                                         |
| `M`    | `MECH`     | Mechanical       |                                         |
| `P`    | `PLUM`     | Plumbing         |                                         |
| `E`    | `ELEC`     | Electrical       |                                         |
| `F`    | `FIRP`     | Fire Protection  |                                         |
| `T`    | `TECH`     | Technology       | Maps to TECH at app level, TELE in UDS EID table |

> **Note on T:** `T` historically appears as "Technology" or "Telecommunications" on drawings. The app-level canonical code is `TECH`. In the UDS full discipline table, `T_` maps to `TELE`. For classification purposes use `TECH`.

---

## Step 3 — Multi-Letter Alias Lookup

If the designator is two or more letters, check the alias table before falling back to unknown.

| Alias  | Canonical4 | Display Name               | UDS Letter | Confidence | Basis   |
|--------|------------|----------------------------|------------|------------|---------|
| `FP`   | `FIRP`     | Fire Protection            | `F`        | 0.95       | ALIAS   |
| `FA`   | `FIRA`     | Fire Alarm                 | `F`        | 0.95       | ALIAS   |
| `DDC`  | `CTRL`     | Direct Digital Controls    | —          | 0.95       | ALIAS   |
| `ATC`  | `CTRL`     | Automatic Temperature Control | —       | 0.95       | ALIAS   |
| `SEC`  | `TECH`     | Security                   | —          | 0.90       | ALIAS   |
| `AV`   | `TECH`     | Audio/Video                | —          | 0.90       | ALIAS   |
| `IT`   | `TECH`     | Information Technology     | —          | 0.90       | ALIAS   |
| `SV`   | `SURV`     | Survey                     | —          | 0.85       | ALIAS   |
| `DM`   | `DEMO`     | Demolition                 | —          | 0.85       | ALIAS   |
| `EX`   | `UNKN`     | Existing                   | —          | 0.80       | ALIAS   |

> **`EX` note:** "EX" maps to `UNKN` with 0.80 confidence because "Existing" is a modifier, not a full discipline. Further context from sheet title is needed to classify these.

The `confidence` field reflects how reliably the alias predicts the discipline. A confidence < 0.85 should trigger a secondary check against the sheet title keywords.

---

## Step 4 — Disambiguation: `C` (Civil vs. Controls)

The single-letter `C` is the most common ambiguity in practice. It defaults to `CIVL` (Civil) per UDS, but many firms use `C` for Controls (DDC/BAS) drawings.

### Controls Keywords (high-signal words in title → classify as `CTRL`)
```
CONTROL, CONTROLS, DDC, BAS, ATC, AUTOMATION, BMS,
SEQUENCE, POINT, DIAGRAM, TEMPERATURE CONTROL, HVAC CONTROLS
```

### Civil Keywords (confirm Civil classification)
```
CIVIL, SITE, EARTHWORK, GRADING, DRAINAGE, PAVING,
UTILITY, STORM, SANITARY, ROAD, EROSION, SURVEY
```

### Algorithm
```
1. Designator is "C"
2. Normalize sheet title to uppercase
3. If title contains any CONTROLS_KEYWORD → return CTRL (confidence 0.85)
4. If title contains any CIVIL_KEYWORD → return CIVL (confidence 0.85)
5. No title match → return CIVL (confidence 0.72, default)
```

The default fallback to `CIVL` reflects that Civil is statistically more common than Controls for `C`-prefix drawings in a typical building project drawing set.

---

## Step 5 — Discipline Sort Order

Once classified, drawings are sorted within a discipline grouping using the **heuristic order table**. This is explicitly a pragmatic ordering based on typical construction document organization, not a normative standard.

| Canonical4 | Sort Order | Notes                                        |
|------------|-----------|----------------------------------------------|
| `GENR`     | 10        | Cover/General first                          |
| `SURV`     | 20        | Survey/Existing                              |
| `DEMO`     | 25        | Demolition (cross-discipline)                |
| `CIVL`     | 30        | Civil                                        |
| `LAND`     | 40        | Landscape                                    |
| `ARCH`     | 50        | Architectural                                |
| `INTR`     | 55        | Interiors (if separate from Arch)            |
| `STRU`     | 60        | Structural                                   |
| `MECH`     | 70        | Mechanical                                   |
| `FIRP`     | 75        | Fire Protection (sprinkler/plumbing-adjacent)|
| `PLUM`     | 80        | Plumbing                                     |
| `ELEC`     | 90        | Electrical                                   |
| `FIRA`     | 95        | Fire Alarm (electrical-adjacent)             |
| `FIRE`     | 100       | Legacy Fire fallback (back-compat only)      |
| `TECH`     | 110       | Technology / Low voltage                     |
| `CTRL`     | 120       | Controls / BAS                               |
| `VEND`     | 130       | Vendor/Deferred submittals                   |
| `SPEC`     | 140       | Specification-only sheets                    |
| `UNKN`     | 999       | Unknown — always last                        |

For any `Canonical4` not in this table, use order value `999`.

---

## Complete Classification Flow

```
input: (sheet_number: &str, sheet_title: Option<&str>)

1. Extract designator from sheet_number (uppercase, strip digits/separators)
2. If len == 1:
   a. Look up in UDS_DESIGNATORS
   b. If found and designator == "C": run Civil/Controls disambiguation with title
   c. Return canonical4
3. If len >= 2:
   a. Check ALIAS_MAPPINGS (exact match, uppercase)
   b. If found: return (canonical4, confidence)
   c. Check first letter in UDS_DESIGNATORS as fallback
   d. If first-letter found: return (canonical4, confidence 0.70)
4. Not classified: return (UNKN, 0.0)
```

---

## App-Level Canonical4 Codes — Complete Reference

These are the codes used at the classification/ordering layer, distinct from the full EID-level codes in the UDS discipline table (see [UDS_DISCIPLINES.md](UDS_DISCIPLINES.md)):

| Canonical4 | Description                    | UDS Basis             |
|------------|--------------------------------|-----------------------|
| `GENR`     | General                        | `G_` → `GENL` in UDS |
| `SURV`     | Survey / Mapping               | `V_` → `SURV` in UDS |
| `DEMO`     | Demolition (generic)           | `D_` ambiguous; app extension |
| `CIVL`     | Civil                          | `C_` → `CIVL` in UDS |
| `LAND`     | Landscape                      | `L_` → `LAND` in UDS |
| `ARCH`     | Architectural                  | `A_` → `ARCH` in UDS |
| `INTR`     | Interiors                      | `I_` → `INTR` in UDS |
| `STRU`     | Structural                     | `S_` → `STRC` in UDS (name differs) |
| `MECH`     | Mechanical                     | `M_` → `MECH` in UDS |
| `PLUM`     | Plumbing                       | `P_` → `PLUM` in UDS |
| `FIRP`     | Fire Protection                | `F_` → `FIRE` in UDS (name differs) |
| `FIRA`     | Fire Alarm                     | `FA` → `FIRA` in UDS |
| `FIRE`     | Fire (legacy fallback)         | Back-compat only; prefer `FIRP` |
| `ELEC`     | Electrical                     | `E_` → `ELEC` in UDS |
| `TECH`     | Technology / IT / Low-Voltage  | `T_` → `TELE` in UDS (name differs) |
| `CTRL`     | Controls / DDC / BAS           | App extension; no UDS letter |
| `VEND`     | Vendor / Deferred              | App extension                 |
| `SPEC`     | Specifications                 | App extension                 |
| `UNKN`     | Unknown / Unclassified         | App extension                 |

---

## Implementation Notes for Rust

### Modules
- Classification logic → `crates/engine/src/classifier.rs` (or `crates/standards-data/src/classifier.rs` if kept as pure data + pure functions)
- Data tables → `crates/standards-data/src/aec.rs`

### Suggested Types

```rust
pub struct DesignatorEntry {
    pub letter: &'static str,       // Single UDS letter
    pub canonical4: &'static str,   // e.g. "CIVL"
    pub display_name: &'static str,
    pub ambiguous: bool,            // true for "C"
}

pub struct AliasEntry {
    pub alias: &'static str,        // Multi-letter alias, e.g. "FP"
    pub canonical4: &'static str,
    pub display_name: &'static str,
    pub uds_designator: Option<&'static str>, // Parent UDS letter if applicable
    pub confidence: f32,
}

pub struct ClassificationResult {
    pub canonical4: &'static str,
    pub confidence: f32,
    pub source: ClassificationSource,
}

pub enum ClassificationSource {
    UdsDesignator,
    Alias,
    Disambiguation,
    FallbackFirstLetter,
    Unknown,
}
```

### Confidence Thresholds

Use the canonical 4-tier system from [AEC_STANDARDS.md](./AEC_STANDARDS.md) §Confidence Scoring Reference:

| Range | Action | Basis label in audit trail |
|-------|--------|----------------------------|
| **≥0.95** | Auto-apply | `UDS` or `ALIAS` |
| **0.80–0.95** | Auto-apply + flag | `ALIAS` or `HEURISTIC` |
| **0.70–0.80** | Apply + queue for review | `HEURISTIC` |
| **<0.70** | Escalate to human / use `UNKN` | `UNKNOWN` |

### Keyword Matching
Use case-insensitive substring search (`.to_uppercase().contains(keyword)`). The prototype used `keywords.some(k => title.toUpperCase().includes(k))`.
