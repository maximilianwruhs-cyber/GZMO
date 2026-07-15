# VM200 — Remote Embed + Rerank Layer

**Source:** `GZMO/scripts/vm200/`  
**Parent:** [110-external-nodes/SYSTEM.md](./SYSTEM.md)

---

## Capability

Offloads vector **embedding** and **reranking** from CT101's 8 GiB LXC to VM200's GTX 1070 eGPU. Single consolidated `llama-server` on **port 8081** serves both `/v1/embeddings` and `/v1/rerank` — replacing obsolete separate `:8082` rerank service.

---

## How it works

### Deploy script

```1:6:github-clone/GZMO/scripts/vm200/deploy-retrieval-layer.sh
#!/usr/bin/env bash
# Deploy consolidated embed+rerank on VM200 :8081 (single llama-server, both models).
# The server loads Qwen3-Reranker-0.6B.F16.gguf as primary model and also
# registers gzmo-embed as an alias so both /v1/embeddings and /v1/rerank work.
# Rerank on :8082 is OBSOLETE — consolidated to :8081.
```

Flow: rsync GGUF models → install `llama-embed.service` → disable legacy `llama-rerank` → smoke test embed + rerank.

### Systemd unit

```10:24:github-clone/GZMO/scripts/vm200/llama-embed.service
ExecStart=/usr/local/bin/llama-server \
  -m /opt/models/Qwen3-Reranker-0.6B.F16.gguf \
  --embedding \
  --pooling rank \
  --reranking \
  --alias gzmo-rerank \
  --embeddings \
  --pooling last \
  --alias gzmo-embed \
  -ngl 99 \
  -c 2048 \
  --parallel 1 \
  --port 8081 \
  --host 0.0.0.0
```

### CT101 config wiring

From deploy script output (live-verified):

```
[embeddings] url = http://192.168.31.110:8081/v1  model = gzmo-embed
[rerank]     url = http://192.168.31.110:8081/v1  model = gzmo-rerank
```

Consumed by `gzmo-core/src/memory/embeddings.rs` and `rerank.rs` during vault open and Qdrant sync.

---

## Interfaces

| Interface | Value |
|-----------|-------|
| Host | `192.168.31.110` (VM200) |
| SSH | `maximilian@192.168.31.110` via `GZMO_VM200_SSH_KEY` |
| Embed endpoint | `POST http://192.168.31.110:8081/v1/embeddings` |
| Rerank endpoint | `POST http://192.168.31.110:8081/v1/rerank` |
| Models API | `GET http://192.168.31.110:8081/v1/models` |
| Optional librarian | `:8083` — disabled on CT101 (`[librarian]` off) |
| Deploy scripts | `deploy-retrieval-layer.sh`, `deploy-rerank.sh` (legacy), `deploy-librarian.sh` |

---

## THINKING nodes

> **THINKING — vm200:consolidated server**
> - *Reviewed:* One llama-server process hosts both embed and rerank aliases.
> - *Insight:* Halves GPU memory fragmentation vs dual-service layout on 8 GB VRAM class GPU.
> - *Risk / limitation:* Single point of failure — both recall paths down if service crashes.
> - *Enhancement:* systemd watchdog + CT101 daemon degrade to keyword-only recall. [CT101-safe]

> **THINKING — vm200:LAN exposure**
> - *Reviewed:* `--host 0.0.0.0` binds all interfaces on homelab LAN.
> - *Insight:* Simple wiring for CT101 outbound HTTP without reverse proxy.
> - *Risk / limitation:* No auth on OpenAI-compatible API — LAN trust model only.
> - *Enhancement:* Token auth or bind to `192.168.31.110` only + firewall. [GZMO-next]

> **THINKING — vm200:model copy via rsync**
> - *Reviewed:* Deploy pulls GGUF from workstation HuggingFace cache paths.
> - *Insight:* Operator workstation is staging area for model artifacts.
> - *Risk / limitation:* Large model sync over LAN can timeout; no checksum verify in script.
> - *Enhancement:* Content-hash skip in rsync + version pin in `gzmo.toml`. [CT101-safe]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| Remote VM200 HTTP | Could colocate on workstation 5070 Ti for GZMO-next |
| Qwen3 0.6B embed/rerank | Lab embedding piece with beat-gate latency baseline |
| Manual deploy script | Ansible/terraform for VM200 retrieval stack |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Daemon heartbeat probe for `:8081/v1/models` | [CT101-safe] |
| 2 | Graceful recall degradation when VM200 down | [CT101-safe] |
| 3 | API auth on llama-server | [GZMO-next] |
| 4 | Automated deploy from CI on model update | [GZMO-next] |
| 5 | VRAM usage alert via Observatory GPU panel | [CT101-safe] |
