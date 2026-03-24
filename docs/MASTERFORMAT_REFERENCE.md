# CSI MasterFormat Reference

> **Parent document:** [AEC_STANDARDS.md](./AEC_STANDARDS.md) — see §Specifications Standards for the classification overview and 3-part section format.
> **Source:** Recovered from prototype — generated from `UDS.xlsx` tab `divisions`, plus hand-maintained `masterformatDivisions.ts` and `legacySections.generated.ts`.
> **Standard:** CSI MasterFormat 2018 (primary), with pre-2004 legacy migration table.
> **Status:** Canonical data ready for implementation in `crates/standards-data`.
> **Last Verified:** 2026-03-01 (prototype generation timestamp); MasterFormat 2018 verified 2026-01-17.

---

## Overview

CSI MasterFormat is the construction industry's standard specification numbering system. This project uses it to:

1. **Identify spec sections** in PDF documents (by parsing section numbers like `03 30 00`)
2. **Classify document type** — a PDF dominated by Division 03 content is likely a concrete spec
3. **Sort and group** specifications in output artifacts
4. **Migrate legacy codes** — projects prior to 2004 used a 16-division system; those codes must be mapped forward

---

## MasterFormat 2018 — Full Division Table

The `divisionCODE` is the internal 4-character code used for classification and filename generation.

> **Special sort note:** Division 00 (Procurement) has `order: 999` — it sorts at the end in drawing-set context because front-end contract documents are typically processed separately from technical specs. The section ranges in [AEC_STANDARDS.md](./AEC_STANDARDS.md) supersede the simpler note that was previously in that document.

| divisionID | divisionCODE | Division                                           | Description                              | Order |
|------------|--------------|----------------------------------------------------|------------------------------------------|-------|
| `00`       | `PROC`       | Procurement and Contracting Requirements           | Front-end, bidding, contracts            | 999   |
| `01`       | `GNRL`       | General Requirements                               | Admin, procedures, temporary works       | 1     |
| `02`       | `EXST`       | Existing Conditions                                | Survey, demo, hazmat, subsurface         | 2     |
| `03`       | `CONC`       | Concrete                                           | Cast-in-place, precast, toppings         | 3     |
| `04`       | `MASN`       | Masonry                                            | Unit masonry, stone, veneers             | 4     |
| `05`       | `METL`       | Metals                                             | Structural, misc, ornamental metals      | 5     |
| `06`       | `WOOD`       | Wood, Plastics, and Composites                     | Rough carpentry, finish, plastics        | 6     |
| `07`       | `THMO`       | Thermal and Moisture Protection                    | Roofing, waterproofing, insulation       | 7     |
| `08`       | `OPEN`       | Openings                                           | Doors, frames, windows, curtain wall     | 8     |
| `09`       | `FINI`       | Finishes                                           | Gypsum, ceilings, flooring, paint        | 9     |
| `10`       | `SPEC`       | Specialties                                        | Misc specialties, toilet accessories     | 10    |
| `11`       | `EQPT`       | Equipment                                          | Fixed and movable equipment              | 11    |
| `12`       | `FURN`       | Furnishings                                        | Furniture, casework, art                 | 12    |
| `13`       | `SPLC`       | Special Construction                               | PEMBs, clean rooms, pools, etc.          | 13    |
| `14`       | `CONV`       | Conveying Equipment                                | Elevators, escalators, lifts             | 14    |
| `20`       | `MSUP`       | Mechanical Support                                 | Common/mechanical support (reserved)     | 20    |
| `21`       | `FSPR`       | Fire Suppression                                   | Sprinklers, standpipes, agents           | 21    |
| `22`       | `PLUM`       | Plumbing                                           | Plumbing systems                         | 22    |
| `23`       | `MECH`       | Heating, Ventilating, and Air Conditioning (HVAC)  | HVAC systems                             | 23    |
| `25`       | `AUTO`       | Integrated Automation                              | Controls and automation                  | 25    |
| `26`       | `ELEC`       | Electrical                                         | Power, distribution, lighting            | 26    |
| `27`       | `COMM`       | Communications                                     | IT, data, telecom                        | 27    |
| `28`       | `SECU`       | Electronic Safety and Security                     | FA, access control, CCTV, security       | 28    |
| `31`       | `EWRK`       | Earthwork                                          | Excavation, fill, piles                  | 31    |
| `32`       | `EXIM`       | Exterior Improvements                              | Paving, landscaping, site furnishings    | 32    |
| `33`       | `UTIL`       | Utilities                                          | Site utilities, pipe, structures         | 33    |
| `34`       | `TRAN`       | Transportation                                     | Roads, rails, airfield systems           | 34    |
| `35`       | `WTRM`       | Waterway and Marine Construction                   | Marine, coastal, dredging                | 35    |
| `40`       | `PROC`       | Process Integration                                | Process systems integration              | 40    |
| `41`       | `MPHE`       | Material Processing and Handling Equipment         | Conveyors, cranes, process equip.        | 41    |
| `42`       | `PHCD`       | Process Heating, Cooling, and Drying Equipment     | Kilns, dryers, furnaces                  | 42    |
| `43`       | `PGAS`       | Process Gas and Liquid Handling, Purification, Storage | Process piping, tanks, treatment    | 43    |
| `44`       | `PWCE`       | Pollution and Waste Control Equipment              | Pollution control, waste systems         | 44    |
| `45`       | `IMFG`       | Industry-Specific Manufacturing Equipment          | Specialized industrial equipment         | 45    |
| `46`       | `WWTR`       | Water and Wastewater Equipment                     | Treatment and plant equipment            | 46    |
| `48`       | `PWRG`       | Electrical Power Generation                        | On-site generation                       | 48    |
| `49`       | `RSVR`       | Reserved for Future Expansion                      | Currently unused                         | 49    |

