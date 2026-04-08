# CLI Reference — `backend-cli`

`backend-cli` is the deterministic PDF processing command-line interface.  Every
invocation produces a [`WorkflowResponse`](#output-format) JSON object on stdout
and appends a structured audit bundle entry to the `--audit-dir` directory.

---

## Global flags

| Flag | Default | Description |
|---|---|---|
| `--audit-dir <DIR>` | `audit_output` | Directory for per-session audit JSON files (created if absent). |

---

## Output format

Every subcommand prints a JSON object to stdout:

```json
{
  "request_id": "req-1-<timestamp_ms>",
  "session_id": "session-<timestamp_ms>",
  "operation_id": "op-1-<timestamp_ms>",
  "contracts_version": "0.1.0",
  "result": {
    "status": "succeeded",
    "summary": "human-readable description of outcome",
    "warnings": [],
    "error_code": null,
    "output_artifacts": []
  },
  "audit_events": [ ... ]
}
```

**`result.status`** is one of:

| Value | Meaning |
|---|---|
| `"succeeded"` | All operations completed without errors. |
| `"succeeded_with_warnings"` | Completed; inspect `warnings` for non-fatal issues. |
| `"failed"` | Operation could not complete; see `summary` and `error_code`. |

**Exit codes:**

| Code | Condition |
|---|---|
| `0` | Normal exit — inspect `result.status` for the operation outcome. |
| `1` | Unrecoverable infrastructure error (e.g. cannot write audit directory or session bundle). |

---

## Subcommands

### `extract`

Extract a [`LayoutTranscript`] from a PDF file.  The transcript contains all
text spans with their normalised bounding boxes and font metadata.

```
backend-cli extract --input <PDF> [--output <JSON>] [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <PDF>` | yes | Path to the source PDF. |
| `--output <JSON>` | no | Path for the output transcript JSON. |
| `--dry-run` | no | Validate arguments only; skip all processing. |

**Output:** `LayoutTranscript` JSON (pages → spans with `BBox`, font metrics, page dimensions).

**Example:**

```bash
backend-cli extract --input spec.pdf --output transcript.json
```

---

### `segment`

Segment a `LayoutTranscript` JSON into CSI MasterFormat document sections using
the footer-oracle algorithm.  Produces a `SegmentIndex` with section boundaries,
coverage statistics, and extracted chrome metadata (project ID, firm, dates).

```
backend-cli segment --input <TRANSCRIPT_JSON> [--output <JSON>] [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <TRANSCRIPT_JSON>` | yes | Path to a transcript JSON (produced by `extract`). |
| `--output <JSON>` | no | Path for the output `SegmentIndex` JSON. |
| `--dry-run` | no | Validate arguments only; skip processing. |

**Output:** `SegmentIndex` JSON — `sections[]` with `section_id`, `section_title`,
`start_page`, `end_page`, `confidence`; `coverage` with `coverage_ratio` and
`pages_missing_footer`; `chrome_metadata` with project-level strings.

**Example:**

```bash
backend-cli segment --input transcript.json --output segment-index.json
```

---

### `parse`

Run the full extract → segment → parse pipeline on a PDF, producing a hierarchical
AST.  Each section's AST follows the 5-level CSI outline structure
(PART / Article / Paragraph / Item / Sub-item).

```
backend-cli parse --input <PDF> [--output <JSON>] [--section <ID>] [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <PDF>` | yes | Path to the source PDF. |
| `--output <JSON>` | no | Path for the output `ParsedDocument` JSON. |
| `--section <ID>` | no | Only parse the named CSI section (e.g. `"23 82 16"`); all sections when omitted. |
| `--dry-run` | no | Validate arguments only; skip processing. |

**Output:** `ParsedDocument` JSON — `sections[]` each with a `SectionAst` tree,
`SectionLayout` geometry, and per-node `x_indent` values.

**Example:**

```bash
backend-cli parse --input spec.pdf --output parsed.json --section "23 82 16"
```

---

### `edit`

Apply surgical edit operations (insert / delete / replace) to a `ParsedDocument`
AST.  The entire operation batch is validated before any mutation is applied; a
single invalid path rejects the whole request.

```
backend-cli edit --input <AST_JSON> --operations <EDIT_JSON> [--output <JSON>] [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <AST_JSON>` | yes | Path to a `ParsedDocument` JSON (produced by `parse`). |
| `--operations <EDIT_JSON>` | yes | Path to an `EditRequest` JSON file. |
| `--output <JSON>` | no | Path for the output (edited) `ParsedDocument` JSON. |
| `--dry-run` | no | Validate arguments only; skip processing. |

**`EditRequest` JSON format:**

```json
{
  "description": "Addendum 3 HVAC updates",
  "operations": [
    { "op": "replace", "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.7", "A."] }, "new_text": "Revised text." },
    { "op": "delete",  "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.8", "B."] } },
    { "op": "insert_after", "path": { "section_id": "23 82 16", "markers": ["PART 2", "2.9"] },
      "new_node": { "tag": "paragraph", "level": 2, "text": "New paragraph text.", "children": [] } }
  ]
}
```

**Operation types:**

| `op` | Required fields | Effect |
|---|---|---|
| `replace` | `path`, `new_text` | Replace the node's text; structure and children are preserved. |
| `delete` | `path` | Remove the node; siblings are renumbered. |
| `insert_after` | `path`, `new_node` | Insert a sibling immediately after the target; renumber. |

**`NodePath`:** `section_id` (canonical CSI ID) + `markers` (ordered sequence of
`AstNode.marker` strings leading to the target node, e.g. `["PART 2", "2.7", "A."]`).

---

### `regenerate`

Render a `ParsedDocument` section to a PDF via headless Chrome.  Chrome must
be on `PATH` or `CHROME_PATH` must be set.

```
backend-cli regenerate --ast <AST_JSON> --chrome-metadata <META_JSON> --output <PDF>
                       [--section <ID>] [--font <FAMILY>] [--font-size <PT>] [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--ast <AST_JSON>` | yes | Path to a `ParsedDocument` JSON (produced by `parse` or `edit`). |
| `--chrome-metadata <META_JSON>` | yes | Path to a `SpecChromeMetadata` JSON for header/footer fields. |
| `--output <PDF>` | yes | Path for the output PDF file. |
| `--section <ID>` | no | CSI section ID to render; renders the first section when omitted. |
| `--font <FAMILY>` | no | CSS font-family override (default: `"Arial, sans-serif"`). |
| `--font-size <PT>` | no | Body font size in points (default: `10`). |
| `--dry-run` | no | Validate and build HTML only; skip Chrome render. |

**Output:** PDF file at `--output`.

---

### `stitch`

Replace one section's pages in the original PDF with a regenerated replacement PDF.
Bookmarks are re-routed; unchanged pages are validated by content hash.

```
backend-cli stitch --input <ORIG_PDF> --segment-index <SEGMENT_JSON>
                   --section <ID> --replacement <REPL_PDF> --output <OUT_PDF> [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <ORIG_PDF>` | yes | Path to the original source PDF. |
| `--segment-index <SEGMENT_JSON>` | yes | Path to a `SegmentIndex` JSON (produced by `segment`). |
| `--section <ID>` | yes | CSI section ID to replace (e.g. `"23 82 16"`). |
| `--replacement <REPL_PDF>` | yes | Path to the replacement PDF (produced by `regenerate`). |
| `--output <OUT_PDF>` | yes | Path for the stitched output PDF. |
| `--dry-run` | no | Validate and compute result without writing the output file. |

**Output:** Stitched PDF at `--output`.

---

### `apply-addendum` *(primary workflow)*

Run the full addendum application pipeline: extract → segment → parse each
section → apply edits → regenerate via Chrome → stitch in descending page order.
Partial success: failed sections are recorded; other sections still proceed.

```
backend-cli apply-addendum --original <PDF> --addendum <MANIFEST_JSON>
                           [--output <OUT_PDF>] [--audit-bundle <DIR>] [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--original <PDF>` | yes | Path to the original (source) spec PDF. |
| `--addendum <MANIFEST_JSON>` | yes | Path to an `AddendumManifest` JSON file. |
| `--output <OUT_PDF>` | no | Path for the revised output PDF (skipped on `--dry-run`). |
| `--audit-bundle <DIR>` | no | Directory for audit artifacts (`change-report.json`, `diagnostics.jsonl`, `metrics.json`). Created if absent. |
| `--dry-run` | no | Parse and edit all sections; skip Chrome render and PDF write. |

**Audit bundle contents** (when `--audit-bundle` is supplied):

| File | Description |
|---|---|
| `change-report.json` | Full `AddendumResult` — per-section outcomes, `pattern_db_version`. |
| `diagnostics.jsonl` | Line-delimited diagnostic events; schema header on line 1. |
| `metrics.json` | Executive summary roll-up (schema `"metrics/v1"`): page counts, coverage ratio, per-section parse + timing data. |

**Example:**

```bash
backend-cli apply-addendum \
  --original SPEC_RWB_LHHS_ALL_ORG.pdf \
  --addendum addendum-3.json \
  --output SPEC_RWB_LHHS_ALL_ORG_ADD3.pdf \
  --audit-bundle audit_output/add3
```

See [WORKFLOW_APPLYADDENDUM.md](WORKFLOW_APPLYADDENDUM.md) for a complete
end-to-end tutorial.

---

### `intake`

Stage 0 intake triage: detect and normalise rotated pages in a PDF.  Produces
a `NormalizedIntakeBundle` JSON describing rotation corrections per page.

```
backend-cli intake --input <PDF> [--output <JSON>] [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <PDF>` | yes | Path to the input PDF. |
| `--output <JSON>` | no | Path for the output `NormalizedIntakeBundle` JSON. |
| `--dry-run` | no | Detect issues and report them without modifying any files on disk. |

---

### `visualize`

Render per-page PNG overlays from a `LayoutTranscript` JSON for layout inspection.
Each span is drawn with its bounding box coloured by font weight.

```
backend-cli visualize --input <TRANSCRIPT_JSON> --output <DIR> [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <TRANSCRIPT_JSON>` | yes | Path to a transcript JSON (produced by `extract`). |
| `--output <DIR>` | yes | Output directory for overlay PNGs (created if absent). |
| `--dry-run` | no | Validate arguments only; skip rendering. |

---

### `visualize-segments`

Render per-page PNG overlays from a `SegmentIndex` JSON.  Section boundaries are
colour-coded; header / body / footer bands are annotated.

```
backend-cli visualize-segments --input <SEGMENT_JSON> --output <DIR> [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <SEGMENT_JSON>` | yes | Path to a `SegmentIndex` JSON (produced by `segment`). |
| `--output <DIR>` | yes | Output directory for overlay PNGs. |
| `--dry-run` | no | Validate arguments only; skip rendering. |

---

### `visualize-ast`

Render a `ParsedDocument` JSON as a collapsible HTML tree for outline inspection.

```
backend-cli visualize-ast --input <AST_JSON> --output <HTML> [--dry-run]
```

| Argument | Required | Description |
|---|---|---|
| `--input <AST_JSON>` | yes | Path to a `ParsedDocument` JSON (produced by `parse`). |
| `--output <HTML>` | yes | Output path for the rendered HTML file. |
| `--dry-run` | no | Validate arguments only; skip rendering. |

---

## Error codes

The `result.error_code` field is set when `result.status == "failed"`.

### `apply-addendum` error codes

| Code | Meaning | Most likely cause | What to check |
|---|---|---|---|
| `MISSING_MANIFEST_PATH` | `--addendum` was not provided. | CLI invocation is missing the flag. | Ensure `--addendum <JSON>` is in the command. |
| `MANIFEST_READ_ERROR` | Cannot read or parse the manifest JSON. | File does not exist, is not readable, or has invalid JSON syntax. | Check the file path and run `jq . addendum.json` to validate syntax. |
| `EMPTY_MANIFEST` | `sections` array is empty. | Manifest file was created without any section entries. | Add at least one entry to the `"sections"` array. |
| `AUDIT_DIR_CREATE_ERROR` | Cannot create the `--audit-bundle` directory. | Insufficient filesystem permissions or invalid path. | Check the path and permissions for the parent directory. |
| `ORCHESTRATOR_ERROR` | Extraction or segmentation failed catastrophically. | Source PDF is password-protected, corrupt, or a scanned image. | Verify the PDF opens in a viewer; check `result.summary` for the specific stage and error message. |

### Per-section failures

Failures at the parse, edit, render, or stitch stage for an individual section
are **not** `ORCHESTRATOR_ERROR` — they are recorded per-section in
`change-report.json` (`section_results[].status = "failed"`, `.reason`).  The
pipeline continues for other sections.

Common per-section failure causes:

| Stage | Failure cause | Resolution |
|---|---|---|
| **Parse** | Section ID not found in `SegmentIndex`. | Verify the `section_id` in the manifest matches the output of `segment`. |
| **Edit** | `NodePath.markers` sequence not found in the AST. | Run `parse` and inspect the AST with `visualize-ast` to find the correct marker chain. |
| **Render** | Chrome not found. | Set `CHROME_PATH` to the Chrome/Chromium executable, or install it on `PATH`. |
| **Render** | Chrome timed out or crashed. | Check `diagnostics.jsonl` for `stage: "Render"` events — the `chrome_stderr_tail` field contains the last Chrome stderr lines. |
| **Stitch** | Unchanged-page content-hash mismatch. | The source PDF was modified between `segment` and `stitch`. Re-run from `extract`. |

### `extract` / `segment` / `parse` errors (no error code — `summary` field)

| Stage | Typical `summary` prefix | Resolution |
|---|---|---|
| Extract | `extraction failed for '…'` | Source PDF is corrupt, encrypted, or a pure raster scan. |
| Segment | `segmentation failed` | Transcript is empty or internal inconsistency — re-run `extract`. |
| Parse | `no sections found matching filter '…'` | `--section` value does not match any segment ID; omit `--section` to parse all sections and inspect the IDs with `segment`. |
