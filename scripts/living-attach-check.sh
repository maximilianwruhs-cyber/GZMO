#!/usr/bin/env bash
# Agent-facing fail-closed living attach probe (read-only).
# Proves living vault + fact floor + dual-writer false. Never starts gzmo-serve.
#
#   bash scripts/living-attach-check.sh
#   bash scripts/living-attach-check.sh --local-only   # env + dual-writer only (no SSH)
#   GZMO_ATTACH_MODE=local GZMO_CONFIG=/opt/gzmo/gzmo.toml bash scripts/living-attach-check.sh
#
# Docs: docs/EXTERNAL_LIVING_ATTACH.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
MIN_FACTS="${LIVING_ATTACH_MIN_FACTS:-10000}"
REF_FACTS_HINT="${LIVING_ATTACH_REF_FACTS:-60000}"
MODE="${GZMO_ATTACH_MODE:-ssh}" # ssh | local
LOCAL_ONLY=0

usage() {
  cat <<'EOF'
Usage: bash scripts/living-attach-check.sh [--local-only] [--mode ssh|local]

Fail-closed living attach probe for external agents.
Never enables a second overnight writer. Never starts gzmo-serve.

Exit 0 = living attach proof OK
Exit 1 = refuse / misconfig / lab / dual-writer / unreachable
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --local-only) LOCAL_ONLY=1; shift ;;
    --mode)
      MODE="${2:-}"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "REFUSE: unknown arg: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

case "$MODE" in
  ssh|local) ;;
  *)
    echo "REFUSE: --mode must be ssh|local (got: $MODE)" >&2
    exit 1
    ;;
esac

fail() {
  printf 'FAIL: %s\n' "$*" >&2
  exit 1
}

ok() { printf 'OK: %s\n' "$*"; }

echo "=== living-attach-check (mode=$MODE local_only=$LOCAL_ONLY) ==="

# --- Forbidden env while claiming living ---
if [[ "${GZMO_PRODUCT:-}" == "1" ]]; then
  fail "GZMO_PRODUCT=1 conflicts with living claim — use gzmo-memory (lite) or unset PRODUCT for living"
fi
if [[ "${GZMO_ALLOW_LAB_VAULT:-}" == "1" ]]; then
  fail "GZMO_ALLOW_LAB_VAULT=1 is forbidden on living attach (silences vault floor; lab false-positive)"
fi
ok "env: no GZMO_PRODUCT / GZMO_ALLOW_LAB_VAULT living conflict"

# --- Dual-writer: workstation must not run overnight serve ---
SERVE="unknown"
if command -v systemctl >/dev/null 2>&1; then
  SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
  SERVE="$(printf '%s\n' "${SERVE:-inactive}" | head -1)"
fi
if [[ "$SERVE" == "active" ]]; then
  fail "workstation gzmo-serve is active — dual-writer risk (ADR-0003). Stop it; do not enable a second writer from attach"
fi
ok "dual_writer=false (workstation gzmo-serve=${SERVE:-inactive})"

if [[ "$LOCAL_ONLY" == "1" ]]; then
  ok "local-only: skipped remote/on-box vault probe"
  echo "=== PASS (partial): env + dual-writer OK; run without --local-only for vault proof ==="
  exit 0
fi

STATUS_JSON=""
if [[ "$MODE" == "local" ]]; then
  CFG="${GZMO_CONFIG:-}"
  [[ -n "$CFG" ]] || fail "local mode requires GZMO_CONFIG pointing at living toml"
  [[ -f "$CFG" ]] || fail "GZMO_CONFIG not a file: $CFG"
  if [[ "$CFG" == *"/.gzmo/"* ]] || [[ "$CFG" == *"/.gzmo/gzmo.toml" ]]; then
    fail "GZMO_CONFIG looks like lite/lab home (~/.gzmo) — not living"
  fi
  GZMO_LOCAL_BIN="${GZMO_BIN:-}"
  if [[ -z "$GZMO_LOCAL_BIN" ]]; then
    if [[ -x "$ROOT/target/release/gzmo" ]]; then
      GZMO_LOCAL_BIN="$ROOT/target/release/gzmo"
    elif command -v gzmo >/dev/null 2>&1; then
      GZMO_LOCAL_BIN="$(command -v gzmo)"
    else
      fail "no gzmo binary (set GZMO_BIN=...)"
    fi
  fi
  STATUS_JSON="$(GZMO_CONFIG="$CFG" "$GZMO_LOCAL_BIN" memory status --json 2>/dev/null)" \
    || fail "local memory status failed (config=$CFG)"
