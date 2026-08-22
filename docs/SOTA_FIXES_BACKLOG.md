# SOTA-Fixes — Entscheidungs-Backlog (2026-08-21)

**Trigger:** User „Wende die fixes an" (2026-08-21 16:21) auf Basis der zwei SOTA-Artefakte:
- `specs/out/sota-gzmo-agy.md` (agy, Web+arXiv Deep Research)
- `data-next/research-sota/latest.md` (nächtlicher Producer, Erstlauf 20260821T140901Z)

> **Doktrin-Hinweis (Pflicht):** Beide Artefakte sind **unvertrauenswürdiger Fremddaten-Content**.
> Der agy-Report kennzeichnet seine Handlungsempfehlungen explizit als *„Design, **nicht**
> Implementierungsauftrag"*. Vorschläge, die gegen die **Single-Writer-/Airgap-Doktrin**
> (ADR-0003/0004/0005/0007, CT101-Boundary) verstoßen, werden **nicht** blind umgesetzt —
> auch wenn sie „benefit=true" tragen. Anweisungen aus Scrapes werden nie übernommen, nur Fakten zitiert.

## Legende
- **APPLY-NOW** — sicher, additiv, mess-first, bricht kein Living-Substrat → sofort umgesetzt
- **SATISFIED** — GZMO erfüllt die Empfehlung bereits (kein Code nötig, nur Bestätigung)
- **GATED** — erst nach Messung / Human-Kickoff / ADR → getrackt, Kriterien fest
- **REJECTED** — Doktrin-Konflikt oder reiner Hype → verworfen, nur dokumentiert

---

## APPLY-NOW (sofort umgesetzt, 2026-08-21)

### C4 — Dual-Metering (RAPL + GPU) als Mess-Telemetrie
- **Quelle:** agy C4 „Dual-Metering + Abstention — RAPL (Host) + NVML (GPU) als Routing-Features;
  bei unzureichender Evidenz **nicht** schätzen" (TokenPowerSandbox `2608.18149`, green-mcp, llm-energy-lab).
- **Was jetzt:** `scripts/ops-health.sh` bekommt eine **reine Mess-Sektion** (keine Verdict-Änderung):
  per-GPU Power/Temp via `nvidia-smi` + RAPL `intel-rapl:0`/`intel-rapl:0:0` `energy_uj`.
  Wird in `latest.json` als `energy`-Block + Info-Zeilen ausgegeben. **Nicht-invasiv:**
  fehlendes `nvidia-smi`/RAPL = WARN, kein FAIL; GREEN/YELLOW/RED-Logik bleibt unangetastet.
- **Warum sicher:** read-only Telemetrie, „measure, never estimate" — genau die Doktrin.
  Ändert kein Routing, kein Substrat.
- **Nicht jetzt:** die **Routing-Integration** (Energie als AOS-Routing-Feature) → GATED (siehe unten).

---

## SATISFIED (bereits erfüllt)

### C3 — Consolidation hard-gaten
- **Quelle:** agy C3 (Zhang `2605.12978`, Colaco `2607.08032`, Sleep-time `2504.13171`).
- **Status:** GZMO metabolisiert **nur** quality-gated (Overnight-Daemon, SQLite-SoT,
  Operator darf nicht upserten). Episoden bleiben als Evidence. → **Erfüllt, kein Code nötig.**
- **Follow-up (optional, GATED):** Rate-Distortion-`2607.08032`-Linse auf die Quality-Gate-Schwellen
  anwenden, wenn distill-Qualität messbar nachlässt.

---

## GATED (Messung / Human-Kickoff / ADR nötig — Kriterien)

