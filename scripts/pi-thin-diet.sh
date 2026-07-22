#!/usr/bin/env bash
# Pi thin diet — allowlist check / apply-core / recommended / purge-denied.
# Never touches CT101. Never starts gzmo-serve. Living memory stays MCP-only.
#
#   bash scripts/pi-thin-diet.sh --check
#   bash scripts/pi-thin-diet.sh --apply-core --dry-run
#   bash scripts/pi-thin-diet.sh --apply-recommended --with spark,ask,web --dry-run
#   bash scripts/pi-thin-diet.sh --purge-denied --dry-run
#
# Docs: docs/PI_PACKAGE_ALLOWLIST.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PI_HOME="${PI_AGENT_HOME:-$HOME/.pi/agent}"
SETTINGS="${PI_SETTINGS:-$PI_HOME/settings.json}"
DRY_RUN=0
MODE=""
WITH_CSV=""
PREFER_GZMO_PI="${GZMO_PI_SOURCE:-git}" # git | npm
WEB_AGENT="pi-web-access"               # pi-web-access | demigod
EXIT_ISSUES=0

usage() {
  cat <<'EOF'
Usage: bash scripts/pi-thin-diet.sh <mode> [options]

Modes (pick one):
  --check                 Report installed vs allowlist / deny / duplicates
  --apply-core            Ensure adapter + single gzmo-pi + optional hsp; dedupe gzmo-pi
  --apply-recommended     Install recommended QoL (requires --with …)
  --purge-denied          Remove known deny-list packages if present

Options:
  --dry-run               Print actions only (no pi install/remove)
  --with <csv>            QoL flags: spark,plan,ask,permissions,web,skillful[,lens,fff,plannotator,compact]
  --prefer-gzmo-pi <src>  git (default) | npm — which gzmo-pi to keep when both present
  --web-agent <which>     pi-web-access (default) | demigod
  -h, --help              This help

Env:
  PI_AGENT_HOME   default ~/.pi/agent
  PI_SETTINGS     default $PI_AGENT_HOME/settings.json
  GZMO_PI_SOURCE  git|npm (same as --prefer-gzmo-pi)

Never touches CT101. Never starts gzmo-serve.
EOF
}

log() { printf '%s\n' "$*"; }
warn() { printf 'WARN: %s\n' "$*" >&2; }
issue() {
  printf 'ISSUE: %s\n' "$*" >&2
  EXIT_ISSUES=1
}
ok() { printf 'OK: %s\n' "$*"; }
act() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf 'DRY-RUN: %s\n' "$*"
  else
    printf 'RUN: %s\n' "$*"
  fi
}

# Deny list — safe exact specs only (never globs that could remove gzmo-pi / adapter).
DENY_SPECS=(
  "npm:pi-memory"
  "npm:@samfp/pi-memory"
  "npm:pi-hermes-memory"
  "npm:@mariozechner/pi-memory"
  "npm:pi-crew"
  "npm:pi-workflow-engine"
  "npm:pi-orchestrator"
  "npm:pi-swarm"
)

# Known alternate subagent stacks (keep only pi-subagents).
DENY_SUBAGENT_SPECS=(
  "npm:pi-crew"
  "npm:@mariozechner/pi-subagents"
)

CORE_ADAPTER="npm:pi-mcp-adapter"
CORE_SUBAGENTS="npm:pi-subagents"
CORE_HSP="npm:hsp-pi"
GZMO_NPM="npm:gzmo-pi"
GZMO_GIT="git:github.com/maximilianwruhs-cyber/gzmo-pi"

declare -A QOL_MAP=(
  [spark]="npm:pi-spark"
  [plan]="npm:pi-plan-mode"
  [ask]="npm:@eko24ive/pi-ask"
  [permissions]="npm:@gotgenes/pi-permission-system"
  [skillful]="npm:pi-skillful"
  [lens]="npm:pi-lens"
  [fff]="npm:@ff-labs/pi-fff"
  [plannotator]="npm:@plannotator/pi-extension"
  [compact]="npm:pi-mega-compact"
)

