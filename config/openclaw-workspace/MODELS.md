# MODELS.md — local inference plane (synced)

**Generated** by `scripts/sync-openclaw-workspace.sh` — edit the source under `config/openclaw-workspace/MODELS.md`.  
**As of:** 2026-10-06

## What runs (live)

| Alias | Ref | Backend |
|-------|-----|---------|
| `local` (default primary) | `vllm/Qwen/Qwen3.6-35B-A3B` | **vLLM** `vllm-blackwell-prime.service` → `127.0.0.1:8000` |
| served ids | `Qwen/Qwen3.6-35B-A3B` · `QuantTrio/Qwen3.6-35B-A3B-AWQ` · `qwen3.6-35b-a3b` | dual RTX 5070 Ti, TP=2, AWQ, fp8 KV, CUDA graphs (`prime-fast`) |
| `flash` | `openrouter/deepseek/deepseek-v4-flash` | OpenRouter fallback (valid key required) |

| Setting | Value |
|---------|-------|
| Config | `~/.openclaw/openclaw.json` |
| Local API key | `vllm-local` (`openclaw-gateway.service.d/vllm.conf` + provider `apiKey`) |
| Gateway | `openclaw-gateway.service` · Telegram `@gzmo0815_bot` |
| Profile | `~/Projects/vllm-blackwell-backend/profiles/prime-fast.env` |
| Measured | `~/Projects/vllm-blackwell-backend/docs/NUCLEAR_MEASURED.md` |

## Chat-safe knobs

- `MAX_MODEL_LEN=32768` · `GPU_MEMORY_UTILIZATION=0.90` · `MAX_NUM_SEQS=4`
- `ENFORCE_EAGER=0` (graphs) · thinking off via `DEFAULT_CHAT_TEMPLATE_KWARGS={"enable_thinking":false}`
- OpenClaw `contextWindow=32768`, `maxTokens=4096`, compaction `reserveTokens≈8000`
- Tool schemas alone exceed 16k tokens — do **not** drop ctx below that for this gateway
- Fat Telegram sessions can CUDA-OOM EngineCore → Telegram **`/new`** or **`/reset`**

## Telegram

```text
/model list
/model local
/model flash
/model status
/model -s flash
/model default
/new
```

## Ops

```bash
systemctl --user status vllm-blackwell-prime openclaw-gateway
curl -fsS -H "Authorization: Bearer vllm-local" http://127.0.0.1:8000/v1/models
systemctl --user restart vllm-blackwell-prime
systemctl --user restart openclaw-gateway
export PATH=~/.local/share/fnm/node-versions/v24.18.0/installation/bin:$PATH
openclaw health
openclaw models status
# real gateway turn (no Telegram deliver):
openclaw agent --agent main --session-id probe-$(date +%s) --message ping --json
```

## Retired (do not restore as primary)

- `llama-prime.service` / llama.cpp on `:8000`
- OpenClaw ids `llamacpp/qwen3.8-27b-abliterated` · key `llamacpp-local`
- CT101 as inference host
