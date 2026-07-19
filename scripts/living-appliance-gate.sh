#!/usr/bin/env bash
# Goal C soft/hard checks for the in-repo living appliance pin.
# Exit 0 when compose pin is valid. Sidecar live probes are soft HOLD unless
# LIVING_APPLIANCE_REQUIRE_LIVE=1.
#
#   bash scripts/living-appliance-gate.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_DIR="${LIVING_APPLIANCE_DIR:-$ROOT/deploy/living-appliance}"
COMPOSE="$COMPOSE_DIR/docker-compose.yml"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/living-appliance"
mkdir -p "$OUT"
LOG="$OUT/gate.log"
: >"$LOG"

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

echo "=== Living appliance gate (goal C) ===" | tee -a "$LOG"

# 1) Compose file present
if [[ -f "$COMPOSE" ]]; then
  row PASS "compose-file" "$COMPOSE"
else
  row FAIL "compose-file" "missing $COMPOSE"
fi

# 2) No committed secrets in compose
if [[ -f "$COMPOSE" ]]; then
  if rg -n 'NEO4J_AUTH=neo4j/[^$\{]' "$COMPOSE" >/dev/null 2>&1 \
    || rg -ni 'Easycheesy|password123|changeme!' "$COMPOSE" >/dev/null 2>&1; then
    row FAIL "compose-secrets" "hardcoded Neo4j auth in compose — use \${NEO4J_AUTH}"
  else
    row PASS "compose-secrets" "NEO4J_AUTH via env substitution"
  fi
fi

# 3) .env.example present
if [[ -f "$COMPOSE_DIR/.env.example" ]]; then
  row PASS "env-example" "$COMPOSE_DIR/.env.example"
else
  row FAIL "env-example" "missing .env.example"
fi

# 3b) Daemon config fragment present
FRAG="${LIVING_APPLIANCE_TOML:-$ROOT/config/living-appliance.gzmo.toml.example}"
if [[ -f "$FRAG" ]] && rg -q '\[redis\]' "$FRAG" && rg -q '\[qdrant\]' "$FRAG"; then
  row PASS "toml-fragment" "$FRAG"
else
  row FAIL "toml-fragment" "missing redis/qdrant fragment at $FRAG"
fi

# 4) docker compose config (needs a throwaway .env if none)
if [[ -f "$COMPOSE" ]] && command -v docker >/dev/null 2>&1; then
  tmpenv="$(mktemp)"
  if [[ -f "$COMPOSE_DIR/.env" ]]; then
    # validate against operator .env without printing it
    if (cd "$COMPOSE_DIR" && docker compose -f docker-compose.yml config >/dev/null 2>>"$LOG"); then
      row PASS "compose-config" "docker compose config ok"
    else
      row FAIL "compose-config" "docker compose config failed — see gate.log"
    fi
  else
    printf 'NEO4J_AUTH=neo4j/gate-check-only\n' >"$tmpenv"
    if (cd "$COMPOSE_DIR" && docker compose --env-file "$tmpenv" -f docker-compose.yml config >/dev/null 2>>"$LOG"); then
      row PASS "compose-config" "docker compose config ok (ephemeral env)"
    else
      row HOLD "compose-config" "docker compose config failed — docker/plugin issue?"
    fi
  fi
  rm -f "$tmpenv"
elif [[ -f "$COMPOSE" ]]; then
  row HOLD "compose-config" "docker not on PATH — skipped"
fi

# 5) Optional live TCP probes (localhost living host)
probe_tcp() {
  local host="$1" port="$2"
  timeout 2 bash -c "echo >/dev/tcp/${host}/${port}" 2>/dev/null
}

REQUIRE_LIVE="${LIVING_APPLIANCE_REQUIRE_LIVE:-0}"
PROBE_HOST="${LIVING_APPLIANCE_HOST:-127.0.0.1}"
live_open=0
for pair in "redis:6379" "qdrant:6333" "neo4j:7687"; do
  name="${pair%%:*}"
  port="${pair##*:}"
  if probe_tcp "$PROBE_HOST" "$port"; then
    row PASS "live:${name}" "${PROBE_HOST}:${port} open"
    live_open=$((live_open + 1))
  else
    if [[ "$REQUIRE_LIVE" == "1" ]]; then
      row FAIL "live:${name}" "${PROBE_HOST}:${port} closed (required)"
    else
      row HOLD "live:${name}" "${PROBE_HOST}:${port} closed — expected off workstation; CT101 has live stack"
    fi
  fi
done

# When all TCP ports are open (or live required), also require protocol smoke.
if [[ "$live_open" -eq 3 || "$REQUIRE_LIVE" == "1" ]]; then
  set +e
  bash "$ROOT/scripts/living-appliance-smoke.sh" >>"$LOG" 2>&1
  smoke_rc=$?
  set -e
  smoke_json="$DATA/living-appliance-smoke/latest.json"
  advice="$(python3 -c "import json;print(json.load(open('$smoke_json')).get('advice',''))" 2>/dev/null || echo smoke_ran)"
  smoke_hold="$(python3 -c "import json;print(json.load(open('$smoke_json')).get('counts',{}).get('hold',0))" 2>/dev/null || echo 0)"
  if [[ "$smoke_rc" -ne 0 ]]; then
    if [[ "$REQUIRE_LIVE" == "1" ]] || [[ "${LIVING_APPLIANCE_REQUIRE_SMOKE:-0}" == "1" ]]; then
      row FAIL "protocol-smoke" "see data-next/living-appliance-smoke/"
    else
      row FAIL "protocol-smoke" "ports open but protocol smoke failed — see living-appliance-smoke/"
    fi
  elif [[ "$smoke_hold" != "0" ]]; then
    if [[ "$REQUIRE_LIVE" == "1" ]] || [[ "${LIVING_APPLIANCE_REQUIRE_SMOKE:-0}" == "1" ]]; then
      row FAIL "protocol-smoke" "$advice"
    else
      row HOLD "protocol-smoke" "$advice"
    fi
  else
    row PASS "protocol-smoke" "$advice"
  fi
else
  row HOLD "protocol-smoke" "skipped — not all sidecar ports open"
fi

export OUT pass fail hold COMPOSE
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["pass"])
fail_n = int(os.environ["fail"])
hold_n = int(os.environ["hold"])
verdict = "GREEN" if fail_n == 0 else "RED"
advice = (
    "living_appliance_pin_ok — compose pin valid (live probes may HOLD off-host)"
    if verdict == "GREEN"
    else "living_appliance_pin_fail — fix compose pin before claiming goal C"
)
payload = {
    "schema": "gzmo.living.appliance/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "compose": os.environ["COMPOSE"],
    "goal": "C",
    "note": "Product MCP (A) must not require this stack.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?

{
  echo "# Living appliance gate"
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

echo "=== gate done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
