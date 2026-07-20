# Research brief: `token-economy` (reactive token budget / Co-Saving router)

**Date:** 2026-07-20  
**Status:** Advisory primary-source inventory (no implementation)  
**Authority for idea pick:** [research/ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md) § Executive pick #3  
**Vault dig:** CT101 `/opt/gzmo/data/vault.db` via SSH `root@192.168.31.202` (jump `pve`)  
**Boundary:** Cite CT101 evidence and local configs; do **not** graft CT101 cloud routing wholesale ([ADR-0001](../../little-tools-lab/docs/adr/0001-two-stack-lab-not-ct101-graft.md); archaeology non-goal).

---

## 1. Problem statement

GZMO already meters and routes cognition, but **token spend is mostly static caps + post-hoc ledgers**, not a reactive economy that budgets, routes, compresses, and caches *before* the call. CT101 curated facts from the Agentic Token Economy blueprint name that gap:

| Fact id (prefix) | Decay | Source file | Content (verbatim from `semantic_vault`) |
|------------------|-------|-------------|------------------------------------------|
| `06c23921` | CuratedVault | `drive-research-agentic-token-economy-blueprint-micro01.md` | `[CONCEPT:Reactive Token Economy] A paradigm that requires the system to actively budget, route, compress, and cache its context usage dynamically.` |
| `b3be19fc` | CuratedVault | same micro01 | `[CONCEPT:Reactive Token Economy] Internalizes economic constraints directly into the agent's reasoning loop and operational infrastructure.` |
| `c605279c` | CuratedVault | same micro01 | `[CONCEPT:Reactive Token Economy] Enables intelligent systems capable of approaching the Pareto frontier of cost and quality.` |
| `5d774056` | CuratedVault | `drive-research-agentic-token-economy-blueprint-micro02.md` | `[CONCEPT:Co-Saving] Reduces multi-agent token usage by an average of 50.85% compared to baseline MAS architectures.` |
| `9c827e76` | CuratedVault | same micro02 | `[CONCEPT:TALE] Stands for Token-Budget-Aware LLM Reasoning.` |

**Supporting vault facts (same blueprint cluster, not in the five-id list):**

- `87031351` — TALE reduces CoT token usage by ~68.64% with negligible accuracy drop.  
- `2cdff379` / `79804d8c` — TALE adjusts reasoning tokens by problem complexity.  
- `84bf12c5` / `9568646d` / `48a3f1a8` — Co-Saving = directed-graph MAS + routing shortcuts.  
- `114c5c65` / `6c7da917` — Three-Layer Budget Enforcement: Token Buckets, Circuit Breakers, Fallback Chains.

**Product framing (archaeology):** *Obolus answers “smart per watt”; `token-economy` answers “smart per token/context”* — shape = little-tool CLI → `budget.json` + optional `[routing]` / context-pack hints; V/F/N = 8/5/8.  
Source: [research/ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md) lines 59–69.

**Lived gap in this repo:** engine/`routing.profiles.*` `max_tokens`, spark bounded caps, scratch injection budget, and Obolus ledgers exist — but there is **no** estimator that picks caps/profile/compress hints from task class + message size + profile before `GatewayRouter` fires. Dreams/KG extract inherit the profile’s full `max_tokens` via unbounded `complete_structured` (see §3).

---

## 2. Primary-source inventory — routing profiles, `max_tokens`, `scratch_max_tokens`

### 2.1 Workstation operator config — `/home/gzmo/github-clone/GZMO/gzmo.toml`

| Section | Key values | Notes |
|---------|------------|-------|
| `[engine.local]` | `max_tokens = 24576` | Comment: 24k guardrail vs full 128k ctx ([gzmo.toml](../gzmo.toml) L174–178; also [docs/CORE_STACK_KNOWLEDGE.md](../docs/CORE_STACK_KNOWLEDGE.md) `[CONFIG:engine.local]`) |
| `[engine.sovereign]` | `max_tokens = 8192` | L189 |
| `[engine.cloud]` | `max_tokens = 8192` | L202 |
| `[spark]` | `max_tokens_hypothesis = 4096`, `max_tokens_verify = 4096` | L71–73 |
| `[context_memory]` | `archive_threshold = 0.90`, `response_reserve = 0.10`, `scratch_max_tokens = 2000`, `context_length = 131072` | L263–267 |
| `[subagent]` | `context_budget_tokens = 32768`, `summary_max_tokens = 800` | L269–274 |
| `[routing]` | `default_engine = "local"` | L390–391 |
| `[routing.mappings]` | dream/spark → `local`; ingest_extract → `local_deterministic`; distill_extract/summary → `librarian`; distill_verify → `local` | L393–406 |
| `[routing.profiles.local_deterministic]` | temp `0.1`, `max_tokens = 24576`, Prime `:8000` | L408–415 |