while [[ $# -gt 0 ]]; do
  case "$1" in
    --check|--apply-core|--apply-recommended|--purge-denied)
      if [[ -n "$MODE" ]]; then
        echo "REFUSE: multiple modes ($MODE and $1)" >&2
        exit 2
      fi
      MODE="${1#--}"
      shift
      ;;
    --dry-run) DRY_RUN=1; shift ;;
    --with)
      WITH_CSV="${2:-}"
      shift 2
      ;;
    --prefer-gzmo-pi)
      PREFER_GZMO_PI="${2:-}"
      shift 2
      ;;
    --web-agent)
      WEB_AGENT="${2:-}"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "REFUSE: unknown arg: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [[ -z "$MODE" ]]; then
  usage >&2
  exit 2
fi

case "$PREFER_GZMO_PI" in
  git|npm) ;;
  *)
    echo "REFUSE: --prefer-gzmo-pi must be git|npm (got: $PREFER_GZMO_PI)" >&2
    exit 2
    ;;
esac

case "$WEB_AGENT" in
  pi-web-access|demigod) ;;
  *)
    echo "REFUSE: --web-agent must be pi-web-access|demigod (got: $WEB_AGENT)" >&2
    exit 2
    ;;
esac

# Hard guardrails — never dual-writer / serve activation from this script.
if [[ "${GZMO_PRODUCT:-}" == "1" && "$MODE" != "check" ]]; then
  warn "GZMO_PRODUCT=1 set — diet still only edits Pi packages; living attach is separate"
fi

if command -v systemctl >/dev/null 2>&1; then
  SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
  SERVE="$(printf '%s\n' "${SERVE:-inactive}" | head -1)"
  if [[ "$SERVE" == "active" ]]; then
    warn "gzmo-serve.service is active (ADR-0003 dual-writer risk) — diet will NOT start/stop it; fix separately"
  fi
fi

read_packages() {
  if [[ ! -f "$SETTINGS" ]]; then
    echo "REFUSE: missing settings: $SETTINGS" >&2
    exit 1
  fi
  python3 - "$SETTINGS" <<'PY'
import json, sys
path = sys.argv[1]
with open(path) as f:
    data = json.load(f)
pkgs = data.get("packages") or []
if not isinstance(pkgs, list):
    raise SystemExit("settings.packages is not a list")
for p in pkgs:
    print(p)
PY
}

pkg_installed() {
  local want="$1"
  local p
  for p in "${INSTALLED[@]+"${INSTALLED[@]}"}"; do
    [[ "$p" == "$want" ]] && return 0
  done
  return 1
}

pi_install() {
  local spec="$1"
  if pkg_installed "$spec"; then
    ok "already installed: $spec"
    return 0
  fi
  act "pi install $spec"
  if [[ "$DRY_RUN" -eq 0 ]]; then
    pi install "$spec"
  fi
}

pi_remove() {
  local spec="$1"
  if ! pkg_installed "$spec"; then
    ok "already absent: $spec"
    return 0
  fi
  act "pi remove $spec"
  if [[ "$DRY_RUN" -eq 0 ]]; then
    pi remove "$spec"
  fi
}

reload_installed() {
  mapfile -t INSTALLED < <(read_packages)
}

web_spec() {
  if [[ "$WEB_AGENT" == "demigod" ]]; then
    printf '%s\n' "npm:@demigodmode/pi-web-agent"
  else
    printf '%s\n' "npm:pi-web-access"
  fi
}

other_web_spec() {
  if [[ "$WEB_AGENT" == "demigod" ]]; then
    printf '%s\n' "npm:pi-web-access"
  else
    printf '%s\n' "npm:@demigodmode/pi-web-agent"
  fi
}

