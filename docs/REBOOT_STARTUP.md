# Reboot & startup

## Topology after reboot

| Tier | What | How it starts |
|------|------|----------------|
| **Workstation** | Prime `:8000` | `gzmo-prime.service` (user systemd) |
| **Workstation** | GZMO daemon | `gzmo-daemon.service` |
| **VM200** | Retrieval router `:8081` | `llama-retrieval-router.service` via `deploy-retrieval-router.sh` |
| **LXC101** | Qdrant, Neo4j, Redis | Always-on LXC services |

GZMO daemon and Pi KB both use VM200 `:8081` for embed + rerank (`gzmo-embed`, `gzmo-rerank` presets).  
Session distill extract/summary → Prime `:8000`.

## One-time setup

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/install-boot-stack.sh              # prime + daemon
./scripts/vm200/deploy-retrieval-router.sh   # VM200 router
```

## After every reboot

```bash
./scripts/after-boot-verify.sh

ssh -i ~/.ssh/id_sidecar_proxmox maximilian@192.168.31.110 \
  'systemctl is-active llama-retrieval-router'
```

## Service commands

```bash
systemctl --user status gzmo-prime gzmo-daemon
journalctl --user -u gzmo-prime -f
```

## Pi KB sync

```bash
./scripts/pi-kb-reindex.sh
```

## Benchmark

```bash
./scripts/vm200/retrieval-bench/runner.py \
  --profile profiles/post-router-qwen3.json --tag smoke
```

See [VM200_RETRIEVAL_BENCH.md](./VM200_RETRIEVAL_BENCH.md).
