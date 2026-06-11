#!/usr/bin/env bash
# Push to GitHub using GITHUB_TOKEN from .env.local (never commit tokens).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

ENV_FILE="${ROOT}/.env.local"
if [[ ! -f "$ENV_FILE" ]]; then
  echo "Missing $ENV_FILE — set GITHUB_TOKEN=ghp_…" >&2
  exit 1
fi

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

if [[ -z "${GITHUB_TOKEN:-}" ]]; then
  echo "GITHUB_TOKEN not set in $ENV_FILE" >&2
  exit 1
fi

REF="${1:-HEAD}"
REMOTE="${GITHUB_REMOTE:-https://github.com/maximilianwruhs-cyber/GZMO.git}"

git push "https://x-access-token:${GITHUB_TOKEN}@${REMOTE#https://}" "$REF"
