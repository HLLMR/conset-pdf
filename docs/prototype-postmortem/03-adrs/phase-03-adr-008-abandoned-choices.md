# Phase 03 - ADR-008 Technology Choices Abandoned, Superseded, or Deferred

## Status

Accepted with corrections. The "abandoned choices" set is real but mixed in maturity: some choices are fully retired, some are superseded-but-still-present, and some are deferred roadmap items.

## Scope

Document which technologies/patterns were rejected or demoted, why they were changed, and what remains active in executable code despite deprecation language.

## Source Evidence

- `packages/core/src/utils/pdfLibBookmarkWriter.ts`
- `packages/core/src/bookmarks/pikepdfBookmarkWriter.ts`
- `packages/core/src/core/applyPlan.ts`
- `packages/core/src/transcript/factory.ts`
- `packages/core/src/analyze/documentContext.ts`
- `packages/core/src/locators/legacyTitleblockLocator.ts`
- `packages/core/src/config/featureFlags.ts`
- `packages/core/src/workflows/specs-patch/specsPatchWorkflow.ts`
- `packages/cli/src/cli.ts`
- `packages/cli/src/commands/specsPatch.ts`
- `packages/core/src/narrative/parse-algorithmic.ts`
- `docs/CLI.md`
- `docs/ARCHITECTURE.md`
- `docs/postMortemDocExtraction.md`

## Context

Prototype v3 iterated quickly across extraction, merge, bookmarking, and editing workflows. Several first-pass technology choices proved unreliable or too costly, leading to sidecar boundaries, ROI-first detection, transcript canonicalization, and workflow standardization.

However, implementation state is uneven: architectural direction often outpaced full runtime replacement.

## Decision

Classify prior choices into three buckets and use this taxonomy for migration and documentation:

- `ABANDONED`: no longer strategic and gated/deprecated
- `SUPERSEDED_PARTIAL`: strategic replacement exists, but legacy path still active in some flows
- `DEFERRED`: intentionally postponed, not implemented as primary runtime behavior

## Choice-by-Choice Record

### 1. pdf-lib bookmark writing

Status: `SUPERSEDED_PARTIAL`

- Strategic replacement: pikepdf/QPDF sidecar path (`pikepdfBookmarkWriter.ts`) for cross-viewer reliability
- Still active: merge apply path currently uses `PdfLibBookmarkWriter` when regenerating bookmarks in `applyPlan.ts`

Why changed:

- `pdf-lib` outline support is limited and implementation relies on fragile low-level dictionary manipulation
- sidecar path adds verification and stronger viewer compatibility behavior

Migration implication:

- bookmark replacement is directionally complete but not fully migrated across all write paths

### 2. PDF.js as primary extraction backend

Status: `SUPERSEDED_PARTIAL`

- Strategic replacement: transcript-first flow prefers PyMuPDF via sidecar (`createTranscriptExtractor` default chain)
- Still active: PDF.js remains fallback extractor and is still loaded in `DocumentContext` for bookmarks during transitional path

Why changed:

- insufficient bbox quality for geometry-critical detection in many real-world sets
- PyMuPDF dict/rawdict path materially improved extraction fidelity

Migration implication:

- "replaced" should be read as "demoted to fallback/auxiliary", not fully removed

### 3. In-memory merge assembly

Status: `SUPERSEDED_PARTIAL`

- Strategic replacement: pikepdf-backed write boundary and disk-streaming direction
- Still active: `applyPlan.ts` assembles output pages in memory with `pdf-lib` before pikepdf final write passthrough

Why changed:

- large-file memory pressure and OOM risk under full in-memory pipelines

Migration implication:

- streaming merge remains an architectural requirement for Rust; prototype has only partial realization

### 4. Legacy locator system

Status: `ABANDONED` (with controlled fallback)

- `LegacyTitleblockLocator` is explicitly marked deprecated/abandoned and feature-flag gated
- ROI profile locator is the intended primary path

Why changed:

- heuristic title-block detection is less deterministic and less configurable than profile-driven ROI

Migration implication:

- keep only if needed for controlled compatibility mode; default should stay ROI/profile-first

### 5. Full generic PDF AST concept (early broad framing)

Status: `ABANDONED` in broad form, but `SPEC_AST` remains active

- Broad "single universal PDF AST for everything" framing is not the active architecture
- Concrete `SpecDoc` AST in specs-patch flow is active and used in workflow/CLI execution

Why changed:

- over-generalized AST ambitions were expensive relative to immediate workflow value
- scoped, domain-specific AST for specs remained practical and is still in use

Migration implication:

- do not discard AST entirely in handoff language; preserve scoped AST lessons and contracts where active

### 6. LLM-assisted narrative parsing

Status: `DEFERRED`

- current runtime parser is explicitly algorithmic (`parse-algorithmic.ts`) with no LLM dependency
- narrative integration is advisory and deterministic in workflows

Why changed:

- algorithmic parser covered dominant cases with lower operational risk and better determinism

Migration implication:

- LLM narrative should remain optional augmentation, not baseline requirement

## Additional Drift Confirmed

Some docs label commands/features as abandoned while executable CLI still registers and runs them (for example `specs-patch` and `assemble-set` in `packages/cli/src/cli.ts`).

This indicates status-language drift that should be corrected or explicitly scoped to maintenance level vs executable availability.

## Alternatives Rejected

### Keep all legacy implementations equally supported

Rejected due to maintenance burden and conflicting behavior guarantees.

### Remove all legacy paths immediately

Rejected because transitional coverage and migration safety require controlled coexistence for some workflows.

### Declare all replacements complete in docs before code migration is complete

Rejected because it creates inaccurate handoff assumptions and hides operational risk.

## Rust Preservation Requirements

Rust should preserve successful directions:

- high-fidelity extraction as primary path
- deterministic profile-first detection
- robust bookmark compatibility behavior
- memory-safe merge architecture
- deterministic narrative parsing baseline

Rust should avoid prototype drift by:

- preventing "partial replacement" language from being treated as complete migration
- codifying explicit feature lifecycle states in docs and CLI surfaces
- separating "deprecated" from "removed" in user-facing contracts

## Source-of-Truth Notes

Executable code was treated as canonical over status prose.

Key clarifications:

- several listed "abandoned" choices are actually superseded-but-still-active in part
- specs AST remains active despite broad AST-abandonment phrasing
- command-level abandonment labels in docs do not always match CLI registration/runtime availability