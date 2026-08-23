#!/usr/bin/env bash
# lib-research-dedup.sh — cross-day URL dedup helpers for the morning-research
# pipelines (research-intel.sh, research-sota.sh).
#
# Defines ONLY functions. Safe to source under `set -euo pipefail`. All JSON
# work happens inside embedded python3 heredocs (no network, no external deps).
#
# Usage (after the stage-2b heredoc that wrote the timestamped archive):
#     source "$ROOT/scripts/lib-research-dedup.sh"
#     cp "$OUT/research-<name>-$stamp.json" "$OUT/latest.json"
#     dedup_findings      "$OUT/latest.json" "$OUT/seen.jsonl"
#     dedup_render_latest "$OUT/latest.json" "$OUT/latest.md" <intel|sota> "$stamp" "$TOP_N"
#     dedup_seen_update   "$OUT/latest.json" "$OUT/seen.jsonl" "$(date -u +%Y-%m-%d)"

# dedup_findings <findings-json> <seen-jsonl>
#   seen-jsonl: one JSON object per line {"url":...,"first_seen":"YYYY-MM-DD"};
#   missing file is treated as empty.
#   Annotates every finding in findings-json["findings"] in place with
#   "repeat" (true iff url already in seen) and "first_seen" (repeats only),
#   adds top-level "new_count"/"repeat_count", rewrites the file in place
#   (indent=2, trailing newline), prints exactly {"new":N,"repeat":M}.
#   Does NOT modify the seen file.
dedup_findings() {
  local findings_json="$1" seen_jsonl="$2"
  python3 - "$findings_json" "$seen_jsonl" <<'PY'
import json, sys
from pathlib import Path

fj = Path(sys.argv[1])
sj = Path(sys.argv[2])

data = json.loads(fj.read_text())

seen = {}
if sj.exists():
    for line in sj.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            o = json.loads(line)
        except Exception:
            continue
        if isinstance(o, dict) and "url" in o:
            seen[o["url"]] = o.get("first_seen", "")

findings = data.get("findings") or []
new_count = 0
repeat_count = 0
for f in findings:
    u = f.get("url", "")
    if u in seen:
        f["repeat"] = True
        f["first_seen"] = seen[u]
        repeat_count += 1
    else:
        f["repeat"] = False
        new_count += 1

data["new_count"] = new_count
data["repeat_count"] = repeat_count

fj.write_text(json.dumps(data, indent=2) + "\n")
print(json.dumps({"new": new_count, "repeat": repeat_count}, separators=(",", ":")))
PY
}

