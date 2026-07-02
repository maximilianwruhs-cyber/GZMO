#!/usr/bin/env bash
# Post-migration layout checks for LXC101 sidecar homing.
set -euo pipefail

GZMO_ROOT="${GZMO_ROOT:-/opt/gzmo}"
REPO="${GZMO_ROOT}/survey_GZMO"
FAIL=0

check_symlink() {
  local name="$1"
  local expected="${GZMO_ROOT}/${name}"
  local link="${REPO}/${name}"
  if [[ ! -L "${link}" ]]; then
    echo "[FAIL] ${link} is not a symlink"
    FAIL=1
    return
  fi
  local resolved
  resolved="$(readlink -f "${link}")"
  if [[ "${resolved}" != "${expected}" ]]; then
    echo "[FAIL] ${link} -> ${resolved} (expected ${expected})"
    FAIL=1
  else
    echo "[OK] ${name} -> ${expected}"
  fi
}

echo "[*] Sidecar layout verification (${GZMO_ROOT})"

for item in data memory skills wiki SOUL.md DREAMS.md; do
  check_symlink "${item}"
done

if grep -q 'maximilian-wruhs\|Schreibtisch' "${GZMO_ROOT}/gzmo.toml" 2>/dev/null; then
  echo "[FAIL] gzmo.toml still contains workstation paths"
  grep -n 'maximilian-wruhs\|Schreibtisch' "${GZMO_ROOT}/gzmo.toml" || true
  FAIL=1
else
  echo "[OK] gzmo.toml has no workstation path leaks"
fi

if [[ ! -d /home/maximilian/knowledge ]]; then
  echo "[FAIL] /home/maximilian/knowledge missing"
  FAIL=1
else
  echo "[OK] /home/maximilian/knowledge exists"
fi

if systemctl is-active --quiet gzmo-daemon 2>/dev/null; then
  echo "[OK] gzmo-daemon active"
else
  echo "[WARN] gzmo-daemon not active"
fi

if [[ "${FAIL}" -eq 0 ]]; then
  echo "RESULT: SIDECAR LAYOUT OK"
else
  echo "RESULT: SIDECAR LAYOUT FAILED"
  exit 1
fi
