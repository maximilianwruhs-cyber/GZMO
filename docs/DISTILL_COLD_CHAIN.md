# Distill Cold Chain

The GZMO distill pipeline has three ingress paths with different episodic behavior:

## Hot archive queue (cold chain)

```
Hot context > 90% → prune_with_archive → enqueue_distill
  → Redis BRPOP (gzmo:distill:pending) or data/distill-queue/*.json
  → run_distill_worker → distill_transcript
  → vault + honeypot + distill_dedup
  → episodic ONLY if DistillSource::MainArchive
  → distill_complete on Synapse
```

**SubArchive** jobs (subagent pruned context) write to vault but **skip episodic** (`session_distill.rs`).

## GZMO chat sessions

Cron 02:15 UTC on `data/sessions/*.json` → `MainArchive` → episodic yes.

## Pi JSONL

`session_end` / `gzmo distill pi` → `MainArchive` → episodic yes.

## Quality gap

Distill extraction uses config-default temperature; skill outputs are chaos-adaptive. Monitor queue depth via `probe-distill-queue.sh`.

## Spark ↔ distill correlation

`spark_distill_bridge` registers `spark_complete` events during daemon synapse poll and logs distill lineage gaps.