> **Code collision note:** Both Division 00 and Division 40 use `divisionCODE: "PROC"`. The lookup-by-code map will return whichever is inserted last (Division 40 wins). If disambiguation is needed, lookup by `divisionID` (the 2-digit string) is authoritative.

---

## MasterFormat — Missing Divisions

Divisions 15–19, 24, 29–30, 36–39, 47 are **unassigned/reserved** in MasterFormat 2018 and are not present in this dataset. Do not infer content from their absence.

---

## Legacy MasterFormat (pre-2004) Migration Table

Prior to 2004, MasterFormat used 16 numbered divisions. Many projects still reference these legacy codes. The migration table maps legacy **section ranges** within each legacy division to the modern 2018 division.

### Key Structural Difference

- **Pre-2004:** 16 divisions, numbered 01–16. Sections were 5-digit: `XXYYYY` (e.g., `15300`).
- **2004+/2018:** ~50 divisions. Sections are 6-digit with spaces: `XX XX XX` (e.g., `21 13 00`).

Divisions 01–14 map **1-to-1**. Divisions 15 and 16 are **many-to-one** (they split into multiple modern divisions based on section number ranges).

### Divisions 01–14 (Direct Mapping)

| Legacy Div | Legacy Title             | Modern Div | Modern CODE |
|------------|--------------------------|------------|-------------|
| `01`       | General Requirements     | `01`       | `GNRL`      |
| `02`       | Site Construction        | `02`       | `EXST`      |
| `03`       | Concrete                 | `03`       | `CONC`      |
| `04`       | Masonry                  | `04`       | `MASN`      |
| `05`       | Metals                   | `05`       | `METL`      |
| `06`       | Wood and Plastics        | `06`       | `WOOD`      |
| `07`       | Thermal & Moisture Protection | `07`  | `THMO`      |
| `08`       | Doors and Windows        | `08`       | `OPEN`      |
| `09`       | Finishes                 | `09`       | `FINI`      |
| `10`       | Specialties              | `10`       | `SPEC`      |
| `11`       | Equipment                | `11`       | `EQPT`      |
| `12`       | Furnishings              | `12`       | `FURN`      |
| `13`       | Special Construction     | `13`       | `SPLC`      |
| `14`       | Conveying Systems        | `14`       | `CONV`      |

### Division 15 (Mechanical — Range-Based Split)

Legacy Division 15 sections were split across modern Divisions 20–25 and 40 based on section number:

| Legacy Range    | Legacy Section Title                  | Modern Div | Modern CODE |
|-----------------|---------------------------------------|------------|-------------|
| `15000–15099`   | Basic Mechanical Materials & Methods  | `20`       | `MSUP`      |
| `15100–15199`   | Building Services Piping              | `22`       | `PLUM`      |
| `15200–15299`   | Process Piping                        | `40`       | `PROC`      |
| `15300–15399`   | Fire Protection Piping                | `21`       | `FSPR`      |
| `15400–15499`   | Plumbing Fixtures & Equipment         | `22`       | `PLUM`      |
| `15500–15599`   | Heat Generation Equipment             | `23`       | `HVAC`      |
| `15600–15699`   | Refrigeration Equipment               | `23`       | `HVAC`      |
| `15700–15799`   | HVAC Equipment                        | `23`       | `HVAC`      |
| `15800–15899`   | Air Distribution                      | `23`       | `HVAC`      |
| `15900–15999`   | HVAC Instrumentation & Controls       | `25`       | `AUTO`      |
| `15950–15959`   | Testing Adjusting Balancing           | `23`       | `HVAC`      |

