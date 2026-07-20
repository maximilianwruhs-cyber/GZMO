#!/usr/bin/env bash
# GZMO product installer — download release binary (or use local build), init ~/.gzmo, wire MCP.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/maximilianwruhs-cyber/GZMO/main/scripts/install-gzmo.sh | bash
#   # or from a clone:
#   ./scripts/install-gzmo.sh
#
# Env:
#   GZMO_VERSION     release tag (default: latest)
#   GZMO_INSTALL_DIR where to place the binary (default: ~/.local/bin)
#   GZMO_BIN         use this binary instead of downloading
#   GZMO_SKIP_MCP    set to 1 to skip Cursor/Pi mcp.json merge
#   GH_TOKEN         needed if the repo/release is private
set -euo pipefail

REPO="${GZMO_REPO:-maximilianwruhs-cyber/GZMO}"
VERSION="${GZMO_VERSION:-latest}"
INSTALL_DIR="${GZMO_INSTALL_DIR:-${HOME}/.local/bin}"
ASSET_PREFIX="gzmo-x86_64-unknown-linux-gnu"

RED=$'\033[31m'
GREEN=$'\033[32m'
DIM=$'\033[2m'
BOLD=$'\033[1m'
RESET=$'\033[0m'

log() { printf '%s\n' "$*"; }
ok() { printf '%s✔%s %s\n' "$GREEN" "$RESET" "$*"; }
die() { printf '%s[!]%s %s\n' "$RED" "$RESET" "$*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "missing required command: $1"
}

api_get() {
  local url="$1"
  if [[ -n "${GH_TOKEN:-${GITHUB_TOKEN:-}}" ]]; then
    curl -fsSL -H "Authorization: Bearer ${GH_TOKEN:-${GITHUB_TOKEN}}" \
      -H "Accept: application/vnd.github+json" "$url"
  else
    curl -fsSL -H "Accept: application/vnd.github+json" "$url"
  fi
}

download_asset() {
  local url="$1" dest="$2"
  if [[ -n "${GH_TOKEN:-${GITHUB_TOKEN:-}}" ]]; then
    curl -fsSL -L -H "Authorization: Bearer ${GH_TOKEN:-${GITHUB_TOKEN}}" \
      -H "Accept: application/octet-stream" -o "$dest" "$url"
  else
    curl -fsSL -L -o "$dest" "$url"
  fi
}

resolve_release_json() {
  local api
  if [[ "$VERSION" == "latest" ]]; then
    api="https://api.github.com/repos/${REPO}/releases/latest"
  else
    api="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
  fi
  api_get "$api"
}

install_from_release() {
  need_cmd curl
  need_cmd python3
  need_cmd tar

  local json
  json="$(resolve_release_json)" || return 1

  local api_url browser_url name tag
  eval "$(
    printf '%s' "$json" | ASSET_PREFIX="$ASSET_PREFIX" python3 -c '
import json, os, shlex, sys
rel = json.load(sys.stdin)
prefix = os.environ["ASSET_PREFIX"]
for a in rel.get("assets") or []:
    name = a.get("name") or ""
    if name.startswith(prefix) and name.endswith(".tar.gz"):
        print("api_url=" + shlex.quote(a.get("url") or ""))
        print("browser_url=" + shlex.quote(a.get("browser_download_url") or ""))
        print("name=" + shlex.quote(name))
        print("tag=" + shlex.quote(rel.get("tag_name") or ""))
        raise SystemExit(0)
raise SystemExit(1)
'
  )" || return 1

  local tmp dl found
  tmp="$(mktemp -d)"
  # Explicit cleanup — RETURN traps + set -u re-fire after main() and trip on unset tmp.
  dl="$browser_url"
  if [[ -n "${GH_TOKEN:-${GITHUB_TOKEN:-}}" && -n "$api_url" ]]; then
    dl="$api_url"
  fi
  if [[ -z "$dl" ]]; then
    rm -rf "$tmp"
    return 1
  fi

  log "${BOLD}Downloading${RESET} ${name} (${tag:-$VERSION})..."
  if ! download_asset "$dl" "${tmp}/asset"; then
    rm -rf "$tmp"
    return 1
  fi

  mkdir -p "$INSTALL_DIR"
  tar -xzf "${tmp}/asset" -C "$tmp"
  found="$(find "$tmp" -type f -name gzmo | head -1)"
  if [[ -z "$found" ]]; then
    rm -rf "$tmp"
    return 1
  fi
  install -m 755 "$found" "${INSTALL_DIR}/gzmo"
  rm -rf "$tmp"
  ok "Installed ${INSTALL_DIR}/gzmo"
  return 0
}

