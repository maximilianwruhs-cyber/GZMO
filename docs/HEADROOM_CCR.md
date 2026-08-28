# Headroom / CCR — recovered from branch (not on living HEAD)

**Status:** Archival recovery (2026-07-19)  
**Upstream inspiration:** [headroomlabs-ai/headroom](https://github.com/headroomlabs-ai/headroom) (Apache-2.0) — not cloned on this workstation  
**Implementation branch:** `origin/feat/context-compress-headroom`  
**Living `main`:** `gzmo-core/src/context_compress/` **absent** — do not expect `[ccr:…]` or `gzmo_retrieve_context` on CT101 until re-landed

---

## What it was

Headroom’s idea (context compression for long agent traces) was ported **into GZMO-core**, not run as a permanent external `headroom proxy`.

| Component | Path on branch | Role |
|-----------|----------------|------|
| NOTICE | `gzmo-core/src/context_compress/NOTICE` | Apache-2.0 attribution to Headroom |
| Router | `…/mod.rs` | `detect_route` → logs / JSON / plain |
| Logs | `…/logs.rs` | Strip ANSI, dedup, cap lines |
| JSON | `…/json.rs` | Structured shrink |
| CCR | `…/ccr.rs` | Redis store `gzmo:ccr:` + TTL, fail-open |
| Bench | `scripts/compression-bench/benchmark_headroom.py` | Compare vs Python Headroom |
| Handoff | `docs/CONTEXT_COMPRESS_PHASE3_HANDOFF.md` | Phase 3 (log routing + scored prune) never started |

Key commits: `a373d33` (Phase 0–1) → `df078f3` (Phase 2 CCR + MCP hot-path) → `c909277` (chore).

---

## CCR flow (how Pi felt “enhanced”)

1. Large tool / MCP / scratch inject exceeds budget  
2. Full text stored in Redis: `gzmo:ccr:{session_id}:{hash}`  
3. Context sees placeholder: `[ccr:<hash> — gzmo_retrieve_context to expand]`  
4. Operator/MCP calls `gzmo_retrieve_context` with that hash to expand  

Documented on the branch in `PI_OPERATOR_GUIDE.md` §4.1a.

Config (branch-era): `[context_compress] enabled = true` plus Redis enabled on CT101.

---

## Non-goals (still valid)

- Do **not** compress `prune.archived` / distill transcripts meant for metabolism  
- Do **not** compress vault / honeypot promotion payloads  
- Do **not** require a long-lived `headroom proxy` process in production  
- Do **not** replace Qdrant / RRF recall  

---

## How to inspect / re-land

```bash
# List files
git ls-tree -r --name-only origin/feat/context-compress-headroom | rg context_compress

# Read NOTICE / CCR
git show origin/feat/context-compress-headroom:gzmo-core/src/context_compress/NOTICE
git show origin/feat/context-compress-headroom:gzmo-core/src/context_compress/ccr.rs | head

# Full Phase 3 plan
git show origin/feat/context-compress-headroom:docs/CONTEXT_COMPRESS_PHASE3_HANDOFF.md
```

Re-land checklist (when you choose to):

1. Cherry-pick or replay `a373d33..c909277` onto a fresh branch from living `main`  
2. Resolve conflicts with current `scratch.rs` / MCP serve / config  
3. Enable only behind `[context_compress]` default **off** until living gate stays GREEN  
4. Add MCP tool smoke for `gzmo_retrieve_context` to `living-readiness-gate` as HOLD/optional  
5. Update [PI_LIVING_STACK.md](./ops/PI_LIVING_STACK.md) “LOST on living HEAD” section  

---

## Relation to Pi subagents + Redis scratch

CCR is **orthogonal** to `ScratchScope::Sub`:

- Scratch = hot recall snippets for the current turn/delegation  
- CCR = overflow store for compressed *content bodies* with retrieve-on-demand  

Both use the **same Redis sidecar** on CT101 with different key prefixes (`gzmo:scratch:` vs `gzmo:ccr:`).
