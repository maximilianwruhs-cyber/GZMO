# OKForge + Observatory — Workstation Production

**Bar:** local forge is hardened, overnight wiki path is automatic and observable, operator can recover from failure without archaeology.

## Services

> **Living overnight metabolism is CT101 `gzmo-daemon`** (2026-07-17 restore).
> Units below are **workstation lab/OKForge** only — keep `gzmo-serve` disabled by default.

| Unit | Role | URL |
|------|------|-----|
| `okforge.service` | Forge + `/observatory` | http://127.0.0.1:3000 |
| `gzmo-serve.service` | Lab metabolism + soft-fail wiki push (off by default) | — |
| ~~`gzmo-scheduler.service`~~ | Offline by default (beat-gate only) | — |
| ~~`gzmo-observatory.service`~~ | **Retired** | was `:7777` |

```bash
systemctl --user status okforge gzmo-serve
journalctl --user -u okforge -f
journalctl --user -u gzmo-serve -f
```

Linger is enabled for user `gzmo` so units survive logout/reboot.

## Credentials

See `~/.config/okforge/CREDENTIALS.md` (mode 0600).

- Browser: `gzmo` / (see credentials file) → `/observatory`
- Wiki PAT: `OKFORGE_TOKEN` in `~/.config/okforge/env` (loaded by `gzmo-serve.service.d/okforge.conf`)

## Knowledge path

1. Metabolism (`gzmo serve`): distill → promote → embed → dream/spark (GREEN gate)
2. Soft-fail satellite at `[wiki] push_cron_*` (default 05:30 UTC): typed OKCP push → `gzmo/gzmo-next-memory`
3. Manual: `gzmo wiki push --origin manual`
4. Meta: `data-next/wiki-push-latest.json` (`healthy`, `commit_sha`) — Observatory Body panel
5. Run record: `data-next/scheduler-runs/latest-wiki.json` (does **not** affect metabolism GREEN)

Wiki failure logs `ok=false` and leaves metabolism verdict GREEN.

## Production smoke

```bash
bash ~/Schreibtisch/okforge/scripts/production-smoke.sh
```

Must print `PRODUCTION SMOKE: GREEN`.

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
- Legacy Observatory disabled
- Wiki push fails write `healthy=false` meta and non-zero job record

## Still intentionally deferred

- `kg_reconcile` on `gzmo serve` overnight
- Chaos on serve
- Re-enabling full `gzmo-scheduler` for overnight
- Full eight-view visual parity with old FastAPI Observatory
- Internet-facing TLS deploy (`deploy/` Docker+Caddy) — not this workstation bar
