# Phase 03 - ADR-007 Privacy-Preserving ML Abstraction

## Status

Accepted. TokenVault-based abstraction with explicit privacy modes is the required boundary for ML-assisted profile generation. Privacy guarantees are mode-dependent and must be documented as such.

## Scope

Capture why the abstraction layer exists, how sensitive text is transformed before compiler calls, what the three privacy modes actually do in code, and what constraints Rust must preserve.

## Source Evidence

- `packages/core/src/transcript/abstraction/abstractTranscript.ts`
- `packages/core/src/transcript/abstraction/tokenVault.ts`
- `packages/core/src/transcript/abstraction/sanitize.ts`
- `packages/core/src/transcript/abstraction/shapeFeatures.ts`
- `packages/core/src/transcript/abstraction/repetitionMetrics.ts`
- `packages/core/src/transcript/abstraction/lineGrouping.ts`
- `packages/core/src/transcript/ml/promptBuilder.ts`
- `packages/core/src/transcript/ml/types.ts`
- `packages/core/src/transcript/ml/apiCompiler.ts`
- `tests/transcript/abstraction/abstractTranscript.test.ts`
- `tests/transcript/ml/promptBuilder.test.ts`
- `tests/transcript/ml/apiCompiler.test.ts`
- `docs/TRANSCRIPT_ARCHITECTURE.md`
- `docs/ML_RULESET_COMPILER.md`

## Context

AEC PDFs frequently include owner names, addresses, contacts, project identifiers, and proprietary content. At the same time, the prototype needs ML assistance for profile proposal workflows.

The architecture therefore inserts a privacy abstraction boundary between extracted transcript content and any external compiler/model call.

## Decision

Use `sanitizeTranscript()` + `TokenVault` as the canonical preprocessing step for ML-assisted profile generation.

This boundary must:

- replace sensitive span text with stable structural placeholders
- preserve geometric and typographic signals needed for document-structure inference
- make privacy policy explicit via mode selection
- keep reversible mapping local (TokenVault), not transmitted in compiler payloads

## How Sensitive Text Is Replaced

### 1. Structured abstraction contract

`AbstractTranscript` stores placeholder-first spans and lines with:

- `placeholderId`
- shape features (`tokenShape`, `charClassFlags`, `lengthBucket`)
- layout context (`bbox`, `fontName`, `fontSize`, style flags)
- repetition and reading-order metadata

This preserves extraction-relevant structure while avoiding raw text in the outbound abstract representation.

### 2. TokenVault mapping boundary

`TokenVault` maintains in-memory mappings from placeholder IDs to original text plus frequency/page stats.

Key behavior:

- non-whitelisted content in non-full-text modes gets shape-based placeholder IDs
- placeholder IDs are generated from structural features, not literal text
- mapping is retained locally for audit/reconstruction helpers, not required by compiler request schema

### 3. Path pseudonymization and optional sampling

`sanitizeTranscript()` HMAC-pseudonymizes file path to `anonymized_<hash>.pdf` and can reduce exposed content via sampling strategy (bands/headings/tables/maxPages).

This reduces incidental metadata leakage and constrains payload size/scope.

## Privacy Modes in Code

From `PrivacyMode` enum:

- `STRICT_STRUCTURE_ONLY`: no whitelist passthrough; maximum abstraction
- `WHITELIST_ANCHORS`: allows known safe anchor keywords (SECTION, PART, etc.)
- `FULL_TEXT_OPT_IN`: explicit mode allowing literal text preservation behavior

Default behavior in `sanitizeTranscript()` and `TokenVault.tokenize()` is `STRICT_STRUCTURE_ONLY`.

## Compiler Boundary Behavior

### Outbound payload shape

`ProfileProposalInput` expects `abstractTranscript`. Prompt construction in `promptBuilder.ts` uses placeholders and structural summaries. Prompt text explicitly frames placeholder usage for privacy.

### What is not sent by default

- TokenVault mappings are not part of `ProfileProposalInput`
- compiler types carry abstract transcript, not raw transcript

### Validation caveat path

`APIRulesetCompiler` may attempt local re-extraction for validation only if abstract transcript file path is not anonymized. With anonymized paths from sanitize flow, it does not auto-load original transcript for validation.

This is an internal local-processing caveat, not outbound payload leakage by itself.

## Test Evidence

`tests/transcript/abstraction/abstractTranscript.test.ts` verifies:

- stable placeholder IDs for identical shape signatures
- placeholder ID format and no obvious sensitive-string leakage
- repetition metrics, line grouping, coordinate metadata

`tests/transcript/ml/promptBuilder.test.ts` verifies deterministic, size-bounded placeholder-first prompts.

`tests/transcript/ml/apiCompiler.test.ts` covers API/compiler integration using abstract transcript inputs.

## Alternatives Rejected

### Send raw transcript to model

Rejected due to privacy/compliance risk and inability to guarantee safe handling of document-sensitive content.

### Regex redaction only

Rejected because it destroys structural signal needed for reliable profile inference and is brittle across diverse document content.

### Fully irreversible abstraction with no local mapping

Rejected for now because debugging/audit/reconstruction workflows require controlled local mapping access.

## Residual Risks and Constraints

### 1. Privacy guarantee is mode-dependent

"No sensitive data sent" is accurate for strict placeholder-first flows, but becomes conditional when `FULL_TEXT_OPT_IN` is selected or caller-provided context contains sensitive free text.

### 2. Local mapping sensitivity

TokenVault mappings keep original text in process memory. This is intentional for local workflows but remains a handling surface that needs explicit lifecycle and access controls in future runtime hardening.

### 3. Validation dependence on original transcript availability

Compiler validation quality can degrade when original transcript is not available (e.g., anonymized path), yielding lower-confidence acceptance behavior.

## Rust Preservation Requirements

Rust implementation must preserve:

- explicit privacy mode contract with strict default
- placeholder-first outbound model payloads
- structural-signal retention (geometry/typography/repetition/ordering)
- deterministic placeholder generation from shape features
- local-only reversible mapping boundary

Rust should strengthen:

- hard runtime guardrails that prevent accidental full-text model submission unless explicit override is acknowledged
- secure lifecycle handling for local reversible mappings
- policy-level enforcement and telemetry around privacy mode usage

## Source-of-Truth Notes

For this ADR, executable abstraction/compiler code and tests were canonical over unconditional privacy prose.

Critical clarifications:

- strict privacy behavior is real and implemented as default
- privacy guarantees are conditional on mode/configuration
- outbound compiler contract uses abstract transcript, not token mappings
- documentation should avoid blanket claims that ignore `FULL_TEXT_OPT_IN` and caller-supplied context pathways