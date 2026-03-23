# Conset PDF: AEC Standards V4.2.1

**Version:** 4.2.1  
**Date:** January 23, 2026  
**Owner:** HLLMR LLC  
**Status:** ✅ ACTIVE
**Doc Status Tag:** Implemented

---

## Overview

This document is the **authoritative reference for AEC domain knowledge** in Conset PDF. It defines the structure, conventions, and standards used in Architecture, Engineering, and Construction documents.

This is a canonical derived document under `MASTER_PLAN_v4.md` per `DOC_GOVERNANCE.md`.

**Audience:** Developers (human and AI agents) who need to understand AEC document structure

**Purpose:** Provide domain expertise so code can correctly parse, classify, and process AEC documents

**Scope:**
- Specifications (CSI MasterFormat)
- Drawings (UDS discipline classification)
- Submittals (equipment tag conventions)
- Common AEC terminology and patterns

---

## Table of Contents

1. [AEC Document Principles](#aec-document-principles)
2. [Specifications Standards](#specifications-standards)
3. [Drawings Standards](#drawings-standards)
4. [Submittals Standards](#submittals-standards)
5. [Common AEC Terminology](#common-aec-terminology)
6. [Confidence Scoring Reference](#confidence-scoring-reference)

---

## AEC Document Principles

### Principle 1: Medium-Specificity

**AEC documents have fundamentally different grammars:**

| Medium | Structure | Organization | Chrome |
|--------|-----------|--------------|--------|
| **Specifications** | Hierarchical outline | Section-based (MasterFormat) | Headers/footers with section IDs |
| **Drawings** | Spatial/visual | Discipline-based (UDS) | Title blocks, revision blocks |
| **Submittals** | Equipment-centric | Tag-based grouping | Form headers, project bands |

**Implication:** Code must respect medium differences. A spec parser ≠ drawing parser.

---

### Principle 2: Standards as Ground Truth

**AEC follows industry standards:**
- **Specifications:** CSI MasterFormat (section numbering)
- **Drawings:** UDS (Unified Discipline System) + NCS (National CAD Standard)
- **Submittals:** Equipment tag conventions (varies by discipline)

**Implication:** Engine understands AEC semantics natively. "Division 23 = HVAC" is knowledge, not inference.

---

### Principle 3: Confidence Must Be Justified

**Every classification has a basis:**
- **High confidence (0.95–1.0):** Exact match to known standard
- **Medium confidence (0.80–0.95):** Alias match or heuristic confirmation
- **Low confidence (<0.80):** Escalate to human review

**Implication:** Never guess. Always provide basis: "UDS" | "ALIAS" | "HEURISTIC" | "MASTERFORMAT" | "UNKNOWN"

---

## Specifications Standards

### CSI MasterFormat 2018

**Structure:** Specifications organized by **divisions** (00-49), each containing **sections** (6-digit codes).

**Format:** `DD SS SS` where:
- `DD` = Division (00-49)
- `SS SS` = Section and subsection

**Example:** `23 82 16` = Division 23 (HVAC), Section 82 (Convection Heating/Cooling), Subsection 16 (Heating Water Coils)

---

### MasterFormat Divisions (00-49)

| Division | Title | Typical Content |
|----------|-------|-----------------|
| **00** | Procurement and Contracting Requirements | Bidding, contracting, submittals |
| **01** | General Requirements | Project management, quality, closeout |
| **02** | Existing Conditions | Demolition, site assessment |
| **03** | Concrete | Concrete forming, reinforcing, casting |
| **04** | Masonry | Unit masonry, stone, mortar |
| **05** | Metals | Structural steel, metal fabrications |
| **06** | Wood, Plastics, and Composites | Rough carpentry, finish carpentry |
| **07** | Thermal and Moisture Protection | Insulation, roofing, waterproofing |
| **08** | Openings | Doors, windows, glazing |
| **09** | Finishes | Flooring, ceilings, painting |
| **10** | Specialties | Signage, lockers, toilet accessories |
| **11** | Equipment | Commercial equipment, theater equipment |
| **12** | Furnishings | Furniture, window treatments |
| **13** | Special Construction | Special structures, pre-engineered structures |
| **14** | Conveying Equipment | Elevators, escalators, lifts |
| **21** | Fire Suppression | Fire sprinklers, standpipes |
| **22** | Plumbing | Domestic water, sanitary waste, storm drainage |
| **23** | HVAC (Heating, Ventilating, and Air Conditioning) | Ductwork, air handling, hydronic heating/cooling |
| **25** | Integrated Automation | Building automation, controls integration |
| **26** | Electrical | Power distribution, lighting, devices |
| **27** | Communications | Voice/data, AV systems |
| **28** | Electronic Safety and Security | Fire alarm, security systems |
| **31** | Earthwork | Excavation, fill, grading |
| **32** | Exterior Improvements | Paving, landscaping, site utilities |
| **33** | Utilities | Water supply, sanitary sewer, storm drainage |

**Note:** Divisions 15-20, 24, 29-30, 34-49 are reserved or less commonly used.

---

### Spec Section Structure (3-Part Format)

**Standard organization within each section:**

```
SECTION XX XX XX - SECTION TITLE

PART 1 — GENERAL
  1.1 SUMMARY
  1.2 REFERENCES
  1.3 SUBMITTALS
  1.4 QUALITY ASSURANCE
  1.5 DELIVERY, STORAGE, AND HANDLING
  1.6 PROJECT CONDITIONS
  1.7 WARRANTY

PART 2 — PRODUCTS
  2.1 MANUFACTURERS
  2.2 MATERIALS
  2.3 ACCESSORIES
  2.4 FABRICATION
  2.5 SOURCE QUALITY CONTROL

PART 3 — EXECUTION
  3.1 EXAMINATION
  3.2 PREPARATION
  3.3 INSTALLATION
  3.4 FIELD QUALITY CONTROL
  3.5 ADJUSTING
  3.6 CLEANING
  3.7 PROTECTION
```

**Outline numbering:**
- **Part:** PART 1, PART 2, PART 3
- **Article:** 1.1, 1.2, 2.1, 2.2, etc.
- **Paragraph:** A., B., C., D., etc. (or 1., 2., 3., etc.)
- **Sub-paragraph:** 1., 2., 3., etc. (or a., b., c., etc.)

**Example hierarchy:**
```
PART 2 — PRODUCTS
  2.7 HYDRONIC HEATING COILS
    A. Basis of Design: Trane Model HPHW.
    B. Acceptable Manufacturers:
      1. Carrier
      2. York
      3. McQuay
    C. Construction:
      1. Tubes: Seamless copper.
      2. Fins: Aluminum.
```

---

### Spec Footer Patterns

**Common footer formats:**

```
[Date]    [Section ID] – [Section Title] - Page [X] of [Y]

Examples:
2025-10-01    23 82 16 – Heating Water Coils - Page 2 of 3
2025-10-01    00 01 10 – TABLE OF CONTENTS - Page 1 of 5
2024-12-15    26 05 00 – Common Work Results for Electrical - Page 7 of 12
```

**Footer components:**
1. **Date:** Project date or revision date
2. **Section ID:** 6-digit MasterFormat code with spaces
3. **Section Title:** Usually abbreviated, may be ALL CAPS
4. **Page counter:** "Page X of Y" format

**Parsing strategy:**
- Footer is **ground truth** for section boundaries (footer-first oracle)
- Headers may be stale (delayed updates), footers are authoritative
- Page-in-section counters validate section boundaries

---

### Section Numbering Conventions

**MasterFormat uses spaces:**
- ✅ Correct: `23 82 16`
- ❌ Wrong: `23-82-16`, `238216`, `23.82.16`

**Leading zeros optional for divisions <10:**
- ✅ Both valid: `01 00 00`, `1 00 00`
- Engine normalizes to 2-digit format: `01 00 00`

**Section ranges:**
- Divisions 00-01: Administrative
- Divisions 02-19: Construction/building envelope
- Divisions 21-29: Mechanical/electrical/plumbing (MEP)
- Divisions 31-33: Site work

---

## Drawings Standards

### Unified Discipline System (UDS)

**Single-letter discipline designators:**

| Code | Discipline | Full Name | Typical Content | Sort Order |
|------|-----------|-----------|-----------------|------------|
| **G** | General | General | Cover sheets, key plans, legends | 10 |
| **H** | Hazmat | Hazardous Materials | Asbestos, lead abatement | 20 |
| **C** | Civil | Civil Engineering | Site plans, grading, utilities | 30 |
| **L** | Landscape | Landscape Architecture | Planting plans, irrigation | 40 |
| **A** | Architectural | Architecture | Floor plans, elevations, details | 50 |
| **I** | Interiors | Interior Design | Finish plans, furniture layouts | 60 |
| **S** | Structural | Structural Engineering | Foundation, framing, structural details | 65 |
| **M** | Mechanical | Mechanical | HVAC plans, ductwork, piping | 70 |
| **P** | Plumbing | Plumbing | Domestic water, sanitary, storm drainage | 80 |
| **E** | Electrical | Electrical | Power, lighting, panel schedules | 90 |
| **FA** | Fire Alarm | Fire Alarm | Fire detection, notification, often integrated with electrical | 92 |
| **FP** | Fire Protection | Fire Protection | Fire sprinklers, standpipes, suppression systems | 95 |
| **T** | Telecom | Telecommunications | Voice/data, CCTV, security | 100 |
| **V** | Vertical Transportation | Elevators/Escalators | Elevator plans, equipment details | 105 |

**Note:** Civil (C) and Controls (C) share the same letter—see disambiguation below.

---

### Common Multi-Letter Aliases

**Industry-standard aliases for disciplines:**

| Alias | Maps To | Discipline | Confidence |
|-------|---------|------------|------------|
| **FA** | FA | Fire Alarm | 0.95 |
| **FP** | FP | Fire Protection | 0.95 |
| **SPRK** | FP | Sprinkler | 0.95 |
| **DDC** | Controls | Direct Digital Controls | 0.95 |
| **BAS** | Controls | Building Automation System | 0.95 |
| **ATC** | Controls | Automatic Temperature Control | 0.95 |
| **CTRL** | Controls | Controls | 0.95 |
| **SPRK** | F | Sprinkler | 0.95 |
| **SITE** | C | Civil (Site) | 0.95 |
| **HVAC** | M | Mechanical (HVAC) | 0.95 |
| **MECH** | M | Mechanical | 0.95 |
| **ELEC** | E | Electrical | 0.95 |
| **ARCH** | A | Architectural | 0.95 |
| **STRU** | S | Structural | 0.95 |

---

### Disambiguation: Civil vs Controls

**"C" is ambiguous—it can mean Civil or Controls.**

**Heuristic keywords:**

**Civil indicators:**
- "Civil", "Site", "Earthwork", "Grading", "Paving", "Utility", "Storm", "Sanitary"

**Controls indicators:**
- "Controls", "DDC", "BAS", "ATC", "Automation", "BMS", "HVAC Controls", "Sequence"

**Algorithm:**
1. Extract sheet ID prefix (e.g., "C1-01" → "C")
2. If title contains "Controls", "DDC", "BAS", "Automation" → Controls (confidence 0.85)
3. If title contains "Civil", "Site", "Earthwork", "Grading" → Civil (confidence 0.85)
4. If no title or ambiguous → Civil (default, lower sort order, confidence 0.72)

**Examples:**
```
Sheet: C1-01, Title: "Civil & Demolition Plan"
→ Civil (confidence 0.85, basis: HEURISTIC)

Sheet: C1-01, Title: "BAS Control System Layout"
→ Controls (confidence 0.85, basis: HEURISTIC)

Sheet: C1-01, Title: "" (no title)
→ Civil (confidence 0.72, basis: HEURISTIC, reason: "ambiguous, default to Civil")
```

---

### Sheet Numbering Conventions

**Standard format:** `[Discipline][Level]-[Sequence]`

**Examples:**
```
G0.01       → General, Level 0 (cover sheet), Sheet 01
A1.01       → Architectural, Level 1, Sheet 01
M1.11       → Mechanical, Level 1, Sheet 11
E2.21C      → Electrical, Level 2, Sheet 21, Revision C
FP-01       → Fire Protection, Sheet 01 (level omitted)
DDC-03      → Controls (DDC), Sheet 03
```

**Level conventions:**
- `0` = Cover sheet, site plan, overall plans
- `1` = First floor / Ground level
- `2` = Second floor
- `B1` = Basement level 1
- `R` = Roof

**Sequence:**
- Usually 2 digits: `01`, `02`, `11`, `21`
- May include revision letter: `A`, `B`, `C`

**Sorting:**
1. Sort by discipline (G → H → C → L → A → I → S → M → P → E → F → T → V)
2. Within discipline, sort by level (0 → 1 → 2 → ... → R)
3. Within level, sort by sequence (01 → 02 → 03)

---

### Title Block Structure

**Typical title block location:** Lower-right corner of sheet

**Standard title block fields:**

| Field | Description | Example |
|-------|-------------|---------|
| **Project Name** | Building/facility name | "Lake Highlands High School" |
| **Project Number** | Client/firm project ID | "RWB Project No. 25063.00" |
| **Sheet Title** | Description of sheet content | "REFLECTED CEILING PLAN - LEVEL 1" |
| **Sheet Number** | Discipline + level + sequence | "M1.11" |
| **Date** | Issue date or revision date | "2025-10-01" |
| **Drawn By** | Initials of drafter | "JD" |
| **Checked By** | Initials of checker | "RM" |
| **Revision** | Revision letter/number | "Rev 3" |
| **Firm Name** | Engineering firm | "RWB Consulting Engineers" |
| **Professional Seal** | PE/RA stamp | (graphical seal) |

**Sheet number may also appear in:**
- Footer (center or right)
- Header (less common)
- Revision block (references which sheets revised)

---

### Drawing Chrome (Furniture)

**Chrome regions to exclude from content:**

**Title block:** Lower-right corner, typically 6" × 4" (varies)

**Revision block:** Upper-right or triangular corner
```
Rev | Description        | Date       | By
----+--------------------+------------+----
 3  | Revised ceiling    | 2025-10-17 | JD
 2  | Added exhaust fans | 2025-09-15 | JD
 1  | For permit         | 2025-08-01 | RM
```

**General notes:** Text blocks with project-wide standards
- "All dimensions are to face of finish unless noted"
- "Contractor to verify all dimensions in field"

**Legends:** Symbol definitions
- Ductwork symbols
- Equipment symbols
- Electrical device symbols

**Sheet footer:** Sheet number, project name (sometimes)

---

### Equipment Schedules (Tables)

**Schedule characteristics:**

**Dense tabular data:**
- 10-20+ columns typical
- Multiple tables per sheet common
- Rotated text in headers (column names vertical)
- Merged cells for titles

**Common schedule types:**
- **Unit Ventilators:** Model, CFM, heating/cooling capacity, electrical
- **Roof-Top Units (RTUs):** Model, CFM, tonnage, power, gas
- **Air Handling Units (AHUs):** CFM, static pressure, filters, coils
- **Exhaust Fans:** CFM, static, motor HP
- **Pumps:** GPM, head, motor HP, efficiency
- **Panels:** Circuit breakers, loads, phases

**Example schedule structure:**
```
UNIT VENTILATOR SCHEDULE - UNIT B

Tag | Model      | CFM  | Heat BTU | Cool Tons | Voltage | Notes
----+------------+------+----------+-----------+---------+-------
UV-1| Trane ECVF | 1200 | 45,000   | 3.0       | 208/3   | Return air damper
UV-2| Trane ECVF | 1500 | 60,000   | 4.0       | 208/3   | CO2 sensor
```

**Parsing challenges:**
- Multi-table detection (where one ends, another begins)
- Header detection (first row vs repeated headers)
- Merged cells (schedule titles span columns)
- Rotated text (column headers vertical)

---

## Submittals Standards

### Submittal Structure

**Typical organization:**

```
Cover sheet
  - Project info (name, number, date)
  - Contractor info
  - Submittal number (e.g., "Submittal 23.1 - Unit Ventilators")
  
Unit pages (one per equipment tag)
  - Equipment tag (e.g., "UV-1")
  - Model number
  - Performance data table
  - Dimension diagram
  - Electrical data
  - Sound data
  
Manufacturer literature
  - Product brochures
  - Installation guides
  - Warranty information
```

---

### Equipment Tag Conventions

**Format varies by discipline, but follows patterns:**

**HVAC:**
- `AHU-1`, `AHU-2` = Air Handling Units
- `RTU-1`, `RTU-2` = Roof-Top Units
- `UV-1`, `UV-2` = Unit Ventilators
- `EF-1`, `EF-2` = Exhaust Fans
- `HWP-1`, `HWP-2` = Hot Water Pumps
- `CHWP-1`, `CHWP-2` = Chilled Water Pumps
- `FC-1`, `FC-2` = Fan Coil Units

**Electrical:**
- `LP-1`, `LP-2` = Lighting Panels
- `PP-1`, `PP-2` = Power Panels
- `MCC-1` = Motor Control Center
- `XFMR-1` = Transformer

**Plumbing:**
- `WH-1` = Water Heater
- `DP-1`, `DP-2` = Domestic Water Pumps
- `SP-1`, `SP-2` = Sump Pumps

**Fire Protection:**
- `FSP-1` = Fire Sprinkler Panel
- `FP-1`, `FP-2` = Fire Pumps

**Pattern recognition:**
- Tag prefix = Equipment type (AHU, RTU, UV, etc.)
- Numeric suffix = Unit number
- Tags are unique per project

---

### Submittal Data Fields

**Common fields to extract:**

**Header fields:**
- Submittal number
- Date
- Project name/number
- Equipment type
- Manufacturer

**Per-unit fields:**
- Tag (e.g., "AHU-1")
- Model number
- Quantity
- Cooling capacity (tons or BTU)
- Heating capacity (BTU)
- Airflow (CFM)
- Static pressure (in. w.g.)
- Voltage / Phase
- Full Load Amps (FLA)
- Sound level (dBA)
- Dimensions (L × W × H)
- Weight (lbs)

**Tidy data format (CSV):**
```csv
submittal_number,tag,model,cfm,cooling_tons,heating_btu,voltage,fla
23.1,AHU-1,Trane TAM150,15000,12.5,150000,480/3,45.2
23.1,AHU-2,Trane TAM120,12000,10.0,120000,480/3,38.5
```

---

## Common AEC Terminology

### Abbreviations

**General:**
| Abbreviation | Meaning |
|--------------|---------|
| **AEC** | Architecture, Engineering, Construction |
| **AHJ** | Authority Having Jurisdiction |
| **BIM** | Building Information Modeling |
| **BOM** | Bill of Materials |
| **CSI** | Construction Specifications Institute |
| **IOM** | Installation & Operations Manual |
| **ISD** | Independent School District |
| **P&ID** | Process & Identification Diagram |
| **PID** | Proportional, Integral, Derivative |
| **TOC** | Table of Contents |
| **PO** | Purchase Order |
| **CO** | Change Order |

**Service Types:**
| Abbreviation | Meaning |
|--------------|---------|
| **SA** | Supply Air |
| **DA** | Discharge Air |
| **RA** | Return Air |
| **OA** | Outdoor Air |
| **MA** | Mixed Air |
| **EA** | Exhaust Air |
| **RfA** | Relief Air |
| **CHW** | Chilled Water |
| **CW** | Condenser Water |
| **HW** | Hot Water (Heating) |
| **DHW** | Domestic Hot Water |
| **CD** | Cold Deck |
| **HD** | Hot Deck |
| **LTG** | Lighting |
| **FA** | Fire Alarm |

**Equipment (HVAC):**
| Abbreviation | Meaning |
|--------------|---------|
| **AHU** | Air Handling Unit |
| **RTU** | Roof-Top Unit |
| **ERV/ERU** | Energy Recovery Ventilator/Unit |
| **HRV** | Heat Recovery Ventilator |
| **FCU** | Fan-Coil Unit |
| **VAV** | Variable Air Volume (Terminal Unit) |
| **FPB** | Fan-Powered Box (VAV) |
| **MAU** | Makeup Air Unit |
| **DOAS** | Dedicated Outdoor Air System |
| **EF** | Exhaust Fan |
| **RLF** | Relief Fan |

**Equipment (Hydronic):**
| Abbreviation | Meaning |
|--------------|---------|
| **CH** | Chiller |
| **CT** | Cooling Tower |
| **B** | Boiler |
| **HX** | Heat Exchanger |
| **P** | Pump |
| **CHWP** | Chilled Water Pump |
| **CHP** | Chiller Pump |
| **CWP** | Condenser Water Pump |
| **HWP** | Hot Water Pump (Heating) |
| **CP** | Circulating Pump |
| **BP** | Booster Pump |
| **FP** | Fire Pump |

**Equipment (Refrigeration/Comfort Cooling):**
| Abbreviation | Meaning |
|--------------|---------|
| **SS** | Split System |
| **AC** | Air Conditioning Unit (Indoor) |
| **CU** | Condensing Unit (Outdoor) |
| **HP** | Heat Pump |
| **WSHP** | Water-Source Heat Pump |

**BAS/Controls Devices:**
| Abbreviation | Meaning |
|--------------|---------|
| **BAS** | Building Automation System |
| **DDC** | Direct Digital Controls |
| **C** | Controller |
| **CX** | Controller Expander |
| **TS** | Temperature Sensor/Thermistor |
| **RS** | Room Sensor |
| **OPS** | Outdoor Pressure Sensor |
| **OATH** | Outdoor Air Temp/Humidity Sensor |
| **DTHC** | Duct Temp/Humid/CO2 Sensor |
| **V** | Valve |
| **VA** | Valve Actuator |
| **D** | Damper |
| **DA** | Damper Actuator |
| **SD** | Smoke Damper |
| **FSD** | Fire/Smoke Damper |
| **ACT** | Actuator |
| **CS** | Current Switch |
| **R** | Relay |
| **FS** | Float Switch |
| **SMK** | Smoke Detector |
| **LTCO** | Low Temperature Cutout |
| **DSP** | Duct Static Pressure Sensor |
| **BSP** | Building Static Pressure Sensor |
| **HSP** | High Static Pressure Switch |
| **LSP** | Low Static Pressure Switch |
| **DPS** | Differential Pressure Switch |
| **AFMS** | Airflow Measurement Station |
| **PT** | Pitot Tube |

**Electrical:**
| Abbreviation | Meaning |
|--------------|---------|
| **VFD** | Variable Frequency Drive |
| **CTC** | Contactor |
| **SSR** | Solid State Relay |
| **CKT** | Circuit |
| **PAN** | Panel |
| **LCP** | Lighting Control Panel |
| **UPS** | Uninterruptible Power Supply |
| **XT** | Transformer |
| **TB** | Terminal Block |
| **ENC** | Enclosure |
| **OFSA** | Outlet/Switch/Fuse Assembly |
| **AWG** | American Wire Gauge |
| **FLA** | Full Load Amps |
| **MCA** | Minimum Circuit Ampacity |

**Wire Colors:**
| Abbreviation | Color |
|--------------|-------|
| **R** | Red |
| **BK** | Black |
| **W** | White |
| **G** | Green |
| **BL** | Blue |
| **BR** | Brown |
| **Y** | Yellow |
| **P** | Pink |
| **PR** | Purple |

**Valve/Damper Positions:**
| Abbreviation | Meaning |
|--------------|---------|
| **NO** | Normally Open |
| **NC** | Normally Closed |
| **FO** | Fail Open |
| **FC** | Fail Closed |
| **C** | Common |

**Personnel:**
| Abbreviation | Meaning |
|--------------|---------|
| **A/E** | Architect/Engineer |
| **M/E** | Mechanical Engineer (Specifying) |
| **GC** | General Contractor |
| **MC** | Mechanical Contractor |
| **PC** | Plumbing Contractor |
| **EC** | Electrical Contractor |
| **MEP** | Mechanical/Electrical/Plumbing (Contractor or Disciplines) |
| **PM** | Project Manager |
| **AE** | Applications Engineer |
| **SUB** | Subcontractor |

**Communication Protocols:**
| Abbreviation | Meaning |
|--------------|---------|
| **IP** | Internet Protocol |
| **MS/TP** | Master/Slave Token-Passing (BACnet) |
| **I/O** | Input/Output |

---

### Units

| Unit | Meaning |
|------|---------|
| **CFM** | Cubic Feet per Minute (airflow) |
| **GPM** | Gallons Per Minute (water flow) |
| **BTU/hr** | British Thermal Units per Hour (heating/cooling capacity) |
| **Tons** | Cooling capacity (1 ton = 12,000 BTU/hr) |
| **kW** | Kilowatts (power) |
| **HP** | Horsepower (1 HP ≈ 0.746 kW) |
| **in. w.g.** | Inches of Water Gauge (pressure) |
| **psi** | Pounds per Square Inch (pressure) |
| **dBA** | Decibels A-weighted (sound level) |
| **°F** | Degrees Fahrenheit (temperature) |

---

### Project Phases

| Phase | Description |
|-------|-------------|
| **Schematic Design (SD)** | Conceptual design, ~15% complete |
| **Design Development (DD)** | Detailed design, ~35% complete |
| **Construction Documents (CD)** | Final design, 100% complete, ready for bid |
| **Bidding** | Contractor selection |
| **Construction Administration (CA)** | Oversight during construction |
| **Closeout** | Final inspections, punch list, as-builts |

---

### Document Types

| Type | Description |
|------|-------------|
| **Addendum** | Changes to bid documents before bidding |
| **Change Order** | Changes to contract during construction |
| **RFI** | Request for Information (contractor questions) |
| **Submittal** | Product data for review |
| **Shop Drawing** | Fabrication drawings for review |
| **As-Built** | Final documentation of what was built |
| **O&M Manual** | Operation & Maintenance manual |

---

## Confidence Scoring Reference

### Basis Types

| Basis | Description | Typical Confidence |
|-------|-------------|-------------------|
| **UDS** | Exact match to UDS single-letter designator | 1.0 |
| **MASTERFORMAT** | Exact match to MasterFormat division | 1.0 |
| **ALIAS** | Match to known multi-letter alias | 0.95 |
| **HEURISTIC** | Keyword-based disambiguation | 0.75–0.85 |
| **UNKNOWN** | No match found | 0.2 |

---

### Confidence Thresholds

| Range | Action | Reason |
|-------|--------|--------|
| **≥0.95** | Auto-apply | High confidence (exact match) |
| **0.80–0.95** | Auto-apply + flag | Medium confidence (alias or strong heuristic) |
| **0.70–0.80** | Apply + review | Low-medium confidence (weak heuristic) |
| **<0.70** | Escalate to human | Low confidence (ambiguous or unknown) |

---

### Example Confidence Calculations

**Drawings Discipline:**
```
Sheet ID: "M1-01", Title: "Mechanical Plan"
→ Designator: M (UDS exact match)
→ Confidence: 1.0, Basis: UDS

Sheet ID: "FP-01", Title: "Fire Protection Plan"
→ Alias: FP (known alias for Fire Protection)
→ Confidence: 0.95, Basis: ALIAS

Sheet ID: "C1-01", Title: "Civil & Demolition"
→ Designator: C (ambiguous), Title contains "Civil"
→ Confidence: 0.85, Basis: HEURISTIC

Sheet ID: "C1-01", Title: "" (no title)
→ Designator: C (ambiguous), no title
→ Confidence: 0.72, Basis: HEURISTIC (default to Civil)

Sheet ID: "ZZ-01", Title: "Unknown System"
→ No match
→ Confidence: 0.2, Basis: UNKNOWN
```

**Specs MasterFormat:**
```
Section ID: "23 82 16"
→ Division: 23 (HVAC - known division)
→ Confidence: 1.0, Basis: MASTERFORMAT

Section ID: "99 00 00"
→ Division: 99 (unknown division, but valid format)
→ Confidence: 0.7, Basis: UNKNOWN

Section ID: "23 82 ABC" (invalid format)
→ Confidence: 0.2, Basis: UNKNOWN
```

---

## Quick Reference

### Spec Footer Parsing
```
Input:  "2025-10-01    23 82 16 – Heating Water Coils - Page 2 of 3"
Parse:  Date=2025-10-01, Section=23 82 16, Title=Heating Water Coils, Page=2/3
```

### Drawing Sheet ID Parsing
```
Input:  "M1.11A"
Parse:  Discipline=M (Mechanical), Level=1, Sequence=11, Revision=A
```

### Equipment Tag Parsing
```
Input:  "AHU-15"
Parse:  Type=AHU (Air Handling Unit), Number=15
```

### Submittal Number Parsing
```
Input:  "Submittal 23.1 - Unit Ventilators"
Parse:  Division=23 (HVAC), Submittal=1, Description=Unit Ventilators
```

---

## Appendix: Sort Orders

### Drawings Discipline Sort Order

```
10  - General (G)
20  - Hazmat (H)
30  - Civil (C)
40  - Landscape (L)
50  - Architectural (A)
60  - Interiors (I)
65  - Structural (S)
70  - Mechanical (M)
80  - Plumbing (P)
90  - Electrical (E)
92  - Fire Alarm (FA)
95  - Fire Protection (FP)
100 - Telecom (T)
105 - Vertical Transportation (V)
110 - Controls (CTRL, DDC, BAS, ATC)
999 - Unknown (?)
```

**Note:** Fire Alarm (FA) typically designed by electrical engineers, hence close to Electrical. Fire Protection (FP) is separate mechanical/plumbing discipline for sprinklers and suppression systems.

---

### Specs Division Sort Order

```
Sort by division number (00 → 01 → 02 → ... → 33)
Within division, sort by section number (23 00 00 → 23 05 00 → 23 82 16)
```

**Typical order in spec book:**
```
Division 00 - Procurement
Division 01 - General Requirements
Division 02 - Existing Conditions
...
Division 23 - HVAC
Division 26 - Electrical
Division 33 - Utilities
```

---

## Revision History

| Version | Date | Changes |
|---------|------|---------|
| 4.0.0 | 2026-01-21 | Initial AEC standards (UDS, MasterFormat) |
| 4.2.0 | 2026-01-23 | **Aligned with Master Plan V4.2.** Added: (1) Spec outline structure (3-part format, article hierarchy), (2) Footer format patterns with examples, (3) Title block structure reference, (4) Sheet numbering conventions, (5) Equipment tag patterns, (6) Common AEC terminology and abbreviations, (7) Equipment schedule characteristics, (8) Submittal structure and data fields, (9) Quick reference section for parsing patterns. Simplified: Removed governance (moved to separate doc), removed code examples (that's DEV_STANDARDS), focused on domain reference only. |
| 4.2.1 | 2026-01-23 | **Discipline sort order fix and abbreviations expansion.** Changed: (1) Separated Fire Alarm (FA, order 92) from Fire Protection (FP, order 95) - FA typically electrical engineering, FP mechanical/plumbing, (2) Added comprehensive abbreviations from industry practice (service types, equipment, BAS devices, wire colors, valve positions, personnel, protocols). |

---

**Status:** ✅ ACTIVE  
**Owner:** HLLMR LLC  
**Last Updated:** January 23, 2026  
**Version:** 4.2.1

---

**End of AEC_STANDARDS Document**
