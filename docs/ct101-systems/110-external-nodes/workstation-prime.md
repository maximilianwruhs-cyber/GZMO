# Workstation Prime — LLM Fallback Node

**Source:** CT101 config `[engine.local]`, live probe, `gzmo-observatory/observatory/collector.py`  
**Parent:** [110-external-nodes/SYSTEM.md](./SYSTEM.md)

---

## Capability

When CT101's **cloud** engine (OpenRouter GLM 5.2) is unavailable or `cloud_first_background` policy applies, the daemon falls back to a **local OpenAI-compatible** LLM on the workstation at `192.168.31.184:8000`. Live probe: **ornith-35b** model served from workstation Prime bench.

CT101 does not host the Prime server — it is purely an outbound HTTP client.

---

## How it works

### Gateway routing (conceptual)

CT101 `gzmo.toml` defines:

- `[engine.cloud]` — primary (`active_mode=cloud`)
- `[engine.local]` — fallback URL `http://192.168.31.184:8000/v1` (typical)

`GatewayRouter` in `gzmo-core` selects engine per `TaskKind` with Obolus metering. Daemon heartbeat pings active engine:

```97:100:github-clone/GZMO/gzmo-cli/src/daemon_cmd.rs
    heartbeat.add_check(HealthPing {
        url: format!("{}/models", config.engine.active_engine().url),
        service_name: "LLM Engine".to_string(),
    });
```

### Observatory Prime poll

```16:16:home/gzmo/gzmo-observatory/observatory/config.py
PRIME_URL = "http://127.0.0.1:8000"
```

```53:54:home/gzmo/gzmo-observatory/observatory/collector.py
        prime = self._fetch_prime()
        gpu = self._fetch_gpu()
```

Observatory reads **local** Prime from workstation loopback — displays model load alongside CT101 chaos metrics. This is telemetry only; CT101 uses LAN IP.

### Operator CLI

Workstation `gzmo chat` / `gzmo tui` may use Prime directly for interactive sessions — separate from CT101 daemon cognition path ([OPERATOR_FRONTEND_DECISION.md](../../OPERATOR_FRONTEND_DECISION.md)).

---

## Interfaces

| Interface | Value |
|-----------|-------|
| CT101 consumer | Outbound `http://192.168.31.184:8000/v1/chat/completions` |
| Workstation bind | `127.0.0.1:8000` (Prime server local) |
| LAN IP | `192.168.31.184` |
| Model (live) | ornith-35b (probe 2026-07-14) |
| Project path | `~/Projects/llama.cpp/prime-bench/` (typical) |
| Config keys | `[engine.local]`, `active_mode`, `cloud_first_background` |

---

## THINKING nodes

> **THINKING — prime:fallback dependency**
> - *Reviewed:* CT101 cognition depends on workstation uptime for fallback path.
> - *Insight:* Acceptable for resilience; not for primary 24/7 if cloud down extended.
> - *Risk / limitation:* Workstation sleep kills fallback — dream/spark may skip.
> - *Enhancement:* Secondary fallback model on VM200 or LXC102 MCP hub. [GZMO-next]

> **THINKING — prime:split bind addresses**
> - *Reviewed:* Observatory uses loopback; CT101 uses LAN IP for same physical host.
> - *Insight:* Correct — CT101 cannot reach workstation `127.0.0.1`.
> - *Risk / limitation:* Firewall changes on workstation break CT101 without local symptom on Observatory.
> - *Enhancement:* Dual health: Observatory LAN probe to `192.168.31.184:8000`. [CT101-safe]

> **THINKING — prime:model drift**
> - *Reviewed:* Live model name (ornith-35b) can change with operator restore scripts.
> - *Insight:* `gzmo.toml` model string must match served model or requests fail opaquely.
> - *Risk / limitation:* No automatic model discovery on CT101 beyond `/models` ping.
> - *Enhancement:* Log resolved model ID in daemon boot banner. [CT101-safe]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| Workstation Prime fallback | GZMO-next may run local engine on same host as scheduler |
| Shared ornith-35b | Lab LLM URL via `LLM_URL` env in recipes |
| Manual restore scripts | Declarative model profile in `gzmo-next.toml` |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Observatory LAN health check for Prime | [CT101-safe] |
| 2 | Alert when cloud+fallback both unreachable | [CT101-safe] |
| 3 | Document model restore procedure in ops runbook | [CT101-safe] |
| 4 | Colocate inference for GZMO-next on workstation GPUs | [GZMO-next] |
| 5 | Automatic model ID sync from `/v1/models` into config | [GZMO-next] |
