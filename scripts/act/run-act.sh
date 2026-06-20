#!/usr/bin/env bash
# Run act in background with log polling (Jules local-action-verification pattern).
# Usage: ./run-act.sh "<act arguments>"
# Example: ./run-act.sh "push -j build"
set -euo pipefail

ACT_ARGS="${1:-}"
LOG_FILE="${ACT_LOG_FILE:-act_output.log}"
TIMEOUT="${ACT_TIMEOUT:-600}"
POLL_INTERVAL="${ACT_POLL:-10}"

if [[ -z "$ACT_ARGS" ]]; then
  echo "Usage: $0 \"<act arguments>\""
  exit 1
fi

if ! docker info >/dev/null 2>&1; then
  echo "Docker is not running."
  exit 1
fi

if ! command -v act &>/dev/null; then
  echo "act not installed — run scripts/act/install-act.sh first."
  exit 1
fi

echo "Starting: act ${ACT_ARGS}"
echo "Log: ${LOG_FILE} | timeout: ${TIMEOUT}s | poll: ${POLL_INTERVAL}s"

if echo "$ACT_ARGS" | grep -q -- '-P '; then
  act ${ACT_ARGS} >"$LOG_FILE" 2>&1 &
else
  act ${ACT_ARGS} -P ubuntu-latest=catthehacker/ubuntu:act-latest >"$LOG_FILE" 2>&1 &
fi

ACT_PID=$!
ELAPSED=0

while kill -0 "$ACT_PID" 2>/dev/null; do
  if [[ $ELAPSED -ge $TIMEOUT ]]; then
    echo "Timeout (${TIMEOUT}s) — killing act..."
    kill "$ACT_PID" 2>/dev/null || true
    wait "$ACT_PID" 2>/dev/null || true
    cat "$LOG_FILE" 2>/dev/null || true
    exit 1
  fi
  sleep "$POLL_INTERVAL"
  ELAPSED=$((ELAPSED + POLL_INTERVAL))
  echo "Running... (${ELAPSED}s/${TIMEOUT}s)"
  tail -n 5 "$LOG_FILE" 2>/dev/null || true
done

wait "$ACT_PID"
EXIT_CODE=$?

echo "--- Full log ---"
cat "$LOG_FILE"
echo "--- End log ---"

if [[ $EXIT_CODE -eq 0 ]]; then
  echo "Local GitHub Actions passed."
else
  echo "Local GitHub Actions failed (exit ${EXIT_CODE})."
fi
exit "$EXIT_CODE"
