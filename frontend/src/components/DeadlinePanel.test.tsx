import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { DeadlinePanel } from "./DeadlinePanel";

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

describe("DeadlinePanel", () => {
  it("displays the shifted deadline and its reason", async () => {
    const user = userEvent.setup();
    stubFetch({
      date: "2026-01-05",
      raw_date: "2026-01-03",
      shifted: true,
      reason: "weekend",
      counting: "calendar",
      federal_state: "BE",
      cached: false,
    });

    render(<DeadlinePanel />);
    await user.click(screen.getByRole("button", { name: "Frist berechnen" }));

    await waitFor(() =>
      expect(screen.getByTestId("deadline-date")).toHaveTextContent("2026-01-05"),
    );
    expect(screen.getByText(/verschoben wegen Wochenende/)).toBeInTheDocument();
  });

  it("supports business-day counting selection", async () => {
    const user = userEvent.setup();
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      text: async () =>
        JSON.stringify({
          date: "2026-01-05",
          raw_date: "2026-01-05",
          shifted: false,
          reason: null,
          counting: "business",
          federal_state: "BE",
          cached: true,
        }),
    });
    vi.stubGlobal("fetch", fetchMock);

    render(<DeadlinePanel />);
    await user.selectOptions(screen.getByLabelText("Zählweise"), "business");
    await user.click(screen.getByRole("button", { name: "Frist berechnen" }));

    await waitFor(() =>
      expect(screen.getByTestId("deadline-date")).toHaveTextContent("2026-01-05"),
    );
    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body.counting).toBe("business");
    expect(screen.getByText(/Cache/)).toBeInTheDocument();
  });
});
