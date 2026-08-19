#!/usr/bin/env bash
# OpenClaw operator morning brief — reads data-next artifacts and outputs a markdown summary
set -euo pipefail
shopt -s nullglob

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"

newest_file() {
    local latest=""
    local candidate

    for candidate in "$@"; do
        if [[ -z "$latest" || "$candidate" -nt "$latest" ]]; then
            latest="$candidate"
        fi
    done

    printf '%s' "$latest"
}

# F1 — Freshness-Helper: prints "Δ Xm" / "Δ Xh" / "Δ Xd" for a file's mtime, or "" if missing.
freshness() {
    local f="$1" now mt age
    [[ -f "$f" ]] || { printf ''; return 0; }
    now=$(date +%s)
    mt=$(stat -c %Y "$f" 2>/dev/null || printf '%s' "$now")
    age=$(( now - mt ))
    if (( age < 3600 )); then printf 'Δ %sm' $(( age / 60 ))
    elif (( age < 86400 )); then printf 'Δ %sh' $(( age / 3600 ))
    else printf 'Δ %sd' $(( age / 86400 )); fi
}

# Emit a freshness annotation line for a section source file (no-op if file missing).
emit_freshness() {
    local delta
    delta="$(freshness "$1")"
    [[ -n "$delta" ]] && printf '_(freshness: %s)_\n\n' "$delta" || true
}

echo "🌅 GZMO Morning Brief — $(date '+%Y-%m-%d %H:%M %Z')"
echo ""

# Ops-health
if [[ -f "$DATA/ops-health/latest.md" ]]; then
    echo "## 🏥 Ops Health"
    emit_freshness "$DATA/ops-health/latest.md"
    cat "$DATA/ops-health/latest.md"
    echo ""
fi

# Serendipity digest
serendipity_src=""
if [[ -f "$DATA/serendipity/digest-$(date +%Y-%m-%d).md" ]]; then
    serendipity_src="$DATA/serendipity/digest-$(date +%Y-%m-%d).md"
    echo "## ✨ Serendipity"
elif [[ -f "$DATA/serendipity/latest.md" ]]; then
    serendipity_src="$DATA/serendipity/latest.md"
    echo "## ✨ Serendipity (latest)"
fi
if [[ -n "$serendipity_src" ]]; then
    emit_freshness "$serendipity_src"
    cat "$serendipity_src"
    echo ""

    # F2 — Serendipity STALE-Signal: local spark report date vs today.
    SPARK_REPORT="$DATA/spark/last-spark-report.json"
    if [[ -f "$SPARK_REPORT" ]]; then
        spark_age_days="$(python3 - "$SPARK_REPORT" <<'PY' 2>/dev/null || printf '0'
import json, sys
from datetime import date, datetime
try:
    with open(sys.argv[1]) as f:
        payload = json.load(f)
except (OSError, json.JSONDecodeError):
    raise SystemExit(0)
spark_date_str = str(payload.get("date", "") or "")
if not spark_date_str:
    raise SystemExit(0)
try:
    spark_date = datetime.strptime(spark_date_str, "%Y-%m-%d").date()
except ValueError:
    raise SystemExit(0)
print(max(0, (date.today() - spark_date).days))
PY
)"
        if [[ -n "${spark_age_days:-}" && "${spark_age_days:-0}" -gt 7 ]]; then
            echo "⚠️ STALE: local spark date is ${spark_age_days}d old (CT101 runs nightly; digest may lag)."
            echo ""
        fi
    fi
fi

# Research scan
if [[ -f "$DATA/research-scan/latest.md" ]]; then
    echo "## 🔬 Research Scan"
    emit_freshness "$DATA/research-scan/latest.md"
    cat "$DATA/research-scan/latest.md"
    echo ""
fi

