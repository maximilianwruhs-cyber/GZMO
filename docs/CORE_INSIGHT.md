# GZMO — Core Insight (living-instance prefill)

**Status:** 2026-07-17 (header corrected for CT101 restore)  
**Role:** Operator-curated, high-density self-knowledge for the **living stack**.  
**Living brain:** CT101 (`/opt/gzmo/`). Workstation `data-next/` is lab scratch only — see [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md).  
**Consumers:** Humans (read), `scripts/seed-core-stack.py` (inject `[TYPE:Name]` bullets into vault + honeypot), then embed + Qdrant sync — **target CT101** unless explicitly seeding a lab vault (`GZMO_ALLOW_LAB_VAULT=1`).  
**Supersedes for living ops:** Stale claims in `docs/CORE_STACK_KNOWLEDGE.md` (2026-06-07 clean-slate era) and outdated SOUL heuristics that still name `gzmo-scheduler` as overnight authority.

> Prefill order after empty nights: seed this document → embed → sync Qdrant → only then expect Dream/Spark to have anchors that are not the same SessionDistill loop.

---

## How to read / extend

Every entity is one card:

```
## [TYPE:Name]
- What / How / Use / Why / Related
- Injected facts:
  - [TYPE:Name] one dense, self-contained statement.
```

**Injection conventions** (match `scripts/seed-core-stack.py`):
`decay_class = Structural`, `confidence = 0.95`, `origin = manual`, `container = obolus`,
`source_file = manual/core_insight_20260717.md`.

**Type tags:** `NODE`, `SERVICE`, `MODEL`, `CONFIG`, `CONCEPT`, `PROC`, `SYSTEM`, `PATH`, `HOWTO`, `POLICY`, `STATE`, `LESSON`.

**Seed (living instance = CT101):**

```bash
# Seed the living vault on CT101 (not data-next/):
scp docs/CORE_INSIGHT.md manual/core_insight_20260717.md ct101:/tmp/
ssh ct101 'bash -lc "
  cd /opt/gzmo/current
  GZMO_CONFIG=/opt/gzmo/gzmo.toml python3 scripts/seed-core-stack.py \
    --doc /tmp/CORE_INSIGHT.md \
    --db /opt/gzmo/data/vault.db \
    --source-file manual/core_insight_20260717.md
  GZMO_CONFIG=/opt/gzmo/gzmo.toml ./target/release/gzmo memory embed
  python3 scripts/sync-vault-to-qdrant.py --db /opt/gzmo/data/vault.db --source honeypot
"'
# Lab only: seed data-next/ with GZMO_ALLOW_LAB_VAULT=1 — never merge into CT101.
```

---

## Domain 0 — Identity (non-negotiable)

### [CONCEPT:GZMO]
- What: A sovereign local agent whose product is a **distillation pipeline**, not a chatbot with a vector attachment.
- How: LLM extracts/verifies; pipeline stores (vault → honeypot → Qdrant / optional ripen → knowledge_core).
- Use: Every feature is judged by whether it improves verified memory or overnight compounding.
- Why: Without the pipeline, presence theater (faces, banners) lies about being alive.
- Related: [CONCEPT:FourLayers], [PROC:Ingest], [POLICY:CurationFirst]
- Injected facts:
  - [CONCEPT:GZMO] GZMO is a distillation pipeline: the LLM thinks (extract, verify, dream) and the pipeline remembers (vault, honeypot, Qdrant, optional knowledge_core).
  - [CONCEPT:GZMO] Honeypot + verify + promote = GZMO; a chatbot with a memory attachment is the wrong product shape.
  - [CONCEPT:GZMO] GZMO is not Telegram/OpenClaw product track, not bulk Takeout ingest, and not a Mem0/Zep/Supermemory reimplementation.

