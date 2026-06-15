#!/usr/bin/env bash
# Pre-reboot: final sync, verify stack, stop GZMO services for a clean reboot.
# Post-reboot: ./scripts/after-boot-verify.sh && systemctl --user start gzmo-prime gzmo-daemon
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "=== prepare-for-reboot $(date -Is) ==="

if pgrep -f 'slow-reingest-migration.sh|run-local-micro-retry.sh' >/dev/null 2>&1; then
  echo "[!] Migration ingest still running — wait for 'migration ingest done' in logs" >&2
  exit 1
fi

if grep -q '^cloud_first_background = true' "$ROOT/gzmo.toml"; then
  echo "[!] cloud_first_background=true — set false for local-only steady state" >&2
  exit 1
fi

if ! grep -q '^active_mode = "local"' "$ROOT/gzmo.toml"; then
  echo "[!] active_mode should be local for post-reboot local stack" >&2
  exit 1
fi

echo "[*] Migration summary"
python3 - <<'PY'
from pathlib import Path
root = Path(".")
ok, fail = set(), set()
for line in (root / "logs/migration-ingest-progress.txt").read_text().splitlines():
    if line.startswith("OK "):
        ok.add(line[3:].strip())
    elif line.startswith("FAIL "):
        fail.add(line[5:].strip())

def parent(n):
    return n.rsplit("-micro", 1)[0] + ".md" if "-micro" in n else n

ok_p = {parent(x) for x in ok}
print(f"  unique_OK={len(ok)} micro_OK={sum(1 for x in ok if '-micro' in x)}")
print(f"  ok_parents={len(ok_p)}/234 parents_only_fail={len({parent(x) for x in fail} - ok_p)}")
PY

echo "[*] Final Qdrant sync"
if [[ -x "$ROOT/scripts/sync-vault-to-qdrant.sh" ]]; then
  "$ROOT/scripts/sync-vault-to-qdrant.sh" 2>&1 | tail -5
fi

echo "[*] Memory status"
"$ROOT/scripts/memory-status.sh" 2>/dev/null || true

echo "[*] Production verify (non-fatal)"
if [[ -x "$ROOT/scripts/verify-production.sh" ]]; then
  "$ROOT/scripts/verify-production.sh" || echo "[WARN] verify-production had failures (re-check after reboot)"
fi

echo "[*] Stopping GZMO services for clean reboot"
systemctl --user stop gzmo-daemon.service 2>/dev/null || true
systemctl --user stop gzmo-prime.service 2>/dev/null || true

if ps aux | grep -q '[t]arget/release/gzmo daemon'; then
  pkill -f 'target/release/gzmo daemon' 2>/dev/null || true
  sleep 1
fi

if curl -sf http://127.0.0.1:8000/v1/models >/dev/null 2>&1; then
  pkill -f 'llama-server.*--port 8000' 2>/dev/null || true
  sleep 2
fi

stray=0
while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  if [[ "$line" == *slow-reingest-migration.sh* ]] \
    || [[ "$line" == *run-local-micro-retry.sh* ]] \
    || [[ "$line" == */target/release/gzmo\ ingest\ * ]]; then
    echo "$line"
    stray=1
  fi
done < <(pgrep -af 'slow-reingest-migration|run-local-micro-retry|gzmo ingest' 2>/dev/null || true)
if [[ "$stray" -eq 1 ]]; then
  echo "[!] Stray migration/ingest processes still running" >&2
  exit 1
fi

echo ""
echo "=== Ready to reboot ==="
echo "  Config: local-only (cloud_first_background=false, ingest on Prime)"
echo "  sudo reboot"
echo ""
echo "After reboot:"
echo "  cd $ROOT"
echo "  ./scripts/after-boot-verify.sh"
echo "  systemctl --user start gzmo-prime.service gzmo-daemon.service"
echo "  ./scripts/verify-production.sh"
