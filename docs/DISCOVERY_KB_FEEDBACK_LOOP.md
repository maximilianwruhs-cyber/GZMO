# Discovery ↔ Knowledge Base Feedback Loop

> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`. Living paths: [CT101_PATH_AUTHORITY.md](./CT101_PATH_AUTHORITY.md).

**Status:** Normative reference (2026-06-18)  
**Scope:** How pi-mentor-discovery cycles read from and write back to GZMO memory — and how `gzmo_daemon` orchestrates the loop.

**Related:**

| Doc | Role |
|-----|------|
| [MEMORY_ARCHITECTURE_SPEC.md](./MEMORY_ARCHITECTURE_SPEC.md) | Vault, honeypot, recall tiers |
| [DISTILL_COLD_CHAIN.md](./DISTILL_COLD_CHAIN.md) | Distill ingress paths |
| [SYNAPSE_EVENT_OWNERSHIP.md](./SYNAPSE_EVENT_OWNERSHIP.md) | Pi vs daemon event writers |
| [INFRASTRUCTURE_OVERVIEW.md](./INFRASTRUCTURE_OVERVIEW.md) §6 | Nightly daemon loop |
| [DISCOVERY_LIFECYCLE.md](./DISCOVERY_LIFECYCLE.md) | Scout vs implement (INLINE/drain overrides older maximal-session narration) |
| [MUTUAL_DISCOVERY_THEATER.md](./MUTUAL_DISCOVERY_THEATER.md) | Human pedagogy theater front door (not scout KPI) |
| [research/mutual-discovery/SOCRATIC_FORUM_THREE_MODES.md](./research/mutual-discovery/SOCRATIC_FORUM_THREE_MODES.md) | Modes A/B/C research (Mode A ≠ scout KPI) |
| [research/mutual-discovery/PI_GZMO_SOCRATIC_KNOWLEDGE_DIALOGUE.md](./research/mutual-discovery/PI_GZMO_SOCRATIC_KNOWLEDGE_DIALOGUE.md) | Emergent mutual-discovery method |

---

## 1. Three orthogonal axes

Discovery architecture uses **three independent dimensions**. Do not collapse them into a single pillar.

| Axis | Question | Examples |
|------|----------|----------|
| **Pillars S/A/B/C** | Which stack layer is audited? | S = honeypot/pedagogy; B = distill/daemon |
| **gzmo_daemon** | Who triggers, writes, gates? | Low-tension spawn; synapse distill; Dream/Spark cron |
| **KB stores** | Where is knowledge retrievable? | Vault → honeypot → Qdrant; wiki (emit-only); Pi `knowledge` collection |

**Pillar B audits the daemon domain; the daemon runs the KB loop for all pillars.**

---

## 2. End-to-end lifecycle

```
TRIGGER (daemon)
  PulseLoop τ ↓ OR pedagogy oscillation low phase
  → ObolusGate T1 (DiscoveryCycle)
  → build_opening() reads vault + prerequisite graph + Neo4j dialogue hints
  → spawn auto-socratic-discovery-cycle.sh

DISCOVERY (external child — gzmo_skills)
  pick pillar → probe plan → Pi dialogue + mentor_teach
  → cycle report (LINK lines + KB Impact closure)
  → link-registry.jsonl append (fingerprints)
  → arc distill: gzmo distill pi <jsonl>  (batched)

KB WRITE (daemon engines)
  SessionDistillEngine → vault (≥0.85) → honeypot eligibility → Neo4j
  Synapse poll session_end → distill pi (dedup coordinated)
  Nightly: Dream 01:00, Qdrant sync 01:45, session distill 02:15, Spark 03:30/22:30

KB READ (next cycle)
  gzmo_memory_search (recall_rrf on honeypot + optional Pi knowledge)
  build_opening() recent_semantic_facts
  Spark anchor pools (SessionDistill decay class)
