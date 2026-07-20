#!/usr/bin/env bash
# Unpark Wave 4.1 demable: AOS CE path smoke — living pin + product boundary.
# Never overwrites ~/.gzmo. Does not start a second overnight writer.
#
#   bash scripts/aos-ce-smoke.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/aos-ce"
mkdir -p "$OUT"
pass=0; fail=0; hold=0
declare -a ROWS=()
row() { local s="$1" n="$2" d="$3"; ROWS+=("$s|$n|$d"); case "$s" in PASS) pass=$((pass+1));; FAIL) fail=$((fail+1));; HOLD) hold=$((hold+1));; esac; echo "[$s] $n — $d"; }

echo "=== AOS CE smoke (Unpark W4.1) ==="
CE_DOC="$ROOT/docs/AOS_CUSTOMER_EDITION.md"
if [[ -f "$CE_DOC" ]] && python3 -c "
from pathlib import Path
t=Path('$CE_DOC').read_text(encoding='utf-8').lower()
need=['sidecar-free','adr-0003','~/.gzmo','living-readiness-gate']
raise SystemExit(0 if all(x in t for x in need) else 1)
"; then
  row PASS "ce-doc" "AOS_CUSTOMER_EDITION.md boundary phrases"
else
  row FAIL "ce-doc" "CE doc missing sidecar-free / ADR-0003 / ~/.gzmo / living-readiness phrases"
fi
[[ -f "$ROOT/deploy/living-appliance/docker-compose.yml" ]] && row PASS "living-pin" "compose pin present" || row FAIL "living-pin" "missing"

bash "$ROOT/scripts/living-appliance-gate.sh" >/dev/null 2>&1 || true
if [[ -f "$DATA/living-appliance/latest.json" ]] && python3 -c "import json;d=json.load(open('$DATA/living-appliance/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "living-pin-gate" "living appliance pin ok"
else
  row FAIL "living-pin-gate" "living-appliance-gate failed"
fi

# Product boundary: stranger path must not require sidecars
bash "$ROOT/scripts/mcp-attach-check.sh" >/dev/null 2>&1 || true
if [[ -f "$DATA/mcp-attach/latest.json" ]] && python3 -c "import json;d=json.load(open('$DATA/mcp-attach/latest.json')); raise SystemExit(0 if d.get('ok') else 1)"; then
  row PASS "product-boundary" "gzmo-memory still product ~/.gzmo"
else
  row FAIL "product-boundary" "product attach broken"
fi

# Dual-writer soft
if systemctl --user is-active gzmo-serve.service >/dev/null 2>&1; then
  row FAIL "dual-writer" "workstation gzmo-serve active — ADR-0003"
else
  row PASS "dual-writer" "workstation serve inactive"
fi

# Golden-path pin: compose services + explicit product non-overwrite
COMPOSE="$ROOT/deploy/living-appliance/docker-compose.yml"
export OUT ROOT COMPOSE
python3 - <<'PY'
import json, re
from datetime import datetime, timezone
from pathlib import Path
out = Path(__import__("os").environ["OUT"])
compose = Path(__import__("os").environ["COMPOSE"])
text = compose.read_text(encoding="utf-8") if compose.is_file() else ""
# top-level service keys under services:
services = []
in_services = False
for line in text.splitlines():
    if re.match(r"^services:\s*$", line):
        in_services = True
        continue
    if in_services:
        if line and not line[0].isspace() and not line.startswith("#"):
            break
        m = re.match(r"^  ([A-Za-z0-9_.-]+):\s*$", line)
        if m:
            services.append(m.group(1))
pin = {
    "schema": "gzmo.unpark.aos_ce.golden_path/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "4.1",
    "compose": str(compose),
    "services": services,
    "overwrites_product_home": False,
    "stranger_default": False,
    "arena_required": False,
    "ok": bool(services) and "redis" in services and "qdrant" in services,
    "advice": "aos_ce_golden_path_ok — CE on C; never overwrites ~/.gzmo",
}
(out / "golden-path.json").write_text(json.dumps(pin, indent=2) + "\n")
print(json.dumps({"golden_path": str(out / "golden-path.json"), "services": services}, indent=2))
PY

if [[ -f "$OUT/golden-path.json" ]] && python3 -c "
import json
d=json.load(open('$OUT/golden-path.json'))
ok=(
  d.get('schema')=='gzmo.unpark.aos_ce.golden_path/v1'
  and d.get('overwrites_product_home') is False
  and d.get('stranger_default') is False
  and d.get('ok') is True
  and isinstance(d.get('services'), list)
  and len(d.get('services') or [])>=2
)
raise SystemExit(0 if ok else 1)
"; then
  row PASS "golden-path" "compose services pin; never overwrites ~/.gzmo"
else
  row FAIL "golden-path" "golden-path.json incomplete"
fi

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
gp_ok = checks.get("golden-path",{}).get("status")=="PASS"
advice="aos_ce_ok — golden-path pin; CE on C not stranger A" if fail_n==0 and gp_ok else (
    "aos_ce_fail" if fail_n else "aos_ce_hold")
payload={"schema":"gzmo.unpark.aos_ce/v1","generated_at":datetime.now(timezone.utc).isoformat(),
  "verdict":verdict,"ok":fail_n==0,"advice":advice,
  "counts":{"pass":pass_n,"fail":fail_n,"hold":hold_n},"wave":"4.1","checks":checks,
  "golden_path": str(out/"golden-path.json"),
  "note":"CE is on top of C; never stranger A default."}
(out/"latest.json").write_text(json.dumps(payload,indent=2)+"\n")
print(json.dumps({"verdict":verdict,"advice":advice,"pass":pass_n,"fail":fail_n,"hold":hold_n},indent=2))
raise SystemExit(0 if fail_n==0 else 1)
PY
