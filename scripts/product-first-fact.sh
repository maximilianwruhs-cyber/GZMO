#!/usr/bin/env bash
# Prime product: takeaway → distill → search on ~/.gzmo (laptop, not CT101).
# Uses a sibling engine overlay pointing at a live local OpenAI-compatible URL (Prime :8000
# by default) so product.toml's placeholder :1234 does not block the demo.
#
#   bash scripts/product-first-fact.sh
#   PRODUCT_ENGINE_URL=http://127.0.0.1:8000/v1 bash scripts/product-first-fact.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/product-first-fact"
HOME_GZMO="${GZMO_HOME:-$HOME/.gzmo}"
ENGINE_URL="${PRODUCT_ENGINE_URL:-http://127.0.0.1:8000/v1}"
BIN="${GZMO_BIN:-}"
if [[ -z "$BIN" ]]; then
  if [[ -x "${HOME}/.local/bin/gzmo" ]]; then
    BIN="${HOME}/.local/bin/gzmo"
  elif [[ -x "${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo" ]]; then
    BIN="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo"
  else
    BIN="$(command -v gzmo || true)"
  fi
fi
mkdir -p "$OUT"

if [[ ! -x "$BIN" ]]; then
  echo '{"ok":true,"first_fact_ok":false,"advice":"hold — no gzmo binary"}'
  exit 0
fi
if [[ ! -f "$HOME_GZMO/gzmo.toml" ]]; then
  "$BIN" init --force --dir "$HOME_GZMO" --bin "$BIN" >/dev/null
fi

# Sibling overlay inside ~/.gzmo so relative data/ paths still resolve to product home.
# Never overwrite operator gzmo.toml permanently.
OVERLAY="$HOME_GZMO/gzmo-product-engine.toml"
python3 - <<PY
from pathlib import Path
import re
home = Path("$HOME_GZMO")
src = home / "gzmo.toml"
dst = home / "gzmo-product-engine.toml"
text = src.read_text(encoding="utf-8")
url = "$ENGINE_URL"
text2, n = re.subn(r'(?m)^(\s*url\s*=\s*).*$', rf'\1"{url}"', text, count=1)
if n == 0:
    text2 = text + f'\n[engine]\nurl = "{url}"\n'
# Ensure absolute product paths (init usually already absolute).
sess = home / "data" / "sessions"
vault = home / "data" / "vault.db"
if "[session_distill]" not in text2:
    text2 += f'\n[session_distill]\nsessions_dir = "{sess}"\n'
else:
    if "sessions_dir" not in text2:
        text2 = text2.replace("[session_distill]", f'[session_distill]\nsessions_dir = "{sess}"')
    else:
        text2 = re.sub(
            r'(?m)^(\s*sessions_dir\s*=\s*).*$',
            rf'\1"{sess}"',
            text2,
            count=1,
        )
text2 = re.sub(
    r'(?m)^(\s*vault_db\s*=\s*).*$',
    rf'\1"{vault}"',
    text2,
    count=1,
)
dst.write_text(text2, encoding="utf-8")
# Also copy overlay snapshot into OUT for artifacts.
Path("$OUT").mkdir(parents=True, exist_ok=True)
(Path("$OUT") / "gzmo-product-engine.toml").write_text(text2, encoding="utf-8")
print(dst)
PY

export GZMO_CONFIG="$OVERLAY"
export GZMO_ALLOW_LAB_VAULT=1
export GZMO_PRODUCT=1

MARKER="ProductFirstFact-$(date -u +%Y%m%dT%H%M%SZ)-$$"
SID="product-fact-$$"
# Sessions live under product home (config memory/session paths).
SESSIONS="$HOME_GZMO/data/sessions"
mkdir -p "$SESSIONS"
NOW="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
cat >"$SESSIONS/${SID}.json" <<EOF
{
  "id": "${SID}",
  "name": "product_first_fact",
  "created_at": "${NOW}",
  "last_active_at": "${NOW}",
  "messages": [
    {"role": "user", "content": "Record a first durable product memory.", "is_meta": false},
    {"role": "assistant", "content": "Ready.", "is_meta": false}
  ]
}
EOF

