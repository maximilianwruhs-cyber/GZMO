#!/usr/bin/env bash
# Shared env resolution for herdr ↔ GZMO metabolism plugin.
set -euo pipefail

PLUGIN_ROOT="${HERDR_PLUGIN_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
# Prefer herdr-injected dirs; never write state into the git-tracked plugin root.
CONFIG_DIR="${HERDR_PLUGIN_CONFIG_DIR:-${XDG_CONFIG_HOME:-$HOME/.config}/herdr/plugins/gzmo.metabolism}"
STATE_DIR="${HERDR_PLUGIN_STATE_DIR:-${XDG_STATE_HOME:-$HOME/.local/state}/herdr/plugins/gzmo.metabolism}"
mkdir -p "$CONFIG_DIR" "$STATE_DIR"

# Optional operator overrides: HERDR_PLUGIN_CONFIG_DIR/env
if [[ -f "${CONFIG_DIR}/env" ]]; then
  # shellcheck disable=SC1091
  set -a
  # shellcheck disable=SC1090
  source "${CONFIG_DIR}/env"
  set +a
fi

# Repo root = integrations/../..
REPO_ROOT="$(cd "${PLUGIN_ROOT}/../.." && pwd)"

resolve_gzmo_bin() {
  if [[ -n "${GZMO_BIN:-}" && -x "${GZMO_BIN}" ]]; then
    printf '%s' "$GZMO_BIN"
    return
  fi
  if command -v gzmo >/dev/null 2>&1; then
    command -v gzmo
    return
  fi
  for cand in \
    "${CARGO_TARGET_DIR:-}/release/gzmo" \
    "${REPO_ROOT}/target/release/gzmo" \
    /home/gzmo/github-clone/temp-bench/target/release/gzmo; do
    if [[ -n "$cand" && -x "$cand" ]]; then
      printf '%s' "$cand"
      return
    fi
  done
  echo "[!] gzmo binary not found; set GZMO_BIN or install via scripts/install-gzmo.sh" >&2
  return 1
}

resolve_gzmo_config() {
  if [[ -n "${GZMO_CONFIG:-}" && -f "${GZMO_CONFIG}" ]]; then
    printf '%s' "$GZMO_CONFIG"
    return
  fi
  # Prefer living next when present (mux metabolism story).
  if [[ -f "${REPO_ROOT}/config/gzmo.toml" ]]; then
    printf '%s' "${REPO_ROOT}/config/gzmo.toml"
    return
  fi
  if [[ -f "${HOME}/.gzmo/gzmo.toml" ]]; then
    printf '%s' "${HOME}/.gzmo/gzmo.toml"
    return
  fi
  echo "[!] No GZMO_CONFIG; run gzmo init or set config in ${CONFIG_DIR}/env" >&2
  return 1
}

export_gzmo_env() {
  export GZMO_BIN
  GZMO_BIN="$(resolve_gzmo_bin)"
  export GZMO_CONFIG
  GZMO_CONFIG="$(resolve_gzmo_config)"
  export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
  # Living next when using clone config.
  if [[ "$GZMO_CONFIG" == *"/config/gzmo.toml" ]]; then
    export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
  fi
}
