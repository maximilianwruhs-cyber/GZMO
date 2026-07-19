# MACHINE — GZMO identity (canonical)

**Version:** 2026-06-04  
**Scope:** What GZMO *is*. Infrastructure and milestones live in `docs/`.

---

## Two sentences

1. **Honeypot + verify + promote = GZMO.**
2. **GZMO = Destillations-Pipeline** — not a chatbot with a memory attachment.

---

## Pipeline (one line)

```text
Any input → prep → extract (:8000) → verify → promote → vault → qualify → honeypot → [ripen → core]
```

- **Vault** — everything that passed verify (ops history, purge, decay).
- **Honeypot** — curated distillate; default for recall, Qdrant, Dream/Spark.
- **Core (M5)** — exportable dense knowledge; ripen pipeline at `scripts/ripen-knowledge-core.py` (charter gates apply; v0 preview with relaxed flags).

The LLM **thinks** (extract, verify, dream). The pipeline **remembers**.

---

## What GZMO is not

- Not a Telegram/OpenClaw product track (historical only).
- Not “ingest all Takeout now.”
- Not Mem0/Zep/Supermemory reimplemented — those are patterns to borrow later.
- Not a Foundry/Gotham (SEIP) platform — that research stays outside this repo.
- For outsiders, the downloadable product is **local memory MCP** (`gzmo init` + `gzmo mcp-serve`), not the private living stack. See [docs/PRODUCT_MCP.md](docs/PRODUCT_MCP.md).

---

## Ops (daily)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/verify-production.sh    # after reboot or infra change
./scripts/memory-status.sh        # vault / honeypot / qdrant counts
```

Roadmap to **local production-ready** (M3–M4): [`docs/ROADMAP_TO_M5.md`](docs/ROADMAP_TO_M5.md).

Operator model (one frontend, GZMO Platform spine): [`docs/ARCHITECTURE_GZMO_PLATFORM.md`](docs/ARCHITECTURE_GZMO_PLATFORM.md).  
**Pi living stack (recovered):** [`docs/PI_LIVING_STACK.md`](docs/PI_LIVING_STACK.md) · upgrade: [`docs/PI_UPGRADE_RUNBOOK.md`](docs/PI_UPGRADE_RUNBOOK.md) · Headroom/CCR: [`docs/HEADROOM_CCR.md`](docs/HEADROOM_CCR.md).
