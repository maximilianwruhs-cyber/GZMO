#!/usr/bin/env bash
# Full-stack audit + tuning scorecard for GZMO + Pi-Mentor-Discovery.
# Exit 0 = no hard failures; exit 1 = at least one [FAIL].
#
# Usage:
#   ./scripts/system-audit.sh           # ~2–4 min
#   SYSTEM_AUDIT_QUICK=1 ./scripts/system-audit.sh   # skip verify-production
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
BIN="${ROOT}/target/release/gzmo"
[[ -x "$BIN" ]] || BIN="${ROOT}/target/debug/gzmo"
SKILLS="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
DISCOVERY_DATA="${PI_MENTOR_DISCOVERY_DATA:-$SKILLS/data/pi-mentor-discovery}"
FAIL=0
WARN=0

pass() { echo "[PASS] $*"; }
fail() { echo "[FAIL] $*"; FAIL=$((FAIL + 1)); }
warn() { echo "[WARN] $*"; WARN=$((WARN + 1)); }
section() { echo; echo "════════════════════════════════════════"; echo " $*"; echo "════════════════════════════════════════"; }

section "A — ARCH-DIR / Sovereignty"
if ./scripts/sovereignty-verify.sh; then pass "sovereignty-verify"; else fail "sovereignty-verify"; fi

section "B — Mechanics (Obolus, hooks, spawn chain)"
MECH="${ROOT}/scripts/mechanics-verify.sh"
[[ -x "$MECH" ]] || MECH="${SKILLS}/scripts/mechanics-verify.sh"
if [[ -x "$MECH" ]]; then
  if "$MECH"; then pass "mechanics-verify"; else fail "mechanics-verify"; fi
else
  warn "mechanics-verify.sh missing at $MECH"
fi

section "C — Production E2E (Prime, embed, vault, Neo4j)"
if [[ "${SYSTEM_AUDIT_QUICK:-0}" == "1" ]]; then
  warn "SYSTEM_AUDIT_QUICK=1 — skipping verify-production"
else
  if ./scripts/verify-production.sh; then pass "verify-production"; else fail "verify-production"; fi
fi

section "D — Live stack"
for svc in gzmo-daemon gzmo-prime hsp-synth hsp-pipeline; do
  if systemctl --user is-active "$svc.service" >/dev/null 2>&1; then
    pass "$svc active"
  else
    fail "$svc inactive"
  fi
done
if curl -sf --max-time 5 http://127.0.0.1:8000/v1/models >/dev/null; then
  pass "Prime :8000"
else
  fail "Prime :8000 unreachable"
fi

section "E — Timers & discovery"
for t in pi-mentor-discovery gzmo-skills-remediation; do
  st="$(systemctl --user is-active "${t}.timer" 2>/dev/null || echo inactive)"
  if [[ "$st" == "active" ]]; then pass "${t}.timer active"; else warn "${t}.timer $st"; fi
done
if [[ -f "$DISCOVERY_DATA/state.json" ]]; then
  jq -r '"discovery: session=\(.session_id) status=\(.session_status) cycle=\(.cycle) pillar=\(.discovery_pillar) duration=\(.session_duration_min)min"' \
    "$DISCOVERY_DATA/state.json"
  ds="$(jq -r '.session_status' "$DISCOVERY_DATA/state.json")"
  if [[ "$ds" == "active" ]]; then
    pass "discovery session active"
  elif [[ "$ds" == "completed_unpublished" ]]; then
    warn "discovery session final unpublished (actionability gate) — check session-final report"
  elif [[ "$ds" == "completed" ]]; then
    pass "discovery session completed (published)"
  else
    warn "discovery session $ds"
  fi
else
  warn "no discovery state.json"
fi
if systemctl --user is-active pi-mentor-discovery.service >/dev/null 2>&1; then
  warn "pi-mentor-discovery.service running (cycle in flight)"
fi

section "F — Obolus & Kurator"
if "$BIN" obolus balance; then pass "obolus balance"; else fail "obolus balance"; fi
if "$BIN" kurator status >/dev/null 2>&1; then
  pass "kurator status"
  "$BIN" kurator status 2>&1 | head -8 | sed 's/^/  /'
else
  fail "kurator status"
fi

