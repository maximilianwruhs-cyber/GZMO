# OpenClaw ↔ GZMO memory — implementation guide

**Status:** Implementation handbook for doctrine-preserving attach  
**USP:** nutrient · Brain Feed · airgap living — OpenClaw is the **operator surface**, not a second overnight brain  
**Wayfinder map:** [`.scratch/openclaw-gzmo-memory/map.md`](../.scratch/openclaw-gzmo-memory/map.md)  
**Contract:** [OPENCLAW_WORKSPACE_CONTRACT.md](./OPENCLAW_WORKSPACE_CONTRACT.md) · [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md) · [ADR-0003](./ADR-0003-one-instance-metabolism.md) · [ADR-0005](./ADR-0005-flywheel-over-frozen-topology.md)  
**Research:** [01 living-attach surface](../.scratch/openclaw-gzmo-memory/research/01-living-attach-surface.md) · [02 memorySearch dual-writer risk](../.scratch/openclaw-gzmo-memory/research/02-memorysearch-dual-writer-risk.md)  
**Sibling (Gateway only):** `~/.openclaw/workspace/.scratch/openclaw-sota/` — do not merge efforts  

**Audience:** operator or agent session applying attach end-to-end after HITL tickets lock (or using the **proposed defaults** in §3 when proceeding under explicit ack).

---

## 0. One-sentence goal

Wire Telegram/OpenClaw so it **searches living memory only via `gzmo-living`**, **writes durable nutrient only via takeaway enqueue**, and keeps OpenClaw builtin memory as **local scratch** (FTS/`MEMORY.md`) with **dreaming off** — never a parallel honeypot/Qdrant/Neo4j brain.

```text
Telegram / OpenClaw Gateway
    │ search                    │ nutrient write
    ▼                           ▼
gzmo-living MCP ──────► CT101 honeypot / wiki (read)
openclaw-takeaway.sh ─► session close --takeaway (no --now)
    │
    └── local only: MEMORY.md / memory/*.md + optional FTS (provider: none)
                    dreaming OFF · no extraPaths → /opt/gzmo | wiki | honeypot
```

---

## 1. Non-goals (never build these)

| Do not | Why |
| ------ | --- |
| Dual overnight writers / `gzmo-serve` while CT101 lives | ADR-0003 |
| Chat → Qdrant `honeypot` upsert / Neo4j auto-graph | Contract never-list |
| `session close --now` from OpenClaw takeaway path | Forces metabolism; dual-writer class |
| `memorySearch.extraPaths` / QMD / memory-wiki aimed at living trees | Shadow living index / wiki dual-write ([research 02](../.scratch/openclaw-gzmo-memory/research/02-memorysearch-dual-writer-risk.md)) |
| OpenClaw dreaming as overnight consolidation while CT101 owns living | Second overnight brain on operator surface |
| Treat OpenClaw / Telegram as the GZMO product brain | Machine / USP doctrine |
| Merge this guide’s work into OpenClaw SOTA Gateway hardening map | Wrong ownership |

---

## 2. Current baseline (2026-08-10 snapshot)

Use as a preflight checklist; re-run probes before apply.

| Probe | Expected healthy | Last known |
| ----- | ---------------- | ---------- |
| `openclaw mcp show gzmo-living` | Server present | **Empty** `mcp.servers` |
| `bash scripts/living-attach-check.sh` | All OK | **FAIL** — vault ~508 facts (&lt; 10k floor at the time; floor now 500 post 2026-07-24 data migration); daemon may still be active |
| `systemctl --user is-active gzmo-serve` (workstation) | `inactive` | OK |
| `ssh ct101` + living `gzmo-daemon` | Reachable / active | Reachable / active |
| Workspace `bin/openclaw-takeaway.sh` | Present | Present |
| Synced `LIVING_ATTACH.md` / ecosystem markers | Present | Present |
| `MEMORY.md` | Optional local | Often absent (OK) |
| `agents.defaults.memorySearch` | Explicit safe posture | Unset → defaults toward openai (tighten on apply) |
| Dreaming | Off | Off |

**Hard gate before claiming “living attach works”:** vault fact floor ≥ 500 (curated-vault floor since the 2026-07-24 data migration; adjust only via explicit living-ops decision — out of scope for OpenClaw wiring alone). Do not fake attach with lab/`PRODUCT` flags.

---

