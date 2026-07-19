#!/usr/bin/env bash
# Thin local concept-gate webhook stub (OKForge merge advice later).
#
# CLI one-shot:
#   bash scripts/concept-gate-webhook.sh
#   echo '{"intent":"wiki_push"}' | bash scripts/concept-gate-webhook.sh --stdin
#
# HTTP (soft local only):
#   bash scripts/concept-gate-webhook.sh --serve   # :8766 POST /gate
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/concept-gate"
MODE="once"
for a in "$@"; do
  case "$a" in
    --serve) MODE="serve" ;;
    --stdin) MODE="stdin" ;;
  esac
done
mkdir -p "$OUT"
export ROOT DATA OUT

run_gate() {
  local intent="${1:-wiki_push}"
  local started
  started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  set +e
  bash "$ROOT/scripts/concept-review-gate.sh" >"$OUT/webhook-gate-stdout.txt" 2>"$OUT/webhook-gate-stderr.txt"
  local ec=$?
  set -e
  python3 - <<PY
import json, os
from pathlib import Path
from datetime import datetime, timezone

out = Path(os.environ["OUT"])
gate = {}
try:
    gate = json.loads((out / "latest.json").read_text(encoding="utf-8"))
except Exception:
    gate = {"verdict": "HOLD", "reason": "gate artifact missing"}

verdict = gate.get("verdict") or ("PASS" if $ec == 0 else "HOLD")
advice = "merge_ok" if verdict == "PASS" else "hold_no_merge"
payload = {
    "schema": "gzmo.concept-gate.webhook/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "intent": """$intent""",
    "verdict": verdict,
    "advice": advice,
    "gate_exit": $ec,
    "pass": gate.get("pass"),
    "hold": gate.get("hold"),
    "checked": gate.get("checked"),
    "next": (
        "bash scripts/wiki-push-gated.sh"
        if verdict == "PASS"
        else "fix vault evidence or GZMO_CONCEPT_GATE=0 (operator bypass)"
    ),
    "note": "Stub only — no OKForge PR merge. Serve satellite already soft-holds on HOLD.",
}
(out / "webhook-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps(payload, indent=2))
PY
  return "$ec"
}

if [[ "$MODE" == "serve" ]]; then
  PORT="${CONCEPT_GATE_WEBHOOK_PORT:-8766}"
  echo "[*] concept-gate webhook on http://127.0.0.1:${PORT}/gate (POST JSON {\"intent\":\"wiki_push\"})"
  exec python3 - <<PY
import json, os, subprocess
from http.server import BaseHTTPRequestHandler, HTTPServer

root = os.environ["ROOT"]
out = os.environ["OUT"]

class H(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        print("[webhook]", fmt % args)

    def _read_json(self):
        n = int(self.headers.get("Content-Length") or 0)
        raw = self.rfile.read(n) if n else b"{}"
        try:
            return json.loads(raw.decode() or "{}")
        except Exception:
            return {}

    def do_GET(self):
        if self.path.rstrip("/") in ("/gate", "/health"):
            path = os.path.join(out, "webhook-latest.json")
            if os.path.isfile(path):
                body = open(path, "rb").read()
            else:
                body = b'{"ok":true,"hint":"POST /gate to run concept-gate"}'
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        self.send_response(404)
        self.end_headers()

    def do_POST(self):
        if self.path.rstrip("/") != "/gate":
            self.send_response(404)
            self.end_headers()
            return
        body = self._read_json()
        intent = body.get("intent") or "wiki_push"
        env = os.environ.copy()
        proc = subprocess.run(
            ["bash", os.path.join(root, "scripts", "concept-gate-webhook.sh")],
            cwd=root,
            env=env,
            capture_output=True,
            text=True,
        )
        # Re-stamp intent on artifact
        try:
            p = os.path.join(out, "webhook-latest.json")
            data = json.loads(open(p, encoding="utf-8").read())
            data["intent"] = intent
            open(p, "w", encoding="utf-8").write(json.dumps(data, indent=2) + "\n")
            resp = json.dumps(data, indent=2).encode()
        except Exception as e:
            resp = json.dumps({"verdict": "HOLD", "error": str(e)}).encode()
        code = 200 if proc.returncode == 0 else 409
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(resp)))
        self.end_headers()
        self.wfile.write(resp)

HTTPServer(("127.0.0.1", int("$PORT")), H).serve_forever()
PY
fi

if [[ "$MODE" == "stdin" ]]; then
  intent="$(python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("intent") or "wiki_push")')"
  run_gate "$intent"
  exit $?
fi

run_gate "wiki_push"
exit $?
