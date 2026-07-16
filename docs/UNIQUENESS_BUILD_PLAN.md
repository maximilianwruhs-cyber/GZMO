# Uniqueness Build Plan — GZMO-next × Little Tools Lab

**Date:** 2026-07-16  
**Depends on:** [`UNIQUENESS_THESIS.md`](UNIQUENESS_THESIS.md), [`SIGNATURE_EXPERIENCES.md`](SIGNATURE_EXPERIENCES.md), [`little-tools-lab/docs/PIECE_ELEVATION_MAP.md`](../../little-tools-lab/docs/PIECE_ELEVATION_MAP.md)  
**Rule:** PR-sized workstreams. Elevate assemblies + seams. No fat daemon rebuild. No CT101 vault import.

---

## Thesis advancement (north star)

Make Dream / Spark / Fuse / honeypot metabolism **undeniable** at small vault scale — then surface it on CLI + Observatory with Austrian pragmatism.

---

## Stop doing (dilutes uniqueness)

- Importing CT101’s 60k vault to “look busy”
- Adding a 47th piece without exceptional ADR
- Grafting lab loops into CT101 (`[assembly]` on legacy)
- Scheduling pedagogy / dice-scheduler / PulseLoop overnight (ADR-0002)
- Relitigating enhancement P0–P2 / stretch S1–S3,S5–S6 as primary work
- Rewriting Observatory as a redesign festival (2–3 signature moments only)
- Claiming uniqueness from Redis+Qdrant+local LLM alone

---

## Workstreams (dependency-ordered)

### W1 — Catalog honesty (docs only)

**Goal:** Stop lying about production roles so elevation targets are clear.  
**Files:** `little-tools-lab/catalog/ASSEMBLIES.md` (neural-finesse vs dream-append), `LITTLE_TOOLS_LAB.md` / project cards for demote list, ADR-0002 callouts on dice/adaptive-tempo.  
**Proof:** Docs review; no beat required.  
**Advances thesis:** Credibility — mature ≠ signature.

### W2 — Honeypot metabolism (identity sentence)

**Goal:** Produce and count honeypot-origin (or equivalent lifecycle) facts on GZMO-next without CT101 import.  
**Files likely:** `honeypot-gate` lifecycle seam; `session-to-dream.sh` / `vault-promote-distill`; vault schema origin writes; `gzmo status` / Observatory vault split.  
**Proof:** Fixture cognition + live dream once; `SELECT origin, count(*) FROM facts GROUP BY origin` shows honeypot > 0.  
**Advances thesis:** MACHINE “Honeypot + verify + promote” becomes true on next.  
**ADR-0002:** N/A (core cognition path).

### W3 — Spark lineage surface

**Goal:** Last successful SparkReport visible to operators.  
**Files:** `cognition-smoke.sh` (persist report under `data-next/`); Observatory `workstation-snapshot.py` + body card; optional `gzmo status` overnight brief.  
**Proof:** After `--spark-run` (fixture or live), Observatory/status shows anchor + verdict.  
**Advances thesis:** Experience B felt.

### W4 — REM in dream recipe

**Goal:** Wire `rem-substrate` into `session-to-dream.sh` when anchors exist (align toml `honeypot_rem_enabled`).  
**Files:** `session-to-dream.sh`, rem-substrate CLI invoke, dream-stats schema if needed, `ASSEMBLIES.md`.  
**Proof:** Fixture dream with sample facts emits REM markdown section; beat-gate knowledge still PASS.  
**Advances thesis:** Dream depth without inline DreamEngine.

### W5 — Fuse & promote ritual polish

**Goal:** Calibration pending is a first-class operator moment.  
**Files:** Observatory calibration card (fused mtime vs live); SOUL-next heuristic for “calibration pending”; ensure `gzmo config promote-fused --diff` is the documented path.  
**Proof:** With fused present, status/Observatory shows pending; after promote, pending clears.  
**Advances thesis:** Experience C.

### W6 — Graph ledger post-dream

**Goal:** Small-vault metabolism = drift ledger, not 60k scale.  
**Files:** `session-to-dream.sh` optional stage or `ops-smoke` extras; `graph-ledger`; schema meta.  
**Proof:** Fixture dream → ledger row + anomaly_count in meta.  
**Advances thesis:** L4 memory-as-organism at small scale.

### W7 — Evidence + recall floor

**Goal:** Quality pieces support promote/verify, not sit orphan.  
**Files:** cognition-smoke optional evidence-locate stage; recall-eval weekly script or Observatory floor; beat-gate hook optional.  
**Proof:** Fixture cognition attaches evidence spans; recall-eval fixture green in CI.  
**Advances thesis:** Grounded promote; anti-generic vs “chat said so”.