> **Note:** Ranges `15950–15959` overlap with `15900–15999`. The more specific range takes precedence.

### Division 16 (Electrical — Range-Based Split)

Legacy Division 16 sections were split across modern Divisions 26–28:

| Legacy Range    | Legacy Section Title                  | Modern Div | Modern CODE |
|-----------------|---------------------------------------|------------|-------------|
| `16000–16099`   | Basic Electrical Materials & Methods  | `26`       | `ELEC`      |
| `16100–16199`   | Wiring Methods Conduits Raceways      | `26`       | `ELEC`      |
| `16200–16299`   | Electrical Power Switchgear           | `26`       | `ELEC`      |
| `16400–16499`   | Low Voltage Distribution              | `26`       | `ELEC`      |
| `16500–16599`   | Lighting                              | `26`       | `ELEC`      |
| `16700–16799`   | Communications                        | `27`       | `COMM`      |
| `16720–16729`   | Fire Alarm                            | `28`       | `SECU`      |
| `16740–16749`   | Telephone Provisions                  | `27`       | `COMM`      |
| `16800–16899`   | Sound & Video                         | `27`       | `COMM`      |

> **Note:** Fire Alarm (`16720–16729`) is a sub-range of Communications (`16700–16799`). The more specific range takes precedence. Fire alarm maps to Division 28 (Electronic Safety and Security), not 27.

---

## Range-Based Lookup Algorithm

To migrate a legacy 5-digit section number to a modern division:

```
1. Parse the 5-digit number as an integer (e.g., "15300" → 15300)
2. Determine the legacy division (first 2 digits): 15300 → div "15"
3. If div is "01"–"14": direct map to corresponding modern division
4. If div is "15": scan ranges from most-specific to least-specific; return first match
5. If div is "16": scan ranges from most-specific to least-specific; return first match
6. Unknown: return None / UNKN
```

The prototype implemented this as `isInLegacyRange(code: string, range: string): boolean` which parsed `"15300–15399"` into start/end integers and tested inclusion.

---

## Section Number Format Detection

When parsing a PDF spec, use these patterns to detect section number format:

| Format         | Regex Pattern            | Example      | Era       |
|----------------|--------------------------|--------------|-----------|
| Modern 6-digit | `\d{2} \d{2} \d{2}`     | `03 30 00`   | 2004+     |
| Legacy 5-digit | `\d{5}`                  | `03300`      | pre-2004  |
| Legacy 5-digit | `\d{2}[- ]\d{3}`         | `03-300`     | pre-2004  |

---

## Implementation Notes for Rust (`crates/standards-data/src/masterformat.rs`)

### Recommended Types

```rust
pub struct DivisionEntry {
    pub division_id: &'static str,    // "03"
    pub division_code: &'static str,  // "CONC"
    pub division: &'static str,       // "Concrete"
    pub division_desc: &'static str,  // "Cast-in-place, precast, toppings"
    pub order: u8,                    // 3 (except Div 00 = 255 or use u16)
    pub mf_version: &'static str,     // "2018"
}

pub struct LegacySectionEntry {
    pub legacy_div_id: &'static str,  // "15"
    pub section_range: &'static str,  // "15300–15399"
    pub section_title: &'static str,  // "Fire Protection Piping"
    pub division_id: &'static str,    // "21"
    pub division_code: &'static str,  // "FSPR"
    pub year: &'static str,           // "pre-2004"
}
```

### Functions Needed

```rust
/// Look up a modern division by 2-digit ID string
pub fn division_by_id(id: &str) -> Option<&'static DivisionEntry>;

/// Look up a modern division by 4-char code  
pub fn division_by_code(code: &str) -> Option<&'static DivisionEntry>;

/// Migrate a legacy 5-digit section code to a modern division
/// Returns None if the code is not recognizable
pub fn migrate_legacy_section(code: &str) -> Option<&'static DivisionEntry>;
```

### Data Version
The prototype stored a `MASTERFORMAT_META` constant: `version: "2018"`, `lastVerified: "2026-01-17"`. Persist this in Rust as a module-level constant.
