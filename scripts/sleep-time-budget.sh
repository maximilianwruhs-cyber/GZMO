#!/usr/bin/env bash
# Sleep-time compute budget (G6) — when-to-dream from nutrient backlog.
# Inspired by Lin et al. arXiv:2504.13171. Never starts a second writer.
#
#   bash scripts/sleep-time-budget.sh
# Artifact: data-next/sleep-time/latest.{json,md}
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/sleep-time"
HOST="${CT101_SSH_HOST:-ct101}"
RUNS="${KEEP_QUALITY_DATA_DIR:-/opt/gzmo/data}/scheduler-runs"
mkdir -p "$OUT"

export OUT HOST RUNS ROOT
python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
host = os.environ["HOST"]
runs = os.environ["RUNS"]
now = datetime.now(timezone.utc).isoformat()

def ssh(cmd: str) -> str:
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
        capture_output=True, text=True, timeout=25,
    )
    return (p.stdout or "").strip()

# Nutrient backlog proxies: distill queue size, missing embeddings, immune candidates
distill_q = ssh("ls /opt/gzmo/data/distill-queue 2>/dev/null | wc -l || echo 0")
missing_emb = ssh(
    "sqlite3 /opt/gzmo/data/vault.db \"SELECT COUNT(*) FROM semantic_vault WHERE embedding IS NULL;\" 2>/dev/null || echo 0"
)
immune_n = ssh(
    "python3 -c \"import json; d=json.load(open('/opt/gzmo/data/immune/latest.json')); print(len(d.get('candidates',[])))\" 2>/dev/null || echo 0"
)

def to_int(s: str) -> int:
    try:
        return int("".join(ch for ch in s.split()[-1] if ch.isdigit()) or "0")
    except Exception:
        return 0

backlog = {
    "distill_queue": to_int(distill_q),
    "missing_embeddings": to_int(missing_emb),
    "immune_candidates": to_int(immune_n),
}
score = (
    backlog["distill_queue"] * 2
    + backlog["missing_embeddings"]
    + backlog["immune_candidates"] * 3
)
# Budget: light / normal / deep REM
if score >= 40:
    budget = "deep"
    dream_passes = 2
    spark_slots = 2
elif score >= 10:
    budget = "normal"
    dream_passes = 1
    spark_slots = 1
else:
    budget = "light"
    dream_passes = 1
    spark_slots = 0  # skip spark if nothing to link

payload = {
    "schema": "gzmo.sleep_time_budget/v1",
    "generated_at": now,
    "backlog": backlog,
    "score": score,
    "budget": budget,
    "advice": {
        "dream_passes": dream_passes,
        "spark_slots": spark_slots,
        "note": "Operator/daemon may skip spark when budget=light; never dual-write",
    },
    "cite": "arXiv:2504.13171 sleep-time compute",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    f"# Sleep-time budget — {budget}",
    "",
    f"- score={score} backlog={backlog}",
    f"- dream_passes={dream_passes} spark_slots={spark_slots}",
    f"- generated_at={now}",
    "",
]
(out / "latest.md").write_text("\n".join(md), encoding="utf-8")
print(json.dumps({"ok": True, "budget": budget, "score": score}, indent=2))
PY