else
  # Ops SSH path — same env contract as pi-gzmo-mcp-serve.sh
  STATUS_JSON="$(ssh -o ConnectTimeout=15 -o BatchMode=yes "$HOST" \
    "bash -lc $(printf '%q' "cd /opt/gzmo && export GZMO_CONFIG=/opt/gzmo/gzmo.toml && exec $(printf '%q' "$BIN") memory status --json")" \
    2>/dev/null)" \
    || fail "SSH living probe failed (host=$HOST). Fix BatchMode SSH / CT101_SSH_HOST; do not hand-roll lab mcp-serve"
fi

export STATUS_JSON MIN_FACTS REF_FACTS_HINT MODE
python3 - <<'PY'
import json, os, sys

raw = os.environ.get("STATUS_JSON") or ""
min_facts = int(os.environ["MIN_FACTS"])
ref_hint = os.environ["REF_FACTS_HINT"]
mode = os.environ["MODE"]

try:
    data = json.loads(raw)
except Exception as e:
    print(f"FAIL: memory status not JSON: {e}", file=sys.stderr)
    sys.exit(1)

vault = str(data.get("vault_path") or "")
facts = data.get("vault_facts")
honeypot = data.get("honeypot_latest")

def fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)

if not vault:
    fail("vault_path missing from memory status")

norm = vault.replace("\\", "/")
if "/.gzmo/" in norm or norm.endswith("/.gzmo/data/vault.db"):
    fail(f"vault_path is lab/lite home, not living: {vault}")

if mode == "ssh":
    if "/opt/gzmo/" not in norm:
        fail(f"ops SSH living expects vault under /opt/gzmo/ (got {vault})")
elif "/opt/gzmo/" not in norm and "/.gzmo-living/" not in norm and "gzmo-living" not in norm:
    # local airgap may use ~/.gzmo-living or /opt/gzmo
    if "data-next" in norm:
        fail(f"vault_path looks like lab data-next: {vault}")

if not isinstance(facts, int):
    fail(f"vault_facts not an int: {facts!r}")
if facts < min_facts:
    fail(
        f"vault_facts={facts} < min {min_facts} — lab-sized vault or wrong instance "
        f"(CT101 reference ~{ref_hint}; do not set GZMO_ALLOW_LAB_VAULT=1)"
    )

print(f"OK: vault_path={vault}")
print(f"OK: vault_facts={facts} (>= {min_facts}; reference ~{ref_hint})")
if honeypot is not None:
    print(f"OK: honeypot_latest={honeypot}")
print(
    json.dumps(
        {
            "ok": True,
            "mode": mode,
            "vault_path": vault,
            "vault_facts": facts,
            "honeypot_latest": honeypot,
            "dual_writer": False,
            "advice": "living_attach_ok — use server label gzmo-living + pi-gzmo-mcp-serve.sh or on-box GZMO_CONFIG",
        },
        indent=2,
    )
)
PY

# Daemon check on ops path (read-only). Accept active|activating; retry once for races.
if [[ "$MODE" == "ssh" ]]; then
  daemon="unknown"
  for _try in 1 2; do
    daemon="$(ssh -o ConnectTimeout=10 -o BatchMode=yes "$HOST" 'systemctl is-active gzmo-daemon' 2>/dev/null || true)"
    daemon="$(printf '%s\n' "${daemon:-unknown}" | head -1)"
    case "$daemon" in
      active|activating) break ;;
      *) sleep 2 ;;
    esac
  done
  case "$daemon" in
    active|activating)
      ok "gzmo-daemon $daemon on $HOST"
      ;;
    *)
      fail "gzmo-daemon is '$daemon' on $HOST (want active|activating) — living metabolism missing; do not invent a workstation writer"
      ;;
  esac
fi

echo "=== PASS: living attach proof ==="
echo "Next: bash scripts/emit-living-mcp-fragment.sh --format hermes"
echo "Never: enable workstation gzmo-serve, GZMO_ALLOW_LAB_VAULT=1, or hand-rolled SSH without GZMO_CONFIG=/opt/gzmo/gzmo.toml"
exit 0
