/**
 * Vitest global test setup.
 *
 * - Mocks the Tauri IPC bridge so frontend tests never spawn real subprocesses.
 * - Exposes `setMockResponse(command, response)` for per-test stub configuration.
 */
import "@testing-library/jest-dom";

// ---------------------------------------------------------------------------
// Tauri IPC mock
// ---------------------------------------------------------------------------

type MockResponses = Record<string, unknown>;
const mockResponses: MockResponses = {};

/**
 * Configure a stub return value for an `invoke(command, ...)` call.
 *
 * Call this in `beforeEach` / within individual tests:
 * ```ts
 * setMockResponse("cmd_validate_manifest", { valid: true, errors: [], sections_targeted: 1 });
 * ```
 */
export function setMockResponse(command: string, response: unknown): void {
  mockResponses[command] = response;
}

// Mock @tauri-apps/api/core before tests run.
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (command: string, _args?: unknown) => {
    if (command in mockResponses) {
      return mockResponses[command];
    }
    throw new Error(
      `[test-mock] No stub registered for invoke("${command}"). ` +
        `Call setMockResponse("${command}", yourResponse) in your test.`,
    );
  }),
}));

// Make setMockResponse available globally without import in test files.
declare global {
  // eslint-disable-next-line no-var
  var setMockResponse: (command: string, response: unknown) => void;
}
globalThis.setMockResponse = setMockResponse;
