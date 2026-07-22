#!/usr/bin/env bash
# One-shot living promote + embed on CT101 (no gzmo serve, no overnight wait).
# Fills scheduler-runs/latest-{promote,embed}.json that organ-trace expects.
# Never starts gzmo-serve. Refuses if workstation serve is active.
#
#   bash scripts/living-promote-embed-oneshot.sh
#   LIVING_PROMOTE_LIMIT=500 bash scripts/living-promote-embed-oneshot.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
REMOTE_DATA="${KEEP_QUALITY_DATA_DIR:-/opt/gzmo/data}"
GZMO_BIN_REMOTE="${GZMO_BIN_REMOTE:-/opt/gzmo/current/target/release/gzmo}"
PROMOTE_LIMIT="${LIVING_PROMOTE_LIMIT:-}"
EMBED_LIMIT="${LIVING_EMBED_LIMIT:-2000}"

SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  echo "REFUSE: gzmo-serve active on workstation (dual-writer)" >&2
  exit 1
fi

echo "=== living promote+embed oneshot → $HOST ==="

ssh -o ConnectTimeout=15 -o BatchMode=yes "$HOST" \
  "REMOTE_DATA='$REMOTE_DATA' GZMO_BIN_REMOTE='$GZMO_BIN_REMOTE' PROMOTE_LIMIT='$PROMOTE_LIMIT' EMBED_LIMIT='$EMBED_LIMIT' bash -s" <<'REMOTE'
set -euo pipefail
cd /opt/gzmo
export GZMO_CONFIG="${GZMO_CONFIG:-/opt/gzmo/gzmo.toml}"
BIN="${GZMO_BIN_REMOTE}"
RUNS="${REMOTE_DATA}/scheduler-runs"
mkdir -p "$RUNS"

write_receipt() {
  local job="$1" ok="$2" started="$3" finished="$4" err="${5:-}"
  local night
  night="$(date -u +%Y-%m-%d)"
  local stamp
  stamp="$(date -u +%Y%m%dT%H%M%SZ)"
  local path="$RUNS/${job}-${stamp}.json"
  if [[ -n "$err" ]]; then
    python3 - "$path" "$job" "$ok" "$started" "$finished" "$night" "$err" <<'PY'
import json, sys
path, job, ok, started, finished, night, err = sys.argv[1:]
payload = {
  "job": job,
  "script": "oneshot",
  "args": [],
  "started": started,
  "finished": finished,
  "ok": ok == "1",
  "error": err or None,
  "runner": "oneshot",
  "night_id": night,
  "note": "living-promote-embed-oneshot — daemon path has no serve triad receipts",
}
open(path, "w", encoding="utf-8").write(json.dumps(payload, indent=2) + "\n")
print(path)
PY
  else
    python3 - "$path" "$job" "$ok" "$started" "$finished" "$night" <<'PY'
import json, sys
path, job, ok, started, finished, night = sys.argv[1:]
payload = {
  "job": job,
  "script": "oneshot",
  "args": [],
  "started": started,
  "finished": finished,
  "ok": ok == "1",
  "error": None,
  "runner": "oneshot",
  "night_id": night,
  "note": "living-promote-embed-oneshot — daemon path has no serve triad receipts",
}
open(path, "w", encoding="utf-8").write(json.dumps(payload, indent=2) + "\n")
print(path)
PY
  fi
  cp -f "$path" "$RUNS/latest-${job}.json"
  cp -f "$path" "$RUNS/latest.json"
}

run_job() {
  local job="$1"
  shift
  local started finished rc=0
  started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  set +e
  "$@"
  rc=$?
  set -e
  finished="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  if [[ $rc -eq 0 ]]; then
    write_receipt "$job" 1 "$started" "$finished"
    echo "[OK] $job"
  else
    write_receipt "$job" 0 "$started" "$finished" "exit_$rc"
    echo "[FAIL] $job exit=$rc" >&2
    return "$rc"
  fi
}

PROMOTE_ARGS=(memory promote)
[[ -n "${PROMOTE_LIMIT}" ]] && PROMOTE_ARGS+=("${PROMOTE_LIMIT}")
EMBED_ARGS=(memory embed)
[[ -n "${EMBED_LIMIT}" ]] && EMBED_ARGS+=("${EMBED_LIMIT}")

run_job promote "$BIN" "${PROMOTE_ARGS[@]}"
run_job embed "$BIN" "${EMBED_ARGS[@]}"

echo "=== receipts ==="
ls -la "$RUNS"/latest-promote.json "$RUNS"/latest-embed.json
REMOTE

echo "[OK] living promote+embed oneshot complete"
