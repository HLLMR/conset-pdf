# Phase 04 - IPC Envelope Contract

## Scope

Step 26 captures the stable GUI main/preload/renderer boundary contract for IPC responses.

Primary code contracts:

- `conset-pdf-gui/src/shared/ipc-response.ts`
- `conset-pdf-gui/src/preload.ts`
- `conset-pdf-gui/src/main/ipc/index.ts`

## Canonical Envelope

In `conset-pdf-gui/src/shared/ipc-response.ts`:

- `IpcResponse<T>` (`:23`)
- `createSuccessResponse` (`:37`)
- `createErrorResponse` (`:47`)
- `wrapIpcHandler` (`:69`)

Shape:

- success path: `{ success: true, data: T }`
- error path: `{ success: false, error: { message, code?, stack?, context? } }`

## Error Propagation Contract

`createErrorResponse` extracts and forwards:

- message
- code (if present)
- stack (if present)
- context (if present)

`preload.ts` mirrors this shape and unwraps it via `unwrapResponse` (`:50`) to throw `BridgeError` (`:26`) with preserved `code`, `stack`, and `context`.

Result:

- renderer callers receive direct data on success
- renderer callers receive exception objects with enriched metadata on failure

## Registered IPC Surface

`conset-pdf-gui/src/main/ipc/index.ts` currently registers:

- dialogs
- pdf
- profiles
- detection
- operations aggregator
- history
- system
- merge
- debug
- settings
- logging

This registration list is the active boundary inventory for IPC envelope coverage.

## Stability Rationale

The envelope isolates renderer code from handler-side error evolution:

- handlers can enrich error payloads without changing invoke-call signatures
- preload unwrapping keeps renderer API ergonomic and consistent
- error normalization is centralized instead of duplicated across handlers

## Drift Note

`conset-pdf-gui/docs/IPC_CONTRACTS.md` currently documents `error?: string`, while code uses structured `error` object. For migration/handoff work, code is canonical.

## Rust/Successor Mapping Notes

- Keep the envelope shape and unwrapping behavior even if transport changes from Electron IPC.
- Preserve optional `code`, `stack`, and `context`; they are already consumed by renderer/debug flows.
- Treat channel registration inventory as the canonical endpoint set, then map each channel to typed command/query contracts.

## Evidence

- `conset-pdf-gui/src/shared/ipc-response.ts`
- `conset-pdf-gui/src/preload.ts`
- `conset-pdf-gui/src/main/ipc/index.ts`
- `conset-pdf-gui/src/main/ipc/*.ts`
- `conset-pdf-gui/docs/IPC_CONTRACTS.md`