No `[routing.profiles.librarian]` inline block in this file — `librarian` resolves via `[librarian]` / engine sections (`RoutingConfig::resolve_profile` in [gzmo-core/src/config.rs](../gzmo-core/src/config.rs) L2482–2526).

### 2.2 GZMO-next — `/home/gzmo/github-clone/GZMO/config/gzmo-next.toml`

| Section | Key values |
|---------|------------|
| `[spark]` | `max_tokens_hypothesis = 4096`, `max_tokens_verify = 4096` (L84–85) |
| `[engine.local]` | `max_tokens = 24576` (L184); temp `0.70` |
| `[context_memory]` | `scratch_max_tokens = 2000`, `context_length = 131072` (L238–242) |
| `[routing]` | `default_engine = "local"`; **all** dream/spark/ingest/distill mappings → `"local"` (L245–257) |
| Inline profiles | **None** — no `[routing.profiles.*]` |

Comment at L164–165: RAPL routing omitted (`fits_budget=false`); see fused sibling.

### 2.3 Fused snapshot — `/home/gzmo/github-clone/GZMO/config/gzmo-fused.toml`

Generated by `config-fuse v0.3` (2026-07-19). **Not** a full routing table:

| Key | Value |
|-----|-------|
| `[engine] max_tokens` | `1536` (flat fused engine — **not** 24576) |
| `temperature` | `0.9377` |
| `[routing.rapl]` | `model = "qwen2.5-7b-q4"`, `task = "coding"`, `rapl_watts = 45`, `joules_per_task = 120.5`, `fits_budget = true` |

Interpretation: fused file is a **bench→fuse energy/token calibration fragment**, not the living cognition routing authority. Next.toml explicitly kept RAPL out when `fits_budget=false` for the 35B path.

### 2.4 CT101 live authority — `/opt/gzmo/gzmo.toml` (SSH 2026-07-20)

| Area | Values |
|------|--------|
| `[engine.local]` | `max_tokens = 24576` (Prime via `.184:8000`, ornith-35b GGUF) |
| `[engine.cloud]` | OpenRouter `z-ai/glm-5.2`, `max_tokens = 8192`, `reasoning_effort = "medium"` |
| `[spark]` | hypothesis/verify `4096` / `4096` |
| `[dreams]` TOML keys | Comment + keys `max_tokens_extract = 16384`, `max_tokens_verify = 8192` — **present in TOML** |
| Code binding | Workstation & CT101 `survey_GZMO` `DreamsConfig` **lack** `max_tokens_extract` fields → serde **silently ignores** them (no `deny_unknown_fields` on `GzmoConfig`) |
| `[context_memory]` | `scratch_max_tokens = 2000`, `context_length = 262144` |
| `[context_compress]` | `tool_output_max_tokens = 4000`, `recall_compress_budget = 2000` |
| `[routing]` | `default_engine = "cloud"`, `cloud_first_background = true` |
| Mappings | All dream/spark/ingest/distill → `cloud_ingest_extract` / `cloud_ingest_verify` |
| Profiles | `cloud_ingest_extract` DeepSeek flash, `max_tokens = 24576`; `cloud_ingest_verify` `8192`; `local_deterministic` Prime 24576 |

Doc mirror of TaskKind → profile steady-state (workstation-oriented): [docs/OBOLUS_ROUTING.md](../docs/OBOLUS_ROUTING.md). CT101 live table **diverges** (cloud extract/verify profiles) — see [docs/ct101-systems/40-llm-gateway/engine-profiles.md](../docs/ct101-systems/40-llm-gateway/engine-profiles.md) § CT101 live profile.

### 2.5 Code defaults (`gzmo-core/src/config.rs`)

| Default | Value | Location |
|---------|-------|----------|
| Engine `max_tokens` | `8192` (`default_max_tokens`) | ~L2812 |
| Spark hypothesis / verify | `2048` / `1024` | L1598–1602 |
| Scratch | `2000` | L2061–2062 |
| Pedagogy `internal_max_tokens` | `512` | L785–786 |
| Subagent `summary_max_tokens` | `800` (doc) | [docs/ct101-systems/90-tools-skills/subagent-delegate.md](../docs/ct101-systems/90-tools-skills/subagent-delegate.md) |

