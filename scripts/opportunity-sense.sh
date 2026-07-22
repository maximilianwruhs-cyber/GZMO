#!/usr/bin/env bash
# Opportunity discovery — Sense: gate signals + bet-log + nutrient-depth scars + STACK gaps.
#   bash scripts/opportunity-sense.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/opportunity-discovery"
OPP="$ROOT/research/opportunities"
HOST="${CT101_SSH_HOST:-ct101}"
VAULT_DB="${KEEP_QUALITY_VAULT_DB:-/opt/gzmo/data/vault.db}"
mkdir -p "$OUT"

export ROOT DATA OUT OPP HOST VAULT_DB
python3 - <<'PY'
import json, os, re, subprocess
from datetime import datetime, timezone
from pathlib import Path
import sys

sys.path.insert(0, str(Path(os.environ["ROOT"]) / "scripts"))
from opportunity_lib import load_bets, compute_score

root = Path(os.environ["ROOT"])
data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])
opp = Path(os.environ["OPP"])
host = os.environ.get("HOST", "ct101")
vault_db = os.environ.get("VAULT_DB", "/opt/gzmo/data/vault.db")

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
    "serendipity_cadence": load_json(data / "serendipity" / "cadence-latest.json"),
}

bets = load_bets(opp)
bet_summary = []
covered_stack = set()
for b in bets:
    bet_summary.append({
        "id": b.get("id"),
        "status": b.get("status"),
        "score": compute_score(b),
        "title": b.get("title"),
        "stack_ids": b.get("stack_ids") or [],
    })
    for sid in b.get("stack_ids") or []:
        covered_stack.add(sid)

active = [b for b in bet_summary if b.get("status") == "active"]
horizon = [b for b in bet_summary if b.get("status") == "horizon"]
candidates = [b for b in bet_summary if b.get("status") == "candidate"]

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
if not candidates and not active:
    scars.append("bet_log_starved — Sense found no candidates; add bets or deepen nutrient scars")

# Nutrient-depth: Felt Use / ripen floor on living vault
felt = {"ok": False}
try:
    raw = subprocess.run(
        [
            "ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host,
            f"sqlite3 {vault_db} "
            "\"SELECT COUNT(*) FROM honeypot WHERE is_latest=1; "
            "SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>=1; "
            "SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>=3;\"",
        ],
        capture_output=True, text=True, timeout=20,
    )
    lines = [ln.strip() for ln in (raw.stdout or "").splitlines() if ln.strip().isdigit()]
    if len(lines) >= 3:
        latest, ge1, ge3 = int(lines[0]), int(lines[1]), int(lines[2])
        share3_felt = (ge3 / ge1) if ge1 else 0.0
        share3_latest = (ge3 / latest) if latest else 0.0
        felt = {
            "ok": True,
            "latest": latest,
            "recall_ge1": ge1,
            "recall_ge3": ge3,
            "share_ge3": round(share3_felt, 6),
            "share_ge3_of_latest": round(share3_latest, 6),
            "share_denominator": "recall_ge1",
        }
        # Soft scar: depth among felt facts (ripen needs ≥3)
        if ge3 < 100 or share3_felt < 0.40:
            scars.append(
                f"felt_use_depth_thin — recall≥3={ge3}/{ge1} felt "
                "(ripen dual-gate needs deeper Felt Use; not memory-gym)"
            )
except Exception as e:
    felt = {"ok": False, "error": str(e)[:120]}
    scars.append("felt_use_census_unreachable — could not query living honeypot")

# Serendipity apply staleness
cad = signals.get("serendipity_cadence") or {}
if cad:
    advice = str(cad.get("advice") or "")
    if "no apply" in advice.lower() or "0 apply" in advice.lower() or "remind" in advice.lower():
        if not any(b.get("id") == "serendipity-apply-proof" and b.get("status") in ("active", "candidate") for b in bet_summary):
            scars.append("serendipity_apply_stale — cadence reminding with no recent human apply")
        else:
            scars.append("serendipity_apply_stale — candidate/active bet exists; still needs apply proof")

# Soak honesty: prefer newest GREEN pair; ignore ancient same-hour streaks once
# a later honest sample exists (≥12h from previous counted night).
soak_path = data / "keep-quality" / "soak-log.jsonl"
soak_gap_hours = None
if soak_path.is_file():
    try:
        from datetime import datetime as dt

        def parse(t):
            return dt.fromisoformat(str(t).replace("Z", "+00:00"))

        greens = []
        for line in soak_path.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            r = json.loads(line)
            if r.get("verdict") != "GREEN":
                continue
            t = r.get("generated_at") or r.get("ts") or r.get("timestamp")
            if t:
                greens.append(parse(t))
        greens.sort()
        # Honest nights: walk newest→oldest with ≥18h spacing (same as keep-quality-soak --summary)
        honest = 0
        anchor = None
        for ts in reversed(greens):
            if anchor is None:
                honest = 1
                anchor = ts
                continue
            gap_h = (anchor - ts).total_seconds() / 3600.0
            if gap_h >= 18.0:
                honest += 1
                anchor = ts
        if len(greens) >= 2:
            soak_gap_hours = round(
                abs((greens[-1] - greens[-2]).total_seconds()) / 3600.0, 2
            )
        # Scar only while honest nights incomplete AND the newest pair is too close
        if honest < 3 and soak_gap_hours is not None and soak_gap_hours < 12:
            scars.append(
                f"soak_samples_too_close — min_gap_h={soak_gap_hours} "
                f"(honest_nights={honest}/3; nights claim needs ≥18h spacing)"
            )
        elif honest < 3:
            scars.append(
                f"soak_nights_incomplete — honest_nights={honest}/3 "
                "(need ≥18h-spaced GREEN samples)"
            )
    except Exception:
        pass

