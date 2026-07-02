# Latent-Space vs Text-Based Multi-Agent Communication

Research harness for comparing GZMO's text-based pedagogy orchestrator against RecursiveMAS-style latent collaboration.

## What was implemented

| Component | Location | Purpose |
|-----------|----------|---------|
| Per-agent metrics | `gzmo-core/src/pedagogy/orchestrator_v2.rs` | `AgentCallMetrics`, `OrchestratorMetrics` on each Evaluator→Tutor run |
| Gateway latency + tokens | `gzmo-core/src/gateway.rs` | `take_last_latency_ms()`, structured `info!` logs per LLM call |
| MAS benchmark suite | `attractorbench/src/tests/suites.rs` | `multi_agent` suite (`mas-001` … `mas-005`) |
| MAS compare CLI | `attractorbench/src/mas.rs` | `attractorbench mas-compare` |
| RecursiveMAS bridge | `research/recursivemas_bridge.py` | FastAPI wrapper for Phase 2 A/B tests |
| Baseline script | `research/run_mas_baseline.sh` | Runs text MAS comparison |
| Report script | `research/compare_latent_text.py` | Summarizes JSON results |

## Phase 1 — Baseline (text MAS)

1. Start local inference (llama-server on `:1234` or Prime on `:8000`).
2. Run:

```bash
cd survey_GZMO/research
chmod +x run_mas_baseline.sh
MAS_ENDPOINT=http://localhost:1234/v1 MAS_MODEL=your-model.gguf ./run_mas_baseline.sh
```

3. Inspect `research/output/mas_text_baseline.json`.

GZMO pedagogy runs log orchestrator metrics at `target=gzmo::pedagogy::orchestrator_v2` when `RUST_LOG=info`.

## Phase 2 — Real RecursiveMAS (latent bridge)

### One-time setup

```bash
cd survey_GZMO/research
chmod +x setup_recursivemas.sh run_recursivemas_bridge.sh
./setup_recursivemas.sh
```

This clones [RecursiveMAS](https://github.com/RecursiveMAS/RecursiveMAS) to `~/Projects/RecursiveMAS` (override with `RECURSIVEMAS_ROOT`), creates `.venv-rmas` with Python 3.11 + CUDA PyTorch, and writes `.env.recursivemas`.

On first inference, Hugging Face checkpoints download automatically (~5–8 GB for `sequential_light`: Qwen3-1.7B planner, Llama3.2-1B critic, Qwen2.5-Math-1.5B solver + outer links). Requires NVIDIA GPU. **RTX 50-series (Blackwell)** needs PyTorch `cu128` wheels — `setup_recursivemas.sh` installs these automatically.

### Run the bridge

```bash
source research/.env.recursivemas
./research/run_recursivemas_bridge.sh
# health: curl http://127.0.0.1:8765/health
```

### Mock mode (no GPU)

```bash
RECURSIVEMAS_MOCK=1 python3 recursivemas_bridge.py --port 8765
```

Optional FastAPI stack: `pip install -r requirements-bridge.txt` then `python3 recursivemas_bridge.py --fastapi`.

### Compare with bridge

```bash
RECURSIVEMAS_URL=http://127.0.0.1:8765 ./run_mas_baseline.sh
python3 compare_latent_text.py output/mas_text_baseline.json --latent output/mas_latent_compare.json
```

## Phase 3 — Decision criteria

| Signal | Stick with text (GZMO) | Consider latent bridge |
|--------|------------------------|------------------------|
| Accuracy | Within 5% of bridge | Bridge >10% better on target tasks |
| Latency | 2-agent ≤ single×1.5 | Bridge faster than 2-agent text |
| Tokens | Handoff overhead acceptable | Bridge cuts intermediate tokens >30% |
| Ops | GGUF/sovereign priority | Willing to run PyTorch HF sidecar |

## AttractorBench examples

```bash
# Text handoff costs only
attractorbench mas-compare --modes single,two_agent,four_agent --suite multi_agent --runs 5 \
  --endpoint http://localhost:1234/v1 --model your-model

# Include latent bridge
attractorbench mas-compare --modes single,two_agent,recursive_mas --suite multi_agent \
  --recursive-mas-url http://127.0.0.1:8765 --runs 5
```

## Instrumentation fields

`OrchestratorOutputV2.metrics`:

```json
{
  "calls": [
    {"agent": "evaluator", "latency_ms": 820, "input_tokens": 400, "output_tokens": 120, "total_tokens": 520},
    {"agent": "tutor", "latency_ms": 1100, "input_tokens": 600, "output_tokens": 200, "total_tokens": 800}
  ],
  "total_latency_ms": 1920,
  "total_input_tokens": 1000,
  "total_output_tokens": 320,
  "total_tokens": 1320
}
```
