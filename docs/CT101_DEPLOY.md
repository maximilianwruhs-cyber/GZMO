# CT101 deploy layout

**Living host:** LXC 101 (`192.168.31.202`)  
**Related:** [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md), [CT101_BOUNDARY.md](./CT101_BOUNDARY.md)

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
ssh ct101 'bash -lc "
  cd /opt/gzmo/current && cargo build --release -p gzmo-cli
  chmod +x scripts/ingest-quality/gate-pre-deploy.sh
  systemctl restart gzmo-daemon
"'
```

## Product gate

From workstation:

```bash
bash scripts/ct101-living-smoke.sh
```
