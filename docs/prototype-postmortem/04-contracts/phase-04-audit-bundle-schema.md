# Phase 04 - AuditBundle Schema

**Document Type**: Data Schema / Wire Contract  
**Status**: Complete  
**Date**: 2026-03-19

---

## Purpose

Define the canonical audit artifact emitted by V4 workflows. This closes the Phase 8 contract gap around `AuditBundle` by turning the previously implied report shape into an explicit, versioned wire format.

`AuditBundle` is not just a merge report. It is the per-run evidence package for explainability, supportability, and deterministic verification.

---

## Contract Position

- Rust home: `crates/contracts::audit`
- Serialization: JSON
- Versioning: required top-level `schemaVersion`
- Scope: emitted by merge, split, bookmarks, specs-patch, and future submittal workflows

This schema extends the prototype `MergeReport` contract captured in `phase-04-output-formats-contract.md`. The prototype report remains a valid legacy source, but V4 should treat `AuditBundle` as the canonical audit output.

---

## Top-Level Shape

```json
{
  "schemaVersion": "v1",
  "bundleKind": "audit-bundle",
  "workflow": "merge",
  "runId": "01JPC5C3VBJ1KQ2M9X0Q7Y4N6R",
  "createdAt": "2026-03-19T18:42:11.120Z",
  "engine": {
    "name": "conset-pdf",
    "version": "4.0.0-alpha.1",
    "gitCommit": "abc1234",
    "profileVersion": "layout-v1",
    "privacyMode": "STRICT_STRUCTURE_ONLY"
  },
  "inputs": {
    "workflowInput": {},
    "documents": [],
    "layoutProfiles": []
  },
  "plan": {
    "mode": "replace+insert",
    "summary": {},
    "actions": []
  },
  "rows": [],
  "issues": [],
  "pageArtifacts": [],
  "timing": {},
  "outputs": [],
  "system": {},
  "notices": [],
  "warnings": []
}
```

---

## Required Fields

| Field | Type | Required | Notes |
|---|---|---|---|
| `schemaVersion` | `string` | Yes | Initial value: `v1`. Bump only for breaking JSON contract changes. |
| `bundleKind` | `"audit-bundle"` | Yes | Fixed discriminator. |
| `workflow` | enum | Yes | `merge`, `split`, `bookmarks`, `specs-patch`, `submittals`. |
| `runId` | `string` | Yes | Stable unique run identifier. Use ULID or UUIDv7. |
| `createdAt` | RFC 3339 string | Yes | Creation timestamp of the bundle. |
| `engine` | object | Yes | Runtime and policy metadata. |
| `inputs` | object | Yes | Source files, layout profiles, and normalized workflow input. |
| `plan` | object | Yes | Planned actions and execution summary. |
| `rows` | array | Yes | Per-row inventory/evidence entries. |
| `issues` | array | Yes | Diagnostics emitted during analysis or execution. |
| `pageArtifacts` | array | Yes | Page-level evidence, overlays, and span references. |
| `timing` | object | Yes | End-to-end and per-phase timing. |
| `outputs` | array | Yes | Produced files and checksums. |
| `system` | object | Yes | Host/system context captured for supportability. |
| `notices` | array | Yes | Informational notices; empty array allowed. |
| `warnings` | array | Yes | Non-fatal warning strings; empty array allowed. |

---

## Detailed Shape

### `engine`

```json
{
  "name": "conset-pdf",
  "version": "4.0.0-alpha.1",
  "gitCommit": "abc1234",
  "profileVersion": "layout-v1",
  "privacyMode": "STRICT_STRUCTURE_ONLY",
  "featureFlags": ["legacy_locator_disabled"]
}
```

### `inputs`

```json
{
  "workflowInput": {
    "docType": "drawings",
    "mode": "replace+insert",
    "strict": false,
    "dryRun": false
  },
  "documents": [
    {
      "role": "original",
      "path": "F:/docs/IFC.pdf",
      "sha256": "...",
      "pageCount": 120
    }
  ],
  "layoutProfiles": [
    {
      "name": "mech-drawings-v1",
      "path": "F:/layouts/mech.json",
      "sha256": "...",
      "profileType": "drawings"
    }
  ]
}
```

### `plan`

```json
{
  "mode": "replace+insert",
  "summary": {
    "replaced": 14,
    "inserted": 2,
    "unmatched": 1,
    "finalPages": 122
  },
  "actions": [
    {
      "actionId": "A-0001",
      "kind": "replace",
      "targetId": "M1-01",
      "sourceDocument": "addendum-1",
      "confidence": 0.97,
      "reason": "highest-confidence-match"
    }
  ]
}
```

### `rows`

Each row is the auditable record for one detected or synthesized inventory entry.

