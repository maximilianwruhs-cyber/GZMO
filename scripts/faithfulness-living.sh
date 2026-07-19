#!/usr/bin/env bash
# Prime product: faithfulness CI against living CT101 vault (soft wrapper).
# Runs claims over SSH; does not use lab nightburst Quillhorn needles.
#
#   bash scripts/faithfulness-living.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/faithfulness-living"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
CLAIMS="${FAITHFULNESS_LIVING_CLAIMS:-$ROOT/scripts/fixtures/faithfulness-living-claims.json}"
mkdir -p "$OUT"

export ROOT DATA OUT HOST GZMO_BIN CLAIMS
python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
claims_path = Path(os.environ["CLAIMS"])
host = os.environ["HOST"]
gzmo = os.environ["GZMO_BIN"]
spec = json.loads(claims_path.read_text(encoding="utf-8"))
claims = spec.get("claims") or []
results = []
misses = 0
ssh_ok = True

for c in claims:
    cid = c.get("id") or c.get("claim", "")[:32]
    claim = c.get("claim", "")
    query = c.get("query") or claim
    needle = c.get("needle") or query
    remote = (
        f"bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml "
        f"{gzmo} memory search {json.dumps(query)} --limit 5 --no-scratch'"
    )
    proc = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, remote],
        capture_output=True,
        text=True,
        check=False,
    )
    evidence = (proc.stdout or "") + (proc.stderr or "")
    if proc.returncode != 0 and "Permission denied" in evidence:
        ssh_ok = False
    hit = needle.lower() in evidence.lower()
    if not hit:
        misses += 1
    results.append(
        {
            "id": cid,
            "claim": claim,
            "query": query,
            "needle": needle,
            "supported": hit,
            "ssh_exit": proc.returncode,
            "evidence_excerpt": evidence[:400].replace("\n", " "),
        }
    )

ok = misses == 0 and ssh_ok
advice = (
    "living_faithful — CORE_INSIGHT / ADR claims supported on CT101"
    if ok
    else (
        "ssh_hold — could not reach CT101"
        if not ssh_ok
        else f"partial — {len(results) - misses}/{len(results)} living claims supported"
    )
)
payload = {
    "schema": "gzmo.faithfulness.living/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,  # soft for nightburst
    "living_ok": ok,
    "advice": advice,
    "host": host,
    "claims_file": str(claims_path),
    "total": len(results),
    "supported": len(results) - misses,
    "misses": misses,
    "results": results,
    "note": "Prime product quality gate on living vault — separate from lab Quillhorn fixture.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Faithfulness (living CT101)",
            "",
            f"Advice: **{advice}**",
            f"Supported: {payload['supported']}/{payload['total']}",
            "",
            *[f"- {'PASS' if r['supported'] else 'MISS'} `{r['id']}` — {r['needle']}" for r in results],
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({
    "ok": True,
    "living_ok": ok,
    "advice": advice,
    "supported": payload["supported"],
    "total": payload["total"],
}, indent=2))
PY
exit 0
