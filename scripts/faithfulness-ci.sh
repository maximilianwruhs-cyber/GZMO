#!/usr/bin/env bash
# Faithfulness CI — claims vs vault evidence (gzmo memory search).
# Exit 0 if all claims hit; 1 if any miss. Writes data-next/faithfulness/latest.json.
#
# Usage:
#   bash scripts/faithfulness-ci.sh
#   FAITHFULNESS_CLAIMS=path/to/claims.json bash scripts/faithfulness-ci.sh
#   FAITHFULNESS_MODE=fixture bash scripts/faithfulness-ci.sh   # offline substring vs fixture corpus
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-$HOME/github-clone}/GZMO"
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
GZMO="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
CLAIMS="${FAITHFULNESS_CLAIMS:-$ROOT/scripts/fixtures/faithfulness-claims.json}"
MODE="${FAITHFULNESS_MODE:-vault}"
OUT_DIR="$ROOT/data-next/faithfulness"
mkdir -p "$OUT_DIR"

export ROOT GZMO CLAIMS MODE OUT_DIR
exec python3 - <<'PY'
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
claims_path = Path(os.environ["CLAIMS"])
mode = os.environ.get("MODE", "vault")
out_dir = Path(os.environ["OUT_DIR"])
gzmo = os.environ["GZMO"]

spec = json.loads(claims_path.read_text(encoding="utf-8"))
claims = spec.get("claims") or []
results = []
misses = 0

env = os.environ.copy()
env.setdefault("GZMO_INSTANCE", "next")
env.setdefault("GZMO_CONFIG", str(root / "config" / "gzmo.toml"))
env.setdefault("GZMO_ALLOW_LAB_VAULT", "1")

fixture_corpus = ""
if mode == "fixture":
    # Offline: concatenate claim texts as synthetic evidence (self-consistency gate).
    fixture_corpus = "\n".join(c.get("claim", "") for c in claims)

for c in claims:
    cid = c.get("id") or c.get("claim", "")[:32]
    claim = c.get("claim", "")
    query = c.get("query") or claim
    needle = c.get("needle") or query
    evidence = ""
    hit = False
    if mode == "fixture":
        evidence = fixture_corpus
        hit = needle.lower() in evidence.lower()
    else:
        proc = subprocess.run(
            [gzmo, "memory", "search", query, "--limit", "5", "--no-scratch"],
            capture_output=True,
            text=True,
            env=env,
            check=False,
        )
        evidence = (proc.stdout or "") + (proc.stderr or "")
        hit = proc.returncode == 0 and needle.lower() in evidence.lower()
    if not hit:
        misses += 1
    results.append(
        {
            "id": cid,
            "claim": claim,
            "query": query,
            "needle": needle,
            "supported": hit,
            "evidence_excerpt": evidence[:400].replace("\n", " "),
        }
    )

report = {
    "schema": "gzmo.faithfulness.ci/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "mode": mode,
    "claims_file": str(claims_path),
    "total": len(results),
    "supported": len(results) - misses,
    "misses": misses,
    "ok": misses == 0,
    "results": results,
}
out_path = out_dir / "latest.json"
out_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"ok": report["ok"], "supported": report["supported"], "total": report["total"], "path": str(out_path)}, indent=2))
sys.exit(0 if report["ok"] else 1)
PY
