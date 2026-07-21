#!/usr/bin/env bash
# Install TinyFolder overnight timer on CT101 (run on the living host).
#
#   ssh ct101 'bash /opt/gzmo/current/scripts/install-tinyfolder-overnight-timer.sh'
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UNIT_SRC="$ROOT/systemd"
[[ -f "$UNIT_SRC/gzmo-tinyfolder-overnight.timer" ]] || {
  echo "[!] missing $UNIT_SRC/gzmo-tinyfolder-overnight.timer" >&2
  exit 1
}

mkdir -p /opt/gzmo/data/inbox/processed /opt/gzmo/data/tinyfolder
if [[ ! -f /opt/gzmo/data/inbox/README.md ]]; then
  cat >/opt/gzmo/data/inbox/README.md <<'EOF'
# TinyFolder living inbox

Drop markdown notes here. Nightly timer (~02:45 UTC) enqueues up to 3 pending
notes into distill via `tinyfolder-overnight.sh --on-host` (no CLI chat).

Processed files move to `processed/`.
EOF
fi

install -m 644 "$UNIT_SRC/gzmo-tinyfolder-overnight.service" /etc/systemd/system/
install -m 644 "$UNIT_SRC/gzmo-tinyfolder-overnight.timer" /etc/systemd/system/
chmod +x "$ROOT/scripts/tinyfolder-overnight.sh"
systemctl daemon-reload
systemctl enable --now gzmo-tinyfolder-overnight.timer
systemctl status gzmo-tinyfolder-overnight.timer --no-pager || true
echo "[OK] tinyfolder overnight timer enabled (02:45 UTC)"
echo "     dry-run: bash $ROOT/scripts/tinyfolder-overnight.sh --on-host --dry-run"
