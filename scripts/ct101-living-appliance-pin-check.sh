#!/usr/bin/env bash
# Goal C: compare staged living-appliance pin on CT101 vs live /opt/database-cluster.
# Shape only (images, ports, container_name, NEO4J_AUTH style). Never reads .env / passwords.
#
# Soft HOLD on SSH miss or expected pre-promote drift.
# FAIL when staged pin missing, or when CT101_PIN_REQUIRE=1 and drift remains.
#
#   bash scripts/ct101-living-appliance-pin-check.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
STAGED="${CT101_LIVING_APPLIANCE_DEST:-/opt/gzmo/current/deploy/living-appliance}/docker-compose.yml"
LIVE="${CT101_DATABASE_CLUSTER_COMPOSE:-/opt/database-cluster/docker-compose.yml}"
LOCAL_PIN="$ROOT/deploy/living-appliance/docker-compose.yml"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/living-appliance-pin-ct101"
mkdir -p "$OUT"
LOG="$OUT/check.log"
: >"$LOG"

REQUIRE="${CT101_PIN_REQUIRE:-0}"

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

echo "=== CT101 living appliance pin check (goal C) ===" | tee -a "$LOG"

if [[ ! -f "$LOCAL_PIN" ]]; then
  row FAIL "local-pin" "missing $LOCAL_PIN"
else
  row PASS "local-pin" "$LOCAL_PIN"
fi

STAGED_FILE="$OUT/staged.compose.yml"
LIVE_FILE="$OUT/live.compose.yml"
rm -f "$STAGED_FILE" "$LIVE_FILE"

set +e
ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "test -f $(printf '%q' "$STAGED")" >/dev/null 2>&1
staged_rc=$?
ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "test -f $(printf '%q' "$LIVE")" >/dev/null 2>&1
live_rc=$?
# Reachability: if both ssh fail hard, treat as SSH down
ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" "true" >/dev/null 2>&1
ssh_ok=$?
set -e

if [[ "$ssh_ok" -ne 0 ]]; then
  if [[ "$REQUIRE" == "1" ]]; then
    row FAIL "ssh" "cannot reach $HOST (required)"
  else
    row HOLD "ssh" "cannot reach $HOST — soft skip"
  fi
else
  row PASS "ssh" "$HOST reachable"

  if [[ "$staged_rc" -eq 0 ]]; then
    scp -q -o ConnectTimeout=12 -o BatchMode=yes "${HOST}:${STAGED}" "$STAGED_FILE" 2>>"$LOG" || true
  fi
  if [[ "$live_rc" -eq 0 ]]; then
    scp -q -o ConnectTimeout=12 -o BatchMode=yes "${HOST}:${LIVE}" "$LIVE_FILE" 2>>"$LOG" || true
  fi

  # Redact literal passwords only — keep ${NEO4J_AUTH…} substitution intact
  for f in "$STAGED_FILE" "$LIVE_FILE"; do
    if [[ -f "$f" ]]; then
      sed -E 's/(NEO4J_AUTH=neo4j\/)[^[:space:]"]+/\1<redacted>/' "$f" >"${f}.redacted"
      mv "${f}.redacted" "$f"
    fi
  done

  if [[ -f "$STAGED_FILE" ]]; then
    row PASS "staged-pin" "$STAGED"
  else
    row FAIL "staged-pin" "missing $STAGED — run ct101-sync-living-appliance.sh"
  fi

  if [[ -f "$LIVE_FILE" ]]; then
    row PASS "live-compose" "$LIVE"
  else
    if [[ "$REQUIRE" == "1" ]]; then
      row FAIL "live-compose" "missing $LIVE"
    else
      row HOLD "live-compose" "missing $LIVE"
    fi
  fi

  if [[ -f "$STAGED_FILE" && -f "$LIVE_FILE" ]]; then
    export STAGED_FILE LIVE_FILE LOCAL_PIN REQUIRE OUT
    # shell rows already printed; python adds drift rows + writes final artifact
    DRIFT_ROWS="$(
      python3 - <<'PY'
import json, os, re, sys
from pathlib import Path

staged = Path(os.environ["STAGED_FILE"]).read_text(encoding="utf-8")
live = Path(os.environ["LIVE_FILE"]).read_text(encoding="utf-8")
local = Path(os.environ["LOCAL_PIN"]).read_text(encoding="utf-8")
require = os.environ.get("REQUIRE", "0") == "1"

rows = []
drift = []

def emit(status, name, detail):
    rows.append((status, name, detail))
    print(f"{status}|{name}|{detail}")

def soft(name, detail, ok):
    if ok:
        emit("PASS", name, detail)
    elif require:
        emit("FAIL", name, detail)
        drift.append(name)
    else:
        emit("HOLD", name, detail)
        drift.append(name)

def service_block(text, name):
    m = re.search(rf"(?ms)^  {re.escape(name)}:\n(.*?)(?=^  \w|\Z)", text)
    return m.group(0) if m else ""

