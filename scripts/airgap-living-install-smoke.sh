#!/usr/bin/env bash
# Airgap living install smoke — stranger/one-box path readiness (USP ADR-0004).
# Proves the *install path* is demable. Never claims keep-quality living GREEN
# when sidecars/LLM are missing (honest degrade → lite/incomplete).
#
#   bash scripts/airgap-living-install-smoke.sh
# Artifact: data-next/airgap-living-install/latest.{json,md}
#
# Optional:
#   AIRGAP_SMOKE_REQUIRE_LIVE=1  — FAIL if Redis/Qdrant/Neo4j not reachable
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/airgap-living-install"
COMPOSE_DIR="${LIVING_APPLIANCE_DIR:-$ROOT/deploy/living-appliance}"
TOML_EX="${LIVING_APPLIANCE_TOML:-$ROOT/config/living-appliance.gzmo.toml.example}"
LIVING_HOME="${GZMO_LIVING_HOME:-${HOME}/.gzmo-living}"
REQUIRE_LIVE="${AIRGAP_SMOKE_REQUIRE_LIVE:-0}"
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

echo "=== Airgap living install smoke ===" | tee -a "$LOG"

# Doctrine / installer surfaces
[[ -f "$ROOT/docs/AIRGAP_LIVING.md" ]] && row PASS "doc-airgap" "docs/AIRGAP_LIVING.md" || row FAIL "doc-airgap" "missing"
[[ -f "$ROOT/docs/ADR-0004-airgap-living-usp.md" ]] && row PASS "doc-adr" "ADR-0004" || row FAIL "doc-adr" "missing ADR-0004"
[[ -x "$ROOT/scripts/install-living-airgap.sh" ]] && row PASS "installer" "install-living-airgap.sh" || row FAIL "installer" "missing installer"
[[ -x "$ROOT/scripts/living-appliance-gate.sh" ]] && row PASS "appliance-gate" "living-appliance-gate.sh" || row FAIL "appliance-gate" "missing"
[[ -f "$COMPOSE_DIR/docker-compose.yml" ]] && row PASS "compose-pin" "$COMPOSE_DIR/docker-compose.yml" || row FAIL "compose-pin" "missing compose"
[[ -f "$COMPOSE_DIR/.env.example" ]] && row PASS "env-example" ".env.example (no committed secrets)" || row FAIL "env-example" "missing"
[[ -f "$TOML_EX" ]] && row PASS "toml-example" "$TOML_EX" || row FAIL "toml-example" "missing living toml example"

# Compose must not hardcode secrets
if [[ -f "$COMPOSE_DIR/docker-compose.yml" ]]; then
  if rg -n 'NEO4J_AUTH=neo4j/[^$\{]' "$COMPOSE_DIR/docker-compose.yml" >/dev/null 2>&1 \
    || rg -ni 'Easycheesy|password123|changeme!' "$COMPOSE_DIR/docker-compose.yml" >/dev/null 2>&1; then
    row FAIL "compose-secrets" "hardcoded auth in compose"
  else
    row PASS "compose-secrets" "NEO4J_AUTH via env"
  fi
fi

# ADR-0003: refuse second overnight writer on this workstation
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
DAEMON_USER="$(systemctl --user is-active gzmo-daemon.service 2>/dev/null || true)"
DAEMON_USER="$(printf '%s\n' "$DAEMON_USER" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  row FAIL "dual-writer" "gzmo-serve active — install-living-airgap must refuse (ADR-0003)"
else
  row PASS "dual-writer" "serve=${SERVE:-inactive} (installer refuses when active)"
fi
if [[ "$DAEMON_USER" == "active" ]]; then
  row HOLD "local-daemon" "gzmo-daemon user unit active — ensure THIS box is the sole overnight writer"
else
  row PASS "local-daemon" "no user gzmo-daemon (ok for CT101-as-living workstation)"
fi

