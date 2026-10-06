### Cron jobs (GZMO + OpenClaw)

**`cron` tool DENIED** — use exec / files. Refresh: `bash bin/list-gzmo-crons.sh` → updates via sync.

1. `CRON_JOBS.md` + `GZMO_ECOSYSTEM_CRON.md`
2. Never claim “no cron jobs” without those checks

### Inference (workstation Prime — vLLM dual-Blackwell)

- **Unit:** `systemctl --user status vllm-blackwell-prime`
- **URL:** `http://127.0.0.1:8000/v1` · **Auth:** `Bearer vllm-local`
- **OpenClaw primary:** `vllm/Qwen/Qwen3.6-35B-A3B`
- **Served ids:** `Qwen/Qwen3.6-35B-A3B` · `QuantTrio/Qwen3.6-35B-A3B-AWQ` · `qwen3.6-35b-a3b`
- **Profile/repo:** `~/Projects/vllm-blackwell-backend` · `profiles/prime-fast.env` (32k ctx, TP=2, graphs)
- **Ops sheet:** **`MODELS.md`** (synced) · measured: repo `docs/NUCLEAR_MEASURED.md`
- **Gateway:** `openclaw-gateway.service` · Telegram model switches in `MODELS.md`
- **Retired:** `llama-prime` / `llamacpp/*` primary · key `llamacpp-local`

### Living host (post-CT101)

- **CT101 decommissioned.** Do not SSH `ct101` / `192.168.31.202` as if alive.
- **Living:** workstation `gzmo-daemon` · root `~/.gzmo/`
- **Evolve:** same host, user timers under `~/github-clone/GZMO`
- Mutex still applies before any second `gzmo-serve`

### Living memory (gzmo-living MCP)

- Search: `gzmo_memory_search` / `gzmo_memory_status`
- Write nutrient: `bash bin/openclaw-takeaway.sh …`
- Playbook: `LIVING_ATTACH.md` · `ECOSYSTEM.md`

Personal/local nicknames → `TOOLS.local.md` (not overwritten by sync).