Operator TOMLs override spark to 4096/4096 — defaults are the floor when keys omitted.

---

## 3. Call sites for token caps

### 3.1 Dreams / KG pipeline

| Site | Behavior | Citation |
|------|----------|----------|
| `DreamEngine::with_gateways` | Separate extract vs verify `LlmGateway` (Obolus routing) | [gzmo-core/src/dreams.rs](../gzmo-core/src/dreams.rs) L74–88 |
| `KgPromoter::extract` | `complete_structured` — **no** `max_tokens` override → profile/`effective_max_tokens` | [gzmo-core/src/memory/kg_extract.rs](../gzmo-core/src/memory/kg_extract.rs) L595–598 |
| `KgPromoter::verify` | `complete_structured_with_temp` on `verify_gateway` — temp only, no bounded tokens | same file L657–665 |
| CT101 TOML dream caps | Written but **not wired** into `DreamsConfig` | §2.4 |

### 3.2 Spark

| Site | Cap | Citation |
|------|-----|----------|
| Hypothesis | `complete_structured_bounded(..., Some(max_tokens_hypothesis))` then `.min(config.max_tokens)` inside gateway | [gzmo-core/src/spark.rs](../gzmo-core/src/spark.rs) L406–414; [gateway.rs](../gzmo-core/src/gateway.rs) L794–796 |
| Verify | `Some(max_tokens_verify)` | spark.rs ~L471–476 |

### 3.3 Gateway / chaos / chat / TUI

| Site | Behavior | Citation |
|------|----------|----------|
| `TurboQuantGateway::effective_max_tokens` | Chaos override vs config; chaos uses `max(chaos, config/2)` | [gateway.rs](../gzmo-core/src/gateway.rs) L383–393 |
| `set_chaos_overrides` | PulseLoop → REPL/skills | gateway.rs L354–363; skills `*_rs` set overrides from snapshot |
| Chaos Lorenz map | `llm_max_tokens ∈ [128, 512]` from Lorenz y | [gzmo-chaos/src/pulse.rs](../gzmo-chaos/src/pulse.rs) L651–654 |
| Chat `/chaos` UI | Prints `snap.llm_max_tokens` | [gzmo-cli/src/chat.rs](../gzmo-cli/src/chat.rs) L1109–1111 |
| TUI runner | `context_budget = active_profile.max_tokens * 4` (char-ish budget for agent context) | [gzmo-cli/src/tui/runner.rs](../gzmo-cli/src/tui/runner.rs) L227 |
| Pedagogy | `internal_max_tokens` (default 512) on agent_call | [pedagogy/orchestrator.rs](../gzmo-core/src/pedagogy/orchestrator.rs) ~L309 |

### 3.4 Context / scratch (input-side budgets)

| Site | Behavior | Citation |
|------|----------|----------|
| Hot window prune | `estimate_*_tokens` + `archive_threshold` of hot budget | [context.rs](../gzmo-core/src/context.rs) L73–124 |
| Scratch inject | Trim recall to `scratch_max_tokens` | [memory/scratch.rs](../gzmo-core/src/memory/scratch.rs) L377+; config L2046–2048 |
| Shell skills helper | `GZMO_LLM_MAX_TOKENS` default 512 | [skills/_llm_helper.sh](../skills/_llm_helper.sh) L19 |

---

## 4. Contrast with Obolus / IpW (per-watt vs per-token)

**Do not re-ship Obolus.** Sibling product + daemon gate already cover energy/token *accounting* and autonomy deny/warn.

