import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError, calculateDeadline, redact, validateCompliance } from "./api";

function mockFetch(response: Partial<Response> & { jsonText?: string }) {
  const fetchMock = vi.fn().mockResolvedValue({
    ok: response.ok ?? true,
    status: response.status ?? 200,
    text: async () => response.jsonText ?? "{}",
  });
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
}

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("api client", () => {
  it("posts the redact payload and parses the response", async () => {
    const fetchMock = mockFetch({
      jsonText: JSON.stringify({
        text: "[EMAIL_REDACTED]",
        counts: { email: 1, iban: 0, phone: 0 },
        total_redacted: 1,
        audit_id: null,
      }),
    });

    const result = await redact("a@b.de");

    expect(fetchMock).toHaveBeenCalledWith(
      "/api/v1/redact",
      expect.objectContaining({ method: "POST" }),
    );
    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body).toEqual({ text: "a@b.de" });
    expect(result.total_redacted).toBe(1);
  });

  it("throws ApiError with the server message on a 4xx", async () => {
    mockFetch({ ok: false, status: 422, jsonText: JSON.stringify({ error: "unknown federal state 'XX'" }) });

    await expect(
      calculateDeadline({
        start_date: "2026-01-01",
        days_to_add: 1,
        federal_state: "XX",
        counting: "calendar",
      }),
    ).rejects.toMatchObject({ name: "ApiError", status: 422 });

    await expect(
      calculateDeadline({
        start_date: "2026-01-01",
        days_to_add: 1,
        federal_state: "XX",
        counting: "calendar",
      }),
    ).rejects.toThrow("unknown federal state 'XX'");
  });

  it("reports a network failure as status 0", async () => {
    vi.stubGlobal("fetch", vi.fn().mockRejectedValue(new TypeError("failed")));
    await expect(redact("x")).rejects.toBeInstanceOf(ApiError);
  });

  it("sends only provided compliance fields", async () => {
    const fetchMock = mockFetch({
      jsonText: JSON.stringify({ tax_id: null, register_number: null, valid: true }),
    });
    await validateCompliance({ register_number: "HRB 12345 B" });
    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body).toEqual({ register_number: "HRB 12345 B" });
  });
});