```

### Critical invariant

**Markdown cycle reports are not KB.** Durable recall comes from **Pi arc JSONL distill**, not report files. LINK lines in reports are **promotion candidates** tracked in `link-registry.jsonl` and verified after distill.

---

## 3. Read contract

During discovery, Pi may read:

| Tool / store | Layer | Pillar bias |
|--------------|-------|-------------|
| `gzmo_memory_search` (MCP) | Honeypot RRF + scratch | S, A, C |
| `knowledge_search` | Pi Qdrant `knowledge` | S |
| `gzmo_wiki_search` | Wiki emit-only | A, C |
| `read` | Document layer | all |
| `gzmo_health` | Infrastructure | C |

Configured per pillar in `~/gzmo_skills/data/pi-mentor-discovery/pillars.json` → `kb_touchpoints.read`.

---

## 4. Write contract

| Path | Trigger | Target stores |
|------|---------|---------------|
| **Arc distill** | Every N cycles / token budget / stale max | Vault, honeypot, Neo4j via SessionDistillEngine |
| **session_end distill** | Daemon synapse poll (60s) | Same engine; dedup with arc distill |
| **Curated ingest** (operator) | Manual `gzmo ingest` on `knowledge/curated/discovery-*.md` | Full ingest pipeline |
| **Kurator fixer** | FAIL/GAP in reports | Code/config — indirect KB via tools |

Promotion gates (unchanged): verify pass, vault confidence ≥0.85, honeypot rules in `honeypot.rs`.

**Provenance env** (set by discovery shell before `gzmo distill pi`):

- `GZMO_DISCOVERY_CYCLE`
- `GZMO_DISCOVERY_PILLAR`
- `GZMO_CORRELATION_ID`

Synthetic source path: `sessions/discovery-{cycle}.md` (honeypot-eligible, traceable).

---

## 5. Recall contract

After promotion, recall is verified by:

1. **`scripts/discovery-kb-metrics.sh`** — baseline dashboard
2. **`scripts/discovery-kb-recall-smoke.sh`** — top-3 hits for LINK `recall_query` (Phase 3)
3. Post-distill hook in `run-remediation-hooks.sh post-distill`

Primary recall path: `SqliteVault::search_recall` → 6-stream RRF on honeypot → VM200 rerank.

---

## 6. gzmo_daemon orchestrator map

| Subsystem | Schedule | KB read | KB write |
|-----------|----------|---------|----------|
| Low-tension watcher | 5s | Vault (opening) | Spawns discovery |
| Synapse poll | 60s | events.jsonl | Episodic; session_end → distill |
| Distill worker | Redis BRPOP | Archive queue | Vault, Neo4j |
| DreamEngine | 01:00 UTC | Episodic + honeypot REM | Vault, Neo4j |
| SparkEngine | 03:30, 22:30 | Honeypot pools | Neo4j HYPOTHESIZED_LINK |
| IngestEngine | FS watcher | Files | Vault, honeypot |
| Qdrant sync | 01:45 UTC | Vault embeddings | Qdrant honeypot |
| Session distill cron | 02:15 UTC | data/sessions/*.json | Vault |
| Kurator monitor | with synapse poll | Reports (CLI) | spawn.recommended only |

Entry: [`gzmo-cli/src/daemon_cmd.rs`](../gzmo-cli/src/daemon_cmd.rs)

---

## 7. Synapse events (discovery ↔ KB)

| event_type | Owner | KB effect |
|------------|-------|-----------|
| `mentor_teach` | Pi | Episodic summary (via poll) |
| `session_end` | Pi | Triggers `gzmo distill pi` |
| `distill_complete` | GZMO | Audit; carries promotion metadata |
| `discovery_session_start/end` | gzmo_skills | Session boundary audit |
| `pedagogy.oscillation_*` | GZMO | Discovery spawn; knowledge_delta |

---

## 8. LINK / GAP typology

**LINK** (promotion candidate):

```text
LINK: L01: source —relationship→ target | EVIDENCE: path or command | WHY: one phrase
```

Optional in KB Impact table: `recall_query` — phrase for post-distill smoke.

**GAP** (KB silent):

```text
GAP: topic X has no honeypot coverage | suggested_store: vault|honeypot|wiki
```

Registry: `~/gzmo_skills/data/pi-mentor-discovery/link-registry.jsonl`  
Fingerprint: `sha256(normalize(source + relationship + target))`

---

## 9. Known gaps (tracked)

| ID | Gap | Phase |
|----|-----|-------|
| G1 | Reports not auto-ingested | 1 — link registry + distill provenance |
| G2 | ~96% distill dedup discard | 1 — novelty gate before arc distill |
| G3 | spark_distill_bridge logging only | 3 |
| G4 | Triple distill paths | 2 — coordination |
| G5 | Empty knowledge_delta on oscillation complete | 2 — vault metrics |
| G6 | topic_shift_distill stub | 4 |
| G7 | platform_search concat not RRF | 3 |
| G8 | Qdrant lag until 01:45 | 3 — post-distill sync |
| G9 | AUTO discovery redundancy | 3 — fingerprint defer |
| G10 | Kurator nondeterministic KB | by design |
| G11 | Neo4j endpoint gate drops ops edges | 4 — DISTILL_ENDPOINT_POLICY whitelists |
| G12 | Eval green ≠ recall green | 4 — DISCOVERY_LOOP gate |

---

## 10. Operator commands

```bash
# Baseline metrics
./scripts/discovery-kb-metrics.sh

# Recall smoke (after distill)
./scripts/discovery-kb-recall-smoke.sh

# Manual discovery session
cd ~/gzmo_skills && ./scripts/start-pi-mentor-discovery-session.sh

# Check spawn log (AUTO)
tail ~/gzmo_skills/data/pi-mentor-discovery/logs/auto-socratic-spawn.log
```

---

## 11. Quality targets (30 days)

| Metric | Target |
|--------|--------|
| Novel LINKs / discovery cycle | ≥2 |
| Distill dedup skip rate | <50% |
| Discovery-sourced honeypot facts | ≥15/week |
| recall-smoke pass rate | ≥66% |
| Double distill / session | <1% |

Measure via `./scripts/discovery-kb-metrics.sh` → `data/discovery-kb-metrics/latest.json`.

Optional eval gate: `DISCOVERY_LOOP=1 scripts/ingest-quality/eval-quick.sh`

---

## 12. Curated promotion (operator-gated)

High-signal LINK lines from session finals may be promoted without auto-ingest:

1. Copy crucial LINKs to `~/Schreibtisch/knowledge/curated/discovery-{cycle_id}.md`
2. Header: `# [DISCOVERY:cycle-N] pillar=S`
3. Run: `gzmo ingest ~/Schreibtisch/knowledge/curated/discovery-{cycle_id}.md`

This path uses the full ingest pipeline (verify + honeypot rules), unlike raw report markdown.
