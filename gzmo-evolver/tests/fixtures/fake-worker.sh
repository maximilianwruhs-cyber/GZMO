#!/bin/bash
# Deterministic fake OMP for hermetic worker tests.
# Records env *names* (not values) and argv; emits source-shaped v18 JSONL.
set -euo pipefail

if [[ "${1:-}" == "--version" ]]; then
  echo "18.0.11"
  exit 0
fi

OUT_BASE="${HOME:-/tmp}"
# Prefer sibling of HOME (candidate output dir) when present.
if [[ -d "$(dirname "$OUT_BASE")" ]]; then
  REC_DIR="$(dirname "$OUT_BASE")"
else
  REC_DIR="$OUT_BASE"
fi
REC="${REC_DIR}/omp-invocation.txt"
{
  printf 'ARGS:'
  printf '%q ' "$@"
  printf '\n'
  # Names only — never values (credential regression surface).
  env | cut -d= -f1 | sort
} >"$REC"

# Optional workspace mutation when --cwd is provided.
CWD=""
prev=""
for a in "$@"; do
  if [[ "$prev" == "--cwd" ]]; then
    CWD="$a"
  fi
  prev="$a"
done

if [[ -n "$CWD" && -d "$CWD/.git" ]]; then
  cd "$CWD"
  echo "fake-worker-change" >>README 2>/dev/null || echo "fake-worker-change" >NEW_FILE
  if command -v git >/dev/null 2>&1; then
    git add -A >/dev/null 2>&1 || true
    git -c user.email=fake@worker -c user.name=fake commit -m "fake-worker" >/dev/null 2>&1 || true
  fi
fi

# Official OMP v18 --mode json shapes (session v3, paired tools, usage, agent_end).
cat <<'JSONL'
{"type":"session","version":3,"id":"fake-session"}
{"type":"tool_execution_start","toolCallId":"tool-1","toolName":"bash","args":{}}
{"type":"tool_execution_end","toolCallId":"tool-1","toolName":"bash","result":"ok"}
{"type":"message_end","message":{"role":"assistant","stopReason":"stop","usage":{"input":10,"output":5,"cacheRead":1,"cacheWrite":2,"totalTokens":18}}}
{"type":"agent_end","messages":[]}
JSONL
