# CT101 deploy layout

**Living host:** LXC 101 (`192.168.31.202`)  
**Related:** [CT101_PATH_AUTHORITY.md](./CT101_PATH_AUTHORITY.md), [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md), [CT101_BOUNDARY.md](./CT101_BOUNDARY.md)

## Canonical paths

| Path | Role |
|------|------|
| `/opt/gzmo/` | Runtime root (`WorkingDirectory`, `gzmo.toml`, `data/`, `.env`) |
| `/opt/gzmo/current` | Symlink → release tree (today: `survey_GZMO`) |
| `/opt/gzmo/current/target/release/gzmo` | Production binary |
| `/opt/gzmo/data/vault.db` | Living SQLite vault |
| `/opt/gzmo/gzmo.toml` | Living config (`GZMO_CONFIG`) |
| `/opt/gzmo/data/gzmo_mentor.sock` | Living Unix mentor API (`ping` / `status` / `teach`) |

Do not treat `survey_GZMO` as the public name — use `/opt/gzmo/current`.

## Sidecars (living appliance)

Living Redis / Qdrant / Neo4j are the **living appliance** (airgap USP substrate). In-repo pin (matches CT101 `/opt/database-cluster` topology):

| Path | Role |
|------|------|
| [`deploy/living-appliance/`](../../deploy/living-appliance/) | Compose pin |
| [`config/living-appliance.gzmo.toml.example`](../../config/living-appliance.gzmo.toml.example) | Daemon sidecar fragment |
| [`scripts/living-appliance-up.sh`](../../scripts/living-appliance-up.sh) | `docker compose up -d` + gate |
| [`docs/LIVING_APPLIANCE.md`](../LIVING_APPLIANCE.md) · [`AIRGAP_LIVING.md`](../AIRGAP_LIVING.md) | Living doctrine |

```bash
bash scripts/living-appliance-up.sh
bash scripts/living-appliance-gate.sh
```

Production CT101 may keep sidecars under `/opt/database-cluster`; prefer aligning that host with the in-repo pin over time. Daemon config: `/opt/gzmo/gzmo.toml` (not product `~/.gzmo`).

## Systemd

```ini
WorkingDirectory=/opt/gzmo
Environment=GZMO_CONFIG=/opt/gzmo/gzmo.toml
ExecStartPre=/opt/gzmo/current/scripts/ingest-quality/gate-pre-deploy.sh
ExecStart=/opt/gzmo/current/target/release/gzmo daemon
```

After path changes: `systemctl daemon-reload && systemctl restart gzmo-daemon`.

## Operator access

```bash
ssh ct101                 # ProxyJump via pve
ssh pve "pct exec 101 -- …"   # fallback
```

## Deploying a new binary

Build **on CT101** (workstation glibc is newer — scp’d binaries fail with `GLIBC_2.39 not found`):

```bash
rsync -az --exclude target --exclude .git \
  /home/gzmo/github-clone/GZMO/ ct101:/opt/gzmo/current/
# Force-sync Rust sources when partial trees lag (memory/*.rs especially):
rsync -az gzmo-core/src/ ct101:/opt/gzmo/current/gzmo-core/src/
ssh ct101 'bash -lc "
  cd /opt/gzmo/current && cargo build --release -p gzmo-cli
  chmod +x scripts/ingest-quality/gate-pre-deploy.sh   # required — rsync drops +x
  systemctl restart gzmo-daemon
"'
```

## M5 ripen honesty

Living job `honeypot_ripen` shells `scripts/export-knowledge-core.py` (dual gate: confidence≥0.90 ∧ recall≥3 ∧ allowed origins). **0 rows overnight** usually means starved `recall_count`, not an empty `knowledge_core.db` (often already tens of thousands of rows). After Felt Use / living search:

```bash
GZMO_CONFIG=/opt/gzmo/gzmo.toml gzmo ripen status
python3 /opt/gzmo/current/scripts/export-knowledge-core.py \
  --db /opt/gzmo/data/vault.db --output /opt/gzmo/data/knowledge_core.db
# advice + gates → /opt/gzmo/data/ripen/latest.json
```

Prefer job prompt path `/opt/gzmo/current/scripts/…` (not a stale `survey_GZMO` string).

## Sync docs/scripts only (no rebuild)

When USP / Brain Feed shell gates land on `main` but the binary is unchanged, sync scripts + docs and restore `+x` (rsync drops execute bits):

```bash
# From workstation clone on main (dual-writer safe — never restarts gzmo-daemon):
bash scripts/ct101-brain-feed-sync.sh
# HOST override: CT101_SSH_HOST=ct101 bash scripts/ct101-brain-feed-sync.sh
```

Do **not** restart `gzmo-daemon` for script-only syncs. The sync script asserts `ActiveEnterTimestamp` unchanged.

## TinyFolder overnight timer (living host)

```bash
# After script sync (or binary deploy):
ssh ct101 'bash /opt/gzmo/current/scripts/install-tinyfolder-overnight-timer.sh'
# Drop notes into /opt/gzmo/data/inbox/ — timer fires ~02:45 UTC
```

## Quality / Brain Feed gates (workstation → CT101)

```bash
bash scripts/keep-quality-gate.sh          # USP living quality bar
bash scripts/keep-quality-soak.sh --summary
bash scripts/brain-feed-check.sh           # vault-nourishing Unpark nutrients
bash scripts/brain-intel-promote.sh        # calibration/Arena suggestion only (no auto)
```

See [KEEP_QUALITY.md](../KEEP_QUALITY.md) · [BRAIN_FEED.md](../BRAIN_FEED.md).

## Product gate

From workstation (SSH):

```bash
bash scripts/ct101-living-smoke.sh
```

On CT101 (local, no SSH):

```bash
bash /opt/gzmo/current/scripts/ct101-living-smoke-local.sh
```

### Hourly timers

**On CT101** (preferred living gate):

```bash
sudo cp /opt/gzmo/current/systemd/ct101-living-smoke.{service,timer} /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now ct101-living-smoke.timer
systemctl list-timers ct101-living-smoke.timer
```

**On workstation** (operator mirror):

```bash
mkdir -p ~/.config/systemd/user
cp systemd/gzmo-ct101-living-smoke.{service,timer} ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now gzmo-ct101-living-smoke.timer
```

### Embed backfill (drift)

Full ops scar book: [CT101_QDRANT_EMBED_OPS.md](./CT101_QDRANT_EMBED_OPS.md).

```bash
ssh ct101 'bash /opt/gzmo/current/scripts/ct101-embed-backfill-loop.sh'
```

### Qdrant orphan prune

After large syncs, remove points not in `honeypot is_latest=1` (**`--dry-run` first**):

```bash
ssh ct101 'python3 /opt/gzmo/current/scripts/ct101-qdrant-prune-orphans.py --dry-run'
ssh ct101 'python3 /opt/gzmo/current/scripts/ct101-qdrant-prune-orphans.py'
```
