# M3 local baseline — Prime `:8000`

**Captured:** 2026-06-25 (after local-first rollback)  
**Purpose:** Reference point for M3 dream/spark signal on local Prime (Gemma 4 31B). Use for nightly regression checks and future cloud-path comparison.

Machine-readable snapshot: [`data/eval/m3-local-baseline-2026-06-25.json`](../data/eval/m3-local-baseline-2026-06-25.json)

## Steady-state config

| Setting | Value |
|---------|-------|
| `engine.active_mode` | `local` |
| `routing.cloud_first_background` | `false` |
| `compliance.allow_cloud_engine` | `false` |
| Prime | `gzmo-prime.service` enabled, `:8000` |
| Daemon | `gzmo-daemon.service` (nightly dream via `[dreams].cron`) |
| Sidecar | LXC101 Neo4j/Qdrant/Redis unchanged |

Rollback log: [CLOUD_MIGRATION.md](./CLOUD_MIGRATION.md#rollback-to-local-2026-06-25)

## Dream reliability (2026-06-25)

Changes that unblocked E2E extract/verify on Prime:

| Knob | Value | Notes |
|------|-------|-------|
| `[dreams].chunk_chars` | `12000` | Smaller chunks → less JSON truncation |
| `[dreams].max_tokens_extract` | `16384` | Bounded structured extract |
| `[dreams].max_tokens_verify` | `8192` | Bounded verifier |
| `[dreams].exclude_episodic_substrings` | expanded | Drops janitor/SSD/telemetry noise |
| `[bibliothek].min_dream_cycles` | `15` | Promotion gate (21 cycles completed) |
| `[dreams].honeypot_rem_enabled` | `true` | REM reads honeypot anchors + vector distillates |

Binary: `cargo build --release -p gzmo-cli` (2026-06-25).

## Eval tier 0 (2026-06-25)

```bash
scripts/ingest-quality/eval-quick.sh
scripts/sovereignty-verify.sh
```

| Check | Result |
|-------|--------|
| `sovereignty-verify.sh` | **PASS** (`active_mode=local`) |
| Retrieval probes | **PASS** (all queries) |
| FTS / honeypot index | `honeypot_fts=36195`, stale=0 |
| Vault | `58405` truths |
| Qdrant honeypot / knowledge | `23095` / `18892` |
| MemScore composite | **0.729** |
| Fact recall (golden) | **181/219 = 82.6%** |
| Anti-entity violations | **0** |

## Reference dream — daemon 2026-06-24

First successful E2E dream after reliability fixes (daemon PID 1587925).

| Metric | Value |
|--------|-------|
| Started (UTC) | 2026-06-25 02:52:37 |
| Completed (UTC) | 2026-06-25 03:29:09 |
| Duration | ~36 min |
| Episodic raw | 29 135 chars |
| Light filtered | 3 898 chars |
| Honeypot REM | 2 845 chars |
| Verify kept | **17 entities**, **3 relations** |
| Vault promoted | **19** truths (`origin=verified_dream`) |
| Output | [`DREAMS.md`](../DREAMS.md) — `# Dream Consolidation — 2026-06-24` |

Journal grep:

```bash
journalctl --user -u gzmo-daemon.service --since "2026-06-25 02:52:00" \
  | grep -iE 'Light Phase|Verify Phase|Dream cycle complete|kept_entities'
```

**Note:** No VRAM spike expected when Gemma 31B is already resident on Prime. GPU util 13–24% during extract/verify is normal. With `llama-server -np 1`, parallel `gzmo distill pi` contends for the slot but did not block this run.

## Isolated dream protocol

Use when validating a specific date without daemon timer or Pi distill queue contention:

```bash
# 1. Pause background cognition
systemctl --user stop gzmo-daemon.service
systemctl --user stop pi-mentor-discovery.timer 2>/dev/null || true
pkill -f 'gzmo distill pi' || true
rm -f data/cycle-guard.json   # only if no live gzmo dream/distill PID

# 2. Run (from repo root)
./target/release/gzmo dream 2026-06-23

# 3. Restore
systemctl --user start gzmo-daemon.service
systemctl --user start pi-mentor-discovery.timer 2>/dev/null || true
```

Success criteria:

- `Verify Phase complete` with `kept_entities > 0`
- `Dream cycle complete entities=N` with `N > 0`
- `DREAMS.md` section for the target date updated

## Isolated dream — CLI 2026-06-23

Executed 2026-06-25 after baseline doc capture. Daemon was stopped; Pi distill and an auto-restarted daemon briefly contended for Prime (`-np 1`) during the long extract wait.

| Metric | Value |
|--------|-------|
| Started (UTC) | 2026-06-25 03:39:46 |
| Completed (UTC) | 2026-06-25 05:39:13 |
| Wall time | ~2 h (queue wait); verify+deep &lt; 1 min once isolated |
| Episodic raw | 36 431 chars |
| Light filtered | 26 259 chars |
| Honeypot REM | 2 119 chars |
| Verify kept | **21 entities**, **12 relations** |
| Vault promoted | **47** truths |
| Output | [`DREAMS.md`](../DREAMS.md) — `# Dream Consolidation — 2026-06-23` |

Log: `/tmp/gzmo-dream-2026-06-23.log`

## Spark status (partial)

Spark runs and promotes hypotheses when JSON parses cleanly. Intermittent failures observed on 2026-06-24:

- `missing field anchor_label` (truncated JSON)
- `expected ':'` (malformed hypothesis)

Spark is **not** part of this baseline pass/fail gate; dream E2E with `entities > 0` is the primary M3 signal.

## M3 gate checklist

| Criterion | Status (2026-06-25) |
|-----------|---------------------|
| Local-first steady state | Done |
| `sovereignty-verify.sh` PASS | Done |
| Eval tier 0 PASS (MemScore ≥ 0.7) | Done (0.729) |
| Dream E2E `entities > 0` on Prime | Done (17 on 2026-06-24) |
| Isolated dream on richer episodic day | Done (`2026-06-23`: 21 entities, 12 relations) |
| One week meaningful nightly output | Open (roadmap) |
| 3 manual recall questions from honeypot | Open (operator) |

## Related

- [ROADMAP_TO_M5.md](./ROADMAP_TO_M5.md) — Block B (M3)
- [EVAL_TIERS.md](./EVAL_TIERS.md) — tier definitions
- [MEMORY_ARCHITECTURE_SPEC.md](./MEMORY_ARCHITECTURE_SPEC.md) — honeypot / dream pipeline
