#!/usr/bin/env bash
# Spine demo — stranger-5min proof of the two product pillars.
# 1) Product memory MCP cold path  2) Lab recall-proof / metabolism signal
# Does not start overnight serve; does not touch CT101 vault.
#
#   bash scripts/spine-demo.sh
#   SKIP_PRODUCT_MCP=1 bash scripts/spine-demo.sh   # status-only if binary missing
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/spine-demo"
BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
mkdir -p "$OUT"

PRODUCT_OK=0
PRODUCT_NOTE="skipped"
PRODUCT_LOG="$OUT/product-mcp.log"
if [[ "${SKIP_PRODUCT_MCP:-0}" != "1" && -x "$BIN" ]]; then
  if KEEP_VERIFY_DIR=1 VERIFY_DIR="$OUT/product-verify" \
    GZMO_BIN="$BIN" bash "$ROOT/scripts/verify-product-mcp.sh" >"$PRODUCT_LOG" 2>&1; then
    PRODUCT_OK=1
    PRODUCT_NOTE="verify-product-mcp PASS"
  else
    PRODUCT_OK=0
    PRODUCT_NOTE="verify-product-mcp FAIL (see product-mcp.log)"
  fi
elif [[ ! -x "$BIN" ]]; then
  PRODUCT_NOTE="no gzmo binary — set GZMO_BIN or build release"
else
  PRODUCT_NOTE="SKIP_PRODUCT_MCP=1"
fi

# Soft CT101 owner probe (SSH may HOLD; still records dual-writer risk).
bash "$ROOT/scripts/ct101-living-probe.sh" >/dev/null 2>&1 || true

# Keep human loop: takeaway → distill enqueue (no --now).
bash "$ROOT/scripts/takeaway-ritual-lab.sh" >/dev/null 2>&1 || true

# Soft faithfulness (fixture mode — offline, no vault dependency).
FAITH_LOG="$OUT/faithfulness-fixture.log"
if FAITHFULNESS_MODE=fixture bash "$ROOT/scripts/faithfulness-ci.sh" >"$FAITH_LOG" 2>&1; then
  FAITH_OK=1
else
  FAITH_OK=0
fi

# Soft prime-product living signals (may HOLD if SSH down).
bash "$ROOT/scripts/product-stranger-path.sh" >/dev/null 2>&1 || true
# Skip heavy living distill inside spine-demo by default (run ct101-takeaway-recall.sh alone).
if [[ "${SPINE_LIVING_HEAVY:-0}" == "1" ]]; then
  bash "$ROOT/scripts/ct101-takeaway-recall.sh" >/dev/null 2>&1 || true
fi
bash "$ROOT/scripts/faithfulness-living.sh" >/dev/null 2>&1 || true

export ROOT DATA OUT BIN PRODUCT_OK PRODUCT_NOTE FAITH_OK
python3 - <<'PY'
import json, os, re
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()
product_ok = os.environ.get("PRODUCT_OK") == "1"
product_note = os.environ.get("PRODUCT_NOTE", "")

recall_path = data / "recall-proof.md"
recall = {"present": recall_path.is_file(), "path": str(recall_path)}
if recall_path.is_file():
    text = recall_path.read_text(encoding="utf-8", errors="replace")
    m = re.search(r"(\d+)/(\d+)\s*HIT", text)
    if m:
        recall["hits"] = int(m.group(1))
        recall["total"] = int(m.group(2))
        recall["pct"] = round(100.0 * int(m.group(1)) / max(1, int(m.group(2))), 1)
    recall["verdict_line"] = next(
        (ln.strip() for ln in text.splitlines() if "HIT" in ln and "%" in ln),
        None,
    )

watchdog = {}
wp = data / "scheduler-runs" / "latest-watchdog.json"
try:
    watchdog = json.loads(wp.read_text(encoding="utf-8"))
except Exception:
    pass

ct101 = {}
try:
    ct101 = json.loads((data / "ct101-living" / "latest.json").read_text(encoding="utf-8"))
except Exception:
    pass

takeaway = {}
try:
    takeaway = json.loads((data / "takeaway-ritual" / "latest.json").read_text(encoding="utf-8"))
except Exception:
    pass

faith_ok = os.environ.get("FAITH_OK") == "1"
faith = {}
try:
    faith = json.loads((data / "faithfulness" / "latest.json").read_text(encoding="utf-8"))
except Exception:
    pass

stranger = {}
try:
    stranger = json.loads((data / "product-stranger" / "latest.json").read_text(encoding="utf-8"))
except Exception:
    pass
living_takeaway = {}
try:
    living_takeaway = json.loads((data / "ct101-takeaway-recall" / "latest.json").read_text(encoding="utf-8"))
except Exception:
    pass
living_faith = {}
try:
    living_faith = json.loads((data / "faithfulness-living" / "latest.json").read_text(encoding="utf-8"))
except Exception:
    pass

owner = {
    "living_production": "CT101 (/opt/gzmo/, gzmo-daemon)",
    "lab_scratch": "workstation data-next/",
    "rule": "Never overnight gzmo serve on workstation while CT101 lives",
    "doc": "docs/CT101_BOUNDARY.md",
    "probe_advice": ct101.get("advice"),
    "living_proof": ct101.get("living_proof"),
    "dual_writer_risk": (ct101.get("workstation") or {}).get("dual_writer_risk"),
}

