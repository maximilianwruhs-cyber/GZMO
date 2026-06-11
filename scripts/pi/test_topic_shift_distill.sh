#!/usr/bin/env bash
# Smoke: topic-shift config + embed endpoint + cosine distance above threshold.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/gzmo.toml}"

echo "== topic_shift config in gzmo.toml =="
ENABLED="$(python3 -c "
import tomllib, pathlib
cfg = tomllib.loads(pathlib.Path('gzmo.toml').read_text())
print(cfg.get('session_distill', {}).get('topic_shift_enabled', False))
")"
THRESH="$(python3 -c "
import tomllib, pathlib
cfg = tomllib.loads(pathlib.Path('gzmo.toml').read_text())
print(cfg.get('session_distill', {}).get('topic_shift_threshold', 0.35))
")"

if [[ "$ENABLED" != "True" ]]; then
  echo "WARN: topic_shift_enabled is not true — enable in [session_distill] for live Pi hook" >&2
else
  echo "OK: topic_shift_enabled=true threshold=$THRESH"
fi

echo "== embed endpoint + unrelated-topic distance =="
python3 <<'PY'
import json, math, pathlib, tomllib, urllib.request, sys

cfg = tomllib.loads(pathlib.Path("gzmo.toml").read_text())
embed = cfg.get("embeddings", {})
url = embed.get("url", "").rstrip("/") + "/embeddings"
model = embed.get("model", "")
threshold = float(cfg.get("session_distill", {}).get("topic_shift_threshold", 0.35))

def embed_text(text: str) -> list[float]:
    body = json.dumps({"input": text, "model": model}).encode()
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"}, method="POST")
    with urllib.request.urlopen(req, timeout=20) as resp:
        data = json.loads(resp.read())
    return data["data"][0]["embedding"]

def cosine_distance(a, b):
    dot = sum(x * y for x, y in zip(a, b))
    na = math.sqrt(sum(x * x for x in a))
    nb = math.sqrt(sum(x * x for x in b))
    if na == 0 or nb == 0:
        return 1.0
    return 1.0 - dot / (na * nb)

topic_a = "kubernetes pod scheduling affinity rules and node selectors for production clusters"
topic_b = "sourdough bread fermentation starter hydration ratios and oven steam techniques"

e1 = embed_text(topic_a)
e2 = embed_text(topic_b)
dist = cosine_distance(e1, e2)
print(f"unrelated distance: {dist:.4f} (threshold {threshold})")
if dist <= threshold:
    print("FAIL: unrelated topics should exceed threshold for shift detection", file=sys.stderr)
    sys.exit(1)
print("OK: embed path reachable and shift would trigger for unrelated topics")
PY

echo "== partial range distill (fixture) =="
FIXTURE="${ROOT}/tests/fixtures/pi_session_minimal.jsonl"
GZMO_BIN="${GZMO_BIN:-$ROOT/target/release/gzmo}"
if [[ ! -x "$GZMO_BIN" ]]; then
  GZMO_BIN="$ROOT/target/debug/gzmo"
fi
OUT="$("$GZMO_BIN" distill pi "$FIXTURE" --from-turn 1 --max-turns 1 2>&1 || true)"
echo "$OUT" | tail -3
if echo "$OUT" | grep -qE 'skipped|distilled|vault|Session'; then
  echo "OK: topic-shift distill smoke passed"
else
  echo "FAIL: unexpected range distill output" >&2
  exit 1
fi
