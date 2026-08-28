#!/usr/bin/env bash
# Hero installer narrative: full living Keep on THIS machine (airgap USP).
# Brings up sidecar compose pin, prints daemon + local MCP next steps.
# Does NOT start a second overnight writer if CT101 (or another host) already owns metabolism.
#
#   bash scripts/install-living-airgap.sh
#   GZMO_BIN=./target/release/gzmo bash scripts/install-living-airgap.sh
#
# Lite bootstrap (no sidecars / no overnight) remains:
#   bash scripts/install-gzmo.sh
#
# Docs: docs/AIRGAP_LIVING.md · docs/adr/ADR-0004-airgap-living-usp.md
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INSTALL_DIR="${GZMO_INSTALL_DIR:-${HOME}/.local/bin}"
LIVING_HOME="${GZMO_LIVING_HOME:-${HOME}/.gzmo-living}"
COMPOSE_DIR="$ROOT/deploy/living-appliance"

RED=$'\033[31m'
GREEN=$'\033[32m'
DIM=$'\033[2m'
BOLD=$'\033[1m'
RESET=$'\033[0m'

log() { printf '%s\n' "$*"; }
ok() { printf '%s✔%s %s\n' "$GREEN" "$RESET" "$*"; }
warn() { printf '%s!%s %s\n' "$RED" "$RESET" "$*"; }
die() { printf '%s[!]%s %s\n' "$RED" "$RESET" "$*" >&2; exit 1; }

log ""
log "${BOLD}GZMO — airgap living (USP)${RESET}"
log "${DIM}One box · local engines · Redis/Qdrant/Neo4j · overnight metabolism · local MCP${RESET}"
log ""

# Dual-writer guard (workstation)
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  die "gzmo-serve is active on this workstation — stop it before enabling another overnight writer (ADR-0003)"
fi

# Binary
bin=""
if [[ -n "${GZMO_BIN:-}" ]]; then
  [[ -x "$GZMO_BIN" ]] || die "GZMO_BIN not executable: $GZMO_BIN"
  bin="$GZMO_BIN"
elif [[ -x "$ROOT/target/release/gzmo" ]]; then
  bin="$ROOT/target/release/gzmo"
elif command -v gzmo >/dev/null 2>&1; then
  bin="$(command -v gzmo)"
else
  die "No gzmo binary. Build: cargo build --release -p gzmo-cli  (or set GZMO_BIN=...)"
fi
ok "Using binary: $bin"

# Sidecars
if [[ ! -f "$COMPOSE_DIR/docker-compose.yml" ]]; then
  die "missing compose pin: $COMPOSE_DIR/docker-compose.yml"
fi
if [[ ! -f "$COMPOSE_DIR/.env" ]]; then
  cp "$COMPOSE_DIR/.env.example" "$COMPOSE_DIR/.env"
  warn "Wrote $COMPOSE_DIR/.env from example — set NEO4J_AUTH before production use"
fi

log ""
log "${BOLD}Starting living appliance sidecars${RESET} (Redis / Qdrant / Neo4j) ..."
bash "$ROOT/scripts/living-appliance-up.sh"
ok "Sidecar pin up + gate/smoke attempted"

# Living home sketch (not product ~/.gzmo)
mkdir -p "$LIVING_HOME/data"
EXAMPLE="$ROOT/config/living-appliance.gzmo.toml.example"
TARGET_TOML="$LIVING_HOME/gzmo.toml"
if [[ ! -f "$TARGET_TOML" ]]; then
  if [[ -f "$EXAMPLE" ]]; then
    cp "$EXAMPLE" "$TARGET_TOML"
    # Point vault under living home
    if ! grep -q 'vault_db' "$TARGET_TOML" 2>/dev/null; then
      cat >>"$TARGET_TOML" <<EOF

[memory]
vault_db = "$LIVING_HOME/data/vault.db"
EOF
    fi
    ok "Wrote living config sketch: $TARGET_TOML"
  else
    warn "No living-appliance.gzmo.toml.example — create $TARGET_TOML manually"
  fi
else
  ok "Living config exists: $TARGET_TOML"
fi

# Local MCP fragment (stdio) for this box
MCP_FRAG="$LIVING_HOME/mcp-living.fragment.json"
python3 - <<PY
import json
from pathlib import Path
bin_path = "$bin"
cfg = "$TARGET_TOML"
frag = {
  "mcpServers": {
    "gzmo-living": {
      "command": bin_path,
      "args": ["mcp-serve"],
      "env": {"GZMO_CONFIG": cfg},
    }
  }
}
path = Path("$MCP_FRAG")
path.write_text(json.dumps(frag, indent=2) + "\n", encoding="utf-8")
print(path)
PY
ok "Local MCP fragment: $MCP_FRAG"

log ""
log "${BOLD}Next (on this box — sole overnight writer)${RESET}"
log "  1. Point [llm]/engines at ${BOLD}127.0.0.1${RESET} Prime/embed (airgap honesty)"
log "  2. Set Neo4j password in compose .env + process env (never commit)"
log "  3. Init vault if empty:  GZMO_CONFIG=$TARGET_TOML $bin init --force   # or migrate"
log "  4. Run daemon as sole writer:  GZMO_CONFIG=$TARGET_TOML $bin daemon"
log "     (systemd: see scripts/install-daemon-systemd.sh — only if THIS box owns metabolism)"
log "  5. Merge $MCP_FRAG into Cursor/Pi mcp.json as ${BOLD}gzmo-living${RESET}"
log "     (stdio only — docs/MCP_LOCAL_ATTACH.md)"
log "  6. Smoke (install path, not living GREEN):  bash scripts/airgap-living-install-smoke.sh"
log "  7. Quality (sole writer only):  bash scripts/keep-quality-gate.sh"
log ""
log "${DIM}If another host (e.g. CT101) already runs overnight writers, do NOT enable daemon here.${RESET}"
log "${DIM}Lite bootstrap without sidecars: bash scripts/install-gzmo.sh${RESET}"
log "${DIM}Docs: docs/AIRGAP_LIVING.md${RESET}"
log ""
ok "Airgap living scaffold ready"
