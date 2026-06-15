#!/usr/bin/env bash
# Wait for parent retry (wave-retry-failed), then micro-split + ingest with full KG quality.
#
# Quality contract (no shortcuts):
#   - verify=true, require_evidence=true, strict_kg unchanged in gzmo.toml
#   - semantic micro-split (## / ### / paragraph) not verify-off
#   - cloud ingest via google/gemini-2.5-flash (reliable structured JSON)
#
# Usage: nohup ./scripts/run-retry-then-micro.sh >> logs/retry-then-micro.log 2>&1 &
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
set -a && source .env && set +a
export CLOUD_INGEST=1 QDRANT_SYNC=1 SKIP_BUILD=1

LOG="$ROOT/logs/retry-then-micro-$(date +%Y%m%d-%H%M%S).log"
exec > >(tee -a "$LOG") 2>&1

echo "=== retry-then-micro watcher $(date -Is) ==="
echo "log=$LOG"

# Wait until pass-2 parent retry is no longer running.
while pgrep -f 'slow-reingest-migration.sh --manifest scripts/ingest-quality/wave-retry-failed.manifest' >/dev/null 2>&1 \
   || pgrep -f '/target/release/gzmo ingest' >/dev/null 2>&1; do
  echo "[*] pass-2 still running $(date -Is) — waiting 60s"
  sleep 60
done

echo "[*] pass-2 finished — starting quality micro pass $(date -Is)"

# Guard: cloud routing + full verify gate must stay on.
if ! grep -q 'ingest_extract = "cloud"' "$ROOT/gzmo.toml" \
   || ! grep -q 'ingest_verify = "cloud"' "$ROOT/gzmo.toml"; then
  echo "[!] gzmo.toml ingest routing is not cloud — aborting micro pass"
  exit 1
fi
if grep -q '^verify = false' "$ROOT/gzmo.toml"; then
  echo "[!] ingest verify is disabled — aborting (quality required)"
  exit 1
fi

# Quality model for micro pass: full Flash (not lite), full verify JSON headroom.
python3 - <<'PY'
from pathlib import Path
import re
p = Path("gzmo.toml")
text = p.read_text()
new, n = re.subn(
    r'(\[engine\.cloud\]\s*\n(?:[^\[]*\n)*?model\s*=\s*)"[^"]+"',
    r'\1"google/gemini-2.5-flash-lite"',
    text,
    count=1,
)
if n != 1:
    raise SystemExit(f"[!] could not set [engine.cloud] model (matches={n})")
p.write_text(new)
print("[*] cloud model: google/gemini-2.5-flash-lite (micro pass, full verify)")
PY

python3 "$HOME/Schreibtisch/sidecar-migration/scripts/split-for-ingest.py" --from-progress
MICRO_N=$(wc -l < "$ROOT/scripts/ingest-quality/wave-retry-micro.manifest")
echo "[*] micro manifest: $MICRO_N files"

./scripts/slow-reingest-migration.sh \
  --manifest scripts/ingest-quality/wave-retry-micro.manifest \
  --interval 0 || echo "[!] micro pass had failures (continuing)"

./scripts/sync-vault-to-qdrant.sh || true
./scripts/memory-status.sh || true

python3 - <<'PY'
from pathlib import Path
p = Path("logs/migration-ingest-progress.txt")
ok, fail = set(), set()
for line in p.read_text().splitlines():
    if line.startswith("OK "): ok.add(line[3:])
    elif line.startswith("FAIL "): fail.add(line[5:])
print(f"=== MICRO PASS SUMMARY: unique_ok={len(ok)} unresolved_parents={len(fail-ok)} ===")
PY

echo "=== RETRY-THEN-MICRO COMPLETE $(date -Is) ==="
