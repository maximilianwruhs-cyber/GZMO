#!/usr/bin/env bash
# Soft Keep check: is the published GitHub release still the stranger appliance tip?
# Exit 0 always (soft). Writes data-next/release-freshness/latest.{json,md}.
#
#   bash scripts/release-freshness-check.sh
# Env:
#   GZMO_REPO          default maximilianwruhs-cyber/GZMO
#   RELEASE_FRESH_MAX  commits tip may be ahead of latest tag and still PASS (default 5)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/release-freshness"
REPO="${GZMO_REPO:-maximilianwruhs-cyber/GZMO}"
MAX="${RELEASE_FRESH_MAX:-5}"
mkdir -p "$OUT"

api_get() {
  local url="$1"
  if [[ -n "${GH_TOKEN:-${GITHUB_TOKEN:-}}" ]]; then
    curl -fsSL -H "Authorization: Bearer ${GH_TOKEN:-${GITHUB_TOKEN}}" \
      -H "Accept: application/vnd.github+json" "$url"
  else
    curl -fsSL -H "Accept: application/vnd.github+json" "$url"
  fi
}

TAG=""
PUBLISHED=""
COMMITS_AHEAD=0
STATUS="HOLD"
ADVICE="could_not_resolve_latest_release"
OK=0

if json="$(api_get "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null)"; then
  eval "$(
    printf '%s' "$json" | python3 -c '
import json, shlex, sys
rel = json.load(sys.stdin)
print("TAG=" + shlex.quote(rel.get("tag_name") or ""))
print("PUBLISHED=" + shlex.quote(rel.get("published_at") or ""))
'
  )"
fi

if [[ -n "$TAG" ]] && git -C "$ROOT" rev-parse "$TAG" >/dev/null 2>&1; then
  COMMITS_AHEAD="$(git -C "$ROOT" rev-list --count "${TAG}..HEAD" 2>/dev/null || echo 999)"
  if [[ "$COMMITS_AHEAD" -le "$MAX" ]]; then
    STATUS="PASS"
    ADVICE="release_fresh — latest ${TAG} matches tip (ahead=${COMMITS_AHEAD})"
    OK=1
  else
    STATUS="HOLD"
    ADVICE="release_stale — tip is ${COMMITS_AHEAD} commits ahead of ${TAG}; cut a new v* tag so install-gzmo.sh ships Keep features"
    OK=0
  fi
elif [[ -n "$TAG" ]]; then
  STATUS="HOLD"
  ADVICE="release_tag_unresolved — ${TAG} not in this clone; fetch tags or check remote"
  OK=0
fi

export OUT TAG PUBLISHED COMMITS_AHEAD STATUS ADVICE OK MAX REPO
python3 - <<'PY'
import json, os, pathlib
from datetime import datetime, timezone
out = pathlib.Path(os.environ["OUT"])
payload = {
    "schema": "gzmo.release.freshness/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "repo": os.environ["REPO"],
    "latest_tag": os.environ.get("TAG") or None,
    "published_at": os.environ.get("PUBLISHED") or None,
    "commits_ahead": int(os.environ["COMMITS_AHEAD"]),
    "max_ahead": int(os.environ["MAX"]),
    "status": os.environ["STATUS"],
    "ok": os.environ["OK"] == "1",
    "advice": os.environ["ADVICE"],
    "stranger_install": "curl -fsSL https://raw.githubusercontent.com/maximilianwruhs-cyber/GZMO/main/scripts/install-gzmo.sh | bash",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
(out / "latest.md").write_text(
    f"# Release freshness\n\n"
    f"**Status:** {payload['status']}\n\n"
    f"- Latest tag: `{payload['latest_tag']}`\n"
    f"- Published: {payload['published_at']}\n"
    f"- Tip ahead: {payload['commits_ahead']} (max {payload['max_ahead']})\n"
    f"- Advice: {payload['advice']}\n"
)
print(json.dumps({"status": payload["status"], "ok": payload["ok"], "advice": payload["advice"]}, indent=2))
PY
