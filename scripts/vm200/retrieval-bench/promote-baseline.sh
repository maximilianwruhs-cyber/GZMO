#!/usr/bin/env bash
# Promote latest retrieval-bench run to baseline-lock.json
set -euo pipefail

BENCH_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RUNS="${BENCH_DIR}/runs"
LOCK="${BENCH_DIR}/baseline-lock.json"

latest="$(find "${RUNS}" -name summary.json -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)"
if [[ -z "${latest}" ]]; then
  echo "[!] No runs found under ${RUNS}" >&2
  exit 1
fi
run_dir="$(dirname "${latest}")"
python3 -c "
import json, pathlib, datetime
run = pathlib.Path('${run_dir}')
summary = json.loads((run / 'summary.json').read_text())
lock = {
    'promoted_at': datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'run_id': summary.get('run_id'),
    'run_dir': str(run),
    'summary': summary,
}
pathlib.Path('${LOCK}').write_text(json.dumps(lock, indent=2) + '\n')
print('[OK] baseline → ${LOCK}')
print('  run_id:', summary.get('run_id'))
"
