#!/usr/bin/env bash
# Prime product: CT101 takeaway → distill → recall (living vault).
# Soft nightburst exit; living_proof flag records same-sitting HIT.
#
#   bash scripts/ct101-takeaway-recall.sh
#   CT101_TAKEAWAY_DISTILL=0 bash scripts/ct101-takeaway-recall.sh  # enqueue only
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ct101-takeaway-recall"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
DISTILL="${CT101_TAKEAWAY_DISTILL:-1}"
mkdir -p "$OUT"

MARKER="LivingTakeaway-$(date -u +%Y%m%dT%H%M%SZ)-$$"
SID="living-takeaway-$$"
NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
REMOTE_SESS="/opt/gzmo/data/sessions/${SID}.json"
LOG="$OUT/remote.log"

ssh_ct() {
  ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "$@"
}

: >"$LOG"
SEED_OK=0
CLOSE_OK=0
DISTILL_OK=0
SEARCH_OK=0
HIT=0

{
  echo "=== CT101 takeaway→recall ($HOST) marker=$MARKER ==="
  # Seed minimal session on living host.
  if ssh_ct "cat > '$REMOTE_SESS'" <<EOF
{
  "id": "${SID}",
  "name": "living_takeaway_recall",
  "created_at": "${NOW}",
  "last_active_at": "${NOW}",
  "messages": [
    {
      "role": "user",
      "content": "Prime-product living takeaway proof session.",
      "is_meta": false
    },
    {
      "role": "assistant",
      "content": "Ready to record a durable living takeaway.",
      "is_meta": false
    }
  ]
}
EOF
  then
    SEED_OK=1
    echo "OK: seeded $REMOTE_SESS"
  else
    echo "FAIL: seed session"
  fi

  if [[ "$SEED_OK" == "1" ]]; then
    if ssh_ct "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN session close $SID --takeaway \"$MARKER: living takeaway feeds CT101 metabolism\"'"; then
      CLOSE_OK=1
      echo "OK: session close --takeaway"
    else
      echo "FAIL: session close"
    fi
  fi

  if [[ "$CLOSE_OK" == "1" && "$DISTILL" == "1" ]]; then
    if ssh_ct "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN distill $SID'"; then
      DISTILL_OK=1
      echo "OK: distill $SID"
    else
      echo "FAIL: distill (daemon may still metabolize overnight)"
    fi
  elif [[ "$CLOSE_OK" == "1" ]]; then
    echo "SKIP: distill (CT101_TAKEAWAY_DISTILL=0) — await daemon / overnight"
  fi

  if [[ "$CLOSE_OK" == "1" ]]; then
    SEARCH_OUT="$(ssh_ct "bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml $GZMO_BIN memory search \"$MARKER\" --limit 5 --no-scratch'" || true)"
    echo "$SEARCH_OUT"
    SEARCH_OK=1
    if echo "$SEARCH_OUT" | grep -Fqi "$MARKER"; then
      HIT=1
      echo "OK: recall HIT for marker"
    else
      echo "HOLD: marker not in search yet (seeded; overnight may still promote)"
    fi
  fi
} >>"$LOG" 2>&1 || true

# Re-read flags from log if subshell lost them — parse simply from log.
grep -q "OK: seeded" "$LOG" && SEED_OK=1 || true
grep -q "OK: session close" "$LOG" && CLOSE_OK=1 || true
grep -q "OK: distill" "$LOG" && DISTILL_OK=1 || true
grep -q "OK: recall HIT" "$LOG" && HIT=1 || true
grep -q "Platform recall\|Honeypot" "$LOG" && SEARCH_OK=1 || true

export OUT MARKER SID HOST SEED_OK CLOSE_OK DISTILL_OK SEARCH_OK HIT DISTILL LOG
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
hit = os.environ.get("HIT") == "1"
close_ok = os.environ.get("CLOSE_OK") == "1"
distill_ok = os.environ.get("DISTILL_OK") == "1"
seed_ok = os.environ.get("SEED_OK") == "1"
living_proof = hit and close_ok
advice = (
    "living_hit — takeaway distilled and recalled on CT101 same sitting"
    if living_proof
    else (
        "seeded_await — takeaway closed; distill/promote still pending overnight"
        if close_ok
        else "hold — could not seed/close on CT101 (see remote.log)"
    )
)
payload = {
    "schema": "gzmo.ct101.takeaway-recall/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "living_proof": living_proof,
    "advice": advice,
    "host": os.environ.get("HOST"),
    "session_id": os.environ.get("SID"),
    "marker": os.environ.get("MARKER"),
    "steps": {
        "seed_ok": seed_ok,
        "close_ok": close_ok,
        "distill_ok": distill_ok,
        "distill_requested": os.environ.get("DISTILL") == "1",
        "search_ok": os.environ.get("SEARCH_OK") == "1",
        "recall_hit": hit,
    },
    "log": str(out / "remote.log"),
    "note": "Prime product loop on living vault — workstation lab takeaway stays separate.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# CT101 takeaway → recall",
            "",
            f"Advice: **{advice}**",
            f"Marker: `{payload['marker']}`",
            f"Session: `{payload['session_id']}`",
            f"Steps: seed={seed_ok} close={close_ok} distill={distill_ok} hit={hit}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "living_proof": living_proof, "advice": advice, "hit": hit}, indent=2))
PY
exit 0
