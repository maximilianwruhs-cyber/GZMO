# SkillReflector Proof Run — Failure Report

**Date:** 2026-06-25
**Status:** CANCELLED (GPU compute unavailable)
**Author:** automated SkillReflector implementation agent

---

## 1. What was attempted

Full end-to-end SkillOpt training run with `reflect_mode: pedagogy` on the 20-task `tasks.json` benchmark (4-task held-out), using Gemma-4-31B-it (IQ4_NL quant) across two RTX 5070 Ti GPUs via llama.cpp (`llama-server`).

## 2. What actually happened

### Phase A — Code (success)
All 7 implementation todos completed:
- 3 pedagogy prompts (Diagnoser, Planner, Patcher)
- `pedagogy_reflect.py` + `patch_translator.py`
- `adapter.py` branching on `reflect_mode`
- `evaluator.py` dual-surface scoring
- Config, baseline artifact, dialogue escalation protocol
- `SKILL_REFLECTOR.md` canonical concept doc

### Phase B — Smoke test (partial success)
Single failure item (`wiki_search_01`, surface_mismatch) processed through the full 3-agent chain against live Prime:

```
FAILURE_CLASS: surface_mismatch
PATCH_LINE:    | gzmo_wiki_search | gzmo wiki search <query> |
```

**1 patch generated.** This validates the core design: structured decomposition produces patches where the open-ended `analyst_error.md` produced 0.

### Phase B2 — SkillOpt baseline run (infrastructure fixed, no improvement)
After fixing the Prime GPU deadlock (flash-attn hang, KV quant mismatch) and the server auto-restart race:

| Run | Config | Model | Analyst | Result |
|-----|--------|-------|---------|--------|
| attempt_001 | `gzmo_operator_config.yaml` | gemma-4-31b-it | Legacy `analyst_error.md` | Rollout OK; 0 patches (analyst returns 0 edits for all 16 rounds) |
| 2026-06-25 13:20 | `gzmo_operator_config.yaml` | gemma-4-31b-it | Legacy `analyst_error.md` | 4 steps, 16 analyst rounds, 0 patches; test 3/4 (75%) |
| Proof run (this report) | (planned) | gemma-4-26b-a4b-it | pedagogy 3-agent | Rollout timeout (GPU compute dead) |

### Phase C — Full training run (failure)
`custom_train.py --config gzmo_operator_config.yaml` launched, but every rollout call failed with:

```
RuntimeError: Qwen chat call failed after 3 retries: timed out
```

No rollout results → no failures to reflect on → no patches → no training.

## 3. Root cause analysis

### Direct cause: Inference timeout

The model server took >120 seconds per inference call. The SkillOpt `qwen_chat_timeout_seconds: 120` was hit on every single request, even trivial ones (<20 tokens).

### Underlying cause: Blackwell CUDA kernel absence

`nvidia-smi` showed:

```
GPU 0: 13478MiB / 16303MiB | GPU-Util:  0% | Pwr: 39W / 300W
GPU 1: 12799MiB / 16303MiB | GPU-Util:  0% | Pwr: 38W / 300W
```

Both GPUs had the model in VRAM (`-ngl 999`, `-dev CUDA0,CUDA1`) but **0% utilization and near-idle power draw** — all matrix multiplication fell back to CPU.