## 3. Proposed HITL defaults (until grilling tickets close)

Wayfinder tickets **03–07** are still open. Proceeding to implement without closing them requires operator ack of these **proposed** answers (aligned with research):

| Ticket | Proposed lock |
| ------ | ------------- |
| **Semantic recall path of record** | `gzmo-living` MCP is sole **living** semantic recall. OpenClaw `memorySearch` may be FTS-only over local `MEMORY.md` / `memory/*` — never living truth. |
| **OpenClaw dreaming vs CT101** | Dreaming **off** (`memory-core.config.dreaming.enabled: false`). |
| **Nutrient vs local MEMORY.md** | Durable facts → `bin/openclaw-takeaway.sh`. Local `MEMORY.md` / daily `memory/*.md` = operator scratch / session notes only — never equated to honeypot. |
| **SOTA Memory posture handoff** | SOTA ticket adopts: `memorySearch.provider: "none"`, `sources: ["memory"]`, no `extraPaths`, dreaming off, citations `auto` or off. |
| **Final checklist** | This document §§5–8. |

Fog left for later (do not block Phase A–C):

- Read-only **mirrors** of wiki exports under `extraPaths` (default: **no**)
- Dual-citation UX when MCP + local FTS both fire
- Heartbeat copy about memory planes

---

## 4. Development phases

### Phase A — Living host readiness (CT101 / mutex holder)

**Owner:** living ops  
**Exit:** `living-attach-check.sh` green

1. Confirm living-host mutex claim (default CT101).
2. Confirm `gzmo-daemon` active; workstation `gzmo-serve` inactive unless mutex moved.
3. Restore/grow vault to satisfy fact floor (or document approved floor change).
4. Fix BatchMode SSH `ct101` if needed (`CT101_SSH_HOST`, keys).
5. Prove:

```bash
cd ~/github-clone/GZMO
bash scripts/living-attach-check.sh
# Prefer also: SSH memory status --json shows living vault_path + healthy fact count
```

### Phase B — OpenClaw Gateway MCP attach

**Owner:** workstation OpenClaw  
**Exit:** `openclaw mcp show gzmo-living` + successful probe/tools list

```bash
cd ~/github-clone/GZMO
# Refuses if GZMO_PRODUCT=1 or GZMO_ALLOW_LAB_VAULT=1
bash scripts/install-openclaw-living-attach.sh
# If check fails but you must stage config only (not preferred):
# bash scripts/install-openclaw-living-attach.sh --no-probe
```

Verify:

```bash
openclaw mcp show gzmo-living
openclaw mcp probe gzmo-living --json
openclaw mcp list
```

Expected tools (ops excluded at install): `gzmo_memory_search`, `gzmo_memory_status`, `gzmo_memory_profile`, `gzmo_wiki_search`, plus other product memory tools per `gzmo-core` serve surface — **not** `gzmo_ops_health` / `gzmo_discovery_status` unless explicitly allowed.

Restart / reload if Gateway already running:

```bash
openclaw mcp reload
# or: systemctl --user restart openclaw-gateway
```

### Phase C — Nutrient write path

**Owner:** workstation  
**Exit:** takeaway artifact `ok: true` (or documented living refuse with actionable cause)

```bash
bash ~/.openclaw/workspace/bin/openclaw-takeaway.sh 'OpenClawAttachSmoke: durable nutrient probe'
# Inspect:
#   ~/github-clone/GZMO/data-next/openclaw-attach/takeaway-latest.json
```

Refuse conditions to fix (not bypass):

- Workstation `gzmo-serve` active while CT101 living
- Read-only `/opt/gzmo/data/sessions` on CT101
- SSH/session seed failures

Never add `--now`, Qdrant curl, or Neo4j writes “to make it work.”

### Phase D — OpenClaw local memory posture (CLI 2026.7.1-2 keys)

**Owner:** OpenClaw config  
**Exit:** `openclaw config validate` + `openclaw memory status` shows intentional provider; dreaming off

Dry-run first:

```bash
openclaw config patch --dry-run --stdin <<'EOF'
{
  agents: {
    defaults: {
      memorySearch: {
        enabled: true,
        provider: "none",
        sources: ["memory"],
      },
    },
  },
  plugins: {
    entries: {
      "memory-core": {
        config: {
          dreaming: { enabled: false },
        },
      },
    },
  },
  memory: {
    citations: "auto",
  },
}
EOF
```

