#!/usr/bin/env bash
# Promote-by-loop ritual (ADR-0005 / LTL ADR-0003) — never silent.
#
#   # Record-only after kit PASS:
#   PROMOTE_LOOP=knowledge PROMOTE_ACK=1 bash scripts/promote-loop.sh
#
#   # Living apply (knowledge|cognition — disposable-vault doctrine: protect writer+recipe):
#   bash scripts/living-host-mutex.sh claim --host ct101 --note "promote-apply cognition"
#   PROMOTE_LOOP=cognition PROMOTE_ACK=1 PROMOTE_APPLY=1 bash scripts/promote-loop.sh
#   bash scripts/living-host-mutex.sh release
#
# Whole-host cutover still needs CUTOVER_APPROVED=1 — this script refuses that path.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
LAB="${LITTLE_TOOLS_LAB_ROOT:-$CLONE/little-tools-lab}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/beat-gate/promotions"
LOOP="${PROMOTE_LOOP:-}"
ACK="${PROMOTE_ACK:-}"
APPLY="${PROMOTE_APPLY:-0}"
LIVING_HOST="${PROMOTE_LIVING_HOST:-ct101}"
LIVING_PROMOTIONS="${PROMOTE_LIVING_PROMOTIONS:-/opt/gzmo/data/beat-gate/promotions}"
# Narrow blast radius: only these loops may living-apply (config/ops/discovery stay record-only)
APPLY_LOOPS="${PROMOTE_APPLY_LOOPS:-knowledge,cognition}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"

usage() {
  echo "Usage: PROMOTE_LOOP=<config|ops|cognition|knowledge|discovery> PROMOTE_ACK=1 bash $0" >&2
  echo "  PROMOTE_APPLY=1 — knowledge|cognition only; requires mutex claim + dual_writer_risk=false" >&2
  exit 2
}

[[ -n "$LOOP" ]] || usage
[[ "$ACK" == "1" ]] || {
  echo "error: set PROMOTE_ACK=1 after reviewing beat-gate PASS (no silent promote)" >&2
  exit 2
}
case "$LOOP" in
  config|ops|cognition|knowledge|discovery) ;;
  *) echo "error: unsupported loop '$LOOP'" >&2; usage ;;
esac
if [[ "${CUTOVER_APPROVED:-}" == "1" ]]; then
  echo "error: CUTOVER_APPROVED=1 is whole-host cutover — use cutover tooling, not promote-loop" >&2
  exit 2
fi
IFS=',' read -r -a _APPLY_OK <<< "$APPLY_LOOPS"
_apply_ok=0
for _l in "${_APPLY_OK[@]}"; do
  [[ "$LOOP" == "$_l" ]] && _apply_ok=1 && break
done
if [[ "$APPLY" == "1" && "$_apply_ok" != "1" ]]; then
  echo "error: PROMOTE_APPLY=1 not enabled for loop='$LOOP' (allowed: $APPLY_LOOPS)" >&2
  exit 2
fi

mkdir -p "$OUT"
MUTEX_JSON="$("$ROOT/scripts/living-host-mutex.sh" status 2>/dev/null || echo '{}')"

echo "=== promote-loop: beat-gate fixture ($LOOP) ==="
META="$OUT/pre-${LOOP}-meta.json"
bash "$LAB/scripts/beat-gate.sh" --loop "$LOOP" --fixture --meta "$META"

echo "=== promote-loop: mutex / dual-writer ==="
export MUTEX_JSON OUT LOOP META ROOT LAB APPLY LIVING_HOST LIVING_PROMOTIONS APPLY_LOOPS
python3 - <<'PY'
import json, os, shutil, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
loop = os.environ["LOOP"]
meta_path = Path(os.environ["META"])
root = Path(os.environ["ROOT"])
lab = Path(os.environ["LAB"])
apply = os.environ.get("APPLY", "0") == "1"
living_host = os.environ.get("LIVING_HOST") or "ct101"
living_promotions = os.environ.get("LIVING_PROMOTIONS") or "/opt/gzmo/data/beat-gate/promotions"
apply_loops = {
    x.strip()
    for x in (os.environ.get("APPLY_LOOPS") or "knowledge,cognition").split(",")
    if x.strip()
}
RECIPE_BY_LOOP = {
    "knowledge": "session-to-dream.sh",
    "cognition": "cognition-smoke.sh",
}

