#!/usr/bin/env bash
# Thin nightburst scoreboard: metabolism + wiki + Arena → sanitized JSON/HTML.
# Local stranger-demo surface (OKForge /observatory remains agent-discovery).
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-$HOME/github-clone}/GZMO"
DATA="$ROOT/data-next"
OUT_DIR="$DATA/arena"
mkdir -p "$OUT_DIR"

exec python3 - "$DATA" "$OUT_DIR" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

data = Path(sys.argv[1])
out_dir = Path(sys.argv[2])
runs = data / "scheduler-runs"
wiki_meta = data / "wiki-push-latest.json"
arena = out_dir / "latest.json"
watchdog = runs / "latest-watchdog.json"

METAB = ["distill", "promote", "embed", "dream", "spark"]


def load_json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return None


jobs = {}
for job in METAB:
    r = load_json(runs / f"latest-{job}.json")
    if not r:
        jobs[job] = {"status": "missing"}
        continue
    jobs[job] = {
        "status": "ok" if r.get("ok") else "fail",
        "finished": r.get("finished"),
        "runner": r.get("runner") or r.get("script"),
    }

wiki = load_json(wiki_meta) or {}
wiki_pub = {
    "healthy": wiki.get("healthy"),
    "concepts_written": wiki.get("concepts_written"),
    "commit_sha": (wiki.get("commit_sha") or "")[:12],
    # omit tokens / absolute secret paths
}

arena_raw = load_json(arena) or {}
arena_pub = {
    "champion": arena_raw.get("champion"),
    "z": arena_raw.get("z"),
    "quality": arena_raw.get("quality"),
    "efficiency": arena_raw.get("efficiency"),
    "elapsed_ms": arena_raw.get("elapsed_ms"),
    "joules": arena_raw.get("joules"),
    "euro_cost": arena_raw.get("euro_cost"),
    "electricity_c_kwh": arena_raw.get("electricity_c_kwh"),
    "electricity_live": arena_raw.get("electricity_live"),
    "energy_source": arena_raw.get("energy_source"),
    "finished": arena_raw.get("finished"),
}

wd = load_json(watchdog) or {}
watch_pub = {
    "stale": wd.get("stale"),
    "detail": wd.get("detail"),
    "threshold_secs": wd.get("threshold_secs"),
}

organs_raw = load_json(data / "organ-trace" / "latest.json") or {}
organs_pub = {
    "organs_fired": organs_raw.get("organs_fired"),
    "ok_count": organs_raw.get("ok_count"),
    "jobs": [
        {"job": j.get("job"), "organ": j.get("organ"), "ok": j.get("ok")}
        for j in (organs_raw.get("jobs") or [])[:12]
    ],
}

faith_raw = load_json(data / "faithfulness" / "latest.json") or {}
faith_pub = {
    "ok": faith_raw.get("ok"),
    "supported": faith_raw.get("supported"),
    "total": faith_raw.get("total"),
    "mode": faith_raw.get("mode"),
}

board = {
    "schema": "gzmo.nightburst.scoreboard/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "metabolism": jobs,
    "watchdog": watch_pub,
    "wiki": wiki_pub,
    "arena": arena_pub,
    "organs": organs_pub,
    "faithfulness": faith_pub,
    "links": {
        "okforge_observatory": "http://127.0.0.1:3000/observatory",
        "scoreboard_html": str(out_dir / "scoreboard.html"),
    },
}

(out_dir / "scoreboard.json").write_text(json.dumps(board, indent=2) + "\n", encoding="utf-8")

rows = "".join(
    f"<tr><td>{j}</td><td>{v.get('finished') or '—'}</td><td>{v.get('status')}</td></tr>"
    for j, v in jobs.items()
)
arena_z = arena_pub.get("z")
wiki_sha = wiki_pub.get("commit_sha") or "—"
euro = arena_pub.get("euro_cost")
joules = arena_pub.get("joules")
price = arena_pub.get("electricity_c_kwh")
live = "live" if arena_pub.get("electricity_live") else "fallback"
html = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<title>GZMO nightburst scoreboard</title>
<style>
  body {{ font-family: ui-sans-serif, system-ui, sans-serif; margin: 2rem; max-width: 52rem; color: #1a1a1a; background: #f7f5f0; }}
  h1 {{ font-size: 1.4rem; margin-bottom: 0.25rem; }}
  .sub {{ color: #555; margin-bottom: 1.5rem; }}
  table {{ border-collapse: collapse; width: 100%; margin: 1rem 0; }}
  th, td {{ border-bottom: 1px solid #ccc; text-align: left; padding: 0.4rem 0.5rem; font-size: 0.95rem; }}
  .pill {{ display: inline-block; padding: 0.15rem 0.5rem; border: 1px solid #888; border-radius: 999px; font-size: 0.8rem; }}
  a {{ color: #0b3d91; }}
</style>
</head>
<body>
  <h1>GZMO nightburst scoreboard</h1>
  <p class="sub">Sanitized local demo — no tokens, no session bodies. Generated {board['generated_at']}</p>
  <p>
    <span class="pill">watchdog: {"STALE" if watch_pub.get("stale") else "fresh"}</span>
    <span class="pill">wiki sha {wiki_sha}</span>
    <span class="pill">arena z={arena_z}</span>
    <span class="pill">€={euro if euro is not None else "—"}</span>
  </p>
  <h2>Metabolism</h2>
  <table>
    <thead><tr><th>Job</th><th>Last run</th><th>Result</th></tr></thead>
    <tbody>{rows}</tbody>
  </table>
  <h2>Arena champion</h2>
  <p>{arena_pub.get("champion") or "—"} · quality={arena_pub.get("quality")} · elapsed_ms={arena_pub.get("elapsed_ms")} · joules={joules} ({arena_pub.get("energy_source")}) · {price} ¢/kWh ({live}) · €={euro}</p>
  <h2>Wiki plane</h2>
  <p>healthy={wiki_pub.get("healthy")} · concepts={wiki_pub.get("concepts_written")} · sha={wiki_sha}</p>
  <h2>Living tool zoo</h2>
  <p>organs_fired={organs_pub.get("organs_fired")} · ok={organs_pub.get("ok_count")}</p>
  <h2>Faithfulness CI</h2>
  <p>ok={faith_pub.get("ok")} · {faith_pub.get("supported")}/{faith_pub.get("total")} ({faith_pub.get("mode")})</p>
  <p><a href="http://127.0.0.1:3000/observatory">OKForge Observatory</a> (agent discovery) · this page is the metabolism/Arena board</p>
</body>
</html>
"""
(out_dir / "scoreboard.html").write_text(html, encoding="utf-8")
print(json.dumps({"scoreboard": str(out_dir / "scoreboard.json"), "html": str(out_dir / "scoreboard.html")}, indent=2))
PY