Apply only after dry-run OK (and HITL ack of §3):

```bash
openclaw config patch --stdin <<'EOF'
{ /* same payload */ }
EOF
openclaw config validate
openclaw memory status --agent main
```

**Forbidden in this phase:**

```json5
// DO NOT
{
  agents: {
    defaults: {
      memorySearch: {
        extraPaths: ["/opt/gzmo/data", "~/github-clone/GZMO/wiki", "…honeypot…"],
      },
    },
  },
  memory: { backend: "qmd", qmd: { paths: ["…living…"] } },
  plugins: {
    entries: {
      "memory-core": { config: { dreaming: { enabled: true } } },
      "memory-wiki": { config: { vault: { path: "…/GZMO/wiki" } } },
    },
  },
}
```

Optional: create empty local curated file so FTS has a root:

```bash
touch ~/.openclaw/workspace/MEMORY.md
# Keep content operator-scratch; durable lore → takeaway
```

### Phase E — Agent playbooks (behavior)

**Owner:** workspace markdown (respect sync markers)

1. Run `bash scripts/sync-openclaw-workspace.sh` after install so ecosystem blocks stay canonical.
2. Outside markers (or in `TOOLS.local.md` / `AGENTS.md` non-marker regions), add operator rules:

   - Living questions → MCP `gzmo_memory_search` (and friends), not inventing vault lore.
   - Durable facts → `bash bin/openclaw-takeaway.sh '…'`.
   - Local `MEMORY.md` = scratch; never claim it is honeypot.
   - Never Qdrant/Neo4j/chat dual-writer tools.

3. Ensure Telegram agent tool policy allows the MCP tools you need and does **not** expose host exec that can upsert living stores.

### Phase F — Handoff to OpenClaw SOTA map

**Owner:** SOTA grilling ticket **Memory posture**

Paste this resolution gist into that ticket when closing:

> Per GZMO map **SOTA Memory posture handoff**: enable only local FTS — `agents.defaults.memorySearch.provider: "none"`, `sources: ["memory"]`, no `extraPaths`; `memory-core` dreaming **off**; living recall is `gzmo-living` MCP (separate install). Citations `auto`.

Do **not** pull Gateway sandbox/model hardening into this guide; that remains SOTA.

### Phase G — Verification matrix (definition of done)

| # | Check | Pass criteria |
| - | ----- | ------------- |
| G1 | `living-attach-check.sh` | Exit 0 |
| G2 | `openclaw mcp show gzmo-living` | Configured; command = `pi-gzmo-mcp-serve.sh` |
| G3 | `openclaw mcp probe gzmo-living` | Tools include search/status; ops tools excluded |
| G4 | Agent turn can invoke `gzmo_memory_search` | Returns living hits or honest empty — not config error |
| G5 | Takeaway smoke | Artifact `ok: true` or known living refuse with fix path |
| G6 | `openclaw memory status` | Provider `none` (or disabled); Dreaming off; no living `extraPaths` |
| G7 | `openclaw config validate` | OK |
| G8 | `openclaw security audit` / doctor | No new critical from memory posture; dual-writer still inactive |
| G9 | Dual-writer sanity | Workstation `gzmo-serve` inactive while CT101 living |
| G10 | Doctrine spot-check | No Qdrant upsert / Neo4j auto-graph / `--now` in takeaway scripts |

When G1–G10 pass and HITL tickets 03–07 are closed (or §3 explicitly acked), the **wayfinder destination** is satisfied for apply; remaining fog (§3) can stay open without blocking attach.

---

## 5. Ordered apply runbook (copy/paste)

```bash
# 0) Repo root
cd ~/github-clone/GZMO

# 1) Living readiness
bash scripts/living-attach-check.sh   # must be green for full attach

# 2) Install MCP + takeaway + playbooks
bash scripts/install-openclaw-living-attach.sh

# 3) Prove MCP
openclaw mcp show gzmo-living
openclaw mcp probe gzmo-living --json

# 4) Nutrient smoke
bash ~/.openclaw/workspace/bin/openclaw-takeaway.sh 'OpenClawAttachSmoke: …'
cat data-next/openclaw-attach/takeaway-latest.json

# 5) Local memory posture (after dry-run)
openclaw config patch --dry-run --stdin <<'EOF'
{
  agents: { defaults: { memorySearch: { enabled: true, provider: "none", sources: ["memory"] } } },
  plugins: { entries: { "memory-core": { config: { dreaming: { enabled: false } } } } },
  memory: { citations: "auto" },
}
EOF
# then drop --dry-run to apply

openclaw config validate
openclaw memory status --agent main

# 6) Sync workspace ecosystem files
bash scripts/sync-openclaw-workspace.sh

# 7) Gateway reload if needed
systemctl --user restart openclaw-gateway
# or: openclaw mcp reload

# 8) Final matrix — §4 Phase G
```

