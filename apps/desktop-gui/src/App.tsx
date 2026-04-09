import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * Conset PDF Desktop — App root.
 *
 * Sprint 11.0 scaffold: renders the application shell and verifies the IPC
 * round-trip with the Tauri backend via a test button. The NOT_IMPLEMENTED
 * response from the backend confirms the invoke boundary is wired.
 */
function App() {
  const [ipcResponse, setIpcResponse] = useState<string | null>(null);
  const [ipcError, setIpcError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function testIpc() {
    setLoading(true);
    setIpcResponse(null);
    setIpcError(null);
    try {
      // cmd_validate_manifest is a pure Rust command that requires no filesystem
      // access — ideal smoke-test for the IPC round-trip.
      const result = await invoke<{
        valid: boolean;
        errors: string[];
        sections_targeted: number;
      }>("cmd_validate_manifest", {
        manifestJson: '{"section_edits":[]}',
        segmentIndexJson: '{"sections":[]}',
      });
      setIpcResponse(JSON.stringify(result, null, 2));
    } catch (e) {
      setIpcError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <main
      style={{
        fontFamily: "system-ui, sans-serif",
        padding: "2rem",
        maxWidth: "800px",
        margin: "0 auto",
      }}
    >
      <h1 style={{ fontSize: "1.5rem", fontWeight: "bold", marginBottom: "1rem" }}>
        Conset PDF
      </h1>
      <p style={{ color: "#666", marginBottom: "1.5rem" }}>
        Sprint 11.0 scaffold — IPC round-trip test.
      </p>

      <button
        onClick={testIpc}
        disabled={loading}
        style={{
          padding: "0.5rem 1.5rem",
          background: loading ? "#ccc" : "#1e64c8",
          color: "white",
          border: "none",
          borderRadius: "4px",
          cursor: loading ? "not-allowed" : "pointer",
          fontSize: "0.9rem",
        }}
      >
        {loading ? "Testing IPC…" : "Test IPC Round-Trip"}
      </button>

      {ipcResponse && (
        <pre
          style={{
            marginTop: "1rem",
            padding: "1rem",
            background: "#f0f9e8",
            border: "1px solid #acc",
            borderRadius: "4px",
            fontSize: "0.8rem",
            overflow: "auto",
          }}
        >
          {ipcResponse}
        </pre>
      )}

      {ipcError && (
        <pre
          style={{
            marginTop: "1rem",
            padding: "1rem",
            background: "#fdf0f0",
            border: "1px solid #caa",
            borderRadius: "4px",
            fontSize: "0.8rem",
            overflow: "auto",
          }}
        >
          Error: {ipcError}
        </pre>
      )}
    </main>
  );
}

export default App;
