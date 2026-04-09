import { describe, it, expect, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";

describe("App — IPC round-trip smoke test", () => {
  beforeEach(() => {
    setMockResponse("cmd_validate_manifest", {
      valid: false,
      errors: ["Manifest contains no section edits"],
      sections_targeted: 0,
    });
  });

  it("renders the application heading", () => {
    render(<App />);
    expect(screen.getByText("Conset PDF")).toBeInTheDocument();
  });

  it("shows the IPC test button", () => {
    render(<App />);
    expect(
      screen.getByRole("button", { name: /Test IPC Round-Trip/i }),
    ).toBeInTheDocument();
  });

  it("test button invokes cmd_validate_manifest and displays response", async () => {
    const user = userEvent.setup();
    render(<App />);
    const btn = screen.getByRole("button", { name: /Test IPC Round-Trip/i });
    await user.click(btn);
    // Response JSON should appear in the pre block
    expect(await screen.findByText(/"valid": false/)).toBeInTheDocument();
  });
});
