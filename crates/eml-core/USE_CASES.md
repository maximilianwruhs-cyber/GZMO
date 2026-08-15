# EML Core — speculative notes (not a roadmap)

**EML = Exp-Minus-Log:** `eml(x, y) = exp(x) - ln(y)` — universelles Primitiv,
aus dem alle elementaren Funktionen synthetisiert werden können.

This table is **not** scheduled work. Nothing in `gzmo-core` calls this crate.
Do not implement rows from this file. Crate evolution is: honest calculator →
algebra that pays rent → one formula-IR call site only if a bench or radius wins.

---

## 🔷 Memory / Recall Pipeline

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 1 | **Honeypot Confidence Scoring** – Deterministische Qualifikation von Facts statt LLM-Heuristik: `confidence = eml(evidence, noise_floor)` | `memory/honeypot.rs` |
| 2 | **Fact Promotion Threshold** – Wann ein Fact vom Honeypot ins Core Memory promoted wird, via EML-Schwellwertfunktion | `memory/honeypot.rs`, `ripen.rs` |
| 3 | **RRF Recall Fusion Weight** – Reciprocal Rank Fusion Scores deterministisch berechnen | `memory/recall_rrf.rs` |
| 4 | **Episodic Decay Curve** – Alterung von episodischen Facts als EML-Ausdruck | `memory/episodic.rs` |
| 5 | **KG Extraction Confidence** – Confidence-Scores für extrahierte Knowledge Graph Triples | `memory/kg_extract.rs` |
| 6 | **Felt Use / Retrieval Frequency** – Nutzungs-Gewichtung für Lifecycle-Entscheidungen | `memory/felt_use.rs`, `lifecycle.rs` |
| 7 | **Vault Backend Scoring** – Deterministische Metriken für Vault-Operationen | `memory/vault_backend.rs` |

---

## 🔷 Session / Distillation / Ingest

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 8 | **Session Distillation Weighting** – Fact-Gewichtung basierend auf Wiederholung, Relevanz, Aktualität | `session_distill.rs` |
| 9 | **Ingest Pipeline Quality Scoring** – Datei-Bewertungsscores für die Ingest-Reihenfolge | `ingest.rs` |
| 10 | **Ingest Eval Metrics** – Deterministische Evaluations-Metriken für ingestierte Inhalte | `ingest_eval_cmd.rs` |

---

## 🔷 Spark / Dream / Metabolism

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 11 | **Spark Cross-Domain Similarity** – Ähnlichkeitskurven zwischen Facts aus verschiedenen Domänen | `spark.rs` |
| 12 | **Dream Consolidation Weight** – Konsolidierungs-Gewichte für nächtliche Dream Cycles | `dreams.rs` |
| 13 | **Metabolism Priority** – EML-basierte Dringlichkeit für den Metabolismus-Timer | `metabolism.rs` |
| 14 | **Pulse/Dream Timing** – Wann der nächste Pulse oder Dream getriggert wird | `dream_cmd.rs`, `metabolism_cmd.rs` |

---

## 🔷 Orchestrator / Scheduler

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 15 | **Wave Resolution Priority** – Deterministische Priorität für Pipeline-Wellen | `orchestrator.rs` |
| 16 | **Task Dependency Scoring** – Abhängigkeits-Gewichtung für Task-Graphen | `orchestrator.rs` |
| 17 | **Pedagogy EDF Scoring** – Earliest Deadline First mit EML-basierten Prioritätsformeln | `pedagogy/edf.rs` |
| 18 | **Knowledge Snapshot Relevance** – Snapshot-Relevanz als EML-Ausdruck | `pedagogy/knowledge_snapshot.rs` |

---

## 🔷 Gateway / LLM Routing

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 19 | **Model Selection Cost Function** – Routing-Entscheidungen (local vs. cloud) via EML-Kostenfunktion | `gateway.rs`, AOS Gateway |
| 20 | **Context Window Relevance** – Sliding-Window Relevance Scoring | `context.rs` |
| 21 | **Energy Budget Routing** – EML-basierte Budget-Verteilung auf Model-Endpoints | `gateway.rs`, Obolus |