def field(block, key):
    m = re.search(rf"(?m)^\s+{re.escape(key)}:\s*(.+)$", block)
    return m.group(1).strip().strip('"') if m else ""

def ports(block):
    return sorted(set(re.findall(r'"(\d+):\d+"', block)))

def neo_auth_style(block):
    if re.search(r"NEO4J_AUTH=\$\{NEO4J_AUTH", block):
        return "env_substitution"
    if re.search(r"NEO4J_AUTH=neo4j/", block):
        return "inline_literal"
    if "NEO4J_AUTH=" in block:
        return "other"
    return "missing"

want_ports = {
    "redis": {"6379"},
    "qdrant": {"6333", "6334"},
    "neo4j": {"7474", "7687"},
}

for svc in ("redis", "qdrant", "neo4j"):
    sb, lb = service_block(staged, svc), service_block(live, svc)
    if not sb:
        emit("FAIL", f"staged:{svc}", "service missing in staged pin")
        continue
    if not lb:
        soft(f"live:{svc}", "service missing in live cluster compose", False)
        continue
    emit("PASS", f"service:{svc}", "present in staged + live")

    sc, lc = field(sb, "container_name"), field(lb, "container_name")
    soft(f"name:{svc}", f"staged={sc or '?'} live={lc or '?'}", bool(sc and lc and sc == lc))

    si, li = field(sb, "image"), field(lb, "image")
    soft(f"image:{svc}", f"staged={si or '?'} live={li or '?'}", bool(si and li and si == li))

    sp, lp = set(ports(sb)), set(ports(lb))
    w = want_ports[svc]
    soft(f"ports:{svc}", f"staged={sorted(sp)} live={sorted(lp)}", w.issubset(sp) and w.issubset(lp))

sa = neo_auth_style(service_block(staged, "neo4j"))
la = neo_auth_style(service_block(live, "neo4j"))
if sa == "env_substitution":
    emit("PASS", "staged-secrets", "NEO4J_AUTH via ${NEO4J_AUTH}")
else:
    emit("FAIL", "staged-secrets", f"staged auth style={sa}")

if la == "env_substitution":
    emit("PASS", "live-secrets", "live uses env substitution")
elif la == "inline_literal":
    soft("live-secrets", "live compose still has inline NEO4J_AUTH — migrate on promote", False)
else:
    soft("live-secrets", f"live auth style={la}", False)

local_q = field(service_block(local, "qdrant"), "image")
staged_q = field(service_block(staged, "qdrant"), "image")
soft("sync-fresh", f"local={local_q or '?'} staged={staged_q or '?'}", bool(local_q and staged_q and local_q == staged_q))

# Write side channel for bash
Path(os.environ["OUT"], "drift.json").write_text(
    json.dumps({"rows": rows, "drift": drift}, indent=2) + "\n", encoding="utf-8"
)
PY
    )"

    while IFS= read -r line; do
      [[ -z "$line" ]] && continue
      IFS='|' read -r st name detail <<<"$line"
      row "$st" "$name" "$detail"
    done <<<"$DRIFT_ROWS"
  fi
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold REQUIRE ROWS_TSV HOST STAGED LIVE LOCAL_PIN
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

drift_path = out / "drift.json"
drift = []
if drift_path.is_file():
    drift = json.loads(drift_path.read_text()).get("drift", [])

verdict = "GREEN" if fail_n == 0 else "RED"
if fail_n == 0 and hold_n > 0:
    advice = "living_pin_ct101_hold — staged ok; pre-promote drift vs live (expected)"
elif fail_n == 0:
    advice = "living_pin_ct101_ok — staged matches live shape"
else:
    advice = "living_pin_ct101_fail — staged pin missing or invalid"

payload = {
    "schema": "gzmo.living.appliance.pin_ct101/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "drift": drift,
    "paths": {
        "host": os.environ.get("HOST", ""),
        "staged": os.environ.get("STAGED", ""),
        "live": os.environ.get("LIVE", ""),
        "local": os.environ.get("LOCAL_PIN", ""),
    },
    "require": require,
    "goal": "C",
    "checks": checks,
    "note": "Never reads .env. Workstation Neo4j is throwaway.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

md = [
    "# CT101 living appliance pin check",
    "",
    f"Verdict: **{verdict}**",
    "",
    "| Status | Check | Detail |",
    "|--------|-------|--------|",
]
for name, c in checks.items():
    md.append(f"| {c['status']} | {name} | {c['detail']} |")
md += ["", f"Drift: `{', '.join(drift) if drift else 'none'}`", "", "See docs/LIVING_APPLIANCE.md"]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n, "drift": drift}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
CHECK_EXIT=$?

echo "=== pin check done (exit $CHECK_EXIT) ===" | tee -a "$LOG"
exit "$CHECK_EXIT"
