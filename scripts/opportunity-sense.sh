#!/usr/bin/env bash
# Opportunity discovery — Sense: gather gate signals + bet-log snapshot.
#   bash scripts/opportunity-sense.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/opportunity-discovery"
OPP="$ROOT/research/opportunities"
mkdir -p "$OUT"

export ROOT DATA OUT OPP
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
import sys

sys.path.insert(0, str(Path(os.environ["ROOT"]) / "scripts"))
from opportunity_lib import load_bets, compute_score

root = Path(os.environ["ROOT"])
data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])
opp = Path(os.environ["OPP"])

def load_json(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None

signals = {
    "keep_quality": load_json(data / "keep-quality" / "latest.json"),
    "brain_feed": load_json(data / "brain-feed" / "latest.json"),
    "brain_intel": load_json(data / "brain-intel" / "latest.json"),
    "living_readiness": load_json(data / "living-readiness" / "latest.json"),
}

bets = load_bets(opp)
bet_summary = []
for b in bets:
    bet_summary.append({
        "id": b.get("id"),
        "status": b.get("status"),
        "score": compute_score(b),
        "title": b.get("title"),
    })

active = [b for b in bet_summary if b.get("status") == "active"]
horizon = [b for b in bet_summary if b.get("status") == "horizon"]

scars = []
kq = signals.get("keep_quality") or {}
if kq.get("verdict") == "RED":
    scars.append("keep_quality_RED")
bf = signals.get("brain_feed") or {}
if bf.get("verdict") == "RED":
    scars.append("brain_feed_RED")
if not active:
    scars.append("no_active_bet")
if len(active) > 1:
    scars.append("multiple_active_bets")

# Seed prompts from doctrine (not LLM)
prompts = [
    "What upgrade feeds the living vault without Cursor tourism?",
    "What only airgap overnight metabolism can do that Mem0/cloud memory cannot?",
    "What Brain Feed P0 gap still needs a ship slice?",
]

payload = {
    "schema": "gzmo.opportunity.sense/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "signals": {
        "keep_quality_verdict": (kq or {}).get("verdict"),
        "brain_feed_verdict": (bf or {}).get("verdict"),
        "brain_intel_verdict": (signals.get("brain_intel") or {}).get("verdict"),
        "living_readiness_verdict": (signals.get("living_readiness") or {}).get("verdict"),
    },
    "bets": bet_summary,
    "active_count": len(active),
    "horizon_count": len(horizon),
    "scars": scars,
    "sense_prompts": prompts,
    "doc": "docs/OPPORTUNITY_DISCOVERY.md",
    "advice": "opportunity_sense_ok — run opportunity-rank.sh next",
}
(out / "sense-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest-sense.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"ok": True, "active": len(active), "bets": len(bet_summary), "scars": scars}, indent=2))
PY
