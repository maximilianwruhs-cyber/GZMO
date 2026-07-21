#!/usr/bin/env bash
# Claim / release the overnight living-host mutex (ADR-0005).
# Never leave two overnight writers racing the same vault.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
STATE_DIR="$DATA/living-host"
STATE_FILE="$STATE_DIR/claim.json"

usage() {
  cat <<'EOF'
Usage:
  living-host-mutex.sh status
  living-host-mutex.sh claim --host ct101|workstation|appliance [--note TEXT]
  living-host-mutex.sh release

ADR-0005: one overnight writer per vault. Claiming workstation living requires
stopping CT101 daemon writers (and vice versa) before overnight jobs run.

This script records the claim and checks local dual-writer risk (gzmo-serve /
gzmo-scheduler user units). It does NOT SSH-stop CT101 — print the ops checklist.
EOF
  exit 1
}

serve_active() {
  local s
  s="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
  echo "${s:-inactive}"
}

sched_active() {
  local s
  s="$(systemctl --user is-active gzmo-scheduler.service 2>/dev/null || true)"
  echo "${s:-inactive}"
}

status_json() {
  mkdir -p "$STATE_DIR"
  local claim="{}"
  if [[ -f "$STATE_FILE" ]]; then
    claim="$(cat "$STATE_FILE")"
  fi
  python3 - "$claim" "$(serve_active)" "$(sched_active)" <<'PY'
import json, sys
claim = json.loads(sys.argv[1] or "{}")
serve, sched = sys.argv[2], sys.argv[3]
host = claim.get("host") or "none"
dual = serve == "active" or sched == "active"
# Dual-writer risk: workstation units active while claim says ct101 (or unknown)
risk = bool(dual and host in ("ct101", "none", ""))
out = {
    "claim": claim,
    "workstation_serve": serve,
    "workstation_scheduler": sched,
    "dual_writer_risk": risk,
    "ok": not risk or host == "workstation",
}
print(json.dumps(out, indent=2))
PY
}

cmd_status() {
  status_json
}

cmd_claim() {
  local host="" note=""
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --host) host="${2:-}"; shift 2 ;;
      --note) note="${2:-}"; shift 2 ;;
      *) usage ;;
    esac
  done
  case "$host" in
    ct101|workstation|appliance) ;;
    *) echo "error: --host must be ct101|workstation|appliance" >&2; exit 2 ;;
  esac
  mkdir -p "$STATE_DIR"
  python3 - "$STATE_FILE" "$host" "$note" <<'PY'
import json, os, sys
from datetime import datetime, timezone
path, host, note = sys.argv[1], sys.argv[2], sys.argv[3]
doc = {
    "host": host,
    "claimed_at": datetime.now(timezone.utc).isoformat(),
    "note": note or "",
    "adr": "ADR-0005",
}
with open(path, "w", encoding="utf-8") as f:
    json.dump(doc, f, indent=2)
    f.write("\n")
print(json.dumps(doc, indent=2))
PY
  echo
  echo "=== mutex checklist ==="
  case "$host" in
    workstation)
      cat <<'EOF'
Claimed: workstation living
1. On CT101: stop gzmo-daemon / scheduler overnight writers (ops).
2. Confirm no other host writes this vault.
3. Enable local gzmo-serve / scheduler only for this claim window.
4. When done: living-host-mutex.sh release && restore CT101 writers if desired.
EOF
      ;;
    ct101)
      cat <<'EOF'
Claimed: CT101 living (reference)
1. On workstation: systemctl --user stop gzmo-serve gzmo-scheduler (if active).
2. Confirm dual_writer_risk=false via living-host-mutex.sh status.
3. CT101 remains sole overnight writer.
EOF
      local s
      s="$(serve_active)"
      if [[ "$s" == "active" ]]; then
        echo "[!] workstation gzmo-serve is ACTIVE — stop it before overnight" >&2
        exit 3
      fi
      ;;
    appliance)
      cat <<'EOF'
Claimed: appliance living
1. Stop overnight writers on CT101 and workstation.
2. Point vault/sidecars at the appliance data dir.
3. release when moving claim elsewhere.
EOF
      ;;
  esac
  echo
  cmd_status
}

cmd_release() {
  mkdir -p "$STATE_DIR"
  if [[ -f "$STATE_FILE" ]]; then
    mv "$STATE_FILE" "$STATE_DIR/claim.prev.json"
    echo "Released claim (previous → $STATE_DIR/claim.prev.json)"
  else
    echo "No active claim file"
  fi
  cmd_status
}

main() {
  local op="${1:-}"
  shift || true
  case "$op" in
    status) cmd_status ;;
    claim) cmd_claim "$@" ;;
    release) cmd_release ;;
    -h|--help|"") usage ;;
    *) usage ;;
  esac
}

main "$@"
