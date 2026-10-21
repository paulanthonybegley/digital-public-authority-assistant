import { describe, expect, it } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import App from "./App";

describe("App", () => {
  it("renders the redaction tab by default", () => {
    render(<App />);
    expect(
      screen.getByRole("heading", { name: /DSGVO-Schwärzung/ }),
    ).toBeInTheDocument();
  });

  it("switches between the three tabs", async () => {
    const user = userEvent.setup();
    render(<App />);

    await user.click(screen.getByRole("tab", { name: "Fristen" }));
    expect(
      screen.getByRole("heading", { name: /Fristberechnung/ }),
    ).toBeInTheDocument();

    await user.click(screen.getByRole("tab", { name: "Stammdaten" }));
    expect(
      screen.getByRole("heading", { name: /Stammdatenprüfung/ }),
    ).toBeInTheDocument();
  });
});
