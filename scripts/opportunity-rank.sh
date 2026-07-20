#!/usr/bin/env bash
# Opportunity discovery — Rank: score bet log + ship-bar filter.
#   bash scripts/opportunity-rank.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/opportunity-discovery"
OPP="$ROOT/research/opportunities"
mkdir -p "$OUT"

# Refresh sense first (soft)
bash "$ROOT/scripts/opportunity-sense.sh" >/dev/null 2>&1 || true

export ROOT DATA OUT OPP
python3 - <<'PY'
import json, os, sys
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(os.environ["ROOT"]) / "scripts"))
from opportunity_lib import load_bets, compute_score, ship_bar

out = Path(os.environ["OUT"])
opp = Path(os.environ["OPP"])
bets = load_bets(opp)

rows = []
for b in bets:
    score = compute_score(b)
    rows.append({
        "id": b.get("id"),
        "title": b.get("title"),
        "status": b.get("status"),
        "score": score,
        "uniqueness": b.get("uniqueness"),
        "brain_profit": b.get("brain_profit"),
        "credit_cost": b.get("credit_cost"),
        "attention_cost": b.get("attention_cost"),
        "usp_fit": b.get("usp_fit"),
        "ship_bar": ship_bar(b) if b.get("status") != "horizon" else False,
        "path": b.get("path"),
    })

ranked = sorted(
    [r for r in rows if r["status"] != "horizon" and r["score"] is not None],
    key=lambda r: (-int(r["score"]), r["id"]),
)
horizon = [r for r in rows if r["status"] == "horizon"]
shipable = [r for r in ranked if r["ship_bar"] and r["status"] in ("candidate", "active")]

payload = {
    "schema": "gzmo.opportunity.rank/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "ranked": ranked,
    "horizon": horizon,
    "shipable": shipable,
    "top": shipable[0] if shipable else (ranked[0] if ranked else None),
    "advice": (
        f"opportunity_rank_ok — top shipable={shipable[0]['id']}"
        if shipable
        else "opportunity_rank_hold — no shipable bets (raise scores or add candidates)"
    ),
    "doc": "docs/OPPORTUNITY_DISCOVERY.md",
}
(out / "rank-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

md = ["# Opportunity rank", "", f"Generated: {payload['generated_at']}", "", "| Rank | Id | Score | Status | Ship bar | Title |", "|------|----|-------|--------|----------|-------|"]
for i, r in enumerate(ranked, 1):
    md.append(
        f"| {i} | `{r['id']}` | {r['score']} | {r['status']} | {'yes' if r['ship_bar'] else 'no'} | {r['title']} |"
    )
if horizon:
    md += ["", "## Horizon (not scored)", ""]
    for r in horizon:
        md.append(f"- `{r['id']}` — {r['title']}")
md += ["", payload["advice"], ""]
(out / "rank-latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"ok": True, "top": payload.get("top"), "shipable": len(shipable), "advice": payload["advice"]}, indent=2))
PY
