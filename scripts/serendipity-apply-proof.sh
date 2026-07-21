#!/usr/bin/env bash
# Serendipity apply proof — one human-gated promote that closes the 0-apply remind.
# Default: dry-run + report. Apply only with --apply or SERENDIPITY_APPLY_PROOF=1.
# Never changes serendipity-cadence auto_apply (stays false).
#
#   bash scripts/serendipity-apply-proof.sh
#   bash scripts/serendipity-apply-proof.sh --apply
#   SERENDIPITY_APPLY_PROOF=1 bash scripts/serendipity-apply-proof.sh
#
# Artifact: data-next/serendipity/apply-proof-latest.{json,md}
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/serendipity"
APPLY=0
if [[ "${SERENDIPITY_APPLY_PROOF:-0}" == "1" ]]; then
  APPLY=1
fi
for a in "$@"; do
  case "$a" in
    --apply) APPLY=1 ;;
    -h|--help)
      echo "Usage: $0 [--apply]"
      exit 0
      ;;
  esac
done
mkdir -p "$OUT"

echo "=== Serendipity apply proof (apply=$APPLY) ==="

# Always dry-run first for review artifact
bash "$ROOT/scripts/serendipity-promote.sh"
DRY_JSON="$OUT/promote-latest.json"
cands=0
if [[ -f "$DRY_JSON" ]]; then
  cands="$(python3 -c "import json;print(json.load(open('$DRY_JSON')).get('candidate_count',0))")"
fi

apply_ran=0
apply_ok=0
if [[ "$APPLY" == "1" ]]; then
  if [[ "${cands:-0}" == "0" ]]; then
    echo "[!] No candidates — refuse apply" >&2
  else
    apply_ran=1
    set +e
    SERENDIPITY_PROMOTE_APPLY=1 bash "$ROOT/scripts/serendipity-promote.sh"
    apply_rc=$?
    set -e
    if [[ $apply_rc -eq 0 ]] && python3 -c "
import json
d=json.load(open('$OUT/promote-latest.json'))
raise SystemExit(0 if (not d.get('dry_run') and d.get('ok') and d.get('applied')) else 1)
"; then
      apply_ok=1
    fi
  fi
fi

# Refresh cadence so dry-run reminder stays current (does not wipe proof artifact)
bash "$ROOT/scripts/serendipity-cadence.sh" >/dev/null 2>&1 || true

export OUT APPLY apply_ran apply_ok cands ROOT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
apply = os.environ.get("APPLY") == "1"
apply_ran = os.environ.get("apply_ran") == "1"
apply_ok = os.environ.get("apply_ok") == "1"
cands = int(os.environ.get("cands") or 0)
now = datetime.now(timezone.utc)
now_iso = now.isoformat()

promote = {}
if (out / "promote-latest.json").is_file():
    promote = json.loads((out / "promote-latest.json").read_text(encoding="utf-8"))

# After a successful apply, cadence's follow-up dry-run overwrites promote-latest.
# Prefer dedicated apply stamp / proof memory; also append cadence-log so reminders clear.
log_path = out / "cadence-log.jsonl"
applied_from_proof = []
if apply_ok:
    # Recover applied list from newest promote-* with applied[] if latest is dry
    applied_from_proof = list(promote.get("applied") or [])
    if not applied_from_proof:
        stamps = sorted(out.glob("promote-*.json"), reverse=True)
        for sp in stamps[:8]:
            try:
                d = json.loads(sp.read_text(encoding="utf-8"))
            except Exception:
                continue
            if d.get("applied"):
                applied_from_proof = list(d["applied"])
                promote = d
                break
    entry = {
        "generated_at": now_iso,
        "dry_run": False,
        "candidate_count": cands,
        "applied_count": len(applied_from_proof) or 1,
        "had_apply": True,
        "source": "serendipity-apply-proof",
        "advice": "serendipity_apply_proof_logged",
    }
    with log_path.open("a", encoding="utf-8") as f:
        f.write(json.dumps(entry, separators=(",", ":")) + "\n")

