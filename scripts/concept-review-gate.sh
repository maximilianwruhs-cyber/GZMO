#!/usr/bin/env bash
# Concept review gate — hold wiki concepts that lack vault evidence.
# Soft-fail style: writes data-next/concept-gate/latest.json; exit 1 on HOLD.
#
#   bash scripts/concept-review-gate.sh
#   CONCEPT_GATE_LIMIT=10 bash scripts/concept-review-gate.sh
#   CONCEPT_GATE_META=path/to/wiki-push-latest.json bash scripts/concept-review-gate.sh
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-$HOME/github-clone}/GZMO"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
GZMO="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
META="${CONCEPT_GATE_META:-$ROOT/data-next/wiki-push-latest.json}"
LIMIT="${CONCEPT_GATE_LIMIT:-12}"
OUT_DIR="$ROOT/data-next/concept-gate"
mkdir -p "$OUT_DIR"

export ROOT GZMO META LIMIT OUT_DIR
exec python3 - <<'PY'
import json
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
meta_path = Path(os.environ["META"])
limit = int(os.environ.get("LIMIT", "12"))
out_dir = Path(os.environ["OUT_DIR"])
gzmo = os.environ["GZMO"]

STOP = {
    "the", "for", "and", "with", "from", "that", "this", "used", "as", "in",
    "a", "an", "of", "to", "is", "are", "at", "by", "on", "or", "be",
}

def slug_to_query(path: str) -> tuple[str, str]:
    """concepts/person-meridian-vesper-….md → ('Meridian Vesper', 'Meridian')"""
    base = Path(path).stem
    parts = base.split("-")
    if parts and parts[0] in {
        "person", "system", "object", "place", "project", "decision",
        "tool", "policy", "concept", "proc",
    }:
        parts = parts[1:]
    # Drop trailing hash-ish token
    if parts and re.fullmatch(r"[0-9a-f]{6,}", parts[-1]):
        parts = parts[:-1]
    meaningful = [p for p in parts if p not in STOP and not p.isdigit()][:4]
    if not meaningful:
        meaningful = parts[:3] or ["unknown"]
    query = " ".join(w.capitalize() for w in meaningful[:3])
    needle = meaningful[0].capitalize()
    # Prefer multi-word proper nouns when present
    if len(meaningful) >= 2:
        needle = f"{meaningful[0].capitalize()}-{meaningful[1].capitalize()}"
        query = needle.replace("-", " ")
        # Also try spaced form in search; needle for hit uses either
    return query, needle

if not meta_path.is_file():
    report = {
        "schema": "gzmo.concept-gate/v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "verdict": "HOLD",
        "reason": f"missing meta {meta_path}",
        "checked": 0,
        "pass": 0,
        "hold": 0,
        "results": [],
    }
    (out_dir / "latest.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps({"verdict": "HOLD", "reason": report["reason"]}, indent=2))
    sys.exit(1)

meta = json.loads(meta_path.read_text(encoding="utf-8"))
paths = list(meta.get("paths") or [])[:limit]
env = os.environ.copy()
env.setdefault("GZMO_INSTANCE", "next")
env.setdefault("GZMO_CONFIG", str(root / "config" / "gzmo.toml"))
env.setdefault("GZMO_ALLOW_LAB_VAULT", "1")

results = []
holds = 0
for path in paths:
    query, needle = slug_to_query(path)
    proc = subprocess.run(
        [gzmo, "memory", "search", query, "--limit", "5", "--no-scratch"],
        capture_output=True,
        text=True,
        env=env,
        check=False,
    )
    text = (proc.stdout or "") + (proc.stderr or "")
    # Accept either hyphenated or spaced needle
    n1 = needle.lower()
    n2 = needle.replace("-", " ").lower()
    hit = proc.returncode == 0 and (n1 in text.lower() or n2 in text.lower())
    # Fallback: first token alone
    if not hit and "-" in needle:
        tok = needle.split("-")[0].lower()
        hit = proc.returncode == 0 and tok in text.lower() and len(tok) >= 4
    if not hit:
        holds += 1
    results.append(
        {
            "path": path,
            "query": query,
            "needle": needle,
            "verdict": "PASS" if hit else "HOLD",
            "evidence_excerpt": text[:240].replace("\n", " "),
        }
    )

verdict = "PASS" if holds == 0 and results else ("HOLD" if holds else "PASS")
if not results:
    verdict = "HOLD"
report = {
    "schema": "gzmo.concept-gate/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "meta": str(meta_path),
    "commit_sha": (meta.get("commit_sha") or "")[:12],
    "concepts_written": meta.get("concepts_written"),
    "checked": len(results),
    "pass": len(results) - holds,
    "hold": holds,
    "verdict": verdict,
    "results": results,
}
(out_dir / "latest.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
md = [
    f"# Concept review gate — {report['generated_at']}",
    "",
    f"**Verdict:** {verdict} · pass={report['pass']}/{report['checked']} · sha={report['commit_sha']}",
    "",
    "| Path | Query | Verdict |",
    "|------|-------|---------|",
]
for r in results:
    md.append(f"| `{Path(r['path']).name}` | {r['query']} | {r['verdict']} |")
md.append("")
(out_dir / "latest.md").write_text("\n".join(md), encoding="utf-8")
print(json.dumps({"verdict": verdict, "pass": report["pass"], "hold": holds, "checked": len(results)}, indent=2))
sys.exit(0 if verdict == "PASS" else 1)
PY
