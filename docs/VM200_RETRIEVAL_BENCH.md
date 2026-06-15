# VM200 retrieval benchmark

HTTP benchmark for embed + rerank on VM200 (and E2E `gzmo memory search`).

## Quick run

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/vm200/retrieval-bench/runner.py \
  --profile scripts/vm200/retrieval-bench/profiles/post-router-qwen3.json \
  --tag smoke
```

Requires PyYAML (`pip install pyyaml`) for full workloads; falls back to minimal probes without it.

## Profiles

| Profile | Use |
|---------|-----|
| `profiles/pre-router-baseline.json` | Legacy `:8081` embed + `:8082` bge rerank |
| `profiles/post-router-qwen3.json` | Unified router `:8081` |

## Metrics captured

- **Embed:** p50/p90/p95/p99 latency, 1024-dim check
- **Rerank:** p50/p95 at batch sizes 1, 5, 15, 40; top-score sanity (> 1e-6)
- **VRAM:** `nvidia-smi` on VM200 via SSH
- **E2E:** `gzmo memory search` wall time (if binary built)

Output: `scripts/vm200/retrieval-bench/runs/<run_id>/summary.json`

## Promote baseline

```bash
./scripts/vm200/retrieval-bench/promote-baseline.sh
# → baseline-lock.json
```

## Suggested gates (post-cutover)

| Check | Gate |
|-------|------|
| Embed p95 | ≤ pre-cutover + 15% |
| Rerank batch-40 p95 | ≤ pre-cutover + 20% |
| Rerank top score | > 0.01 on canonical probe |
| VRAM peak | < 6.5 GB on 8 GB GTX 1070 |

Pair with quality harness:

```bash
scripts/ingest-quality/eval-quick.sh
python3 scripts/ingest-quality/run-recall-eval.py --batch all --backend gzmo --match strict --track rrf
```
