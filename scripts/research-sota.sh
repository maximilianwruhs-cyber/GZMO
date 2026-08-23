#!/usr/bin/env bash
# GZMO research-sota — recursive nightly SOTA / Deep-Research pipeline (no dual-writer, no Qdrant).
# Stage 1 "Stack Lens": local Prime analyzes the live GZMO stack (docs/ADRs/organ-trace)
#   and derives research queries dynamically (NO static keywords).
# Stage 2 "Deep SOTA": fetch arXiv/GitHub/HF, sqlite-dedupe, then Prime "SOTA Synthesizer":
#   (a) group findings by functional Baugruppe, (b) assign TRL (1-9), (c) name the
#   conventional baseline, (d) ONE concrete GZMO integration lever. Strict:
#   generic AI news → benefit=false.
# Output: data-next/research-sota/{latest.md,latest.json} + timestamped archive.
# Top findings -> bin/openclaw-takeaway.sh (Brain Feed, CT101 metabolizes overnight).
#
#   bash scripts/research-sota.sh
#   RESEARCH_SOTA_MAX_PER_SOURCE=2 RESEARCH_SOTA_TOP=3 bash scripts/research-sota.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/research-sota"
CACHE_DIR="$DATA/.cache"
DB="$CACHE_DIR/research-sota.db"
HOST="${CT101_SSH_HOST:-ct101}"
MAX_PER_SOURCE="${RESEARCH_SOTA_MAX_PER_SOURCE:-2}"
TOP_N="${RESEARCH_SOTA_TOP:-3}"
TAKEAWAY_BIN="${TAKEAWAY_BIN:-$HOME/.openclaw/workspace/bin/openclaw-takeaway.sh}"
PRIME_URL="${PRIME_CHAT_URL:-http://127.0.0.1:8000/v1/chat/completions}"
PRIME_BASE_URL="${PRIME_URL%/chat/completions}"
# Auto-detect loaded Prime model (survives model swaps); explicit PRIME_MODEL wins
if [[ -z "${PRIME_MODEL:-}" ]]; then
  PRIME_MODEL="$(curl -sfL --max-time 5 "${PRIME_BASE_URL}/models" 2>/dev/null \
    | python3 -c 'import json,sys
try:
    d=json.load(sys.stdin)
    m=d["data"][0]["id"]
    print(m.rsplit("/", 1)[-1] if ".gguf" in m else m)
except Exception:
    print("")' 2>/dev/null || true)"
fi
[[ -n "$PRIME_MODEL" ]] || { echo "[!] Prime /v1/models unreachable — cannot run research-sota" >&2; exit 2; }
echo "[i] prime model: $PRIME_MODEL"

mkdir -p "$OUT" "$CACHE_DIR"
stamp="$(date -u +%Y%m%dT%H%M%SZ)"

