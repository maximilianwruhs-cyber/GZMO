#!/usr/bin/env bash
# Living tool zoo — which metabolism/soft-fail jobs fired (from scheduler-runs).
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-$HOME/github-clone}/GZMO"
DATA="$ROOT/data-next"
RUNS="$DATA/scheduler-runs"
OUT_DIR="$DATA/organ-trace"
mkdir -p "$OUT_DIR"

exec python3 - "$RUNS" "$OUT_DIR" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

runs = Path(sys.argv[1])
out_dir = Path(sys.argv[2])

# Map job names → organ / assembly labels (opportunity-map living zoo).
ORGANS = {
    "distill": "session-distill",
    "promote": "honeypot-gate / vault-promote",
    "embed": "embeddings + qdrant-sync",
    "dream": "dream-append / REM",
    "spark": "spark-link",
    "wiki": "wiki-okforge-push",
    "dream-compact": "dreams_md compact",
    "watchdog": "missed-run watchdog",
}

fired = []
if runs.is_dir():
    for path in sorted(runs.glob("latest-*.json")):
        job = path.name.removeprefix("latest-").removesuffix(".json")
        if job in ("json",):
            continue
        try:
            r = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        # Watchdog uses different schema
        if job == "watchdog":
            fired.append(
                {
                    "job": job,
                    "organ": ORGANS.get(job, job),
                    "ok": r.get("ok", not r.get("stale", True)),
                    "finished": r.get("checked_at"),
                    "detail": r.get("detail"),
                }
            )
            continue
        fired.append(
            {
                "job": job,
                "organ": ORGANS.get(job, job),
                "ok": bool(r.get("ok")),
                "finished": r.get("finished"),
                "runner": r.get("runner") or r.get("script"),
                "error": r.get("error"),
            }
        )

# Prefer metabolism order then satellites
order = ["distill", "promote", "embed", "dream", "spark", "wiki", "dream-compact", "watchdog"]
fired.sort(key=lambda x: order.index(x["job"]) if x["job"] in order else 99)

trace = {
    "schema": "gzmo.organ-trace/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "runs_dir": str(runs),
    "organs_fired": len(fired),
    "ok_count": sum(1 for f in fired if f.get("ok")),
    "jobs": fired,
}

(out_dir / "latest.json").write_text(json.dumps(trace, indent=2) + "\n", encoding="utf-8")

# Human markdown
lines = [
    f"# Organ trace — {trace['generated_at']}",
    "",
    f"Jobs with `latest-*` records: **{trace['organs_fired']}** (ok={trace['ok_count']})",
    "",
    "| Job | Organ | Result | Last |",
    "|-----|-------|--------|------|",
]
for f in fired:
    mark = "OK" if f.get("ok") else "FAIL/STALE"
    lines.append(
        f"| {f['job']} | {f['organ']} | {mark} | {f.get('finished') or '—'} |"
    )
lines.append("")
(out_dir / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({"organs_fired": trace["organs_fired"], "ok_count": trace["ok_count"], "path": str(out_dir / "latest.json")}, indent=2))
PY