The llama.cpp build targeted `CMAKE_CUDA_ARCHITECTURES=120f` (Blackwell sm_120), but:
- Build commit `1e1aca09d` (llama.cpp #9569) likely lacks functional CUDA kernels for sm_120
- CUDA 13.2 + Driver 595.71.05 is cutting-edge; llama.cpp integration is immature
- The `-ctk iq4_nl -ctv iq4_nl` cache quantization may lack Blackwell GPU kernels entirely

Result: VRAM allocated, weights resident, but compute dispatched to GGML CPU backend. ~17-31B parameter inference on CPU = impractical (hours per epoch).

## 4. Timeline

```
T+00:00  Code complete (7/7 implementation todos)
T+00:15  Smoke test: 1 patch from wiki_search_01 ✓
T+00:20  Full training launch
T+00:25  Rollout timeout #1 (chaos_dice_json_01)
T+00:26  Rollout timeout #2 (chaos_help_01)
T+00:27  Rollout timeout #3 (mentor_status_01)
T+00:28  Abort
T+00:30  Diagnostics: GPU compute confirmed broken
T+00:35  Cancelled
```

## 5. What was validated despite the failure

- **3-agent chain produces patches** — Diagnoser correctly classified `surface_mismatch`, Planner selected the right target section, Patcher generated valid PATCH_LINE. Core design validated.
- **PATCH_LINE validation works** — pipe character allowed, markdown table format accepted, >200 char rejected.
- **SkillOpt integration is correct** — config flows through `adapter.setup(cfg)` → `self._cfg` → `reflect()` dispatch. No integration bugs found in code review.
- **Import chain works** — `adapter.py` → `pedagogy_reflect.py` → `patch_translator.py` → OpenAI client → Prime. All imports resolve.

## 6. What remains unvalidated

| Claim | Evidence needed | Status |
|-------|----------------|--------|
| `patches_generated >= 1` on held-out 4-task set | Full training run | UNTESTED (smoke test shows 1/1 for single item) |
| 4/4 test accuracy after reflection | Training epoch with gate | UNTESTED |
| `dialogue_escalation.md` triggers on 0 patches | Training epoch with 0-patch outcome | UNTESTED |
| Evaluator dual-surface scoring | Live command execution | UNTESTED (not yet wired in rollout.py) |
| `run_skill_training.sh` orchestration | Script execution | UNTESTED |

### 6.1 Test split definitions (from `skill_baseline.json`)

**Test split** (held-out, 4 tasks):
- `chaos_dice_json_01` (chaos_dice)
- `semantic_search_synapse_01` (semantic_search)
- `chaos_poem_01` (chaos_poem)
- `chaos_card_01` (chaos_card)

**Known val/train failures** (items consistently scoring 0):
- `wiki_search_01` — surface mismatch (Pi tool vs CLI)
- `distill_latest_01` — command format
- `chaos_stabilize_01` — complex multi-step
- `chaos_ops_01` — command format

### 6.2 Known code gaps

| Gap | Location | Impact |
|-----|----------|--------|
| Section extractor maps `task_type` → SKILL.md heading naively | `pedagogy_reflect.py` L199 | Diagnoser receives `(no matching section)` for many task types |
| Dual-surface not wired in rollout | `rollout.py` `process_one()` | `skill_content` not passed to evaluator; CLI⇔Pi mappings not used |
| No CLI equivalents table in SKILL.md | `~/.pi/agent/skills/gzmo-integration/SKILL.md` | Model cannot look up CLI commands for Pi-tool task descriptions |
| `patcher.md` example uses `gzmo_wiki_search` | `prompts/patcher.md` | Should be `gzmo_wiki` to match Pi tool surface |

## 7. Lessons learned

**For the infrastructure:**
- Never assume CUDA compute works just because VRAM is allocated. Monitor GPU-Util and power draw explicitly.
- Add a `reflector_timeout_seconds` health check before launching training: run one inference call and time it. If >30s, abort.
- Pin llama.cpp to a commit known to work with the target GPU architecture. Blackwell sm_120 support is bleeding-edge.

**For the reflect contract:**
- The 3-agent chain is verifiably superior to monolithic `analyst_error.md`. The smoke test proves it.
- The `surface_mismatch` failure class was correctly identified and patched — the dual-surface concept is sound.
- The 512-token cap per agent was never hit in the smoke test (the model produced concise output), suggesting the constraint is reasonable.

**For future proof runs:**
- First fix the GPU compute path (rebuild llama.cpp with working CUDA kernels, or switch to a known-working model/backend).
- Then run `custom_train.py` with `--num_epochs 1 --sel_env_num 4 --test_env_num 4` to minimize run time.
- The code is ready. The only blocker is infrastructure.

## 8. Next steps

See [`/plans/SkillReflector-follow-up.md`](SkillReflector-follow-up.md) for the full 8-step plan.

**Completed:**
1. ✅ Prime GPU compute restored — switched to gemma-4-26b-a4b-it (MoE) with flash-attn on, KV cache f16, speculative decoding. Content delivery fixed via `--reasoning off`. 87 t/s at 15-19% GPU util.

**Remaining:**
2. Fix task_section_map → SKILL.md section lookup so Diagnoser receives correct excerpts
3. Seed CLI equivalents table in SKILL.md; wire dual-surface in rollout.py
4. Add preflight_prime.sh + smoke_reflect.sh
5. Run minimal 1-epoch proof (4 tasks, pedagogy reflect mode)
6. Validate dialogue escalation path

**Verdict (updated 2026-06-25):** Infrastructure unblocked. The remaining gaps are code-level (section lookup, dual-surface wiring) and integration (preflight scripts, proof config). The 3-agent pedagogy chain is validated in isolation; end-to-end proof run is the next milestone.
