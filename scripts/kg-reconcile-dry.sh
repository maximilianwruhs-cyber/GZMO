#!/usr/bin/env bash
# Neo4j ontology reconcile — always dry-run. Never mutates living graph.
# Pulls a small graph sample from CT101 Neo4j when possible; else fixture.
#
#   bash scripts/kg-reconcile-dry.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/kg-reconcile"
HOST="${CT101_SSH_HOST:-ct101}"
KG_BIN="${KG_RECONCILE_BIN:-$HOME/github-clone/temp-bench/target/release/kg-reconcile}"
FIXTURE="${KG_RECONCILE_FIXTURE:-$HOME/github-clone/kg-reconcile/fixtures/sample-graph.json}"
mkdir -p "$OUT"

serve="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
serve="$(printf '%s\n' "$serve" | head -1)"
if [[ "$serve" == "active" ]]; then
  echo "[WARN] gzmo-serve active — continuing dry-run only (no apply path exists)" >&2
fi

input="$OUT/input-latest.json"
mode="fixture"

if [[ ! -x "$KG_BIN" ]] && [[ -d "$HOME/github-clone/kg-reconcile" ]]; then
  (cd "$HOME/github-clone/kg-reconcile" && CARGO_TARGET_DIR="$HOME/github-clone/temp-bench/target" cargo build --release -q) || true
fi
BIN="$KG_BIN"
[[ -x "$BIN" ]] || BIN="$(command -v kg-reconcile || true)"

exported=0
if ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'true' 2>/dev/null; then
  sample="$(ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" \
    'docker exec sidecar-neo4j cypher-shell -u neo4j --format plain "MATCH (n) RETURN labels(n)[0] AS type, coalesce(n.name,n.id,\"\") AS name LIMIT 40;" 2>/dev/null || true')"
  if [[ -n "$sample" ]]; then
    if python3 - "$sample" "$input" <<'PY'
import json, sys
from pathlib import Path
raw = sys.argv[1]
out = Path(sys.argv[2])
entities = []
for line in raw.splitlines():
    line = line.strip()
    if not line or line.lower().startswith("type"):
        continue
    parts = [p.strip().strip('"') for p in line.split(",")]
    if len(parts) >= 2 and parts[0] and parts[1]:
        entities.append({"type": parts[0], "name": parts[1], "observations": []})
if len(entities) < 3:
    raise SystemExit(1)
out.write_text(json.dumps({"entities": entities, "relations": []}, indent=2) + "\n")
print(len(entities))
PY
    then
      exported=1
      mode="ct101-sample"
    fi
  fi
fi

if [[ "$exported" -eq 0 ]]; then
  cp "$FIXTURE" "$input"
  mode="fixture"
fi

report="$OUT/report-latest.json"
ok=false
detail=""
if [[ -n "${BIN:-}" && -x "$BIN" ]]; then
  if "$BIN" run --input "$input" >"$report" 2>"$OUT/run.err"; then
    ok=true
    detail="kg-reconcile dry-run ok mode=$mode"
  else
    detail="kg-reconcile failed: $(head -c 400 "$OUT/run.err" 2>/dev/null || true)"
    python3 - "$report" "$detail" "$mode" <<'PY'
import json, sys
from pathlib import Path
from datetime import datetime, timezone
Path(sys.argv[1]).write_text(json.dumps({
  "dry_run": True,
  "ok": False,
  "error": sys.argv[2],
  "mode": sys.argv[3],
  "generated_at": datetime.now(timezone.utc).isoformat(),
}, indent=2) + "\n")
PY
  fi
else
  detail="kg-reconcile binary missing — fixture copied only"
  python3 - "$report" "$detail" "$mode" "$input" <<'PY'
import json, sys
from pathlib import Path
from datetime import datetime, timezone
inp = json.loads(Path(sys.argv[4]).read_text())
Path(sys.argv[1]).write_text(json.dumps({
  "dry_run": True,
  "ok": False,
  "error": sys.argv[2],
  "mode": sys.argv[3],
  "entity_count": len(inp.get("entities") or []),
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "advice": "build kg-reconcile: cd ~/github-clone/kg-reconcile && cargo build --release",
}, indent=2) + "\n")
PY
fi

python3 - "$OUT" "$report" "$mode" "$ok" "$detail" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
out, report_path = Path(sys.argv[1]), Path(sys.argv[2])
mode, ok_s, detail = sys.argv[3], sys.argv[4], sys.argv[5]
ok = ok_s == "true"
report = {}
if report_path.is_file():
    try:
        report = json.loads(report_path.read_text())
    except Exception as e:
        report = {"parse_error": str(e)}
inner = report.get("report") if isinstance(report.get("report"), dict) else report
payload = {
    "schema": "gzmo.kg_reconcile_dry/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "dry_run": True,
    "mode": mode,
    "detail": detail,
    "report_summary": {
        "dry_run": report.get("dry_run", True),
        "entity_fixes": inner.get("entity_fixes") if isinstance(inner, dict) else None,
        "relation_fixes": inner.get("relation_fixes") if isinstance(inner, dict) else None,
        "error": report.get("error"),
    },
    "advice": "kg_reconcile_dry only — never flip apply until beat-gate + human ack",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
(out / "latest.md").write_text(
    f"# kg-reconcile dry\n\nok={ok} mode={mode}\n\n{detail}\n",
    encoding="utf-8",
)
print(json.dumps(payload, indent=2))
sys.exit(0)
PY
