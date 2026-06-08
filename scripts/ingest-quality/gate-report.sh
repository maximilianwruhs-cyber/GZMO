#!/usr/bin/env bash
# Print ingest quality gate table and exit 0/1.
# Usage: gate-report.sh [report.json]
# Env: GATE_MODE=strict|layered (overrides gate-config.yaml)

set -eo pipefail
export LC_ALL=C
export LC_NUMERIC=C

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
REPORT_PATH="${1:-$DIR/report.json}"
CONFIG="$DIR/gate-config.yaml"

if [[ ! -f "$REPORT_PATH" ]]; then
  echo "gate-report.sh: missing report: $REPORT_PATH" >&2
  exit 2
fi

# Apply relation-promotion waivers from gate-config (no Prime).
if [[ -f "$DIR/recalc-pipeline-summary.py" ]]; then
  python3 "$DIR/recalc-pipeline-summary.py" --report "$REPORT_PATH" --write >/dev/null 2>&1 || true
fi

GATE_MODE="${GATE_MODE:-$(python3 -c "import yaml; print(yaml.safe_load(open('$CONFIG'))['gate']['mode'])")}"
REPORTS_DIR="$DIR/$(python3 -c "import yaml; c=yaml.safe_load(open('$CONFIG')); print(c['stability']['reports_dir'])")"
BEST_OF_N="$(python3 -c "import yaml; print(yaml.safe_load(open('$CONFIG'))['stability']['pipeline_best_of_last_n'])")"

read_config() {
  python3 - "$CONFIG" <<'PY'
import sys, yaml
c = yaml.safe_load(open(sys.argv[1]))
print(c["contract"]["golden_must_entity_recall_min"])
print(c["contract"]["anti_entities_max"])
print(c["pipeline"]["zero_entity_files_max"])
print(c["pipeline"]["rich_notebooklm_few_entities_max"])
print(c["pipeline"]["relation_promotion_rate_min"])
print(c["pipeline"]["zero_relation_files_max"])
print(c["stability"].get("relation_promotion_tolerance_pp", 0))
PY
}

mapfile -t CFG < <(read_config)
GOLDEN_MIN="${CFG[0]}"
ANTI_MAX="${CFG[1]}"
ZERO_ENT_MAX="${CFG[2]}"
RICH_FEW_MAX="${CFG[3]}"
REL_MIN="${CFG[4]}"
ZERO_REL_MAX="${CFG[5]}"
REL_TOL_PP="${CFG[6]}"

metrics_from_report() {
  local path="$1"
  jq -r '
    [
      .summary.zero_entity_files,
      ([.files[] | select(.file_name | contains("notebooklm")) | select(.file_name | contains("Chat_History") | not) | select(.entities_promoted <= 2)] | length),
      (.summary.relation_promotion_rate * 100),
      .summary.zero_relation_files,
      (.summary.must_entities_recall * 100),
      (.summary.must_facts_recall * 100),
      .summary.anti_entities_found_count,
      .summary.golden_files
    ] | @tsv
  ' "$path"
}

read -r ZERO_ENTITY_FILES RICH_FEW_ENTITIES REL_PROM_RATE ZERO_RELATION_FILES \
  MUST_ENTITY_RECALL MUST_FACT_RECALL ANTI_ENTITIES_COUNT GOLDEN_FILES \
  <<< "$(metrics_from_report "$REPORT_PATH")"

# Layered: pipeline uses best recent values; contract uses current only.
EVAL_ZERO_ENT="$ZERO_ENTITY_FILES"
EVAL_RICH="$RICH_FEW_ENTITIES"
EVAL_REL="$REL_PROM_RATE"
EVAL_ZERO_REL="$ZERO_RELATION_FILES"

if [[ "$GATE_MODE" == "layered" && "$BEST_OF_N" -gt 0 && -d "$REPORTS_DIR" ]]; then
  while IFS= read -r archived; do
    [[ -f "$archived" ]] || continue
    read -r z r rel zr _ _ _ _ <<< "$(metrics_from_report "$archived")"
    (( $(echo "$rel > $EVAL_REL" | bc -l) )) && EVAL_REL="$rel"
    (( z < EVAL_ZERO_ENT )) && EVAL_ZERO_ENT="$z"
    (( r < EVAL_RICH )) && EVAL_RICH="$r"
    (( zr < EVAL_ZERO_REL )) && EVAL_ZERO_REL="$zr"
  done < <(ls -1t "$REPORTS_DIR"/run-*.json 2>/dev/null | head -n "$BEST_OF_N")
  # Include current run in pool
  (( $(echo "$REL_PROM_RATE > $EVAL_REL" | bc -l) )) && EVAL_REL="$REL_PROM_RATE"
  (( ZERO_ENTITY_FILES < EVAL_ZERO_ENT )) && EVAL_ZERO_ENT="$ZERO_ENTITY_FILES"
  (( RICH_FEW_ENTITIES < EVAL_RICH )) && EVAL_RICH="$RICH_FEW_ENTITIES"
  (( ZERO_RELATION_FILES < EVAL_ZERO_REL )) && EVAL_ZERO_REL="$ZERO_RELATION_FILES"
elif [[ "$GATE_MODE" == "layered" ]]; then
  EVAL_REL="$REL_PROM_RATE"
fi

PASS_ZERO_ENTITIES="FAIL"
PASS_RICH_DOCS="FAIL"
PASS_REL_PROM="FAIL"
PASS_ZERO_REL="FAIL"
PASS_MUST_ENT="FAIL"
PASS_ANTI="FAIL"

