# Pi upgrade runbook (stop the breakages)

**Status:** Active (2026-07-19)  
**Why:** Pi was customized via packages + `mcp.json` + discovery CLI flags + host paths. Official Pi updates often wiped one layer and left the others half-alive.

---

## Before upgrading Pi

1. Snapshot agent home:
   ```bash
   cp -a ~/.pi/agent ~/.pi/agent.bak-$(date -u +%Y%m%dT%H%M%SZ)
   ```
2. Record versions:
   ```bash
   pi --version || true
   python3 -c "import json;print(json.load(open('$HOME/.pi/agent/settings.json')).get('lastChangelogVersion'))"
   ```
3. Decide mode for this host: **living** (CT101) or **product** (`~/.gzmo`) — not both silently.

---

## After upgrading Pi

### 1. Packages still listed

`~/.pi/agent/settings.json` should still include (merge, don’t replace blindly):

```json
"packages": [
  "npm:pi-mcp-adapter",
  "npm:pi-subagents",
  "npm:gzmo-pi"
]
```

Optional: `npm:hsp-pi` (audio). Prefer **one** of `npm:gzmo-pi` *or* git gzmo-pi, not both fighting.

Subagents snippet: [pi-settings-subagents.snippet.json](pi-settings-subagents.snippet.json).

### 2. MCP servers in `mcp.json` (required)

`pi-mcp-adapter` reads **`~/.pi/agent/mcp.json`**, not only `settings.json`.

**Living:**

```bash
bash scripts/install-shared-mcp.sh
# Expect gzmo-living → scripts/pi-gzmo-mcp-serve.sh → CT101
```

**Product:**

```bash
bash scripts/install-product-mcp.sh
# Expect gzmo-memory → GZMO_CONFIG=~/.gzmo/gzmo.toml GZMO_PRODUCT=1
```

Verify:

```bash
bash scripts/mcp-attach-check.sh          # product hygiene (gzmo-memory)
# Living: in Pi/Cursor call tools on gzmo-living → path /opt/gzmo/… , facts ~60k
```

### 3. Discovery / automation scripts

Grep for footguns after upgrade:

```bash
rg -n --no-heading 'no-extensions|--extension |maximilian-wruhs|survey_GZMO' \
  ~/github-clone/gzmo_skills/scripts ~/github-clone/GZMO/scripts 2>/dev/null || true
```

- **Never** pass `--no-extensions` if you need MCP  
- Remove stale `--extension` paths that do not exist  

### 4. Session / Redis continuity

For living operator work:

```bash
./scripts/pi-gzmo-memory.sh session
./scripts/pi-gzmo-memory.sh prep "smoke query"
```

### 5. Smoke ladder

| Step | Command / action |
|------|------------------|
| CT101 living | `bash scripts/ct101-living-smoke.sh` |
| Living gate | `bash scripts/living-readiness-gate.sh` |
| Product gate | `bash scripts/product-readiness-gate.sh` |
| Pi MCP | `gzmo_memory_status` then `gzmo_memory_search` |
| Optional CCR | only if Headroom branch re-landed — see [HEADROOM_CCR.md](HEADROOM_CCR.md) |

---

## Anti-patterns

| Don’t | Do |
|-------|----|
| Edit only `settings.json` packages | Also rewrite `mcp.json` via install scripts |
| Point Pi at workstation `data-next/` for “production” memory | Use CT101 living attach |
| Expect Redis cache from `pi-subagents` alone | Use GZMO MCP / `delegate_task` / bridge session id |
| Assume `[ccr:…]` works on living `main` | Re-land `feat/context-compress-headroom` first |
| Store Neo4j passwords in committed docs | Pull from CT101 `.env` via `install-shared-mcp.sh`; rotate if leaked into settings |

---

## Pin policy (recommended)

1. Pin Pi version in operator notes after a green smoke  
2. Pin `pi-mcp-adapter` / `pi-subagents` versions when upstream churns  
3. Treat gzmo-pi git vs npm as a deliberate choice; delete the unused entry  
4. After any pin bump: run this runbook’s smoke ladder before discovery cron  
