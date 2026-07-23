#!/usr/bin/env bash
# Install workstation user systemd timers for ecosystem evolve plane.
# Does NOT enable gzmo-serve. CT101 remains the overnight writer.
#
#   bash scripts/install-ecosystem-evolve-timers.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UNIT_SRC="$ROOT/systemd/user"
DST="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
mkdir -p "$DST"

units=(
  gzmo-ops-health.service
  gzmo-ops-health.timer
  gzmo-research-scan.service
  gzmo-research-scan.timer
  gzmo-kg-reconcile-dry.service
  gzmo-kg-reconcile-dry.timer
  gzmo-organ-watchdog.service
  gzmo-organ-watchdog.timer
  gzmo-ecosystem-evolve-daily.service
  gzmo-ecosystem-evolve-daily.timer
  gzmo-ecosystem-evolve-weekly.service
  gzmo-ecosystem-evolve-weekly.timer
)

for u in "${units[@]}"; do
  [[ -f "$UNIT_SRC/$u" ]] || { echo "[!] missing $UNIT_SRC/$u" >&2; exit 1; }
  install -m 644 "$UNIT_SRC/$u" "$DST/$u"
done

chmod +x \
  "$ROOT/scripts/ops-health.sh" \
  "$ROOT/scripts/research-scan.sh" \
  "$ROOT/scripts/kg-reconcile-dry.sh" \
  "$ROOT/scripts/ecosystem-evolve-daily.sh" \
  "$ROOT/scripts/ecosystem-evolve-weekly.sh" \
  "$ROOT/scripts/organ-watchdog-check.sh" \
  "$ROOT/scripts/install-ecosystem-evolve-timers.sh"

systemctl --user daemon-reload
systemctl --user enable --now \
  gzmo-ops-health.timer \
  gzmo-research-scan.timer \
  gzmo-kg-reconcile-dry.timer \
  gzmo-organ-watchdog.timer \
  gzmo-ecosystem-evolve-daily.timer \
  gzmo-ecosystem-evolve-weekly.timer

# Keep living smoke if present
systemctl --user enable --now gzmo-ct101-living-smoke.timer 2>/dev/null || true

echo "[OK] ecosystem evolve timers enabled (user)"
systemctl --user list-timers --all | rg -i 'gzmo-|okforge' || true
echo
echo "Manual run:"
echo "  systemctl --user start gzmo-ops-health.service"
echo "  systemctl --user start gzmo-ecosystem-evolve-daily.service"
echo "  systemctl --user start gzmo-ecosystem-evolve-weekly.service"
