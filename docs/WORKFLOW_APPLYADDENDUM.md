# `apply-addendum` Workflow Tutorial

This guide walks through a complete end-to-end run — from a real spec book and an
addendum manifest through to a revised PDF and an audit bundle you can inspect.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| `backend-cli` built | `cargo build --release --bin backend-cli` |
| `PDFIUM_LIB_PATH` set | Path to the directory containing `pdfium.dll` / `libpdfium.so`. |
| Chrome / Chromium | Required for actual PDF render; skip with `--dry-run` for validation only. |

Check that the binary is usable:

```bash
./target/release/backend-cli --version
```

---

## Overview of the pipeline

```
Source PDF
    │
    ▼
[extract]  →  LayoutTranscript  (text spans, bounding boxes, font metrics)
    │
    ▼
[segment]  →  SegmentIndex  (section boundaries, coverage stats, chrome metadata)
    │
    ▼
 For each section in the manifest:
    ├── [parse]        →  SectionAst (5-level CSI outline tree)
    ├── [edit]         →  mutated SectionAst
    ├── [regenerate]   →  replacement PDF page(s)  [skipped on --dry-run]
    └── [stitch]       →  splice into source PDF (last → first)
    │
    ▼
Revised Output PDF  +  Audit Bundle
```

`apply-addendum` runs all of these steps in one command.

---

## Step 1 — Inspect the source PDF's sections

Before authoring a manifest, confirm the canonical section IDs:

```bash
backend-cli extract --input SPEC_RWB_LHHS_ALL_ORG.pdf --output transcript.json
backend-cli segment --input transcript.json --output segment-index.json
```

Open `segment-index.json` and look at `sections[]`:

```json
{
  "sections": [
    { "section_id": "23 05 00", "section_title": "Common Work Results for HVAC",
      "start_page": 3, "end_page": 14, "confidence": 1.0 },
    { "section_id": "23 82 16", "section_title": "Air Coils",
      "start_page": 15, "end_page": 22, "confidence": 1.0 }
  ],
  "coverage": { "coverage_ratio": 0.965, "pages_missing_footer": 19 }
}
```

Use the values in `section_id` verbatim in your manifest.

---

## Step 2 — Inspect a section's AST

To know the exact `markers` for an edit path, parse the section and visualise
its AST:

```bash
backend-cli parse --input SPEC_RWB_LHHS_ALL_ORG.pdf \
                  --output parsed.json \
                  --section "23 82 16"

backend-cli visualize-ast --input parsed.json --output ast.html
```

Open `ast.html` in a browser.  Each node shows its path markers.

A typical outline looks like:

```
PART 2 — PRODUCTS
  └─ 2.7  AIR COILS, GENERAL
       ├─ A.  Performance: ...
       ├─ B.  Casing: ...
       └─ C.  Coil Circuiting: ...
```

The `markers` for node "B" under "2.7" is `["PART 2", "2.7", "B."]`.

---

## Step 3 — Author the manifest

Create `addendum-3.json`:

```json
{
  "description": "Addendum 3 — HVAC air coil spec update",
  "issue_date": "2025-10-17",
  "sections": [
    {
      "section_id": "23 82 16",
      "operations": [
        {
          "op": "replace",
          "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.7", "B."] },
          "new_text": "Casing: 18-gauge galvanised sheet steel with full drain pan."
        },
        {
          "op": "insert_after",
          "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.7", "C."] },
          "new_node": {
            "tag": "item",
            "level": 3,
            "text": "Access panels: removable without tools on both sides.",
            "children": []
          }
        }
      ]
    }
  ]
}
```

### `AddendumManifest` JSON reference

| Field | Type | Required | Description |
|---|---|---|---|
| `description` | `string` | no | Human-readable addendum label, copied to `change-report.json`. |
| `issue_date` | `string` | no | Issue date string (any displayable format); overrides date extracted from the source PDF. |
| `project_metadata` | `SpecChromeMetadata` | no | Project-level header/footer metadata override (see below). |
| `sections` | `SectionEditSpec[]` | **yes** | Ordered list of sections to patch. Must not be empty. |

### `SectionEditSpec` reference

