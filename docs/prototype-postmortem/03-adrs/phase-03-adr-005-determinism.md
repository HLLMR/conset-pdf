# Phase 03 - ADR-005 Determinism as a Design Invariant

## Status

Accepted as a hard architectural invariant. The prototype implements substantial determinism controls, but some controls are backend-dependent and should be tightened in Rust.

## Scope

Enumerate the concrete non-determinism sources the prototype actively addresses, identify residual non-determinism still tolerated in current code/tests, and define what Rust must preserve or strengthen.

## Source Evidence

- `packages/core/src/transcript/canonicalize.ts`
- `packages/core/src/transcript/factory.ts`
- `packages/core/src/transcript/types.ts`
- `packages/core/src/analyze/readingOrder.ts`
- `packages/core/src/utils/sort.ts`
- `packages/core/src/locators/roiSheetLocator.ts`
- `packages/core/src/core/planner.ts`
- `packages/core/src/core/applyPlan.ts`
- `packages/core/src/bookmarks/sidecar/bookmark-writer.py`
- `tests/transcript/determinism.test.ts`
- `docs/MASTER_PLAN_v4.md`

## Context

The product promise and V4 non-negotiables require deterministic behavior:

- same input + same profile + same engine version should produce the same outcome
- outputs must be auditable and explainable
- low-confidence paths must be explicit, not silent

Without explicit determinism controls, the prototype would drift run-to-run due to extractor variance, floating-point ordering effects, implicit tie-breaking, and metadata noise.

## Decision

Treat determinism as a first-class architectural constraint, not a testing convenience.

The implementation strategy is:

- normalize intermediate representations before consumers use them
- enforce explicit ordering and tie-break rules
- compute hashes from canonical data, excluding volatile fields
- avoid runtime randomness in decision paths
- preserve safe, deterministic output write boundaries

## Determinism Controls Implemented

### 1. Date-field exclusion from canonical hashes

`canonicalizeTranscript()` computes `contentHash` and `spanHash` from canonicalized page/span data and excludes `extractionDate` from hash inputs.

This directly removes one major non-determinism source: timestamp variation between runs.

### 2. Stable ordering of transcript spans

Transcript spans are canonicalized with explicit order rules:

- primary: y coordinate
- secondary: x coordinate
- fixed tolerance for near-equal values
- deterministic span ID regeneration after sorting

This gives downstream modules a stable structural basis instead of raw extractor iteration order.

### 3. Rotation/coordinate normalization

Canonicalization normalizes page rotation to `0` and enforces consistent bbox ordering (`x0<x1`, `y0<y1`).

This removes orientation-driven variance and prevents downstream logic from depending on extractor-specific coordinate conventions.

### 4. Deterministic sort/tie-break patterns across modules

The codebase uses explicit sort and tie-break logic in key flows:

- natural ID ordering in merge/split/assembly helpers
- deterministic score tie-breakers in ROI matching
- stable visual reading-order assembly rules
- deterministic standards comparators with tertiary tie-breaks

This reduces accidental non-determinism from unspecified array/object iteration order.

### 5. Deterministic planner-driven merge ordering

`planMerge()` produces an explicit ordered page plan, and `applyMergePlan()` copies pages in that order.

The merge pipeline therefore relies on deterministic planning outputs rather than heuristic runtime traversal at write time.

### 6. Deterministic final write boundary

Final merge output writes are routed through pikepdf passthrough and temp-output handling, which stabilizes output handling and avoids direct partial writes to target paths.

### 7. No active runtime randomness in core logic

Current core sources show no active `Math.random` or Python `random` usage in decision paths.

So for the prototype, "Python random seeding" is currently a preventive requirement rather than a control around existing random behavior.

## Validation Evidence in Tests

`tests/transcript/determinism.test.ts` explicitly asserts:

- `contentHash` stability across repeated extraction
- span count and ID stability
- `spanHash` stability
- bbox alignment within tolerance checks

The same test file also documents a critical caveat:

- PyMuPDF path is expected to be fully deterministic
- PDF.js fallback may show slight coordinate-level non-determinism, and tests relax strict hash equality in that mode

## Residual Non-Determinism and Gaps

### 1. Backend-dependent determinism quality

Determinism strength is not uniform across extractors:

- strong under PyMuPDF
- weaker under PDF.js fallback due to coordinate precision variance

This is explicitly acknowledged in tests and should be treated as a known prototype limitation.

### 2. Floating-point representation sensitivity in hashes

Canonical hash inputs currently use raw numeric string serialization of bbox/font metrics.

This is deterministic per runtime/output stream in many cases, but can remain sensitive to tiny extractor precision differences. There is no global quantization pass before hashing today.

### 3. Tolerance-based ordering can still be backend-sensitive

Several sort paths use tolerances to group "same line" behavior. That is necessary, but means tiny numeric drift can change group boundaries in edge cases if upstream extraction varies enough.

## Alternatives Rejected

### Accept probabilistic output with confidence-only reporting

Rejected because auditability and reproducibility are core product requirements.

### Push determinism to test harness only

Rejected because deterministic behavior must be designed into runtime data flow, not retrofitted only in tests.

### Keep volatile metadata in canonical hashes

Rejected because timestamps and other volatile fields destroy reproducibility guarantees.

## Rust Preservation Requirements

Rust must preserve and strengthen the prototype's determinism controls:

- canonical transcript/data normalization before downstream consumers
- explicit stable ordering and tie-break contracts at all decision points
- hash computation over canonical, quantized values with volatile fields excluded
- no runtime randomness in extraction/planning/selection logic
- deterministic write/overwrite semantics with temp-file safety

Rust should improve on prototype gaps by:

- enforcing numeric quantization policy before hash and tie-break comparisons
- minimizing fallback paths with weaker geometric determinism
- making determinism guarantees backend-agnostic, not extractor-conditional

## Prototype Invariants Worth Preserving

- deterministic hashes must exclude volatile run metadata
- span IDs should be derived from canonical order, not extractor-native order
- planner output order must be explicit and replayable
- sorting logic must include deterministic tie-breakers
- write paths must avoid partial target mutation

## Source-of-Truth Notes

For this ADR, executable code and determinism tests were treated as canonical over absolute determinism prose.

Important current-state clarifications:

- determinism is strongly enforced in the PyMuPDF path
- PDF.js fallback is intentionally tolerated with weaker strictness in tests
- no active random-number-based decision logic is present in current core paths
- full determinism invariants are a hard requirement for Rust, including stronger numeric normalization