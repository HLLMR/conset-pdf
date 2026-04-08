# `apply-sheet-addendum` Workflow Tutorial

This guide walks through a complete end-to-end run — from a drawing-set PDF and
an addendum PDF through to a patched output PDF and an audit bundle you can inspect.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| `backend-cli` built | `cargo build --release --bin backend-cli` |
| `PDFIUM_LIB_PATH` set | Path to the directory containing `pdfium.dll` / `libpdfium.so`. |
| Original drawing-set PDF | The base drawing set, e.g. `DWG_RWB_LHHS_ALL_ORG.pdf`. |
| Addendum drawing-set PDF | The addendum containing replacement sheets, e.g. `DWG_RWB_LHHS_ALL_ADD2.pdf`. |

Check that the binary is usable:

```bash
./target/release/backend-cli --version
```

---

## Overview of the pipeline

```
Original Drawing PDF     Addendum Drawing PDF
         │                        │
         ▼                        ▼
    [extract]               [extract]
         │                        │
         ▼                        ▼
  LayoutTranscript          LayoutTranscript
         │                        │
         └────────┐ ┌─────────────┘
                  ▼ ▼
         DrawingAddendumManifest
                  │
                  ▼
        [apply-sheet-addendum]
          │   ├── index-drawing (both PDFs)
          │   ├── match sheet IDs
          │   ├── extract replacement pages (lopdf)
          │   └── stitch into original PDF
          │
          ▼
   Output Drawing PDF  +  Audit Bundle
        (change-report.json + metrics.json)
```

`apply-sheet-addendum` runs all of these steps in one command.

---

## Step 1 — Examine the original drawing set

First, extract and index the original drawing set to learn which sheet IDs are present:

```bash
backend-cli extract \
  --input DWG_RWB_LHHS_ALL_ORG.pdf \
  --output transcript-org.json

backend-cli index-drawing \
  --input transcript-org.json \
  --output drawing-index-org.json
```

Open `drawing-index-org.json` and look at `sheets[]`:

```json
{
  "schema_version": "1.0.0",
  "sheet_count": 59,
  "sheets": [
    {
      "sheet_id": "A-001",
      "start_page": 0,
      "end_page": 1,
      "page_count": 2,
      "chrome": {
        "sheet_id": "A-001",
        "sheet_title": "COVER SHEET",
        "discipline": "ARCH",
        "revision": "",
        "confidence": 0.85
      },
      "superseded_by": null,
      "is_schedule_sheet": false
    }
  ]
}
```

Note the `sheet_id` values — these are the canonical IDs used in the manifest.

---

## Step 2 — Examine the addendum PDF

Repeat for the addendum to confirm which sheets it contains:

```bash
backend-cli extract \
  --input DWG_RWB_LHHS_ALL_ADD2.pdf \
  --output transcript-add2.json

backend-cli index-drawing \
  --input transcript-add2.json \
  --output drawing-index-add2.json
```

Match the `sheet_id` values between both indexes to identify the replacements.

---

## Step 3 — Create a `DrawingAddendumManifest`

Create a JSON manifest file (e.g. `addendum-manifest.json`) specifying the replacement:

```json
{
  "schema_version": "1.0.0",
  "original_drawing_set": "DWG_RWB_LHHS_ALL_ORG.pdf",
  "addendum_pdf": "DWG_RWB_LHHS_ALL_ADD2.pdf",
  "output_path": "DWG_RWB_LHHS_ALL_ORG_patched.pdf",
  "audit_bundle_dir": "audit-output/",
  "dry_run": false,
  "sheets": [
    {
      "sheet_id": "M-201",
      "addendum_pages": null
    },
    {
      "sheet_id": "E-101",
      "addendum_pages": null
    }
  ]
}
```

When `addendum_pages` is `null`, the orchestrator locates the sheet by its `sheet_id`
in the addendum index and uses those pages automatically.

### `DrawingAddendumManifest` field reference

