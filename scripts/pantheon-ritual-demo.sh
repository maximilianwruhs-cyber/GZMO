#!/usr/bin/env bash
# Unpark Wave 2.1 demable: inventory thin pantheon skills + felt dice/card/story + re-land checklist.
# Does not merge feat attractor stack; never invents DICE_MASTER_*.
# Never starts PulseLoop / living daemon.
#
#   bash scripts/pantheon-ritual-demo.sh
#   bash scripts/pantheon-ritual-demo.sh --skip-felt
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/pantheon-ritual"
mkdir -p "$OUT"

SKIP_FELT=0
for arg in "$@"; do
  case "$arg" in
    --skip-felt) SKIP_FELT=1 ;;
    -h|--help)
      echo "Usage: $0 [--skip-felt]"
      exit 0
      ;;
  esac
done

export ROOT OUT
python3 - <<'PY'
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__import__("os").environ["ROOT"])
out = Path(__import__("os").environ["OUT"])
skills = root / "gzmo-core" / "src" / "skills"
archive = root / "docs" / "research" / "pantheon"

thin = {}
for name in ("dice.rs", "card.rs", "story.rs"):
    p = skills / name
    thin[name] = {"present": p.is_file(), "bytes": p.stat().st_size if p.is_file() else 0}

archives = sorted([p.name for p in archive.glob("*.md")]) if archive.is_dir() else []
feat_hits = []
for pat in ("dice_loop.rs", "attractor_common.rs", "card_forge.rs", "dice_corpus.rs"):
    hits = list(skills.rglob(pat))
    if hits:
        feat_hits.append(str(hits[0].relative_to(root)))

reland = out / "RELAND_CHECKLIST.md"
reland.write_text(
    """# Pantheon feat re-land checklist (Unpark Wave 2)

Separate ritual PR only — do not invent ghost `DICE_MASTER_*`.

1. [x] Slice A full on main (dispatch / nested cascade / card_forge) — verify with pantheon-ritual-check
2. [ ] Keep chaos off CT101 living KPI ([CHAOS_LIVING_VS_RITUAL.md](../../docs/CHAOS_LIVING_VS_RITUAL.md))
3. [ ] `bash scripts/pantheon-ritual-check.sh` → prefer feat-stack PASS
4. [ ] Living faithfulness + takeaway-recall still PASS
5. [ ] Skills bridge docs updated
6. [ ] **Hold:** Slice C.1 pedagogy oscillator — lab-only; never `daemon_cmd` / living PulseLoop

Thin main stubs (`dice`/`card`/`story`) remain the installable ritual surface.
Daemon `dice_loop` fire stays unwired by design.
Front door theater: [PANTHEON_DEMO.md](../../docs/PANTHEON_DEMO.md).
""",
    encoding="utf-8",
)

payload = {
    "schema": "gzmo.unpark.pantheon.demo/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "2.1",
    "ok": True,
    "thin_skills": thin,
    "archives": archives,
    "feat_hits_on_main": feat_hits,
    "reland_checklist": str(reland),
    "c1_deferred": True,
    "daemon_dice_loop_fire": False,
    "felt_demo": True,
    "advice": "pantheon_ritual_demo_ok — Slice A inventory + felt; C.1 + daemon dice_loop fire deferred",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY

if [[ "$SKIP_FELT" -eq 0 ]]; then
  echo "[felt] rolling dice / card / story via chaos_skill.sh …"
  CHAOS_SH="$ROOT/scripts/pi/chaos_skill.sh"
  FELT_OK=1
  declare -a FELT_ROWS=()
  run_felt() {
    local name="$1"; shift
    local raw="$OUT/felt-${name}.json"
    if bash "$CHAOS_SH" "$@" >"$raw" 2>"$OUT/felt-${name}.err"; then
      FELT_ROWS+=("$name|ok|$raw")
      echo "[felt] $name OK → $raw"
    else
      FELT_OK=0
      FELT_ROWS+=("$name|fail|$raw")
      echo "[felt] $name FAIL (see $OUT/felt-${name}.err)" >&2
    fi
  }
  run_felt dice dice d20 --json
  run_felt card card --json
  run_felt story story --json

  export OUT FELT_OK
  FELT_ROWS_TSV="$(printf '%s\n' "${FELT_ROWS[@]}")"
  export FELT_ROWS_TSV
  python3 - <<'PY'
import json, os, re
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
rows = []
for line in os.environ.get("FELT_ROWS_TSV", "").splitlines():
    if not line.strip():
        continue
    name, status, path = line.split("|", 2)
    entry = {"name": name, "status": status, "path": path}
    p = Path(path)
    if p.is_file() and p.stat().st_size > 0:
        try:
            d = json.loads(p.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            d = {"raw_text": p.read_text(encoding="utf-8", errors="replace")[:2000]}
        display = (
            d.get("display_plain")
            or d.get("display")
            or ""
        )
        # strip ANSI for markdown preview
        plain = re.sub(r"\x1b\[[0-9;]*m", "", display)
        entry.update(
            {
                "skill": d.get("skill"),
                "ok": d.get("ok"),
                "tier": d.get("tier"),
                "band": d.get("band"),
                "display_chars": d.get("display_chars") or len(plain),
                "preview": plain.strip()[:900],
            }
        )
    rows.append(entry)

felt_ok = os.environ.get("FELT_OK") == "1" and all(r.get("status") == "ok" for r in rows)
payload = {
    "schema": "gzmo.unpark.pantheon.felt/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": felt_ok,
    "samples": rows,
    "advice": "pantheon_felt_ok" if felt_ok else "pantheon_felt_partial — check felt-*.err / GZMO_BIN",
}
(out / "felt-latest.json").write_text(json.dumps(payload, indent=2) + "\n")

lines = [
    "# Pantheon felt sample",
    "",
    f"Generated: {payload['generated_at']}",
    f"Verdict: {'OK' if felt_ok else 'PARTIAL'}",
    "",
    "Ritual one-shots via `scripts/pi/chaos_skill.sh` — not living KPI.",
    "",
]
for r in rows:
    lines.append(f"## {r['name']} (`{r.get('skill') or '?'}`)")
    lines.append("")
    lines.append(f"- status: {r['status']}")
    if r.get("tier"):
        lines.append(f"- tier/band: {r.get('tier')} / {r.get('band')}")
    lines.append(f"- chars: {r.get('display_chars', 0)}")
    lines.append("")
    preview = r.get("preview") or "(no display)"
    lines.append("```")
    lines.append(preview)
    lines.append("```")
    lines.append("")
(out / "felt-latest.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print(json.dumps({"felt_ok": felt_ok, "samples": [r["name"] for r in rows]}, indent=2))
PY
else
  echo "[felt] skipped (--skip-felt)"
fi

bash "$ROOT/scripts/pantheon-ritual-check.sh"
echo "[OK] pantheon ritual demo → $OUT"
