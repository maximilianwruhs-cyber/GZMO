#!/usr/bin/env bash
# Gap-filling research scan — read-only against living CT101, write local inbox.
# Does NOT run overnight metabolism and does NOT start gzmo-serve.
# Optional: RESEARCH_SCAN_APPLY_TINYFOLDER=1 copies notes into CT101 inbox.
#
#   bash scripts/research-scan.sh
#   RESEARCH_SCAN_TOP_N=3 bash scripts/research-scan.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/research-scan"
INBOX="$DATA/inbox/research"
HOST="${CT101_SSH_HOST:-ct101}"
TOP_N="${RESEARCH_SCAN_TOP_N:-3}"
PRIME_URL="${PRIME_CHAT_URL:-http://127.0.0.1:8000/v1/chat/completions}"
# Base URL for /v1/models (works whether PRIME_URL is .../v1 or .../v1/chat/completions)
PRIME_BASE_URL="${PRIME_URL%/chat/completions}"
# Auto-detect loaded Prime model (survives model swaps); explicit PRIME_MODEL wins
if [[ -z "${PRIME_MODEL:-}" ]]; then
  PRIME_MODEL="$(curl -sfL --max-time 5 "${PRIME_BASE_URL}/models" 2>/dev/null \
    | python3 -c 'import json,sys
try:
    d=json.load(sys.stdin)
    m=d["data"][0]["id"]
    # llama.cpp may return the GGUF path; keep the basename as model id
    print(m.rsplit("/", 1)[-1] if ".gguf" in m else m)
except Exception:
    print("")' 2>/dev/null || true)"
fi
[[ -n "$PRIME_MODEL" ]] || PRIME_MODEL="qwen3.6-35b-mtp" # last-resort legacy default
APPLY="${RESEARCH_SCAN_APPLY_TINYFOLDER:-0}"
mkdir -p "$OUT" "$INBOX"

serve="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
serve="$(printf '%s\n' "$serve" | head -1)"
if [[ "$serve" == "active" && "$APPLY" == "1" ]]; then
  echo "[!] refuse APPLY while gzmo-serve active (dual-writer)" >&2
  APPLY=0
fi

# Pull recent living context (read-only)
dreams_snip=""
spark_snip=""
if ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" 'true' 2>/dev/null; then
  dreams_snip="$(ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" \
    'tail -c 12000 /opt/gzmo/DREAMS.md 2>/dev/null || true')"
  spark_snip="$(ssh -o ConnectTimeout=8 -o BatchMode=yes "$HOST" \
    'ls -1t /opt/gzmo/data/spark/*report*.json /opt/gzmo/data/spark/latest*.json 2>/dev/null | head -1 | xargs -r head -c 8000 || true')"
fi

# Local lab fallbacks
if [[ -z "$dreams_snip" && -f "$ROOT/DREAMS.md" ]]; then
  dreams_snip="$(tail -c 12000 "$ROOT/DREAMS.md" 2>/dev/null || true)"
fi
if [[ -z "$spark_snip" && -f "$DATA/spark/latest-card.md" ]]; then
  spark_snip="$(head -c 8000 "$DATA/spark/latest-card.md" 2>/dev/null || true)"
fi

stamp="$(date -u +%Y%m%dT%H%M%SZ)"
context_file="$OUT/context-$stamp.txt"
{
  echo "=== DREAMS snip ==="
  echo "$dreams_snip"
  echo
  echo "=== SPARK snip ==="
  echo "$spark_snip"
} >"$context_file"

gaps_file="$OUT/gaps-$stamp.json"
# Prefer local Prime; fall back to heuristic keyword extraction
python3 - "$context_file" "$gaps_file" "$TOP_N" "$PRIME_URL" "$PRIME_MODEL" <<'PY'
import json, os, re, sys, urllib.request
from pathlib import Path

ctx_path, gaps_path, top_n, url, model = sys.argv[1:6]
top_n = int(top_n)
text = Path(ctx_path).read_text(encoding="utf-8", errors="replace")

def heuristic(t: str, n: int):
    # Pull TitleCase / backtick tokens and frequent nouns-ish words
    toks = re.findall(r"`([A-Za-z][A-Za-z0-9_./-]{2,})`|\\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\\b", t)
    flat = []
    for a, b in toks:
        flat.append(a or b)
    stop = {"SYSTEM", "STATE", "POLICY", "TOOL", "PROJECT", "DECISION", "CONCEPT", "Lesson", "True", "False"}
    counts = {}
    for w in flat:
        if w in stop:
            continue
        counts[w] = counts.get(w, 0) + 1
    ranked = sorted(counts.items(), key=lambda x: (-x[1], x[0]))[:n]
    return [
        {
            "topic": k,
            "score": min(1.0, 0.5 + 0.1 * v),
            "why": f"recurring token frequency={v} in dream/spark snip",
            "query": f"{k} GZMO architecture implications airgap living memory",
        }
        for k, v in ranked
    ] or [
        {
            "topic": "Brain Feed nutrient depth",
            "score": 0.7,
            "why": "fallback — thin dream/spark snip",
            "query": "how to deepen Felt Use recall>=3 without memory gym",
        }
    ]

