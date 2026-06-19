#!/usr/bin/env bash
# Discovery ↔ KB feedback loop baseline dashboard.
# Writes data/discovery-kb-metrics/latest.json (and timestamped copy).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
METRICS_DIR="$ROOT/data/discovery-kb-metrics"
VAULT_DB="${GZMO_VAULT_DB:-$ROOT/data/vault.db}"
SYNAPSE="$ROOT/data/Synapse/events.jsonl"
LINK_REGISTRY="${GZMO_LINK_REGISTRY:-$HOME/gzmo_skills/data/pi-mentor-discovery/link-registry.jsonl}"
DISTILL_LOG="${GZMO_DISTILL_LOG:-$HOME/gzmo_skills/data/pi-mentor-discovery/logs/distill.log}"
REPORTS_DIR="${GZMO_DISCOVERY_REPORTS:-$HOME/gzmo_skills/data/pi-mentor-discovery/reports}"

mkdir -p "$METRICS_DIR"
TS="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
STAMP="$(date -u +"%Y%m%dT%H%M%SZ")"

vault_count=0
honeypot_count=0
discovery_sourced=0
distill_dedup_rows=0

if [[ -f "$VAULT_DB" ]]; then
  read -r vault_count honeypot_count discovery_sourced distill_dedup_rows < <(
    python3 - "$VAULT_DB" <<'PY'
import sqlite3, sys
db = sys.argv[1]
c = sqlite3.connect(db)
vault = c.execute("SELECT COUNT(*) FROM semantic_vault").fetchone()[0]
try:
    hp = c.execute("SELECT COUNT(*) FROM honeypot WHERE is_latest=1").fetchone()[0]
except Exception:
    hp = 0
try:
    disc = c.execute(
        "SELECT COUNT(*) FROM semantic_vault WHERE source_file LIKE 'sessions/discovery-%'"
    ).fetchone()[0]
except Exception:
    disc = 0
try:
    dedup = c.execute("SELECT COUNT(*) FROM distill_dedup").fetchone()[0]
except Exception:
    dedup = 0
print(vault, hp, disc, dedup)
PY
  )
fi

# Distill dedup skip rate from distill.log (heuristic; capped at 1.0)
distill_attempts=0
distill_skips=0
if [[ -f "$DISTILL_LOG" ]]; then
  distill_attempts="$(grep -cE 'Distill OK|WARN: distill failed' "$DISTILL_LOG" 2>/dev/null || echo 0)"
  distill_skips="$(grep -cE 'Duplicate transcript \(dedup\)|skipped:.*dedup' "$DISTILL_LOG" 2>/dev/null || echo 0)"
fi
dedup_skip_rate=0
if [[ "$distill_attempts" -gt 0 ]]; then
  dedup_skip_rate="$(python3 -c "print(min(1.0, round(${distill_skips}/${distill_attempts}, 4)))")"
fi

# Link registry stats (30 days)
link_total=0
link_novel_30d=0
if [[ -f "$LINK_REGISTRY" ]]; then
  link_total="$(wc -l < "$LINK_REGISTRY" | tr -d ' ')"
  cutoff="$(date -u -d '30 days ago' +%Y-%m-%d 2>/dev/null || date -u -v-30d +%Y-%m-%d 2>/dev/null || echo "")"
  if [[ -n "$cutoff" ]]; then
    link_novel_30d="$(awk -v cut="$cutoff" '$0 >= cut {c++} END{print c+0}' "$LINK_REGISTRY" 2>/dev/null || echo 0)"
  fi
fi

# Spark ↔ distill overlap (last 500 synapse lines)
spark_sessions=0
distill_sessions=0
overlap=0
if [[ -f "$SYNAPSE" ]]; then
  read -r spark_sessions distill_sessions overlap < <(
    tail -n 500 "$SYNAPSE" | python3 <<'PY'
import json, sys
spark, distill = set(), set()
for line in sys.stdin:
    line = line.strip()
    if not line:
        continue
    try:
        ev = json.loads(line)
    except json.JSONDecodeError:
        continue
    et = ev.get("event_type") or ev.get("type") or ""
    data = ev.get("data") or {}
    sid = data.get("session_id") or data.get("sessionId") or ""
    if et == "spark_complete" and sid:
        spark.add(sid)
    if et == "distill_complete" and sid:
        distill.add(sid)
overlap = len(spark & distill)
print(len(spark), len(distill), overlap)
PY
  )
fi

# Honeypot reject reasons (last 20)
reject_sample='[]'
reject_path="$ROOT/data/honeypot_reject.jsonl"
if [[ -f "$reject_path" ]]; then
  reject_sample="$(tail -n 20 "$reject_path" | jq -s '[.[].reason // .reject_reason // "unknown"] | group_by(.) | map({reason: .[0], count: length})' 2>/dev/null || echo '[]')"
fi

report_count=0
if [[ -d "$REPORTS_DIR" ]]; then
  report_count="$(find "$REPORTS_DIR" -maxdepth 1 -name 'cycle-*.md' 2>/dev/null | wc -l | tr -d ' ')"
fi

out="$METRICS_DIR/latest.json"
jq -n \
  --arg ts "$TS" \
  --arg vault_db "$VAULT_DB" \
  --argjson vault_count "$vault_count" \
  --argjson honeypot_count "$honeypot_count" \
  --argjson discovery_sourced "$discovery_sourced" \
  --argjson distill_dedup_rows "$distill_dedup_rows" \
  --argjson distill_attempts "$distill_attempts" \
  --argjson distill_skips "$distill_skips" \
  --argjson dedup_skip_rate "$dedup_skip_rate" \
  --argjson link_registry_total "$link_total" \
  --argjson link_entries_30d "$link_novel_30d" \
  --argjson spark_sessions "$spark_sessions" \
  --argjson distill_sessions "$distill_sessions" \
  --argjson spark_distill_overlap "$overlap" \
  --argjson discovery_report_count "$report_count" \
  --argjson honeypot_reject_sample "$reject_sample" \
  '{
    generated_at: $ts,
    vault_db: $vault_db,
    counts: {
      semantic_vault: $vault_count,
      honeypot_latest: $honeypot_count,
      discovery_sourced_vault: $discovery_sourced,
      distill_dedup_rows: $distill_dedup_rows,
      cycle_reports: $discovery_report_count
    },
    distill: {
      log_attempts: $distill_attempts,
      log_dedup_skips: $distill_skips,
      dedup_skip_rate_estimate: $dedup_skip_rate
    },
    discovery_links: {
      registry_total: $link_registry_total,
      registry_entries_30d: $link_entries_30d
    },
    spark_distill: {
      spark_sessions_in_tail: $spark_sessions,
      distill_sessions_in_tail: $distill_sessions,
      session_overlap: $spark_distill_overlap
    },
    honeypot_reject_sample: $honeypot_reject_sample,
    targets: {
      novel_links_per_cycle: 2,
      dedup_skip_rate_max: 0.5,
      discovery_sourced_per_week: 15,
      recall_smoke_pass_min: 0.66
    }
  }' > "$out"

cp "$out" "$METRICS_DIR/metrics-${STAMP}.json"
echo "Wrote $out"
jq '{generated_at, counts, distill, discovery_links, spark_distill}' "$out"