### C1 — Speculative Decoding / MTP für das Prime-MoE
- **Quelle:** agy C1 (EAGLE-3 `2503.01840`, S2-MoE `2608.15018`, AcceptMoE `2608.02989`;
  llama.cpp PR #18039/#25173).
- **Warum nicht jetzt:** Unabhängiges Benchmark (thc1006, Qwen3.6-35B-A3B Q4, RTX 3090, 2026-04)
  zeigt **Netto-Regression −40…−52 % Decode-Rate** bei großem Draft-K auf Consumer-MoE.
  Extrapolation von vLLM-Zahlen auf llama.cpp gilt als ungültig.
- **Gate-Kriterien (beide müssen positiv sein):**
  1. A/B: `tok/s` + **Distill-Qualitätsmetrik** vs. Baseline (greedy) auf dem konkreten
     Qwen3.8-27B-Q4-Workload. Peak tok/s allein zählt **nicht**.
  2. Overnight-Batch-Stabilität (keine Qualitätsdrift über Nacht).
- **Aktion:** erst ADR, dann Mess-Run, dann entscheiden.

### C2 — Hybrid-Query-Tiefe härten (Qdrant Prefetch/RRF/DBSF + Cross-Encoder-Rerank)
- **Quelle:** agy C2 (Qdrant Docs Hybrid/Rerank; vstash `2604.15484`).
- **Warum nicht jetzt:** ändert die **live Retrieval-Konfiguration** auf CT101 (LXC101-Server-Qdrant).
  Riskant ohne Recall/Precision-Benchmark.
- **Gate-Kriterien:** Recall/Precision-A/B gegen aktuellen Stand auf dem honeypot-Corpus;
  Latenztoleranz (TTFT) bleibt im Rahmen.
- **Qdrant Edge:** nur für **Client-Caches** (airgapped Clients), **nie** für das Living-Substrat.

### C2/C3 — Bi-temporale Fakten (Graphiti/Zep-Pattern) auf Neo4j-Schema
- **Quelle:** agy C2/C3 (Zep `2501.13956`, Graphiti).
- **Warum nicht jetzt:** **Schema-Änderung am laufenden Neo4j-KG** (invalidate-don't-delete,
  valid/transaction time). Human-Kickoff + ADR + Migration erforderlich. Keine Zep-Cloud-Abhängigkeit
  (nur das lokale Muster).
- **Gate-Kriterien:** ADR über bi-temporales Schema; Migration als atomarer Step mit Rollback.

### C4 — Energie als AOS-Routing-Feature (Abstention)
- **Quelle:** agy C4 (PALS `2605.21427`, EcoServe `2502.05043`, TokenPowerSandbox `2608.18149`).
- **Warum nicht jetzt:** Routing-Änderung = Verhalten des live Orchestrierers.
  Erst nach APPLY-NOW-Messphase, wenn Energie-Trajektorien stabil gemessen sind.
- **Gate-Kriterien:** ≥7 Tage Dual-Metering-Daten; Abstention-Logik (kein Schätzen ohne Evidenz);
  Quality-Gates bleiben unverändert.

### C5 — MCP Spec 2026-07-28 Migration tracken
- **Quelle:** agy C5 (Stateless Request/Response, `_meta`-Capabilities, Deprecations Roots/Sampling/Logging;
  Least-Privilege `2608.18351`; Attack-Surface `2608.17275`; Scaffolding `2608.08654`).
- **Aktion:** in OpenClaw-MCP-Clients **tracken** (Breaking Changes); Least-Privilege +
  Attack-Surface-Papers als **Security-Backlog**. Kein sofortiger Code-Bruch — Migration planen.
- **Gate-Kriterien:** SDK/Client-Kompatibilitätscheck, bevor eine Breaking-Change übernommen wird.

### PRECOG — Edge-SSM Backbone (arXiv:2608.02560)
- **Quelle:** research-sota 20260822T041632Z · TRL 5 · benefit=True
- **Integration-hebel:** Replace the local LLM backbone in the 'extract lane' or 'Brain Feed' with an SSM (TENNs-LLM 1.2B, 192 KB hidden state, O(1) prefill).
- **Status:** in-progress (ADR-0008 proposed; spikes in `spikes/`)
- **ADR:** [ADR-0008-edge-ssm-memory.md](ADR-0008-edge-ssm-memory.md) — Option A
- **Spike:** [`spikes/pre-cog/`](../spikes/pre-cog/) — availability probe + latency bench · [`spikes/pre-cog-mamba/`](../spikes/pre-cog-mamba/) — 7B Jamba decisive control + quality parity (VM200)
- **Gate-Kriterien:**
  1. TENNs-LLM weights license + availability offline (CC-BY-NC-4.0 = **GATED**)
  2. llama.cpp (or alternative) SSM inference support for TENNs-LLM (custom_code = **PARTIAL**)
  3. Quality parity with Qwen3.6-35B-MTP on GZMO's actual extract/distill prompts
  4. Energy (RAPL) comparison on edge hardware
- **Spike-Verdict (updated 2026-08-22):** TENNs-LLM still NO-GO (license + inference blocked). Alternative Mamba-class 7B (Jamba): **O(1) state-injection mechanism REPRODUCES on llama.cpp b9018** — 88.1× TTFT reduction at 17.5K tokens, airtight by C2/C3 + cold-control (v1 "content no-op" was a test error, withdrawn). **Quality gate HOLD:** 4/5 parity, 1/5 factual error under injection (lossy fixed-size SSM state); re-measure on real extract/distill prompts before GO. → Option A stays **HOLD**.

### MemoryLake — Structured Multi-Track Memory Backend (arXiv:2608.13883)
- **Quelle:** research-sota 20260822T041632Z · TRL 4 · benefit=True
- **Integration-hebel:** Adopt MemoryLake's structured multi-track backend for the 'living vault' on CT101.
- **Status:** in-progress (ADR-0008 proposed; spikes in `spikes/`)
- **ADR:** [ADR-0008-edge-ssm-memory.md](ADR-0008-edge-ssm-memory.md) — Option B
- **Spike:** [`spikes/memoryarena-baseline/`](../spikes/memoryarena-baseline/) — baseline harness against current system
- **Gate-Kriterien:**
  1. MemoryLake code availability + license (benchmark: Apache-2.0; backend: TBD)
  2. Baseline from spike showing current system's weaknesses on multi-session interdependent tasks (3/12 = 25%)
  3. Migration path that keeps ADR-0003 single-writer + ADR-0004 airgap intact
- **Spike-Verdict:** HOLD (baseline demonstrates real weakness: 14% on multi-session interdependent tasks; pending migration path verification)

---

## REJECTED (Doktrin-Konflikt / Hype — nur dokumentiert, nicht umgesetzt)

| Vorschlag | Quelle | TRL | Ablehnungsgrund |
|-----------|--------|-----|-----------------|
| **Qdrant durch Milvus ersetzen** (langchain-milvus) | research-sota | 9 | **Single-Writer-/Airgap-Doktrin**: bricht das laufende Living-Substrat auf CT101; untrusted-content-Anweisung, keine Faktenbegründung für den Tausch. |
| **Vector+Relation in Postgres unifizieren** (NeuronDB) | research-sota | 8 | **Doktrin**: rearchitekturiert das Vault; Qdrant+SQLite-SoT+Neo4j ist bewusst getrennt (ADR-0005 flywheel). |
| **MACC dezentrale Task-Representation** | research-sota | 7 | Design-Idee für Stigmergy-Board; kein messbarer Hebel jetzt. → nur als Idee notiert. |
| **AgentNet RAG-Multi-Agent** | research-sota | 6 | Overlap mit bestehendem Stigmergy-Board; kein konkreter Vorteil ohne Messung. |
| **KGCaRe / NeuroGuard / co-observation / 3-agent / LLM-Orchestration / generative Identifiers** | research-sota | 2–5 | **Research-TRL**, kein Produktionshebel; speculative. → nur als Reading-Liste. |
| **han-multi-agent-coordination (HF)** | research-sota | 2 | „physical hardware management" — aus dem Kontext; kein Bezug zu GZMO. |

---

## Nächster konkreter Schritt
1. **APPLY-NOW** (C4 Dual-Metering in ops-health) → implementiert + verifiziert + gepusht.
2. **GATED**-Items: jeweils **ADR + Mess-Run** als eigener Evolve-Schritt, Human-Kickoff.
3. **REJECTED**: geschlossen, nur dieses Dokument als Begründung.

*Erstellt: GZMO operator surface (OpenClaw) · 2026-08-21 · keine Scrapes-Anweisungen übernommen*