meta = json.loads(meta_path.read_text(encoding="utf-8"))
mutex = {}
try:
    mutex = json.loads(os.environ.get("MUTEX_JSON") or "{}")
except Exception:
    mutex = {}

gate = (meta.get("metrics") or {}).get("gate_passed")
beats = meta.get("beats_incumbent")
baseline_id = meta.get("baseline_id")
baseline_path = meta.get("baseline_path")
dual = mutex.get("dual_writer_risk")
claim = mutex.get("claim") or {}
claim_host = claim.get("host")

ok = bool(gate) and bool(beats) and bool(baseline_id) and dual is not True
apply_error = None
applied = None
rollback = None
mode = "record_only"

if apply:
    mode = "living_apply"
    if not claim_host:
        ok = False
        apply_error = "mutex_claim_required — bash scripts/living-host-mutex.sh claim --host ct101 --note 'promote-apply knowledge'"
    elif dual is True:
        ok = False
        apply_error = "refused_dual_writer — stop workstation gzmo-serve/scheduler before living apply"
    elif loop not in apply_loops:
        ok = False
        apply_error = f"apply_loop_not_enabled — allowed={sorted(apply_loops)}"
    elif not ok:
        apply_error = "beat_gate_blocked — need gate_passed+baseline_id before apply"
    else:
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        recipe_name = RECIPE_BY_LOOP.get(loop, f"{loop}-recipe")
        baseline_src = Path(baseline_path) if baseline_path else (lab / f"fixtures/beat-baselines/{loop}.v1.json")
        if not baseline_src.is_file():
            ok = False
            apply_error = f"baseline_missing:{baseline_src}"
        else:
            # Local rollback + handoff package
            pkg = out / f"handoff-{loop}-{stamp}"
            pkg.mkdir(parents=True, exist_ok=True)
            shutil.copy2(baseline_src, pkg / baseline_src.name)
            recipe = {
                "schema": "gzmo.promote_loop.handoff_recipe/v1",
                "loop": loop,
                "recipe": recipe_name,
                "baseline_id": baseline_id,
                "baseline_file": baseline_src.name,
                "living_host": living_host,
                "living_promotions": living_promotions,
                "doctrine": "protect_writer_and_recipe_regenerate_vault",
                "rollback": [
                    f"On {living_host}: restore {living_promotions}/rollback/ from this apply stamp",
                    f"Or delete {living_promotions}/living-applied.json to clear the promote pin",
                    "Do not wipe vault unless intentionally regenerating (disposable-vault doctrine)",
                ],
                "non_goals": [
                    "whole-host cutover (needs CUTOVER_APPROVED=1)",
                    "silent toml/model overnight swap",
                    "multi-loop apply in one command",
                ],
            }
            (pkg / "handoff-recipe.json").write_text(json.dumps(recipe, indent=2) + "\n", encoding="utf-8")
            (pkg / "handoff-recipe.md").write_text(
                "\n".join([
                    f"# {loop} living handoff — {stamp}",
                    "",
                    f"- loop: `{loop}`",
                    f"- baseline: `{baseline_id}`",
                    f"- recipe: `{recipe_name}`",
                    f"- host: `{living_host}:{living_promotions}`",
                    "",
                    "## Rollback",
                    "",
                    f"1. SSH `{living_host}`",
                    f"2. `rm -rf {living_promotions}/current && mv {living_promotions}/rollback {living_promotions}/current` (if rollback/ present)",
                    f"3. Or remove `{living_promotions}/living-applied.json`",
                    "",
                    "Vault wipe is acceptable under disposable-vault doctrine; restore **recipe pin**, not facts.",
                    "",
                ]) + "\n",
                encoding="utf-8",
            )

            # Snapshot remote promotions into rollback/, then install current/
            # Preserve prior loop baselines already in current/ (multi-loop pins).
            remote_prep = (
                f"bash -lc 'set -euo pipefail; "
                f"mkdir -p {living_promotions}; "
                f"if [ -d {living_promotions}/current ] || [ -f {living_promotions}/living-applied.json ]; then "
                f"  rm -rf {living_promotions}/rollback; "
                f"  mkdir -p {living_promotions}/rollback; "
                f"  if [ -d {living_promotions}/current ]; then cp -a {living_promotions}/current/. {living_promotions}/rollback/; fi; "
                f"  if [ -f {living_promotions}/living-applied.json ]; then cp -a {living_promotions}/living-applied.json {living_promotions}/rollback/; fi; "
                f"  for f in {living_promotions}/living-applied-*.json; do "
                f"    [ -f \"$f\" ] && cp -a \"$f\" {living_promotions}/rollback/; "
                f"  done; "
                f"fi; "
                f"mkdir -p {living_promotions}/current; "
                f"if [ -d {living_promotions}/rollback ]; then "
                f"  for f in {living_promotions}/rollback/*.v1.json; do "
                f"    [ -f \"$f\" ] && cp -an \"$f\" {living_promotions}/current/; "
                f"  done; "
                f"fi'"
            )
            p = subprocess.run(
                ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", living_host, remote_prep],
                capture_output=True, text=True,
            )
            if p.returncode != 0:
                ok = False
                apply_error = f"remote_prep:{(p.stderr or p.stdout)[:300]}"
            else:
                scp = subprocess.run(
                    [
                        "scp", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", "-q",
                        str(pkg / baseline_src.name),
                        str(pkg / "handoff-recipe.json"),
                        str(pkg / "handoff-recipe.md"),
                        f"{living_host}:{living_promotions}/current/",
                    ],
                    capture_output=True, text=True,
                )
                if scp.returncode != 0:
                    ok = False
                    apply_error = f"scp_handoff:{(scp.stderr or scp.stdout)[:300]}"
                else:
                    living_applied = {
                        "schema": "gzmo.promote_loop.living_applied/v1",
                        "applied_at": datetime.now(timezone.utc).isoformat(),
                        "loop": loop,
                        "baseline_id": baseline_id,
                        "recipe": recipe_name,
                        "handoff_package": str(pkg),
                        "claim_host": claim_host,
                        "claim_note": claim.get("note"),
                        "rollback_dir": f"{living_promotions}/rollback",
                        "soak_next": "brain-feed-check.sh + living probe GREEN after one overnight",
                    }
                    local_applied = pkg / "living-applied.json"
                    local_applied.write_text(json.dumps(living_applied, indent=2) + "\n", encoding="utf-8")
                    scp2 = subprocess.run(
                        [
                            "scp", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", "-q",
                            str(local_applied),
                            f"{living_host}:{living_promotions}/living-applied.json",
                        ],
                        capture_output=True, text=True,
                    )
                    # Keep per-loop pin so knowledge + cognition can coexist
                    subprocess.run(
                        [
                            "scp", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", "-q",
                            str(local_applied),
                            f"{living_host}:{living_promotions}/living-applied-{loop}.json",
                        ],
                        capture_output=True, text=True,
                    )
                    if scp2.returncode != 0:
                        ok = False
                        apply_error = f"scp_living_applied:{(scp2.stderr or scp2.stdout)[:300]}"
                    else:
                        # Soft living prove: binary + promotions pin present (no daemon restart)
                        prove = subprocess.run(
                            [
                                "ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", living_host,
                                f"bash -lc 'test -x /opt/gzmo/current/target/release/gzmo && "
                                f"test -f {living_promotions}/living-applied.json && "
                                f"test -f {living_promotions}/current/{baseline_src.name} && "
                                f"echo living_apply_prove_ok'",
                            ],
                            capture_output=True, text=True,
                        )
                        if prove.returncode != 0 or "living_apply_prove_ok" not in (prove.stdout or ""):
                            ok = False
                            apply_error = f"living_prove:{(prove.stderr or prove.stdout)[:300]}"
                        else:
                            applied = living_applied
                            rollback = {
                                "local_package": str(pkg),
                                "remote_rollback": f"{living_promotions}/rollback",
                                "clear_pin": f"rm {living_promotions}/living-applied.json",
                            }

stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
if apply and applied and ok:
    advice = (
        f"promote_loop_living_applied — {loop} pinned on {living_host} "
        f"(baseline={baseline_id}); soak after overnight BF GREEN"
    )
    next_steps = [
        "bash scripts/living-host-mutex.sh release",
        "bash scripts/brain-feed-check.sh",
        "bash scripts/ct101-living-probe.sh",
        "After overnight: re-run BF + living probe; then soak promote-loop-living-apply bet",
    ]
elif apply and apply_error:
    advice = f"promote_loop_apply_failed — {apply_error}"
    next_steps = [
        "Fix mutex/dual-writer/beat-gate, then retry PROMOTE_APPLY=1",
        "Whole-host still needs CUTOVER_APPROVED=1",
    ]
elif ok:
    advice = (
        f"promote_loop_record_ok — review artifact; "
        f"PROMOTE_APPLY=1 allowed for {sorted(apply_loops)}"
    )
    next_steps = [
        "bash scripts/living-host-mutex.sh claim --host ct101 --note 'promote-apply knowledge'",
        "PROMOTE_LOOP=knowledge PROMOTE_ACK=1 PROMOTE_APPLY=1 bash scripts/promote-loop.sh",
        "bash scripts/living-host-mutex.sh release",
    ]
else:
    advice = "promote_loop_blocked — need gate_passed+baseline_id and dual_writer_risk!=true"
    next_steps = [
        "Fix beat-gate fixture for loop",
        "bash scripts/living-host-mutex.sh status",
    ]

