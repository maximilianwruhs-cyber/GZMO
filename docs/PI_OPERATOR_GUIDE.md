# Pi Agent — Operator & Infrastructure Guide

**Status:** 2026-06-04 (canonical for pi-rust)  
**Audience:** Pi agent when CWD is `survey_GZMO`  
**Repo:** `/home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO`

This is the **single entry document**: where you are, what the stack is, what you may do, and what is forbidden. Deeper detail lives in linked docs — read them in §7 order when unsure.

**Shipped baseline:** [`PI_GZMO_PLATFORM_IMPLEMENTATION_HANDOFF.md`](./PI_GZMO_PLATFORM_IMPLEMENTATION_HANDOFF.md)  
**Remaining tasks (step-by-step):** [`PI_GZMO_REMAINING_TASKS_IMPLEMENTATION_GUIDE.md`](./PI_GZMO_REMAINING_TASKS_IMPLEMENTATION_GUIDE.md)

---

## 1. Who you are in this stack

| Role | You (pi-rust) | GZMO Platform |
|------|----------------|---------------|
| **Job** | Daily operator / coding agent for Max | Memory + ingest + daemon pipeline |
| **Cognition** | Completions via **Prime** `http://127.0.0.1:8000/v1` | Same Prime for extract, verify, dream, spark |
| **Hot memory** | `./scripts/pi-gzmo-memory.sh` → `gzmo memory *` | Redis scratch, archive @ 90%, honeypot recall |
| **Graph** | Shared **Neo4j MCP** (optional) | Daemon + tools via MCP stdio |
| **Not your product UI** | — | `gzmo chat`, `gzmo tui` (legacy debug only) |

**One sentence:** You think on **Prime**; you remember through the **platform memory bridge**, not by inventing Redis/vault clients.

---

## 2. Where you are (topology)

```text
[You: pi-rust @ workstation]
    │
    ├─ HTTP :8000  → Prime (Gemma 4 26B-A4B, 256K ctx) — chat / code / reasoning
    ├─ scripts/pi-gzmo-memory.sh → gzmo CLI → vault + honeypot + Redis scratch
    └─ MCP (optional) → Neo4j @ 192.168.31.202:7687

[Workstation]
    ├─ gzmo daemon — ingest, dream, spark, janitor (do not replace with ad-hoc scripts)
    ├─ data/vault.db, honeypot SQLite, data/Synapse/events.jsonl
    └─ gzmo.toml — single config spine (read-only unless Max asks)

[VM200 — 192.168.31.110]
    └─ :8081 retrieval router — gzmo-embed + gzmo-rerank (production GZMO search)
       (legacy :8082 rerank and :8083 librarian retired; distill on Prime :8000)

[LXC101 — 192.168.31.202]
    ├─ :6333 Qdrant — collection honeypot (active), knowledge (legacy, read-only)
    ├─ :7687 Neo4j
    └─ :6379 Redis — scratch + distill queue (via gzmo, not direct pi access)
```

**Parked / ignore:** Sovereign `:8010`, VM200 brain `:8080`, FrankenMoE, vLLM lore in `~/Projects/swap/docs/`.

---

## 3. Live counts (baseline MEGA V, 2026-06-04)

Run anytime:

```bash
./scripts/memory-status.sh
./scripts/verify-baseline-green.sh   # exit 0 = full green
```

| Store | ~Count | Notes |
|-------|--------|-------|
| Vault | 3653 | Verified ingest history |
| Honeypot (+ FTS) | 1424 | Default recall / Qdrant `honeypot` |
| Qdrant `honeypot` | 1424 | Must match SQLite (0% drift) |
| Qdrant `knowledge` | 3245 | **Legacy — do not query as primary; delete only per runbook + Max keyword** |

**Baseline labels:** `baseline-m4-post-sprint` (ingest), `baseline-m4-platform-20260604` (platform + ops).

---

## 4. What you CAN do

### 4.1 Every user turn (memory)

```bash
./scripts/pi-gzmo-memory.sh turn-start
./scripts/pi-gzmo-memory.sh search "<topic>" --limit 5
# … work with Prime …
./scripts/pi-gzmo-memory.sh recall    # paste [RECALL] into context if needed
```

Shortcut: `./scripts/pi-gzmo-memory.sh prep "<query>"`  
New conversation: `./scripts/pi-gzmo-memory.sh session-new`

**Do not** call `./target/release/gzmo memory …` directly — use this script so `GZMO_SESSION_ID` stays stable and Redis scratch survives across invocations.

