#!/usr/bin/env bash
# Post-distill recall smoke: verify LINK recall_query hits in gzmo memory search.
# Reads ~/gzmo_skills/data/pi-mentor-discovery/link-registry.jsonl (recent entries).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
LINK_REGISTRY="${GZMO_LINK_REGISTRY:-$HOME/gzmo_skills/data/pi-mentor-discovery/link-registry.jsonl}"
OUT_DIR="$ROOT/data/discovery-kb-metrics"
LIMIT="${DISCOVERY_RECALL_SMOKE_LIMIT:-5}"

mkdir -p "$OUT_DIR"
TS="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
STAMP="$(date -u +"%Y%m%dT%H%M%SZ")"

if [[ ! -x "$GZMO_BIN" ]]; then
  echo "WARN: gzmo binary missing at $GZMO_BIN — skipping recall smoke" >&2
  exit 0
fi

if [[ ! -f "$LINK_REGISTRY" ]]; then
  echo "No link registry at $LINK_REGISTRY — nothing to smoke" >&2
  exit 0
fi

results='[]'
passed=0
total=0

while IFS= read -r line; do
  [[ -z "$line" ]] && continue
  query="$(echo "$line" | jq -r '.recall_query // empty')"
  link_id="$(echo "$line" | jq -r '.link_id // "unknown"')"
  [[ -z "$query" ]] && continue
  total=$((total + 1))
  [[ "$total" -gt "$LIMIT" ]] && break

  set +e
  search_out="$("$GZMO_BIN" memory search "$query" 2>/dev/null | head -c 4000)"
  search_rc=$?
  set -e

  hit=0
  if [[ $search_rc -eq 0 ]] && [[ -n "$search_out" ]]; then
    hit=1
    passed=$((passed + 1))
  fi

  results="$(jq -n \
    --argjson arr "$results" \
    --arg link_id "$link_id" \
    --arg query "$query" \
    --argjson hit "$hit" \
    --arg preview "${search_out:0:200}" \
    '$arr + [{link_id: $link_id, recall_query: $query, hit: ($hit == 1), preview: $preview}]')"
done < <(tail -n "$LIMIT" "$LINK_REGISTRY")

pass_rate=0
if [[ "$total" -gt 0 ]]; then
  pass_rate="$(python3 -c "print(round(${passed}/${total}, 4))")"
fi

out="$OUT_DIR/recall-smoke-${STAMP}.json"
jq -n \
  --arg ts "$TS" \
  --argjson total "$total" \
  --argjson passed "$passed" \
  --argjson pass_rate "$pass_rate" \
  --argjson results "$results" \
  '{generated_at: $ts, total_queries: $total, passed: $passed, pass_rate: $pass_rate, results: $results}' > "$out"

ln -sfn "$out" "$OUT_DIR/recall-smoke-latest.json"
echo "Recall smoke: ${passed}/${total} (rate=${pass_rate}) → $out"