# Research gaps
gap_files=("$DATA/research-scan"/gaps-*.json)
if (( ${#gap_files[@]} > 0 )); then
    latest_gaps="$(newest_file "${gap_files[@]}")"
    echo "### Research Gaps"
    emit_freshness "$latest_gaps"
    if ! python3 - "$latest_gaps" <<'PY'
import json
import sys

try:
    with open(sys.argv[1]) as f:
        payload = json.load(f)
except (OSError, json.JSONDecodeError) as exc:
    print(f"(gap parse failed: {exc})")
    raise SystemExit(0)

if isinstance(payload, dict):
    gaps = payload.get("gaps")
elif isinstance(payload, list):
    gaps = payload
else:
    gaps = None

if not isinstance(gaps, list):
    print("(gap parse failed: not a list or dict)")
    raise SystemExit(0)

valid_gaps = [gap for gap in gaps if isinstance(gap, dict)]
skipped = len(gaps) - len(valid_gaps)
if skipped:
    print(f"(skipped {skipped} malformed gap item(s))")

if not valid_gaps:
    print("(no valid gaps in latest scan)")

for gap in valid_gaps:
    title = next(
        (
            value
            for key in ("title", "query", "topic")
            if (value := gap.get(key)) is not None and str(value).strip()
        ),
        "?",
    )
    print(f"- {str(title).strip()[:120]}")
PY
    then
        echo "(gap parse failed: python error)"
    fi
    echo ""
fi

# Research inbox
inbox=("$DATA/inbox/research/"*.md)
if (( ${#inbox[@]} > 0 )); then
    sorted_inbox=("${inbox[@]}")
    for ((i = 0; i < ${#sorted_inbox[@]}; i++)); do
        for ((j = i + 1; j < ${#sorted_inbox[@]}; j++)); do
            if [[ "${sorted_inbox[$j]}" -nt "${sorted_inbox[$i]}" ]]; then
                tmp="${sorted_inbox[$i]}"
                sorted_inbox[$i]="${sorted_inbox[$j]}"
                sorted_inbox[$j]="$tmp"
            fi
        done
    done

    echo "### 📥 Research Inbox"
    inbox_limit=${#sorted_inbox[@]}
    if (( inbox_limit > 10 )); then
        inbox_limit=10
    fi
    for ((i = 0; i < inbox_limit; i++)); do
        f="${sorted_inbox[$i]}"
        # F5 — robust H1 read (no `head|sed` under pipefail race).
        title="$(sed -n '1p' "$f" 2>/dev/null | sed 's/^# //' || true)"
        echo "- $title"
    done
    omitted=$((${#sorted_inbox[@]} - inbox_limit))
    if (( omitted > 0 )); then
        echo "… and $omitted more omitted"
    fi

    # F5 — Inbox duplicate-title hint across ALL inbox files.
    dup_titles=()
    for f in "${sorted_inbox[@]}"; do
        h1="$(sed -n '1p' "$f" 2>/dev/null || true)"
        dup_titles+=("$h1")
    done
    if (( ${#dup_titles[@]} > 0 )); then
        dup_count="$(printf '%s\n' "${dup_titles[@]}" | sort | uniq -d | wc -l | tr -d ' ')"
        if [[ -n "${dup_count:-}" && "${dup_count:-0}" -gt 0 ]]; then
            echo "⚠️ ${dup_count} duplicate titles across ${#sorted_inbox[@]} files"
        fi
    fi
fi
echo ""

# F3 — Research Intel (before TinyFolder)
if [[ -f "$DATA/research-intel/latest.md" ]]; then
    echo "## 🧠 Research Intel"
    emit_freshness "$DATA/research-intel/latest.md"
    ri_lines="$(wc -l < "$DATA/research-intel/latest.md" 2>/dev/null || printf '0')"
    if [[ "${ri_lines:-0}" -gt 120 ]]; then
        head -60 "$DATA/research-intel/latest.md" 2>/dev/null || true
        echo "… (truncated, ${ri_lines} lines total)"
    else
        cat "$DATA/research-intel/latest.md"
    fi
    echo ""
fi

# Tinyfolder (latest drop)
drop_files=("$DATA/tinyfolder-inbox"/drop-*.md)
if (( ${#drop_files[@]} > 0 )); then
    latest_drop="$(newest_file "${drop_files[@]}")"
    echo "## 📦 TinyFolder (latest drop)"
    emit_freshness "$latest_drop"
    cat "$latest_drop"
    echo ""
fi

# Distill queue
queue=("$DATA/distill-queue"/*.jsonl)
if (( ${#queue[@]} > 0 )); then
    echo "## 🧪 Distill Queue"
    # newest jsonl as freshness source
    newest_queue="$(newest_file "${queue[@]}")"
    emit_freshness "$newest_queue"

    # F4 — total entries across all jsonl (robust).
    total_entries="$(cat "${queue[@]}" 2>/dev/null | wc -l | tr -d ' ' || printf '0')"
    echo "Σ ${total_entries} entries across ${#queue[@]} files"

    # newest 7 files by mtime desc
    sorted_queue=("${queue[@]}")
    for ((i = 0; i < ${#sorted_queue[@]}; i++)); do
        for ((j = i + 1; j < ${#sorted_queue[@]}; j++)); do
            if [[ "${sorted_queue[$j]}" -nt "${sorted_queue[$i]}" ]]; then
                tmp="${sorted_queue[$i]}"
                sorted_queue[$i]="${sorted_queue[$j]}"
                sorted_queue[$j]="$tmp"
            fi
        done
    done
    show_limit=${#sorted_queue[@]}
    if (( show_limit > 7 )); then
        show_limit=7
    fi
    for ((i = 0; i < show_limit; i++)); do
        f="${sorted_queue[$i]}"
        lines="$(wc -l < "$f" 2>/dev/null || printf '0')"
        echo "- $(basename "$f"): $lines entries"
    done

    # F4 — drain signal: warn if no archive entry fresher than 3 days
    archive_root="$DATA/distill-queue/archive"
    fresh_count="$(find "$archive_root" -maxdepth 2 -type f -mtime -3 2>/dev/null | wc -l | tr -d ' ')"
    if (( fresh_count == 0 )); then
        echo "consumer: no recent drain (no archive entry in last 3d; queue accumulates)"
    fi
fi
echo ""

# Evolve daily log
if [[ -f "$DATA/ecosystem-evolve/daily.log" ]]; then
    echo "## 🔄 Evolve Daily"
    tail -5 "$DATA/ecosystem-evolve/daily.log" 2>/dev/null | sed 's/^/  /'
fi

echo ""

# Stigmergy Board
STIGMERGY_ROOT="${HOME}/.memory/stigmergy"
if [[ -d "$STIGMERGY_ROOT" ]]; then
    echo "## 📋 Stigmergy Board"
    for lane in pending claimed done; do
        lane_files=("$STIGMERGY_ROOT/$lane"/*.json)
        count=${#lane_files[@]}
        echo "- $lane: $count tasks"
        if [[ "$lane" == "pending" && "$count" -gt 0 ]]; then
            for f in "${lane_files[@]}"; do
                tid=$(basename "$f" .json)
                echo "  ○ $tid"
            done
        fi
        if [[ "$lane" == "done" && "$count" -gt 0 ]]; then
            sorted_done=("${lane_files[@]}")
            for ((i = 0; i < ${#sorted_done[@]}; i++)); do
                for ((j = i + 1; j < ${#sorted_done[@]}; j++)); do
                    if [[ "${sorted_done[$j]}" -nt "${sorted_done[$i]}" ]]; then
                        tmp="${sorted_done[$i]}"
                        sorted_done[$i]="${sorted_done[$j]}"
                        sorted_done[$j]="$tmp"
                    fi
                done
            done

            recent_done=""
            recent_count=${#sorted_done[@]}
            if (( recent_count > 3 )); then
                recent_count=3
            fi
            for ((i = 0; i < recent_count; i++)); do
                recent_done+="$(basename "${sorted_done[$i]}" .json), "
            done
            echo "  (last 3: ${recent_done%, })"
        fi
    done
    echo ""
fi

echo "---"
echo "Brief generated by OpenClaw operator surface"
