#!/usr/bin/env bash
# Unpark Wave 4.3 demable: wiki search mind + sanitized Observatory scoreboard.
# Never starts gzmo-daemon wiki jobs. Never pushes wiki without an explicit separate gate.
#
#   bash scripts/wiki-observatory-demo.sh
#   WIKI_MIND_QUERY=Lint bash scripts/wiki-observatory-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/wiki-observatory"
QUERY="${WIKI_MIND_QUERY:-Lint}"
mkdir -p "$OUT"

echo "[1/4] wiki-mind-check (seeded search)…"
WIKI_MIND_QUERY="$QUERY" bash "$ROOT/scripts/wiki-mind-check.sh" >/tmp/wiki-obs-mind.log 2>&1 || {
  echo "wiki-mind-check failed — see /tmp/wiki-obs-mind.log" >&2
  exit 1
}

echo "[2/4] nightburst-scoreboard (sanitized public mind)…"
bash "$ROOT/scripts/nightburst-scoreboard.sh" >/tmp/wiki-obs-scoreboard.log 2>&1 || {
  echo "nightburst-scoreboard failed — see /tmp/wiki-obs-scoreboard.log" >&2
  exit 1
}

echo "[3/4] aos-poll-dashboard (soft read-only)…"
bash "$ROOT/scripts/aos-poll-dashboard.sh" >/tmp/wiki-obs-aos.log 2>&1 || true

echo "[4/4] assemble felt + demo inventory…"
export ROOT OUT DATA QUERY
python3 - <<'PY'
import json
import os
import re
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
out = Path(os.environ["OUT"])
data = Path(os.environ["DATA"])
query = os.environ.get("QUERY") or "Lint"

wiki_mind = data / "wiki-mind"
search_txt = wiki_mind / "wiki-search.txt"
mind_latest = wiki_mind / "latest.json"
board_json = data / "arena" / "scoreboard.json"
board_html = data / "arena" / "scoreboard.html"
aos_dash = data / "aos-poll" / "dashboard.json"
gate = data / "concept-gate" / "latest.json"

def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None

mind = load(mind_latest) or {}
board = load(board_json) or {}
aos = load(aos_dash) or {}
gate_p = load(gate) or {}

search_preview = ""
hits = []
if search_txt.is_file():
    raw = search_txt.read_text(encoding="utf-8", errors="replace")
    # Drop noisy boot logs; keep hit lines
    lines = []
    for line in raw.splitlines():
        if re.search(r"wiki/|\.md\)|Lint|No wiki", line):
            lines.append(line)
        elif line.startswith("[") and "wiki/" in line:
            lines.append(line)
    # Prefer lines that look like search hits
    hit_lines = [ln for ln in raw.splitlines() if "wiki/" in ln and ".md" in ln]
    hits = hit_lines[:8]
    search_preview = "\n".join(hits or lines[:12])[:1600]

wiki_pages = len(list((root / "wiki").rglob("*.md"))) if (root / "wiki").is_dir() else 0

payload = {
    "schema": "gzmo.unpark.wiki_observatory.demo/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "4.3",
    "ok": bool(
        mind.get("ok") is True
        and board_json.is_file()
        and board_html.is_file()
        and board.get("schema") == "gzmo.nightburst.scoreboard/v1"
    ),
    "seed_query": query,
    "wiki": {
        "pages": wiki_pages,
        "mind_verdict": mind.get("verdict"),
        "search_hits": len(hits),
        "search_path": str(search_txt) if search_txt.is_file() else None,
    },
    "scoreboard": {
        "json": str(board_json) if board_json.is_file() else None,
        "html": str(board_html) if board_html.is_file() else None,
        "wiki_sha": (board.get("wiki") or {}).get("commit_sha"),
        "wiki_healthy": (board.get("wiki") or {}).get("healthy"),
        "concepts_written": (board.get("wiki") or {}).get("concepts_written"),
        "arena_z": (board.get("arena") or {}).get("z"),
        "gate_verdict": (board.get("concept_gate") or {}).get("verdict")
        or gate_p.get("verdict"),
        "hsp_events": (board.get("hsp") or {}).get("events"),
        "euro_night_total": (board.get("euro_night") or {}).get("euro_night_total"),
        "okforge_observatory": (board.get("links") or {}).get("okforge_observatory"),
    },
    "aos_dashboard": {
        "path": str(aos_dash) if aos_dash.is_file() else None,
        "living": ((aos.get("living") or {}).get("verdict")),
        "arena_required": aos.get("arena_required"),
    },
    "daemon_jobs_touched": False,
    "wiki_push_applied": False,
    "advice": (
        "wiki_observatory_demo_ok — search + sanitized scoreboard; no push / no daemon wiki jobs"
        if mind.get("ok") and board_json.is_file()
        else "wiki_observatory_demo_hold — mind or scoreboard missing"
    ),
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")

# Copy a sanitized scoreboard excerpt next to felt for convenience
if board:
    pub = {
        "generated_at": board.get("generated_at"),
        "wiki": board.get("wiki"),
        "arena": {
            k: (board.get("arena") or {}).get(k)
            for k in ("champion", "z", "quality", "energy_source", "euro_cost")
        },
        "concept_gate": board.get("concept_gate"),
        "hsp": board.get("hsp"),
        "euro_night": board.get("euro_night"),
        "links": board.get("links"),
    }
    (out / "scoreboard-excerpt.json").write_text(json.dumps(pub, indent=2) + "\n")

lines = [
    "# Wiki / Observatory felt sample",
    "",
    f"Generated: {payload['generated_at']}",
    f"Verdict: {'OK' if payload['ok'] else 'HOLD'}",
    "",
    "Theater only — **not** living GREEN gate, **not** auto wiki push.",
    "",
    f"## Wiki search (`{query}`)",
    "",
    f"- pages tracked: {wiki_pages}",
    f"- mind verdict: `{payload['wiki']['mind_verdict']}`",
    f"- hits: {payload['wiki']['search_hits']}",
    "",
    "```",
    search_preview or "(no search preview)",
    "```",
    "",
    "## Sanitized scoreboard (Observatory-shaped)",
    "",
    f"- html: `{payload['scoreboard']['html']}`",
    f"- wiki sha / healthy / concepts: "
    f"`{payload['scoreboard']['wiki_sha']}` / "
    f"`{payload['scoreboard']['wiki_healthy']}` / "
    f"{payload['scoreboard']['concepts_written']}",
    f"- arena z: `{payload['scoreboard']['arena_z']}`",
    f"- concept gate: `{payload['scoreboard']['gate_verdict']}`",
    f"- HSP events: `{payload['scoreboard']['hsp_events']}`",
    f"- €/night: `{payload['scoreboard']['euro_night_total']}`",
    f"- OKForge Observatory: `{payload['scoreboard']['okforge_observatory']}`",
    "",
    "## Hard rules",
    "",
    "1. Scoreboard is sanitized — no tokens, no session bodies.",
    "2. `wiki-push-gated.sh` stays a separate operator path (gate PASS required).",
    "3. Never wire wiki-mind into living-readiness overnight GREEN.",
    "",
]
(out / "felt-latest.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print(json.dumps({"ok": payload["ok"], "advice": payload["advice"], "query": query}, indent=2))
PY

bash "$ROOT/scripts/wiki-observatory-check.sh"
echo "[OK] wiki-observatory demo → $OUT"
