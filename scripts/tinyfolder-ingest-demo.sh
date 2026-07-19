#!/usr/bin/env bash
# Unpark Wave 1.3 demable: drop a sample note into tinyFolder inbox + dry-run ingest.
# Does not enqueue overnight metabolism. Never touches ~/.gzmo product vault by default.
#
#   bash scripts/tinyfolder-ingest-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/tinyfolder"
INBOX="${TINYFOLDER_INBOX:-$DATA/tinyfolder-inbox}"
mkdir -p "$OUT" "$INBOX"

SAMPLE="$INBOX/unpark-wave13-$(date -u +%Y%m%dT%H%M%SZ).md"
cat >"$SAMPLE" <<EOF
# tinyFolder Unpark Wave 1.3 sample

Operator drop for ingest experiments. Generated $(date -u +%Y-%m-%dT%H:%M:%SZ).
Not a living overnight fact unless an operator runs gated ingest intentionally.
EOF

BIN="${GZMO_BIN:-}"
if [[ -z "$BIN" ]]; then
  [[ -x "$ROOT/target/release/gzmo" ]] && BIN="$ROOT/target/release/gzmo"
  [[ -z "$BIN" && -x "${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo" ]] \
    && BIN="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo"
fi

DRY_LOG="$OUT/ingest-dry-run.log"
: >"$DRY_LOG"
if [[ -n "$BIN" ]]; then
  # Soft dry-run — ingest may be disabled in local toml (drop still demable)
  set +e
  GZMO_ALLOW_LAB_VAULT=1 "$BIN" ingest --dry-run "$SAMPLE" >"$DRY_LOG" 2>&1
  rc=$?
  set -e
  echo "ingest_dry_run_exit=$rc (non-zero ok if [ingest] disabled)" >>"$DRY_LOG"
else
  echo "no gzmo binary — drop only" >"$DRY_LOG"
fi

python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
out = Path("$OUT")
payload = {
  "schema": "gzmo.unpark.tinyfolder.demo/v1",
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "sample": "$SAMPLE",
  "inbox": "$INBOX",
  "dry_run_log": str(out / "ingest-dry-run.log"),
  "ok": True,
  "wave": "1.3",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY

bash "$ROOT/scripts/tinyfolder-check.sh"
echo "[OK] tinyFolder demo → $SAMPLE"