| Axis | Obolus / IpW (shipped / spiked) | Proposed `token-economy` |
|------|----------------------------------|---------------------------|
| Primary currency | Joules / watts / Arena `z`; token `E_total` as **proxy** for gates | Prompt+completion tokens, context pack size, reasoning budget (TALE) |
| Decision time | Preflight deny/warn (governance); IpW writes **advice** JSON | Pre-call **estimate → recommend caps / compress / profile snippet** |
| Routing | Static `TaskKind → profile` ([OBOLUS_ROUTING.md](../docs/OBOLUS_ROUTING.md)); IpW `task_class → route` by z/watts ([ipw-router.policy.toml](../config/ipw-router.policy.toml)) | Co-Saving-style **graph shortcuts** + budget-aware profile hints (advisory TOML) |
| Enforcement | T0 warn / T1 defer / T2 deny ([OBOLUS_GOVERNANCE.md](../docs/OBOLUS_GOVERNANCE.md)) | Advisory CLI only in v0 (emit files; operator or fuse applies) |
| Metabolism | IpW must **never** block distill/dream ([OBOLUS_ARENA_BOUNDARY.md](../docs/OBOLUS_ARENA_BOUNDARY.md); `ipw-route.sh` note L129) | Same non-goal: never hard-block overnight jobs |
| Ledger | `data/Obolus/ledger.jsonl`, η roadmap ([OBOLUS_EFFICIENCY.md](../docs/OBOLUS_EFFICIENCY.md)); RAPL observability ([OBOLUS_ENERGY.md](../docs/OBOLUS_ENERGY.md)) | Consumes estimates; may *read* ledger later — does not replace it |

**IpW primary scripts (cite, do not fold into living gate):**

- [scripts/ipw-route.sh](../scripts/ipw-route.sh) — schema `gzmo.ipw.route/v1` → `data-next/ipw-router/latest.json`  
- [config/ipw-router.policy.toml](../config/ipw-router.policy.toml) — `z_floor`, `watts_ceiling`, routes by task class  
- [scripts/ipw-route-check.sh](../scripts/ipw-route-check.sh) — asserts not wired into `living-readiness-gate.sh`

**Archaeology one-liner:** Obolus = smart per watt; token-economy = smart per token/context ([ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md) L66).

---

## 5. Proposed CLI + `budget.json` / TOML snippet schemas (advisory only)

### 5.1 CLI sketch

```text
token-economy estimate  --task <class> --messages <path|stdin> [--profile <name>] [--config gzmo.toml]
token-economy recommend --from budget.json [--emit-toml routing.snippet.toml]
token-economy diff      --budget budget.json --against gzmo.toml   # show delta vs live caps
token-economy smoke     --fixtures fixtures/token-economy/
```

Exit codes (proposed): `0` ok, `1` estimate/schema error, `2` over hard ceiling (advisory warn only unless `--strict`).

### 5.2 `budget.json` (proposed schema `gzmo.token_economy.budget/v1`)

```json
{
  "schema": "gzmo.token_economy.budget/v1",
  "generated_at": "2026-07-20T09:00:00Z",
  "inputs": {
    "task_class": "spark_hypothesis",
    "message_chars": 4200,
    "estimated_input_tokens": 1200,
    "profile": "local",
    "context_length": 131072
  },
  "estimator": {
    "method": "chars_per_token",
    "chars_per_token": 3.5,
    "tale_complexity": "low|medium|high",
    "recommended_reasoning_tokens": 256,
    "recommended_max_tokens": 2048
  },
  "co_saving": {
    "shortcut_eligible": false,
    "skip_stages": [],
    "compress_hints": ["trim_tool_output", "scratch_cap"]
  },
  "enforcement_layers": {
    "token_bucket": { "call_budget": 4096, "session_budget": 50000 },
    "circuit_breaker": { "trip_on_pct_of_ctx": 0.9 },
    "fallback_chain": ["local_deterministic", "local", "defer"]
  },
  "obolus_contrast": {
    "does_not_set": ["watts_ceiling", "arena_z", "blocks_distill"],
    "may_read": ["ledger E_total"]
  },
  "note": "Advisory only — does not rewrite engine URLs or block metabolism."
}
```

Rationale ties: TALE complexity → reasoning budget (`9c827e76`, `2cdff379`); Co-Saving shortcuts (`5d774056`, `84bf12c5`); three-layer enforcement (`114c5c65`); IpW sibling note pattern (`ipw-route.sh` L129).

### 5.3 Optional emitted TOML snippet

```toml
# Generated by token-economy — review before merge; do not auto-apply on CT101.
[routing.profiles.token_economy_suggested]
# provider/url/model omitted on purpose — caps only
max_tokens = 2048
temperature = 0.2

[spark]
# only if task_class was spark_*
max_tokens_hypothesis = 2048
max_tokens_verify = 1024

[context_memory]
scratch_max_tokens = 1500
```

Merge policy: lab / `data-next` only; CT101 remains frozen ([ADR-0001](../../little-tools-lab/docs/adr/0001-two-stack-lab-not-ct101-graft.md)).

