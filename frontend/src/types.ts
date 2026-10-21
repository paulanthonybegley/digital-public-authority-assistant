export interface RedactionCounts {
  email: number;
  iban: number;
  phone: number;
}

export interface RedactResponse {
  text: string;
  counts: RedactionCounts;
  total_redacted: number;
  audit_id: string | null;
}

export type ShiftReason = "weekend" | "public_holiday";

export interface DeadlineResponse {
  date: string;
  raw_date: string;
  shifted: boolean;
  reason: ShiftReason | null;
  counting: "calendar" | "business" | string;
  federal_state: string;
  cached: boolean;
}

export interface TaxIdReport {
  valid: boolean;
  normalized: string | null;
  error: string | null;
  error_message: string | null;
}

export interface RegisterNumberReport {
  valid: boolean;
  normalized: string | null;
  error: string | null;
  error_message: string | null;
}

export interface ComplianceResponse {
  tax_id: TaxIdReport | null;
  register_number: RegisterNumberReport | null;
  valid: boolean;
}

export const FEDERAL_STATES = [
  "BB", "BE", "BW", "BY", "HB", "HE", "MV", "NI", "NW", "RP", "SL", "SN", "ST", "SH", "TH",
] as const;

export type FederalState = (typeof FEDERAL_STATES)[number];
