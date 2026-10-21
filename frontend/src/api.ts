import type {
  ComplianceResponse,
  DeadlineResponse,
  RedactResponse,
} from "./types";

export class ApiError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

async function postJson<T>(path: string, body: unknown): Promise<T> {
  let response: Response;
  try {
    response = await fetch(path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
  } catch (cause) {
    throw new ApiError("Netzwerkfehler: Dienst nicht erreichbar", 0);
  }

  const raw = await response.text();
  const parsed = raw ? JSON.parse(raw) : null;

  if (!response.ok) {
    const message =
      (parsed && typeof parsed.error === "string" && parsed.error) ||
      `Anfrage fehlgeschlagen (HTTP ${response.status})`;
    throw new ApiError(message, response.status);
  }
  return parsed as T;
}

export function redact(text: string): Promise<RedactResponse> {
  return postJson<RedactResponse>("/api/v1/redact", { text });
}

export function calculateDeadline(input: {
  start_date: string;
  days_to_add: number;
  federal_state: string;
  counting: "calendar" | "business";
}): Promise<DeadlineResponse> {
  return postJson<DeadlineResponse>("/api/v1/deadline", input);
}

export function validateCompliance(input: {
  tax_id?: string;
  register_number?: string;
}): Promise<ComplianceResponse> {
  return postJson<ComplianceResponse>("/api/v1/compliance/validate", input);
}