# Installer source contains dual-writer die
if rg -q 'gzmo-serve is active' "$ROOT/scripts/install-living-airgap.sh" \
  && rg -q 'ADR-0003' "$ROOT/scripts/install-living-airgap.sh"; then
  row PASS "installer-refuse" "install-living-airgap encodes ADR-0003 refuse"
else
  row FAIL "installer-refuse" "installer missing dual-writer refuse text"
fi

# MCP fragment contract: stdio / local binary (example or generated)
MCP_OK=0
if [[ -f "$LIVING_HOME/mcp-living.fragment.json" ]]; then
  if python3 -c "
import json
d=json.load(open('$LIVING_HOME/mcp-living.fragment.json'))
s=(d.get('mcpServers') or {}).get('gzmo-living') or {}
cmd=s.get('command') or ''
args=s.get('args') or []
ok=bool(cmd) and 'mcp-serve' in args and not str(cmd).startswith('http')
raise SystemExit(0 if ok else 1)
"; then
    row PASS "mcp-fragment" "$LIVING_HOME/mcp-living.fragment.json (stdio gzmo-living)"
    MCP_OK=1
  else
    row HOLD "mcp-fragment" "fragment present but not stdio gzmo-living shape"
  fi
else
  row HOLD "mcp-fragment" "no ~/.gzmo-living fragment yet — run install-living-airgap.sh"
fi

# Soft living-appliance gate (compose validity; live probes soft unless required)
bash "$ROOT/scripts/living-appliance-gate.sh" >>"$LOG" 2>&1 || true
LA="$DATA/living-appliance/latest.json"
if [[ -f "$LA" ]] && python3 -c "import json;d=json.load(open('$LA')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "appliance-gate-run" "$(python3 -c "import json;print(json.load(open('$LA')).get('advice','ok'))")"
else
  row HOLD "appliance-gate-run" "living-appliance-gate soft — see data-next/living-appliance/"
fi

# Live sidecar probes (honest degrade — never living GREEN if down)
live_redis=0
live_qdrant=0
live_neo=0
if command -v redis-cli >/dev/null 2>&1 && redis-cli -h 127.0.0.1 -p 6379 ping 2>/dev/null | grep -qi pong; then
  live_redis=1
fi
if curl -sf --max-time 2 http://127.0.0.1:6333/readyz >/dev/null 2>&1 \
  || curl -sf --max-time 2 http://127.0.0.1:6333/ >/dev/null 2>&1; then
  live_qdrant=1
fi
# Neo4j bolt is harder without cypher-shell; compose health is enough soft signal
if docker ps --format '{{.Names}}' 2>/dev/null | grep -qiE 'neo4j|living'; then
  live_neo=1
fi

live_n=$((live_redis + live_qdrant + live_neo))
if (( live_n == 3 )); then
  row PASS "sidecars-live" "redis+qdrant+neo4j signals present on localhost"
elif (( live_n > 0 )); then
  if [[ "$REQUIRE_LIVE" == "1" ]]; then
    row FAIL "sidecars-live" "partial sidecars ($live_n/3) — AIRGAP_SMOKE_REQUIRE_LIVE=1"
  else
    row HOLD "sidecars-live" "partial sidecars ($live_n/3) — lite/incomplete, not living GREEN"
  fi
else
  if [[ "$REQUIRE_LIVE" == "1" ]]; then
    row FAIL "sidecars-live" "no localhost sidecars — AIRGAP_SMOKE_REQUIRE_LIVE=1"
  else
    row HOLD "sidecars-live" "no localhost sidecars — install path ok; living claim forbidden"
  fi
fi

# Local LLM honesty soft probe (Prime)
llm_ok=0
if curl -sf --max-time 2 http://127.0.0.1:8000/health >/dev/null 2>&1 \
  || curl -sf --max-time 2 http://127.0.0.1:8000/v1/models >/dev/null 2>&1; then
  llm_ok=1
  row PASS "local-llm" "localhost:8000 responds"