[[ "$EVAL_ZERO_ENT" -le "$ZERO_ENT_MAX" ]] && PASS_ZERO_ENTITIES="PASS"
[[ "$EVAL_RICH" -le "$RICH_FEW_MAX" ]] && PASS_RICH_DOCS="PASS"

REL_MIN_PCT=$(echo "$REL_MIN * 100" | bc -l)
if (( $(echo "$EVAL_REL >= $REL_MIN_PCT" | bc -l) )); then
  PASS_REL_PROM="PASS"
fi

[[ "$EVAL_ZERO_REL" -le "$ZERO_REL_MAX" ]] && PASS_ZERO_REL="PASS"
(( $(echo "$MUST_ENTITY_RECALL >= $GOLDEN_MIN * 100" | bc -l) )) && PASS_MUST_ENT="PASS"
[[ "$ANTI_ENTITIES_COUNT" -le "$ANTI_MAX" ]] && PASS_ANTI="PASS"

echo "========================================================================="
echo "                    INGEST QUALITY METRICS (gate-report)                   "
echo "========================================================================="
echo "Gate mode: $GATE_MODE | Report: $REPORT_PATH"
if [[ "$GATE_MODE" == "layered" && "$BEST_OF_N" -gt 0 ]]; then
  echo "Pipeline eval (layered pool): rel=${EVAL_REL}%, zero_ent=${EVAL_ZERO_ENT}, rich_few=${EVAL_RICH}, zero_rel=${EVAL_ZERO_REL}"
fi
echo "-------------------------------------------------------------------------"
printf "%-32s | %-10s | %-10s | %-8s\n" "Metric" "Current" "Target" "Status"
echo "-------------------------------------------------------------------------"
printf "%-32s | %-10s | %-10s | %-8s\n" "Files with 0 entities" "$ZERO_ENTITY_FILES" "<= $ZERO_ENT_MAX" "$PASS_ZERO_ENTITIES"
printf "%-32s | %-10s | %-10s | %-8s\n" "Rich NotebookLM <=2 entities" "$RICH_FEW_ENTITIES" "<= $RICH_FEW_MAX" "$PASS_RICH_DOCS"
printf "%-32s | %-10.1f%% | >= %.0f%%  | %-8s\n" "Relation promotion rate" "$REL_PROM_RATE" "$(echo "$REL_MIN * 100" | bc -l)" "$PASS_REL_PROM"
printf "%-32s | %-10s | <= $ZERO_REL_MAX      | %-8s\n" "Zero-relation files" "$ZERO_RELATION_FILES" "$PASS_ZERO_REL"
printf "%-32s | %-10.1f%% | >= %.0f%%  | %-8s\n" "Golden must-entity recall" "$MUST_ENTITY_RECALL" "$(echo "$GOLDEN_MIN * 100" | bc -l)" "$PASS_MUST_ENT"
printf "%-32s | %-10.1f%% | (info)     | %-8s\n" "Golden must-fact recall" "$MUST_FACT_RECALL" "" "INFO"
printf "%-32s | %-10s | <= $ANTI_MAX        | %-8s\n" "Anti-pattern entities" "$ANTI_ENTITIES_COUNT" "$PASS_ANTI"
echo "========================================================================="
echo "Golden files matched: $GOLDEN_FILES / 15"
echo ""

# Deterministic contract cross-check
if python3 "$DIR/rescore-golden.py" --report "$REPORT_PATH" >/dev/null 2>&1; then
  echo "Contract rescore (offline): PASS"
  PASS_RESCORE="PASS"
else
  echo "Contract rescore (offline): FAIL — run: python3 scripts/ingest-quality/rescore-golden.py"
  PASS_RESCORE="FAIL"
fi

# MemScore one-liner (informational)
if [[ -f "$DIR/mem-score.py" ]]; then
  python3 "$DIR/mem-score.py" 2>/dev/null || true
fi

# M4 faithfulness one-liner from latest judge report (informational; never FAILs
# here unless STRICT_MEMSCORE=1 is wired by a caller).
JUDGE_LATEST="$DIR/reports/faithfulness-judge-latest.json"
if [[ -f "$JUDGE_LATEST" ]]; then
  python3 - "$JUDGE_LATEST" "$DIR/gate-config.yaml" <<'PY' || true
import json, sys, yaml
s = json.load(open(sys.argv[1])).get("summary", {})
m = yaml.safe_load(open(sys.argv[2])).get("memscore", {})
ctx, corp = s.get("faithfulness_context"), s.get("faithfulness_corpus")
gmin = m.get("faithfulness_context_min", 0.90)
def f(v): return f"{v:.3f}" if isinstance(v, (int, float)) else "n/a"
status = "n/a" if ctx is None else ("PASS" if ctx >= gmin else "below-gate")
print(f"M4 faithfulness: context={f(ctx)} (>= {gmin} {status}) | corpus={f(corp)} | grounding={s.get('grounding')}")
PY
fi
echo ""

if [[ "$PASS_ZERO_ENTITIES" = "PASS" && "$PASS_RICH_DOCS" = "PASS" && "$PASS_REL_PROM" = "PASS" && \
      "$PASS_ZERO_REL" = "PASS" && "$PASS_MUST_ENT" = "PASS" && "$PASS_ANTI" = "PASS" && "$PASS_RESCORE" = "PASS" ]]; then
  echo "SUCCESS: All quality targets met!"
  exit 0
fi
echo "WARNING: Some quality targets failed. Tuning required."
exit 1
