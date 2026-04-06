# Risk Register

**Version:** 1.6.0  
**Date:** April 6, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

## Scope

Active project risks with severity, owner, mitigation, and decision date.

| ID | Risk | Severity | Owner | Mitigation | Decision Date | Status |
|---|---|---|---|---|---|---|
| R-001 | Documentation drift between canonical and archived docs could reintroduce conflicting guidance. | High | Documentation owner | Keep authority order and decision rule in index and governance doc; apply phase triage quickly. | 2026-03-22 | Active |
| R-002 | Foundation completeness claim may hide remaining scope gaps if not continuously reconciled with tests. | High | Engineering lead | Run alignment audit in Phase F and keep gap register current. | 2026-03-22 | **In Progress** — Phase E complete, gap-register.md updated through v1.5.0 (G-001–G-039). Phase F alignment audit complete; capability-matrix updated with accurate status. |
| R-003 | Historical prototype guidance may be accidentally treated as normative by agents or contributors. | Medium | Documentation owner | Mark archived docs clearly and keep canonical list at top of index. | 2026-03-22 | Active |
| R-004 | Capability status may become stale without regular updates to this library. | Medium | Project maintainer | Update this library in each planning milestone and release checkpoint. | 2026-03-22 | Active |
| R-005 | No real PDF→IR conversion path exists — engine and pdf-extraction crates are not connected, so no end-to-end pipeline is possible. | Critical | Engineering lead | Gap G-001/G-004/G-005 must be closed before any real extraction output can be produced. Track in gap-register.md. | 2026-03-22 | **Closed** — Phase 1 Band 0 closed G-001, G-004, G-005: `PdfiumExtractor::extract_page()` implemented with real pdfium-render calls; `SpanData`→`BoundingBox`→`normalize_bbox()`→`BBox`→`Span` conversion chain wired in `crates/engine/src/pipeline/extraction.rs`; end-to-end extract pipeline operational. 31/31 integration tests pass. |
| R-006 | Autonomous ROI strategy rollout may regress extraction reliability if deterministic scoring and fallback diagnostics are under-specified. | High | Architecture owner | Implement G-011 with strict deterministic ranking/tie-break rules and explicit ROI diagnostics; keep admin-only override path (G-012) until corpus thresholds are met. | 2026-03-23 | Open |
| R-007 | `lopdf` write-path correctness may silently corrupt page structure, bookmarks, or byte-identical unchanged pages if page-level mutations are not validated. | High | Engineering lead | Require round-trip validation tests for every `lopdf` mutation: verify unchanged pages are byte-identical, bookmarks resolve correctly, and page count matches expectations. G-014/G-016 implementations must include these tests before merge. | 2026-03-23 | Open |
| R-008 | Autonomous document-type classification may misclassify pages in mixed-medium PDFs, causing downstream medium-specific processors to operate on wrong input. | Medium | Architecture owner | Classification output is advisory only (Non-Negotiable #18 revision, D-012); no processing workflow executes without explicit user confirmation. Classification must emit confidence per page-range, not binary assignment. | 2026-03-23 | Open |
| R-009 | Geometry-only title-block localization may underperform on atypical CAD exports (nonstandard borders, sparse grid lines), reducing deterministic confidence coverage. | Medium | Architecture owner | Implement G-018/G-019 with explicit failure codes and confidence thresholds; gate fallback to template replay and optional AI path only after deterministic evidence bundle is produced. | 2026-03-23 | Open |
| R-010 | Auto-learned firm/layout templates may drift or become stale, propagating systematic extraction errors across future sets. | High | Engineering lead | Implement G-021 with template status lifecycle (unconfirmed/confirmed/deprecated), divergence checks against fresh detection, and forced review when drift thresholds exceed policy. | 2026-03-23 | Open |
| R-011 | Optional AI fallback could erode privacy expectations or become de facto baseline if invocation boundaries are weak. | Medium | Product owner | Enforce G-023 contract: explicit opt-in UI, low-confidence gate, cropped-region payload only, and audit log marker for each fallback invocation. Track fallback usage rate and block silent auto-invocation. | 2026-03-23 | Open |
| R-012 | Local micro-ML confidence assist could introduce non-reproducible outcomes if model versions or preprocessing drift across environments. | High | Engineering lead | Enforce G-024/G-025 with fixed model artifact hashing, deterministic preprocessing, and audit logging of fusion inputs/outputs per decision. | 2026-03-23 | Open |
| R-013 | Power-user LLM API usage may leak sensitive project data or produce over-trusted hallucinated instructions. | High | Security owner | Enforce G-026 with explicit opt-in, payload minimization/redaction, advisory default, and mandatory user promotion to executable instruction manifests. | 2026-03-23 | Open |
| R-014 | OCR path may degrade extraction quality on noisy raster scans, causing false positives in downstream parsing and segmentation. | Medium | Architecture owner | Implement G-027 confidence thresholds, review gating, and mixed-source (`ocr` vs `vector`) provenance-aware parsing rules. | 2026-03-23 | Open |
| R-015 | Schedule schema instability across JSON/CSV/XML exports may break downstream integrations. | Medium | Product owner | Implement G-028/G-029 with canonical schema versioning, compatibility tests, and explicit deprecation policy for field changes. | 2026-03-23 | Open |
| R-016 | Replaying correction manifests onto later revisions may apply valid-looking but semantically wrong fixes if scope guards are weak. | High | Engineering lead | Implement G-030 with target identity checks, divergence detection, dry-run preview, and explicit operator approval on replay drift. | 2026-03-23 | Open |
| R-017 | Privacy/redaction failures on external API paths may expose sensitive drawing/spec content beyond intended scope. | Critical | Security owner | Implement G-033 with payload manifests, policy-enforced region redaction, privacy modes, and outbound audit logging before any external request path ships. | 2026-03-23 | Open |
| R-018 | Batch orchestration/resume logic may create inconsistent partial state or duplicate mutations across retries. | High | Engineering lead | Implement G-034 with idempotent job state transitions, resumable checkpoints, and explicit mutation markers per file/page. | 2026-03-23 | Open |
| R-019 | Instruction DSL errors or ambiguous semantics could execute unsafe or unintended automation at scale. | High | Architecture owner | Implement G-035 with strict typing, dry-run validation, lint rules, and deterministic resolver behavior before execute is allowed. | 2026-03-23 | Open |
| R-020 | Standards normalization may drift from canonical UDS/NCS/MasterFormat references if runtime mappings fork from the maintained standards scaffold. | Medium | Standards owner | Implement G-037 directly against canonical standards docs/data and require versioned raw-to-canonical evidence for mapping changes. | 2026-03-23 | Open |
| R-021 | Cross-document entity resolution and diff workflows may create false links or noisy deltas that reduce trust in review outputs. | Medium | Architecture owner | Implement G-031/G-038/G-039 with evidence-backed linking, confidence thresholds, exception routing, and regression fixtures for rename/add/remove edge cases. | 2026-03-23 | Open |

## Review Cadence

- Review and refresh at each phase boundary.
- Close risks only after evidence is linked in canonical docs or test artifacts.
- Escalate any risk that cannot be tied to current canonical evidence.
