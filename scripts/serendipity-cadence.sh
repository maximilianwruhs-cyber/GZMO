#!/usr/bin/env bash
# Brain Feed — serendipity apply cadence (cron-friendly, no auto-apply).
# Runs digest + promote dry-run, appends cadence log, sets reminder advice.
# Human apply: SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh
#
#   bash scripts/serendipity-cadence.sh
#   SERENDIPITY_CADENCE_MAX_AGE_HOURS=168 bash scripts/serendipity-cadence.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/serendipity"
LOG_JSONL="$OUT/cadence-log.jsonl"
MAX_AGE_H="${SERENDIPITY_CADENCE_MAX_AGE_HOURS:-168}"
mkdir -p "$OUT"

# Soft digest (lab spark / dreams if present)
if [[ -x "$ROOT/scripts/serendipity-digest.sh" ]]; then
  bash "$ROOT/scripts/serendipity-digest.sh" >/dev/null 2>&1 || true
fi

set +e
bash "$ROOT/scripts/serendipity-promote.sh"
promote_rc=$?
set -e

export OUT LOG_JSONL MAX_AGE_H promote_rc ROOT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone, timedelta
from pathlib import Path

out = Path(os.environ["OUT"])
log_path = Path(os.environ["LOG_JSONL"])
max_age_h = int(os.environ["MAX_AGE_H"])
now = datetime.now(timezone.utc)

promote = {}
p = out / "promote-latest.json"
if p.is_file():
    try:
        promote = json.loads(p.read_text(encoding="utf-8"))
    except Exception as e:
        promote = {"ok": False, "parse_error": str(e)}

# Prior applies from cadence log + this promote
applied_total = 0
last_apply_at = None
rows = []
if log_path.is_file():
    for line in log_path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            rows.append(json.loads(line))
        except Exception:
            pass
for r in rows:
    applied_total += int(r.get("applied_count") or 0)
    if r.get("had_apply") and r.get("generated_at"):
        last_apply_at = r["generated_at"]

this_applied = len(promote.get("applied") or [])
had_apply = this_applied > 0 or bool(promote.get("apply_error") is None and not promote.get("dry_run", True) and this_applied)
# dry-run path: count prior only
if promote.get("dry_run", True):
    had_apply = False
    this_applied = 0
else:
    had_apply = this_applied > 0
    applied_total += this_applied
    if had_apply:
        last_apply_at = now.isoformat()

candidates = int(promote.get("candidate_count") or 0)
stale = True
if last_apply_at:
    try:
        ts = datetime.fromisoformat(last_apply_at.replace("Z", "+00:00"))
        stale = (now - ts) > timedelta(hours=max_age_h)
    except Exception:
        stale = True
elif applied_total == 0:
    stale = True

# Advice
if promote.get("dual_writer"):
    advice = "serendipity_cadence_hold — dual_writer; stop gzmo-serve"
    ok = False
elif candidates == 0:
    advice = "serendipity_cadence_hold — 0 candidates; wait for living spark"
    ok = True
elif stale and candidates > 0:
    advice = (
        f"serendipity_cadence_remind — {candidates} candidates; "
        "human gate: SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh "
        f"(no apply in last {max_age_h}h)"
    )
    ok = True
elif applied_total > 0 and not stale:
    advice = f"serendipity_cadence_ok — applies_logged={applied_total}; dry-run candidates={candidates}"
    ok = True
else:
    advice = f"serendipity_cadence_ok — dry-run candidates={candidates}; apply when clear (≤3)"
    ok = True

entry = {
    "generated_at": now.isoformat(),
    "dry_run": bool(promote.get("dry_run", True)),
    "candidate_count": candidates,
    "applied_count": this_applied,
    "had_apply": had_apply,
    "promote_rc": int(os.environ.get("promote_rc") or 0),
    "advice": advice,
}
with log_path.open("a", encoding="utf-8") as f:
    f.write(json.dumps(entry, separators=(",", ":")) + "\n")

checklist = [
    "1. After spark night (or when candidates>0): bash scripts/serendipity-cadence.sh",
    "2. Review data-next/serendipity/promote-latest.json takeaways (≤3)",
    "3. If clear: SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh",
    "4. Confirm dual-writer still inactive; brain-feed-check stays GREEN",
]
payload = {
    "schema": "gzmo.brain_feed.serendipity_cadence/v1",
    "generated_at": now.isoformat(),
    "ok": ok and bool(promote.get("ok", True)),
    "advice": advice,
    "candidates": candidates,
    "applied_total_logged": applied_total + (0 if promote.get("dry_run", True) else 0),
    "applied_total": applied_total if not promote.get("dry_run", True) else applied_total,
    "last_apply_at": last_apply_at,
    "stale_apply": stale,
    "max_age_hours": max_age_h,
    "checklist": checklist,
    "promote_path": str(p) if p.is_file() else None,
    "cadence_log": str(log_path),
    "auto_apply": False,
    "doc": "docs/BRAIN_FEED.md",
}
# Fix applied_total: recount from full log after append
applied_total = 0
last_apply_at = None
for line in log_path.read_text(encoding="utf-8").splitlines():
    if not line.strip():
        continue
    try:
        r = json.loads(line)
    except Exception:
        continue
    applied_total += int(r.get("applied_count") or 0)
    if r.get("had_apply") and r.get("generated_at"):
        last_apply_at = r["generated_at"]
payload["applied_total"] = applied_total
payload["last_apply_at"] = last_apply_at
# Honest HOLD path satisfies bet if never junk-applied
payload["honest_hold_ok"] = applied_total == 0 and candidates >= 0 and ok

(out / "cadence-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Serendipity cadence",
    "",
    f"Advice: **{advice}**",
    "",
    f"- Candidates: {candidates}",
    f"- Applies logged: {applied_total}",
    f"- Last apply: {last_apply_at or 'never'}",
    f"- Auto-apply: **false**",
    "",
    "## Checklist",
    "",
]
md += [f"- {c}" for c in checklist]
md += ["", "See docs/BRAIN_FEED.md", ""]
(out / "cadence-latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"ok": payload["ok"], "advice": advice, "candidates": candidates, "applied_total": applied_total}, indent=2))
raise SystemExit(0 if payload["ok"] else 1)
PY
