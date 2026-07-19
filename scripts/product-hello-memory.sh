#!/usr/bin/env bash
# Prime product: end-to-end hello after attach — status + first-fact + search.
#
#   bash scripts/product-hello-memory.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/product-hello"
mkdir -p "$OUT"

bash "$ROOT/scripts/mcp-attach-check.sh" >/dev/null 2>&1 || true
bash "$ROOT/scripts/product-first-fact.sh" >/dev/null 2>&1 || true

export DATA OUT ROOT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])
now = datetime.now(timezone.utc).isoformat()

def load(p):
    try:
        return json.loads(Path(p).read_text(encoding="utf-8"))
    except Exception:
        return {}

attach = load(data / "mcp-attach" / "latest.json")
first = load(data / "product-first-fact" / "latest.json")
stranger = load(data / "product-stranger" / "latest.json")

ok = bool(attach.get("ok")) and bool(first.get("first_fact_ok"))
advice = (
    "hello_ok — attach + first fact remembered on ~/.gzmo"
    if ok
    else (
        "partial — attach ok; first fact needs live engine (Prime :8000)"
        if attach.get("ok")
        else "hold — fix MCP attach then re-run"
    )
)
payload = {
    "schema": "gzmo.product.hello/v1",
    "generated_at": now,
    "ok": ok,
    "advice": advice,
    "mcp_attach": attach.get("advice"),
    "first_fact": first.get("advice"),
    "stranger": stranger.get("advice"),
    "marker": first.get("marker"),
    "note": "Stranger feel: Cursor/Pi attach → durable takeaway → searchable memory.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Product hello memory",
            "",
            f"Advice: **{advice}**",
            f"- Attach: {attach.get('advice')}",
            f"- First fact: {first.get('advice')}",
            f"- Marker: `{first.get('marker')}`",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": ok, "advice": advice}, indent=2))
PY
exit 0
