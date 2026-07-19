#!/usr/bin/env bash
# ZPD tutor lab spike — soft-fail weekly pedagogy job (never on GREEN metabolism).
# Dry-run against zpd-tutor catalog; topic defaults to Rust (vault hint is advisory).
#
#   bash scripts/zpd-tutor-lab.sh
#   bash scripts/zpd-tutor-lab.sh --topic Rust
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/zpd-tutor"
ZPD_ROOT="$CLONE/zpd-tutor"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$CLONE/temp-bench/target}"

TOPIC=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --topic) TOPIC="${2:-}"; shift 2 ;;
    *) shift ;;
  esac
done

mkdir -p "$OUT"
ZPD_BIN="${ZPD_BIN:-$CARGO_TARGET_DIR/release/zpd-tutor}"
if [[ ! -x "$ZPD_BIN" && -d "$ZPD_ROOT" ]]; then
  echo "[*] building zpd-tutor…"
  (cd "$ZPD_ROOT" && cargo build --release -q) || true
fi

CONCEPTS="$ZPD_ROOT/fixtures/pedagogy-concepts.json"
# Resolve topic to catalog entry (default Rust).
if [[ -z "$TOPIC" || "$TOPIC" == *"honeypot"* || "$TOPIC" == *"skill"* ]]; then
  TOPIC="Rust"
fi
if [[ -f "$CONCEPTS" ]]; then
  RESOLVED="$(TOPIC="$TOPIC" CONCEPTS="$CONCEPTS" python3 - <<'PY'
import json, os
topic = os.environ["TOPIC"]
cats = json.load(open(os.environ["CONCEPTS"], encoding="utf-8")).get("concepts") or []
names = [c.get("topic", "") for c in cats]
for n in names:
    if n.lower() == topic.lower():
        print(n)
        raise SystemExit(0)
# fuzzy contains
tl = topic.lower()
for n in names:
    if tl in n.lower() or n.lower() in tl:
        print(n)
        raise SystemExit(0)
print(names[0] if names else "Rust")
PY
)"
  TOPIC="$RESOLVED"
fi

REPORT="$OUT/session-report.json"
STATUS=0
if [[ -x "$ZPD_BIN" && -f "$CONCEPTS" ]]; then
  set +e
  (
    cd "$OUT"
    "$ZPD_BIN" session --topic "$TOPIC" --concepts "$CONCEPTS" --dry-run \
      >"$OUT/zpd.stdout" 2>"$OUT/zpd.stderr"
  )
  STATUS=$?
  set -e
  if [[ -f "$OUT/session-report.json" ]]; then
    :
  elif [[ -f session-report.json ]]; then
    mv -f session-report.json "$REPORT"
  fi
else
  echo "[!] zpd-tutor unavailable — fixture stub" >&2
  TOPIC="${TOPIC:-Rust}"
  python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
out = Path("$OUT")
topic = """$TOPIC"""
rep = {
    "schema": "zpd.session/fixture",
    "topic": topic,
    "mode": "dry-run-stub",
    "interactions": [],
    "generated_at": datetime.now(timezone.utc).isoformat(),
}
(out / "session-report.json").write_text(json.dumps(rep, indent=2) + "\n")
PY
  STATUS=0
fi

export OUT TOPIC STATUS REPORT
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
topic = os.environ["TOPIC"]
status = int(os.environ["STATUS"])
report_path = out / "session-report.json"
report = {}
if report_path.is_file():
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except Exception:
        report = {}

turns = len(report.get("interactions") or report.get("turns") or [])
payload = {
    "schema": "gzmo.zpd.lab/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": status == 0 and report_path.is_file(),
    "soft_fail": True,
    "on_green_gate": False,
    "topic": topic,
    "session_report": str(report_path) if report_path.is_file() else None,
    "exit": status,
    "turns": turns,
    "note": "Weekly lab spike — never scheduled on GREEN metabolism; skill patches are human-promoted.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# ZPD tutor lab",
            "",
            f"Topic: **{topic}**",
            f"ok={payload['ok']} · turns={turns}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({k: payload[k] for k in ("ok", "topic", "turns", "soft_fail", "session_report")}, indent=2))
PY
exit 0
