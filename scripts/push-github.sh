#!/usr/bin/env bash
# Push GZMO repos to GitHub (manual fallback — discovery cycle auto-pushes via discovery-github-backup.sh).
set -euo pipefail

GZMO_SKILLS_ROOT="${GZMO_SKILLS_ROOT:-$HOME/gzmo_skills}"
BACKUP_SCRIPT="$GZMO_SKILLS_ROOT/scripts/discovery-github-backup.sh"

if [[ -x "$BACKUP_SCRIPT" ]]; then
  exec "$BACKUP_SCRIPT" --flush "$@"
fi

# Legacy single-repo push if backup script missing
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
ENV_FILE="${ROOT}/.env.local"
[[ -f "$ENV_FILE" ]] || { echo "Missing $ENV_FILE" >&2; exit 1; }
set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a
[[ -n "${GITHUB_TOKEN:-}" ]] || { echo "GITHUB_TOKEN not set" >&2; exit 1; }
REF="${1:-HEAD}"
REMOTE="${GITHUB_REMOTE:-https://github.com/maximilianwruhs-cyber/GZMO.git}"
git push "https://x-access-token:${GITHUB_TOKEN}@${REMOTE#https://}" "$REF"