### [POLICY:Sovereignty]
- What: Primary cognition stays local; retrieval sidecars may be remote LAN.
- How: Prime on workstation `:8000`; embed/rerank on VM200; secrets only in `.env`.
- Use: Prefer `/mode local`; cloud is optional burst.
- Why: Operator owns the vault and the weights.
- Related: [SERVICE:Prime], [NODE:Workstation]
- Injected facts:
  - [POLICY:Sovereignty] Primary inference is local Prime on the workstation; VM200 is for embed/rerank only; secrets stay in .env and are never committed.

---

## Domain 1 — Living instance vs CT101 (ADR-0003)

### [POLICY:ADR0003]
- What: One living instance only; living host is CT101 (amended 2026-07-17).
- How: CT101 `gzmo-daemon` + `/opt/gzmo/` vault; workstation = operator + Prime fallback; never two overnight writers.
- Use: Production ops via `ssh ct101` / `pct exec 101`; workstation CLI/chat for operator work only.
- Why: Dual-stack forever was drowning the overnight product claim; 2026-07-15 workstation cutover was reversed.
- Related: [POLICY:CT101Living], [SYSTEM:GzmoDaemon], [PATH:opt_gzmo]
- Injected facts:
  - [POLICY:ADR0003] ADR-0003 (2026-07-16, host amended 2026-07-17): one living instance — CT101 gzmo-daemon is production metabolism; workstation is not a second overnight brain.
  - [POLICY:ADR0003] Living overnight metabolism is CT101 gzmo-daemon.service (cloud-first + Prime fallback) — not workstation gzmo-serve or gzmo-scheduler.
  - [POLICY:ADR0003] Workstation gzmo/gzmo chat and gzmo memory mcp remain operator/lab surfaces; Observatory over data-next/ is a lab viewer, not production control plane.
  - [POLICY:ADR0003] Product gate is CT101 systemd + journal + vault/honeypot counts + Docker sidecars (see CT101_RESTORE_LIVING.md).
  - [POLICY:ADR0003] Never enable workstation gzmo-serve overnight while CT101 gzmo-daemon is the living writer.

### [POLICY:CT101Living]
- What: CT101 (`/opt/gzmo/...`, cloud-first daemon, ~60k vault) is the living production host.
- How: Keep gzmo-daemon + sidecars healthy; do not graft lab loops into CT101 gzmo.toml; do not import data-next into CT101 vault.
- Use: Operate production on CT101; treat workstation data-next/ as lab/dev scratch only.
- Why: The 60k curated vault and colocated sidecars are the compounding memory plane.
- Related: [LESSON:CT101Scale], [LESSON:DualStackTrap], [STATE:LivingRestore]
- Injected facts:
  - [POLICY:CT101Living] CT101 is living production as of 2026-07-17 restore; never edit CT101 gzmo.toml to point loops at lab scripts.
  - [POLICY:CT101Living] Cutover 2026-07-15 briefly put production on workstation data-next/; restore 2026-07-17 returned authority to CT101 without vault merge from data-next/.

### [LESSON:CT101Scale]
- What: CT101 proved the stack can be operationally mature at large scale.
- How: Live probe era ~60k vault / ~37k honeypot / ~24k Qdrant points / large Synapse bus / Neo4j graph; cloud-first cognition with Prime fallback.
- Use: Keep quality gates and honeypot discipline on the living CT101 host.
- Why: Scale without curation is noise; scale with gates is the moat.
- Related: [CONCEPT:Honeypot], [POLICY:CurationFirst]
- Injected facts:
  - [LESSON:CT101Scale] CT101 demonstrated production-scale memory (~60k vault, ~37k honeypot, ~24k Qdrant points) with verify gates — proof the pipeline works when curation holds.
  - [LESSON:CT101Scale] CT101 capability maturity is per-subsystem (dream/spark/ingest/memory/MCP/synapse), not a binary up/down for the whole machine.