| Field | Type | Required | Description |
|---|---|---|---|
| `schema_version` | string | yes | Must be `"1.0.0"`. |
| `original_drawing_set` | string | yes | Path to the original drawing-set PDF. |
| `addendum_pdf` | string | yes | Path to the addendum PDF containing replacement sheets. |
| `output_path` | string | yes | Destination path for the patched output PDF. |
| `audit_bundle_dir` | string or null | no | Directory for `change-report.json` and `metrics.json`. |
| `dry_run` | bool | no | When `true`, validate sheet matching without writing any PDF. |
| `sheets` | array | yes | One `SheetReplaceSpec` per sheet to replace. |

### `SheetReplaceSpec` field reference

| Field | Type | Required | Description |
|---|---|---|---|
| `sheet_id` | string | yes | Sheet ID to replace (e.g. `"M-201"`); must exist in both drawing sets. |
| `addendum_pages` | array or null | no | Explicit 0-based page indices in the addendum PDF. When `null`, auto-detected from the addendum index. |

---

## Step 4 — Dry-run to validate sheet matching

Before overwriting any file, run with `--dry-run` to confirm that every sheet ID is
found in both PDFs:

```bash
backend-cli apply-sheet-addendum \
  --manifest addendum-manifest.json \
  --dry-run
```

A successful dry-run prints:

```json
{
  "result": {
    "status": "succeeded",
    "summary": "dry_run: argument validation passed — no replacement performed"
  }
}
```

If a sheet ID is not found in the addendum, the response will have `status: "failed"`
and `result.summary` will name the missing sheet ID.

---

## Step 5 — Production run

Remove `--dry-run` (or set `"dry_run": false` in the manifest) and run:

```bash
backend-cli apply-sheet-addendum \
  --manifest addendum-manifest.json \
  --audit-bundle audit-output/
```

The patched PDF is written to `output_path`.  A summary is printed to stdout:

```json
{
  "result": {
    "status": "succeeded",
    "summary": "Replaced 2 sheet(s) — M-201, E-101"
  }
}
```

---

## Step 6 — Review the audit bundle

The audit bundle directory contains:

### `change-report.json`

Records per-sheet replacement details and any rename events:

```json
{
  "schema_version": "1.0.0",
  "generated_at": "2026-04-08T12:00:00Z",
  "original_drawing_set": "DWG_RWB_LHHS_ALL_ORG.pdf",
  "addendum_pdf": "DWG_RWB_LHHS_ALL_ADD2.pdf",
  "output_path": "DWG_RWB_LHHS_ALL_ORG_patched.pdf",
  "sheet_results": [
    {
      "sheet_id": "M-201",
      "status": "replaced",
      "pages_replaced": 2
    }
  ],
  "renames": [
    {
      "original_sheet_id": "M-201",
      "addendum_sheet_id": "M-201",
      "original_title": "MECHANICAL EQUIPMENT PLAN — LEVEL 1",
      "addendum_title": "MECHANICAL EQUIPMENT PLAN — LEVEL 1 (REVISED)"
    }
  ]
}
```

### `metrics.json`

High-level counts for monitoring:

```json
{
  "schema": "metrics/v1",
  "total_pages_input": 120,
  "total_pages_output": 120,
  "sheets_detected": 59,
  "sheets_replaced": 2,
  "elapsed_ms": 4312
}
```

---

## Typical failure scenarios

| Symptom | Likely cause | Resolution |
|---|---|---|
| `MISSING_MANIFEST_PATH` | `--manifest` not provided. | Add `--manifest <JSON>` to the command. |
| `MANIFEST_READ_ERROR` | Manifest JSON is malformed. | Run `jq . addendum-manifest.json` to validate syntax. |
| `EMPTY_MANIFEST` | `sheets` array is empty. | Add at least one sheet entry to the manifest. |
| Sheet not found in dry-run | `sheet_id` in manifest doesn't appear in the addendum index. | Run `index-drawing` on the addendum and compare `sheet_id` values. Title-block detection may have failed on atypical sheets. |
| `ORCHESTRATOR_ERROR` | Extraction failed on one of the PDFs. | Verify both PDFs open in a viewer and are not password-protected. |

---

## Extracting schedule tables

After applying the addendum, extract any schedule sheets with:

```bash
backend-cli extract-schedules \
  --input transcript-org.json \
  --output schedules.json
```

See [CLI Reference — `extract-schedules`](CLI_REFERENCE.md#extract-schedules) for the output schema.
