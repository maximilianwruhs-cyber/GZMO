#!/usr/bin/env bash
# Unpark Wave 2.1 demable: inventory thin pantheon skills + archive + re-land checklist.
# Does not merge feat attractor stack; never invents DICE_MASTER_*.
#
#   bash scripts/pantheon-ritual-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/pantheon-ritual"
mkdir -p "$OUT"

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
    "advice": "pantheon_ritual_demo_ok — Slice A inventory; C.1 + daemon dice_loop fire deferred",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY

bash "$ROOT/scripts/pantheon-ritual-check.sh"
echo "[OK] pantheon ritual demo → $OUT"
