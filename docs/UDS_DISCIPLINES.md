# UDS Drawing Discipline Reference

> **Parent document:** [AEC_STANDARDS.md](./AEC_STANDARDS.md) — see §Drawings Standards for the classification overview and sort-order heuristic.
> **Source:** Recovered from prototype — generated from `UDS.xlsx`, tab `disciplines`.
> **Standard:** AIA/NCS Uniform Drawing System (UDS), compatible with CADstandards.
> **Status:** Canonical data ready for implementation in `crates/standards-data`.
> **Last Verified:** 2026-03-01 (prototype generation timestamp)

---

## Overview

The UDS discipline system organizes construction drawing sets by **Discipline ID** (a single letter on the drawing sheet number, e.g., `A`, `S`, `M`) and an optional **modifier letter** to form a **2-character Extended ID** (`disciplineEid`, e.g., `AB`, `SF`, `MA`).

Each discipline and sub-discipline is mapped to a **4-character discipline CODE** used internally for sorting, filename generation, and classification logic.

### Terminology

| Field           | Length | Example  | Meaning                                   |
|-----------------|--------|----------|-------------------------------------------|
| `disciplineID`  | 1 char | `A`      | Top-level UDS letter (on sheet number)    |
| `disciplineEid` | 2 char | `AB`     | Extended ID = disciplineID + modifier     |
| `disciplineCODE`| 4 char | `ABLD`   | Internal 4-char canonical code            |
| `disciplineFull`| string | `Architectural Building` | Human-readable full name     |
| `disciplineDesc`| string | `Plans, elevations` | Short usage description         |
| `order`         | int    | 420      | Canonical sort position (ascending)       |
| `udsStandard`   | bool   | `true`   | Whether this entry is from UDS standard   |

The `disciplineEid` uses `X_` (underscore) to denote "no modifier" (i.e., just the bare letter with no second character).

---

## App-Level Canonical4 Codes

The classification layer uses a **simplified set of ~20 top-level canonical codes** (distinct from the per-EID `disciplineCODE` values) for sorting and high-level grouping. These appear in `drawingsDesignators.ts` and `drawingsOrderHeuristic.ts`:

| Canonical4 | Discipline       | Notes                                      |
|------------|------------------|--------------------------------------------|
| `GENR`     | General          | Cover, symbols, legends                    |
| `SURV`     | Survey/Mapping   | Also covers existing conditions            |
| `DEMO`     | Demolition       | Cross-discipline demolition work           |
| `CIVL`     | Civil            | Default for `C` designator (see disambiguation) |
| `LAND`     | Landscape        |                                            |
| `ARCH`     | Architectural    |                                            |
| `INTR`     | Interiors        | If separate from Arch                      |
| `STRU`     | Structural       | Note: UDS EID table uses `STRC` for `S_`; `STRU` is the app-level code |
| `MECH`     | Mechanical       |                                            |
| `PLUM`     | Plumbing         |                                            |
| `FIRP`     | Fire Protection  | Note: UDS EID table uses `FIRE` for `F_`; `FIRP` is the app-level code |
| `FIRA`     | Fire Alarm       | Sub-discipline of Fire Protection          |
| `FIRE`     | Fire (legacy)    | Back-compat fallback only                  |
| `ELEC`     | Electrical       |                                            |
| `TECH`     | Technology       | Note: UDS EID table uses `TELE` for `T_`; `TECH` is the app-level code |
| `CTRL`     | Controls         | DDC/BAS/ATC — not a UDS discipline letter; app extension |
| `VEND`     | Vendor/Deferred  | App extension for deferred submittals      |
| `SPEC`     | Specifications   | App extension for spec-only sheets         |
| `UNKN`     | Unknown          | Fallback when classification fails         |

> **Key discrepancy resolved:** The UDS-generated table uses `STRC` (for `S_`), `FIRE` (for `F_`), and `TELE` (for `T_`), but the app-level classification and ordering layer — as documented in [AEC_STANDARDS.md](./AEC_STANDARDS.md) — uses `STRU`, `FIRP`, and `TECH` as the canonical4 codes. **These app-level codes are the resolved standard.** The EID-level `disciplineCODE` values from this table are used for sub-discipline lookup only, not for top-level classification.