cadence = {}
if (out / "cadence-latest.json").is_file():
    cadence = json.loads((out / "cadence-latest.json").read_text(encoding="utf-8"))

# Recount log
applied_total = 0
last_apply = None
if log_path.is_file():
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            r = json.loads(line)
        except Exception:
            continue
        applied_total += int(r.get("applied_count") or 0)
        if r.get("had_apply") and r.get("generated_at"):
            last_apply = r["generated_at"]

applied = applied_from_proof or (promote.get("applied") or [])
stale = applied_total == 0

if apply and apply_ok and (applied or applied_total > 0):
    advice = (
        f"serendipity_apply_proof_ok — applied={max(len(applied), 1)}; "
        f"cadence_applies_logged={applied_total}; auto_apply=false"
    )
    ok = True
    verdict = "GREEN"
elif apply and apply_ran and not apply_ok:
    advice = f"serendipity_apply_proof_fail — {promote.get('apply_error') or promote.get('advice')}"
    ok = False
    verdict = "RED"
elif apply and cands == 0:
    advice = "serendipity_apply_proof_hold — 0 candidates; wait for spark"
    ok = True
    verdict = "HOLD"
elif not apply:
    advice = (
        f"serendipity_apply_proof_dry — candidates={cands}; "
        "re-run with --apply or SERENDIPITY_APPLY_PROOF=1 when clear"
    )
    ok = True
    verdict = "HOLD"
else:
    advice = "serendipity_apply_proof_hold"
    ok = True
    verdict = "HOLD"

# Refresh cadence-latest stale flag after log append
if (out / "cadence-latest.json").is_file() and apply_ok:
    cadence["applied_total"] = applied_total
    cadence["last_apply_at"] = last_apply
    cadence["stale_apply"] = False
    cadence["advice"] = (
        f"serendipity_cadence_ok — applies_logged={applied_total}; "
        f"dry-run candidates={int(cadence.get('candidates') or cands)}"
    )
    (out / "cadence-latest.json").write_text(json.dumps(cadence, indent=2) + "\n", encoding="utf-8")

payload = {
    "schema": "gzmo.brain_feed.serendipity_apply_proof/v1",
    "generated_at": now_iso,
    "verdict": verdict,
    "ok": ok,
    "advice": advice,
    "apply_requested": apply,
    "apply_ran": apply_ran,
    "apply_ok": apply_ok,
    "candidate_count": cands,
    "applied": applied,
    "applied_total_logged": applied_total,
    "last_apply_at": last_apply,
    "stale_apply": stale and not apply_ok,
    "auto_apply": False,
    "cadence_log": str(log_path),
    "promote_latest": str(out / "promote-latest.json"),
    "doc": "docs/BRAIN_FEED.md",
    "operator": [
        "Default is dry-run only — never auto-apply from cadence",
        "bash scripts/serendipity-apply-proof.sh --apply  # ≤3 takeaways",
        "Or: SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh",
    ],
}
(out / "apply-proof-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "apply-proof-latest.md").write_text(
    "\n".join([
        "# Serendipity apply proof",
        "",
        f"Verdict: **{verdict}**",
        f"- Advice: {advice}",
        f"- Auto-apply: **false**",
        f"- Last apply: {last_apply or 'never'}",
        f"- Applies logged: {applied_total}",
        "",
        "See docs/BRAIN_FEED.md",
        "",
    ]) + "\n",
    encoding="utf-8",
)
print(json.dumps({
    "verdict": verdict,
    "ok": ok,
    "advice": advice,
    "apply_ok": apply_ok,
    "candidates": cands,
    "applied_count": len(applied),
    "applied_total_logged": applied_total,
    "last_apply_at": last_apply,
    "auto_apply": False,
}, indent=2))
raise SystemExit(0 if ok else 1)
PY