### W8 — Chaos / pedagogy felt (ADR-0002 safe)

**Goal:** Mentor hour + Thought Cabinet as *chat/assemble rituals*, never cron.  
**Files:** SOUL-next heuristics; `OPERATOR` or runbook “weekly mentor”; chat docs for cabinet-sim / research-budget; **no** `jobs.rs` changes for pedagogy/chaos.  
**Proof:** Docs + manual `gzmo assemble pedagogy --fixture` green; scheduler job list unchanged.  
**Advances thesis:** L5 moat without ADR violation.  
**ADR-0002:** Explicit — lab/chat only.

### W9 — Lorenz deepen (optional fuse)

**Goal:** Shared Lorenz RK4 behind lorenz-map / tempo-bench (contract-safe via ltl-common or thin shared crate — pieces still don’t import each other).  
**Files:** lorenz-map, tempo-bench, possibly baseline-bench; PATH_DECISION if shared code path changes.  
**Proof:** Fixture bench-to-fuse still green; no piece↔piece import.  
**Advances thesis:** Signature chaos math, less duplication.

### W10 — Observatory soul path fix

**Goal:** Snapshot reads `SOUL-next.md` on next instance.  
**Files:** `gzmo-observatory/workstation-snapshot.py` (soul excerpt path).  
**Proof:** Local observatory shows next soul lines.  
**Advances thesis:** L1 SOUL vs implementation.

---

## Suggested sequencing

```text
W1 (docs) → W2 (honeypot) → W3 (spark surface) → W4 (REM)
                ↘ W5 (fuse UX) → W6 (ledger)
W7 parallel after W2
W8 anytime (docs/ritual) — never blocks W2–W5
W9 after fuse path stable
W10 small parallel
```

---

## Beat / fixture proof matrix

| Workstream | Minimum proof |
|------------|---------------|
| W1 | Doc review |
| W2 | `cognition-smoke --fixture` + vault origin query after live dream |
| W3 | Spark report path exists + Observatory/status read |
| W4 | `session-to-dream --fixture` + REM section |
| W5 | `gzmo instance status` + promote-fused --diff |
| W6 | graph-ledger fixture in dream/ops meta |
| W7 | evidence-locate + recall-eval fixture CI |
| W8 | pedagogy-smoke --fixture; jobs.rs unchanged |
| W9 | bench-to-fuse --fixture |
| W10 | Observatory local snapshot |

---

## Session opener (for implementation agent)

```
You are implementing GZMO uniqueness elevation for GZMO-next × Little Tools Lab.

Read first:
  GZMO/docs/UNIQUENESS_THESIS.md
  GZMO/docs/SIGNATURE_EXPERIENCES.md
  GZMO/docs/UNIQUENESS_BUILD_PLAN.md
  little-tools-lab/docs/PIECE_ELEVATION_MAP.md

Env:
  export GZMO_CLONE_ROOT=/home/gzmo/github-clone
  export GZMO_INSTANCE=next
  export GZMO_CONFIG=$GZMO_CLONE_ROOT/GZMO/config/gzmo-next.toml
  export LITTLE_TOOLS_LAB_ROOT=$GZMO_CLONE_ROOT/little-tools-lab
  export CARGO_TARGET_DIR=$GZMO_CLONE_ROOT/temp-bench/target

HARD LAWS:
  ADR-0001 — no CT101 graft
  Closed set 46 — no 47th without exceptional justification
  Piece contract — no piece↔piece imports; recipes pass paths
  ADR-0002 — no pedagogy/chaos/dice-scheduler on gzmo-scheduler cron
  Prefer thin scheduler + lab recipes
  Do not import CT101 vault

START with W1 then W2. Ship PR-sized commits. Prove each workstream with the
Build Plan proof matrix. Prefer Dream/Spark/Fuse/honeypot legibility over new features.

Optional canvas: ~/.cursor/projects/home-gzmo/canvases/gzmo-ltl-uniqueness-map.canvas.tsx
```

---

## Open stretch (only if serves uniqueness)

- S4 gVisor shell sandbox — only if it enables a signature “sovereign execution” story; otherwise leave.

---

## Done when

An operator can, in one week without reading the codebase:

1. Read overnight dream and see vault growth  
2. Point at a spark hypothesis/verdict  
3. See honeypot count > 0 and rising  
4. Diff and promote fused calibration consciously  
5. Run mentor assemble without fearing overnight cron creep
