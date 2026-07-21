#!/usr/bin/env bash
# Felt Use depth census — living honeypot recall≥1 / recall≥3 for honest ripen.
# Does NOT run memory-gym searches. Side-effect measurement only.
#
#   bash scripts/felt-use-depth.sh
# Artifact: data-next/felt-use-depth/latest.{json,md}
#
# Env:
#   CT101_SSH_HOST / KEEP_QUALITY_VAULT_DB
#   FELT_USE_MIN_GE3          soft floor for recall≥3 count (default 100)
#   FELT_USE_MIN_SHARE_GE3    soft floor for deep share among felt facts
#                             (ge3/ge1; default 0.40). Not ge3/latest — that
#                             share shrinks as the vault grows and is unreachable.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/felt-use-depth"
HOST="${CT101_SSH_HOST:-ct101}"
VAULT_DB="${KEEP_QUALITY_VAULT_DB:-/opt/gzmo/data/vault.db}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
MIN_GE3="${FELT_USE_MIN_GE3:-100}"
MIN_SHARE="${FELT_USE_MIN_SHARE_GE3:-0.40}"
mkdir -p "$OUT"

export OUT HOST VAULT_DB GZMO_BIN MIN_GE3 MIN_SHARE ROOT
python3 - <<'PY'
import json, os, re, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
host = os.environ["HOST"]
vault = os.environ["VAULT_DB"]
gzmo_bin = os.environ["GZMO_BIN"]
min_ge3 = int(os.environ["MIN_GE3"])
min_share = float(os.environ["MIN_SHARE"])
now = datetime.now(timezone.utc).isoformat()

def ssh(cmd: str, timeout: int = 25) -> tuple[int, str, str]:
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
        capture_output=True, text=True, timeout=timeout,
    )
    return p.returncode, p.stdout or "", p.stderr or ""

census = {"ok": False}
sql = (
    f"sqlite3 '{vault}' \""
    "SELECT "
    "(SELECT COUNT(*) FROM honeypot WHERE is_latest=1), "
    "(SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>=1), "
    "(SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>=3);\""
)
rc, stdout, stderr = ssh(sql)
nums = [int(x) for x in re.findall(r"\d+", stdout.replace("|", "\n")) if x.isdigit()]
# Prefer pipe-separated single line
if "|" in stdout:
    parts = [p.strip() for p in stdout.strip().split("|") if p.strip().isdigit()]
    if len(parts) >= 3:
        nums = [int(parts[0]), int(parts[1]), int(parts[2])]
elif len(nums) >= 3:
    nums = nums[:3]

if rc == 0 and len(nums) >= 3:
    latest, ge1, ge3 = nums[0], nums[1], nums[2]
    share1 = (ge1 / latest) if latest else 0.0
    # Depth among felt facts (honest nutrient signal). Vault-wide ge3/latest
    # is retained as share_ge3_of_latest for trend only — not the floor.
    share3_felt = (ge3 / ge1) if ge1 else 0.0
    share3_latest = (ge3 / latest) if latest else 0.0
    census = {
        "ok": True,
        "latest": latest,
        "recall_ge1": ge1,
        "recall_ge3": ge3,
        "share_ge1": round(share1, 6),
        "share_ge3": round(share3_felt, 6),
        "share_ge3_of_latest": round(share3_latest, 6),
        "share_denominator": "recall_ge1",
    }
else:
    census = {
        "ok": False,
        "error": (stderr or stdout or f"ssh_rc={rc}")[:240],
    }

