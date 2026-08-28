# Beat Gates — Lab vs Legacy Parity Proof

**Source:** `little-tools-lab/scripts/beat-gate.sh`, [CT101_BOUNDARY.md](../../ops/CT101_BOUNDARY.md)  
**Parent:** [120-two-stack-boundary/SYSTEM.md](./SYSTEM.md)

---

## Capability

Compares Little Tools Lab **recipe output** against CT101 legacy expectations per cognition loop. Emits `beat-meta.json` (S2 stage) with pass/fail and metrics. Explicitly **does not authorize** changes to CT101 — reference baseline only until full-stack cutover.

From [CT101_BOUNDARY.md](../../ops/CT101_BOUNDARY.md):

> **beat-gate** — Reference baseline only | Proves lab recipes match or beat legacy behavior

---

## How it works

### CLI

```14:32:github-clone/little-tools-lab/scripts/beat-gate.sh
usage() {
  echo "Usage: $0 --loop config|ops|cognition|knowledge [--fixture|--live] [--meta PATH] [--verify-baseline RATE]"
  echo "  cognition honors VAULT_PATH (e.g. GZMO/data-next/vault.db for GZMO-next)"
}
// ...
[[ -n "$LOOP" ]] || usage
```

### Per-loop recipe dispatch

```45:68:github-clone/little-tools-lab/scripts/beat-gate.sh
case "$LOOP" in
  config)
    bash "$LAB/scripts/gzmo-handoff.sh" "--$MODE" --meta "$RECIPE_META"
  ;;
  ops)
    bash "$LAB/scripts/ops-smoke.sh" "--$MODE" --meta "$RECIPE_META"
  ;;
  cognition)
    bash "$LAB/scripts/cognition-smoke.sh" "--$MODE" --meta "$RECIPE_META"
  ;;
  knowledge)
    bash "$LAB/scripts/session-to-dream.sh" "--$MODE" --stats "$RECIPE_META"
  ;;
esac
```

### Meta comparison binary

```71:77:github-clone/little-tools-lab/scripts/beat-gate.sh
"$CARGO_TARGET_DIR/release/beat-gate-meta" \
  --loop-kind "$LOOP" \
  --mode "$MODE" \
  --recipe-meta "$RECIPE_META" \
  --meta-out "$META" \
  --verify-baseline "$VERIFY_BASELINE"
```

Builds `beat-gate-meta` from `little-tools-lab/common` if missing.

### Modes

| Mode | Purpose |
|------|---------|
| `--fixture` | Deterministic offline comparison (CI-friendly) |
| `--live` | Against real vault/LLM paths (workstation or GZMO-next data) |

`VERIFY_BASELINE` rate controls how strict regression vs CT101 reference is.

---

## Interfaces

| Interface | Value |
|-----------|-------|
| Script path | `little-tools-lab/scripts/beat-gate.sh` |
| Also at | `github-clone/scripts/beat-gate.sh` (wrapper) |
| Lab root | `LITTLE_TOOLS_LAB_ROOT` or `$GZMO_CLONE_ROOT/little-tools-lab` |
| Output | `META` env or `/tmp/beat-meta.json` |
| Cognition vault | `VAULT_PATH` (e.g. `GZMO/data-next/vault.db`) |
| Cargo target | `CARGO_TARGET_DIR` default `$ROOT/temp-bench/target` |

### Loop mapping to CT101 legacy

| Beat loop | CT101 legacy equivalent | Lab recipe |
|-----------|---------------------------|------------|
| `config` | `gzmo.toml` calibration | `gzmo-handoff.sh` |
| `ops` | Daemon health / sidecars | `ops-smoke.sh` |
| `cognition` | Spark/distill smoke | `cognition-smoke.sh` |
| `knowledge` | Dream narrative stats | `session-to-dream.sh` |

---

## THINKING nodes

> **THINKING — beat-gate:boundary disclaimer**
> - *Reviewed:* Header comment: "Does NOT authorize CT101 changes".
> - *Insight:* Prevents beat-gate green from triggering per-loop production graft.
> - *Risk / limitation:* Operators may misread pass as "deploy to CT101".
> - *Enhancement:* Require explicit `CUTOVER_APPROVED=1` for any CT101 migration tooling. [CT101-safe]

> **THINKING — beat-gate:four-loop coverage**
> - *Reviewed:* config/ops/cognition/knowledge — matches S3 stack-ready checklist.
> - *Insight:* Aligns with scheduler job table + assembly slices.
> - *Risk / limitation:* Discovery automation not in beat-gate — separate validation.
> - *Enhancement:* Fifth loop `discovery` with gzmo_skills fixture reports. [GZMO-next]

> **THINKING — beat-gate:beat-gate-meta binary**
> - *Reviewed:* Rust comparator merges recipe meta into scored output.
> - *Insight:* Single artifact for CI upload / trend tracking.
> - *Risk / limitation:* Baseline drift if CT101 reference updated without lab sync.
> - *Enhancement:* Versioned baseline fixtures checked into little-tools-lab. [GZMO-next]

---

## Advancement (ADR-0005)

| Today | Target |
|-------|--------|
| Manual `beat-gate.sh --fixture/--live` | CI nightly loops green |
| Single-loop PASS + operator ack | **Promote-by-loop** into current living host |
| Whole-host migration | Still `CUTOVER_APPROVED=1` only |
| beat-meta.json local | Central beat history for promote decisions |

Silent CI green still does **not** auto-deploy. Promote-by-loop replaces “full assembly or nothing.”

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | CI workflow running all four loops `--fixture` | [GZMO-next] |
| 2 | Versioned CT101 baseline fixtures in lab repo | [GZMO-next] |
| 3 | Discovery loop beat-gate for report schema | [GZMO-next] |
| 4 | Observatory beat-history panel | [GZMO-next] |
| 5 | Document beat-gate pass ≠ CT101 deploy in operator training | [CT101-safe] |