else
  row HOLD "local-llm" "no localhost LLM — overnight must pause, not fail-open to cloud"
fi

# Mode classification — never emit living_green from this smoke alone
mode="install_path_ready"
living_claim="forbidden"
if (( fail > 0 )); then
  mode="install_path_broken"
  living_claim="forbidden"
elif (( live_n == 3 && llm_ok == 1 )); then
  mode="box_looks_living_capable"
  living_claim="unproven — run keep-quality-gate.sh on THIS box only if sole writer"
elif (( live_n == 3 )); then
  mode="sidecars_up_llm_missing"
  living_claim="forbidden — incomplete living"
elif (( live_n > 0 )); then
  mode="partial_sidecars"
  living_claim="forbidden — lite/incomplete"
else
  mode="install_path_ready_offline"
  living_claim="forbidden — no sidecars (lite only)"
fi

export OUT pass fail hold mode living_claim live_n llm_ok MCP_OK REQUIRE_LIVE
set +e
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["pass"])
fail_n = int(os.environ["fail"])
hold_n = int(os.environ["hold"])
mode = os.environ["mode"]
living_claim = os.environ["living_claim"]
verdict = "GREEN" if fail_n == 0 else "RED"
# Explicit: GREEN here means install/smoke path, NOT keep-quality living GREEN
advice = (
    f"airgap_install_smoke_ok — mode={mode}; living_claim={living_claim}"
    if verdict == "GREEN"
    else "airgap_install_smoke_hold — fix FAIL rows (never claim living GREEN for lite)"
)
payload = {
    "schema": "gzmo.usp.airgap_living_install_smoke/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "mode": mode,
    "living_claim": living_claim,
    "living_green_claimed": False,
    "keep_quality_required_for_living": True,
    "sidecars_live_count": int(os.environ.get("live_n") or 0),
    "local_llm": bool(int(os.environ.get("llm_ok") or 0)),
    "mcp_fragment_ok": bool(int(os.environ.get("MCP_OK") or 0)),
    "require_live": os.environ.get("REQUIRE_LIVE") == "1",
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "doc": "docs/AIRGAP_LIVING.md",
    "operator": [
        "GREEN = install path demable, not keep-quality living GREEN",
        "Missing sidecars/LLM ⇒ lite/incomplete — do not enable overnight writer",
        "If CT101 (or another host) owns metabolism, do not start daemon here (ADR-0003)",
        "Full living proof: bash scripts/keep-quality-gate.sh on the sole-writer box",
    ],
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Airgap living install smoke",
    "",
    f"Verdict: **{verdict}** (install path — not keep-quality)",
    "",
    f"- Mode: `{mode}`",
    f"- Living claim: **{living_claim}**",
    f"- living_green_claimed: **false**",
    f"- Advice: {advice}",
    "",
    "See docs/AIRGAP_LIVING.md · ADR-0004",
    "",
]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({
    "verdict": verdict,
    "advice": advice,
    "mode": mode,
    "living_claim": living_claim,
    "living_green_claimed": False,
    "pass": pass_n,
    "fail": fail_n,
    "hold": hold_n,
}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

{
  echo "# Airgap living install smoke"
  echo
  echo "Verdict: **$(python3 -c "import json;print(json.load(open('$OUT/latest.json'))['verdict'])")**"
  echo
  echo "| Status | Check | Detail |"
  echo "|--------|-------|--------|"
  for r in "${ROWS[@]}"; do
    IFS='|' read -r st name detail <<<"$r"
    detail="${detail//|/\\|}"
    echo "| $st | $name | $detail |"
  done
  echo
  echo "living_green_claimed: false — see docs/AIRGAP_LIVING.md"
  echo
} >"$OUT/latest.md"

echo "=== airgap-living-install-smoke done (exit $GATE_EXIT) ===" | tee -a "$LOG"
exit "$GATE_EXIT"
