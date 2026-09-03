#!/usr/bin/env bash
# Hermetic fixture mission producer for Stage 1 acceptance.
# Reads only fixture files under the checkout and writes under GZMO_DATA_NEXT.
# No network, no ambient scripts, no credentials.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:?GZMO_DATA_NEXT required}"
OUT="$DATA/opportunity-discovery"
OPP="$ROOT/research/opportunities"
mkdir -p "$OUT"

# RFC3339 millis UTC for MissionAdapter refresh window.
# Prefer GNU date (PATH=/usr/bin:/bin); python3 only as local fallback.
utc_now() {
  if NOW="$(date -u +"%Y-%m-%dT%H:%M:%S.%3NZ" 2>/dev/null)" && [[ "$NOW" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}\.[0-9]{3}Z$ ]]; then
    printf '%s\n' "$NOW"
    return 0
  fi
  if command -v python3 >/dev/null 2>&1; then
    python3 - <<'PY'
from datetime import datetime, timezone
print(datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%f")[:-3] + "Z")
PY
    return 0
  fi
  echo "utc_now: need GNU date %3N or python3" >&2
  exit 1
}

# Exactly one active bet (status: active) — two-active or zero fail closed.
mapfile -t ACTIVE < <(
  for f in "$OPP"/*.md; do
    [[ -f "$f" ]] || continue
    if grep -q '^status:[[:space:]]*active[[:space:]]*$' "$f"; then
      printf '%s\n' "$f"
    fi
  done | sort
)
if [[ "${#ACTIVE[@]}" -ne 1 ]]; then
  NOW="$(utc_now)"
  cat >"$OUT/next-mission.json" <<EOF
{"schema":"gzmo.opportunity.next_mission/v1","generated_at":"$NOW","ok":false,"advice":"need_exactly_one_active_bet have=${#ACTIVE[@]}"}
EOF
  echo "need_exactly_one_active_bet have=${#ACTIVE[@]}" >&2
  exit 1
fi

BET_FILE="${ACTIVE[0]}"
BET_ID="$(grep -E '^id:[[:space:]]*' "$BET_FILE" | head -1 | sed 's/^id:[[:space:]]*//;s/[[:space:]]*$//')"
TITLE="$(grep -E '^title:[[:space:]]*' "$BET_FILE" | head -1 | sed 's/^title:[[:space:]]*//;s/[[:space:]]*$//')"
SCORE="$(grep -E '^score:[[:space:]]*' "$BET_FILE" | head -1 | sed 's/^score:[[:space:]]*//;s/[[:space:]]*$//')"
SHIP="$(grep -E '^ship_bar:[[:space:]]*' "$BET_FILE" | head -1 | sed 's/^ship_bar:[[:space:]]*//;s/[[:space:]]*$//')"

if [[ -z "$BET_ID" || -z "$TITLE" || -z "$SCORE" || "$SHIP" != "true" ]]; then
  echo "fixture bet metadata incomplete" >&2
  exit 1
fi

MD="$OUT/next-mission.md"
cat >"$MD" <<EOF
# Mission card — ${TITLE}

## Mission

**Bet id:** \`${BET_ID}\`
**Title:** ${TITLE}
**Score:** ${SCORE}
**Source:** \`research/opportunities/$(basename "$BET_FILE")\`

Body of the fixture mission. Stay hermetic; mutate only allowed paths.

## Constraints

- stay hermetic
- no network
- no credentials
- one normalized candidate commit only

## Verify

\`\`\`bash
true
\`\`\`
EOF

# Absolute path required by mission_md contract.
if command -v realpath >/dev/null 2>&1; then
  MD_ABS="$(realpath "$MD")"
else
  MD_ABS="$(cd "$(dirname "$MD")" && pwd)/$(basename "$MD")"
fi

NOW="$(utc_now)"
JSON="$OUT/next-mission.json"
cat >"$JSON" <<EOF
{"schema":"gzmo.opportunity.next_mission/v1","generated_at":"${NOW}","ok":true,"bet_id":"${BET_ID}","title":"${TITLE}","score":${SCORE},"ship_bar":true,"mission_md":"${MD_ABS}","advice":"fixture-opportunity-ok","automation_note":"hermetic-fixture-only"}
EOF

printf '%s\n' "{}"
exit 0
