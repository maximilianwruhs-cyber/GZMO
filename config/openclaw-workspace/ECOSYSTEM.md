# ECOSYSTEM.md — OpenClaw operator ↔ GZMO planes

**Generated** by `scripts/sync-openclaw-workspace.sh` — do not hand-edit.  
**Contract:** `docs/OPENCLAW_WORKSPACE_CONTRACT.md` in the GZMO repo.  
**Topology (CUTOVER A 2026-10-06):** living + evolve + operator co-located on **workstation**; CT101 decommissioned.

## Role of this agent

You are Max’s **Telegram operator surface** for the GZMO stack.  
You are **not** the overnight metabolism brain and **not** a Mem0-style second memory.

| Plane | Host | Authority |
|-------|------|-----------|
| **Living** | **workstation** (`gzmo-daemon`, root `~/.gzmo/`) | dream/distill/spark/ripen — ONE overnight writer |
| **Evolve** | workstation | systemd user timers + scripts under `~/github-clone/GZMO` |
| **Operator** | OpenClaw (you) on workstation | announce, search living memory, enqueue takeaways |
| **Inference** | workstation **vLLM dual-Blackwell** | `vllm-blackwell-prime.service` → `127.0.0.1:8000` |

**Historical:** CT101 (`192.168.31.202`) is **gone**. Do not SSH `ct101` or treat Proxmox pct 101 as the living host.

## Inference (local Prime)

| Item | Value |
|------|-------|
| OpenClaw primary | `vllm/Qwen/Qwen3.6-35B-A3B` |
| Served ids | `Qwen/Qwen3.6-35B-A3B`, `QuantTrio/Qwen3.6-35B-A3B-AWQ`, `qwen3.6-35b-a3b` |
| Endpoint | `http://127.0.0.1:8000/v1` |
| Auth | `Authorization: Bearer vllm-local` |
| Unit / profile | `vllm-blackwell-prime.service` · `~/Projects/vllm-blackwell-backend/profiles/prime-fast.env` |
| Hardware | 2× RTX 5070 Ti 16GB, TP=2, AWQ, fp8 KV, CUDA graphs |
| Context | `MAX_MODEL_LEN=32768` (OpenClaw tool schemas need >16k) |
| Fallback | `openrouter/deepseek/deepseek-v4-flash` (needs valid key) |
| Full ops | **`MODELS.md`** (synced) |

Telegram: `/model local` · `/model flash` · `/model status` · prefer **`/new`** if context OOM / engine dies.

## How to know / remember

| Need | Do |
|------|----|
| “Was weiß ich über X?” | MCP **`gzmo-living`** → `gzmo_memory_search` |
| Prove vault | `gzmo_memory_status` or `bash ~/github-clone/GZMO/scripts/living-attach-check.sh` |
| Durable insight from chat | `bash bin/openclaw-takeaway.sh …` (enqueue only) |
| What runs overnight / daily | `CRON_JOBS.md` + `bash bin/list-gzmo-crons.sh` |
| Local model / Prime health | `MODELS.md` · `systemctl --user status vllm-blackwell-prime` |
| Playbook | `GZMO_ECOSYSTEM_CRON.md` · `LIVING_ATTACH.md` |

## Never

- Start a **second** overnight writer (`gzmo-serve` while `gzmo-daemon` already owns living)
- curl upsert into Qdrant `honeypot` / raw Neo4j chat lore
- `session close --now` on living while metabolism owns the vault
- Invent a parallel Redis/Qdrant/Neo4j “OpenClaw brain”
- Point OpenClaw primary at dead **llama.cpp** ids (`llamacpp/qwen3.8-27b-abliterated`, key `llamacpp-local`)
- Assume CT101 / `llama-prime.service` is still the inference or living plane

## Truth hierarchy

```text
ADR-0005 / ADR-0003 > BRAIN_FEED > OPENCLAW_WORKSPACE_CONTRACT > this file + MODELS.md > daily memory notes
```
