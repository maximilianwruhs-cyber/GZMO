#!/usr/bin/env bash
# Goal C: daemon-facing health against the living compose pin.
# Materializes a lab GZMO_CONFIG under data-next/ (never ~/.gzmo / product A).
# Requires redis + qdrant + distill_queue + neo4j bolt from `gzmo health`.
#
# Soft HOLD when sidecars/binary missing. FAIL when config is live but
# required sidecar probes fail.
#
#   bash scripts/living-appliance-health-smoke.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="${LIVING_APPLIANCE_DIR:-$ROOT/deploy/living-appliance}"
FRAG="${LIVING_APPLIANCE_TOML:-$ROOT/config/living-appliance.gzmo.toml.example}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
HOME_DIR="${LIVING_APPLIANCE_HOME:-$DATA/living-appliance-home}"
OUT="$DATA/living-appliance-health"
mkdir -p "$OUT" "$HOME_DIR"
LOG="$OUT/health.log"
: >"$LOG"

REQUIRE="${LIVING_APPLIANCE_REQUIRE_HEALTH:-0}"
HOST="${LIVING_APPLIANCE_HOST:-127.0.0.1}"

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

resolve_bin() {
  if [[ -n "${GZMO_BIN:-}" && -x "${GZMO_BIN}" ]]; then
    echo "$GZMO_BIN"
    return
  fi
  if [[ -x "$ROOT/target/release/gzmo" ]]; then
    echo "$ROOT/target/release/gzmo"
    return
  fi
  local cand="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo"
  if [[ -x "$cand" ]]; then
    echo "$cand"
    return
  fi
  if command -v gzmo >/dev/null 2>&1; then
    # Prefer real path over shell alias
    type -P gzmo 2>/dev/null || command -v gzmo
    return
  fi
  echo ""
}

echo "=== Living appliance health smoke (goal C) ===" | tee -a "$LOG"

BIN="$(resolve_bin)"
if [[ -z "$BIN" ]]; then
  if [[ "$REQUIRE" == "1" ]]; then
    row FAIL "gzmo-bin" "no gzmo binary (set GZMO_BIN)"
  else
    row HOLD "gzmo-bin" "no gzmo binary — build release or set GZMO_BIN"
  fi
else
  row PASS "gzmo-bin" "$BIN"
fi

if [[ ! -f "$FRAG" ]]; then
  row FAIL "toml-fragment" "missing $FRAG"
else
  row PASS "toml-fragment" "$FRAG"
fi

# Soft skip when sidecars are not up (expected off workstation without up.sh)
sidecar_ports=0
for port in 6379 6333 7687; do
  probe_tcp "$port" && sidecar_ports=$((sidecar_ports + 1)) || true
done

if [[ "$sidecar_ports" -lt 3 ]]; then
  if [[ "$REQUIRE" == "1" ]]; then
    row FAIL "sidecars" "${HOST} redis/qdrant/neo4j not all open ($sidecar_ports/3)"
  else
    row HOLD "sidecars" "${HOST} sidecars $sidecar_ports/3 — run living-appliance-up.sh"
  fi
else
  row PASS "sidecars" "${HOST} redis/qdrant/neo4j open"
fi