payload = {
    "schema": "gzmo.promote_loop/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "loop": loop,
    "mode": mode,
    "ack": True,
    "apply": apply,
    "beats_incumbent": beats,
    "gate_passed": gate,
    "baseline_id": baseline_id,
    "baseline_path": baseline_path,
    "mutex": {
        "dual_writer_risk": dual,
        "claim_host": claim_host,
        "claim_note": claim.get("note"),
    },
    "meta": str(meta_path),
    "applied": applied,
    "rollback": rollback,
    "apply_error": apply_error,
    "advice": advice,
    "next": next_steps,
}
path = out / f"promote-{loop}-{stamp}.json"
path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    f"# Promote-loop — {loop}",
    "",
    f"Verdict: **{'OK' if ok else 'BLOCKED'}**",
    "",
    f"- gate_passed: `{gate}`",
    f"- baseline_id: `{baseline_id}`",
    f"- dual_writer_risk: `{dual}`",
    f"- claim_host: `{claim_host}`",
    f"- mode: `{mode}`",
    "",
]
if applied:
    md += [
        "## Living apply",
        "",
        f"- host pin: `{living_host}:{living_promotions}/living-applied.json`",
        f"- recipe: `{applied.get('recipe')}`",
        f"- rollback: `{rollback}`",
        "",
    ]
if apply_error:
    md += [f"**Apply error:** {apply_error}", ""]
md += [advice, ""]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"ok": ok, "path": str(path), "advice": advice, "mode": mode}, indent=2))
raise SystemExit(0 if ok else 1)
PY
