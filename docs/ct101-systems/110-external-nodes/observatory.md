# GZMO Observatory — Operator glass (production)

**Surfaces:** `gzmo observatory` (TUI / `--json`) + OKForge `/observatory` on `:3000`  
**Parent:** [110-external-nodes/SYSTEM.md](./SYSTEM.md)  
**Gate:** `bash scripts/okforge-observatory-check.sh` → `data-next/okforge-observatory/`  
**Forge ops:** [OKFORGE_PRODUCTION.md](../../OKFORGE_PRODUCTION.md)

---

## Capability

Read-only operator glass for the **knowledge plane** (local forge + wiki push) and workstation unit honesty. It does **not** write the living vault. Overnight metabolism stays on the living host (`gzmo-daemon` on CT101 today).

| Surface | What it is | What it is not |
|---------|------------|----------------|
| `gzmo observatory` | Health LED board (units + probes + wiki plane) | Chat cockpit |
| `gzmo observatory --json` | Scriptable snapshot, fail-closed if OKForge wiki plane is DOWN | Living GREEN gate |
| `http://127.0.0.1:3000/observatory` | In-forge agent-discovery UI | Public SKU / internet TLS |
| FastAPI `:7777` (`gzmo-observatory.service`) | **Retired** | Do not start; do not require |

Theater (sanitized scoreboard / wiki-mind demo) stays on `bash scripts/wiki-observatory-demo.sh` and is **not** this production bar.

---

## How it works

### LED honesty (telescope)

`gzmo-serve` and `gzmo-scheduler` **inactive** is **expected-offline** (UP), not a red outage. `gzmo-serve` **active** is DOWN (dual-writer risk). `llama-prime.service` inactive is UNKNOWN — judge the LLM via health probes, not the unit name.

When `[wiki] backend = "okforge"`:

- `okforge.service` — systemd
- `okforge_http` — GET `/observatory` (401/403 still counts as up)
- `wiki_push` — `wiki-push-latest.json` (`healthy`, `commit_sha`)

Failed OKCP pushes **write** `healthy=false` so the glass cannot go silent.

### Living wiki satellite (no second writer)

```bash
bash scripts/wiki-okforge-living-push.sh          # CT101 honeypot dump → local OKCP
bash scripts/wiki-okforge-living-push.sh --dry-run
```

Refuses if workstation `gzmo-serve` is active. Does not copy `vault.db`. Timer: `gzmo-wiki-okforge-push.timer` (05:30 UTC).

---

## Interfaces

| Interface | Value |
|-----------|-------|
| Forge UI | `http://127.0.0.1:3000/observatory` (`okforge.service`) |
| CLI | `gzmo observatory` / `gzmo observatory --json` |
| Wiki push | `gzmo wiki push` / `--from-json` living dump |
| Production gate | `scripts/okforge-observatory-check.sh` |
| Retired sidecar | `:7777` FastAPI — historical only |

---

## Historical FastAPI sidecar (retired)

The 2026-07 workstation dashboard (`gzmo-observatory/observatory/*.py`) polled CT101 every 5s via `ssh pve` → `pct exec 101`, plus LAN Qdrant/Neo4j. That unit is **disabled**. Insights still useful:

> **THINKING — observatory:SSH single point**  
> One round-trip minimized latency; SSH failure showed a single error blob. Prefer per-subsystem last-good timestamps if a remote poller is ever revived.

> **THINKING — observatory:LAN sidecar reads**  
> Direct Qdrant/Neo4j LAN reads bypassed SSH. Flat LAN trust; no TLS on sidecar HTTP.

Do not re-enable `:7777` as a second control plane.

---

## Advancement

| Now | Later (deferred) |
|-----|------------------|
| Honest LEDs + `--json` + living OKCP satellite | Eight-view visual parity with old FastAPI UI |
| Localhost forge + daily backup | Internet-facing TLS (`deploy/` Docker+Caddy) |
| Soft-fail wiki (not living GREEN) | Public “mind” SKU |

Private R&D: do not publicize the OKForge / herdr mirrors as GZMO product.