install_from_local_build() {
  local root=""
  if [[ -n "${BASH_SOURCE[0]:-}" && -f "${BASH_SOURCE[0]}" ]]; then
    root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  fi
  [[ -n "$root" && -x "${root}/target/release/gzmo" ]] || return 1
  mkdir -p "$INSTALL_DIR"
  install -m 755 "${root}/target/release/gzmo" "${INSTALL_DIR}/gzmo"
  ok "Installed from local build → ${INSTALL_DIR}/gzmo"
  return 0
}

merge_mcp_standalone() {
  python3 - <<'PY'
import json, os, pathlib
home = pathlib.Path(os.environ["HOME"]) / ".gzmo"
frag_path = home / "mcp.json"
if not frag_path.exists():
    raise SystemExit("missing ~/.gzmo/mcp.json — run gzmo init first")
frag = json.loads(frag_path.read_text())
gm = frag["mcpServers"]["gzmo-memory"]
targets = [
    (pathlib.Path.home() / ".cursor" / "mcp.json", "Cursor"),
    (pathlib.Path.home() / ".pi" / "agent" / "mcp.json", "Pi"),
    (pathlib.Path.home() / ".config" / "mcp" / "mcp.json", "Global"),
]
for path, label in targets:
    path.parent.mkdir(parents=True, exist_ok=True)
    cur = {"mcpServers": {}}
    if path.exists():
        cur = json.loads(path.read_text())
        cur.setdefault("mcpServers", {})
    cur["mcpServers"]["gzmo-memory"] = gm
    path.write_text(json.dumps(cur, indent=2) + "\n")
    print(f"[OK] {label} → {path}")
PY
}

main() {
  log ""
  log "${BOLD}GZMO — lite Memory MCP (bootstrap)${RESET}"
  log "${DIM}SQLite vault · Cursor/Pi · stdio MCP · no sidecars / no overnight writer${RESET}"
  log "${DIM}USP (full living on one airgapped box): bash scripts/install-living-airgap.sh${RESET}"
  log ""

  local bin=""
  if [[ -n "${GZMO_BIN:-}" ]]; then
    [[ -x "$GZMO_BIN" ]] || die "GZMO_BIN not executable: $GZMO_BIN"
    mkdir -p "$INSTALL_DIR"
    install -m 755 "$GZMO_BIN" "${INSTALL_DIR}/gzmo"
    bin="${INSTALL_DIR}/gzmo"
    ok "Installed ${bin}"
  elif install_from_release; then
    bin="${INSTALL_DIR}/gzmo"
  elif install_from_local_build; then
    bin="${INSTALL_DIR}/gzmo"
  else
    die "No release asset and no target/release/gzmo. Build with: cargo build --release -p gzmo-cli (private repo: export GH_TOKEN=...)"
  fi

  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *) log "${DIM}Tip: add ${INSTALL_DIR} to PATH${RESET}" ;;
  esac

  log ""
  log "${BOLD}Initializing${RESET} ~/.gzmo ..."
  "$bin" init --force --bin "$bin"
  ok "Product home ready"

  if [[ "${GZMO_SKIP_MCP:-0}" != "1" ]]; then
    local root=""
    if [[ -n "${BASH_SOURCE[0]:-}" && -f "${BASH_SOURCE[0]}" ]]; then
      root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
    fi
    if [[ -n "$root" && -x "${root}/scripts/install-product-mcp.sh" ]]; then
      GZMO_BIN="$bin" GZMO_HOME="${HOME}/.gzmo" bash "${root}/scripts/install-product-mcp.sh"
    else
      merge_mcp_standalone
    fi
    ok "MCP wired (gzmo-memory)"
  fi

  log ""
  log "${GREEN}${BOLD}Done.${RESET} Lite attach ready — Cursor/Pi: ${BOLD}gzmo_memory_status${RESET}, then ${BOLD}gzmo_memory_search${RESET}."
  log "${DIM}Lite docs: docs/PRODUCT_MCP.md · USP living: docs/AIRGAP_LIVING.md · MCP: docs/MCP_LOCAL_ATTACH.md${RESET}"
  log ""
}

main "$@"
