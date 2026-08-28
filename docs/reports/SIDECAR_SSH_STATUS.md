# Sidecar SSH Access Status

**Updated:** 2026-07-08

## Key inventory

| Key | Path | Status |
|-----|------|--------|
| `id_sidecar_proxmox` | `~/.ssh/id_sidecar_proxmox` | **Missing** — documented in swap/GZMO docs but not provisioned on this workstation |
| `id_ed25519` | `~/.ssh/id_ed25519` | **Present** — works for `maximilian@192.168.31.110` (VM200) |

## Reachability matrix

| Target | Method | Result |
|--------|--------|--------|
| VM200 (`192.168.31.110`) | `ssh -i ~/.ssh/id_ed25519 maximilian@192.168.31.110` | OK (hostname: `ollamagpu`) |
| Proxmox (`192.168.31.200`) | `ssh root@192.168.31.200` | Password auth required (pubkey not authorized) |
| CT101 (`192.168.31.202`) | Direct SSH as `maximilian` | Denied (pubkey) |
| CT101 | `pct exec 101` via Proxmox root | OK |
| CT101 Qdrant | `curl http://192.168.31.202:6333/collections` | OK |

## Recommended ops commands

Use `-F /dev/null` if system ssh_config has permission issues.

```bash
# VM200 retrieval layer
ssh -F /dev/null -i ~/.ssh/id_ed25519 maximilian@192.168.31.110 "nvidia-smi"

# CT101 daemon health (via Proxmox)
ssh -F /dev/null root@192.168.31.200 \
  "pct exec 101 -- /opt/gzmo/survey_GZMO/target/release/gzmo health"

# Restart sidecar databases
ssh -F /dev/null root@192.168.31.200 \
  "pct exec 101 -- sh -c 'cd /opt/database-cluster && docker compose restart'"
```

## Provisioning `id_sidecar_proxmox` (optional)

```bash
ssh-keygen -t ed25519 -f ~/.ssh/id_sidecar_proxmox -N "" -C "gzmo-sidecar-proxmox"
python3 ~/github-clone/swap/scripts/authorize_vm.py   # VM200
# Then authorize on Proxmox root and CT101 via pct exec authorized_keys
```
