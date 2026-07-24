#!/usr/bin/env bash
# Skill-local entry for Telegram /character (GZMO-safe chooser).
set -euo pipefail
export OPENCLAW_CHARACTER_FORCE=1
CHOOSER="${OPENCLAW_CHARACTER_CHOOSER:-$HOME/github-clone/GZMO/scripts/openclaw-choose-character.sh}"
if [[ ! -f "$CHOOSER" ]]; then
  echo "REFUSE: chooser missing: $CHOOSER" >&2
  exit 2
fi
if [[ $# -eq 0 ]]; then
  set -- who
fi
exec bash "$CHOOSER" "$@"
