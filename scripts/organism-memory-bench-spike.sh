#!/usr/bin/env bash
# Thin LoCoMo / LongMemEval-style spike against living honeypot (borrow-eval).
# Airgap-honest: uses local MCP/search path when available; no cloud required.
# Does NOT claim LoCoMo SOTA — satellite metric beside Keep-quality.
#
#   bash scripts/organism-memory-bench-spike.sh
# Artifact: data-next/organism-bench/latest.{json,md}
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/organism-bench"
HOST="${CT101_SSH_HOST:-ct101}"
VAULT="${KEEP_QUALITY_VAULT_DB:-/opt/gzmo/data/vault.db}"
mkdir -p "$OUT"

# Tiny fixture inspired by LoCoMo single-hop / temporal probes (not the full dataset).
FIXTURE="$OUT/fixture-queries.json"
cat >"$FIXTURE" <<'JSON'
[
  {"id": "q1", "type": "single_hop", "query": "ADR-0003 one living instance"},
  {"id": "q2", "type": "single_hop", "query": "Brain Feed"},
  {"id": "q3", "type": "temporal", "query": "promote honeypot"},
  {"id": "q4", "type": "multi_hop", "query": "airgap living"},
  {"id": "q5", "type": "abstention", "query": "zzzz-nonexistent-token-9f3a2"}
]
JSON

export OUT HOST VAULT FIXTURE ROOT
python3 - <<'PY'
import json, os, subprocess, re
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
host = os.environ["HOST"]
vault = os.environ["VAULT"]
fixture = json.loads(Path(os.environ["FIXTURE"]).read_text(encoding="utf-8"))
now = datetime.now(timezone.utc).isoformat()

def ssh_search(q: str) -> list[str]:
    # Prefer FTS / LIKE on honeypot — no LLM judge required for spike.
    q_esc = q.replace("'", "''")
    tokens = [t for t in re.split(r"\s+", q.strip()) if len(t) >= 3][:4]
    if not tokens:
        return []
    like = " OR ".join(f"content_norm LIKE '%{t.lower().replace(chr(39), '')}%'" for t in tokens)
    sql = (
        f"sqlite3 {vault} \"SELECT substr(content,1,120) FROM honeypot "
        f"WHERE is_latest=1 AND ({like}) "
        f"ORDER BY recall_count DESC, confidence DESC LIMIT 3;\""
    )
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, sql],
        capture_output=True, text=True, timeout=30,
    )
    lines = [ln.strip() for ln in (p.stdout or "").splitlines() if ln.strip()]
    return lines

results = []
hits = 0
for item in fixture:
    rows = ssh_search(item["query"])
    hit = bool(rows)
    if item["type"] == "abstention":
        # success = no hit
        ok = not hit
    else:
        ok = hit
    if ok:
        hits += 1
    results.append({"id": item["id"], "type": item["type"], "ok": ok, "n": len(rows), "sample": rows[:1]})

n = len(fixture)
score = hits / n if n else 0.0
payload = {
    "schema": "gzmo.organism_bench_spike/v1",
    "generated_at": now,
    "bench": "locomo_inspired_spike",
    "cite": ["arXiv:2402.17753 LoCoMo", "research/sleep-consolidation-sota-2026-07-22.md"],
    "score": round(score, 4),
    "hits": hits,
    "n": n,
    "results": results,
    "advice": "satellite_only — Keep-quality remains USP bar; improve Felt Use/utility if score < 0.6",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    f"# Organism bench spike\n\nscore={score:.2%} ({hits}/{n})\n\n{payload['advice']}\n",
    encoding="utf-8",
)
print(json.dumps({"ok": True, "score": score, "hits": hits, "n": n}, indent=2))
PY
