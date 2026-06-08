#!/usr/bin/env bash
# Re-eval top N files with worst golden fact recall (Prime). Updates report.json per file.
# Usage: N=5 scripts/ingest-quality/patch-worst-facts.sh
# Prereq: reports/missing-facts-*.json from report-missing-facts.py

set -eo pipefail
export LC_ALL=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(cd "$DIR/../.." >/dev/null 2>&1 && pwd)"
cd "$PROJECT_ROOT"

N="${N:-5}"
CORPUS="${GZMO_WAVE1_CORPUS:-$HOME/Schreibtisch/knowledge/archive/gzmo_obolus}"
JSON="$(ls -1t "$DIR"/reports/missing-facts-*.json 2>/dev/null | head -1)"

if [[ -z "$JSON" || ! -f "$JSON" ]]; then
  echo "patch-worst-facts: run report-missing-facts.py first" >&2
  exit 2
fi

unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli -q

echo "[*] Top $N files from $JSON"
python3 - "$JSON" "$N" "$CORPUS" <<'PY' | while IFS= read -r path; do
import json, sys
data = json.loads(open(sys.argv[1]).read())
corpus = sys.argv[3]
for row in data.get("top", [])[: int(sys.argv[2])]:
    print(f"{corpus}/{row['file']}")
PY
  echo "--- patch: $(basename "$path")"
  python3 "$DIR/patch-report-file.py" "$path" || echo "[!] failed: $path" >&2
done

python3 "$DIR/recalc-pipeline-summary.py" --write
python3 "$DIR/refresh-report-contract.py"
python3 "$DIR/report-missing-facts.py" --top 10
bash "$DIR/check-contract.sh"
