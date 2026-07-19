#!/usr/bin/env bash
# Goal C: protocol smoke against CT101 living sidecars (not workstation throwaway).
# Redis PING / Qdrant ready / Neo4j cypher via remote .env — never prints secrets.
#
#   bash scripts/ct101-living-appliance-smoke.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
CLUSTER="${CT101_DATABASE_CLUSTER:-/opt/database-cluster}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/living-appliance-smoke"
mkdir -p "$OUT"
LOG="$OUT/smoke.log"
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

echo "=== CT101 living appliance smoke (goal C) ===" | tee -a "$LOG"

set +e
REMOTE_OUT="$(
  ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
    "CLUSTER=$(printf '%q' "$CLUSTER") bash -s" <<'REMOTE'
set -euo pipefail
CLUSTER="${CLUSTER:?}"
status=0
echo "compose-file|PASS|$CLUSTER/docker-compose.yml"

if docker exec sidecar-redis redis-cli ping 2>/dev/null | grep -qi '^PONG$'; then
  echo "redis-pong|PASS|PONG via sidecar-redis"
else
  echo "redis-pong|FAIL|redis PING failed"
  status=1
fi

if curl -fsS --max-time 5 http://127.0.0.1:6333/readyz >/dev/null 2>&1 \
  || curl -fsS --max-time 5 http://127.0.0.1:6333/ >/dev/null 2>&1; then
  echo "qdrant-ready|PASS|http://127.0.0.1:6333 ready"
else
  echo "qdrant-ready|FAIL|qdrant HTTP probe failed"
  status=1
fi

ENV_FILE=""
for f in "$CLUSTER/.env" /opt/gzmo/current/deploy/living-appliance/.env; do
  if [[ -f "$f" ]]; then ENV_FILE="$f"; break; fi
done
AUTH=""
if [[ -n "$ENV_FILE" ]]; then
  AUTH="$(grep -E '^[[:space:]]*NEO4J_AUTH=' "$ENV_FILE" | head -1 | sed -E 's/^[[:space:]]*NEO4J_AUTH=//; s/^["'\'']//; s/["'\'']$//')"
fi
if [[ -z "$AUTH" ]]; then
  AUTH="$(docker inspect sidecar-neo4j --format '{{range .Config.Env}}{{println .}}{{end}}' 2>/dev/null | grep '^NEO4J_AUTH=' | head -1 | cut -d= -f2- || true)"
fi
if [[ -z "$AUTH" || "$AUTH" != neo4j/* ]]; then
  echo "neo4j-auth|FAIL|NEO4J_AUTH unresolved on CT101"
  status=1
else
  user="${AUTH%%/*}"
  pass="${AUTH#*/}"
  if docker exec sidecar-neo4j cypher-shell -u "$user" -p "$pass" 'RETURN 1 AS ok;' >/dev/null 2>&1; then
    echo "neo4j-auth|PASS|cypher-shell RETURN 1 ok"
  else
    echo "neo4j-auth|FAIL|cypher-shell auth/query failed"
    status=1
  fi
fi
exit "$status"
REMOTE
)"
ssh_rc=$?
set -e

if [[ "$ssh_rc" -ne 0 && -z "$REMOTE_OUT" ]]; then
  row HOLD "ssh" "cannot reach $HOST"
else
  row PASS "ssh" "$HOST reachable"
  while IFS= read -r line; do
    [[ -z "$line" ]] && continue
    IFS='|' read -r name st detail <<<"$line"
    # remote emits name|status|detail
    row "$st" "$name" "$detail"
  done <<<"$REMOTE_OUT"
fi

# If ssh failed mid-way, remote may have mixed exit — trust FAIL rows
ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV HOST
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
# Recompute from rows (authoritative)
checks = {}
pass_n = fail_n = hold_n = 0
for line in os.environ.get("ROWS_TSV", "").splitlines():
    if not line.strip():
        continue
    st, name, detail = line.split("|", 2)
    checks[name] = {"status": st, "detail": detail}
    if st == "PASS":
        pass_n += 1
    elif st == "FAIL":
        fail_n += 1
    else:
        hold_n += 1
verdict = "GREEN" if fail_n == 0 else "RED"
if fail_n == 0 and hold_n > 0:
    advice = "living_appliance_smoke_hold — CT101 smoke incomplete"
elif fail_n == 0:
    advice = "living_appliance_smoke_ok — CT101 redis_pong + qdrant_ready + neo4j_auth"
else:
    advice = "living_appliance_smoke_fail — CT101 protocol check failed"
payload = {
    "schema": "gzmo.living.appliance.smoke/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "host": os.environ.get("HOST", ""),
    "target": "ct101",
    "goal": "C",
    "checks": checks,
    "note": "Workstation Neo4j is throwaway — this smoke targets CT101.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = ["# CT101 living appliance smoke", "", f"Verdict: **{verdict}**", "", "| Status | Check | Detail |", "|--------|-------|--------|"]
for n, c in checks.items():
    md.append(f"| {c['status']} | {n} | {c['detail']} |")
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
SMOKE_EXIT=$?
echo "=== CT101 smoke done (exit $SMOKE_EXIT) ===" | tee -a "$LOG"
exit "$SMOKE_EXIT"
