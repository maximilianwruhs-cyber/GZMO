# M3–M5 Final Handoff — After This Session

**Date:** 2026-07-08
**Daemon:** ✅ Running (PID 123467, new binary, cloud mode)
**Codebase:** ✅ Ahead of docs — M3 implemented, M4 scaffolded, M5 started

---

## What Was Done This Session

| Area | Detail |
|---|---|
| **Access** | SSH keys generated + installed on PVE + VM200 |
| **Sys Janitor** | ↓ 48×/day → 4×/day (every 6h) |
| **Deploy scripts** | Updated for consolidated embed+rerank on :8081 |
| **Rust toolchain** | Installed on both workstation + CT101 |
| **ripen.rs** | M5 Rust engine written, compiled into daemon |
| **knowledge_core.db** | Exported 32,161 rows (confidence ≥ 0.95) |
| **Qdrant knowledge_core** | Synced 32,661 points |
| **Pre-deploy gate** | Wired into systemd, tests vault integrity |
| **Bash helpers** | `vault_stats.py`, `honeypot_stats.py`, `check_qdrant.py` in /tmp |

---

## Baseline Evaluation (M4)

| Metric | Value | Target | Gap |
|---|---|---|---|
| **Recall@5 (RRF)** | **0.3077** | ≥ 0.85 | 🔴 Biggest gap |
| **Recall@5 (RRF strict)** | 0.5977 | ≥ 0.85 | 🟡 Moderate |
| **Must-entities recall** | 1.0 | ≥ 0.90 | ✅ Done |
| **Must-facts recall** | 0.696 | ≥ 0.85 | 🟡 Moderate |
| **Faithfulness (judge)** | 0.806 | ≥ 0.9 | 🟡 |
| **Faithfulness (context)** | 0.806 | ≥ 0.9 | 🟡 |
| **Anti-entities** | 0 | 0 | ✅ Done |
| **Entity promotion rate** | 0.937 | ≥ 0.80 | ✅ Done |
| **Honeypot ratio** | 0.785 | 0.10–0.30 | 🟡 Slightly high |

---

## The Recall@5 Problem — Root Cause

The eval shows specific lost facts like:
- Query: *"What is the role of the Awareness Agent?"*
- Expected: *"Du bist das sensorische Bewusstsein des OpenClaw-Systems"* (from `awareness_agentmd.md`)
- Found: `curated/02_notebooklm/cybernetics-and-mythos...` (unrelated curated doc)

**Cause:** Vector search prefers general curated documents over specific source files because:
1. Curated docs have more tokens → stronger vector signal
2. RRF prefetch multiplier (4×) still lets curated docs dominate
3. No source-file boost in the ranking

### Fix Steps (in order of impact)

**Step 1: Tune RRF parameters** (edit `/opt/gzmo/gzmo.toml`)
```toml
[rerank]
prefetch_multiplier = 8          # was 4 — more candidates before rerank
```
Then re-run eval:
```bash
python3 scripts/ingest-quality/run-recall-eval.py --batch 1 --match normalized
```

**Step 2: Add rerank boost for source file match**
The RRF pipeline should boost results whose `source_file` contains the query's source file name. This is a rust change in `recall_rrf.rs`:
```
score(d) += 1.0  if query.source_hint matches d.source_file
```

**Step 3: Tune BM25 vs vector weight**
In `rrf.rs`:
```rust
// Give BM25 higher weight for entity-name queries
const RRF_K: f64 = 60.0;  // lower = more weight to top ranks
```
Try RRF_K = 40 for BM25 stream to boost keyword-precise matches.

**Step 4: Improve chunking** (if steps 1–3 aren't enough)
Current `chunk_chars = 28000` — large chunks dilute specific facts. Test with 8000:
```toml
[ingest]
chunk_chars = 8000
```
Re-ingest the wave files and re-run eval.

---

## M5 Ripen Engine — How to Trigger

The `ripen.rs` code is compiled into the daemon but needs a cron job to fire:

```toml
# Add to /opt/gzmo/gzmo.toml:
[orchestration.jobs.honeypot_ripen]
cron = "0 0 * * * *"
disabled = false
prompt = "(system job — ripen runs internally)"
```

Then restart:
```bash
systemctl restart gzmo-daemon
```

Check output:
```bash
journalctl -u gzmo-daemon | grep ripen
python3 scripts/export-knowledge-core.py --min-confidence 0.90 --min-recall 0
```

---

## Quick Reference (daily ops)

```bash
# Check daemon
ssh pve "pct exec 101 -- systemctl status gzmo-daemon"

# Tail logs
ssh pve "pct exec 101 -- journalctl -u gzmo-daemon -f"

# Run eval
ssh pve "pct exec 101 -- timeout 300 python3 /opt/gzmo/survey_GZMO/scripts/ingest-quality/run-recall-eval.py --batch 1 --match normalized"

# Rebuild daemon after Rust changes
ssh pve "pct exec 101 -- bash -c '
  source \$HOME/.cargo/env && cd /opt/gzmo/survey_GZMO && cargo build --release && systemctl restart gzmo-daemon
'"

# Check vault stats
ssh pve "pct exec 101 -- python3 /tmp/vault_stats.py"

# Export knowledge core
ssh pve "pct exec 101 -- python3 /opt/gzmo/survey_GZMO/scripts/export-knowledge-core.py"

# Check Qdrant
ssh pve "pct exec 101 -- python3 /tmp/check_qdrant.py"
```

---

## Current State (Maturity Map)

```
M0  ████████████████████  DONE
M1  ████████████████████  DONE  
M2  ████████████████████  DONE
M3  ████████████████████  DONE (code ahead of docs)
M4  ████████░░░░░░░░░░░░  30.8% recall vs 85% target — RECALL TUNING REMAINS
M5  ██████░░░░░░░░░░░░░░  ripen compiled, export done, Qdrant synced
```