### [LESSON:DualStackTrap]
- What: Keeping CT101 and next as co-equal product brains created status lies and authority fights.
- How: Agents reported inactive workstation units as outages while the real overnight runner was elsewhere; SOUL heuristics lagged ADR amendments.
- Use: When units disagree with docs, trust ADR-0003 (amended) + CT101 daemon journal + vault counts.
- Why: Observability without authority model is worse than silence.
- Related: [POLICY:ADR0003], [POLICY:CT101Living], [SYSTEM:GzmoServe]
- Injected facts:
  - [LESSON:DualStackTrap] After 2026-07-17 restore, inactive gzmo-serve/gzmo-scheduler on the workstation is expected — CT101 gzmo-daemon is the overnight authority.
  - [LESSON:DualStackTrap] Never freestyle ecosystem status from inactive workstation unit LEDs alone — check CT101 gzmo-daemon and vault counts.

### [SYSTEM:GzmoServe]
- What: Thin overnight metabolism runner for **lab/dev** on the workstation (not production after 2026-07-17).
- How: `gzmo serve` / `gzmo-serve.service`; typed Rust jobs; writes `data-next/scheduler-runs/`; soft-fail OKForge wiki satellite.
- Use: Keep disabled by default; only for explicit lab/beat-gate sessions with CT101 overnight writers stopped.
- Why: One long-running authority for overnight compounding.
- Related: [PROC:Metabolism], [PATH:scheduler_runs]
- Injected facts:
  - [SYSTEM:GzmoServe] gzmo-serve.service is lab/dev only after 2026-07-17; never enable overnight alongside CT101 gzmo-daemon.
  - [SYSTEM:GzmoServe] gzmo serve ignores lab assembly backends; [assembly]=lab only affects transitional gzmo daemon / gzmo assemble.

### [SYSTEM:GzmoScheduler]
- What: Thin lab-recipe cron for beat-gates only.
- How: `gzmo-scheduler.service` — disabled by default after cutover; must not share overnight authority with gzmo-serve.
- Use: Explicit beat-gate sessions only: stop serve → start scheduler → stop scheduler → start serve.
- Why: Lab parity without poisoning living metabolism.
- Related: [POLICY:ADR0003], [SYSTEM:GzmoServe]
- Injected facts:
  - [SYSTEM:GzmoScheduler] gzmo-scheduler.service is the lab recipe cron for beat-gates only; it stays disabled/offline by default and must not run overnight alongside gzmo-serve.
  - [SYSTEM:GzmoScheduler] An inactive gzmo-scheduler unit after the 2026-07-16 cutover is expected, not an outage.

---

## Domain 2 — Topology (living = CT101)

### [NODE:CT101]
- What: Sole living metabolism host (daemon + vault + Docker sidecars).
- How: LXC 101 `192.168.31.202` — `/opt/gzmo/`, `gzmo-daemon.service`, cloud-first cognition.
- Use: Production ops via `ssh ct101` / `pct exec 101`; product gate `scripts/ct101-living-smoke.sh`.
- Why: 24/7 overnight writer colocated with Redis/Qdrant/Neo4j.
- Related: [SERVICE:Prime], [NODE:Workstation], [NODE:VM200], [PATH:opt_gzmo]
- Injected facts:
  - [NODE:CT101] CT101 (192.168.31.202) is the living GZMO brain: gzmo-daemon, /opt/gzmo vault (~60k facts), Docker Redis/Qdrant/Neo4j, mentor socket at /opt/gzmo/data/gzmo_mentor.sock.
  - [NODE:CT101] Build release binaries on CT101; workstation glibc is newer and scp'd gzmo binaries fail with missing GLIBC.

### [NODE:Workstation]
- What: Operator frontend + Prime fallback host (not overnight metabolism).
- How: Ryzen 9950X, dual RTX 5070 Ti (no NVLink → layer-split Prime over PCIe).
- Use: Cursor/Pi/CLI; Prime at `:8000`; lab `data-next/` only with overnight writers off.
- Why: Local weights and interactive tooling without dual overnight writers.
- Related: [SERVICE:Prime], [POLICY:CT101Living]
- Injected facts:
  - [NODE:Workstation] The workstation (192.168.31.184) is operator + Prime fallback — not the living overnight brain after the 2026-07-17 CT101 restore.
  - [NODE:Workstation] Without NVLink, Prime is layer-split across both GPUs over PCIe; CUDA graph capture must stay disabled when it corrupts dual-GPU output.

