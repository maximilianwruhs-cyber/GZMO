#!/usr/bin/env bash
# Watch promote-loop soak until GREEN, then mark bet soaked (G2).
# Honest: never fakes pin age or overnight. Polls promote-loop-soak-check.sh.
#
#   bash scripts/promote-loop-soak-watch.sh
#   PROMOTE_SOAK_WATCH_MAX_HOURS=10 bash scripts/promote-loop-soak-watch.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAX_H="${PROMOTE_SOAK_WATCH_MAX_HOURS:-10}"
SLEEP_S="${PROMOTE_SOAK_WATCH_SLEEP_SECS:-300}"
DEADLINE=$(( $(date +%s) + MAX_H * 3600 ))
OPP="$ROOT/research/opportunities/promote-loop-living-apply.md"
HONEST="$ROOT/research/opportunities/soak-honest-nights.md"

echo "=== promote-loop soak watch (max ${MAX_H}h, poll ${SLEEP_S}s) ==="
SOAK_JSON="${GZMO_DATA_NEXT:-$ROOT/data-next}/beat-gate/promotions/soak-latest.json"
while true; do
  bash "$ROOT/scripts/promote-loop-soak-check.sh" >/tmp/promote-soak-watch.out 2>/tmp/promote-soak-watch.err || true
  VERDICT="$(python3 -c "import json; print(json.load(open('$SOAK_JSON')).get('verdict','?'))" 2>/dev/null || echo '?')"
  echo "[$(date -u +%H:%M:%SZ)] verdict=$VERDICT"
  if [[ "$VERDICT" == "GREEN" ]]; then
    python3 - <<PY
from pathlib import Path
import re
from datetime import datetime, timezone
p = Path("$OPP")
text = p.read_text(encoding="utf-8")
text2 = re.sub(r"(?m)^status:.*$", "status: soaked", text, count=1)
stamp = datetime.now(timezone.utc).strftime("%Y-%m-%d")
if "Soak closed" not in text2:
    text2 += f"\n\n## Soak closed {stamp}\n\n`promote-loop-soak-check.sh` GREEN — Done when #4 met.\n"
p.write_text(text2, encoding="utf-8")
print("marked promote-loop-living-apply soaked")
PY
    # Refresh honest nights counter (may still be 2/3)
    bash "$ROOT/scripts/keep-quality-soak.sh" --summary >/tmp/kq-soak-sum.json 2>&1 || true
    HONEST_N="$(python3 -c "import json; print(json.load(open('/tmp/kq-soak-sum.json')).get('honest_nights',0))" 2>/dev/null || echo 0)"
    if [[ "$HONEST_N" -ge 3 ]]; then
      python3 - <<PY
from pathlib import Path
import re
from datetime import datetime, timezone
p = Path("$HONEST")
if p.exists():
    text = p.read_text(encoding="utf-8")
    text = re.sub(r"(?m)^status:.*$", "status: soaked", text, count=1)
    p.write_text(text, encoding="utf-8")
    print("marked soak-honest-nights soaked")
PY
    fi
    echo "[OK] soak watch complete"
    # Prefer next nutrient bet: Felt Use + utility
    ROOT="$ROOT" python3 - <<'PY'
from pathlib import Path
import re
import glob
import os
root = Path(os.environ["ROOT"])
promo = (root / "research/opportunities/promote-loop-living-apply.md").read_text(encoding="utf-8")
if "status: soaked" not in promo:
    raise SystemExit(0)
felt = root / "research/opportunities/felt-use-mass-growth.md"
text = felt.read_text(encoding="utf-8")
active = []
for p in glob.glob(str(root / "research/opportunities/*.md")):
    if p.endswith("README.md"):
        continue
    t = Path(p).read_text(encoding="utf-8")
    parts = t.split("---", 2)
    fm = parts[1] if t.startswith("---") and len(parts) > 2 else ""
    if "status: active" in fm:
        active.append(p)
if active:
    print("skip activate felt-use; still active:", active)
else:
    text = re.sub(r"(?m)^status:.*$", "status: active", text, count=1)
    felt.write_text(text, encoding="utf-8")
    print("activated felt-use-mass-growth")
PY
    exit 0
  fi
  if [[ $(date +%s) -ge $DEADLINE ]]; then
    echo "HOLD: deadline reached without GREEN (do not fake)" >&2
    exit 2
  fi
  sleep "$SLEEP_S"
done
