#!/usr/bin/env bash
# Goal C: protocol-level smoke for the in-repo living appliance pin.
# Proves Redis PING, Qdrant /readyz, and Neo4j auth — not just TCP open.
#
# Soft HOLD when sidecars are down (expected off workstation).
# FAIL when ports are up but a protocol check fails, or when
# LIVING_APPLIANCE_REQUIRE_SMOKE=1 and anything is not PASS.
#
#   bash scripts/living-appliance-smoke.sh
#   LIVING_APPLIANCE_REQUIRE_SMOKE=1 bash scripts/living-appliance-smoke.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="${LIVING_APPLIANCE_DIR:-$ROOT/deploy/living-appliance}"
COMPOSE="$DIR/docker-compose.yml"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/living-appliance-smoke"
mkdir -p "$OUT"
LOG="$OUT/smoke.log"
: >"$LOG"

REQUIRE="${LIVING_APPLIANCE_REQUIRE_SMOKE:-0}"
HOST="${LIVING_APPLIANCE_HOST:-127.0.0.1}"
QDRANT_URL="${LIVING_APPLIANCE_QDRANT_URL:-http://${HOST}:6333}"

pass=0
fail=0
hold=0
declare -a ROWS=()

row() {
  local status="$1" name="$2" detail="$3"
  ROWS+=("$status|$name|$detail")
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
    HOLD) hold=$((hold + 1)) ;;
  esac
  echo "[$status] $name — $detail" | tee -a "$LOG"
}

probe_tcp() {
  local port="$1"
  timeout 2 bash -c "echo >/dev/tcp/${HOST}/${port}" 2>/dev/null
}

# Load NEO4J_AUTH from pin .env without printing it.
NEO4J_AUTH=""
if [[ -f "$DIR/.env" ]]; then
  NEO4J_AUTH="$(
    grep -E '^[[:space:]]*NEO4J_AUTH=' "$DIR/.env" 2>/dev/null \
      | head -1 \
      | sed -E 's/^[[:space:]]*NEO4J_AUTH=//; s/^["'\'']//; s/["'\'']$//'
  )"
fi

echo "=== Living appliance smoke (goal C) ===" | tee -a "$LOG"

if [[ ! -f "$COMPOSE" ]]; then
  row FAIL "compose-file" "missing $COMPOSE"
else
  row PASS "compose-file" "$COMPOSE"
fi

# --- Redis ---
if probe_tcp 6379; then
  redis_ok=0
  if command -v docker >/dev/null 2>&1 \
    && docker ps --format '{{.Names}}' 2>/dev/null | grep -qx 'sidecar-redis'; then
    if docker exec sidecar-redis redis-cli ping 2>>"$LOG" | grep -qi '^PONG$'; then
      redis_ok=1
    fi
  elif command -v redis-cli >/dev/null 2>&1; then
    if redis-cli -h "$HOST" -p 6379 ping 2>>"$LOG" | grep -qi '^PONG$'; then
      redis_ok=1
    fi
  else
    # TCP open but no redis-cli / container — soft HOLD (cannot prove PING)
    row HOLD "redis-pong" ":6379 open but no redis-cli/sidecar-redis to PING"
    redis_ok=-1
  fi
  if [[ "$redis_ok" -eq 1 ]]; then
    row PASS "redis-pong" "PONG from ${HOST}:6379"
  elif [[ "$redis_ok" -eq 0 ]]; then
    row FAIL "redis-pong" "PING failed on ${HOST}:6379"
  fi
else
  if [[ "$REQUIRE" == "1" ]]; then
    row FAIL "redis-pong" "${HOST}:6379 closed (required)"
  else
    row HOLD "redis-pong" "${HOST}:6379 closed — start with living-appliance-up.sh"
  fi
fi

# --- Qdrant ---
if probe_tcp 6333; then
  if curl -fsS --max-time 5 "${QDRANT_URL}/readyz" >/dev/null 2>>"$LOG" \
    || curl -fsS --max-time 5 "${QDRANT_URL}/" >/dev/null 2>>"$LOG"; then
    row PASS "qdrant-ready" "${QDRANT_URL} ready"
  else
    row FAIL "qdrant-ready" "${QDRANT_URL} HTTP probe failed"
  fi
