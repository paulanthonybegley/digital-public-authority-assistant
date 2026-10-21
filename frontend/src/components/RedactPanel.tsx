import { useState } from "react";
import { redact } from "../api";
import type { RedactResponse } from "../types";
import { Feedback } from "./Feedback";

export function RedactPanel() {
  const [text, setText] = useState("");
  const [result, setResult] = useState<RedactResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    setLoading(true);
    setError(null);
    try {
      setResult(await redact(text));
    } catch (err) {
      setResult(null);
      setError(err instanceof Error ? err.message : "Unbekannter Fehler");
    } finally {
      setLoading(false);
    }
  }

  return (
    <section aria-labelledby="redact-heading">
      <h2 id="redact-heading">1 · DSGVO-Schwärzung</h2>
      <p className="hint">
        E-Mail-Adressen, deutsche IBANs und Telefonnummern werden vor der
        Akteneinsicht automatisch maskiert.
      </p>
      <form onSubmit={onSubmit}>
        <label htmlFor="redact-text">Freitext / Schriftsatz</label>
        <textarea
          id="redact-text"
          rows={7}
          value={text}
          onChange={(e) => setText(e.target.value)}
          placeholder="Sehr geehrte Damen und Herren, Kontakt: max@example.de, IBAN DE89 3704 0044 0532 0130 00, Tel. 030/1234567 …"
        />
        <button type="submit" disabled={loading}>
          {loading ? "Schwärze…" : "Anonymisieren"}
        </button>
      </form>

      <Feedback error={error} loading={loading && !result} loadingText="Schwärze…" />

      {result && (
        <div className="result" aria-live="polite">
          <h3>Ergebnis</h3>
          <output data-testid="redacted-text">{result.text}</output>
          <ul className="counts">
            <li>E-Mails: {result.counts.email}</li>
            <li>IBANs: {result.counts.iban}</li>
            <li>Telefonnummern: {result.counts.phone}</li>
            <li>
              <strong>Gesamt: {result.total_redacted}</strong>
            </li>
          </ul>
          <p className="meta">
            Audit-ID: {result.audit_id ?? "— (keine Persistenz konfiguriert)"}
          </p>
        </div>
      )}
    </section>
  );
}