### [NODE:VM200]
- What: Retrieval model node (embeddings + rerank).
- How: `192.168.31.110` — see living `/opt/gzmo/gzmo.toml` `[embeddings]` / `[rerank]`.
- Use: Configured on CT101 living toml.
- Why: Offload frequent light inference from workstation GPUs.
- Related: [SERVICE:Embed], [SERVICE:Rerank]
- Injected facts:
  - [NODE:VM200] VM200 (192.168.31.110) is the retrieval layer for living CT101: embeddings and rerank per /opt/gzmo/gzmo.toml.

### [NODE:Sidecars]
- What: Docker persistence colocated with the living daemon on CT101.
- How: Redis `:6379`, Qdrant `:6333`, Neo4j `:7687` on CT101.
- Use: Living `/opt/gzmo/gzmo.toml` points at these.
- Why: Low-latency vault↔vector↔graph on one LXC.
- Related: [SERVICE:Qdrant], [SERVICE:Redis], [SERVICE:Neo4j]
- Injected facts:
  - [NODE:Sidecars] Living Redis, Qdrant, and Neo4j run as Docker sidecars on CT101; workstation database-cluster ports are lab-only.
  - [SERVICE:Qdrant] Living Qdrant is http://localhost:6333 collection honeypot (1024-dim) on CT101; SQLite honeypot remains source of truth.
  - [SERVICE:Redis] Living Redis is redis://localhost:6379 with distill queue gzmo:distill:pending on CT101.

### [SERVICE:Prime]
- What: Local cognition fallback on the workstation.
- How: `http://192.168.31.184:8000/v1` from CT101.
- Use: Cloud-first daemon falls back here.
- Why: Sovereign weights without dual overnight writers.
- Related: [MODEL:Prime], [HOWTO:EngineHealth]
- Injected facts:
  - [SERVICE:Prime] From CT101, Prime is http://192.168.31.184:8000/v1 — never assume localhost:8000 tunnels on the living host.
  - [SERVICE:Prime] systemd unit llama-prime.service may be inactive while a manual llama-server still answers on :8000 — judge LLM health by /v1/models, not unit LED alone.
  - [SERVICE:Parked] Sovereign FrankenMoE :8010 is intentionally not required; do not block living ops on :8010.

### [MODEL:Prime]
- Injected facts:
  - [MODEL:Prime] Living Prime fallback is a local llama.cpp GGUF on the workstation served at :8000; confirm via /v1/models and /opt/gzmo/gzmo.toml [engine.local].

### [SERVICE:Embed]
- Injected facts:
  - [SERVICE:Embed] Living vault/honeypot embeddings come from VM200 per [embeddings] in /opt/gzmo/gzmo.toml (typically :8081, 1024-dim).

### [SERVICE:Rerank]
- Injected facts:
  - [SERVICE:Rerank] Living recall is post-filtered by VM200 reranker after RRF fusion when [rerank] is enabled in /opt/gzmo/gzmo.toml.

### [SERVICE:Neo4j]
- Injected facts:
  - [SERVICE:Neo4j] Neo4j on CT101 holds entity/relation graph via mcp-neo4j-memory; written by ingest/dream/spark when MCP is up.

### [CONFIG:gzmo_toml]
- Injected facts:
  - [CONFIG:gzmo_toml] Living config is /opt/gzmo/gzmo.toml with data under /opt/gzmo/data/; always export GZMO_CONFIG=/opt/gzmo/gzmo.toml on CT101.
  - [CONFIG:gzmo_toml] Skills-cwd or lab gzmo.toml (localhost:8000, tiny vault) must never hijack selfheal/discovery — quarantine those files.

---

## Domain 3 — Paths (living)