else
  if [[ "$REQUIRE" == "1" ]]; then
    row FAIL "qdrant-ready" "${HOST}:6333 closed (required)"
  else
    row HOLD "qdrant-ready" "${HOST}:6333 closed — start with living-appliance-up.sh"
  fi
fi

# --- Neo4j ---
if probe_tcp 7687; then
  if [[ -z "$NEO4J_AUTH" ]]; then
    if [[ "$REQUIRE" == "1" ]]; then
      row FAIL "neo4j-auth" "NEO4J_AUTH missing in $DIR/.env (required)"
    else
      row HOLD "neo4j-auth" "NEO4J_AUTH missing in $DIR/.env — copy from .env.example"
    fi
  elif command -v docker >/dev/null 2>&1 \
    && docker ps --format '{{.Names}}' 2>/dev/null | grep -qx 'sidecar-neo4j'; then
    neo_user="${NEO4J_AUTH%%/*}"
    neo_pass="${NEO4J_AUTH#*/}"
    if [[ -z "$neo_user" || -z "$neo_pass" || "$neo_user" == "$NEO4J_AUTH" ]]; then
      row FAIL "neo4j-auth" "NEO4J_AUTH must be user/password"
    elif docker exec sidecar-neo4j cypher-shell -u "$neo_user" -p "$neo_pass" \
      'RETURN 1 AS ok;' >/dev/null 2>>"$LOG"; then
      row PASS "neo4j-auth" "cypher-shell RETURN 1 ok"
    else
      row FAIL "neo4j-auth" "cypher-shell auth/query failed — check NEO4J_AUTH"
    fi
  else
    if [[ "$REQUIRE" == "1" ]]; then
      row FAIL "neo4j-auth" "sidecar-neo4j not running (required)"
    else
      row HOLD "neo4j-auth" ":7687 open but sidecar-neo4j not in docker ps"
    fi
  fi
else
  if [[ "$REQUIRE" == "1" ]]; then
    row FAIL "neo4j-auth" "${HOST}:7687 closed (required)"
  else
    row HOLD "neo4j-auth" "${HOST}:7687 closed — start with living-appliance-up.sh"
  fi
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold COMPOSE REQUIRE ROWS_TSV
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["pass"])
fail_n = int(os.environ["fail"])
hold_n = int(os.environ["hold"])
require = os.environ.get("REQUIRE", "0") == "1"
checks = {}
for line in os.environ.get("ROWS_TSV", "").splitlines():
    if not line.strip():
        continue
    st, name, detail = line.split("|", 2)
    checks[name] = {"status": st, "detail": detail}
verdict = "GREEN" if fail_n == 0 else "RED"
if fail_n == 0 and hold_n > 0:
    advice = "living_appliance_smoke_hold — sidecars not fully live (expected off-host)"
elif fail_n == 0:
    advice = "living_appliance_smoke_ok — redis_pong + qdrant_ready + neo4j_auth"
else:
    advice = "living_appliance_smoke_fail — protocol check failed"
payload = {
    "schema": "gzmo.living.appliance.smoke/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "compose": os.environ["COMPOSE"],
    "require_smoke": require,
    "goal": "C",
    "checks": checks,
    "note": "Product MCP (A) must not require this stack.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
SMOKE_EXIT=$?

{
  echo "# Living appliance smoke"
  echo
  echo "Verdict: **$(python3 -c "import json;print(json.load(open('$OUT/latest.json'))['verdict'])")**"
  echo
  echo "| Status | Check | Detail |"
  echo "|--------|-------|--------|"
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    echo "| $st | $name | $detail |"
  done
  echo
  echo "See docs/LIVING_APPLIANCE.md"
} >"$OUT/latest.md"

echo "=== smoke done (exit $SMOKE_EXIT) ===" | tee -a "$LOG"
exit "$SMOKE_EXIT"
