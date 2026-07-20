# Reboot & startup

## Topology after reboot

| Tier | What | How it starts |
|------|------|----------------|
| **Workstation** | Prime `:8000` | **`llama-prime.service`** (user systemd; enabled + linger) |
| **Workstation** | Pi KB embed `:8002` | Optional `gzmo-embed.service` (CPU, `GZMO_EMBED_NGL=0`) — soft |
| **Workstation** | Local lab daemon | Optional `gzmo-daemon.service` / manual `gzmo daemon` — **not** CT101 living |
| **VM200** | Embed `:8081`, rerank `:8082`, librarian `:8083` | `deploy-retrieval-layer.sh` (systemd on VM) |
| **LXC101 / CT101** | Qdrant, Neo4j, **living `gzmo-daemon`** | Always-on — **do not reboot with the workstation** |

Pi `knowledge_search` → local `:8002` when up (fallback VM200 `:8081`).  
Product MCP → `~/.gzmo` only. Living MCP → CT101 / `gzmo-living` label — never mix.

## One-time setup

```bash
cd ~/github-clone/GZMO
# Prime (current workstation path)
# unit lives at ~/.config/systemd/user/llama-prime.service — already enabled if linger=yes

# Optional full stack units from repo templates:
./scripts/install-boot-stack.sh   # installs gzmo-embed / gzmo-prime / gzmo-daemon
```

`install-boot-stack` enables linger and optional `gzmo-*` units. This host’s **canonical Prime** unit is still **`llama-prime`**.

## After every reboot

```bash
# 0. Product MCP attach (often drifts after reboot)
MCP_ATTACH_FIX=1 bash scripts/mcp-attach-check.sh

# 1. Workstation verify
./scripts/after-boot-verify.sh

# 2. VM200 (if retrieval down)
ssh maximilian@192.168.31.110 \
  'systemctl is-active llama-embed llama-rerank llama-librarian'

# 3. Keep gates
bash scripts/production-readiness-gate.sh

# 4. Pi KB catch-up (changed files only)
./scripts/pi-kb-reindex.sh
```

## Manual stack (without systemd)

```bash
./scripts/start-production.sh --daemon
# Pi KB local embed if needed:
systemctl --user start gzmo-embed.service   # only if installed
```

## Service commands

```bash
systemctl --user status llama-prime.service
journalctl --user -u llama-prime -f

# Optional units (if install-boot-stack was run):
systemctl --user status gzmo-embed gzmo-prime gzmo-daemon
```

If Prime restart-loops while a stray manual `llama-server :8000` is running, stop the stray process, then `systemctl --user restart llama-prime`.

## Optional: HSP

```bash
systemctl --user enable --now hsp-synth.service hsp-pipeline.service
```

## Config references

- Pi KB: `~/.pi/agent/knowledge-base.json`, `~/.pi/agent/docs/KNOWLEDGE_BASE.md`
- GZMO lab: `gzmo.toml` / `config/gzmo-next.toml`
- Product: `~/.gzmo/gzmo.toml`
- Reboot card (runtime): `data-next/reboot-prep/latest.md`