---

## Full Discipline Table

Sorted by `order` field (canonical drawing-set sort order). All entries are `udsStandard: true`.

### G — General
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `G_` | `GENL` | General (no modifier)         | Cover, symbols, code, typicals | 10    |
| `GG` | `GGRA` | General Graphics              | Symbols, legends, drawing lists| 20    |
| `GN` | `GNOT` | General Notes                 | General & keynotes sheets      | 30    |
| `GT` | `GTMP` | General Temporary Works       | Phasing, temporary protections | 40    |

### H — Hazardous Materials
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `H_` | `HAZD` | Hazardous (no modifier)       | Overall haz-mat sheets         | 50    |
| `HC` | `HCON` | Hazmat Containment            | Barriers, enclosures           | 60    |
| `HR` | `HREM` | Hazmat Removal / Abatement    | Abatement drawings             | 70    |

### V — Survey / Mapping
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `V_` | `SURV` | Survey / Mapping (no modifier)| Overall surveys                | 80    |
| `VA` | `VALG` | Survey Alignment / Grid       | Control, layout, grids         | 90    |
| `VT` | `VTOP` | Topographic Survey            | Topo plans                     | 100   |
| `VU` | `VUTL` | Utility Survey                | Existing utilities             | 110   |

### B — Geotechnical
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `B_` | `GEOT` | Geotechnical (no modifier)    | Overall geotech info           | 120   |
| `BB` | `BBOR` | Boring Logs                   | Borings and test pits          | 130   |
| `BF` | `BFDN` | Foundation Investigation      | Subsurface recommendations     | 140   |

### C — Civil
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `C_` | `CIVL` | Civil (no modifier)           | General civil                  | 150   |
| `CA` | `CALG` | Civil Alignment / Roadway     | Centerlines, road geometry     | 160   |
| `CD` | `CDEM` | Civil Demolition              | Civil/site demo                | 170   |
| `CG` | `CGRD` | Civil Grading                 | Grading / earthwork            | 180   |
| `CP` | `CPRK` | Civil Parking / Paving        | Parking lots, paving           | 190   |
| `CR` | `CROW` | Civil Right-of-Way            | ROW, easements                 | 200   |
| `CS` | `CSIT` | Civil Site                    | General site layout            | 210   |
| `CU` | `CUTC` | Civil Utilities               | Site utilities                 | 220   |
| `CW` | `CWRK` | Civil Works                   | Levees, dams, canals           | 230   |
| `CZ` | `CCAN` | Civil Canals / Channels       | Channels, canals               | 240   |

> **Disambiguation note:** `C` is ambiguous — it can mean Civil or Controls (DDC/BAS). See [DRAWINGS_CLASSIFICATION.md](DRAWINGS_CLASSIFICATION.md) for the keyword disambiguation algorithm.

### L — Landscape
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `L_` | `LAND` | Landscape (no modifier)       | Overall landscape              | 250   |
| `LA` | `LARC` | Landscape Architecture        | Planting & hardscape           | 260   |
| `LD` | `LDEM` | Landscape Demolition          | Demo of landscape              | 270   |
| `LI` | `LIRR` | Landscape Irrigation          | Irrigation systems             | 280   |
| `LL` | `LLGH` | Landscape Lighting            | Site/landscape lighting        | 290   |
| `LS` | `LSIT` | Landscape Site                | Grading, paths, plazas         | 300   |

### S — Structural
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `S_` | `STRC` | Structural (no modifier)      | General structural             | 310   |
| `SA` | `SAFL` | Structural Flood / Coastal    | Floodwalls, seawalls           | 320   |
| `SB` | `SBRG` | Structural Bridge             | Bridge structures              | 330   |
| `SD` | `SDEM` | Structural Demolition         | Structural demo                | 340   |
| `SF` | `SFRM` | Structural Framing            | Framing systems                | 350   |
| `SG` | `SGRA` | Structural General            | Notes, schedules               | 360   |
| `SN` | `SFND` | Structural Foundations        | Footings, piles, piers         | 370   |
| `SR` | `SREH` | Structural Rehabilitation     | Strengthening, repair          | 380   |
| `ST` | `STRN` | Structural Tanks / Vessels    | Tanks, silos                   | 390   |