section "G — Discovery pain signals"
CLOG="$DISCOVERY_DATA/logs/cycle.log"
CONFIG_ENV="$DISCOVERY_DATA/config.env"
if [[ -f "$CLOG" ]]; then
  to="$(grep -c 'timed out' "$CLOG" 2>/dev/null || echo 0)"
  er="$(grep -c '^\[.*\] ERROR' "$CLOG" 2>/dev/null || echo 0)"
  sk="$(grep -c 'SKIP:' "$CLOG" 2>/dev/null || echo 0)"
  pu="$(grep -c 'unpublished' "$CLOG" 2>/dev/null || echo 0)"
  echo "  7d baseline: timeouts=$to errors=$er skips=$sk unpublished=$pu"

  if [[ -f "$CONFIG_ENV" ]]; then
    cutoff="$(date -u -d "@$(stat -c %Y "$CONFIG_ENV")" +%Y-%m-%dT%H-%M-%SZ)"
    # Attribute by cycle report timestamp (cycle-N-ISOZ.md), not session start — config can reload mid-session.
    read -r post_sessions post_cycles post_timeouts post_errors post_evals post_passes post_last_to <<<"$(
      awk -v cutoff="$cutoff" '
        function extract_cycle_ts(line,    pos, chunk) {
          pos = index(line, "cycle-")
          if (pos == 0) return ""
          chunk = substr(line, pos)
          if (match(chunk, /cycle-[0-9]+-[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}-[0-9]{2}-[0-9]{2}Z/)) {
            chunk = substr(chunk, RSTART, RLENGTH)
            sub(/^cycle-[0-9]+-/, "", chunk)
            return chunk
          }
          return ""
        }
        function extract_session_id(line,    i, sid) {
          i = index(line, "session=")
          if (i == 0) return ""
          sid = substr(line, i + 8, 20)
          if (sid ~ /^[0-9]{4}-[0-9]{2}-[0-9]{2}T/) return sid
          return ""
        }
        function flush_block(    ts, i, n, line) {
          ts = cycle_ts
          if (ts == "") ts = extract_cycle_ts(block)
          if (ts == "" || ts < cutoff) {
            block = ""
            cycle_ts = ""
            return
          }
          cycles++
          if (pending_sid != "" && !(pending_sid in seen_sid)) {
            sessions++
            seen_sid[pending_sid] = 1
          }
          n = split(block, lines, "\n")
          for (i = 1; i <= n; i++) {
            line = lines[i]
            if (line ~ /timed out/) timeouts++
            if (line ~ /^\[.*\] ERROR/) errors++
            if (line ~ /EVAL status=pass/) passes++
            if (line ~ /EVAL status=/) evals++
            if (line ~ /Invoking pi dialogue.*timeout=/ && match(line, /timeout=[0-9]+/)) {
              last_to = substr(line, RSTART + 8, RLENGTH - 8)
            }
          }
          block = ""
          cycle_ts = ""
        }
        /START cycle=/ {
          if (block != "") flush_block()
          pending_sid = extract_session_id($0)
          block = $0 "\n"
          next
        }
        /OK report=.*cycle-[0-9]+-[0-9]{4}-/ {
          cycle_ts = extract_cycle_ts($0)
          if (block != "") block = block $0 "\n"
          flush_block()
          next
        }
        /ERROR:.*cycle-[0-9]+-[0-9]{4}-/ {
          cycle_ts = extract_cycle_ts($0)
          if (block != "") block = block $0 "\n"
          flush_block()
          next
        }
        /SESSION (FINAL|COMPLETE)/ {
          if (block != "") flush_block()
          next
        }
        {
          if (block != "") block = block $0 "\n"
        }
        END {
          if (block != "") flush_block()
          printf "%d %d %d %d %d %d %s\n",
            sessions + 0, cycles + 0, timeouts + 0, errors + 0, evals + 0, passes + 0,
            (last_to != "" ? last_to : "n/a")
        }
      ' "$CLOG"
    )"
    if [[ "$post_last_to" =~ ^[0-9]+$ ]]; then
      post_to_label="${post_last_to}s"
    else
      post_to_label="$post_last_to"
    fi
    echo "  post-config (since $cutoff): sessions=$post_sessions cycles=$post_cycles evals=$post_evals pass=$post_passes timeouts=$post_timeouts errors=$post_errors last_dialogue_timeout=$post_to_label"
    if (( post_cycles == 0 )); then
      echo "  note: no post-config cycles yet — start a new discovery session to validate tuning"
    elif (( post_evals >= 2 )); then
      if (( post_timeouts * 100 / post_evals > 30 )); then
        warn "post-config timeout rate high ($post_timeouts/$post_evals evals) — try DISCOVERY_MENTOR_EXCHANGES_MAX=1"
      fi
      if (( post_errors > 2 )); then warn "post-config discovery errors ($post_errors) — review cycle.log"; fi
    elif (( post_timeouts > 0 )); then
      warn "post-config timeouts ($post_timeouts) — sample small ($post_evals evals), re-check after 1–2 sessions"
    fi
  else
    warn "no config.env — cannot compute post-config discovery metrics"
    if [[ "$to" -gt 10 ]]; then warn "high pi dialogue timeouts ($to) — consider PI_DIALOGUE_TIMEOUT_SEC or compact health"; fi
    if [[ "$er" -gt 5 ]]; then warn "discovery errors ($er) — review cycle.log"; fi
  fi