# dedup_render_latest <findings-json> <md-out> <format> <stamp> <top_n>
#   format is "intel" or "sota". Reproduces the EXACT existing latest.md
#   structure for that format (header, Top findings, All findings, and the
#   Synthese-Tabelle for sota), with dedup changes:
#     - header findings line: findings: N (new: A, repeat: B)
#     - "## Top findings": only NEW findings (max top_n); if ZERO new, show the
#       top-1 repeat finding with its title suffixed " [repeat, first seen YYYY-MM-DD]"
#     - every "## All findings" bullet: new -> " (new)",
#       repeat -> " (repeat, first seen YYYY-MM-DD)"
#   Writes <md-out> (no trailing newline, matching the heredoc render).
dedup_render_latest() {
  local findings_json="$1" md_out="$2" format="$3" stamp="$4" top_n="$5"
  python3 - "$findings_json" "$md_out" "$format" "$stamp" "$top_n" <<'PY'
import json, sys
from pathlib import Path

data = json.loads(Path(sys.argv[1]).read_text())
md_out = Path(sys.argv[2])
fmt = sys.argv[3]
stamp = sys.argv[4]
top_n = int(sys.argv[5])

queries = data.get("queries") or []
findings = data.get("findings") or []
new_count = data.get("new_count", sum(1 for f in findings if not f.get("repeat")))
repeat_count = data.get("repeat_count", sum(1 for f in findings if f.get("repeat")))

md = [f"# research-{fmt} — {stamp}", "",
      f"queries: {json.dumps(queries)}", "",
      f"findings: {len(findings)} (new: {new_count}, repeat: {repeat_count})", ""]

if fmt == "sota":
    md += ["## Synthese-Tabelle", "",
           "| Baugruppe | SOTA (Quelle/ID) | TRL | Konventioneller Standard | Integration-Hebel |",
           "|---|---|---|---|---|"]
    for it in findings:
        bg = it.get("baugruppe") or "-"
        src = f"{it['source']}/{it['id']}"[:60]
        trl = str(it.get("trl") or "-")
        conv = (it.get("conventional_standard") or "-").replace("|", "/")[:60]
        lev = (it.get("integration_lever") or "-").replace("|", "/")[:80]
        md.append(f"| {bg} | {src} | {trl} | {conv} | {lev} |")
    md.append("")

md.append("## Top findings")
new_findings = [f for f in findings if not f.get("repeat")]
if new_findings:
    top = new_findings[:top_n]
else:
    top = [f for f in findings if f.get("repeat")][:1]
for t in top:
    title = t["title"]
    if t.get("repeat"):
        title = f"{title} [repeat, first seen {t.get('first_seen', '?')}]"
    md.append(f"### {title}")
    if fmt == "sota":
        md.append(f"- baugruppe: {t.get('baugruppe', '-')} · TRL: {t.get('trl', '-')} · benefit: {t['benefit']}")
        md.append(f"- source: {t['source']} · published: {t.get('published', '?')}")
    else:
        md.append(f"- source: {t['source']} · published: {t.get('published', '?')} · benefit: {t['benefit']}")
    md.append(f"- url: {t['url']}")
    if fmt == "sota":
        if t.get("conventional_standard"):
            md.append(f"- konventioneller Standard: {t['conventional_standard']}")
    if t.get("why"):
        md.append(f"- why: {t['why']}")
    if fmt == "sota":
        if t.get("integration_lever"):
            md.append(f"- integration-hebel: {t['integration_lever']}")
    else:
        if t.get("integration_point"):
            md.append(f"- integration: {t['integration_point']}")
    md.append("")

md.append("## All findings")
for f in findings:
    line = f"- [{f['source']}] {f['title']} — {f['url']} (benefit={f['benefit']}"
    if fmt == "sota":
        line += f", TRL={f.get('trl', '-')}"
    line += ")"
    if f.get("repeat"):
        line += f" (repeat, first seen {f.get('first_seen', '?')})"
    else:
        line += " (new)"
    md.append(line)

md_out.write_text("\n".join(md), encoding="utf-8")
PY
}

# dedup_seen_update <findings-json> <seen-jsonl> <today>
#   Appends {"url","first_seen":today} for every finding with repeat==false
#   whose url is not already in the seen file. Prunes entries with first_seen
#   older than 30 days before <today>. Rewrites seen-jsonl (creates if missing).
#   Malformed lines are dropped. Idempotent for a given <today>.
dedup_seen_update() {
  local findings_json="$1" seen_jsonl="$2" today="$3"
  python3 - "$findings_json" "$seen_jsonl" "$today" <<'PY'
import json, sys
from datetime import datetime, timedelta
from pathlib import Path

data = json.loads(Path(sys.argv[1]).read_text())
sj = Path(sys.argv[2])
today = sys.argv[3]
today_d = datetime.strptime(today, "%Y-%m-%d").date()
cutoff = today_d - timedelta(days=30)

entries = {}
order = []
if sj.exists():
    for line in sj.read_text().splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            o = json.loads(line)
        except Exception:
            continue
        if not isinstance(o, dict) or "url" not in o or "first_seen" not in o:
            continue
        u = o["url"]
        fs = o["first_seen"]
        try:
            fs_d = datetime.strptime(fs, "%Y-%m-%d").date()
        except Exception:
            continue
        if fs_d < cutoff:
            continue
        if u not in entries:
            entries[u] = fs
            order.append(u)

for f in data.get("findings") or []:
    if f.get("repeat"):
        continue
    u = f.get("url", "")
    if not u or u in entries:
        continue
    entries[u] = today
    order.append(u)

sj.write_text("".join(json.dumps({"url": u, "first_seen": entries[u]}) + "\n" for u in order))
PY
}