### [PATH:opt_gzmo]
- What: Living runtime root on CT101.
- How: `/opt/gzmo/` — `gzmo.toml`, `data/`, `.env`; `/opt/gzmo/current` → release tree.
- Use: Daemon WorkingDirectory; mentor socket → `/opt/gzmo/data/gzmo_mentor.sock`.
- Why: Stable public paths; do not treat `survey_GZMO` as the SoT name.
- Related: [PATH:vault_db], [NODE:CT101]
- Injected facts:
  - [PATH:opt_gzmo] Living runtime is /opt/gzmo/ with binary at /opt/gzmo/current/target/release/gzmo and config /opt/gzmo/gzmo.toml.
  - [PATH:vault_db] Living SQLite authority is /opt/gzmo/data/vault.db (semantic_vault + honeypot + evidence); stop writers before destructive maintenance.
  - [PATH:mentor_sock] Living Unix mentor API is /opt/gzmo/data/gzmo_mentor.sock (ping/status/teach); chaos-free, dedicated daemon thread.
  - [PATH:data_next] Workstation data-next/ is lab/dev scratch only after 2026-07-17 restore — never merge into CT101 vault without an explicit migrate ADR.
  - [PATH:wiki_dir] wiki/ is git-tracked emit-only synthesis; never re-ingest wiki pages into vault/honeypot.

---

## Domain 4 — Memory model (corrected truths)

### [CONCEPT:FourLayers]
- Injected facts:
  - [CONCEPT:FourLayers] Memory has four layers: vault (all verified facts), honeypot (Tier-1 curated), evidence (Tier-2 spans), knowledge_core (M5 ripened); Qdrant mirrors honeypot only.
  - [CONCEPT:Honeypot] Honeypot qualifies facts at confidence >=0.85 with non-empty source_file, excluding [relation:] rows and low-trust source patterns — quality over coverage.
  - [CONCEPT:Honeypot] Honeypot is the default for recall, Dream/Spark anchors, and Qdrant sync — not the full vault soup.
  - [CONCEPT:Evidence] Evidence stores quotable source spans 1:1 with honeypot facts for strict recall grounding.
  - [CONCEPT:Episodic] Episodic markdown is the raw daily stream and dream substrate; it is not the primary RAG store.
  - [CONCEPT:Wiki] Wiki pages are derived/emit-only and must never be circularly re-ingested into honeypot.