# ── Stage 1: Stack Lens — live stack context (local + read-only CT101 mirror) ──
lens_file="$OUT/lens-$stamp.txt"
{
  echo "=== ADRs (head) ==="
  for f in "$ROOT"/docs/ADR-*.md; do
    [[ -f "$f" ]] || continue
    echo "--- $(basename "$f") ---"
    head -c 1500 "$f"
    echo
  done
  echo
  echo "=== data-next/organ-trace (latest scheduler runs) ==="
  find "$DATA/organ-trace" -name '*.json' -mtime -7 2>/dev/null | head -8 | while read -r f; do
    echo "--- ${f#$ROOT/} ---"
    head -c 1200 "$f"
    echo
  done
  echo
  echo "=== data-next/inbox (recent) ==="
  ls -1t "$DATA"/inbox/*/*.md 2>/dev/null | head -6 | while read -r f; do
    echo "--- ${f#$ROOT/} ---"
    head -c 800 "$f"
    echo
  done
  echo
  echo "=== STACK_LIVE_CHAINS ==="
  head -c 2500 "$ROOT/docs/STACK_LIVE_CHAINS.md" 2>/dev/null || echo "(missing)"
} >"$lens_file"
echo "[i] stack lens: $(wc -c <"$lens_file") bytes"

# ── Stage 1: derive research queries (LLM, no static keywords) ──
queries_file="$OUT/queries-$stamp.json"
python3 - "$lens_file" "$queries_file" "$PRIME_URL" "$PRIME_MODEL" <<'PY'
import json, re, sys, urllib.request
from pathlib import Path

lens_path, queries_path, url, model = sys.argv[1:5]
text = Path(lens_path).read_text(encoding="utf-8", errors="replace")

prompt = (
    "You are the Stack Lens of GZMO (sovereign airgap living-memory stack: "
    "Stigmergy board, AOS energy routing, ADOS signed envelopes, RAPL energy, "
    "living vault on CT101, Brain Feed overnight metabolism, Docling extract lane).\n\n"
    "From the live stack snapshot below, derive the 3 most valuable RESEARCH QUERIES "
    "worth fetching from arXiv / GitHub / Hugging Face right now to surface state-of-the-art "
    "techniques relevant to GZMO's functional building blocks. "
    "Rules: queries must follow from what the stack actually is and what it lacks "
    "(no generic AI keywords); each query must be a single search string that works "
    "on all three sources; prefer concrete recent work (models, frameworks, techniques).\n\n"
    "Return ONLY a JSON array of exactly 3 strings, nothing else, no markdown, no prose.\n\n"
    + text[:14000]
)
payload = {"model": model, "messages": [{"role": "user", "content": prompt}],
           "max_tokens": 1024, "temperature": 0.2,
           # Qwen3.x reasoning mode burns the token budget on hidden thinking → empty content;
           # disable per-request (llama-server ignores unknown fields on non-reasoning models)
           "chat_template_kwargs": {"enable_thinking": False}}

# Internal GZMO names are not searchable externally — never leak them into queries
INTERNAL = {"ct101", "workstation", "vm200", "lxc101", "gzmo", "brain feed", "stigmergy", "ados", "aos"}

def sanitize(q: str) -> str:
    q = q.lower()
    for tok in INTERNAL:
        q = q.replace(tok, "")
    return re.sub(r"\s+", " ", q).strip(" ,;-")

queries, lens_err = [], None
for attempt in range(2):
    try:
        req = urllib.request.Request(url, data=json.dumps(payload).encode(),
                                     headers={"Content-Type": "application/json"}, method="POST")
        with urllib.request.urlopen(req, timeout=120) as r:
            body = json.loads(r.read().decode())
        content = body["choices"][0]["message"]["content"].strip()
        content = re.sub(r"^```(?:json)?|```$", "", content, flags=re.M).strip()
        m = re.search(r"\[.*\]", content, flags=re.S)
        if not m:
            raise ValueError(f"no JSON array in LLM response: {content[:200]!r}")
        raw = json.loads(m.group(0))
        if not isinstance(raw, list) or not all(isinstance(q, str) for q in raw):
            raise ValueError("queries not a list of strings")
        queries = [sanitize(q) for q in raw if sanitize(q)]
        if not queries:
            raise ValueError("all queries sanitized away")
        break
    except Exception as e:
        lens_err = str(e)
        queries = []
else:
    # Fallback: derive from ADR titles + organ-trace tokens (deterministic, still stack-derived)
    import collections
    toks = re.findall(r"`([A-Za-z][A-Za-z0-9_./-]{2,})`|\b([A-Z][a-z]+(?:[A-Z][a-z]+)+)\b", text)
    flat = [a or b for a, b in toks]
    stop = INTERNAL | {"true", "false", "none", "pass", "fail", "json", "data", "next"}
    counts = collections.Counter(t for t in flat if t.lower() not in stop)
    top = [t for t, _ in counts.most_common(3)] or ["airgap living memory"]
    queries = [f"{t} sovereign AI airgap" for t in top]

Path(queries_path).write_text(json.dumps({"queries": queries, "llm_error": lens_err}, indent=2) + "\n")
print(json.dumps({"queries": queries, "llm_error": lens_err}, indent=2))
PY

# ── Stage 2a: fetch arXiv / GitHub / HF + sqlite dedupe ──
fetch_file="$OUT/fetch-$stamp.json"
python3 - "$queries_file" "$fetch_file" "$DB" "$MAX_PER_SOURCE" <<'PY'
import json, re, sqlite3, subprocess, sys, urllib.parse
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path

# NOTE: external HTTPS fetches go through curl subprocess — on this host
# urllib.request.urlopen hangs indefinitely on certain HuggingFace/arXiv
# responses (IPv6/keep-alive). curl resolves reliably in <1s. The function
# signatures, dedupe logic, and output shape are identical to research-intel.
UA = "gzmo-research-sota/1.0"

queries = json.loads(Path(sys.argv[1]).read_text())["queries"]
fetch_path, db_path, max_per = sys.argv[2], sys.argv[3], int(sys.argv[4])

con = sqlite3.connect(db_path)
con.execute("CREATE TABLE IF NOT EXISTS seen (key TEXT PRIMARY KEY, first_seen TEXT, title TEXT)")
seen = {r[0] for r in con.execute("SELECT key FROM seen")}

def curl_text(url, timeout=25):
    r = subprocess.run(["curl", "-sfL", "--max-time", str(timeout), "-A", UA, url],
                       capture_output=True, text=True, timeout=timeout + 5)
    if r.returncode != 0:
        raise RuntimeError(f"curl exit {r.returncode}: {r.stderr.strip()[:200]}")
    return r.stdout

def fetch_json(url, timeout=20):
    return json.loads(curl_text(url, timeout))

STOP = {"the", "and", "for", "with", "local", "airgap", "airgapped", "offline", "sovereign", "ai", "a", "of", "in", "on"}

def terms(q, n=3):
    words = [w for w in re.split(r"[^a-zA-Z0-9_]+", q.lower()) if len(w) > 2]
    kept = [w for w in words if w not in STOP] or words
    return kept[:n]

def fetch_arxiv(q):
    out = []
    # long quoted phrases match nothing on arXiv → AND of key terms instead
    query = " AND ".join(f"all:{t}" for t in terms(q, 3))
    u = ("http://export.arxiv.org/api/query?search_query="
         + urllib.parse.quote(query) + f"&start=0&max_results={max_per}&sortBy=submittedDate&sortOrder=descending")
    root = ET.fromstring(curl_text(u, 25))
    ns = {"a": "http://www.w3.org/2005/Atom"}
    for e in root.findall("a:entry", ns):
        key = (e.findtext("a:id", "", ns) or "").strip()
        if key in seen:
            continue
        seen.add(key)
        con.execute("INSERT OR IGNORE INTO seen (key, first_seen, title) VALUES (?,?,?)",
                    (key, datetime.now(timezone.utc).isoformat(),
                     (e.findtext("a:title", "", ns) or "").strip()[:200]))
        out.append({"source": "arxiv", "id": key,
                    "title": re.sub(r"\s+", " ", e.findtext("a:title", "", ns)).strip(),
                    "url": key,
                    "published": (e.findtext("a:published", "", ns) or "")[:10],
                    "summary": re.sub(r"\s+", " ", e.findtext("a:summary", "", ns)).strip()[:500]})
    return out

def fetch_github(q):
    out = []
    short = " ".join(terms(q, 4))
    u = f"https://api.github.com/search/repositories?q={urllib.parse.quote(short)}&sort=stars&per_page={max_per}"
    try:
        d = fetch_json(u)
    except Exception:
        return out
    for it in d.get("items", []):
        key = it.get("full_name", "")
        if key in seen:
            continue
        seen.add(key)
        con.execute("INSERT OR IGNORE INTO seen (key, first_seen, title) VALUES (?,?,?)",
                    (key, datetime.now(timezone.utc).isoformat(), key))
        out.append({"source": "github", "id": key,
                    "title": f"{key} — {it.get('description') or ''}"[:200],
                    "url": it.get("html_url", key),
                    "published": (it.get("pushed_at") or "")[:10],
                    "summary": f"stars={it.get('stargazers_count', 0)} lang={it.get('language') or '-'}"})
    return out

def fetch_hf(q):
    out = []
    short = " ".join(terms(q, 3))
    u = f"https://huggingface.co/api/models?search={urllib.parse.quote(short)}&sort=trendingScore&limit={max_per}"
    try:
        d = fetch_json(u)
    except Exception:
        return out
    for it in d:
        key = it.get("id", "")
        if key in seen:
            continue
        seen.add(key)
        con.execute("INSERT OR IGNORE INTO seen (key, first_seen, title) VALUES (?,?,?)",
                    (key, datetime.now(timezone.utc).isoformat(), key))
        out.append({"source": "huggingface", "id": key,
                    "title": key,
                    "url": f"https://huggingface.co/{key}",
                    "published": (it.get("lastModified") or "")[:10],
                    "summary": f"likes={it.get('likes', 0)} downloads={it.get('downloads', 0)} tags={','.join((it.get('tags') or [])[:5])}"})
    return out

results, errors = [], {}
for q in queries:
    for name, fn in (("arxiv", fetch_arxiv), ("github", fetch_github), ("huggingface", fetch_hf)):
        try:
            got = fn(q)
            for item in got:
                item["query"] = q
            results.extend(got)
        except Exception as e:
            errors.setdefault(name, str(e))
con.commit()
con.close()
Path(fetch_path).write_text(json.dumps({"findings": results, "errors": errors, "seen_total": len(seen)}, indent=2) + "\n")
print(json.dumps({"findings": len(results), "errors": errors}, indent=2))
PY

# ── Stage 2b: SOTA Synthesizer (LLM) + output + takeaway ──
python3 - "$fetch_file" "$queries_file" "$OUT" "$stamp" "$PRIME_URL" "$PRIME_MODEL" "$TOP_N" <<'PY'
import json, re, sys, urllib.request
from datetime import datetime, timezone
from pathlib import Path

fetch = json.loads(Path(sys.argv[1]).read_text())
queries = json.loads(Path(sys.argv[2]).read_text())["queries"]
out, stamp, url, model, top_n = Path(sys.argv[3]), sys.argv[4], sys.argv[5], sys.argv[6], int(sys.argv[7])
findings = fetch.get("findings") or []

synthesized, synth_err = [], None
if findings:
    listing = "\n".join(
        f"- [{i}] ({f['source']}/{f['id']}) {f['title']} | {f.get('summary','')[:200]}"
        for i, f in enumerate(findings)
    )
    prompt = (
        "You are the SOTA Synthesizer for GZMO (sovereign airgap living-memory stack: "
        "Stigmergy board, AOS energy routing, ADOS signed envelopes, RAPL energy, "
        "living vault on CT101, Brain Feed overnight metabolism, Docling extract lane).\n\n"
        "For each finding below, synthesize a state-of-the-art assessment:\n"
        "  (a) baugruppe — the GZMO functional building block it maps to "
        "(e.g. 'memory/vault', 'energy routing', 'extract lane', 'stigmergy board', 'LLM inference', 'envelope/signing');\n"
        "  (b) trl — Technology Readiness Level 1-9 (your best estimate from the source);\n"
        "  (c) conventional_standard — the conventional / herkömmliche baseline this SOTA replaces;\n"
        "  (d) integration_lever — ONE concrete GZMO integration point (which component, what changes);\n"
        "  (e) benefit — true only if it is genuinely useful for GZMO with a real lever; "
        "strictly false for generic AI news with no concrete GZMO applicability.\n"
        "Index refers to the list.\n\n"
        f"Queries: {json.dumps(queries)}\n\nFindings:\n{listing}\n\n"
        "Return ONLY a JSON array of objects "
        "{index, baugruppe, trl, conventional_standard, integration_lever, benefit, why} — no markdown."
    )
    payload = {"model": model, "messages": [{"role": "user", "content": prompt}],
               "max_tokens": 2400, "temperature": 0.1,
               "chat_template_kwargs": {"enable_thinking": False}}
    try:
        req = urllib.request.Request(url, data=json.dumps(payload).encode(),
                                     headers={"Content-Type": "application/json"}, method="POST")
        with urllib.request.urlopen(req, timeout=240) as r:
            body = json.loads(r.read().decode())
        content = body["choices"][0]["message"]["content"].strip()
        content = re.sub(r"^```(?:json)?|```$", "", content, flags=re.M).strip()
        m = re.search(r"\[.*\]", content, flags=re.S)
        if not m:
            raise ValueError("no JSON array")
        synthesized = json.loads(m.group(0))
        if not isinstance(synthesized, list):
            raise ValueError("not a list")
    except Exception as e:
        synth_err = str(e)
        synthesized = []

# Merge + rank
items = []
for i, f in enumerate(findings):
    ev = next((e for e in synthesized if e.get("index") == i), None)
    items.append({**f,
                  "baugruppe": (ev or {}).get("baugruppe", ""),
                  "trl": (ev or {}).get("trl"),
                  "conventional_standard": (ev or {}).get("conventional_standard", ""),
                  "integration_lever": (ev or {}).get("integration_lever", ""),
                  "benefit": bool(ev and ev.get("benefit")),
                  "why": (ev or {}).get("why", "")})
items.sort(key=lambda x: (not x["benefit"], -(x.get("trl") or 0), x["source"]))
top = items[:top_n]

now = datetime.now(timezone.utc).isoformat()
md = [f"# research-sota — {stamp}", "",
      f"queries: {json.dumps(queries)}", "",
      f"findings: {len(items)} (benefit: {sum(1 for i in items if i['benefit'])}, synth_error: {synth_err or 'none'})", "",
      "## Synthese-Tabelle", ""]
md.append("| Baugruppe | SOTA (Quelle/ID) | TRL | Konventioneller Standard | Integration-Hebel |")
md.append("|---|---|---|---|---|")
for it in items:
    bg = it.get("baugruppe") or "-"
    src = f"{it['source']}/{it['id']}"[:60]
    trl = str(it.get("trl") or "-")
    conv = (it.get("conventional_standard") or "-").replace("|", "/")[:60]
    lev = (it.get("integration_lever") or "-").replace("|", "/")[:80]
    md.append(f"| {bg} | {src} | {trl} | {conv} | {lev} |")
md.append("")
md.append("## Top findings")
for t in top:
    md.append(f"### {t['title']}")
    md.append(f"- baugruppe: {t.get('baugruppe','-')} · TRL: {t.get('trl','-')} · benefit: {t['benefit']}")
    md.append(f"- source: {t['source']} · published: {t.get('published','?')}")
    md.append(f"- url: {t['url']}")
    if t.get("conventional_standard"):
        md.append(f"- konventioneller Standard: {t['conventional_standard']}")
    if t["why"]:
        md.append(f"- why: {t['why']}")
    if t["integration_lever"]:
        md.append(f"- integration-hebel: {t['integration_lever']}")
    md.append("")
md.append("## All findings")
for f in items:
    md.append(f"- [{f['source']}] {f['title']} — {f['url']} (benefit={f['benefit']}, TRL={f.get('trl','-')})")

synthesis = [
    {"baugruppe": it.get("baugruppe") or "-",
     "sota": f"{it['source']}/{it['id']}",
     "trl": it.get("trl"),
     "conventional_standard": it.get("conventional_standard", ""),
     "integration_lever": it.get("integration_lever", ""),
     "benefit": it["benefit"]}
    for it in items
]

(out / f"research-sota-{stamp}.json").write_text(
    json.dumps({"schema": "gzmo.research_sota/v1", "generated_at": now, "ok": True,
                "queries": queries, "findings": items, "synthesis": synthesis,
                "fetch_errors": fetch.get("errors"), "synth_error": synth_err,
                "seen_total": fetch.get("seen_total")}, indent=2) + "\n")
(out / f"research-sota-{stamp}.md").write_text("\n".join(md), encoding="utf-8")
print(json.dumps({"findings": len(items), "benefit": sum(1 for i in items if i["benefit"]),
                  "top": [t["title"] for t in top], "synth_error": synth_err}, indent=2))
PY

# ── Cross-day URL dedup: latest.{json,md} with NEW/REPEAT split ──
# shellcheck source=scripts/lib-research-dedup.sh
source "$ROOT/scripts/lib-research-dedup.sh"
cp "$OUT/research-sota-$stamp.json" "$OUT/latest.json"
dedup_findings "$OUT/latest.json" "$OUT/seen.jsonl"
dedup_render_latest "$OUT/latest.json" "$OUT/latest.md" sota "$stamp" "$TOP_N"
dedup_seen_update "$OUT/latest.json" "$OUT/seen.jsonl" "$(date -u +%Y-%m-%d)"

# ── Top findings → Brain Feed (takeaway; CT101 metabolizes overnight) ──
if [[ -x "$TAKEAWAY_BIN" && -f "$OUT/latest.json" ]]; then
  python3 - "$OUT/latest.json" "$TOP_N" <<'PY' | while IFS= read -r line; do
import json, sys
from pathlib import Path
d = json.loads(Path(sys.argv[1]).read_text())
top = [f for f in d["findings"] if f.get("benefit")][:int(sys.argv[2])]
if top:
    print("research-sota: " + "; ".join(f"{f['title']} [TRL{f.get('trl','-')} {f['integration_lever'][:80]}]" for f in top))
PY
    [[ -n "$line" ]] && bash "$TAKEAWAY_BIN" "$line" || true
  done
fi

echo "[i] research-sota done → $OUT/latest.{md,json}"