---

## 🔷 Identity / Profile / Agent

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 22 | **Identity Confidence Scoring** – Deterministische Confidence für Agent-Identitäten | `identity.rs` |
| 23 | **Profile Similarity Metric** – Ähnlichkeit zwischen GZMO-Profilen | `profile.rs` |
| 24 | **Subagent Delegation Score** – Welcher Subagent bekommt welchen Task | `subagent.rs` |
| 25 | **Tool Selection Weight** – Tool-Routing im Delegate-Tool | `tools/delegate.rs` |

---

## 🔷 Health / Monitoring

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 26 | **Dynamic Health Thresholds** – Schwellwerte für Health-Checks statt hardcoded Konstanten | `health.rs` |
| 27 | **Stealth Discovery Priority** – Entdeckungs-Priorität bei Hintergrund-Scans | `stealth.rs` |
| 28 | **Watcher Reaction Priority** – File-Watcher Reaktions-Priorität | `watcher.rs` |
| 29 | **Scanner Scoring** – Scan-Priorität für Directory Scanning | `scanner.rs` |

---

## 🔷 Chaos Engine

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 30 | **Lorenz Parameter Optimization** – EML-basierte Parameter-Berechnung für den Lorenz-Attraktor | `gzmo-chaos/lorenz.rs` |
| 31 | **Chaos Pulse Timing** – Puls-Trigger Timing via EML-Formel | `gzmo-chaos/pulse.rs`, `triggers.rs` |
| 32 | **Chaos Thought Confidence** – Deterministische Metriken für generierte Chaos-Gedanken | `gzmo-chaos/thoughts.rs` |

---

## 🔷 Edge / Airgap USP

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 33 | **Symbolic Regression auf GZMO-Daten** – `eml_sr` nutzen um Formeln aus Memory/Spark-Daten zu lernen | extern: `eml_sr` Crate |
| 34 | **Battery-Aware Scheduling** – Energievorhersage für Aufgaben auf Edge-Geräten | Obolus + Scheduler |
| 35 | **Lossy Compression numerischer Metriken** – Metrik-Zeitreihen als EML-Ausdrücke speichern | vault / config |
| 36 | **Exekutionszeit-Prädiktion** – Task-Laufzeit via EML modellieren | Orchestrator |

---

## 🔷 EML-Eigene Fähigkeiten (Meta)

| # | Use Case | Beschreibung |
|---|----------|-------------|
| 37 | **Symbolic Differentiation** – EML-Ausdrücke automatisch differenzieren | neues Modul |
| 38 | **Expression Simplification** – EML-Bäume algebraisch vereinfachen | neues Modul |
| 39 | **Range/Overflow Analysis** – Worst-Case ComplexBall Radius vorhersagen | `ComplexBall` erweitern |
| 40 | **Expression Serialization** – EML-Bäume als Config-Format in `gzmo.toml` | Config-Layer |
| 41 | **Experiment Tracking Formeln** – Hyperparameter als EML-Ausdrücke für A/B-Vergleiche | extern |

---

## 🔷 MCP / Agent Integration

| # | Use Case | Betroffene Module |
|---|----------|-------------------|
| 42 | **MCP Server für EML-Evaluation** – Agent kann EML-Ausdrücke zur Laufzeit evaluieren | neuer MCP-Server |
| 43 | **Agent-Tool: EML-Rechner** – `execute()` als OpenClaw-MCP-Tool | AOS Tool |
| 44 | **Config als EML-Ausdrücke** – Schwellwerte in `gzmo.toml` als serialisierte `EmlExpr`-Bäume | Config |

---

## Prioritäts-Empfehlung

Die einsteigerfreundlichsten Use Cases mit dem meisten unmittelbaren Impact:

1. **#1** Honeypot Confidence Scoring
2. **#3** RRF Recall Weight
3. **#15** Orchestrator Priority

Dort existiert bereits Scoring-Logik, die man deterministisch machen kann.