```json
{
  "rowId": "row-001",
  "normalizedId": "M1-01",
  "displayId": "M1-01",
  "title": "First Floor Mechanical Plan",
  "pageIndex": 12,
  "source": "roi",
  "confidence": 0.97,
  "bbox": { "x0": 0.73, "y0": 0.90, "x1": 0.94, "y1": 0.96 },
  "spans": ["span-101", "span-102"],
  "decision": {
    "action": "replace",
    "rationale": "matched original sheet M1-01 with higher-confidence addendum page"
  }
}
```

Required row fields:

| Field | Type | Notes |
|---|---|---|
| `rowId` | `string` | Stable row identifier. Must remain stable across re-analysis when the underlying logical row is unchanged. |
| `normalizedId` | `string` | Canonical sheet/section identifier. |
| `pageIndex` | `number` | 0-based page index. |
| `source` | `string` | Detection origin, for example `roi`, `footer`, `legacy-titleblock`, `manual-correction`. |
| `confidence` | `number` | `0.0` to `1.0`. |

### `issues`

```json
{
  "issueId": "ISS-0007",
  "severity": "warning",
  "code": "ROI_NO_PATTERN_MATCH",
  "message": "ROI text extracted but no valid sheet identifier matched.",
  "pageIndex": 0,
  "rowId": null,
  "evidence": {
    "spans": ["span-009"],
    "bbox": { "x0": 0.72, "y0": 0.88, "x1": 0.95, "y1": 0.96 }
  }
}
```

### `pageArtifacts`

This section closes the prototype gap around overlays and reproducible visual evidence.

```json
{
  "pageIndex": 12,
  "pageLabel": "13",
  "renderRef": "artifacts/pages/page-0013.png",
  "overlayRefs": [
    "artifacts/overlays/page-0013-roi.svg",
    "artifacts/overlays/page-0013-issues.svg"
  ],
  "spans": [
    {
      "id": "span-101",
      "text": "M1-01",
      "bbox": { "x0": 0.73, "y0": 0.90, "x1": 0.79, "y1": 0.94 }
    }
  ]
}
```

### `timing`

```json
{
  "analyzeMs": 1240,
  "applyCorrectionsMs": 9,
  "executeMs": 2187,
  "totalMs": 3436,
  "perPage": [
    { "pageIndex": 12, "analyzeMs": 14 }
  ]
}
```

### `outputs`

```json
[
  {
    "kind": "pdf",
    "path": "F:/output/IFC+Add4.pdf",
    "sha256": "...",
    "pageCount": 122
  },
  {
    "kind": "audit-json",
    "path": "F:/output/IFC+Add4.audit.json",
    "sha256": "..."
  }
]
```

### `system`

```json
{
  "os": "windows",
  "osVersion": "10.0.26100",
  "arch": "x86_64",
  "hostname": "WORKSTATION-01"
}
```

---

## Serialization Rules

1. JSON must be stable and deterministic.
2. Ordered arrays representing execution or output must be emitted in deterministic order.
3. Bounding boxes use the canonical Conset coordinate space: top-left origin, normalized `0.0` to `1.0`.
4. File references inside the bundle should be relative to the bundle root when possible.
5. Sensitive text handling must honor `privacyMode`. In `STRICT_STRUCTURE_ONLY`, literal span text may be redacted or tokenized.

---

## Minimum V4 Acceptance Rules

An `AuditBundle` is valid only if all of the following are true:

1. `schemaVersion`, `workflow`, `runId`, `engine`, `inputs`, `plan`, `rows`, `issues`, `timing`, and `outputs` are present.
2. Every modified page in the workflow has at least one `rows[]`, `issues[]`, or `pageArtifacts[]` entry referencing it.
3. Every emitted output file includes a SHA-256 checksum.
4. `FULL_TEXT_OPT_IN` is recorded explicitly in `engine.privacyMode`.
5. `rowId` and `issueId` values are unique within the bundle.

---

## Rust Mapping Guidance

Recommended Rust structure split:

- `AuditBundle`
- `AuditEngineInfo`
- `AuditInputs`
- `AuditDocumentRef`
- `AuditLayoutProfileRef`
- `AuditPlan`
- `AuditPlanAction`
- `AuditRow`
- `AuditDecision`
- `AuditIssue`
- `AuditEvidence`
- `AuditPageArtifact`
- `AuditSpanRef`
- `AuditTiming`
- `AuditPerPageTiming`
- `AuditOutputRef`
- `AuditSystemInfo`

Use `serde(rename_all = "camelCase")` throughout to preserve wire compatibility.

---

## Cross-References

- `phase-04-output-formats-contract.md`
- `phase-06-non-negotiables-rust-constraints.md`
- `phase-07-rust-port-primer.md`
- `phase-09-ops-telemetry-lessons.md`
- `phase-09-roi-coordinate-spec.md`