mode_check() {
  log "=== pi-thin-diet --check (settings=$SETTINGS prefer=$PREFER_GZMO_PI) ==="
  log "Installed packages (${#INSTALLED[@]}):"
  local p
  for p in "${INSTALLED[@]+"${INSTALLED[@]}"}"; do
    log "  - $p"
  done
  log ""

  if pkg_installed "$CORE_ADAPTER"; then
    ok "core: $CORE_ADAPTER"
  else
    issue "missing core: $CORE_ADAPTER"
  fi

  if pkg_installed "$CORE_SUBAGENTS"; then
    ok "core: $CORE_SUBAGENTS (single subagent stack)"
  else
    issue "missing core: $CORE_SUBAGENTS"
  fi

  local has_npm=0 has_git=0
  pkg_installed "$GZMO_NPM" && has_npm=1
  pkg_installed "$GZMO_GIT" && has_git=1

  if [[ "$has_npm" -eq 1 && "$has_git" -eq 1 ]]; then
    issue "duplicate gzmo-pi: both $GZMO_NPM and $GZMO_GIT (prefer $PREFER_GZMO_PI)"
  elif [[ "$has_npm" -eq 1 || "$has_git" -eq 1 ]]; then
    ok "single gzmo-pi present"
  else
    issue "missing gzmo-pi (want one of $GZMO_NPM or $GZMO_GIT)"
  fi

  if pkg_installed "$CORE_HSP"; then
    ok "optional: $CORE_HSP"
  else
    log "NOTE: optional hsp-pi not installed"
  fi

  # Web duplicate
  local web_a="npm:pi-web-access" web_b="npm:@demigodmode/pi-web-agent"
  if pkg_installed "$web_a" && pkg_installed "$web_b"; then
    issue "duplicate web packs: $web_a and $web_b (pick one)"
  fi

  # Deny hits
  local d
  for d in "${DENY_SPECS[@]}" "${DENY_SUBAGENT_SPECS[@]}"; do
    if pkg_installed "$d"; then
      issue "deny-list present: $d"
    fi
  done

  # Unknown (informational)
  local known=(
    "$CORE_ADAPTER" "$CORE_SUBAGENTS" "$CORE_HSP" "$GZMO_NPM" "$GZMO_GIT"
    "npm:pi-spark" "npm:pi-plan-mode" "npm:@eko24ive/pi-ask"
    "npm:@gotgenes/pi-permission-system" "npm:pi-web-access"
    "npm:@demigodmode/pi-web-agent" "npm:pi-skillful" "npm:pi-lens"
    "npm:@ff-labs/pi-fff" "npm:@plannotator/pi-extension" "npm:pi-mega-compact"
  )
  for p in "${INSTALLED[@]+"${INSTALLED[@]}"}"; do
    local k known_hit=0
    for k in "${known[@]}"; do
      if [[ "$p" == "$k" ]]; then known_hit=1; break; fi
    done
    if [[ "$known_hit" -eq 0 ]]; then
      warn "not on allowlist (review): $p"
    fi
  done

  if [[ "$EXIT_ISSUES" -eq 0 ]]; then
    ok "check clean (no allowlist/deny/duplicate issues)"
  else
    warn "check found issues — see ISSUE lines; try --apply-core / --purge-denied"
  fi
  return "$EXIT_ISSUES"
}