pillars = {
    "metabolism": {
        "name": "Living overnight memory metabolism",
        "lab_proof": recall,
        "watchdog_stale": watchdog.get("stale"),
        "ok": bool(recall.get("present") and recall.get("hits", 0) >= max(1, int(0.8 * recall.get("total", 1)))),
    },
    "product_mcp": {
        "name": "Product memory MCP appliance",
        "ok": product_ok,
        "note": product_note,
        "verify": "scripts/verify-product-mcp.sh",
    },
}

supports = {
    "takeaway_ritual": {
        "ok": bool(takeaway.get("ritual_ok")),
        "advice": takeaway.get("advice"),
        "session_id": takeaway.get("session_id"),
    },
    "faithfulness_fixture": {
        "ok": faith_ok,
        "verdict": faith.get("verdict") or faith.get("ok"),
        "mode": faith.get("mode") or "fixture",
    },
    "product_stranger": {
        "ok": bool(stranger.get("ok")),
        "advice": stranger.get("advice"),
    },
    "ct101_takeaway_recall": {
        "ok": bool(living_takeaway.get("living_proof")),
        "advice": living_takeaway.get("advice"),
    },
    "faithfulness_living": {
        "ok": bool(living_faith.get("living_ok")),
        "advice": living_faith.get("advice"),
        "supported": living_faith.get("supported"),
        "total": living_faith.get("total"),
    },
}

demo_ok = pillars["metabolism"]["ok"] and pillars["product_mcp"]["ok"]
support_ok = supports["takeaway_ritual"]["ok"] and supports["faithfulness_fixture"]["ok"]
advice = (
    "demable — both pillars green on this host"
    if demo_ok and support_ok
    else (
        "demable_pillars — Keep supports partial (takeaway/faithfulness)"
        if demo_ok
        else (
            "partial — strengthen failing pillar before packaging AOS/CE"
            if pillars["metabolism"]["ok"] or pillars["product_mcp"]["ok"]
            else "hold — neither pillar demable yet"
        )
    )
)

payload = {
    "schema": "gzmo.spine.demo/v1",
    "generated_at": now,
    "ok": demo_ok,
    "advice": advice,
    "owner": owner,
    "pillars": pillars,
    "supports": supports,
    "parked": [
        "Arena / € / RAPL deepen",
        "HSP event bus",
        "AOS CE packaging / edge fleet",
        "Cognis / escape-loop / portable rewrite",
        "OKCP marketplace polish",
    ],
    "keep": [
        "felt-recall + watchdog",
        "session takeaway → distill",
        "product MCP verify",
        "faithfulness CI / concept-gate for wiki quality",
    ],
    "note": "Spine focus spike — stop expanding the zoo; demo the two pillars.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

def yn(ok):
    return "PASS" if ok else "HOLD"

md = [
    "# Spine demo",
    "",
    f"Generated: {now}",
    f"Advice: **{advice}**",
    "",
    "## Living vault owner",
    "",
    f"- Production: {owner['living_production']}",
    f"- Lab: {owner['lab_scratch']}",
    f"- Rule: {owner['rule']}",
    f"- Probe: {owner.get('probe_advice') or 'n/a'}",
    f"- Living proof: {owner.get('living_proof')}",
    f"- Dual-writer risk: {owner.get('dual_writer_risk')}",
    "",
    "## Pillars",
    "",
    f"- Metabolism (lab recall-proof): **{yn(pillars['metabolism']['ok'])}**"
    + (
        f" — {recall.get('hits')}/{recall.get('total')} HIT"
        if recall.get("hits") is not None
        else ""
    ),
    f"- Product MCP: **{yn(pillars['product_mcp']['ok'])}** — {product_note}",
    "",
    "## Keep supports",
    "",
    f"- Takeaway ritual: **{yn(supports['takeaway_ritual']['ok'])}** — {supports['takeaway_ritual'].get('advice')}",
    f"- Faithfulness (fixture): **{yn(supports['faithfulness_fixture']['ok'])}**",
    f"- Product stranger: **{yn(supports['product_stranger']['ok'])}** — {supports['product_stranger'].get('advice')}",
    f"- CT101 takeaway→recall: **{yn(supports['ct101_takeaway_recall']['ok'])}** — {supports['ct101_takeaway_recall'].get('advice')}",
    f"- Faithfulness living: **{yn(supports['faithfulness_living']['ok'])}** — {supports['faithfulness_living'].get('advice')}",
    "",
    "## Keep (strengthen)",
    "",
]
for k in payload["keep"]:
    md.append(f"- {k}")
md += ["", "## Park (do not expand)", ""]
for k in payload["parked"]:
    md.append(f"- {k}")
md += ["", payload["note"], ""]
(out / "latest.md").write_text("\n".join(md), encoding="utf-8")
print(json.dumps({
    "ok": demo_ok,
    "advice": advice,
    "pillars": {
        "metabolism": pillars["metabolism"]["ok"],
        "product_mcp": pillars["product_mcp"]["ok"],
    },
    "supports": {
        "takeaway_ritual": supports["takeaway_ritual"]["ok"],
        "faithfulness_fixture": supports["faithfulness_fixture"]["ok"],
    },
}, indent=2))
PY

# Soft exit 0 for nightburst; ok flag is in JSON.
exit 0
