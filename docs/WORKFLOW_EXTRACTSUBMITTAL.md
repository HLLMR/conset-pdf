# Workflow: Extract Submittal Equipment Data

This workflow extracts structured equipment data from a mechanical/electrical
submittal PDF and exports it as a tidy `EquipmentDataset` (JSON or CSV).

**Subcommands used:** `extract` → `index-submittal` → `extract-submittal`

**Prerequisite:** The `PDFIUM_LIB_PATH` environment variable must point to
the directory containing the PDFium shared library.

---

## Step 1 — Extract transcript

```bash
backend-cli extract \
  --input SUB_Rsmd_TAS_AAON-RTU.pdf \
  --output transcript.json
```

This produces a `LayoutTranscript` JSON containing all text spans with their
normalised bounding boxes, font sizes, and page-level geometry.  The transcript
is the universal input for all subsequent pipeline stages.

**Expected:** `result.status = "succeeded"` and `transcript.json` written.

---

## Step 2 — Index submittal units

```bash
backend-cli index-submittal \
  --input transcript.json \
  --output submittal-index.json
```

The `SubmittalSegmentEngine` analyses the transcript and detects:
- Unit boundaries (cover page, per-unit page ranges)
- Unit tags (`RTU-1`, `AHU-1`, etc.)
- Manufacturer and model strings from header text
- Cover pages (flagged as `is_cover: true`)

**Expected:** `submittal-index.json` with `unit_count ≥ 1`.

**Inspect the index:**

```bash
jq '.units[] | {tag: .unit_tag, pages: "\(.start_page)-\(.end_page)", model: .model}' \
  submittal-index.json
```

---

## Step 3 — Extract equipment data

```bash
backend-cli extract-submittal \
  --input transcript.json \
  --index submittal-index.json \
  --output equipment-dataset.json \
  --format json
```

For each non-cover unit the engine:
1. Scopes the transcript to the unit's page range
2. Extracts key-value pairs (label / value) from header and dense-text regions
3. Extracts structured performance tables (cooling capacity, electrical data, etc.)
4. Assembles all records into a `TidyRow` list with provenance fields

**Expected:** `equipment-dataset.json` with `record_count ≥ 1`.

**Inspect record count:**

```bash
jq '{unit_count, record_count, units: [.unit_summaries[] | {tag: .unit_tag, records: .record_count}]}' \
  equipment-dataset.json
```

---

## Step 4 — Export CSV (optional)

```bash
backend-cli extract-submittal \
  --input transcript.json \
  --index submittal-index.json \
  --output equipment-dataset.csv \
  --format csv
```

The CSV output has exactly **14 columns** (RFC 4180 format):

```
packet_name,revision_id,item_tag,equipment_type,section,field,value_raw,value_num,unit,page,bbox,confidence,source,schema_version
```

Import directly into Excel, pandas, or a database.

---

## Step 5 — Review audit bundle (optional)

Include `--audit-bundle <DIR>` to write a per-run audit bundle:

```bash
backend-cli extract-submittal \
  --input transcript.json \
  --index submittal-index.json \
  --output equipment-dataset.json \
  --audit-bundle audit/
```

This writes:
- `audit/unit-report.json` — per-unit record counts, confidence, and warnings
- `audit/metrics.json` — total records extracted, elapsed time, schema version

---

## `TidyRow` field reference

| Field | Type | Description |
|---|---|---|
| `packet_name` | string | Submittal packet stem (from `SubmittalIndex.packet_name`). |
| `revision_id` | string | Addendum or revision identifier (empty if none). |
| `item_tag` | string | Equipment tag, e.g. `"RTU-1"`. |
| `equipment_type` | string | Equipment category (from `UnitEntry.item_type`), e.g. `"Rooftop Unit"`. |
| `section` | string | CSI section reference if present (empty otherwise). |
| `field` | string | Property name, e.g. `"Cooling Capacity"`. |
| `value_raw` | string | Raw extracted value string, e.g. `"25 tons"`. |
| `value_num` | number\|null | Numeric portion parsed from `value_raw`, or `null`. |
| `unit` | string | Unit portion parsed from `value_raw`, e.g. `"tons"`. |
| `page` | number | 0-based page index where the value was found. |
| `bbox` | string | Normalised bounding box `"x,y,w,h"` (top-left origin, [0,1] space). |
| `confidence` | number | Extraction confidence score [0.0, 1.0]. |
| `source` | string | `"kv"` for key-value pair, `"table"` for table row. |
| `schema_version` | string | Always `"1.0.0"`. |

---

## `EquipmentDataset` JSON schema reference

```json
{
  "schema_version": "1.0.0",
  "packet_name": "<stem of the source PDF>",
  "unit_count": <integer>,
  "record_count": <integer>,
  "unit_summaries": [
    {
      "unit_tag": "<string>",
      "record_count": <integer>,
      "kv_record_count": <integer>,
      "table_record_count": <integer>,
      "avg_confidence": <float 0.0-1.0>,
      "warnings": ["<string>", ...]
    }
  ],
  "records": [
    {
      "schema_version": "1.0.0",
      "packet_name": "<string>",
      "revision_id": "<string>",
      "item_tag": "<string>",
      "equipment_type": "<string>",
      "section": "<string>",
      "field": "<string>",
      "value_raw": "<string>",
      "value_num": <float|null>,
      "unit": "<string>",
      "page": <integer>,
      "bbox": "<x,y,w,h>",
      "confidence": <float>,
      "source": "kv" | "table"
    }
  ]
}
```

---

## Common issues

| Symptom | Likely cause | Remedy |
|---|---|---|
| `unit_count = 0` | Submittal is a raster scan with no extractable text. | Run `pattern-dev inspect --input PDF.pdf` to check span counts per page. If all pages have 0 spans, the PDF is image-only. |
| `record_count = 0` but `unit_count ≥ 1` | Unit detected but KV/table extraction found no structured data. | Check `audio/unit-report.json` for per-unit warnings.  Inspect the transcript for text quality. |
| `INVALID_INDEX` error | `--index` file was not produced by `index-submittal` or is corrupt. | Re-run `index-submittal` to regenerate the index file. |
| CSV import has 13 columns | Output file was produced before Sprint 10.4; the schema changed. | Re-run `extract-submittal` to regenerate. |

---

*See also:* [CLI_REFERENCE.md — Submittal subcommands](CLI_REFERENCE.md#submittal-subcommands-phase-10), [ARCHITECTURE.md](ARCHITECTURE.md)
