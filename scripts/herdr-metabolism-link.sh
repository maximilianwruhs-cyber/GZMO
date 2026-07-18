#!/usr/bin/env bash
# Link (or re-link) the local herdr ↔ GZMO metabolism plugin.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PLUGIN="${ROOT}/integrations/herdr-gzmo-metabolism"

if ! command -v herdr >/dev/null 2>&1; then
  echo "[!] herdr not on PATH" >&2
  exit 1
fi

chmod +x "${PLUGIN}/bin/"*.sh
# Unlink quietly if already linked, then link.
herdr plugin unlink gzmo.metabolism >/dev/null 2>&1 || true
herdr plugin link "$PLUGIN"
herdr plugin enable gzmo.metabolism >/dev/null 2>&1 || true

echo "[OK] linked gzmo.metabolism → $PLUGIN"
echo "Actions:"
herdr plugin action list --plugin gzmo.metabolism 2>/dev/null || herdr plugin list --plugin gzmo.metabolism
echo ""
echo "Try:"
echo "  herdr plugin action invoke gzmo.metabolism.ensure-mcp"
echo "  TAKEAWAY='…' herdr plugin action invoke gzmo.metabolism.session-close"
echo "  herdr plugin pane open --plugin gzmo.metabolism --entrypoint close-ritual"
echo "Optional config: \$(herdr plugin config-dir gzmo.metabolism)/env"
