# Risk Register

**Version:** 1.0.0  
**Date:** March 22, 2026  
**Owner:** HLLMR LLC  
**Status:** ACTIVE  
**Doc Status Tag:** Implemented

## Scope

Active project risks with severity, owner, mitigation, and decision date.

| ID | Risk | Severity | Owner | Mitigation | Decision Date | Status |
|---|---|---|---|---|---|---|
| R-001 | Documentation drift between canonical and archived docs could reintroduce conflicting guidance. | High | Documentation owner | Keep authority order and decision rule in index and governance doc; apply phase triage quickly. | 2026-03-22 | Active |
| R-002 | Foundation completeness claim may hide remaining scope gaps if not continuously reconciled with tests. | High | Engineering lead | Run alignment audit in Phase F and keep gap register current. | 2026-03-22 | **In Progress** — Phase E complete, gap-register.md v1.0.0 created (G-001–G-010). Phase F alignment audit complete; capability-matrix updated with accurate status. |
| R-003 | Historical prototype guidance may be accidentally treated as normative by agents or contributors. | Medium | Documentation owner | Mark archived docs clearly and keep canonical list at top of index. | 2026-03-22 | Active |
| R-004 | Capability status may become stale without regular updates to this library. | Medium | Project maintainer | Update this library in each planning milestone and release checkpoint. | 2026-03-22 | Active |
| R-005 | No real PDF→IR conversion path exists — engine and pdf-extraction crates are not connected, so no end-to-end pipeline is possible. | Critical | Engineering lead | Gap G-001/G-004/G-005 must be closed before any real extraction output can be produced. Track in gap-register.md. | 2026-03-22 | Open |

## Review Cadence

- Review and refresh at each phase boundary.
- Close risks only after evidence is linked in canonical docs or test artifacts.
- Escalate any risk that cannot be tied to current canonical evidence.
