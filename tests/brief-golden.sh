#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BRIEF="$REPO_ROOT/scripts/openclaw-morning-brief.sh"

# syntax check
if ! bash -n "$BRIEF" 2>/dev/null; then
  echo "[brief-golden] FAIL: syntax error in openclaw-morning-brief.sh"; exit 1
fi

TMPOUT=$(mktemp)
TMPERR=$(mktemp)
trap 'rm -f "$TMPOUT" "$TMPERR"' EXIT

if ! bash "$BRIEF" >"$TMPOUT" 2>"$TMPERR"; then
  echo "[brief-golden] FAIL: brief exited non-zero"; exit 1
fi

if [[ -s "$TMPERR" ]]; then
  echo "[brief-golden] FAIL: stderr not empty:"; head -5 "$TMPERR"; exit 1
fi

REQUIRED_MARKERS=(
  "🏥 Ops Health"
  "✨ Serendipity"
  "📥 Research Inbox"
  "🧪 Distill Queue"
  "freshness:"
)

# Σ … entries across (regex)
if ! grep -qP 'Σ .* entries across' "$TMPOUT"; then
  echo "[brief-golden] FAIL: missing 'Σ ... entries across'"; exit 1
fi

for m in "${REQUIRED_MARKERS[@]}"; do
  if ! grep -qF "$m" "$TMPOUT"; then
    echo "[brief-golden] FAIL: missing marker '$m'"; exit 1
  fi
done

# conditional: 🧠 Research Intel only if source exists
if [[ -f "$REPO_ROOT/data-next/research-intel/latest.md" ]]; then
  if ! grep -qF "🧠 Research Intel" "$TMPOUT"; then
    echo "[brief-golden] FAIL: missing '🧠 Research Intel' (latest.md exists)"; exit 1
  fi
fi

# conditional: 🧠 Brain Feed only if source exists
if [[ -f "$REPO_ROOT/data-next/brain-feed/latest.md" ]]; then
  if ! grep -qF "🧠 Brain Feed" "$TMPOUT"; then
    echo "[brief-golden] FAIL: missing '🧠 Brain Feed' (latest.md exists)"; exit 1
  fi
fi

# negative: gap parse failed must NOT appear
if grep -qF "gap parse failed" "$TMPOUT"; then
  echo "[brief-golden] FAIL: 'gap parse failed' found in output"; exit 1
fi

echo "[brief-golden] PASS"
