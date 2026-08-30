#!/usr/bin/env bash
# Nightly living-Keep research → drafts only.
# Writes ~/.gzmo-living/data/research-intel/. Never touches vault.db.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LIVING="${GZMO_LIVING_HOME:-$HOME/.gzmo-living}"
OUT="$LIVING/data/research-intel"

if [[ -f "$LIVING/.env" ]]; then
  set -a
  # shellcheck disable=SC1091
  source "$LIVING/.env"
  set +a
fi

mkdir -p "$OUT"
export PYTHONPATH="$ROOT/scripts${PYTHONPATH:+:$PYTHONPATH}"
python3 "$ROOT/scripts/living_research.py" \
  --living-home "$LIVING" \
  --repo "$ROOT" \
  --out "$OUT"

# shellcheck source=scripts/lib-research-dedup.sh
source "$ROOT/scripts/lib-research-dedup.sh"
if [[ -f "$OUT/latest.json" ]]; then
  stamp="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1])).get("generated_at","")[:20])' "$OUT/latest.json" 2>/dev/null || date -u +%Y%m%dT%H%M%SZ)"
  dedup_findings "$OUT/latest.json" "$OUT/seen.jsonl"
  dedup_render_latest "$OUT/latest.json" "$OUT/latest.md" intel "$stamp" "${RESEARCH_INTEL_TOP:-3}"
  dedup_seen_update "$OUT/latest.json" "$OUT/seen.jsonl" "$(date -u +%Y-%m-%d)"
fi

echo "[i] living research-intel drafts → $OUT/latest.md"
if [[ "${JULES_DISPATCH:-}" == "1" ]]; then
  echo "[i] Jules dispatch is on (one real-file mission / 7 days per file)"
fi
