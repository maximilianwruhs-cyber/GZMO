# Reboot & startup

## Topology after reboot

| Tier | What | How it starts |
|------|------|----------------|
| **Cognition** | Cloud LLM (OpenRouter) | Fully online (needs outbound HTTPS) |
| **Retrieval** | VM200 `:8081` | `llama-retrieval-router.service` via `deploy-retrieval-router.sh` |
| **Persistence** | LXC101 | Always-on Docker services (Neo4j, Qdrant, Redis) |
| **Orchestration** | GZMO daemon (LXC101) | `gzmo-daemon.service` (system systemd unit) |

GZMO daemon on LXC101 uses VM200 `:8081` for embed + rerank (`gzmo-embed`, `gzmo-rerank` presets), and OpenRouter for all LLM cognition.

## One-time setup

On LXC101 (homelab homing):
```bash
# Deployed via scripts/lxc101/deploy-gzmo-daemon.sh
# Systemd service starts automatically on boot:
sudo systemctl enable --now gzmo-daemon.service
```

## After every reboot

Verify all services are up:
```bash
ssh -i ~/.ssh/id_sidecar_proxmox maximilian@192.168.31.202 \
  'sudo systemctl is-active gzmo-daemon'

ssh -i ~/.ssh/id_sidecar_proxmox maximilian@192.168.31.110 \
  'systemctl is-active llama-retrieval-router'
```

## Service commands (on LXC101)

```bash
sudo systemctl status gzmo-daemon
sudo journalctl -u gzmo-daemon -f
```

## Pi KB sync (on LXC101)

```bash
cd /opt/gzmo/survey_GZMO
./scripts/pi-kb-reindex.sh
```
