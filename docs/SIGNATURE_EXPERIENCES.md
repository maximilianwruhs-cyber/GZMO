# Signature Experiences — GZMO-next (operator-felt)

**Date:** 2026-07-16  
**Rule:** Sharpen existing loops. No new subsystems.  
**Aesthetic:** Austrian pragmatism — you know it worked when a file or status line proves it.

---

## Experience A — Overnight Dream Brief

**One week story:** You sleep. The machine consolidates. In the morning you read a short dream report that cites real sessions, not vibes.

| Field | Detail |
|-------|--------|
| **Trigger** | `gzmo-scheduler` dream job 01:00 UTC |
| **Pieces / recipes** | `session-to-dream.sh` → `session-distill` + `ltl-common` `dream-append` / `vault-promote-distill` (elevate: add `rem-substrate`) |
| **Artifacts** | `data-next/DREAMS.md`, vault rows, `dream-stats.json` meta |
| **Surface** | Observatory dreams panel; SOUL heuristic “What happened overnight?” → read `DREAMS.md` + `journalctl --user -u gzmo-scheduler` |
| **You know it worked when…** | `DREAMS.md` gains a dated section with `mode: librarian_live` (or honest heuristic), vault fact count rises, `scheduler-runs` shows `dream` ok |

**Live proof (2026-07-16):** `DREAMS.md` already has librarian_live sessions and 11 facts promoted; vault at **111** facts.

**Sharpen:** Emit honeypot-origin promotions into the dream summary; optional REM markdown from `rem-substrate`.

---

## Experience B — Spark Serendipity (twice daily)

**One week story:** Twice a day the machine links a *stale* fact to something recent. You can point at the hypothesis and verdict.

| Field | Detail |
|-------|--------|
| **Trigger** | Spark cron 03:30 / 22:30 → `cognition-smoke.sh --live --spark-run` |
| **Pieces** | `session-distill` → `honeypot-gate` → **spark-link** → `rrf-recall` |
| **Artifacts** | `SparkReport` JSON (`selection`, `hypothesis`, `verdict`, `promoted`); cognition-smoke-meta |
| **Surface** | Today: meta under lab temp dirs / recipe output. Elevate: last spark card in Observatory + `gzmo status` overnight brief |
| **You know it worked when…** | A spark report shows non-zero `stale_sweetness` mid-window anchor and a verify verdict — not a random recent fact |

**Code:** [`spark-link/src/scoring.rs`](../../spark-link/src/scoring.rs) `stale_sweetness`.

---

## Experience C — Fuse & Promote (calibration theatre)

**One week story:** You run (or overnight handoff runs) benches; Lorenz maps chaos state to LLM knobs; fuse gates on verify pass_rate; you review a sibling TOML and promote deliberately.

| Field | Detail |
|-------|--------|
| **Trigger** | `gzmo assemble handoff --live --apply` or scheduler 04:00 `gzmo-handoff.sh`; chat `/calibrate` is fixture rehearsal |
| **Pieces** | temp-bench → verify-suite → **lorenz-map** → tempo/speed/top-p/rapl → **config-fuse** |
| **Artifacts** | `config/gzmo-next-fused.toml`, `fuse-meta.json` (`gate_passed`) |
| **Surface** | `gzmo instance status` (fused present); `gzmo config promote-fused --diff\|--apply`; Observatory calibration pending |
| **You know it worked when…** | Fused file newer than live config; `--diff` shows engine/inference deltas; promote is a conscious human act |

**Live proof:** `gzmo instance status` reports fused TOML **present — review + promote-fused**.

---

## Experience D — Ops Heartbeat (honest green)

**One week story:** Startup (and ops assemble) prove endpoints, Synapse distill health, and plan-gate. False greens fail.

| Field | Detail |
|-------|--------|
| **Trigger** | Scheduler startup `ops-smoke.sh --live`; `gzmo assemble ops --live` |
| **Pieces** | endpoint-scan → synapse-health → plan-gate (+ live redis/qdrant/neo4j metrics) |
| **Artifacts** | `ops-smoke-meta.json`, `scheduler-runs/latest.json` |
| **Surface** | Observatory body “GZMO-next — the mind”; `gzmo health` |
| **You know it worked when…** | Meta PASS with real sidecar flags; unhealthy Synapse stalls are *reported* (not hidden) |

**Live proof (2026-07-16):** ops-smoke **PASS**; `healthy=false` due to stale fixture sessions; queue_depth=4; redis/qdrant/neo4j true.

---

## Experience E — Mentor Hour (lab-only, ADR-0002)

**One week story:** Once a week you deliberately run pedagogy. The machine tutors inside a ZPD — not overnight cron.

| Field | Detail |
|-------|--------|
| **Trigger** | Manual `gzmo assemble pedagogy --live` (or fixture first) |
| **Pieces** | **zpd-tutor** → pedagogy-bench → skill-patch |
| **Artifacts** | `SessionReport`, pedagogy-smoke-meta |
| **Surface** | CLI only (by design); document as mentor ritual in SOUL-next heuristics |
| **You know it worked when…** | Session report shows level movement inside `zpd_range`; no scheduler job exists for pedagogy |

**Boundary:** Do not add pedagogy to `gzmo-scheduler` without ADR amendment + beat-gate.

---

## Experience F (stretch) — Honeypot Metabolism Visible

Not fully felt yet — elevation target #1.

| Field | Detail |
|-------|--------|
| **Trigger** | Distill + gate + promote across dream/spark/discovery |
| **Pieces** | honeypot-gate lifecycle + vault promote + optional discovery-smoke |
| **Artifacts** | Vault rows with honeypot origin / decay; gate classify outcomes |
| **Surface** | `/status` and Observatory vault counts split: vault vs honeypot vs pending |
| **You know it worked when…** | `SELECT origin, count(*) …` shows honeypot > 0 and rising week-over-week without CT101 import |

**Live baseline:** honeypot-origin facts on next vault = **36** (2026-07-16; rising via dream/cognition promote seam).

---

## Loops that are plumbing (keep, don’t brag)

- Qdrant vault sync — necessary vector hygiene  
- Redis distill queue file fallback — reliability  
- Endpoint scan alone — commodity health check  

Brag about Dream / Spark / Fuse / Honeypot. Mention ops when it catches a stall.

---

## Related

- [`UNIQUENESS_THESIS.md`](UNIQUENESS_THESIS.md)  
- [`little-tools-lab/docs/PIECE_ELEVATION_MAP.md`](../../little-tools-lab/docs/PIECE_ELEVATION_MAP.md)  
- [`UNIQUENESS_BUILD_PLAN.md`](UNIQUENESS_BUILD_PLAN.md)