### [CONCEPT:SessionDistillPaths]
- What: Correct model of SessionDistill vs episodic directories (fixes stale spark loop).
- How: Distill reads session JSON under `/opt/gzmo/data/sessions/`; episodic lives under `/opt/gzmo/data/memory/`; synthetic path labels in facts are metadata, not proof that `memory/` is empty or unused.
- Use: When Spark repeats “memory/ is empty,” check `/opt/gzmo/data/memory/` and whether Dream had today's episodic — do not invent a consolidation outage.
- Why: Overnight 2026-07-16/17 sparks looped on a wrong empty-memory story while lab `data-next/memory/` had day files and serve ran.
- Related: [SYSTEM:SessionDistill], [SYSTEM:Dream], [PATH:memory_dir]
- Injected facts:
  - [CONCEPT:SessionDistillPaths] SessionDistill consumes /opt/gzmo/data/sessions/*.json; it does not write the primary episodic store — episodic appends go to /opt/gzmo/data/memory/YYYY-MM-DD.md.
  - [CONCEPT:SessionDistillPaths] A Spark hypothesis that memory/ is empty is often looking at the wrong path or treating synthetic session path labels as filesystem emptiness — verify /opt/gzmo/data/memory/ before concluding consolidation is broken.
  - [CONCEPT:SessionDistillPaths] Spark may promote low-confidence (0.6) quarantine links and re-anchor on the same SessionDistill fact; that is serendipity noise, not proof overnight metabolism failed.

### [PROC:Ingest]
- Injected facts:
  - [PROC:Ingest] Ingest: prep → Prime extract (:8000, low temp) → verify → promote → vault → honeypot+evidence if qualified → Neo4j MCP → episodic receipt → optional wiki emit → later Qdrant sync.
  - [PROC:Ingest] gzmo ingest-eval is dry-run contract only; only live gzmo ingest writes memory stores.
  - [PROC:Recall] Recall fuses honeypot FTS, evidence FTS, graph/keyword, and vectors via RRF, then reranks when configured.

### [PROC:Metabolism]
- What: Overnight job chain on CT101 gzmo-daemon.
- How: Daemon-scheduled dream/spark/distill/promote/embed/qdrant sync per /opt/gzmo/gzmo.toml.
- Use: Read journalctl -u gzmo-daemon and gzmo health on CT101.
- Why: Compounding without human presence.
- Related: [NODE:CT101], [SYSTEM:Dream], [SYSTEM:Spark]
- Injected facts:
  - [PROC:Metabolism] Living overnight metabolism is CT101 gzmo-daemon (cloud-first + Prime fallback) — not workstation gzmo-serve.
  - [PROC:Metabolism] Dream skipping with No episodic data for DATE is a successful no-op when that day's /opt/gzmo/data/memory file lacks consolidatable content — not a daemon crash.
  - [PROC:Metabolism] Promote reporting 0 candidates and embed nothing-to-backfill can still be a healthy night if prior facts already matured and embeddings exist.
  - [PROC:Metabolism] Honeypot↔Qdrant drift WARN usually means missing honeypot embeddings (is_latest without vectors), not a dead Qdrant — run gzmo memory embed then sync.

---

## Domain 5 — Cognition engines (living defaults)

### [SYSTEM:Dream]
- Injected facts:
  - [SYSTEM:Dream] DreamEngine consolidates episodic logs into vault/honeypot/Neo4j with verify (min_confidence typically 0.85); living schedule is CT101 gzmo-daemon cron windows.
  - [SYSTEM:Dream] Manual dream: gzmo dream; overnight narrative under /opt/gzmo/data/.

### [SYSTEM:SessionDistill]
- Injected facts:
  - [SYSTEM:SessionDistill] SessionDistill (~02:15 UTC living) distills /opt/gzmo/data/sessions into vault/honeypot; corrupt session JSON without is_meta is skipped with a warning.

### [SYSTEM:Spark]
- Injected facts:
  - [SYSTEM:Spark] Spark samples honeypot anchors and may write Neo4j HYPOTHESIZED_LINK at quarantine confidence ~0.6; it must not be treated as honeypot truth.
  - [SYSTEM:Spark] Repeated Spark text about SessionDistill vs empty memory across nights is a stale-anchor loop — seed core insight and diversify honeypot anchors rather than restarting gzmo-scheduler.

### [SYSTEM:WikiEngine]
- Injected facts:
  - [SYSTEM:WikiEngine] Wiki sync/lint/search are gardener jobs; OKForge push publishes concepts externally when OKFORGE_TOKEN is configured — soft-fail relative to metabolism GREEN.

### [SYSTEM:Chaos]
- Injected facts:
  - [SYSTEM:Chaos] Chaos PulseLoop is opt-in for stdio chat and forced live in gzmo --repl ops console; metabolism does not depend on chaos.
  - [SYSTEM:Chaos] Instruments ONLINE/FALLEN mean chaos.alive, not LLM reachability — keep those signals separate.

---

## Domain 6 — Operator surfaces

### [HOWTO:Status]
- Injected facts:
  - [HOWTO:Status] For ecosystem overview use gzmo status or chat /status|/ecosystem first — never invent service tables from memory or inactive LEDs.
  - [HOWTO:Overnight] To answer what happened overnight: journalctl -u gzmo-daemon on CT101 plus gzmo health — not workstation gzmo-serve LEDs.
  - [HOWTO:EngineHealth] Judge Prime with curl http://192.168.31.184:8000/v1/models from CT101; treat llama-prime.service inactive as unit state only.
  - [HOWTO:SeedCoreInsight] Prefill living memory into /opt/gzmo/data/vault.db via seed-core-stack.py then gzmo memory embed and honeypot Qdrant sync — never seed data-next/ as production.
  - [HOWTO:SyncQdrant] After seeding or ingest, sync honeypot to Qdrant immediately if same-night recall must see new facts.
  - [HOWTO:VerifyProduction] After reboot/infra change run scripts/verify-production.sh when available; it checks plumbing, not eval quality.

### [CONCEPT:OpsConsole]
- Injected facts:
  - [CONCEPT:OpsConsole] gzmo --repl is the sovereign ops console (rail/transcript/Lorenz/instruments); gzmo/gzmo chat remains the stdio daily driver.
  - [CONCEPT:OpsConsole] gzmo observatory and gzmo metabolism are separate triage boards — do not stuff every LED into the chat cockpit.
  - [CONCEPT:OpsConsole] Release-binary alias trap: a shell gzmo may point at temp-bench/target/release/gzmo — rebuild that artifact or TUI/code changes are invisible.

### [CONCEPT:HotMemory]
- Injected facts:
  - [CONCEPT:HotMemory] Stdio chat (and TUI after parity) uses AgentSession scratch for per-turn hot memory; memory_search results land in scratch for the turn only.
  - [CONCEPT:HotMemory] Subagents use GatewayRouter for delegated TaskKind::Chat work; primary turns may still use the active TurboQuantGateway profile directly.

---

## Domain 7 — CT101 subsystem lessons (keep the wisdom)

### [LESSON:HoneypotDiscipline]
- Injected facts:
  - [LESSON:HoneypotDiscipline] CT101 taught honeypot is intentionally narrower than vault — missing source_file drops otherwise good facts; backfill provenance rather than lowering the floor blindly.
  - [LESSON:HoneypotDiscipline] Manual FTS sync after honeypot writes matters; bulk promote paths that skip FTS make recall go dark.

### [LESSON:DiscoveryCaution]
- Injected facts:
  - [LESSON:DiscoveryCaution] CT101 discovery/auto-socratic cycles can run while publish is blocked by eval placeholders — never treat unpublished discovery as production knowledge.
  - [LESSON:DiscoveryCaution] Do not add DiscoveryEngine into living gzmo-scheduler/serve overnight until fixture beat-gates stay green.

### [LESSON:SynapseScale]
- Injected facts:
  - [LESSON:SynapseScale] Synapse is an append-only observability bus, never a state store; at CT101 scale it needed rotation discipline past hundreds of thousands of events.

### [LESSON:CloudVsLocal]
- Injected facts:
  - [LESSON:CloudVsLocal] Living CT101 runs cloud-first (OpenRouter) with Prime fallback on 192.168.31.184:8000; workstation local-first is lab/chat only.

### [LESSON:AssemblyGuard]
- Injected facts:
  - [LESSON:AssemblyGuard] Two-stack assembly guards exist so lab recipes cannot silently rewrite CT101 inline loops; living metabolism is typed Rust in gzmo-daemon, not lab recipe authority.

### [LESSON:BeatGates]
- Injected facts:
  - [LESSON:BeatGates] Beat-gates (config/ops/cognition/knowledge smokes) are the honesty layer before trusting a composed runtime — fixtures first, --live second.

---

## Domain 8 — Failure modes & anti-patterns

### [POLICY:CurationFirst]
- Injected facts:
  - [POLICY:CurationFirst] Never ingest raw migration piles or chat dumps; only condensed, operator-approved material becomes long-term memory.
  - [POLICY:NoAutoMigration] There is no automated re-migration of Takeout/inbox into honeypot; injection is a deliberate act.
  - [POLICY:NoBulkSwap] Never swap Prime weights or bulk-ingest a corpus without the relevant quality/eval gate.

### [POLICY:AntiEmptyNight]
- What: How empty metabolism nights happen and how to prevent false emptiness.
- How: Dream needs episodic content; Spark needs diverse honeypot anchors; agents must not read wrong paths.
- Use: Seed CORE_INSIGHT; keep chatting/logging into /opt/gzmo/data/memory; fix corrupt sessions.
- Why: 2026-07-17 dream skipped (no episodic for the day); sparks looped stale anchors while serve was healthy.
- Related: [HOWTO:SeedCoreInsight], [CONCEPT:SessionDistillPaths]
- Injected facts:
  - [POLICY:AntiEmptyNight] An empty Dream is usually missing consolidatable episodic text for that date under /opt/gzmo/data/memory/, not a dead daemon.
  - [POLICY:AntiEmptyNight] Prefill Structural honeypot facts (this document) so Spark/Dream have non-stale anchors when chat volume is low.
  - [POLICY:AntiEmptyNight] Corrupt session JSON (e.g. missing is_meta) is skipped by distill — repair or delete the file rather than blaming metabolism.

### [POLICY:StatusHonesty]
- Injected facts:
  - [POLICY:StatusHonesty] Color LEDs without expected-offline semantics lie: disabled scheduler and chaos.alive vs LLM ping must be labeled separately.
  - [POLICY:StatusHonesty] Execution over simulation — never fabricate PIDs, counts, or service states; tool output and status commands only.

---

## Domain 9 — Current living state (2026-07-17)

### [STATE:LivingRestore]
- Injected facts:
  - [STATE:LivingRestore] As of 2026-07-17 restore, production is CT101 (gzmo-daemon + /opt/gzmo vault ~60k facts); workstation gzmo-serve is stopped/disabled; data-next/ is lab scratch only.
  - [STATE:LivingCutover] The 2026-07-15/16 workstation cutover (data-next/, gzmo-serve overnight, CT101 frozen) is historical — superseded by LivingRestore.
  - [STATE:LivingCutover] On 2026-07-17 morning (pre-restore), gzmo-serve had run spark/dream/distill/promote/embed/wiki while gzmo-scheduler stayed inactive — then serve was disabled as part of CT101 restore.
  - [STATE:EnginesEnabled] Living CT101 runs cloud-first daemon cognition with Prime fallback; workstation serve engines apply only if lab serve is explicitly re-enabled.
  - [STATE:CoreInsightPurpose] docs/CORE_INSIGHT.md exists to prefill Structural honeypot self-knowledge so the living system does not wake into an empty or stale-looped recall field.

---

## Domain 10 — MCP & frontends

### [SYSTEM:MCP]
- Injected facts:
  - [SYSTEM:MCP] Two MCP surfaces: Neo4j memory (mcp__memory__*) and gzmo-memory (gzmo mcp-serve / gzmo memory mcp) for search/status/wiki tools.
  - [CONCEPT:Pi] Pi may be an operator frontend that must reach GZMO memory only through approved bridges/scripts — never by writing Redis/vault SQL directly.
  - [CONCEPT:Obolus] Obolus/task routing and metering lessons from CT101 inform GatewayRouter task kinds; living chat still centers the active engine profile for primary turns.

---

## Domain 11 — Operator doctrine (compressed)

### [POLICY:Doctrine]
- Injected facts:
  - [POLICY:Doctrine] Prefer real tool output and deterministic status commands over multi-hop reconnaissance that burns tokens.
  - [POLICY:Doctrine] Progressive disclosure: boards and observatory stay outside the chat cockpit until a deliberate drill-in exists.
  - [POLICY:Doctrine] Lorenz/instruments in --repl are physiology, not a mascot face — honest warming/stale states beat fake motion.
  - [POLICY:Doctrine] Lab recipes remain beat-gate fixtures; they are not the long-term production brain — CT101 gzmo-daemon is.

---

*End of CORE_INSIGHT. Extend only with verified, operator-approved cards in this format; re-seed with `scripts/seed-core-stack.py` into `/opt/gzmo/data/vault.db` on CT101.*