---

## 6. Schema / CLI skew notes (OpenClaw 2026.7.1-2)

| Docs-forward | Live CLI |
| ------------ | -------- |
| `memory.search` | `agents.defaults.memorySearch` |
| `agents.entries` | `agents.list` |
| QMD “removed” | Schema may still accept `memory.backend` / `memory.qmd` — **do not use** toward living paths |

Always: `openclaw config patch --dry-run` before apply. After upgrades, re-read schema and re-validate this guide’s patches.

---

## 7. Failure catalog

| Symptom | Likely cause | Fix |
| ------- | ------------ | --- |
| `living-attach-check` FAIL, vault facts &lt; 500 | Living vault not populated / wrong vault | Living ops restore; do not lower floor casually |
| `mcp set` probe fail | SSH / CT101 mcp-serve / vault refuse | Fix living; use `--no-probe` only to stage, then re-probe |
| Takeaway `Read-only file system` on sessions | CT101 data mounts | Fix CT101 permissions/mounts |
| Agent “can’t find memory” | MCP not in `mcp.servers` or tools filtered | Re-run install; `mcp tools` filters; Gateway reload |
| Vector search paused / openai errors | Default provider without key | Set `provider: "none"` |
| Accidental second brain | `extraPaths` or dreaming on | Remove paths; `dreaming.enabled: false`; reindex local only if needed |

---

## 8. Doc / code touch list (when implementing)

| Artifact | Change |
| -------- | ------ |
| `~/.openclaw/openclaw.json` | `mcp.servers.gzmo-living`; safe `memorySearch`; dreaming off |
| `~/.openclaw/workspace/bin/openclaw-takeaway.sh` | Installed by script |
| `LIVING_ATTACH.md` / ecosystem hybrids | Via install + `sync-openclaw-workspace.sh` |
| `AGENTS.md` / `TOOLS.local.md` | Operator recall/write rules outside markers |
| SOTA `issues/03-memory-posture.md` | Close with §4 Phase F gist |
| This map’s tickets 03–07 | Resolve HITL; append Decisions so far |
| Optional ADR | Only if you change attach doctrine (hard to reverse) — prefer contract amend over new ADR |

---

## 9. Suggested engineering follow-ons (after attach is green)

These are **product improvements**, not required for destination:

1. **CI/cron canary:** nightly `living-attach-check` + `openclaw mcp probe` → alert on fail.  
2. **Agent skill:** thin OpenClaw skill wrapping takeaway + “living vs local” decision tree.  
3. **Vault floor UX:** clearer operator message when attach refuses &lt;500 facts.  
4. **Session takeaway from Telegram:** slash command → `openclaw-takeaway.sh` without raw shell.  
5. **Read-only export mirrors:** if fog graduates, define a **generated** snapshot dir (not live honeypot) and a separate grilling ticket before any `extraPaths`.

---

## 10. Wayfinder close-out criteria

The map [OpenClaw ↔ GZMO memory attach](../.scratch/openclaw-gzmo-memory/map.md) is complete when:

1. Tickets 03–07 are `resolved` (or §3 defaults formally acked on the map).  
2. Phases A–G pass on the host.  
3. Decisions so far indexes each answer by **ticket title**.  
4. No open product decision remains on whether MCP vs builtin is living truth.

Then hand off day-2 ops to contract + this guide; do not keep a perpetual “conflate stores” ticket — that remains **out of scope**.

---

## Related

- Install: `scripts/install-openclaw-living-attach.sh`  
- Prove: `scripts/living-attach-check.sh`  
- Takeaway: `scripts/openclaw-takeaway.sh`  
- Sync: `scripts/sync-openclaw-workspace.sh`  
- Bridge: `scripts/pi-gzmo-mcp-serve.sh`  
