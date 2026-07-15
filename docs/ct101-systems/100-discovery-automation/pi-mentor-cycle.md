# Pi Mentor Discovery Cycle

**Source:** `gzmo_skills/scripts/pi-mentor-discovery-cycle.sh`  
**Parent:** [100-discovery-automation/SYSTEM.md](./SYSTEM.md)

---

## Capability

Runs a timed **Pi ↔ GZMO mentor** infrastructure discovery arc: pillar-assigned probes, batched distill every N cycles, session-final report generation, deterministic eval gates, and conditional publish. Feeds the implementation queue when findings pass.

---

## How it works

### Session bootstrap

```6:18:github-clone/gzmo_skills/scripts/pi-mentor-discovery-cycle.sh
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
GZMO_ROOT="${GZMO_ROOT:-$HOME/Projects/_foundation-audit/survey_GZMO}"
DATA_DIR="${PI_MENTOR_DISCOVERY_DATA:-$ROOT/data/pi-mentor-discovery}"
STATE_FILE="$DATA_DIR/state.json"
LOCK_FILE="$DATA_DIR/.cycle.lock"
GZMO_BIN="${GZMO_BIN:-$GZMO_ROOT/target/release/gzmo}"
PI_DISCOVERY_CWD="${PI_DISCOVERY_CWD:-$ROOT}"
```

Defaults: 60 min sessions, 6 cycles per distill, 4 mentor exchanges max, strict Socratic mode.

### Implementation pipeline hook

```84:105:github-clone/gzmo_skills/scripts/pi-mentor-discovery-cycle.sh
run_discovery_implementation_pipeline() {
  local report_path="$1"
  local session_id="$2"
  // ...
  GZMO_ROOT="$GZMO_ROOT" GZMO_BIN="$GZMO_BIN" \
    "$SCRIPTS_DIR/run-discovery-implementation-pipeline.sh" \
    --report "$report_path" --session-id "$session_id" --label "$label" \
    >>"$LOG_DIR/remediation.log" 2>&1
}
```

### Session-final eval + publish gate

The cycle ends with `eval-pi-mentor-discovery-report.sh` (deterministic, no LLM):

- Structural sections: Executive summary, Findings, Snapshot header
- ISREL: probe activity in logs (`gzmo_health`, `systemctl`, etc.)
- Mentor teach: `gzmo_mentor_teach` in logs or arc JSONL
- Pillar evidence patterns from `pillars.json`
- **Placeholder detection** via `discovery-findings-lib.sh`:

```5:21:github-clone/gzmo_skills/scripts/lib/discovery-findings-lib.sh
discovery_findings_placeholder_pattern='(<title>|\(3–5 sentences\)|\(2–3 sentences|\(repeat as needed\)|TODO|TBD|\(what the next)'
discovery_findings_is_placeholder() {
  // ...
  [[ "$text" =~ $discovery_findings_placeholder_pattern ]] && return 0
```

```215:216:github-clone/gzmo_skills/scripts/lib/discovery-findings-lib.sh
  if grep -qiE "$discovery_findings_placeholder_pattern" "$report"; then
    add_reason "report contains template placeholder text"
```

**Live failure mode (2026-07-14):** Pi emitted report text matching template instructions → eval fail → `published=false` → auto-socratic logs `SKIP: cycle ran successfully but did not publish`.

### OBOLUS on fixer autospawn

```115:118:github-clone/gzmo_skills/scripts/pi-mentor-discovery-cycle.sh
  if ! (cd "$GZMO_ROOT" && "$GZMO_BIN" obolus preflight spawn_discovery_fix); then
    local obolus_rc=$?
    log "OBOLUS: fixer autospawn blocked (exit $obolus_rc)"
```

### Residual Pi cleanup

```144:152:github-clone/gzmo_skills/scripts/pi-mentor-discovery-cycle.sh
kill_residual_pi() {
  if [[ -n "${arc_session_id:-}" ]]; then
    pkill -9 -f "$arc_session_id" 2>/dev/null || true
  fi
  pkill -9 -f 'pi -p.*(infra-discovery|discovery-arc|discovery-final)' 2>/dev/null || true
}
```

---

## Interfaces

| Interface | Value |
|-----------|-------|
| State | `$DATA_DIR/state.json` (`published`, `last_published_report`, `session_plan`) |
| Reports | `$DATA_DIR/reports/session-final-*.md` |
| Eval script | `eval-pi-mentor-discovery-report.sh` (exit 0=pass, 1=warn, 2=fail) |
| Metrics | `$LOG_DIR/cycle-metrics.jsonl` |
| Pillars | `$DATA_DIR/pillars.json` via `pillar-registry.sh` |
| Pi timeouts | `PI_DIALOGUE_TIMEOUT_SEC=540`, `PI_REPORT_TIMEOUT_SEC=240` |
| Timer | `pi-mentor-discovery.timer` (user systemd on CT101) |

---

## THINKING nodes

> **THINKING — pi-mentor:watchdog alignment**
> - *Reviewed:* `align_pi_watchdog_timeouts` ensures fallback timeout ≥ primary dialogue budget.
> - *Insight:* Prevents premature kill during long OpenRouter mentor exchanges.
> - *Risk / limitation:* Network stalls still appear as hung Pi until watchdog fires.
> - *Enhancement:* Active ping to Pi MCP health between exchanges. [CT101-safe]

> **THINKING — pi-mentor:placeholder eval**
> - *Reviewed:* Regex gate catches template instruction text leaked into final report.
> - *Insight:* Strong guard against "successful" cycles that publish no actionable findings.
> - *Risk / limitation:* False positives if legitimate report mentions "TODO" in evidence quotes.
> - *Enhancement:* Scope placeholder check to Executive summary / Findings sections only. [CT101-safe]

> **THINKING — pi-mentor:session-final rewrite**
> - *Reviewed:* On eval fail, cycle can rewrite report with feedback (`final_eval_failed` branch).
> - *Insight:* One retry path before marking unpublished.
> - *Risk / limitation:* Rewrite still depends on Pi following prompt — may re-leak placeholders.
> - *Enhancement:* Strip template blocks from prompt before final report phase. [CT101-safe]

> **THINKING — pi-mentor:implementation pipeline**
> - *Reviewed:* Prefers `run-discovery-implementation-pipeline.sh` over raw fixer spawn.
> - *Insight:* Probes-first ordering respects CT101 sidecar-only policy.
> - *Risk / limitation:* Pipeline failure only warns in log — publish bit independent.
> - *Enhancement:* Block publish if implementation registration fails. [GZMO-next]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| Pi OpenRouter dialogue on CT101 | Lab discovery recipe with beat-gate against CT101 report schema |
| Bash eval gates | Rust `kurator` report validator in gzmo-core |
| File state.json | Event-sourced session state in Redis |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Prompt hardening — no template literals in session-final output | [CT101-safe] |
| 2 | Section-scoped placeholder eval | [CT101-safe] |
| 3 | Publish metrics dashboard (pass/unpublished ratio) | [CT101-safe] |
| 4 | Migrate eval gates to Rust for speed + testability | [GZMO-next] |
| 5 | Pillar rotation fairness audit in `auto-triggers.jsonl` | [CT101-safe] |