else
  warn "no cycle.log"
fi

section "H — Daemon REM/verify (2h)"
rem="$(journalctl --user -u gzmo-daemon.service --since "2 hours ago" --no-pager 2>/dev/null \
  | grep -c 'REM/verify pipeline failed' || true)"
dream_partial="$(journalctl --user -u gzmo-daemon.service --since "2 hours ago" --no-pager 2>/dev/null \
  | grep -c 'Dream consolidation partial' || true)"
echo "  REM/verify failures (2h): $rem"
echo "  dream partial (2h): $dream_partial"
if [[ "$rem" -gt 20 ]]; then warn "frequent REM/verify JSON parse failures — dream yield low"; fi

section "I — Tuning scorecard"
echo "  Config: $DISCOVERY_DATA/config.env"
if [[ -f "$DISCOVERY_DATA/config.env" ]]; then
  grep -E '^(DISCOVERY_SESSION_SIZE|PI_DIALOGUE_TIMEOUT_SEC|DISCOVERY_MENTOR_EXCHANGES_MAX|DISCOVERY_HEALTH_COMPACT|max_ctx_pressure_pct)' \
    "$DISCOVERY_DATA/config.env" 2>/dev/null | sed 's/^/    /' || true
  grep -E '^(DISCOVERY_SESSION_SIZE|PI_DIALOGUE_TIMEOUT_SEC|DISCOVERY_MENTOR_EXCHANGES_MAX)' \
    "$ROOT/gzmo.toml" 2>/dev/null | head -3 | sed 's/^/    gzmo.toml: /' || true
fi
"$BIN" obolus balance 2>&1 | grep -E 'E_total|ctx_%' | sed 's/^/    /'

section "J — Discovery remediation closure"
TRACKER="${DISCOVERY_DATA}/remediation-tracker.json"
if [[ -x "${SKILLS}/scripts/run-discovery-implementation-pipeline.sh" ]]; then
  pass "run-discovery-implementation-pipeline.sh present"
else
  warn "run-discovery-implementation-pipeline.sh missing — session-final actions may not auto-implement"
fi
if [[ -x "${SKILLS}/scripts/verify-discovery-action-pipeline.sh" ]]; then
  pass "verify-discovery-action-pipeline.sh present"
else
  warn "verify-discovery-action-pipeline.sh missing"
fi
if [[ -x "$BIN" ]]; then
  if "$BIN" kurator remediation-status 2>/dev/null | sed 's/^/  /'; then
    open_count="$("$BIN" kurator remediation-status --json 2>/dev/null | python3 -c 'import json,sys; d=json.load(sys.stdin); print(sum(1 for f in d.get("findings",[]) if f.get("status")!="fixed"))' 2>/dev/null || echo 0)"
    if [[ "${open_count:-0}" -gt 0 ]]; then
      warn "open FAIL/GAP without verified fix ($open_count) — check discovery_fix.failed events"
    else
      pass "no open remediation findings (or tracker empty)"
    fi
  else
    warn "gzmo kurator remediation-status failed"
  fi
elif [[ -f "$TRACKER" ]]; then
  warn "gzmo binary missing — remediation tracker exists at $TRACKER"
else
  echo "  (no remediation tracker yet)"
fi

section "Summary"
echo "  FAIL=$FAIL  WARN=$WARN"
if (( FAIL > 0 )); then
  echo "SYSTEM AUDIT: FAIL — fix [FAIL] items before relying on autospawn/discovery"
  exit 1
fi
if (( WARN > 0 )); then
  echo "SYSTEM AUDIT: PASS with $WARN warning(s) — review tuning scorecard"
  exit 0
fi
echo "SYSTEM AUDIT: PASS"
exit 0
