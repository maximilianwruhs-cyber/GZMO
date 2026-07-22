#!/usr/bin/env bash
# Thin nightburst scoreboard: metabolism + wiki + Arena → sanitized JSON/HTML.
# Local stranger-demo surface (OKForge /observatory remains agent-discovery).
#
# By default prefers CT101 living scheduler-runs (avoids lab 2020 stubs in
# workstation data-next/). Override: SCOREBOARD_SOURCE=lab
set -euo pipefail

ROOT="${GZMO_CLONE_ROOT:-$HOME/github-clone}/GZMO"
DATA="$ROOT/data-next"
OUT_DIR="$DATA/arena"
HOST="${CT101_SSH_HOST:-ct101}"
SOURCE="${SCOREBOARD_SOURCE:-living}"
mkdir -p "$OUT_DIR"

RUNS_DIR="$DATA/scheduler-runs"
if [[ "$SOURCE" == "living" ]]; then
  LIVING_RUNS="$DATA/scheduler-runs-living"
  mkdir -p "$LIVING_RUNS"
  if scp -o ConnectTimeout=10 -o BatchMode=yes \
    "$HOST:/opt/gzmo/data/scheduler-runs/latest-"*.json \
    "$LIVING_RUNS/" >/tmp/scoreboard-scp.log 2>&1; then
    RUNS_DIR="$LIVING_RUNS"
  else
    echo "[warn] living scheduler-runs scp failed — falling back to $RUNS_DIR (see /tmp/scoreboard-scp.log)" >&2
  fi
fi

exec python3 - "$DATA" "$OUT_DIR" "$RUNS_DIR" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

data = Path(sys.argv[1])
out_dir = Path(sys.argv[2])
runs = Path(sys.argv[3])
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

gate_raw = load_json(data / "concept-gate" / "latest.json") or {}
gate_pub = {
    "verdict": gate_raw.get("verdict"),
    "pass": gate_raw.get("pass"),
    "hold": gate_raw.get("hold"),
    "checked": gate_raw.get("checked"),
}

hsp_raw = load_json(data / "hsp-metabolism" / "latest.json") or {}
hsp_pub = {
    "events": len(hsp_raw.get("events") or []),
    "ts": hsp_raw.get("ts"),
}

euro_raw = load_json(out_dir / "euro-night.json") or {}
euro_pub = {
    "euro_night_total": euro_raw.get("euro_night_total"),
    "arena_euro_sum": euro_raw.get("arena_euro_sum"),
    "metabolism_euro_est": euro_raw.get("metabolism_euro_est"),
    "arena_runs": euro_raw.get("arena_runs"),
}

pw_raw = load_json(data / "price-window" / "latest.json") or {}
pw_sug = pw_raw.get("suggestions") or {}
price_pub = {
    "live": pw_raw.get("live"),
    "cheapest_24h_c_kwh": (pw_raw.get("cheapest_24h") or {}).get("c_kwh"),
    "distill_suggest": (pw_sug.get("distill") or {}).get("suggested_start_utc"),
    "dream_suggest": (pw_sug.get("dream") or {}).get("suggested_start_utc"),
    "distill_shift_h": (pw_sug.get("distill") or {}).get("shift_hours"),
    "dream_shift_h": (pw_sug.get("dream") or {}).get("shift_hours"),
}

board = {
    "schema": "gzmo.nightburst.scoreboard/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "scheduler_runs_dir": str(runs),
    "metabolism": jobs,
    "watchdog": watch_pub,
    "wiki": wiki_pub,
    "arena": arena_pub,
    "organs": organs_pub,
    "faithfulness": faith_pub,
    "concept_gate": gate_pub,
    "hsp": hsp_pub,
    "euro_night": euro_pub,
    "price_window": price_pub,
    "links": {
        "okforge_observatory": "http://127.0.0.1:3000/observatory",
        "scoreboard_html": str(out_dir / "scoreboard.html"),
        "aos_status": str(data / "aos-status" / "latest.json"),
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
euro_night = euro_pub.get("euro_night_total")
joules = arena_pub.get("joules")
price = arena_pub.get("electricity_c_kwh")
live = "live" if arena_pub.get("electricity_live") else "fallback"
gate_v = gate_pub.get("verdict") or "—"
hsp_n = hsp_pub.get("events")
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
    <span class="pill">€ burst={euro if euro is not None else "—"}</span>
    <span class="pill">€/night={euro_night if euro_night is not None else "—"}</span>
    <span class="pill">gate {gate_v}</span>
    <span class="pill">hsp events={hsp_n if hsp_n is not None else "—"}</span>
    <span class="pill">price {"live" if price_pub.get("live") else "—"} cheap={price_pub.get("cheapest_24h_c_kwh")}</span>
  </p>
  <h2>Metabolism</h2>
  <table>
    <thead><tr><th>Job</th><th>Last run</th><th>Result</th></tr></thead>
    <tbody>{rows}</tbody>
  </table>
  <h2>Arena champion</h2>
  <p>{arena_pub.get("champion") or "—"} · quality={arena_pub.get("quality")} · elapsed_ms={arena_pub.get("elapsed_ms")} · joules={joules} ({arena_pub.get("energy_source")}) · {price} ¢/kWh ({live}) · €={euro}</p>
  <h2>€/night</h2>
  <p>total={euro_night} · arena_sum={euro_pub.get("arena_euro_sum")} ({euro_pub.get("arena_runs")} runs) · metabolism_est={euro_pub.get("metabolism_euro_est")}</p>
  <h2>Wiki plane</h2>
  <p>healthy={wiki_pub.get("healthy")} · concepts={wiki_pub.get("concepts_written")} · sha={wiki_sha}</p>
  <h2>Concept gate</h2>
  <p>verdict={gate_v} · pass={gate_pub.get("pass")} · hold={gate_pub.get("hold")} · checked={gate_pub.get("checked")}</p>
  <h2>Living tool zoo</h2>
  <p>organs_fired={organs_pub.get("organs_fired")} · ok={organs_pub.get("ok_count")}</p>
  <h2>Faithfulness CI</h2>
  <p>ok={faith_pub.get("ok")} · {faith_pub.get("supported")}/{faith_pub.get("total")} ({faith_pub.get("mode")})</p>
  <h2>HSP motif</h2>
  <p>events={hsp_n} · ts={hsp_pub.get("ts") or "—"}</p>
  <h2>Price window</h2>
  <p>live={price_pub.get("live")} · cheapest_24h={price_pub.get("cheapest_24h_c_kwh")} ¢/kWh · distill→{price_pub.get("distill_suggest")} (Δ{price_pub.get("distill_shift_h")}h) · dream→{price_pub.get("dream_suggest")} (Δ{price_pub.get("dream_shift_h")}h)</p>
  <p><a href="http://127.0.0.1:3000/observatory">OKForge Observatory</a> · AOS feed: <code>data-next/aos-status/latest.json</code></p>
</body>
</html>
"""
(out_dir / "scoreboard.html").write_text(html, encoding="utf-8")
print(json.dumps({"scoreboard": str(out_dir / "scoreboard.json"), "html": str(out_dir / "scoreboard.html")}, indent=2))
PY
