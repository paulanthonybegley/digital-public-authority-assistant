import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { CompliancePanel } from "./CompliancePanel";

function stubFetch(payload: unknown, ok = true, status = 200) {
  vi.stubGlobal(
    "fetch",
    vi.fn().mockResolvedValue({
      ok,
      status,
      text: async () => JSON.stringify(payload),
    }),
  );
}

afterEach(() => vi.unstubAllGlobals());

describe("CompliancePanel", () => {
  it("shows a valid overall result", async () => {
    const user = userEvent.setup();
    stubFetch({
      tax_id: {
        valid: true,
        normalized: "DE123456788",
        error: null,
        error_message: null,
      },
      register_number: {
        valid: true,
        normalized: "HRB 12345 B",
        error: null,
        error_message: null,
      },
      valid: true,
    });

    render(<CompliancePanel />);
    await user.click(screen.getByRole("button", { name: "Prüfen" }));

    await waitFor(() =>
      expect(screen.getByTestId("compliance-overall")).toHaveTextContent("gültig"),
    );
    expect(screen.getByTestId("tax-id-result")).toHaveTextContent("DE123456788");
  });

  it("shows an invalid result with the specific error", async () => {
    const user = userEvent.setup();
    stubFetch({
      tax_id: {
        valid: false,
        normalized: "DE123456789",
        error: "invalid_check_digit",
        error_message: "mod-11 check digit verification failed",
      },
      register_number: null,
      valid: false,
    });

    render(<CompliancePanel />);
    await user.click(screen.getByRole("button", { name: "Prüfen" }));

    await waitFor(() =>
      expect(screen.getByTestId("compliance-overall")).toHaveTextContent("ungültig"),
    );
    expect(screen.getByTestId("tax-id-result")).toHaveTextContent(
      "mod-11 check digit verification failed",
    );
  });
});
