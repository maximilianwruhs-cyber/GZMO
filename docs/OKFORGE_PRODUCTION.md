# OKForge + Observatory — Workstation Production

**Bar:** local forge is hardened, overnight wiki path is automatic and observable, operator can recover from failure without archaeology.

## Services

| Unit | Role | URL |
|------|------|-----|
| `okforge.service` | Forge + `/observatory` | http://127.0.0.1:3000 |
| `gzmo-scheduler.service` | Overnight loops + wiki push | — |
| ~~`gzmo-observatory.service`~~ | **Retired** | was `:7777` |

```bash
systemctl --user status okforge gzmo-scheduler
journalctl --user -u okforge -f
```

Linger is enabled for user `gzmo` so units survive logout/reboot.

## Credentials

See `~/.config/okforge/CREDENTIALS.md` (mode 0600).

- Browser: `gzmo` / (see credentials file) → `/observatory`
- Wiki PAT: `OKFORGE_TOKEN` in `~/.config/okforge/env` (loaded by scheduler drop-in)

## Knowledge path

1. Distill / dream / 05:30 catch-up → `wiki-okforge-push.sh --live`
2. `gzmo wiki push` → OKCP session → commit on `main` (`AGENT_REQUIRE_PR=false` locally)
3. Repo: `gzmo/gzmo-next-memory`
4. Meta: `data-next/wiki-push-latest.json` (`healthy`, `commit_sha`) — Body panel in Observatory

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
- Wiki push fails write `healthy=false` meta and non-zero exit

## Still intentionally deferred

- `kg_reconcile.dry_run=true` until one verified Neo4j apply
- Ingest watcher off (batch inbox only)
- Full eight-view visual parity with old FastAPI Observatory
- Internet-facing TLS deploy (`deploy/` Docker+Caddy) — not this workstation bar
