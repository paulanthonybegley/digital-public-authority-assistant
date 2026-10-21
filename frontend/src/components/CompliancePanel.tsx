import { useState } from "react";
import { validateCompliance } from "../api";
import type { ComplianceResponse } from "../types";
import { Feedback } from "./Feedback";

export function CompliancePanel() {
  const [taxId, setTaxId] = useState("DE123456788");
  const [registerNumber, setRegisterNumber] = useState("HRB 12345 B");
  const [result, setResult] = useState<ComplianceResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    setLoading(true);
    setError(null);
    try {
      setResult(
        await validateCompliance({
          tax_id: taxId.trim() || undefined,
          register_number: registerNumber.trim() || undefined,
        }),
      );
    } catch (err) {
      setResult(null);
      setError(err instanceof Error ? err.message : "Unbekannter Fehler");
    } finally {
      setLoading(false);
    }
  }

  return (
    <section aria-labelledby="compliance-heading">
      <h2 id="compliance-heading">3 · Stammdatenprüfung</h2>
      <p className="hint">
        Prüft die deutsche USt-IdNr. (Modulo-11) und das Handelsregisterformat.
      </p>
      <form onSubmit={onSubmit}>
        <label htmlFor="tax-id">USt-IdNr.</label>
        <input
          id="tax-id"
          value={taxId}
          onChange={(e) => setTaxId(e.target.value)}
          placeholder="DE123456788"
          aria-describedby="tax-id-hint"
        />
        <small id="tax-id-hint">Format: DE + 9 Ziffern</small>

        <label htmlFor="register-number">Handelsregisternummer</label>
        <input
          id="register-number"
          value={registerNumber}
          onChange={(e) => setRegisterNumber(e.target.value)}
          placeholder="HRB 12345 B"
        />

        <button type="submit" disabled={loading}>
          {loading ? "Prüfe…" : "Prüfen"}
        </button>
      </form>

      <Feedback error={error} loading={loading && !result} loadingText="Prüfe…" />

      {result && (
        <div className="result" aria-live="polite">
          <h3>
            Gesamtergebnis:{" "}
            <span
              className={result.valid ? "ok" : "bad"}
              data-testid="compliance-overall"
            >
              {result.valid ? "gültig" : "ungültig"}
            </span>
          </h3>
          {result.tax_id && (
            <p data-testid="tax-id-result">
              USt-IdNr.: {result.tax_id.valid ? "gültig" : "ungültig"}
              {result.tax_id.normalized && ` (${result.tax_id.normalized})`}
              {result.tax_id.error_message && ` — ${result.tax_id.error_message}`}
            </p>
          )}
          {result.register_number && (
            <p data-testid="register-result">
              Handelsregister: {result.register_number.valid ? "gültig" : "ungültig"}
              {result.register_number.normalized &&
                ` (${result.register_number.normalized})`}
              {result.register_number.error_message &&
                ` — ${result.register_number.error_message}`}
            </p>
          )}
        </div>
      )}
    </section>
  );
}