---

## 6. Estimator inputs

| Input | Source today | How estimator should use it |
|-------|--------------|-----------------------------|
| **Task class** | `TaskKind` snake_case (`dream_extract`, `spark_hypothesis`, `chat`, …) — [config.rs](../gzmo-core/src/config.rs) TaskKind; IpW also uses coarser `chat`/`overnight`/`heavy_bench` | Map to prior caps (spark 4k, dream→profile 24k, pedagogy 512, chaos 128–512) and Co-Saving shortcut eligibility (multi-stage pipelines only) |
| **Message size** | Raw char length of system+user (+ tool payloads); `context::estimate_text_tokens(content, chars_per_token)` ([context.rs](../gzmo-core/src/context.rs) L73+) | `estimated_input_tokens`; drive compress hints when approaching `context_length * (1 - response_reserve) * archive_threshold` |
| **Profile** | Resolved name from `[routing.mappings]` + inline profile `max_tokens` / temp | Soft ceiling: recommended_max ≤ profile.max_tokens; never invent CT101 cloud URLs |
| Optional: scratch pressure | Current scratch used vs `scratch_max_tokens` | Lower inject budget / force distill enqueue |
| Optional: Obolus window | Rolling `E_total` / ctx% ([OBOLUS_GOVERNANCE.md](../docs/OBOLUS_GOVERNANCE.md)) | Scale down recommendations; **do not** implement deny (Obolus owns gates) |
| Optional: TALE complexity class | Heuristic on query length / schema complexity / “verify vs extract” | Set `recommended_reasoning_tokens` (TALE) independently of completion JSON size |

---

## 7. Fixtures / smoke plan and explicit non-goals

### 7.1 Fixtures / smoke (proposed)

| Fixture | Assert |
|---------|--------|
| `fixtures/token-economy/small_chat.json` | Low tale_complexity → small `recommended_max_tokens` (≪ 24576) |
| `fixtures/token-economy/spark_pair.json` | Caps ≤ configured spark hypothesis/verify |
| `fixtures/token-economy/dense_dream_chunk.json` | May recommend high cap but ≤ profile max; emit compress hints |
| `fixtures/token-economy/over_ctx.json` | Message estimate > hot budget → circuit_breaker + compress_hints |
| Smoke script | Parse `budget.json` schema; `recommend` TOML validates under `toml` crate; **no** SSH to CT101; **no** rewrite of `/opt/gzmo/gzmo.toml` |
| Regression vs IpW | `blocks_distill` absent/false; no watts fields required |

Pattern borrow: [scripts/ipw-route-check.sh](../scripts/ipw-route-check.sh) PASS/HOLD/FAIL rows; keep out of `living-readiness-gate.sh`.

### 7.2 Explicit non-goals

1. **No grafting CT101 cloud routing wholesale** — do not copy `cloud_ingest_*` / `default_engine = "cloud"` / OpenRouter DeepSeek table into next or lab defaults (archaeology L69; ADR-0001; UNIQUENESS two-stack).  
2. **Do not re-ship Obolus** — no duplicate ledger, η CLI, RAPL sampler, or T0–T2 gate.  
3. **Do not make IpW / token-economy a living required path** — advice files under `data-next/` only.  
4. **Do not silently “fix” CT101** by wiring ignored `dreams.max_tokens_*` TOML keys without an intentional next/lab change.  
5. **Do not import CT101 vault** into `data-next` ([UNIQUENESS_THESIS](../docs/UNIQUENESS_THESIS.md) / archaeology boundary).  
6. **No Arena DNA / QLoRA fold** ([OBOLUS_ARENA_BOUNDARY.md](../docs/OBOLUS_ARENA_BOUNDARY.md)).

---

## Provenance

| Pass | Action |
|------|--------|
| 1 | Read archaeology pick + five fact ids |
| 2 | SSH CT101 `semantic_vault` SELECT for exact content |
| 3 | Diff `gzmo.toml`, `config/gzmo-next.toml`, `config/gzmo-fused.toml`, `/opt/gzmo/gzmo.toml` |
| 4 | Trace dreams/spark/gateway/context/TUI/chat call sites |
| 5 | Contrast OBOLUS_* docs + IpW scripts/policy |
| 6 | Draft advisory schemas + smoke/non-goals |

**Report path:** `GZMO/research/token-economy-primary-sources-2026-07-20.md`
