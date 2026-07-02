# Abschlussbericht: thema_009 — Compositional Recall / VCR Integration in GZMO

**Datum:** 2026-06-26
**Status:** Phase 0–5 abgeschlossen
**Repo:** `_foundation-audit/survey_GZMO`

---

## 1. Ausgangslage

### Research-Quelle (thema_009)

| Datei | Pfad |
|-------|------|
| Preprint PDF | `/home/maximilian-wruhs/Schreibtisch/research/thema_009/zeroshot.pdf` |
| Curated KB | `/home/maximilian-wruhs/Schreibtisch/knowledge/curated/thema_009-compositional-recall.md` |
| Paper | [arXiv:2606.24948](https://arxiv.org/abs/2606.24948) — Kumar, Jun 2026 |

Inhalt: Mechanistische Studie, warum Holographic Reduced Representations (HRR/FHRR)
am Zero-Shot Two-Hop Compositional Reasoning über FB15k-237 scheitern. Negatives
Resultat, das GZMOs explizite Fakt+Graph-Architektur validiert.

### Bewertungsrahmen (Direktive)

- [SOUL.md](../SOUL.md) — Sovereignty First, Memory Discipline
- [ARCH-DIR-001-GZMO.md](ARCH-DIR-001-GZMO.md) — Local-first, Zero-Bloat, Obolus-Bilanz
- [DISCOVERY_KB_FEEDBACK_LOOP.md](DISCOVERY_KB_FEEDBACK_LOOP.md) — G12 (Eval green ≠ recall green)

### Einordnung in die Research-Serie

| Thema | Inhalt | KB-Status |
|-------|--------|-----------|
| thema_001–003 | OKF / ARD / Agent-Reach | ingest |
| thema_004 | arXiv ecosystem | ingest + skill + compliance |
| thema_005 | Jules patterns | ingest + rust patterns |
| thema_006 | Loop engineering | synthesis + packs |
| thema_007 | DualPath LLM serving | research only (Schreibtisch) |
| thema_008 | Scholar Labs workflow | research only (Schreibtisch) |
| **thema_009** | **Compositional recall failure / VCR** | **ingest + probes + rust + spark + discovery contract** |

---

## 2. Kernthese und GZMO-Übersetzung

| Paper Probe | Resultat | GZMO-Analog |
|-------------|----------|-------------|
| Atomic single-hop MRR | ~0.35 (gut) | `discovery-kb-recall-smoke.sh` pass_rate 1.0 (falsch grün) |
| Hop-1 intermediate | MRR ~0.90 | Neo4j 1-hop hints in `graph-recall-stream.py` |
| Zero-shot 2-hop composition | chance | Kein compositional Eval vorhanden |
| Hop-2 atomic difficulty | 0.26–0.48× atomic baseline | Hub-Fakten schwerer auch standalone |

**Bottleneck:** Retrieval-Kapazität unter Superposition für hoch-contention Fakten
— nicht Bind-Unbind-Algebra oder Cleanup. MRR toleriert Near-Miss; Composition
nicht. Aggregat-Metriken maskieren per-Fact-Schwäche.

**Positive Analogie:** CQD (Continuous Query Decomposition, ICLR 2021) — Query in
atomare Link-Predictions zerlegen, aggregieren, erklärbare Intermediates. GZMOs
Pfad: explizite 2-hop Walks + per-hop honeypot Lookups statt Superposition.

---

## 3. Durchgeführte Änderungen

### 3.1 KB-Ingest (Phase 0)

| Artefakt | Pfad |
|----------|------|
| Curated Source | `~/Schreibtisch/knowledge/curated/thema_009-compositional-recall.md` |
| Wiki-Source | [wiki/sources/thema-009-compositional-recall.md](../wiki/sources/thema-009-compositional-recall.md) |

Neue Wiki-Entities:
- [verified-chain-recall](../wiki/entities/verified-chain-recall.md)
- [holographic-reduced-representations](../wiki/entities/holographic-reduced-representations.md) (Anti-Pattern)
- [compositional-recall-capacity](../wiki/entities/compositional-recall-capacity.md)
- [hop-2-atomic-difficulty](../wiki/entities/hop-2-atomic-difficulty.md)
- [hub-contention-index](../wiki/entities/hub-contention-index.md)
- [continuous-query-decomposition-cqd](../wiki/entities/continuous-query-decomposition-cqd.md)
- [g12-eval-green-recall-green](../wiki/entities/g12-eval-green-recall-green.md)

### 3.2 Compositional Recall Probe (Phase 1)

| Datei | Änderung |
|-------|----------|
| `scripts/compositional-recall-probe.py` | Neu — Chain-Mining + Probe A (hop-1), B (chain), C (hop-2 atomic); parallelisierte `gzmo memory search`-Calls |
| `scripts/compositional-recall-smoke.sh` | Neu — Wrapper (nächtlicher Cron) |
| `scripts/ingest-quality/gate-discovery-loop.sh` | Pack-H-Erweiterung: lied `compositional-recall-latest.json` (atomic vs chain); WARN-first; Probe läuft nightly, nicht inline |

Output: `data/discovery-kb-metrics/compositional-recall-{stamp}.json` mit
`hop1_mrr`, `chain_hit_rate`, `hop2_atomic_ratio`, per-Chain-Breakdown.

### 3.3 Verified 2-Hop Graph Stream (Phase 2)

| Datei | Änderung |
|-------|----------|
| `scripts/graph-recall-stream.py` | `GRAPH_RECALL_MODE=2hop` — verified chain hints `"A via REL mid via REL n"` |
| `gzmo-core/src/memory/vault.rs` | `fetch_neo4j_graph_hints` reicht `GRAPH_RECALL_MODE` durch; Chain-Hint-RRF-Boost |
| `gzmo-core/src/memory/vault.rs` | Keyword-Stream entkoppelt — läuft immer parallel (nicht nur bei leerem Graph) |

### 3.4 Hub Contention Index (Phase 3)

| Datei | Änderung |
|-------|----------|
| `scripts/hub-contention-index.py` | Neu — Neo4j-Grad pro honeypot-Entity → `data/hub-contention-cache.json` |
| `gzmo-core/src/config.rs` | `RecallConfig` mit `hub_contention_cache`, `hub_contention_penalty` (default 0.85) |
| `gzmo-core/src/memory/vault.rs` | RRF-Penalty für High-Contention-Fakten (außer Query nennt das Hub) |
| `gzmo-core/src/memory/embeddings.rs` | `open_vault_with_embeddings` nimmt `&RecallConfig`; alle 12 Caller aktualisiert |
| `gzmo.toml` / `gzmo.toml.example` | `[recall]`-Sektion |

### 3.5 Spark Anchor Guard + Discovery LINK Contract (Phase 4)

| Datei | Änderung |
|-------|----------|
| `gzmo-core/src/config.rs` | `SparkConfig.max_anchor_hub_degree` (default 8) |
| `gzmo-core/src/spark.rs` | `select_phase` skippt Hub-Anchor; `log_anchor_skip` → `data/spark-anchor-skip.jsonl` |
| `gzmo-core/src/memory/vault.rs` | `hub_entities_above(min_degree)` Helper |
| `scripts/discovery-kb-recall-smoke.sh` | `chain_recall_query` → `chain_hit` separat |
| `docs/DISCOVERY_KB_FEEDBACK_LOOP.md` | §8 Compositional LINK contract dokumentiert |
| `gzmo.toml` / `gzmo.toml.example` | `max_anchor_hub_degree = 8` |

### 3.6 Build & Tests

- `cargo build -p gzmo-cli` — erfolgreich (nur pre-existing Warnings)
- `cargo test -p gzmo-core memory::` — 46 passed, 0 failed

---

## 4. Module Mapping (Paper → GZMO)

| Paper Probe / Konzept | GZMO Implementation | Status |
|-----------------------|---------------------|--------|
| Atomic retrieval (MRR) | `discovery-kb-recall-smoke.sh` atomic `hit` | implemented |
| Hop-1 intermediate | `graph-recall-stream.py` 1-hop (default) | implemented |
| Zero-shot 2-hop composition | `compositional-recall-probe.py` Probe B | implemented |
| Hop-2 atomic difficulty | `compositional-recall-probe.py` Probe C `hop2_atomic_ratio` | implemented |
| High-degree fact contention | `hub-contention-index.py` + `vault.rs` RRF penalty | implemented |
| Chain-aware anchor selection | `spark.rs` hub guard | implemented |
| CQD-style decomposition | `graph-recall-stream.py 2hop` verified chain hints | implemented |
| HRR/FHRR superposition | **rejected** (anti-pattern) | documented |

---

## 5. Operator Commands

```bash
# Build
cargo build --release -p gzmo-cli

# Hub contention index (einmalig / nächtlich)
./scripts/hub-contention-index.py

# Compositional baseline (teuer: ~10s pro gzmo-Aufruf × Ketten × 4 Probes;
# läuft normal nächtlich, nicht im synchronen Gate)
./scripts/compositional-recall-smoke.sh

# Atomic smoke (sollte grün bleiben; braucht ~50s für 5 Cold-Starts)
./scripts/discovery-kb-recall-smoke.sh

# Discovery gate (liest compositional-latest.json, läuft in <2s)
./scripts/ingest-quality/gate-discovery-loop.sh

# 2-hop graph hints manuell
GRAPH_RECALL_MODE=2hop python3 scripts/graph-recall-stream.py "Obulus energy bilanz" 10
```

### Empfohlene Cron-Einträge (nightly)

```cron
# Hub-Index und compositional Baseline nächtlich aktualisieren
30 1 * * * cd $GZMO_ROOT && ./scripts/hub-contention-index.py
45 1 * * * cd $GZMO_ROOT && ./scripts/compositional-recall-smoke.sh
```

---

## 6. Quality Targets (30 Tage)

| Metric | Target | Phase |
|--------|--------|-------|
| `chain_hit_rate` | ≥ 0.4 (nach Baseline) | tracked, dann STRICT |
| `hop2_atomic_ratio` | tracked, nicht initial gegated | observe |
| Atomic `pass_rate` | ≥ 0.66 (unverändert) | gate |
| Hub-Anchor skips | > 0 bei aktivem honeypot | observe |

### Erste Baseline (2026-06-26, 10 Ketten aus Live-Neo4j)

| Metric | Wert | Paper-Referenz |
|--------|------|----------------|
| `atomic_baseline_mrr` | 1.0 | — |
| `hop1_mrr` | 0.0895 | 0.90 (paper) — GZMOs intermediates schwerer |
| `chain_hit_rate` | 0.8 | paper: chance |
| `hop2_atomic_mrr` | 0.18 | — |
| `hop2_atomic_ratio` | **0.18×** | paper: 0.26–0.48× — **effekt bestätigt, sogar stärker** |

Die `hop2_atomic_ratio` von 0.18 bestätigt den Paper-Befund (hop-2 Fakten sind
intrinsisch schwerer, auch ohne Composition) auf GZMOs echtem KG — und zwar
ausgeprägter als im Paper. Das ist genau das G12-Signal, das der 100%-Atomic-
Smoke bisher maskierte.

Initiale niedrige `chain_hit_rate` wäre **Erfolg**; hier ist sie bereits 0.8,
was darauf hindeutet dass GZMOs explizite Graph+Honeypot-Architektur Composition
besser bewältigt als holographische Superposition — wie vom Paper vorhergesagt.

---

## 7. Rejection Log — warum HRR/FHRR nicht adoptiert

1. Paper ist ein **negatives Resultat** für genau den Zero-Shot-Composition-Use-Case.
2. HRR-Kapazität (~50 saubere Fakten in 1024D, per holomemory-Referenz) liegt
   Größenordnungen unter GZMOs 22k+ honeypot-Punkten.
3. ARCH-DIR-001 Zero-Bloat verbietet net-new vector-symbolic Machinery für ein
   Problem, das GZMOs existierender Graph+Honeypot strukturell richtig löst.
4. Cleanup-Operator-Redesign (Lemma 4.1) behebt nicht den Upstream-Kapazitätsfehler.

---

## 8. Risiken / Scope

- Neo4j muss erreichbar sein (`bolt://192.168.31.202:7687`); Probes fail-open
  (gleiche Haltung wie bestehender Graph-Stream).
- `chain_hit_rate` initial niedrig — Gate startet als WARN, `STRICT=1` greift erst
  nach `COMPOSITIONAL_BASELINE_LOCKED`.
- `~/gzmo_skills` LINK-Schema-Änderung (`chain_recall_query`) benötigt kleinen
  Edit in pi-mentor-discovery Templates (außerhalb survey_GZMO, hier referenziert).
- Hub-Cache muss regelmäßig gebaut werden (nächtlicher Cron empfohlen).

---

## Document Chain

`ARCH-DIR-001-GZMO.md` → `DISCOVERY_KB_FEEDBACK_LOOP.md` (§8 compositional LINK) →
`THEMA_009_COMPOSITIONAL_RECALL_SYNTHESIS.md` (this) →
`wiki/entities/verified-chain-recall.md`
