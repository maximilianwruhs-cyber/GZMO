#!/usr/bin/env bash
# Wait for honest soak night #3 (≥18h after last counted GREEN), then record sample.
# Never backdates timestamps.
#
#   bash scripts/honest-soak-night-watch.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HONEST="$ROOT/research/opportunities/soak-honest-nights.md"
SLEEP_S="${HONEST_SOAK_SLEEP_SECS:-600}"
MAX_H="${HONEST_SOAK_WATCH_MAX_HOURS:-24}"
DEADLINE=$(( $(date +%s) + MAX_H * 3600 ))

echo "=== honest soak night watch (max ${MAX_H}h) ==="
while true; do
  bash "$ROOT/scripts/keep-quality-soak.sh" --summary >/tmp/kq-honest.json 2>/tmp/kq-honest.err || true
  # Also run a fresh keep-quality sample when eligible
  N="$(python3 -c "import json; print(json.load(open('/tmp/kq-honest.json')).get('honest_nights',0))" 2>/dev/null || echo 0)"
  ADVICE="$(python3 -c "import json; print(json.load(open('/tmp/kq-honest.json')).get('advice','?'))" 2>/dev/null || echo '?')"
  echo "[$(date -u +%H:%M:%SZ)] honest_nights=$N advice=$ADVICE"
  if [[ "$N" -ge 3 ]]; then
    python3 - <<PY
from pathlib import Path
import re
from datetime import datetime, timezone
p = Path("$HONEST")
if p.exists():
    text = p.read_text(encoding="utf-8")
    text = re.sub(r"(?m)^status:.*$", "status: soaked", text, count=1)
    stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    if "Honest nights closed" not in text:
        text += f"\n\n## Honest nights closed {stamp}\n\nhonest_nights≥3.\n"
    p.write_text(text, encoding="utf-8")
    print("marked soak-honest-nights soaked")
PY
    exit 0
  fi
  # Attempt to append a new GREEN sample (spacing enforced by soak summary)
  bash "$ROOT/scripts/keep-quality-gate.sh" >/tmp/kq-gate-out.txt 2>&1 || true
  bash "$ROOT/scripts/keep-quality-soak.sh" >/tmp/kq-soak-append.txt 2>&1 || true
  if [[ $(date +%s) -ge $DEADLINE ]]; then
    echo "HOLD: honest night 3 not reached within deadline" >&2
    exit 2
  fi
  sleep "$SLEEP_S"
done