| Field | Type | Required | Description |
|---|---|---|---|
| `section_id` | `string` | **yes** | Canonical CSI section ID (e.g. `"23 82 16"`). Must match a section in the `SegmentIndex`. |
| `operations` | `EditOperation[]` | no (defaults to `[]`) | Ordered edits to apply. An empty list re-renders the section unchanged — useful for a chrome metadata refresh. |
| `chrome_override` | `SpecChromeMetadata` | no | Per-section header/footer metadata override; takes precedence over `project_metadata`. |

### `EditOperation` reference

#### `replace`

```json
{ "op": "replace", "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.7", "B."] },
  "new_text": "Replacement text." }
```

Replaces `AstNode.text`; the node's structural position, marker, and children are preserved.

#### `delete`

```json
{ "op": "delete", "path": { "section_id": "23 82 16", "markers": ["PART 1", "1.2", "C."] } }
```

Removes the node and renumbers the remaining siblings in the same level.

#### `insert_after`

```json
{
  "op": "insert_after",
  "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.9"] },
  "new_node": { "tag": "paragraph", "level": 2, "text": "New sub-paragraph.", "children": [] }
}
```

Inserts a new sibling immediately after the target; all subsequent siblings are renumbered.
`new_node.tag` must match the sibling level. Valid tags: `"part"`, `"article"`,
`"paragraph"`, `"item"`, `"sub_item"`.

### `SpecChromeMetadata` reference

Controls the project-level information that appears in the page header and footer of
each rendered section.  All fields are optional; empty strings are treated as absent.

| Field | Example | Description |
|---|---|---|
| `project_id` | `"RWB-2025-001"` | Project number. |
| `project_name` | `"LHHS Renovation"` | Project name. |
| `firm_name` | `"RWBA Architects"` | Issuing firm. |
| `issue_date` | `"2025-10-17"` | Document issue date. |
| `section_id` | `"23 82 16"` | Auto-populated from segment index; rarely overridden. |
| `section_title` | `"Air Coils"` | Auto-populated from segment index. |

Merge order (lowest wins): extracted source-PDF metadata → `project_metadata`
→ per-section `chrome_override`.

---

## Step 4 — Dry-run validation

Always run with `--dry-run` first.  Dry-run parses and edits every section but
skips Chrome rendering and PDF write.

```bash
backend-cli apply-addendum \
  --original SPEC_RWB_LHHS_ALL_ORG.pdf \
  --addendum  addendum-3.json \
  --audit-bundle audit_output/add3-dry \
  --dry-run
```

Inspect the JSON response on stdout:

```json
{
  "result": {
    "status": "succeeded",
    "summary": "dry_run: apply-addendum (Addendum 3 — HVAC air coil spec update): 1/1 section(s) patched successfully"
  }
}
```

Check `audit_output/add3-dry/change-report.json`:

```json
{
  "total_sections": 1,
  "succeeded": 1,
  "failed": 0,
  "section_results": [
    { "section_id": "23 82 16", "status": "success",
      "pages_removed": 8, "pages_inserted": 0 }
  ],
  "pattern_db_version": "1.0.0"
}
```

Review `audit_output/add3-dry/metrics.json` for coverage and timing:

```json
{
  "schema": "metrics/v1",
  "total_pages_input": 571,
  "sections_detected": 89,
  "section_coverage_ratio": 0.965,
  "per_section": [
    {
      "section_id": "23 82 16",
      "parse_node_count": 312,
      "unclassified_count": 1,
      "unclassified_ratio": 0.003,
      "render_ms": null,
      "stitch_ms": 0
    }
  ]
}
```

`render_ms` is `null` on dry-run because Chrome is not invoked.

---

## Step 5 — Production run

```bash
backend-cli apply-addendum \
  --original SPEC_RWB_LHHS_ALL_ORG.pdf \
  --addendum  addendum-3.json \
  --output    SPEC_RWB_LHHS_ALL_ORG_REV3.pdf \
  --audit-bundle audit_output/add3
```

Verify the output:

```bash
# Check the status in the JSON response printed to stdout
backend-cli apply-addendum ... | jq .result.status

# Inspect per-section outcomes
cat audit_output/add3/change-report.json | jq '.section_results[] | {id: .section_id, status: .status}'

# Check performance timing
cat audit_output/add3/metrics.json | jq '{total_ms: .total_elapsed_ms, sections: .sections_patched}'
```

---

## Step 6 — Interpreting the audit bundle

### `change-report.json`

| Field | Description |
|---|---|
| `total_sections` | Number of sections in the manifest. |
| `succeeded` | Sections fully patched. |
| `failed` | Sections that encountered an error. |
| `section_results[]` | Per-section `status` (`"success"` or `"failed"`), `reason` (if failed), `pages_removed`, `pages_inserted`. |
| `pattern_db_version` | Version of the embedded pattern database used, e.g. `"1.0.0"`. |

### `diagnostics.jsonl`

Line 1 is a schema header:
```json
{"schema":"diagnostics/v1","pipeline_version":"0.1.0","generated_at":"2025-10-17T..."}
```

Subsequent lines are `DiagnosticEvent` objects tagged by `stage`:

| `stage` | Key fields |
|---|---|
| `"Extraction"` | `page_count`, `elapsed_ms` |
| `"Segmentation"` | `section_count`, `coverage_ratio`, `pages_missing_footer`, `sections[]` |
| `"Parse"` | `section_id`, `node_count`, `node_distribution` (`unclassified`, per-tag counts) |
| `"Edit"` | `section_id`, `ops_applied`, `ops_failed`, `failures[]` |
| `"Render"` | `section_id`, `elapsed_ms`, `chrome_version`, `chrome_stderr_tail` |
| `"Stitch"` | `section_id`, `elapsed_ms`, `pages_removed`, `pages_inserted`, `hash_mismatches` |

### `metrics.json`

Executive summary roll-up derived from `diagnostics.jsonl`.  Schema `"metrics/v1"`.

| Field | Description |
|---|---|
| `total_pages_input` | Page count of the source PDF. |
| `total_pages_output` | Estimated output page count (`input − removed + inserted`). |
| `sections_detected` | Sections found by the segmenter. |
| `sections_patched` | Sections fully processed (equal to `change-report.succeeded`). |
| `section_coverage_ratio` | Fraction of pages with a detected footer section ID (0–1). |
| `total_elapsed_ms` | Extraction + render + stitch time (excludes parse and edit). |
| `per_section[]` | Per-section: `section_id`, `parse_node_count`, `unclassified_count`, `unclassified_ratio`, `render_ms` (null on dry-run), `stitch_ms`. |

---

## Multi-section manifests

A manifest can patch any number of sections.  Operations within each section
are applied in declaration order; sections are processed in manifest order and
stitched last-to-first so that page offsets for earlier sections remain valid.

```json
{
  "description": "Addendum 4 — Full HVAC package",
  "issue_date": "2025-11-03",
  "project_metadata": {
    "firm_name": "RWBA Architects",
    "project_id": "RWB-2025-001"
  },
  "sections": [
    {
      "section_id": "23 05 00",
      "operations": [
        { "op": "replace",
          "path": { "section_id": "23 05 00", "markers": ["PART 1", "1.1", "A."] },
          "new_text": "Updated scope." }
      ]
    },
    {
      "section_id": "23 82 16",
      "operations": [
        { "op": "delete",
          "path": { "section_id": "23 82 16", "markers": ["PART 3", "3.2", "C."] } }
      ],
      "chrome_override": { "issue_date": "2025-11-03" }
    }
  ]
}
```

---

## Partial-success behaviour

If one section fails (e.g. Chrome is unavailable), the other sections still
proceed.  The `result.status` will be `"succeeded_with_warnings"`.

```json
{
  "result": {
    "status": "succeeded_with_warnings",
    "summary": "apply-addendum: 1/2 section(s) patched successfully; output: '...'"
  }
}
```

Inspect `change-report.json` for which section failed and why:

```json
"section_results": [
  { "section_id": "23 05 00", "status": "success", "pages_removed": 12, "pages_inserted": 11 },
  { "section_id": "23 82 16", "status": "failed",  "reason": "Chrome render failed: ..." }
]
```
