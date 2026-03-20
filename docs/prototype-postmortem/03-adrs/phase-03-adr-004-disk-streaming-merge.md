# Phase 03 - ADR-004 Disk-Based Merge via Pikepdf Sidecar

## Status

Accepted as the intended architectural direction for the prototype and the required direction for Rust, but only partially realized in the current v3 codebase.

## Scope

Capture why large-file merge pushed the system toward a disk-based pikepdf sidecar strategy, what part of that strategy is active today, what remains planned rather than implemented, and what Rust must implement natively.

## Source Evidence

- `packages/core/src/core/mergeAddenda.ts`
- `packages/core/src/core/planner.ts`
- `packages/core/src/core/applyPlan.ts`
- `packages/core/src/utils/pikepdfWriter.ts`
- `packages/core/src/bookmarks/sidecar/bookmark-writer.py`
- `docs/largeFileRefactorPlan.md`
- `docs/WORKFLOWS.md`
- `docs/ARCHITECTURE.md`
- `docs/prototype-postmortem/02-algorithms/phase-02-merge-planning.md`
- `docs/prototype-postmortem/00-admin/source-of-truth.md`

## Context

The merge workflow must combine original and addendum PDFs while preserving:

- deterministic page ordering
- source-page fidelity
- safe output writes
- acceptable behavior on very large construction-document sets

The prototype's in-memory merge path builds a `MergePlan`, creates a new `PDFDocument`, copies source pages into that output document, optionally regenerates bookmarks, and then routes the final write through a pikepdf passthrough step.

That works functionally, but it does not solve the full large-file memory problem because page assembly still happens in memory before pikepdf touches the output.

The architectural response was to move toward a disk-based merge sidecar that would:

- consume a serialized merge plan
- open source PDFs sequentially
- stream/copy pages directly to disk-backed output
- avoid retaining the full assembled output document in Node memory

## Decision

Adopt disk-based merge via pikepdf sidecar as the intended primary architecture for large-file-safe merge execution.

For the prototype, this decision is only partially implemented:

- the final write boundary is routed through pikepdf today
- atomic temp-output handling exists at the write layer today
- the fully serialized-plan, sidecar-executed page assembly path is still planned rather than active

So the decision exists architecturally, but the current executable code stops short of full disk-streaming merge.

## Why This Was Chosen

### In-memory merge is the wrong scaling model

The merge workflow can involve very large PDFs and multiple source documents. A purely in-memory `pdf-lib` assembly path scales poorly because it requires the output document to be built inside the Node process before writing.

The design direction therefore shifted from "assemble in memory, then save" to "treat merge as a disk-oriented copy operation driven by a deterministic plan."

### Merge is already planner-driven

The planner already produces a deterministic ordered list of source pages and replacement/insert/unmatched semantics. That makes merge a good candidate for cross-language handoff because the heavy semantic work is already separated from the copy/write phase.

In other words:

- Node/TypeScript is well suited to planning and orchestration
- a PDF-native engine is better suited to low-level page copying and final writing

### Safe overwrite behavior matters

The large-file refactor plan explicitly calls for temp-file-then-rename semantics. That is the correct shape for destructive output operations because it avoids corrupting the target path on partial failure.

### Consistency with the sidecar strategy

The prototype already adopted pikepdf sidecars for bookmark/object writing. Extending that boundary to merge output was an architecturally consistent next step.

## Current Implementation Shape

### What is active today

`mergeAddenda()`:

- calls `planMerge()`
- then calls `applyMergePlan()` when not in dry-run mode

`applyMergePlan()` currently:

- creates a new `PDFDocument`
- loads source PDFs
- copies pages according to `plan.pages`
- optionally generates bookmarks
- writes the resulting PDF through `writePdfWithPikepdf()`

`writePdfWithPikepdf()` currently:

- saves the in-memory `PDFDocument` to a temp input PDF
- invokes `bookmark-writer.py --mode passthrough`
- writes to a temp output PDF
- copies temp output to final output path

So today's implementation already uses pikepdf as the final write boundary, but not as the primary page-assembly engine.

### What is still planned

`docs/largeFileRefactorPlan.md` describes the fuller architecture:

- serialize merge plan to JSON
- hand that plan to a Python merge sidecar
- stream/copy pages directly from source PDFs to disk output
- avoid Node-side in-memory output assembly entirely

That is not the current executable path.

## Benefits Already Achieved

Even the partial implementation provides some concrete gains:

- final PDF output is normalized through pikepdf rather than raw direct save
- output writes use temp-file-based safety instead of directly mutating the final path
- merge writing is aligned with the broader sidecar/object-writing strategy

## Benefits Intended by the Full Decision

The full disk-streaming architecture is intended to deliver:

- lower peak memory usage on very large merges
- better robustness on >500MB document sets
- cleaner separation between merge planning and PDF object manipulation
- a clear cross-language handoff contract for specialized PDF operations

## Trade-Offs

### Positive

- planner semantics stay in TypeScript where business logic already lives
- PDF-native copy/write work moves toward a toolchain better suited for large-file operations
- destructive output can be mediated through temp-file safety and explicit subprocess boundaries

### Negative

- the full strategy requires one more cross-language boundary and merge-plan serialization contract
- prototype implementation complexity rises because there are now two phases to reason about: planning and physical assembly
- current partial implementation may create false confidence if documentation is read without checking code

## Most Important Current Limitation

The most important limitation is straightforward:

- current merge still assembles the output in memory with `pdf-lib`
- therefore the primary OOM-risking step has not yet been eliminated

This ADR must therefore be read as "decision and direction with partial implementation," not "fully completed architecture."

## Alternatives Rejected

### Stay with pure in-memory `pdf-lib` merge

Rejected as the long-term design because it does not scale well to the large document sets the product needs to handle.

### Build the entire merge engine inside the Node process with no specialized PDF writer boundary

Rejected because the specialized page/object-writing problem is better isolated behind a PDF-native engine boundary.

### Delay the fix until Rust with no prototype-side movement

Rejected because the prototype already needed a safer and more reliable final write path, even before the full streaming merge existed.

## Rust Replacement Direction

Rust should not merely reproduce the prototype's partial state. It should implement the intended architecture directly:

- deterministic merge planning
- native streaming page copy/assembly
- temp-file-then-rename output safety
- no Python runtime dependency

The correct Rust target is therefore closer to the architecture described in `largeFileRefactorPlan.md` than to the prototype's current hybrid implementation.

## Prototype Invariants Worth Preserving

- merge planning remains deterministic and explicit
- source pages are copied, not re-rendered
- final output writes use safe temp-file semantics
- physical assembly should be separable from planning/orchestration
- large-file behavior must be treated as a first-class architecture concern, not an optimization afterthought

## Source-of-Truth Notes

For this ADR, executable code was treated as canonical over planning prose.

Important current-state clarifications:

- pikepdf passthrough write is active today
- fully serialized merge-plan handoff to a dedicated merge sidecar is not active today
- the prototype has moved toward disk-based merge architecture, but has not yet eliminated in-memory page assembly