# OKForge + Observatory — Workstation Production

**Bar:** local forge is hardened, wiki path is automatic and observable **without** `gzmo serve`, operator can recover from failure without archaeology.

**Gate:** `bash scripts/okforge-observatory-check.sh` → `data-next/okforge-observatory/`  
**Living dump:** `bash scripts/wiki-okforge-living-push.sh`  
**CLI glass:** `gzmo observatory` / `gzmo observatory --json`

This is the **knowledge plane** on the telescope. It is **not** the living Keep (CT101 `gzmo-daemon`) and **not** a public GZMO SKU. Private R&D forge — do not publicize.

## Services

> **Living overnight metabolism is CT101 `gzmo-daemon`.**
> Units below are **workstation lab/OKForge** only — keep `gzmo-serve` disabled.

| Unit | Role | URL |
|------|------|-----|
| `okforge.service` | Forge + `/observatory` | http://127.0.0.1:3000 |
| `gzmo-wiki-okforge-push.timer` | Living honeypot → OKCP (no serve) | — |
| `gzmo-okforge-observatory-check.timer` | Knowledge-plane gate every 6h | — |
| `gzmo-serve.service` | Lab metabolism (off by default) | — |
| ~~`gzmo-scheduler.service`~~ | Offline by default (beat-gate only) | — |
| ~~`gzmo-observatory.service`~~ | **Retired** | was `:7777` |

```bash
systemctl --user status okforge
journalctl --user -u okforge -f
bash scripts/okforge-observatory-check.sh
gzmo observatory --json
```

Linger is enabled for user `gzmo` so units survive logout/reboot. On this telescope, systemd unit `WorkingDirectory` is `%h/gzmo_full` (override with a drop-in if the clone path differs).

## Credentials

See `~/.config/okforge/CREDENTIALS.md` (mode 0600).

- Browser: `gzmo` / (see credentials file) → `/observatory`
- Wiki PAT: `OKFORGE_TOKEN` in `~/.config/okforge/env` (never commit)

`okforge-observatory-check.sh` asserts the env name is set; it does not print the value.

## Knowledge path

1. Living Keep (CT101): distill → promote → embed → dream/spark (GREEN gate)
2. **Satellite (no second writer):** `wiki-okforge-living-push.sh` dumps recent honeypot facts over SSH (read-only) and pushes via OKCP to `gzmo/gzmo-next-memory`
3. Manual: `gzmo wiki push --origin manual` or `--from-json FILE`
4. Meta: `wiki-push-latest.json` (`healthy`, `commit_sha`) — Observatory `wiki_push` LED + forge Body panel
5. Failed pushes **write** `healthy=false` so the glass cannot go silent

Wiki failure does **not** flip metabolism GREEN. `gzmo-serve` catch-up cron remains a lab path only.

## Production smoke

```bash
bash scripts/okforge-observatory-check.sh
# docs + dual-writer + forge HTTP + token env + wiki-push meta
bash scripts/okforge-observatory-check.sh --docs-only   # CI / no forge
OKFORGE_CHECK_SOFT=1 bash scripts/okforge-observatory-check.sh  # HTTP WARN if down

# Optional forge-side smoke (private clone, if present):
# bash ~/Schreibtisch/okforge/scripts/production-smoke.sh
```

Must print `okforge_observatory_GREEN` (or YELLOW with only WARN/HOLD). RED = not production.

## Backup / restore

```bash
bash ~/Schreibtisch/okforge/scripts/backup-okforge.sh
# archives under ~/.local/share/okforge-backups/
```

Daily timer (user systemd): `okforge-backup.timer`.

Restore (outline): stop `okforge`, restore `gitea.db` + `gitea-repositories.tar.gz`, start `okforge`.

## Hardening applied

- `HTTP_ADDR=127.0.0.1`, `ROOT_URL=http://127.0.0.1:3000/`
- `DISABLE_REGISTRATION=true`, `RUN_MODE=prod`, Actions off
- Legacy Observatory `:7777` disabled
- Wiki push failures write `healthy=false` meta
- Observatory LEDs: serve/scheduler inactive = expected-offline; serve active = dual-writer DOWN

## Still intentionally deferred

- `kg_reconcile` on `gzmo serve` overnight
- Chaos on serve
- Re-enabling full `gzmo-scheduler` for overnight
- Full eight-view visual parity with old FastAPI Observatory
- Internet-facing TLS deploy (`deploy/` Docker+Caddy) — not this workstation bar
- Publishing OKForge as a GZMO product