gaps = []
prompt = (
    "From the GZMO dream/spark snip, list the top knowledge gaps worth short research. "
    f"Return ONLY JSON array of up to {top_n} objects with keys topic,score,why,query. "
    "Prefer airgap living / Brain Feed / metabolism uniqueness. No markdown.\n\n"
    + text[:9000]
)
payload = {
    "model": model,
    "messages": [{"role": "user", "content": prompt}],
    "max_tokens": 1024,
    "temperature": 0.2,
    # Qwen3.x reasoning mode burns the token budget on hidden thinking → empty content;
    # disable per-request (llama-server ignores unknown fields on non-reasoning models)
    "chat_template_kwargs": {"enable_thinking": False},
}
try:
    req = urllib.request.Request(
        url,
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=90) as r:
        body = json.loads(r.read().decode())
    content = body["choices"][0]["message"]["content"]
    # strip fences if any
    content = re.sub(r"^```(?:json)?|```$", "", content.strip(), flags=re.M).strip()
    m = re.search(r"\[.*\]", content, flags=re.S)
    if m:
        content = m.group(0)
    gaps = json.loads(content)
    if not isinstance(gaps, list):
        gaps = []
except Exception as e:
    gaps = heuristic(text, top_n)
    gaps_meta_err = str(e)
else:
    gaps_meta_err = None

gaps = gaps[:top_n]
Path(gaps_path).write_text(
    json.dumps({"gaps": gaps, "llm_error": gaps_meta_err, "source": "research-scan"}, indent=2) + "\n",
    encoding="utf-8",
)
print(json.dumps({"gap_count": len(gaps), "llm_error": gaps_meta_err}, indent=2))
PY

# Emit inbox notes (local)
mapfile -t NOTES < <(python3 - "$gaps_file" "$INBOX" "$stamp" <<'PY'
import json, sys
from pathlib import Path
gaps = json.loads(Path(sys.argv[1]).read_text())["gaps"]
inbox = Path(sys.argv[2])
stamp = sys.argv[3]
paths = []
for i, g in enumerate(gaps, 1):
    topic = str(g.get("topic") or f"gap-{i}").strip()
    safe = "".join(c if c.isalnum() or c in "-_" else "-" for c in topic)[:48].strip("-") or f"gap-{i}"
    p = inbox / f"{stamp}-{i}-{safe}.md"
    body = (
        f"# Research gap: {topic}\n\n"
        f"- score: {g.get('score')}\n"
        f"- why: {g.get('why')}\n"
        f"- query: {g.get('query')}\n\n"
        f"## Operator note\n"
        f"Generated by research-scan. Review, then optionally drop into CT101 tinyFolder inbox "
        f"for overnight enqueue (never auto-apply while dual-writer risk).\n"
    )
    p.write_text(body, encoding="utf-8")
    paths.append(str(p))
print("\n".join(paths))
PY
)

applied_file="$OUT/applied-$stamp.txt"
: >"$applied_file"
if [[ "$APPLY" == "1" && ${#NOTES[@]} -gt 0 ]]; then
  remote_inbox="/opt/gzmo/data/inbox"
  for note in "${NOTES[@]}"; do
    [[ -n "$note" ]] || continue
    base="$(basename "$note")"
    if scp -o ConnectTimeout=8 -o BatchMode=yes "$note" "$HOST:$remote_inbox/$base" 2>/dev/null; then
      printf '%s\n' "$base" >>"$applied_file"
    fi
  done
fi

notes_file="$OUT/notes-$stamp.txt"
printf '%s\n' "${NOTES[@]}" >"$notes_file"

python3 - "$OUT" "$gaps_file" "$APPLY" "$notes_file" "$applied_file" <<'PY'
import json, sys
from datetime import datetime, timezone
from pathlib import Path
out = Path(sys.argv[1])
gaps_file = Path(sys.argv[2])
apply = sys.argv[3] == "1"
notes = [ln for ln in Path(sys.argv[4]).read_text(encoding="utf-8").splitlines() if ln.strip()]
applied = [ln for ln in Path(sys.argv[5]).read_text(encoding="utf-8").splitlines() if ln.strip()]
gaps = json.loads(gaps_file.read_text())
payload = {
    "schema": "gzmo.research_scan/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": True,
    "gap_count": len(gaps.get("gaps") or []),
    "notes": notes,
    "apply_tinyfolder": apply,
    "applied": applied,
    "llm_error": gaps.get("llm_error"),
    "advice": (
        "research_scan_ok — review data-next/inbox/research; apply via tinyFolder only after ack"
        if notes
        else "research_scan_empty — no gaps emitted"
    ),
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
(out / "latest.md").write_text(
    f"# research-scan\n\n{payload['advice']}\n\nnotes: {len(notes)} applied: {len(applied)}\n",
    encoding="utf-8",
)
print(json.dumps(payload, indent=2))
PY