### 4.1a Hot-Context Compression & Cache-Compress-Retrieve (CCR)
When hot-context compression is active (enabled via `[context_compress]` in `gzmo.toml`), GZMO automatically compresses massive outputs in its memory layer (such as large files, tool outputs, and MCP responses) to fit inside target token budgets.
- **Cache-Compress-Retrieve (CCR) flow:** Large content blocks are cached in Redis under keys matching `gzmo:ccr:{session_id}:{hash}` and replaced with a placeholder: `[ccr:<hash> — gzmo_retrieve_context to expand]`.
- **Retrieval:** If you (or the client) need the full uncompressed text corresponding to a hash, invoke the MCP tool `gzmo_retrieve_context` with the specific `<hash>`. The tool queries the Redis CCR store and returns the full fidelity text.

Details: [PI_GZMO_MEMORY_INTEGRATION.md](./PI_GZMO_MEMORY_INTEGRATION.md), `.pi/GZMO_MEMORY.md`

### 4.2 Health & regression (two tiers)

| Tier | When | Command | ~Time |
|------|------|---------|-------|
| **Quick preflight** | Every operator session start | `./scripts/auto-health-check.sh` | 5–15 s |
| **Deep baseline** | After infra/ingest change, sign-off | `./scripts/auto-health-check.sh --deep` | 20–40 s |

**Quick** checks: Prime, embed (1024-dim POST), rerank (POST), Redis PING, Qdrant, platform memory (`scratch=redis`), git dirty (WARN), subagents (WARN if missing).