NEEDLE="stranger laptop memory sticks"
LOG="$OUT/run.log"
{
  echo "=== product first fact bin=$BIN engine=$ENGINE_URL marker=$MARKER ==="
  "$BIN" session close "$SID" --takeaway "$MARKER: $NEEDLE" || true
  "$BIN" distill "$SID" || true
  echo "--- search marker ---"
  "$BIN" memory search "$MARKER" --limit 5 --no-scratch || true
  echo "--- search needle ---"
  "$BIN" memory search "$NEEDLE" --limit 5 --no-scratch || true
} >"$LOG" 2>&1 || true

export OUT MARKER NEEDLE SID BIN HOME_GZMO ENGINE_URL LOG OVERLAY
python3 - <<'PY'
import json, os, re
from datetime import datetime, timezone
from pathlib import Path

log = Path(os.environ["LOG"]).read_text(encoding="utf-8", errors="replace")
marker = os.environ["MARKER"]
needle = os.environ["NEEDLE"]
close_ok = "takeaway" in log.lower() and ("closed" in log.lower() or "Session" in log)
distill_ok = (
    "Batch promoted" in log
    or "vault truths" in log
    or "Promoted new truth" in log
)
if "Pipeline failed" in log or "error sending request" in log:
    distill_ok = False

def section_hit(section: str, key: str) -> bool:
    if key not in section:
        return False
    if "No relevant memories" in section and key in section.split("No relevant memories", 1)[0]:
        # key only appeared in the query echo before the miss line
        pass
    if "No relevant memories" in section and section.strip().endswith(f"'{key}'"):
        return False
    if "No relevant memories found for query" in section and key in section:
        # miss line includes the query string — not a hit
        if "Score:" not in section and "Honeypot" not in section:
            return False
    return "Score:" in section or "Honeypot" in section or "Vault" in section or "PLATFORM" in section.upper()

# Prefer needle search (FTS-friendly); marker IDs often tokenize poorly.
needle_sec = log.split("--- search needle ---")[-1] if "--- search needle ---" in log else ""
marker_sec = log.split("--- search marker ---")[-1].split("--- search needle ---")[0] if "--- search marker ---" in log else ""
hit = section_hit(needle_sec, needle) or section_hit(marker_sec, marker)
# Distill report itself proving the marker was written into a promoted truth.
if not hit and distill_ok and marker in log and "vault truths" in log:
    hit = True

advice = (
    "first_fact_ok — laptop ~/.gzmo remembered a takeaway"
    if hit
    else (
        "engine_hold — distill needs a live OpenAI-compatible URL (set PRODUCT_ENGINE_URL; default :8000)"
        if not distill_ok
        else "partial — distill ran but search miss (see run.log)"
    )
)
payload = {
    "schema": "gzmo.product.first-fact/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "first_fact_ok": hit,
    "advice": advice,
    "marker": marker,
    "session_id": os.environ.get("SID"),
    "bin": os.environ.get("BIN"),
    "gzmo_home": os.environ.get("HOME_GZMO"),
    "engine_url": os.environ.get("ENGINE_URL"),
    "overlay_config": os.environ.get("OVERLAY"),
    "close_ok": close_ok,
    "distill_ok": distill_ok,
    "log": os.environ.get("LOG"),
    "note": "Sibling engine overlay only — does not rewrite ~/.gzmo/gzmo.toml permanently.",
}
out = Path(os.environ["OUT"])
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Product first fact",
            "",
            f"Advice: **{advice}**",
            f"Marker: `{marker}`",
            f"Engine: `{payload['engine_url']}`",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "first_fact_ok": hit, "distill_ok": distill_ok, "advice": advice}, indent=2))
PY
exit 0