# Materialize lab home + merge living fragment (never product ~/.gzmo)
if [[ -n "$BIN" && -f "$FRAG" && "$sidecar_ports" -eq 3 ]]; then
  if [[ ! -f "$HOME_DIR/gzmo.toml" ]]; then
    echo "[*] gzmo init → $HOME_DIR" | tee -a "$LOG"
    "$BIN" init --force --dir "$HOME_DIR" --bin "$BIN" >>"$LOG" 2>&1 || true
  fi
  if [[ ! -f "$HOME_DIR/gzmo.toml" ]]; then
    row FAIL "lab-home" "init failed — see $LOG"
  else
    # Guard: lab home must not be ~/.gzmo
    if [[ "$(realpath "$HOME_DIR")" == "$(realpath "${HOME}/.gzmo" 2>/dev/null || echo /nonexistent)" ]]; then
      row FAIL "lab-home" "refusing to use ~/.gzmo (product A)"
    else
      NEO4J_AUTH=""
      if [[ -f "$DIR/.env" ]]; then
        NEO4J_AUTH="$(
          grep -E '^[[:space:]]*NEO4J_AUTH=' "$DIR/.env" 2>/dev/null \
            | head -1 \
            | sed -E 's/^[[:space:]]*NEO4J_AUTH=//; s/^["'\'']//; s/["'\'']$//'
        )"
      fi
      NEO4J_PASSWORD=""
      if [[ -n "$NEO4J_AUTH" && "$NEO4J_AUTH" == */* ]]; then
        NEO4J_PASSWORD="${NEO4J_AUTH#*/}"
      fi
      export HOME_DIR FRAG HOST NEO4J_PASSWORD
      python3 - <<'PY' >>"$LOG" 2>&1
from pathlib import Path
import os, re

home = Path(os.environ["HOME_DIR"])
frag = Path(os.environ["FRAG"]).read_text(encoding="utf-8")
cfg_path = home / "gzmo.toml"
text = cfg_path.read_text(encoding="utf-8")

# Enable / overwrite redis + qdrant from fragment values (localhost pin).
redis_block = f'''[redis]
enabled = true
url = "redis://{os.environ["HOST"]}:6379"
distill_queue = "gzmo:distill:pending"
distill_fallback_dir = "data/distill-queue"
'''
qdrant_block = f'''[qdrant]
enabled = true
url = "http://{os.environ["HOST"]}:6333"
collection = "honeypot"
sync_enabled = false
'''
neo_pass = os.environ.get("NEO4J_PASSWORD", "")
mcp_block = f'''[[mcp_servers]]
name = "memory"
command = "uvx"
args = ["--from", "mcp-neo4j-memory", "mcp-neo4j-memory"]

[mcp_servers.env]
NEO4J_URL = "bolt://{os.environ["HOST"]}:7687"
NEO4J_USERNAME = "neo4j"
NEO4J_PASSWORD = "{neo_pass}"
NEO4J_DATABASE = "neo4j"
'''

def upsert_section(src: str, header: str, block: str) -> str:
    # Replace existing [header] … until next top-level [ or EOF
    pat = re.compile(rf"(?ms)^\[{re.escape(header)}\]\n.*?(?=^\[|\Z)")
    if pat.search(src):
        return pat.sub(block.rstrip() + "\n\n", src, count=1)
    return src.rstrip() + "\n\n" + block.rstrip() + "\n"

text = upsert_section(text, "redis", redis_block)
text = upsert_section(text, "qdrant", qdrant_block)
# Drop prior memory mcp server blocks if present, then append pin.
text = re.sub(
    r"(?ms)^\[\[mcp_servers\]\]\nname = \"memory\"\n.*?(?=^\[\[|\Z)",
    "",
    text,
)
text = re.sub(r"(?ms)^\[mcp_servers\.env\]\n(?:NEO4J_[^\n]+\n)+", "", text)
text = text.rstrip() + "\n\n" + mcp_block.rstrip() + "\n"
# Marker so we never confuse with product A
if "living-appliance-lab" not in text:
    text = (
        "# living-appliance-lab — goal C only; not product ~/.gzmo\n"
        + text
    )
cfg_path.write_text(text, encoding="utf-8")
print(f"wrote {cfg_path}")
PY
      row PASS "lab-home" "$HOME_DIR/gzmo.toml (living fragment applied)"
    fi
  fi
elif [[ -n "$BIN" && -f "$FRAG" ]]; then
  row HOLD "lab-home" "skipped — sidecars not fully open"
fi

# Run gzmo health against lab config
if [[ -n "$BIN" && -f "$HOME_DIR/gzmo.toml" && "$sidecar_ports" -eq 3 ]]; then
  REPORT="$OUT/health-report.txt"
  set +e
  GZMO_CONFIG="$HOME_DIR/gzmo.toml" \
    GZMO_ALLOW_LAB_VAULT=1 \
    env -u GZMO_PRODUCT \
    "$BIN" health >"$REPORT" 2>>"$LOG"
  health_rc=$?
  set -e
  {
    echo "--- health report (rc=$health_rc) ---"
    cat "$REPORT"
  } >>"$LOG"

  check_probe() {
    local name="$1"
    if grep -E "^\s*\[OK\] ${name} " "$REPORT" >/dev/null 2>&1 \
      || grep -E "^\s*\[OK\] ${name} —" "$REPORT" >/dev/null 2>&1; then
      row PASS "probe:${name}" "$(grep -E "^\s*\[OK\] ${name}" "$REPORT" | head -1 | sed 's/^[[:space:]]*//')"
    elif grep -E "^\s*\[FAIL\] ${name}" "$REPORT" >/dev/null 2>&1; then
      row FAIL "probe:${name}" "$(grep -E "^\s*\[FAIL\] ${name}" "$REPORT" | head -1 | sed 's/^[[:space:]]*//')"
    else
      row FAIL "probe:${name}" "missing from health report"
    fi
  }

  check_probe redis
  check_probe qdrant
  check_probe distill_queue
  check_probe neo4j
  # mcp_memory is optional (uvx/neo4j-memory may be absent) — note only
  if grep -E "^\s*\[OK\] mcp_memory" "$REPORT" >/dev/null 2>&1; then
    row PASS "probe:mcp_memory" "registered"
  else
    row HOLD "probe:mcp_memory" "optional — bolt neo4j is enough for appliance pin"
  fi
elif [[ "$sidecar_ports" -eq 3 && -z "$BIN" ]]; then
  row HOLD "probes" "skipped — no binary"
fi

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold HOME_DIR REQUIRE ROWS_TSV
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
required_ok = all(
    checks.get(n, {}).get("status") == "PASS"
    for n in ("probe:redis", "probe:qdrant", "probe:distill_queue", "probe:neo4j")
)
blocking_holds = [
    n for n, c in checks.items()
    if c.get("status") == "HOLD" and n != "probe:mcp_memory"
]
if fail_n == 0 and required_ok and not blocking_holds:
    advice = "living_appliance_health_ok — redis + qdrant + distill_queue + neo4j"
elif fail_n == 0 and hold_n > 0:
    advice = "living_appliance_health_hold — binary/sidecars not fully ready"
elif fail_n == 0:
    advice = "living_appliance_health_ok — redis + qdrant + distill_queue + neo4j"
else:
    advice = "living_appliance_health_fail — required sidecar probe failed"
payload = {
    "schema": "gzmo.living.appliance.health/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "lab_home": os.environ.get("HOME_DIR", ""),
    "require_health": require,
    "goal": "C",
    "checks": checks,
    "note": "Lab home under data-next only — never product ~/.gzmo (goal A).",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
HEALTH_EXIT=$?

{
  echo "# Living appliance health smoke"
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
  echo "Lab home: \`$HOME_DIR\` (not ~/.gzmo)"
  echo
  echo "See docs/LIVING_APPLIANCE.md"
} >"$OUT/latest.md"

echo "=== health smoke done (exit $HEALTH_EXIT) ===" | tee -a "$LOG"
exit "$HEALTH_EXIT"
