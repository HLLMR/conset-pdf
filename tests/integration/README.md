# Integration Test Scaffold

This directory contains Step 10 integration-test scaffolds for the monorepo transition.

## Files

- `cli_basic_test.rs`
  - Harness for `backend-cli` executable behavior.
  - Validates `WorkflowResponse` JSON envelope shape for dry-run extract operation.
  - Marked `#[ignore]` because it requires:
    1. built `target/debug/backend-cli(.exe)`
    2. at least one sample PDF in `tests/corpus/`

- `gui_ipc_test.rs`
  - Mock IPC command tests for `apps/desktop-gui`.
  - Calls command handlers directly (`cmd_extract`, `cmd_parse`) using `WorkflowRequest`.
  - Validates response schema and `NOT_IMPLEMENTED` behavior until Phase 1 implementation lands.

## Current Intent (Phase 0/1)

These are scaffolds, not full behavioral assertions yet. They exist to lock integration boundaries:

1. contracts request/response envelope shape
2. backend-cli command surface
3. desktop-gui command handler signatures

## How To Run (Current)

From repository root:

```powershell
cargo build --bin backend-cli
cargo test -p conset-pdf-desktop-gui
```

The `cli_basic_test.rs` scaffold is intentionally ignored for now and is designed for activation once corpus test inputs are stabilized.

## Post-Phase 1 Pass Criteria

1. CLI extract dry-run test unignored and passing.
2. CLI non-dry-run test validates successful extraction against a known sample PDF.
3. GUI IPC tests validate successful command execution paths (not only `NOT_IMPLEMENTED`).
4. Integration tests are wired into CI with deterministic fixtures from `tests/corpus/`.