ripen = {"ok": False}
rc2, ripen_out, ripen_err = ssh(
    f"bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml {gzmo_bin} ripen status'",
    timeout=40,
)
if rc2 == 0 and ripen_out.strip():
    dual = None
    dual_origin = None
    nonzero = None
    m = re.search(r"Dual gate[^\n]*?\*\* *(\d+)", ripen_out)
    if m:
        dual = int(m.group(1))
    if dual is None:
        m = re.search(r"Dual gate[^\n]*?: *(\d+)", ripen_out)
        if m:
            dual = int(m.group(1))
    m = re.search(r"Dual \+ allowed origin[^\n]*?\*\* *(\d+)", ripen_out)
    if m:
        dual_origin = int(m.group(1))
    if dual_origin is None:
        m = re.search(r"Dual \+ allowed origin[^\n]*?: *(\d+)", ripen_out)
        if m:
            dual_origin = int(m.group(1))
    m = re.search(r"Nonzero recall_count[^\n]*?\*\* *(\d+)", ripen_out)
    if m:
        nonzero = int(m.group(1))
    if nonzero is None:
        m = re.search(r"Nonzero recall_count[^\n]*?: *(\d+)", ripen_out)
        if m:
            nonzero = int(m.group(1))
    starved = bool(re.search(r"Starved|starved_recall", ripen_out, re.I))
    ripen = {
        "ok": True,
        "nonzero_recall": nonzero,
        "dual_gate": dual,
        "dual_origin": dual_origin,
        "starved": starved,
        "snippet": ripen_out.strip()[:500],
    }
else:
    ripen = {"ok": False, "error": (ripen_err or ripen_out or f"rc={rc2}")[:200]}

depth_ok = False
if census.get("ok"):
    depth_ok = (
        int(census["recall_ge3"]) >= min_ge3
        and float(census["share_ge3"]) >= min_share
    )

# Census reachable ⇒ ok for gate GREEN; thin depth is HOLD advice, not RED lie
if not census.get("ok"):
    advice = "felt_use_depth_unreachable — living vault census failed"
    verdict = "RED"
    ok = False
elif depth_ok:
    advice = (
        f"felt_use_depth_ok — recall≥3={census['recall_ge3']}/{census['recall_ge1']} felt "
        f"(share_ge3={census['share_ge3']}; of_latest={census.get('share_ge3_of_latest')}) "
        f"ripen_dual={ripen.get('dual_gate')}"
    )
    verdict = "GREEN"
    ok = True
else:
    advice = (
        f"felt_use_depth_thin — recall≥3={census['recall_ge3']}/{census['recall_ge1']} felt "
        f"(share_ge3={census['share_ge3']}; floor ge3>={min_ge3} share>={min_share}). "
        "Grow via real living MCP search side-effects — no memory-gym."
    )
    verdict = "HOLD"
    ok = True  # honest thin ≠ gate RED

payload = {
    "schema": "gzmo.brain_feed.felt_use_depth/v1",
    "generated_at": now,
    "verdict": verdict,
    "ok": ok,
    "advice": advice,
    "depth_ok": depth_ok,
    "floors": {"min_ge3": min_ge3, "min_share_ge3": min_share},
    "census": census,
    "ripen": ripen,
    "baseline_note": "2026-07-20 CT101 baseline ~38743 / 107 / 60",
    "doc": "docs/BRAIN_FEED.md",
    "operator": [
        "Do not open Cursor just to search memory",
        "Living MCP search during real work raises recall_count",
        "Ripen dual-gate needs recall≥3 — see gzmo ripen status on living host",
    ],
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Felt Use depth",
    "",
    f"Verdict: **{verdict}**",
    "",
    f"- Advice: {advice}",
    f"- Depth ok: **{depth_ok}**",
    f"- Census: `{census}`",
    f"- Ripen: dual_gate={ripen.get('dual_gate')} dual_origin={ripen.get('dual_origin')} starved={ripen.get('starved')}",
    "",
    "See docs/BRAIN_FEED.md · docs/KEEP_QUALITY.md",
    "",
]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({
    "verdict": verdict,
    "ok": ok,
    "depth_ok": depth_ok,
    "advice": advice,
    "census": census,
    "ripen_dual": ripen.get("dual_gate"),
}, indent=2))
raise SystemExit(0 if ok else 1)
PY
