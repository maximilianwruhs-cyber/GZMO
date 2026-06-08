# Reboot & startup

## Topology after reboot

| Tier | What | How it starts |
|------|------|----------------|
| **Workstation** | Prime `:8000` | `gzmo-prime.service` (user systemd) |
| **Workstation** | Pi KB embed `:8002` | `gzmo-embed.service` (CPU, `GZMO_EMBED_NGL=0`) |
| **Workstation** | GZMO daemon | `gzmo-daemon.service` |
| **VM200** | Embed `:8081`, rerank `:8082`, librarian `:8083` | `deploy-retrieval-layer.sh` (systemd on VM) |
| **LXC101** | Qdrant, Neo4j | Always-on LXC services |

Pi `knowledge_search` → local `:8002` (fallback VM200 `:8081`).  
GZMO daemon → VM200 `:8081` per `gzmo.toml`.

## One-time setup (already run if you used install-boot-stack)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/install-boot-stack.sh
```

This installs units, enables linger, enables `gzmo-embed`, `gzmo-prime`, `gzmo-daemon`.

## After every reboot

```bash
# 1. Workstation verify
./scripts/after-boot-verify.sh

# 2. VM200 (if retrieval down)
ssh -i ~/.ssh/id_sidecar_proxmox maximilian@192.168.31.110 \
  'systemctl is-active llama-embed llama-rerank llama-librarian'

# 3. Pi KB catch-up (changed files only)
./scripts/pi-kb-reindex.sh
```

## Manual stack (without systemd)

```bash
./scripts/start-production.sh --daemon
# Also starts local :8002 when gzmo.toml points at localhost; Pi always needs :8002:
systemctl --user start gzmo-embed.service
```

## Service commands

```bash
systemctl --user status gzmo-embed gzmo-prime gzmo-daemon
journalctl --user -u gzmo-embed -f
journalctl --user -u gzmo-prime -f
journalctl --user -u gzmo-daemon -f
```

If `gzmo-prime` restart-loops while an old manual `llama-server :8000` is running, stop the stray process or run `systemctl --user restart gzmo-prime` after `start-prime.sh` path fix (uses `llama.cpp/build/bin/llama-server`).

## Optional: HSP

```bash
systemctl --user enable --now hsp-synth.service hsp-pipeline.service
```

## Config references

- Pi KB: `~/.pi/agent/knowledge-base.json`, `~/.pi/agent/docs/KNOWLEDGE_BASE.md`
- GZMO: `gzmo.toml`, `docs/INFRASTRUCTURE_REVIEW.md`