**Always use `./scripts/pi-gzmo-memory.sh`** — never call `gzmo memory` directly (session id won't persist). Memory commands need LAN access to `192.168.31.0/24` (VM200 + LXC101); sandboxed shells without that route degrade to in-memory scratch.

**Deep** runs `verify-baseline-green.sh` (build, tests, eval-quick STRICT, production E2E, pi bridge recall).

```bash
./scripts/auto-health-check.sh              # session start
./scripts/auto-health-check.sh --deep       # same as verify-baseline-green.sh
./scripts/verify-production.sh              # ops-only subset (no M4 eval)
STRICT=1 scripts/ingest-quality/eval-quick.sh   # ingest gates only (~30s)
```

On a single network blip: retry quick preflight after 30–60 s; do not treat one FAIL as “baseline dead” without retry.

### 4.3 Cognition

- **Prime** `:8000` — your main LLM; ctx **262144** in platform config (Gemma 4 26B-A4B).
- **Do not** start alternate brains on random ports without Max.

### 4.3a Mentor dialog (GZMO Socratic brain)

Teaching questions use **GZMO pedagogy** over the daemon Unix socket — not Prime.

| Task | Use |
|------|-----|
| Code, CI, shell, repo grep | **Prime** |
| "Teach me …", how/why/what-is, learn mode | **`gzmo_mentor_teach`** (tools in `gzmo-integration` skill) |

```text
Pi → gzmo_mentor_* → scripts/pi/mentor.sh → data/gzmo_mentor.sock → gzmo daemon → PedagogyOrchestrator
```

- Learner profile: `GZMO_LEARNER_ID=operator` (shared with `gzmo chat` / TUI).
- **Learn mode:** `gzmo_mentor_learn_start` → repeated `gzmo_mentor_teach` → `gzmo_mentor_learn_end`.
- Optional prep: `gzmo_chaos({ command: "learn", args: "<topic>" })` or `run_learn_prep: true` on learn start.
- Check daemon: `gzmo_mentor_ping` / `gzmo mentor ping` (expect `pong`, not `pong (local)`).
- Synapse: mentor tool calls emit `mentor_teach` events; daemon `[synapse_pull]` polls every 60s into episodic.
- **Session end → distill:** on `session_shutdown`, Pi emits `session_end` and spawns `gzmo distill pi <jsonl>` (daemon also tails the bus). Facts land in vault + episodic for Dream.

Present GZMO mentor replies faithfully in learn mode; paraphrase only when needed for clarity.

### 4.4 Code & docs in repo

- Edit application/docs **outside** `gzmo-core/` / `gzmo-cli/` when task is product/docs/ops.
- Read `docs/`, `scripts/`, `memory/`, `.pi/` freely.
- Append session notes to `memory/YYYY-MM-DD.md` or `.pi/WORKING_MEMORY.md` when useful.

### 4.5 Synapse / project features

- Synapse bus: `data/Synapse/events.jsonl` (see `.pi/WORKING_MEMORY.md`).
- Daemon start (if Max asks): `scripts/start-production.sh --daemon` from repo root.

---

## 5. What you must NOT do

| Forbidden | Why |
|-----------|-----|
| `gzmo chat` / `gzmo tui` as daily UI | Legacy harness; pi uses memory bridge |
| Direct Redis / raw vault SQL / Qdrant deletes | Platform owns hot/cold paths |
| `purge --confirm`, mass `ingest-dir` on wave_02 (335 files) | Destructive / ungated |
| Edit `gzmo.toml`, `gzmo-core/**`, `gzmo-cli/**` | Cursor/Rust owner unless Max explicitly assigns you |
| Delete Qdrant `knowledge` | Only `RUN KNOWLEDGE DELETE` + date ≥ 2026-06-11 + runbook |
| Trust `~/Projects/swap/docs/` ports | Stale (8001, vLLM on 8002, etc.) |
| Declare baseline FAIL on one network blip | Retry `auto-health-check.sh` or `--deep` after 30–60s |

**Rust / retrieval tuning:** say *"needs RUST RECALL FOLLOWUP → Cursor"* — see [M4_RECALL_GAP_TRIAGE_MEGA5.md](./M4_RECALL_GAP_TRIAGE_MEGA5.md).

---

## 6. GZMO pipeline (context only)

You do **not** run the full pipeline each turn; the **daemon** does ingest/dream/spark.

```text
input → extract (Prime) → verify → vault → qualify → honeypot → Qdrant honeypot
```

- **Vault** — long-term verified store.  
- **Honeypot** — curated recall field (your `memory search` hits here).  
- **Scratch** — per-turn Redis pad (`[RECALL]`), cleared on `turn-start`.

Identity: [MACHINE.md](../MACHINE.md). Platform API: [ARCHITECTURE_GZMO_PLATFORM.md](./ARCHITECTURE_GZMO_PLATFORM.md).

---

## 7. Reading order (when lost)

| # | Document | Purpose |
|---|----------|---------|
| 1 | **This file** | Pi role, can/can't, commands |
| 2 | [ARCHITECTURE_GZMO_PLATFORM.md](./ARCHITECTURE_GZMO_PLATFORM.md) | Platform vs frontend, P0–P3 |
| 3 | [PI_GZMO_MEMORY_INTEGRATION.md](./PI_GZMO_MEMORY_INTEGRATION.md) | Memory bridge workflow |
| 4 | [INFRASTRUCTURE_OVERVIEW.md](./INFRASTRUCTURE_OVERVIEW.md) | Full infra canonical (ports, services) |
| 5 | [PLATFORM_BASELINE_STATUS.md](./PLATFORM_BASELINE_STATUS.md) | What “green” means today |

Optional depth: [INFRASTRUCTURE_REVIEW.md](./INFRASTRUCTURE_REVIEW.md) (2026-06-01 detail), [MEMORY_ARCHITECTURE_SPEC.md](./MEMORY_ARCHITECTURE_SPEC.md) (cold layers), [MACHINE.md](../MACHINE.md) (identity).

**Pointer map:** [ARTIFACTS.md](./ARTIFACTS.md)

---

## 8. Antigravity / eval (awareness)

- Long eval replays (`replay-wave.sh` ~18–45 min) are for **baseline promotion**, not daily pi work.
- Antigravity handovers: [ANTIGRAVITY_DELEGATION.md](./ANTIGRAVITY_DELEGATION.md) — you do not execute MEGA phases unless Max pastes keywords into *your* session.
- Recall@5 ~25% is **informational** — does not block `verify-baseline-green.sh`.

---

## 9. Quick troubleshooting

| Symptom | Check |
|---------|--------|
| Session start | `./scripts/auto-health-check.sh` |
| Empty `[RECALL]` | Run `search` before `recall`; confirm `turn-start`; `memory status` → `scratch=redis` (not `in-memory`) |
| `scratch=in-memory` | Redis/LAN down — fix network, retry; do not patch Rust |
| Embed warnings | VM110 `:8081` — keyword search still works; vectors may be degraded |
| Prime down | `curl -sf http://127.0.0.1:8000/v1/models` — start Prime / ask Max |
| Probes fail | Qdrant `192.168.31.202:6333` — wait and retry; then `--deep` |

---

## 10. Session checklist (copy)

```text
[ ] CWD = survey_GZMO
[ ] ./scripts/auto-health-check.sh → no FAIL
[ ] turn-start at new user task
[ ] search before claiming "no context"
[ ] recall → [RECALL] if honeypot facts needed
[ ] Prime :8000 for reasoning
[ ] No gzmo chat / no bulk ingest / no purge
[ ] If infra change: verify-baseline-green.sh
```

---

*Canonical pi onboarding — supersedes scattered infra entry points for operator behavior. Infra counts: refresh via `memory-status.sh`.*