mode_apply_core() {
  log "=== pi-thin-diet --apply-core (dry_run=$DRY_RUN prefer=$PREFER_GZMO_PI) ==="
  log "Guard: will not touch CT101; will not start gzmo-serve"

  pi_install "$CORE_ADAPTER"
  reload_installed
  pi_install "$CORE_SUBAGENTS"
  reload_installed

  local has_npm=0 has_git=0
  pkg_installed "$GZMO_NPM" && has_npm=1
  pkg_installed "$GZMO_GIT" && has_git=1

  if [[ "$has_npm" -eq 0 && "$has_git" -eq 0 ]]; then
    if [[ "$PREFER_GZMO_PI" == "git" ]]; then
      pi_install "$GZMO_GIT"
    else
      pi_install "$GZMO_NPM"
    fi
    reload_installed
  elif [[ "$has_npm" -eq 1 && "$has_git" -eq 1 ]]; then
    if [[ "$PREFER_GZMO_PI" == "git" ]]; then
      pi_remove "$GZMO_NPM"
    else
      pi_remove "$GZMO_GIT"
    fi
    reload_installed
  else
    ok "single gzmo-pi already selected"
  fi

  # hsp-pi: ensure only if already intended — install when missing? Spec says optional.
  # apply-core "optional hsp" means: if neither present, leave; if present keep; do not force-install.
  if pkg_installed "$CORE_HSP"; then
    ok "optional hsp-pi kept"
  else
    log "NOTE: hsp-pi absent (optional — install with: pi install $CORE_HSP)"
  fi

  if [[ "$DRY_RUN" -eq 0 ]]; then
    reload_installed
  fi
  ok "apply-core done"
}

mode_apply_recommended() {
  if [[ -z "$WITH_CSV" ]]; then
    echo "REFUSE: --apply-recommended requires --with <csv> (e.g. spark,ask,web)" >&2
    exit 2
  fi
  log "=== pi-thin-diet --apply-recommended (dry_run=$DRY_RUN with=$WITH_CSV web=$WEB_AGENT) ==="
  log "Guard: will not touch CT101; will not start gzmo-serve; will not install memory packs"

  IFS=',' read -r -a flags <<< "$WITH_CSV"
  local f
  for f in "${flags[@]}"; do
    f="$(echo "$f" | tr '[:upper:]' '[:lower:]' | tr -d '[:space:]')"
    [[ -z "$f" ]] && continue
    case "$f" in
      web)
        local want other
        want="$(web_spec)"
        other="$(other_web_spec)"
        if pkg_installed "$other"; then
          warn "other web pack present ($other) — removing so only one remains"
          pi_remove "$other"
          reload_installed
        fi
        pi_install "$want"
        reload_installed
        ;;
      spark|plan|ask|permissions|skillful|lens|fff|plannotator|compact)
        pi_install "${QOL_MAP[$f]}"
        reload_installed
        ;;
      *)
        echo "REFUSE: unknown --with flag: $f" >&2
        echo "Allowed: spark,plan,ask,permissions,web,skillful,lens,fff,plannotator,compact" >&2
        exit 2
        ;;
    esac
  done
  ok "apply-recommended done"
}

mode_purge_denied() {
  log "=== pi-thin-diet --purge-denied (dry_run=$DRY_RUN) ==="
  log "Guard: exact deny specs only; never starts gzmo-serve"

  local d any=0
  # unique deny list
  local -A seen=()
  for d in "${DENY_SPECS[@]}" "${DENY_SUBAGENT_SPECS[@]}"; do
    [[ -n "${seen[$d]:-}" ]] && continue
    seen[$d]=1
    if pkg_installed "$d"; then
      any=1
      pi_remove "$d"
      reload_installed
    fi
  done
  if [[ "$any" -eq 0 ]]; then
    ok "no deny-list packages present"
  else
    ok "purge-denied done"
  fi
}

# --- main ---
if ! command -v pi >/dev/null 2>&1; then
  echo "REFUSE: pi not on PATH" >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "REFUSE: python3 required to read settings.json" >&2
  exit 1
fi

reload_installed

case "$MODE" in
  check) mode_check; exit $? ;;
  apply-core) mode_apply_core ;;
  apply-recommended) mode_apply_recommended ;;
  purge-denied) mode_purge_denied ;;
  *)
    echo "REFUSE: unknown mode $MODE" >&2
    exit 2
    ;;
esac
