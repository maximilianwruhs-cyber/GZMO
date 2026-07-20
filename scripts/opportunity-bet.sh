#!/usr/bin/env bash
# Opportunity discovery — Bet: lock one active bet + emit mission card.
#   bash scripts/opportunity-bet.sh --from <id>
#   bash scripts/opportunity-bet.sh --dry-run --from <id>
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/opportunity-discovery"
OPP="$ROOT/research/opportunities"
FROM=""
DRY=0
for a in "$@"; do
  case "$a" in
    --dry-run) DRY=1 ;;
    --from) shift_mode=1 ;;
    *)
      if [[ "${shift_mode:-0}" == "1" ]]; then FROM="$a"; shift_mode=0
      elif [[ "$a" == --from=* ]]; then FROM="${a#--from=}"
      fi
      ;;
  esac
done
# re-parse simply
FROM=""
DRY=0
args=("$@")
i=0
while (( i < ${#args[@]} )); do
  case "${args[$i]}" in
    --dry-run) DRY=1 ;;
    --from)
      i=$((i + 1))
      FROM="${args[$i]:-}"
      ;;
  esac
  i=$((i + 1))
done

[[ -n "$FROM" ]] || { echo "usage: opportunity-bet.sh --from <id> [--dry-run]" >&2; exit 2; }
mkdir -p "$OUT"

export ROOT DATA OUT OPP FROM DRY
python3 - <<'PY'
import json, os, re, sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(os.environ["ROOT"]) / "scripts"))
from opportunity_lib import load_bets, compute_score, ship_bar

opp = Path(os.environ["OPP"])
out = Path(os.environ["OUT"])
bet_id = os.environ["FROM"]
dry = os.environ.get("DRY", "0") == "1"
today = datetime.now(timezone.utc).strftime("%Y-%m-%d")

bets = load_bets(opp)
target = next((b for b in bets if b.get("id") == bet_id), None)
if not target:
    print(json.dumps({"ok": False, "error": f"unknown_bet:{bet_id}"}))
    raise SystemExit(1)
if target.get("status") == "horizon":
    print(json.dumps({"ok": False, "error": "cannot_activate_horizon_bet"}))
    raise SystemExit(1)
if not ship_bar(target) and target.get("status") != "active":
    print(json.dumps({
        "ok": False,
        "error": "below_ship_bar",
        "score": compute_score(target),
        "hint": "need score>=18, brain_profit>=3, usp_fit>=4",
    }, indent=2))
    raise SystemExit(1)

changed = []
if not dry:
    for b in bets:
        path = Path(b["path"])
        text = path.read_text(encoding="utf-8")
        status = b.get("status")
        new_status = status
        if b.get("id") == bet_id:
            new_status = "active"
        elif status == "active":
            new_status = "candidate"
        if new_status != status:
            def repl_status(m):
                return f"status: {new_status}"
            text2, n = re.subn(r"(?m)^status:\s*\S+", f"status: {new_status}", text, count=1)
            text2, n2 = re.subn(r"(?m)^updated:\s*\S+", f"updated: {today}", text2, count=1)
            path.write_text(text2, encoding="utf-8")
            changed.append({"id": b.get("id"), "from": status, "to": new_status})

# Mission card
card = f"""# Mission card — {target.get('title')}

## Mission

**Bet id:** `{bet_id}`
**Title:** {target.get('title')}
**Score:** {compute_score(target)}
**Why rare:** See `research/opportunities/{bet_id}.md`
**Brain profit:** axis brain_profit={target.get('brain_profit')}
**Done when:** Exit criteria in the bet file.

## Constraints

- USP: airgap living (ADR-0004); Brain Feed nutrients preferred
- One overnight writer (ADR-0003)
- No local-intel quests; no Socratic tourism; no public webserver SKU
- Finish-through: implement → verify → commit → push → PR → CI green → stop with PR URL or blocker

## Verify

```bash
bash scripts/opportunity-discovery-check.sh
bash scripts/brain-feed-check.sh
```

## Bet file

`research/opportunities/{bet_id}.md`
"""
card_path = out / f"mission-{bet_id}.md"
card_path.write_text(card, encoding="utf-8")
(out / "mission-latest.md").write_text(card, encoding="utf-8")

payload = {
    "schema": "gzmo.opportunity.bet/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "dry_run": dry,
    "bet_id": bet_id,
    "ship_bar": ship_bar(target),
    "score": compute_score(target),
    "changed": changed,
    "mission_card": str(card_path),
    "advice": (
        "opportunity_bet_dry_run_ok"
        if dry
        else f"opportunity_bet_active — {bet_id}; paste mission-latest.md into agent kickoff"
    ),
}
(out / "bet-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps(payload, indent=2))
PY
