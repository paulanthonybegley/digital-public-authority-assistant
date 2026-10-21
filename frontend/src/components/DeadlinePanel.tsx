import { useState } from "react";
import { calculateDeadline } from "../api";
import { FEDERAL_STATES, type DeadlineResponse } from "../types";
import { Feedback } from "./Feedback";

const REASON_LABEL: Record<string, string> = {
  weekend: "Wochenende",
  public_holiday: "gesetzlicher Feiertag",
};

export function DeadlinePanel() {
  const [startDate, setStartDate] = useState("2026-01-02");
  const [days, setDays] = useState(14);
  const [state, setState] = useState<string>("BE");
  const [counting, setCounting] = useState<"calendar" | "business">("calendar");
  const [result, setResult] = useState<DeadlineResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function onSubmit(event: React.FormEvent) {
    event.preventDefault();
    setLoading(true);
    setError(null);
    try {
      setResult(
        await calculateDeadline({
          start_date: startDate,
          days_to_add: days,
          federal_state: state,
          counting,
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
    <section aria-labelledby="deadline-heading">
      <h2 id="deadline-heading">2 · Fristberechnung</h2>
      <p className="hint">
        Fällt das Fristende auf ein Wochenende oder einen Feiertag, verschiebt
        es sich auf den nächsten Werktag (VwVfG § 32, ZPO § 193).
      </p>
      <form onSubmit={onSubmit}>
        <label htmlFor="start-date">Startdatum</label>
        <input
          id="start-date"
          type="date"
          value={startDate}
          onChange={(e) => setStartDate(e.target.value)}
          required
        />

        <label htmlFor="days">Tage</label>
        <input
          id="days"
          type="number"
          min={0}
          value={days}
          onChange={(e) => setDays(Number(e.target.value))}
          required
        />

        <label htmlFor="federal-state">Bundesland</label>
        <select
          id="federal-state"
          value={state}
          onChange={(e) => setState(e.target.value)}
        >
          {FEDERAL_STATES.map((code) => (
            <option key={code} value={code}>
              {code}
            </option>
          ))}
        </select>

        <label htmlFor="counting">Zählweise</label>
        <select
          id="counting"
          value={counting}
          onChange={(e) => setCounting(e.target.value as "calendar" | "business")}
        >
          <option value="calendar">Kalendertage</option>
          <option value="business">Werktage</option>
        </select>

        <button type="submit" disabled={loading}>
          {loading ? "Berechne…" : "Frist berechnen"}
        </button>
      </form>

      <Feedback error={error} loading={loading && !result} loadingText="Berechne…" />

      {result && (
        <div className="result" aria-live="polite">
          <h3>Fristende</h3>
          <p className="big-date" data-testid="deadline-date">
            {result.date}
          </p>
          {result.shifted ? (
            <p>
              Berechnet: {result.raw_date} — verschoben wegen{" "}
              {REASON_LABEL[result.reason ?? ""] ?? "unterbrochener Frist"}. (
              {result.federal_state})
            </p>
          ) : (
            <p>Keine Verschiebung erforderlich. ({result.federal_state})</p>
          )}
          <p className="meta">
            Zählweise: {result.counting} · {result.cached ? "Cache" : "frisch berechnet"}
          </p>
        </div>
      )}
    </section>
  );
}