### A — Architectural
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `A_` | `ARCH` | Architectural (no modifier)   | General architectural          | 400   |
| `AA` | `ASIT` | Architectural Site            | Site plans by architect        | 410   |
| `AB` | `ABLD` | Architectural Building        | Plans, elevations              | 420   |
| `AC` | `AOCC` | Area / Occupancy              | Life-safety, occupancy         | 430   |
| `AD` | `ADEM` | Architectural Demolition      | Arch demo                      | 440   |
| `AE` | `AELE` | Architectural Elements        | Stairs, ramps, rails, etc.     | 450   |
| `AF` | `AFIN` | Architectural Finishes        | Finish plans & schedules       | 460   |
| `AG` | `AGRF` | Architectural Graphics        | Signage, graphics              | 470   |
| `AI` | `AINT` | Architectural Interiors       | Interior architecture          | 480   |
| `AJ` | `AUSR` | User-Defined J                | User-defined modifier          | 490   |
| `AK` | `AUSK` | User-Defined K                | User-defined modifier          | 500   |
| `AL` | `ALOD` | Architectural Lodging / Hotel | Guestroom protos, etc.         | 510   |
| `AM` | `AMED` | Architectural Medical         | Healthcare facilities          | 520   |
| `AP` | `APAR` | Architectural Parking Structures | Parking garages             | 530   |
| `AR` | `AROF` | Architectural Roof            | Roof plans and details         | 540   |

### I — Interiors
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `I_` | `INTR` | Interiors (no modifier)       | Interior overall               | 550   |
| `IA` | `IARC` | Interior Architecture         | Partitions, ceilings           | 560   |
| `ID` | `IDEM` | Interiors Demolition          | Interior demo                  | 570   |
| `IF` | `IFUR` | Interior Furniture            | Furniture plans                | 580   |
| `IG` | `IGRA` | Interior Graphics             | Environmental graphics         | 590   |
| `IL` | `ILIT` | Interior Lighting / Finish Plans | RCPs, finish plans          | 600   |
| `IP` | `ISGN` | Interior Signage              | Signage layouts                | 610   |
| `IQ` | `IEQP` | Interior Equipment Layout     | Interior equipment             | 620   |

### Q — Equipment
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `Q_` | `EQPT` | Equipment (no modifier)       | General equipment              | 630   |
| `QA` | `AEQP` | Architectural Equipment       | Built-in arch equipment        | 640   |
| `QE` | `EEQP` | Electrical Equipment          | Panels, gear                   | 650   |
| `QM` | `MEQP` | Mechanical Equipment          | AHUs, pumps                    | 660   |
| `QP` | `PEQP` | Process Equipment             | Process plant equipment        | 670   |
| `QT` | `TEQP` | Telecom Equipment             | Racks, cabinets                | 680   |

### F — Fire Protection
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `F_` | `FIRE` | Fire Protection (no modifier) | General FP                     | 690   |
| `FA` | `FIRA` | Fire Alarm                    | FA systems                     | 700   |
| `FB` | `FBRG` | Fire Barriers / Rated Assemblies | Rated walls, doors          | 710   |
| `FL` | `FLSF` | Life Safety / Egress          | Egress plans                   | 720   |
| `FS` | `FSPR` | Fire Sprinkler                | Sprinkler systems              | 730   |
| `FT` | `FTEC` | Fire Detection Technology     | Detection, special systems     | 740   |

### P — Plumbing
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `P_` | `PLUM` | Plumbing (no modifier)        | General plumbing               | 750   |
| `PA` | `PPLN` | Plumbing Plans                | Overall plumbing               | 760   |
| `PD` | `PDEQ` | Plumbing Demolition           | Plumbing demo                  | 770   |
| `PG` | `PGSW` | Plumbing Gas / Special Waste  | Gas, lab waste                 | 780   |
| `PP` | `PIPG` | Plumbing Piping               | Domestic & process piping      | 790   |
| `PS` | `PSSW` | Plumbing Sanitary / Sewer     | Sanitary, storm                | 800   |
| `PW` | `PWTW` | Plumbing Water                | Water distribution             | 810   |

