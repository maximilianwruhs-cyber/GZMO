#!/usr/bin/env bash
# Local Prime retry for micro files that failed cloud ingest (truncated JSON).
# Requires Prime :8000, daemon stopped, ingest_* = local_deterministic, cloud_first_background = false.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

set -a && source .env && set +a
export QDRANT_SYNC=1 SKIP_BUILD=1
unset CLOUD_INGEST

if grep -q '^verify = false' "$ROOT/gzmo.toml"; then
  echo "[!] ingest verify disabled — refusing retry without quality gate" >&2
  exit 1
fi

if grep -q '^cloud_first_background = true' "$ROOT/gzmo.toml"; then
  echo "[!] cloud_first_background=true — ingest would hit OpenRouter first" >&2
  exit 1
fi

if ! grep -q '^ingest_extract = "local_deterministic"' "$ROOT/gzmo.toml"; then
  echo "[!] ingest_extract must be local_deterministic for this pass" >&2
  exit 1
fi

if pgrep -f '/target/release/gzmo daemon' >/dev/null 2>&1; then
  echo "[!] stop gzmo-daemon before batch ingest" >&2
  exit 1
fi

prime_code="$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/v1/models 2>/dev/null || echo 000)"
if [[ "$prime_code" != "200" ]]; then
  echo "[!] Prime not reachable at :8000 (HTTP $prime_code)" >&2
  exit 1
fi

PROGRESS="$ROOT/logs/migration-ingest-progress.txt"
SRC_MANIFEST="$ROOT/scripts/ingest-quality/wave-retry-micro.manifest"
FAIL_MANIFEST="$ROOT/scripts/ingest-quality/wave-retry-micro-fail.manifest"

python3 - <<'PY'
from pathlib import Path

root = Path.cwd()
progress = root / "logs/migration-ingest-progress.txt"
src = root / "scripts/ingest-quality/wave-retry-micro.manifest"
out = root / "scripts/ingest-quality/wave-retry-micro-fail.manifest"

ok = set()
if progress.exists():
    for line in progress.read_text().splitlines():
        if line.startswith("OK "):
            ok.add(line[3:].strip())

fail_only = []
for line in src.read_text().splitlines():
    line = line.strip()
    if not line or line.startswith("#"):
        continue
    if Path(line).name not in ok:
        fail_only.append(line)

out.write_text("\n".join(fail_only) + ("\n" if fail_only else ""), encoding="utf-8")
print(f"fail_manifest={out} count={len(fail_only)} skipped_ok={len(ok)}")
PY

if [[ ! -s "$FAIL_MANIFEST" ]]; then
  echo "[OK] no failing micro files left to retry"
  ./scripts/memory-status.sh || true
  exit 0
fi

echo "[*] local micro retry: $(wc -l < "$FAIL_MANIFEST") files via Prime :8000"
./scripts/slow-reingest-migration.sh \
  --manifest scripts/ingest-quality/wave-retry-micro-fail.manifest \
  --interval 0 || true

./scripts/sync-vault-to-qdrant.sh || true
./scripts/memory-status.sh || true
