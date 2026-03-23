# Decision Log

**Version:** 1.0.0  
**Date:** March 22, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

## Scope

High-value decisions that shape near-term execution.

| ID | Date | Decision | Rationale | Impacted Docs |
|---|---|---|---|---|
| D-001 | 2026-03-22 | Treat Phase 0 baseline as accepted for intended scope. | Enables controlled transition from foundation to consolidation and alignment phases. | ../MASTER_PLAN_v4.md, ../dev/PHASE_0_IMPLEMENTATION_PLAN_v4_2_1.md |
| D-002 | 2026-03-22 | Adopt formal documentation governance with authority order and decision rule. | Prevents plan drift and conflicting agent guidance. | ../DOC_GOVERNANCE.md, ../DOCUMENTATION_INDEX.md |
| D-003 | 2026-03-22 | Require one Doc Status Tag in all docs markdown files. | Makes lifecycle state explicit and machine-auditable. | ../DOC_GOVERNANCE.md |
| D-004 | 2026-03-22 | Keep prototype-live and prototype-postmortem trees as archived, informative-only sources. | Preserves historical insight without overriding canonical v4 direction. | ../DOCUMENTATION_INDEX.md |
| D-005 | 2026-03-22 | Retire all Phase C merge-target v3 docs after migration extraction. | Removes dead links and eliminates authority ambiguity from deprecated v3 sources. | ../PHASE_C_DOC_TRIAGE_MATRIX.md, ../DOCUMENTATION_INDEX.md |
| D-006 | 2026-03-22 | Require section-level migration closure mapping for Phase D imports. | Makes migration decisions auditable from source insight to canonical section destination. | ../current-state/migration-intake.md |
| D-007 | 2026-03-22 | Accept Phase 0 foundation stubs (engine extractor/processor, extract_page) as known open gaps rather than blocking Phase D doc work. | Docs can be canonical and accurate while code is in progress; capability-matrix must reflect actual state, not aspirational. | ../current-state/gap-register.md, ../current-state/capability-matrix.md |
| D-008 | 2026-03-22 | The Gap Register (gap-register.md) is the authoritative list of open implementation gaps; capability-matrix Open Gaps column links to it. | Single source of truth for gap tracking prevents drift between docs and code state. | ../current-state/gap-register.md, ../current-state/capability-matrix.md |

## Decision Rule Reminder

If guidance is not in the master plan or canonical derived docs, treat it as non-authoritative.

Every future decision entry must reference at least one canonical destination doc.
