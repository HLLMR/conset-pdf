# conset-pdf-contracts

Canonical request/response and audit payload schemas shared across backend and GUI boundaries.

## Versioning policy

During 0.x, the contracts crate version is locked to the engine version.

- `contracts::CONTRACTS_VERSION` is the canonical schema version.
- `apps/backend-cli` and `apps/desktop-gui` must report and validate this exact value.
- Cross-component communication must be rejected when versions do not match.

After v1.0, contracts may move to independent semantic versioning if GUI and engine release cycles diverge.

## Stability commitments during 0.x

- Request/response envelope field names are considered stable within a minor line.
- New fields may be added only as backward-compatible optional fields.
- Removing or renaming existing serialized fields requires a coordinated major migration plan.

## Phase D constraints represented

The contract schema explicitly models Phase D M-003 constraints:

- Session lifecycle logging (`session_started`, `session_ended`)
- Session end operation counts (`operation_counts`)
- Explicit gate semantics (`gate_evaluated`, `feature_disabled`)
- Required reason codes for explicit feature disablement