### D — Process (Industrial)
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `D_` | `PROC` | Process (no modifier)         | General process                | 820   |
| `DA` | `DADM` | Process Admin / Control       | Controls, MCCs                 | 830   |
| `DD` | `DDEM` | Process Demolition            | Process demo                   | 840   |
| `DP` | `DPRO` | Process Piping                | Process piping                 | 850   |
| `DR` | `DREF` | Process Refineries / Plants   | Heavy process facilities       | 860   |

### M — Mechanical
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `M_` | `MECH` | Mechanical (no modifier)      | General mechanical             | 870   |
| `MA` | `MAIR` | Mechanical Airside (Ductwork) | Air distribution               | 880   |
| `MD` | `MDEM` | Mechanical Demolition         | Mechanical demo                | 890   |
| `MH` | `MMAT` | Mechanical Material Handling  | Conveyors, cranes              | 900   |
| `MP` | `MPIP` | Mechanical Piping             | CHW/HW/steam, etc.             | 910   |
| `MR` | `MREF` | Mechanical Refrigeration      | Refrigeration systems          | 920   |
| `MS` | `MHVC` | Mechanical HVAC               | HVAC systems                   | 930   |
| `MT` | `MTEC` | Mechanical Controls / BAS     | Controls, BAS                  | 940   |

### E — Electrical
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `E_` | `ELEC` | Electrical (no modifier)      | General electrical             | 950   |
| `EA` | `ESVC` | Electrical Service / Distribution | Service, feeders           | 960   |
| `ED` | `EDEM` | Electrical Demolition         | Electrical demo                | 970   |
| `EG` | `EGEN` | Electrical General / Power    | General power                  | 980   |
| `EL` | `ELIT` | Electrical Lighting           | Lighting systems               | 990   |
| `EM` | `EEMG` | Emergency / Standby Power     | Generators, UPS                | 1000  |
| `EP` | `EPOW` | Electrical Power              | Branch and distribution        | 1010  |
| `ER` | `ERES` | Electrical Residential / Unit | Unit power layouts             | 1020  |
| `ES` | `ESYS` | Electrical Systems (Low-Voltage) | LV, controls                | 1030  |

### W — Distributed Energy
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `W_` | `DIST` | Distributed Energy (no modifier) | Overall distributed energy  | 1040  |
| `WA` | `WPV`  | Solar Photovoltaic            | PV arrays and gear             | 1050  |
| `WB` | `WBAT` | Battery / Storage Systems     | ESS, batteries                 | 1060  |
| `WC` | `WCOG` | Cogeneration / CHP            | Co-gen plants                  | 1070  |
| `WE` | `WELC` | Electrical Generation         | Non-utility generation         | 1080  |
| `WG` | `WGEO` | Geothermal Systems            | Geothermal loops               | 1090  |

### T — Telecommunications
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `T_` | `TELE` | Telecommunications (no modifier) | General telecom             | 1100  |
| `TA` | `TAVS` | Audio / Visual Systems        | AV, CATV, CCTV                 | 1110  |
| `TC` | `TCLK` | Clock and Program             | Time/bell systems              | 1120  |
| `TI` | `TINT` | Intercom                      | Intercom / PA                  | 1130  |
| `TM` | `TMON` | Monitoring                    | Monitoring / alarms            | 1140  |
| `TN` | `TNET` | Data Networks                 | Network cabling                | 1150  |
| `TT` | `TTEL` | Telephone                     | Voice systems                  | 1160  |
| `TY` | `TSEC` | Security                      | Access control & security      | 1170  |
| `TJ` | `TUSR` | User-Defined J                | User-defined modifier          | 1180  |
| `TK` | `TUSK` | User-Defined K                | User-defined modifier          | 1190  |