# Living organ-trace + promote soak (organism pulse)
organ = load_json(data / "organ-trace" / "latest.json") or load_json(data / "organ-trace" / "living-latest.json")
if organ:
    missed = organ.get("missed_core") or []
    if missed:
        scars.append(f"organ_missed_core — {','.join(missed)}")
    signals["organ_trace_ok"] = organ.get("ok")
else:
    scars.append("organ_trace_missing — run organ-trace.sh --living")

promo_soak = load_json(data / "beat-gate" / "promotions" / "soak-latest.json")
if promo_soak:
    signals["promote_soak_verdict"] = promo_soak.get("verdict")
    if promo_soak.get("verdict") == "HOLD":
        scars.append("promote_loop_soak_hold — wait pin age / overnight GREEN")
    elif promo_soak.get("verdict") == "RED":
        scars.append("promote_loop_soak_RED — fix FAIL before ops-discovery promote")

sleep_b = load_json(data / "sleep-time" / "latest.json")
if sleep_b:
    signals["sleep_budget"] = sleep_b.get("budget")
    if sleep_b.get("budget") == "deep":
        scars.append("sleep_budget_deep — nutrient backlog high; prefer dream over tourism")

# STACK near/singular coverage gaps (nutrient-relevant ids only — no theater)
priority_stack = [
    ("m1", "Felt-recall / Felt Use depth"),
    ("s1", "herdr + metabolism living proof"),
    ("o1", "Living organ-trace (not workstation zoo)"),
    ("m4", "Dream compaction living soft"),
    ("r5", "Missed-run watchdog living"),
    ("o5", "Serendipity promote-back"),
]
stack_gaps = []
for sid, label in priority_stack:
    if sid not in covered_stack:
        stack_gaps.append({"stack_id": sid, "label": label})
        scars.append(f"stack_gap:{sid} — {label} not covered by any bet stack_ids")

# Seed prompts from doctrine + dynamic scars
prompts = [
    "What upgrade feeds the living vault without Cursor tourism?",
    "What only airgap overnight metabolism can do that Mem0/cloud memory cannot?",
    "What Brain Feed P0 gap still needs a ship slice?",
]
for s in scars[:6]:
    prompts.append(f"Scar-driven: {s}")

payload = {
    "schema": "gzmo.opportunity.sense/v2",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "signals": {
        "keep_quality_verdict": (kq or {}).get("verdict"),
        "brain_feed_verdict": (bf or {}).get("verdict"),
        "brain_intel_verdict": (signals.get("brain_intel") or {}).get("verdict"),
        "living_readiness_verdict": (signals.get("living_readiness") or {}).get("verdict"),
        "serendipity_cadence_advice": (cad or {}).get("advice"),
    },
    "felt_use_depth": felt,
    "soak_min_gap_hours": soak_gap_hours,
    "stack_gaps": stack_gaps,
    "bets": bet_summary,
    "active_count": len(active),
    "candidate_count": len(candidates),
    "horizon_count": len(horizon),
    "scars": scars,
    "sense_prompts": prompts,
    "doc": "docs/OPPORTUNITY_DISCOVERY.md",
    "advice": (
        "opportunity_sense_ok — run opportunity-rank.sh next"
        if candidates or active
        else "opportunity_sense_starved — add candidate bets from scars/stack_gaps"
    ),
}
(out / "sense-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest-sense.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Opportunity sense",
    "",
    f"Generated: {payload['generated_at']}",
    "",
    f"- Active: {len(active)} · Candidates: {len(candidates)} · Horizon: {len(horizon)}",
    f"- Felt Use depth: {felt}",
    f"- Soak min gap (h): {soak_gap_hours}",
    "",
    "## Scars",
    "",
]
for s in scars:
    md.append(f"- {s}")
md += ["", "## STACK gaps", ""]
if stack_gaps:
    for g in stack_gaps:
        md.append(f"- `{g['stack_id']}` — {g['label']}")
else:
    md.append("- (none among priority nutrient ids)")
md += ["", f"Advice: {payload['advice']}", ""]
(out / "sense-latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({
    "ok": True,
    "active": len(active),
    "candidates": len(candidates),
    "bets": len(bet_summary),
    "scars": scars,
    "felt_use_depth": felt,
    "stack_gaps": stack_gaps,
}, indent=2))
PY
