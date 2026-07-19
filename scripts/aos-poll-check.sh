#!/usr/bin/env bash
# Unpark Wave 1.4: read-only AOS / intelligence poll against living status (no Arena required).
#   bash scripts/aos-poll-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/aos-poll"
HOST="${CT101_SSH_HOST:-ct101}"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== AOS poll check (Unpark W1.4) ==="
row PASS "scope" "read-only living status — Arena not required"

# Prefer local living-readiness / ct101 probe artifacts; else soft SSH status
if [[ -f "$DATA/living-readiness/latest.json" ]]; then
  v="$(python3 -c "import json;print(json.load(open('$DATA/living-readiness/latest.json')).get('verdict',''))")"
  [[ "$v" == "GREEN" ]] && row PASS "living-readiness" "GREEN" || row HOLD "living-readiness" "verdict=$v"
else
  row HOLD "living-readiness" "run living-readiness-gate.sh first"
fi

set +e
ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'systemctl is-active gzmo-daemon; docker ps --format "{{.Names}}" | grep -c sidecar || true' >/tmp/aos-poll-ssh.txt 2>/dev/null
ssh_rc=$?
set -e
if [[ "$ssh_rc" -eq 0 ]]; then
  row PASS "ssh" "$HOST reachable"
  daemon="$(head -1 /tmp/aos-poll-ssh.txt 2>/dev/null || true)"
  [[ "$daemon" == "active" ]] && row PASS "daemon" "gzmo-daemon active" || row HOLD "daemon" "daemon=$daemon"
else
  row HOLD "ssh" "CT101 unreachable — poll soft"
fi

# Snapshot for dashboard consumers
python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
snap = {
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "sources": ["living-readiness", "ct101 systemd", "sidecar count"],
  "arena_required": False,
  "wave": "1.4",
}
Path("$OUT/snapshot.json").write_text(json.dumps(snap, indent=2) + "\n")
PY
row PASS "snapshot" "$OUT/snapshot.json"

ROWS_TSV="$(printf '%s\n' "${ROWS[@]}")"
export OUT pass fail hold ROWS_TSV
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path
out=Path(os.environ["OUT"]); checks={}
for line in os.environ.get("ROWS_TSV","").splitlines():
    if not line.strip(): continue
    st,n,d=line.split("|",2); checks[n]={"status":st,"detail":d}
fail_n=int(os.environ["fail"]); hold_n=int(os.environ["hold"]); pass_n=int(os.environ["pass"])
verdict="GREEN" if fail_n==0 else "RED"
advice="aos_poll_ok" if fail_n==0 and hold_n==0 else ("aos_poll_hold" if fail_n==0 else "aos_poll_fail")
payload={"schema":"gzmo.unpark.aos_poll/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"1.4","checks":checks}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
