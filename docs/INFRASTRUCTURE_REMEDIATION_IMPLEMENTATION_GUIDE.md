# GZMO Infrastructure Remediation — Implementation Guide

**Created:** 2026-06-14  
**Repo:** `/home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO`  
**Audience:** Human operator + Pi agent implementing the six workstreams identified in the infrastructure discovery audit.

This guide supersedes informal ranking notes from discovery sessions. It is **action-oriented**: each workstream has prerequisites, numbered steps, acceptance criteria, pitfalls, and rollback notes.

**Companion docs:**
- Topology: [`PORTS.md`](./PORTS.md), [`PI_OPERATOR_GUIDE.md`](./PI_OPERATOR_GUIDE.md)
- Pedagogy graphs: [`PEDAGOGY_PERSONALITY_ALIGNMENT_HANDOFF.md`](./PEDAGOGY_PERSONALITY_ALIGNMENT_HANDOFF.md)
- Honeypot pipeline: [`INFRASTRUCTURE_MAP.md`](./INFRASTRUCTURE_MAP.md) § honeypot qualification
- Skill chaos wiring: [`SKILL_GOLDEN_STANDARD.md`](./SKILL_GOLDEN_STANDARD.md) § Daemon IPC

---

## Table of contents

1. [Executive summary](#1-executive-summary)
2. [Dependency graph and rollout order](#2-dependency-graph-and-rollout-order)
3. [Phase 0 — Baseline capture](#3-phase-0--baseline-capture)
4. [Workstream A — Container network gap (Critical)](#4-workstream-a--container-network-gap-critical)
5. [Workstream B — Autopoietic feedback loop (High)](#5-workstream-b--autopoietic-feedback-loop-high)
6. [Workstream C — Low-tension dialogue corpus (High)](#6-workstream-c--low-tension-dialogue-corpus-high)
7. [Workstream D — Display visibility (Medium)](#7-workstream-d--display-visibility-medium)
8. [Workstream E — Prerequisite graphs (Medium)](#8-workstream-e--prerequisite-graphs-medium)
9. [Workstream F — Honeypot rejection observability (Lower)](#9-workstream-f--honeypot-rejection-observability-lower)
10. [Verification matrix](#10-verification-matrix)
11. [Suggested implementation calendar](#11-suggested-implementation-calendar)

---

## 1. Executive summary

| ID | Workstream | Root cause (verified) | Primary fix | Est. effort |
|----|------------|----------------------|-------------|-------------|
| **A** | Container network | Pi container has loopback only; host cron health is green | Route LAN or host-network; dual-perspective health | 2–4 h |
| **B** | Feedback loop | Inbox is transient (drained); Pi paths bypass IPC; empty repo `.mcp.json` | Route all skills via `gzmo chaos skill`; durable audit log; fix MCP | 4–8 h |
| **C** | Low-tension dialogue | Threshold 15%; τ stays 17–21%; edge-trigger only | Lower threshold + Idle-tick trigger + KG-aware openings | 4–6 h |
| **D** | Display visibility | Offset is correct byte cursor; filter only shows `chaos.dice_loop` | Expand event types + throttle + recent panel | 3–5 h |
| **E** | Prerequisite graphs | 4 YAML files, 26 nodes vs thousands of CONCEPTs | Neo4j export + readiness gate on dice cascade | 8–16 h |
| **F** | Honeypot rejection | ~49% vault honeypot → Qdrant; filter opaque | Rejection log + optional review queue | 4–6 h |

**Do first:** A → B (config slice) → C (config slice) → D → F → E.

Workstream E is valuable but not blocking daily Pi operation. Workstream A blocks embed/rerank/Redis recall from inside the container.

---

## 2. Dependency graph and rollout order

```text
Phase 0 (baseline)
    │
    ▼
A Container networking ─────────────────────────────┐
    │                                                │
    ├──► B Feedback (needs LAN for full skill test) │
    │         │                                      │
    │         └──► D Display (daemon events flow)   │
    │                                                │
    └──► C Low-tension (needs mentor socket + LAN)  │
                                                     │
F Honeypot logging (independent) ◄──────────────────┘
E Prerequisite graphs (needs Neo4j MCP + graph export)
```

**Hard dependencies:**
- B step "end-to-end skill feedback test" needs A (or host-network) for embed-dependent skills.
- C `discovery_cycle` spawns scripts in `gzmo_skills` that call `gzmo health` — misleading until A is fixed unless you add perspective labels first.
- E Neo4j export needs working `memory` MCP (fix B `.mcp.json` first).

**Soft dependencies:**
- D is independent but more useful once B ensures richer daemon/skill events exist.

---

## 3. Phase 0 — Baseline capture

Run once before any changes. Store output in `data/pi-mentor-discovery/logs/baseline-pre-remediation.txt`.

### 3.1 Host perspective

```bash
cd /home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO

./scripts/auto-health-check.sh | tee /tmp/gzmo-host-preflight.txt
gzmo health | tee /tmp/gzmo-host-health.txt

# Chaos + dialogue state
cat data/CHAOS_STATE.json | jq '{tick,tension,phase,thoughts_crystallized}'
wc -l data/pedagogy/low_tension_dialogue.jsonl
wc -l data/Synapse/events.jsonl
cat data/synapse-pi-display.state.json | jq .

# Honeypot counts (Python — sqlite3 CLI optional)
python3 - <<'PY'
import sqlite3, json, urllib.request
conn = sqlite3.connect("data/vault.db")
hp = conn.execute("SELECT COUNT(*) FROM honeypot").fetchone()[0]
print("vault_honeypot_rows", hp)
try:
    r = urllib.request.urlopen("http://192.168.31.202:6333/collections/honeypot", timeout=5)
    d = json.loads(r.read())
    print("qdrant_points", d["result"]["points_count"])
    print("sync_ratio", round(d["result"]["points_count"] / hp, 4))
except Exception as e:
    print("qdrant_error", e)
PY

# Feedback inbox (expect missing — transient queue)
ls -la data/chaos_feedback_inbox.jsonl 2>&1 || true
pgrep -af 'gzmo daemon' || true
```

### 3.2 Container perspective

From inside the Pi container (or Cursor sandbox that reproduces loopback-only):

```bash
ip addr
ip route
curl -sf --max-time 3 http://127.0.0.1:8000/v1/models | head -c 80 || echo FAIL:prime
curl -sf --max-time 3 http://192.168.31.110:8081/v1/models | head -c 80 || echo FAIL:embed
python3 -c "import socket; s=socket.create_connection(('192.168.31.202',6379),3); s.sendall(b'PING\r\n'); print(s.recv(16))" || echo FAIL:redis

# MCP memory (may work via host stdio)
# Run gzmo mcp-serve health probes if available in your container entrypoint
```

### 3.3 Acceptance for Phase 0

- [ ] Host preflight: PASS >= 8, FAIL = 0
- [ ] Container: document which of {embed, rerank, redis, qdrant} fail
- [ ] Baseline file committed or saved locally (not required in git)

---

## 4. Workstream A — Container network gap (Critical)

### Goal

Pi runtime environment can reach VM200 (`192.168.31.110`) and LXC101 (`192.168.31.202`) the same way the host can.

### Diagnosis recap

- Container: `127.0.0.1/8` only, empty routing table.
- Host: all probes green.
- Proxies on `1080`/`3128` exist but do **not** currently bridge to LAN targets.
- Neo4j often works via host-spawned MCP stdio, which hides the HTTP/TCP gap.

### Choose one strategy

| Strategy | Pros | Cons | When to pick |
|----------|------|------|--------------|
| **A1: Host network** | Trivial; matches workstation topology | Weakens container isolation | Dev workstation; fastest unblock |
| **A2: Macvlan/ipvlan** | Real LAN IP on container | Needs router/admin setup | Production-like isolation |
| **A3: Host port forwards** | Keeps bridge network | Manual map per port; drift risk | Cannot use host network |
| **A4: MCP-only retrieval** | No route changes | Embed/rerank/Redis still broken for `gzmo memory` | Partial mitigation only |

**Recommendation:** A1 for immediate ROI on a trusted dev machine. Document A3 as fallback.

---

### A1 — Host network mode (recommended)

#### Steps

1. **Identify Pi container launch config** (Docker, devcontainer, systemd, or Cursor sandbox metadata). Locate the `docker run`, `compose.yaml`, or devcontainer `runArgs`.

2. **Add host networking:**

   ```yaml
   # docker-compose example
   services:
     pi-agent:
       network_mode: host
       # Remove ports: mappings — they are ignored in host mode
   ```

   Or devcontainer:

   ```json
   "runArgs": ["--network=host"]
   ```

3. **Preserve environment:**

   ```bash
   export GZMO_ROOT=/home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO
   export NO_PROXY=localhost,127.0.0.1,192.168.31.0/24
   export no_proxy="$NO_PROXY"
   ```

4. **Restart container** and re-run Phase 0 container probes.

5. **Verify memory bridge from container:**

   ```bash
   cd "$GZMO_ROOT"
   ./scripts/pi-gzmo-memory.sh status
   # Expect: scratch=redis, not scratch=in-memory
   ```

#### Acceptance

- [ ] `curl http://192.168.31.110:8081/v1/models` succeeds from container
- [ ] Redis PING succeeds from container
- [ ] `./scripts/pi-gzmo-memory.sh status` shows `scratch=redis`
- [ ] `./scripts/auto-health-check.sh` passes inside container

#### Pitfalls

- `localhost:8000` in container now IS the host Prime — correct for GZMO.
- Host network breaks port-mapping assumptions in compose files.
- Cursor remote sandbox may not allow `--network=host`; use A3 or run Pi on host.

---

### A3 — Host port forwards (fallback)

If you must keep bridge networking, forward on the **host** (not inside container):

```bash
# On host — example forwards (adjust if ports conflict)
sudo socat TCP-LISTEN:18081,fork,reuseaddr TCP:192.168.31.110:8081 &
sudo socat TCP-LISTEN:16379,fork,reuseaddr TCP:192.168.31.202:6379 &
sudo socat TCP-LISTEN:16333,fork,reuseaddr TCP:192.168.31.202:6333 &
```

Point container `gzmo.toml` overrides or env at bridge gateway IP:

```toml
# Container-only overlay gzmo.container.toml
[embeddings]
url = "http://172.17.0.1:18081/v1"   # docker0 gateway — verify with ip route

[redis]
url = "redis://172.17.0.1:16379"
```

**Pitfall:** `172.17.0.1` varies by runtime; detect with `ip route | grep default`.

---

### A5 — Dual-perspective health (do regardless of A1/A3)

The misleading cron snapshot is a **reporting bug**, not a service bug.

#### Steps

1. **Add perspective enum** in `gzmo-core/src/health.rs`:

   ```rust
   pub enum HealthPerspective {
       Host,
       Container,
   }
   ```

2. **Detect container context** (heuristic):

   ```rust
   fn detect_perspective() -> HealthPerspective {
       if std::path::Path::new("/.dockerenv").exists() {
           HealthPerspective::Container
       } else {
           HealthPerspective::Host
       }
   }
   ```

3. **Extend `format_report`** to prefix each run:

   ```text
   GZMO health report [perspective=container hostname=...]
   ```

4. **Update `scripts/auto-health-check.sh`** to export and print:

   ```bash
   export GZMO_HEALTH_PERSPECTIVE="${GZMO_HEALTH_PERSPECTIVE:-host}"
   echo "Perspective: $GZMO_HEALTH_PERSPECTIVE ($(hostname))"
   ```

5. **Cron adjustment** — run both probes on host:

   ```cron
   # Host-native
   0 * * * * cd $GZMO_ROOT && ./scripts/auto-health-check.sh >> logs/health-host.log 2>&1
   # Container (if Pi runs in docker)
   5 * * * * docker exec pi-agent bash -lc 'cd $GZMO_ROOT && GZMO_HEALTH_PERSPECTIVE=container ./scripts/auto-health-check.sh' >> logs/health-container.log 2>&1
   ```

6. **Optional: Synapse event** — append `health_tick` with `data.perspective` from `run_startup_probes`.

#### Acceptance

- [ ] `gzmo health` output includes perspective label
- [ ] Container FAIL / host PASS side-by-side in logs without contradiction
- [ ] Mentor/discovery scripts cite perspective when interpreting green status

#### Files to touch

- `gzmo-core/src/health.rs` — `format_report`, optional `detect_perspective`
- `gzmo-cli/src/health_cmd.rs` — pass perspective into report
- `scripts/auto-health-check.sh` — env label
- `docs/PI_OPERATOR_GUIDE.md` — § health table update

---

## 5. Workstream B — Autopoietic feedback loop (High)

### Goal

Every skill execution that should modulate chaos state actually reaches the PulseLoop, and operators can audit what was emitted vs drained.

### Diagnosis recap

- **Infrastructure exists:** `forward_feedback`, `chaos_skill_cmd.rs`, `feedback_ipc.rs`, `chaos_bootstrap.rs` drain loop.
- **Empty inbox is normal:** `drain_inbox()` deletes the file after rename.
- **Real gaps:** Pi repo `.mcp.json` is 0 bytes; skills invoked outside `gzmo chaos skill` bypass inbox; no durable audit trail.

---

### B1 — Fix empty repo `.mcp.json` (30 min)

#### Steps

1. **Do not hand-edit secrets.** Run the installer:

   ```bash
   cd /home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO
   ./scripts/install-shared-mcp.sh
   ```

2. **Symlink or copy fragment into repo** (for Pi cwd = survey_GZMO):

   ```bash
   cp config/shared-mcp-memory.json .mcp.json
   # Or merge if .mcp.json should track project-local servers only
   python3 - <<'PY'
   import json, pathlib
   root = pathlib.Path(".")
   frag = json.loads((root / "config/shared-mcp-memory.json").read_text())
   target = root / ".mcp.json"
   cur = json.loads(target.read_text()) if target.stat().st_size else {"mcpServers": {}}
   cur.setdefault("mcpServers", {}).update(frag.get("mcpServers", {}))
   target.write_text(json.dumps(cur, indent=2) + "\n")
   PY
   ```

3. **Add CI guard** — fail if `.mcp.json` empty in repo:

   ```bash
   # scripts/verify-mcp-json.sh
   python3 -c "import json, pathlib; d=json.loads(pathlib.Path('.mcp.json').read_text()); assert d.get('mcpServers')"
   ```

4. **Restart Pi** after merge. Confirm log line gone:

   ```text
   Failed to load MCP config from .../.mcp.json: SyntaxError
   ```

#### Acceptance

- [ ] `.mcp.json` valid JSON, non-zero size
- [ ] Pi starts without MCP parse error
- [ ] `mcp__memory__read_graph` available in Pi session

---

### B2 — Durable chaos feedback audit log (2–3 h)

The inbox is a queue, not a log. Add append-only audit without changing drain semantics.

#### Steps

1. **Add constant** in `gzmo-chaos/src/feedback_ipc.rs`:

   ```rust
   const AUDIT_NAME: &str = "chaos_feedback_audit.jsonl";
   ```

2. **In `append_event`**, after successful inbox write:

   ```rust
   fn append_audit(data_dir: &Path, event: &ChaosEvent, source: &str) -> std::io::Result<()> {
       let audit = data_dir.join(AUDIT_NAME);
       let dto = ChaosEventDto::from(event);
       let line = serde_json::json!({
           "ts": chrono::Utc::now().to_rfc3339(),
           "source": source,
           "event": dto,
       });
       // append line to audit
   }
   ```

   Call from:
   - `append_event` with `source="inbox"`
   - `chaos_bootstrap.rs` drain loop with `source="drained"` per event
   - Direct `feedback_tx.send` in daemon with `source="daemon_internal"` (optional second hook)

3. **Add CLI inspect:**

   ```bash
   gzmo chaos feedback-audit --tail 20
   ```

   Implement in `gzmo-cli/src/chaos_feedback_cmd.rs` (new).

4. **Synapse optional mirror** — on drain, emit `chaos.feedback_drained` with event type summary (count by variant).

#### Acceptance

- [ ] `data/chaos_feedback_audit.jsonl` grows on `gzmo chaos skill dice`
- [ ] Inbox still absent after daemon tick (drained)
- [ ] Audit shows both `inbox` and `drained` entries with matching event types

---

### B3 — Route all Pi skill invocations through daemon IPC (2–4 h)

#### Steps

1. **Inventory bypass paths** — grep `gzmo_skills` and Pi extensions:

   ```bash
   rg -l 'skill_dispatch|/dice|/joke|chaos skill' ~/gzmo_skills ~/.pi/agent/extensions
   ```

2. **Canonical wrapper** — ensure every shell skill delegates to Rust:

   ```bash
   # skills/skill_*.sh pattern (already exists for card/pkm)
   exec gzmo chaos skill dice "$@"
   ```

3. **Update `dice_cascade` daemon path** in `gzmo-cli/src/daemon_cmd.rs` — verify headless dice uses `run_registry_skill` which calls `forward_feedback` (already wired for in-process tx; external Pi path uses inbox).

4. **Pi extension hook** — if any extension runs skills via bash, replace with:

   ```typescript
   await bash(`cd ${GZMO_ROOT} && gzmo chaos skill ${cmd} ${args}`);
   ```

5. **Document rule** in `docs/SKILL_GOLDEN_STANDARD.md`:

   > Pi MUST invoke generative skills via `gzmo chaos skill`, never raw `skill_*.sh`, when daemon is running.

#### Acceptance

- [ ] `gzmo chaos skill joke --json` → audit log entry within 1s
- [ ] After 1 daemon tick (~0.35s), `thoughts_incubating` or tension changes (may be subtle)
- [ ] No skill script in `gzmo_skills` calls LLM without chaos IPC

---

### B4 — xAI voice extension (optional, 15 min)

Low priority for chaos loop; removes startup noise.

1. Set `XAI_API_KEY` in `~/.pi/agent/settings.json` under `env`, **or**
2. Remove `"npm:pi-xai-voice"` from `settings.json` packages if unused.

---

### B5 — Mentor feedback verification

Mentor already calls `emit_mentor_chaos_feedback` in `gzmo-cli/src/mentor_ipc.rs`.

Verify:

```bash
rg 'MentorTeach' data/Synapse/events.jsonl | wc -l   # expect >> 0
gzmo chaos feedback-audit --tail 5 | rg MentorTeach  # after B2
```

---

## 6. Workstream C — Low-tension dialogue corpus (High)

### Goal

Socratic dialogues fire regularly when the chaos field is calm, produce varied KG-aware questions, and persist for recall.

### Diagnosis recap

- Config: `threshold = 15.0` in `gzmo.toml` `[pedagogy.low_tension_dialogue]`
- Trigger: **edge only** (`crossed_below_threshold` in `low_tension_dialogue.rs`)
- Current τ ≈ 18–21% → no crossings since tick 582
- `discovery_cycle = true` → spawns full `auto-socratic-discovery-cycle.sh` instead of bare `teach_autonomous`

---

### C1 — Config tuning (30 min)

#### Steps

1. **Edit `gzmo.toml`:**

   ```toml
   [pedagogy.low_tension_dialogue]
   enabled = true
   threshold = 18.0          # was 15.0 — fires in current 17–21% band on downward crossing
   cooldown_secs = 300
   discovery_cycle = true
   idle_ticks_threshold = 120   # NEW — see C2
   ```

2. **Add `idle_ticks_threshold: Option<u64>`** to `LowTensionDialogueConfig` in `gzmo-core/src/config.rs` with `#[serde(default)]`.

3. **Restart daemon** after config change.

#### Acceptance

- [ ] Watcher logs `Low-tension Socratic watcher online` with new threshold
- [ ] Within one cooldown window of τ dipping below 18%, spawn log appears in `gzmo_skills/data/pi-mentor-discovery/logs/auto-socratic-spawn.log`

---

### C2 — Secondary trigger: Idle ticks without dialogue (2–3 h)

Edge-trigger alone misses "already low" plateaus.

#### Steps

1. **In `run_low_tension_watcher`** (`gzmo-cli/src/low_tension_dialogue.rs`), track:

   ```rust
   let mut idle_ticks_while_calm = 0u64;
   ```

2. **Each 5s interval:**

   ```rust
   if snap.phase == Phase::Idle && snap.tension < cfg.threshold {
       idle_ticks_while_calm += 1;
   } else {
       idle_ticks_while_calm = 0;
   }

   let idle_fire = cfg.idle_ticks_threshold
       .map(|n| idle_ticks_while_calm >= n)
       .unwrap_or(false);

   if !crossed && !idle_fire { continue; }
   ```

3. **Default `idle_ticks_threshold`** = 120 ticks at 5s poll ≈ 10 min calm Idle (tune).

4. **Unit test** `idle_trigger_fires_after_n_calm_samples`.

#### Acceptance

- [ ] Dialogue fires when τ stays at 19% for `idle_ticks_threshold` without requiring downward cross
- [ ] `low_tension_dialogue.jsonl` gains new lines

---

### C3 — KG-aware opening generator (4–6 h)

Replace static template with context from Neo4j + learner profile.

#### Steps

1. **New module** `gzmo-core/src/pedagogy/low_tension_opening.rs`:

   ```rust
   pub async fn build_opening(
       snap: &ChaosSnapshot,
       learner: &LearnerProfile,
       graph: Option<&PrerequisiteGraph>,
       neo4j_summary: Option<&str>,
   ) -> String
   ```

2. **Query Neo4j** via existing MCP tool or `gzmo memory search` for:
   - 1–2 recent CONCEPTs with low `recall_count`
   - 1 unmastered prerequisite from graph (if E partial done)

3. **Prompt template:**

   ```text
   [AUTONOMOUS — low tension] τ={tension}%, tick {tick}, phase {phase}.
   Recent graph focus: {concept_hint}.
   Ask ONE Socratic question about {topic} — do not lecture.
   ```

4. **Wire in `low_tension_dialogue.rs`** before `spawn_discovery_cycle` / `teach_autonomous`.

5. **Append KG metadata** to JSONL log:

   ```json
   {"concept_hint": "...", "graph_node": "ownership", ...}
   ```

#### Acceptance

- [ ] Consecutive openings differ (hash distinct)
- [ ] Openings reference real CONCEPTs from graph search
- [ ] No solution leakage (existing EDF leakage checks still pass)

---

### C4 — Persist dialogues to Neo4j (optional, 4 h)

1. After successful `teach_autonomous`, emit MCP `create_entities` or dedicated `gzmo-cli` helper:

   - Entity type: `SOCRATIC_DIALOGUE`
   - Relations: `DIALOGUE_ABOUT` → CONCEPT, `DIALOGUE_WITH` → LEARNER

2. Add recall path in `gzmo memory search` for past openings (avoid repeating questions).

3. **Defer** if C1+C2 already deliver sufficient variety.

---

## 7. Workstream D — Display visibility (Medium)

### Goal

Students see a curated stream of daemon/skill/distill activity in Pi, not just 14 dice-loop events.

### Diagnosis recap

- `synapse-pi-display.state.json` offset ≈ file size in bytes — **correct**
- Filter in `~/.pi/agent/extensions/gzmo-daemon-display.ts` allows only `chaos.dice_loop` from `gzmo_daemon`

---

### D1 — Expand display event types (1–2 h)

#### Steps

1. **Edit `gzmo-daemon-display.ts`:**

   ```typescript
   const DISPLAY_EVENT_TYPES = new Set([
     "chaos.dice_loop",
     "chaos.feedback_drained",   // after B2
     "mentor_teach",
     "ingest_complete",
     "spark_complete",
     "TopicShiftDistill",
   ]);

   const DISPLAY_SOURCES = new Set(["gzmo_daemon", "gzmo_cli", "gzmo_mentor"]);
   ```

2. **Update `isDisplayEvent`:**

   ```typescript
   function isDisplayEvent(row: SynapseRow): boolean {
     return (
       DISPLAY_SOURCES.has(row.source ?? "") &&
       DISPLAY_EVENT_TYPES.has(row.event_type ?? "") &&
       Boolean(row.id)
     );
   }
   ```

3. **Add formatters** in `formatDisplayMessage` for each new type (short markdown).

4. **Settings toggle** in `~/.pi/agent/settings.json`:

   ```json
   "gzmoDaemonDisplay": {
     "enabled": true,
     "maxPerPoll": 2,
     "eventTypes": ["chaos.dice_loop", "mentor_teach"]
   }
   ```

#### Acceptance

- [ ] Pi session shows mentor_teach summaries without agent turn
- [ ] `displayedIds` grows faster than 14 lifetime total
- [ ] No flood: respect `maxPerPoll`

---

### D2 — Display throttle (1 h)

In `pollBus`, after parsing rows:

```typescript
let shown = 0;
for (const row of parseSynapseLines(chunk)) {
  if (!isDisplayEvent(row)) continue;
  if (shown >= (cfg.maxPerPoll ?? 2)) break;
  if (publishRow(row)) shown++;
}
```

---

### D3 — Recent discoveries panel (2 h)

Optional Pi custom message type `gzmo-recent-discoveries`:

1. **Daemon-side** — nightly job or distill completion writes `data/pi-recent-discoveries.json` (last 10 entities/relations).

2. **Extension** — on `session_start`, read file and `pi.sendMessage({ customType: "gzmo-recent-discoveries", ... })`.

3. **Source data** — `session_distill` promoted entities from `data/Synapse/events.jsonl` `ingest_complete` payloads.

---

## 8. Workstream E — Prerequisite graphs (Medium)

### Goal

Structured learning paths cover high-traffic CONCEPT clusters; dice cascade respects readiness.

### Diagnosis recap

- `data/pedagogy/graphs/*.yaml` — 4 files, 26 nodes
- `PrerequisiteGraph::load_dir` merges all YAML at daemon boot (`pedagogy_bridge.rs`)
- `unmastered_prerequisites()` exists but dice cascade does not call it

---

### E1 — Graph validation CLI (1 h)

```bash
gzmo pedagogy graph validate --dir data/pedagogy/graphs
```

Implement in `gzmo-cli/src/pedagogy_graph_cmd.rs`:

- Load dir, run `graph.validate()`, print node count / domains

**Acceptance:** exit 0 on current 4 graphs.

---

### E2 — Auto-generate graphs from Neo4j (6–10 h)

#### Steps

1. **New script** `scripts/export-prerequisite-graph.py`:

   - Query Neo4j for CONCEPT nodes with `PREREQUISITE_OF` or `RELATED_TO` edges
   - Rank clusters by degree centrality
   - Emit `data/pedagogy/graphs/generated-<domain>.yaml`

2. **Cypher starter:**

   ```cypher
   MATCH (c:CONCEPT)-[r:PREREQUISITE_OF]->(p:CONCEPT)
   RETURN c.name, p.name, count(*) AS weight
   ORDER BY weight DESC LIMIT 200
   ```

3. **Human review gate** — generated files land in `data/pedagogy/graphs/pending/`; operator moves to `graphs/` after edit.

4. **Start with top 3 clusters** (largest connected components) — do not attempt 4,691 nodes at once.

#### Acceptance

- [ ] At least 2 new validated YAML graphs with >= 10 nodes each
- [ ] `gzmo pedagogy graph validate` passes
- [ ] Orchestrator planner_context includes new nodes (grep daemon log at boot)

---

### E3 — Readiness gate on dice cascade (3–4 h)

#### Steps

1. **Extend `LearnerProfile`** (`gzmo-core/src/pedagogy/learner.rs`) with:

   ```rust
   pub mastered_concepts: Vec<String>,  // persisted in data/learner/<id>/profile.json
   ```

2. **In `dice_cascade.rs`**, before `dispatch_skill`:

   ```rust
   fn filter_skills_by_readiness(
       skills: &[String],
       graph: Option<&PrerequisiteGraph>,
       mastered: &[String],
   ) -> Vec<String>
   ```

   Drop skills tagged with domain nodes where `unmastered_prerequisites` non-empty.

3. **Skill → graph node map** in `data/dice_cascade.toml`:

   ```toml
   [skill_prereqs]
   card = "game_design"
   pkm = "game_design"
   ```

4. **Fallback:** if filtered pool empty, use original pool (do not block dice).

#### Acceptance

- [ ] Unit test: unmastered `borrowing` blocks `async_rust` skill when mapped
- [ ] Dice still fires when graph None (backward compatible)

---

## 9. Workstream F — Honeypot rejection observability (Lower)

### Goal

Operators see why vault facts do not reach Qdrant; can tune filters and optionally promote rejects.

### Diagnosis recap

- `qualifies_for_honeypot` in `gzmo-core/src/memory/honeypot.rs`
- Vault honeypot ~44k rows; Qdrant ~22k points (~50% sync ratio)
- Gap causes: confidence, source path exclusions, relation rows, boilerplate, sync lag, `is_latest` flags

---

### F1 — Rejection reason enum + log (3 h)

#### Steps

1. **Add enum:**

   ```rust
   pub enum HoneypotRejectReason {
       LowConfidence { got: f32, min: f32 },
       MissingSourceFile,
       ExcludedSourcePattern { pattern: String },
       RelationRow,
       Boilerplate,
       UnverifiedDerived,
       SyncPending,
   }
   ```

2. **Replace bool return** with:

   ```rust
   pub fn honeypot_eligibility(truth: &ExtractedTruth) -> Result<(), HoneypotRejectReason>
   ```

   Keep `qualifies_for_honeypot` as `eligibility(...).is_ok()` for compat.

3. **In `vault.rs` promote paths**, on reject:

   ```rust
   append_honeypot_reject_log(&reject_reason, truth, vault_id)?;
   ```

4. **Log file:** `data/honeypot_reject.jsonl` (append-only).

5. **CLI:**

   ```bash
   gzmo honeypot rejects --tail 50 --reason LowConfidence
   ```

#### Acceptance

- [ ] Ingesting a low-confidence test fact creates reject log line
- [ ] Summary command prints counts by reason for last 24h

---

### F2 — Tune confidence threshold (30 min, data-driven)

1. Run histogram:

   ```bash
   python3 scripts/ingest-quality/honeypot-confidence-histogram.py  # create if missing
   ```

2. If >30% rejects are `0.80–0.85` with high verify pass rate, lower `HONEYPOT_MIN_CONFIDENCE` to `0.82` in `honeypot.rs` **and** align `[ingest] min_confidence` in `gzmo.toml`.

3. Re-run `scripts/sync-vault-to-qdrant.py` and recall eval.

**Do not tune blind** — use F1 logs first.

---

### F3 — Review queue (optional, 4 h)

1. **SQLite table** `honeypot_review_queue` (vault_id, reason, content_preview, ts)

2. **CLI:**

   ```bash
   gzmo honeypot review list
   gzmo honeypot review promote <vault_id>
   ```

3. **Pi panel** — defer until F1 proves volume warrants UI.

---

## 10. Verification matrix

Run after all selected workstreams.

| Check | Command | Pass |
|-------|---------|------|
| Host health | `./scripts/auto-health-check.sh` | FAIL=0 |
| Container health | Same inside container | FAIL=0 (after A) |
| Perspective labels | `gzmo health` | Shows host/container |
| MCP parse | Pi startup log | No JSON error |
| Skill feedback | `gzmo chaos skill dice --json` + audit tail | Event logged + drained |
| Low tension | Wait for τ<18 or idle trigger | New JSONL line |
| Display | Pi session | >=1 non-dice event shown |
| Graph validate | `gzmo pedagogy graph validate` | exit 0 |
| Honeypot log | `wc -l data/honeypot_reject.jsonl` | >0 after ingest test |
| Qdrant drift | compare honeypot count vs Qdrant points | ratio documented |

---

## 11. Suggested implementation calendar

| Day | Focus | Workstreams |
|-----|-------|-------------|
| 1 AM | Host-network or port forwards + dual health | A |
| 1 PM | `.mcp.json` + feedback audit log | B1, B2 |
| 2 AM | Skill routing audit + Pi extension updates | B3 |
| 2 PM | Low-tension config + idle trigger | C1, C2 |
| 3 | Display expansion + throttle | D1, D2 |
| 4 | Honeypot reject log + histogram | F1, F2 |
| 5+ | KG openings + graph export + dice readiness | C3, E1–E3 |

---

## Appendix A — File checklist

| Workstream | Files |
|------------|-------|
| A | `scripts/auto-health-check.sh`, `gzmo-core/src/health.rs`, `gzmo-cli/src/health_cmd.rs`, container compose/devcontainer |
| B | `.mcp.json`, `config/shared-mcp-memory.json`, `gzmo-chaos/src/feedback_ipc.rs`, `gzmo-cli/src/chaos_bootstrap.rs`, `gzmo-core/src/skills/dispatch.rs`, `docs/SKILL_GOLDEN_STANDARD.md` |
| C | `gzmo.toml`, `gzmo-core/src/config.rs`, `gzmo-cli/src/low_tension_dialogue.rs`, new `low_tension_opening.rs` |
| D | `~/.pi/agent/extensions/gzmo-daemon-display.ts`, `~/.pi/agent/settings.json` |
| E | `data/pedagogy/graphs/*.yaml`, `gzmo-core/src/pedagogy/graph.rs`, `gzmo-core/src/skills/dice_cascade.rs`, `data/dice_cascade.toml` |
| F | `gzmo-core/src/memory/honeypot.rs`, `gzmo-core/src/memory/vault.rs`, new CLI `honeypot_cmd.rs` |

---

## Appendix B — What not to do

- **Do not** treat missing `chaos_feedback_inbox.jsonl` as failure — it is drained by design.
- **Do not** "fix" `synapse-pi-display` offset — it tracks byte position, not event IDs.
- **Do not** lower honeypot confidence without F1 histogram evidence.
- **Do not** assume SOCKS `1080` bridges LAN without configuring `ALL_PROXY` + valid upstream.
- **Do not** block dice cascade entirely on readiness — always provide fallback pool.

---

*End of guide. Update this doc when acceptance criteria pass or topology changes.*