### R — Resource (Existing Conditions)
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `R_` | `RSRC` | Resource (no modifier)        | Existing overall               | 1200  |
| `RA` | `RBLD` | Existing Buildings            | Existing architectural         | 1210  |
| `RC` | `RCIV` | Existing Civil / Site         | Existing civil                 | 1220  |
| `RE` | `RELE` | Existing Electrical           | Existing electrical            | 1230  |
| `RL` | `RLAN` | Existing Landscape            | Existing landscape             | 1240  |
| `RM` | `RMEC` | Existing Mechanical           | Existing mechanical            | 1250  |
| `RS` | `RSTR` | Existing Structural           | Existing structural            | 1260  |

### X — Other Disciplines
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `X_` | `OTHR` | Other (no modifier)           | Misc / special                 | 1270  |
| `XA` | `XALT` | Alternate / Optional Work     | Add alternates                 | 1280  |
| `XC` | `XCOM` | Commissioning                 | Cx drawings                    | 1290  |
| `XG` | `XGRD` | Green Building / Sustainability | LEED / sustainable           | 1300  |
| `XM` | `XMED` | Medical Equipment Planning    | Med-equip layouts              | 1310  |
| `XS` | `XSPE` | Specs / Narrative Only        | Narrative or calc sheets       | 1320  |

### Z — Contractor / Shop Drawings
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `Z_` | `SHOP` | Shop Drawings (no modifier)   | General shop                   | 1330  |
| `ZA` | `ZARC` | Architectural Shop Drawings   | Arch-shop                      | 1340  |
| `ZE` | `ZELE` | Electrical Shop Drawings      | Elec-shop                      | 1350  |
| `ZM` | `ZMECH`| Mechanical Shop Drawings      | Mech-shop                      | 1360  |
| `ZS` | `ZSTR` | Structural Shop Drawings      | Struct-shop                    | 1370  |

### O — Operations
| EID  | CODE   | Full Name                     | Description                    | Order |
|------|--------|-------------------------------|--------------------------------|-------|
| `O_` | `OPER` | Operations (no modifier)      | General operations             | 1380  |
| `OA` | `OFCL` | Facility Operations / Management | FOM plans                  | 1390  |
| `OM` | `OMTN` | Operations Maintenance        | O&M diagrams                   | 1400  |
| `OT` | `OTEC` | Operations Technology / Controls | Post-occupancy controls     | 1410  |

---

## Implementation Notes for Rust (`crates/standards-data`)

### Recommended Types

```rust
/// A single UDS discipline sub-entry (one row of the full table)
pub struct DisciplineEntry {
    pub discipline_id: &'static str,    // Single letter, e.g. "A"
    pub discipline: &'static str,       // Group name, e.g. "Architectural"
    pub discipline_eid: &'static str,   // 2-char extended ID, e.g. "AB" or "A_"
    pub discipline_code: &'static str,  // 4-char code, e.g. "ABLD"
    pub discipline_full: &'static str,  // e.g. "Architectural Building"
    pub discipline_desc: &'static str,  // e.g. "Plans, elevations"
    pub order: u16,                     // Sort order (10, 20, ..., 1410)
    pub uds_standard: bool,
}
```

### Lookup Maps Needed
- `disciplines_by_eid(eid: &str) -> Option<&DisciplineEntry>` — primary lookup
- `disciplines_by_code(code: &str) -> Option<&DisciplineEntry>` — reverse lookup
- `disciplines_by_id(id: char) -> Vec<&DisciplineEntry>` — all sub-disciplines for a letter

### Storage
Use `static` arrays with `phf` (perfect hash maps) or runtime `HashMap` built at startup. Given the ~100 entries, `phf` is preferred for zero-runtime-cost lookup.

### Confidence Thresholds (canonical, from AEC_STANDARDS.md)

| Range | Action |
|-------|--------|
| **≥0.95** | Auto-apply |
| **0.80–0.95** | Auto-apply + flag in audit trail |
| **0.70–0.80** | Apply + queue for review |
| **<0.70** | Escalate to human / use `UNKN` |

### Source Regeneration
In the prototype, data was regenerated from `UDS.xlsx` via `scripts/generate-standards-datasets.ts`. This Excel source should be retained for future updates. The data above is the canonical extracted snapshot.
