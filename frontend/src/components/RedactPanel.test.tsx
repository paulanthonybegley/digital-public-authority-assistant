import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { RedactPanel } from "./RedactPanel";

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

describe("RedactPanel", () => {
  it("shows masked text and per-category counts", async () => {
    const user = userEvent.setup();
    stubFetch({
      text: "Kontakt [EMAIL_REDACTED], [IBAN_REDACTED], [PHONE_REDACTED]",
      counts: { email: 1, iban: 1, phone: 1 },
      total_redacted: 3,
      audit_id: "abc",
    });

    render(<RedactPanel />);
    await user.type(
      screen.getByLabelText("Freitext / Schriftsatz"),
      "Kontakt a@b.de, DE89 3704 0044 0532 0130 00, 030/1234567",
    );
    await user.click(screen.getByRole("button", { name: "Anonymisieren" }));

    await waitFor(() =>
      expect(screen.getByTestId("redacted-text")).toHaveTextContent(
        "[EMAIL_REDACTED], [IBAN_REDACTED], [PHONE_REDACTED]",
      ),
    );
    expect(screen.getByText("Gesamt: 3")).toBeInTheDocument();
    expect(screen.getByText(/Audit-ID: abc/)).toBeInTheDocument();
  });

  it("renders a server error message", async () => {
    const user = userEvent.setup();
    stubFetch({ error: "body must be JSON" }, false, 400);

    render(<RedactPanel />);
    await user.click(screen.getByRole("button", { name: "Anonymisieren" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("body must be JSON");
  });
});
