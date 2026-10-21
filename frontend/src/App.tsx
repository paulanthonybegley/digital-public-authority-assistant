import { useState } from "react";
import { RedactPanel } from "./components/RedactPanel";
import { DeadlinePanel } from "./components/DeadlinePanel";
import { CompliancePanel } from "./components/CompliancePanel";

type Tab = "redact" | "deadline" | "compliance";

const TABS: { id: Tab; label: string }[] = [
  { id: "redact", label: "Schwärzung" },
  { id: "deadline", label: "Fristen" },
  { id: "compliance", label: "Stammdaten" },
];

export default function App() {
  const [tab, setTab] = useState<Tab>("redact");

  return (
    <div className="app">
      <header>
        <h1>Digitale Behörden-Assistenz</h1>
        <p className="subtitle">
          Prototyp zur Prüfung eingehender Schriftsätze und Unternehmensdaten
        </p>
      </header>

      <nav role="tablist" aria-label="Funktionen">
        {TABS.map(({ id, label }) => (
          <button
            key={id}
            role="tab"
            id={`tab-${id}`}
            aria-selected={tab === id}
            aria-controls={`panel-${id}`}
            className={tab === id ? "active" : ""}
            onClick={() => setTab(id)}
          >
            {label}
          </button>
        ))}
      </nav>

      <main id={`panel-${tab}`} role="tabpanel" aria-labelledby={`tab-${tab}`}>
        {tab === "redact" && <RedactPanel />}
        {tab === "deadline" && <DeadlinePanel />}
        {tab === "compliance" && <CompliancePanel />}
      </main>

      <footer>
        <span>Rust/Axum · React · PostgreSQL · Redis</span>
      </footer>
    </div>
  );
}
