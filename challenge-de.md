Hier ist ein maßgeschneiderter 90-Minuten-Coding-Challenge-Entwurf, der thematisch im Bereich Justiz / Verwaltung / Unternehmen (GovTech / LegalTech / Compliance) angesiedelt ist.
Im Fokus stehen hierbei nicht abstrakte Algorithmen, sondern typische Praxisprobleme aus diesen Sektoren: Datenschutzkonforme Datenanonymisierung, Fristenberechnung oder Dokumentenprüfung.
Coding Challenge: „Der digitale Behörden-Assistent“
	•	Dauer: 90 Minuten 
	•	Zielgruppe: Softwareentwickler (Backend / Fullstack: z. B. Python, Node.js, Java oder C#)
	•	Kontext: Eine Landesbehörde / ein Justizdienstleister benötigt einen Mikroservice, der eingehende digitale Schriftsätze und Unternehmensdaten vor der Weiterverarbeitung automatisiert prüft, Fristen berechnet und sensible Daten schützt. 
Szenario & Aufgabenstellung
Sie entwickeln einen Prototypen für ein Bürger- und Unternehmensportal. Das System erhält strukturierte JSON-Daten, die Anträge, Klagen oder Unternehmensmeldungen repräsentieren.
In 90 Minuten müssen Sie drei Kernkomponenten (User Stories) implementieren:
	1	DSGVO-konforme Schwärzung (Data Masking): Personenbezogene Daten (Namen, E-Mail-Adressen, IBANs) in Textfeldern müssen mittels Regular Expressions oder Maskierungslogik unkenntlich gemacht werden.
	2	Gesetzliche Fristenberechnung (Legal Date Logic): Unter Berücksichtigung von Wochenenden und Feiertagen muss das offizielle Fristende eines Bescheids ermittelt werden.
	3	Validierung von Unternehmensstammdaten (Compliance Check): Prüfung von Registernummern und Umsatzsteuer-IDs nach vorgegebenen Prüfziffern-Algorithmen.
Zeitlicher Leitfaden (Timeboxing)
	•	00:00 – 00:10 Uhr: Setup, Lesen der Anforderungen & Datenstrukturen.
	•	00:10 – 00:35 Uhr: Task 1 – Text-Anonymisierung (DSGVO).
	•	00:35 – 01:05 Uhr: Task 2 – Fristenberechnung unter Berücksichtigung von Feiertagen.
	•	01:05 – 01:25 Uhr: Task 3 – Validierung von Unternehmensdaten (Prüfziffern).
	•	01:25 – 01:30 Uhr: Finaler Code-Check, Tests und kurzer Abschlusskommentar.
Detaillierte Aufgaben
Task 1: Der Schwärzungs-Automat (DSGVO)
	•	Problem: Dokumente und Freitexte aus Verwaltungsverfahren enthalten sensible Klardaten, die vor der Akteneinsicht geschützt werden müssen. 
	•	Aufgabe: Schreiben Sie eine Funktion mask_sensitive_data(text: str) -> str, die im übergebenen Text folgende Muster maskiert: 
	◦	E-Mail-Adressen: Ersetzen durch [EMAIL_GESCHÄTZT] (oder ***).
	◦	IBANs (DE-Standard): IBANs im Format DE\d{2}\s?(\d{4}\s?){4}\d{2} durch [IBAN_GESCHÄTZT] ersetzen.
	◦	Telefonnummern: Gängige deutsche Formate (z. B. +49 ... oder 030/...) anonymisieren.
Task 2: Die behördliche Fristenberechnung
	•	Problem: Nach deutschem Recht (z. B. VwVfG / ZPO) enden Fristen unter bestimmten Bedingungen. Fällt das Fristende auf einen Samstag, Sonntag oder einen gesetzlichen Feiertag, verschiebt sich das Ende auf den nächsten Werktag. 
	•	Aufgabe: Schreiben Sie eine Funktion calculate_deadline(start_date: str, days_to_add: int, federal_state: str) -> str: 
	◦	Addieren Sie days_to_add (z. B. 1 Monat oder 2 Wochen – für die Challenge ideal: Werktage/Kalendertage ab Startdatum) auf das start_date (Format: YYYY-MM-DD).
	◦	Prüfen Sie, ob das berechnete Datum auf ein Wochenende fällt.
	◦	Bonus/Optional: Berücksichtigen Sie mindestens drei feste Feiertage des Bundeslandes (z. B. Neujahr, Tag der Arbeit, 3. Oktober). Fällt das Datum darauf, verschieben Sie es auf den nächsten Werktag.
Task 3: Compliance & Unternehmensregister-Check
	•	Problem: Unternehmen, die Anträge einreichen, müssen valide Identifikationsmerkmale vorweisen. 
	•	Aufgabe: Implementieren Sie eine Validierungsfunktion für eine vereinfachte Umsatzsteuer-ID (USt-IdNr.) oder eine Handelsregisternummer: 
	◦	Eine deutsche USt-IdNr. beginnt mit dem Länderkürzel DE gefolgt von 9 Ziffern (z. B. DE123456789).
	◦	Schreiben Sie eine Funktion validate_tax_id(tax_id: str) -> bool, die das Format prüft und optional einen vereinfachten Modulo-11-Algorithmus auf die Ziffern anwendet.
Bewertungskriterien (Was der Interviewer / Evaluator sieht)
	1	Korrektheit & Robustheit: Funktionieren die Unit Tests für die Grenzfälle (z. B. Jahreswechsel bei Fristen, ungültige Formate)?
	2	Code-Qualität & Lesbarkeit: Aussagekräftige Variablennamen, saubere Struktur, Modularisierung (kein monolithischer Skript-Spaghetti-Code).
	3	Umgang mit Edge Cases: Was passiert bei leerem Input, falschen Datumsformaten oder Sonderzeichen?
	4	Pragmatismus: In 90 Minuten muss nicht die gesamte deutsche Feiertagslogik abgedeckt sein – es zählt, ob intelligente Platzhalter oder saubere Mock-Logik verwendet werden, um das Ziel zeitgerecht zu erreichen. 