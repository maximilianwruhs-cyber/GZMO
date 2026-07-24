#!/usr/bin/env bash
# OpenClaw → living takeaway enqueue (Brain Feed). Thin wrapper on herdr-living-enqueue.
# Never --now. Refuses dual-writer. Does not upsert Qdrant/Neo4j.
#
#   bash scripts/openclaw-takeaway.sh 'durable fact text'
#   TAKEAWAY='…' bash scripts/openclaw-takeaway.sh
#
# Artifact: data-next/openclaw-attach/takeaway-latest.json (+ herdr living-enqueue.json)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/openclaw-attach"
mkdir -p "$OUT"

if [[ $# -gt 0 ]]; then
  TAKEAWAY="$*"
fi
TAKEAWAY="${TAKEAWAY:-}"
if [[ -z "$TAKEAWAY" ]]; then
  echo "usage: bash scripts/openclaw-takeaway.sh 'takeaway text'" >&2
  exit 2
fi

# Prefix so living distill can attribute OpenClaw nutrient path
if [[ "$TAKEAWAY" != OpenClawTakeaway:* && "$TAKEAWAY" != \[OpenClaw\]* ]]; then
  TAKEAWAY="OpenClawTakeaway: $TAKEAWAY"
fi
export TAKEAWAY

set +e
bash "$ROOT/scripts/herdr-living-enqueue.sh"
rc=$?
set -e

src="$DATA/herdr-metabolism/living-enqueue.json"
if [[ -f "$src" ]]; then
  cp "$src" "$OUT/takeaway-latest.json"
  python3 - "$OUT" "$rc" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
out = Path(sys.argv[1])
rc = int(sys.argv[2])
payload = json.loads((out / "takeaway-latest.json").read_text(encoding="utf-8"))
payload["schema"] = "gzmo.openclaw.takeaway/v1"
payload["wrapper"] = "scripts/openclaw-takeaway.sh"
payload["openclaw_rc"] = rc
payload["generated_at"] = datetime.now(timezone.utc).isoformat()
payload["never"] = [
    "qdrant_upsert",
    "neo4j_raw_write",
    "gzmo_serve_start",
    "session_close_--now",
]
(out / "takeaway-latest.json").write_text(json.dumps(payload, indent=2) + "\n")
md = [
    "# openclaw takeaway",
    "",
    f"ok: **{payload.get('ok')}**",
    f"advice: {payload.get('advice')}",
    f"session_id: {payload.get('session_id')}",
    "",
    "Search living memory via MCP `gzmo-living` — do not curl Qdrant upsert.",
    "",
]
(out / "takeaway-latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"ok": payload.get("ok"), "advice": payload.get("advice"), "rc": rc}, indent=2))
sys.exit(0 if payload.get("ok") else 1)
PY
else
  echo "FAIL: herdr living-enqueue artifact missing" >&2
  exit 1
fi
