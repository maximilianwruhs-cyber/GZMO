#!/usr/bin/env bash
# Emit mission card for the single active opportunity bet (automation / cron entry).
# Does not start an agent by itself — writes data-next artifact + prints path.
#
#   bash scripts/opportunity-next-mission.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/opportunity-discovery"
OPP="$ROOT/research/opportunities"
mkdir -p "$OUT"

bash "$ROOT/scripts/opportunity-sense.sh" >/dev/null 2>&1 || true
bash "$ROOT/scripts/opportunity-rank.sh" >/dev/null 2>&1 || true

export ROOT OUT OPP
python3 - <<'PY'
import json, os, sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(os.environ["ROOT"]) / "scripts"))
from opportunity_lib import load_bets, compute_score, ship_bar

opp = Path(os.environ["OPP"])
out = Path(os.environ["OUT"])
bets = load_bets(opp)
active = [b for b in bets if b.get("status") == "active"]
if len(active) != 1:
    payload = {
        "schema": "gzmo.opportunity.next_mission/v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "ok": False,
        "advice": f"need_exactly_one_active_bet have={len(active)}",
    }
    (out / "next-mission.json").write_text(json.dumps(payload, indent=2) + "\n")
    print(json.dumps(payload, indent=2))
    raise SystemExit(1)

b = active[0]
bet_id = b["id"]
# Ensure bet script wrote a card (dry path ok)
import subprocess
subprocess.run(
    [str(Path(os.environ["ROOT"]) / "scripts" / "opportunity-bet.sh"), "--dry-run", "--from", bet_id],
    check=False,
    capture_output=True,
)
card_src = out / f"mission-{bet_id}.md"
card = card_src.read_text(encoding="utf-8") if card_src.is_file() else f"# Mission\n\nBet `{bet_id}`\n"
(out / "next-mission.md").write_text(card, encoding="utf-8")
payload = {
    "schema": "gzmo.opportunity.next_mission/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "bet_id": bet_id,
    "title": b.get("title"),
    "score": compute_score(b),
    "ship_bar": ship_bar(b),
    "mission_md": str(out / "next-mission.md"),
    "advice": "opportunity_next_mission_ok — paste next-mission.md into a new agent (or cron opens Cloud Agent with this file)",
    "automation_note": (
        "Cursor Automations cannot start from bet status alone until a cron/PR trigger "
        "runs an agent with this mission card. PR babysitter only runs after a PR exists."
    ),
}
(out / "next-mission.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps(payload, indent=2))
PY
