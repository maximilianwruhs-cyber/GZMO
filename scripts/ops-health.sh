#!/usr/bin/env bash
# Stack ops health — workstation + CT101 living probes (read-only).
# Never starts gzmo-serve. Writes data-next/ops-health/latest.{json,md}
#
#   bash scripts/ops-health.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/ops-health"
HOST="${CT101_SSH_HOST:-ct101}"
PRIME_URL="${PRIME_PROBE_URL:-http://127.0.0.1:8000/v1/models}"
OKFORGE_URL="${OKFORGE_PROBE_URL:-http://127.0.0.1:3000/observatory}"
mkdir -p "$OUT"

pass=0
fail=0
warn=0
declare -a ROWS=()

row() {
  local status="$1" name="$2" detail="$3"
  ROWS+=("$status|$name|$detail")
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
    WARN) warn=$((warn + 1)) ;;
  esac
}

probe_http() {
  local url="$1"
  curl -sf --max-time 5 "$url" >/dev/null 2>&1
}

serve="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
serve="$(printf '%s\n' "$serve" | head -1)"
if [[ "$serve" == "active" ]]; then
  row FAIL "dual-writer" "workstation gzmo-serve active — living claim conflict risk"
else
  row PASS "dual-writer" "serve=${serve:-inactive}"
fi

if probe_http "$PRIME_URL"; then
  row PASS "prime" "$PRIME_URL"
else
  row FAIL "prime" "unreachable $PRIME_URL"
fi

if probe_http "$OKFORGE_URL"; then
  row PASS "okforge" "$OKFORGE_URL"
else
  row WARN "okforge" "unreachable $OKFORGE_URL (soft)"
fi

# CT101 living probes (SSH)
if ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'true' 2>/dev/null; then
  row PASS "ct101-ssh" "$HOST"
  daemon="$(ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'systemctl is-active gzmo-daemon' 2>/dev/null || true)"
  if [[ "$daemon" == "active" ]]; then
    row PASS "gzmo-daemon" "active"
  else
    row FAIL "gzmo-daemon" "state=$daemon"
  fi
  sidecars="$(ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'docker ps --format "{{.Names}}:{{.Status}}"' 2>/dev/null || true)"
  for name in sidecar-redis sidecar-qdrant sidecar-neo4j; do
    if echo "$sidecars" | grep -q "${name}:Up"; then
      row PASS "$name" "Up"
    else
      row FAIL "$name" "not Up"
    fi
  done
  # Redis/Qdrant HTTP from CT101 localhost
  if ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'curl -sf --max-time 3 http://127.0.0.1:6333/collections >/dev/null'; then
    row PASS "qdrant-local" "6333"
  else
    row FAIL "qdrant-local" "6333 unreachable on CT101"
  fi
  if ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'redis-cli ping 2>/dev/null | grep -q PONG || docker exec sidecar-redis redis-cli ping 2>/dev/null | grep -q PONG'; then
    row PASS "redis-local" "PONG"
  else
    row WARN "redis-local" "ping soft-fail"
  fi
else
  row FAIL "ct101-ssh" "cannot reach $HOST"
fi

# Living smoke soft (reuse script if present)
if [[ -x "$ROOT/scripts/ct101-living-smoke.sh" ]]; then
  if bash "$ROOT/scripts/ct101-living-smoke.sh" >/dev/null 2>&1; then
    row PASS "living-smoke" "ct101-living-smoke.sh PASS"
  else
    row FAIL "living-smoke" "ct101-living-smoke.sh FAIL"
  fi
fi

verdict=GREEN
[[ "$fail" -eq 0 ]] || verdict=RED
[[ "$fail" -eq 0 && "$warn" -gt 0 ]] && verdict=YELLOW

generated="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
{
  echo "# ops-health — $verdict"
  echo
  echo "generated_at: $generated"
  echo
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    echo "- [$st] $name — $detail"
  done
} >"$OUT/latest.md"

python3 - "$OUT" "$verdict" "$pass" "$fail" "$warn" "$generated" "${ROWS[@]}" <<'PY'
import json, sys
from pathlib import Path
out = Path(sys.argv[1])
verdict, pass_n, fail_n, warn_n, generated = sys.argv[2:7]
rows = []
for raw in sys.argv[7:]:
    st, name, detail = raw.split("|", 2)
    rows.append({"status": st, "name": name, "detail": detail})
payload = {
    "schema": "gzmo.ops_health/v1",
    "generated_at": generated,
    "verdict": verdict,
    "pass": int(pass_n),
    "fail": int(fail_n),
    "warn": int(warn_n),
    "ok": verdict != "RED",
    "rows": rows,
    "advice": (
        "ops_health_RED — fix FAIL rows before overnight trust"
        if verdict == "RED"
        else (
            "ops_health_YELLOW — soft warnings only"
            if verdict == "YELLOW"
            else "ops_health_GREEN — stack probes ok"
        )
    ),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY

[[ "$verdict" != "RED" ]]
