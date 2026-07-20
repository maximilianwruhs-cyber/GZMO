#!/usr/bin/env bash
# Unpark Wave 2.2 demable: session-prep + LINK pack dry-run (theater ≠ living scout KPI).
#   bash scripts/discovery-theater-demo.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/discovery-theater"
ARCHIVE="$ROOT/docs/research/mutual-discovery"
mkdir -p "$OUT"

cat >"$OUT/session-prep.md" <<'EOF'
# Mutual-discovery theater — session prep (Unpark Wave 2.2)

Theater is human pedagogy. Do **not** redefine living publish/timer KPI.

1. Open [MUTUAL_DISCOVERY_THEATER.md](../../docs/MUTUAL_DISCOVERY_THEATER.md)
2. Keep Forum-1 / scout path under [DISCOVERY_LIFECYCLE.md](../../docs/DISCOVERY_LIFECYCLE.md)
3. Confirm `living-readiness-gate` has no theater rows
4. Dry-run the Socratic LINK pack below (scored checklist in `link-dry-run.json`)
EOF

export OUT ARCHIVE ROOT
python3 - <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
archive = Path(os.environ["ARCHIVE"])
root = Path(os.environ["ROOT"])

# LINK pack dry-run: required research docs + scored checklist items
required = [
    "SOCRATIC_FORUM_THREE_MODES.md",
    "PI_GZMO_SOCRATIC_KNOWLEDGE_DIALOGUE.md",
    "PI_MUTUAL_DISCOVERY_VERIFIED_FINDINGS.md",
]
checklist = [
    {
        "id": "forum_modes_doc",
        "label": "Forum three-modes doc present",
        "path": "SOCRATIC_FORUM_THREE_MODES.md",
        "must_contain": ["Mode A", "Modes B/C"],
    },
    {
        "id": "socratic_dialogue_doc",
        "label": "Socratic knowledge dialogue handoff present",
        "path": "PI_GZMO_SOCRATIC_KNOWLEDGE_DIALOGUE.md",
        "must_contain": ["Socratic", "pedagogy"],
    },
    {
        "id": "verified_findings_doc",
        "label": "Verified findings present",
        "path": "PI_MUTUAL_DISCOVERY_VERIFIED_FINDINGS.md",
        "must_contain": ["verified", "mentor"],
    },
    {
        "id": "theater_boundary_doc",
        "label": "Front-door theater doc exists",
        "path": None,
        "abs": root / "docs" / "MUTUAL_DISCOVERY_THEATER.md",
        "must_contain": ["theater", "living"],
    },
    {
        "id": "not_living_kpi",
        "label": "Living readiness gate has no theater rows",
        "path": None,
        "abs": root / "scripts" / "living-readiness-gate.sh",
        "must_not_contain": ["MUTUAL_DISCOVERY", "mutual-discovery"],
    },
]

items = []
score = 0
for c in checklist:
    p = Path(c["abs"]) if c.get("abs") else archive / c["path"]
    exists = p.is_file()
    text = p.read_text(encoding="utf-8", errors="ignore") if exists else ""
    low = text.lower()
    ok = exists
    detail = "missing file"
    if exists:
        missing = [t for t in (c.get("must_contain") or []) if t.lower() not in low]
        banned = [t for t in (c.get("must_not_contain") or []) if t in text]
        if missing:
            ok = False
            detail = f"missing phrases: {missing}"
        elif banned:
            ok = False
            detail = f"forbidden living-KPI refs: {banned}"
        else:
            detail = "ok"
            score += 1
    items.append(
        {
            "id": c["id"],
            "label": c["label"],
            "ok": ok,
            "detail": detail,
            "path": str(p.relative_to(root)) if exists or c.get("path") else str(p),
        }
    )

total = len(checklist)
pct = round(100.0 * score / total, 1) if total else 0.0
dry = {
    "schema": "gzmo.unpark.discovery_theater.link_dry_run/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "2.2",
    "pack": "socratic-link",
    "archive": str(archive.relative_to(root)),
    "required_docs": required,
    "score": score,
    "total": total,
    "pass_pct": pct,
    "ok": score == total,
    "items": items,
    "note": "Theater dry-run only — not a living readiness KPI row",
}
(out / "link-dry-run.json").write_text(json.dumps(dry, indent=2) + "\n")
(out / "link-dry-run.md").write_text(
    "\n".join(
        [
            "# Socratic LINK pack dry-run",
            "",
            f"Score: **{score}/{total}** ({pct}%)",
            "",
            "| id | ok | detail |",
            "|----|----|--------|",
            *[
                f"| {i['id']} | {'✓' if i['ok'] else '✗'} | {i['detail']} |"
                for i in items
            ],
            "",
            dry["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": dry["ok"], "score": f"{score}/{total}", "pass_pct": pct}, indent=2))
if score < total:
    raise SystemExit(f"LINK dry-run incomplete: {score}/{total}")
PY

bash "$ROOT/scripts/discovery-theater-check.sh"
python3 - <<PY
import json
from datetime import datetime, timezone
from pathlib import Path
out = Path("$OUT")
payload = {
  "schema": "gzmo.unpark.discovery_theater.demo/v1",
  "generated_at": datetime.now(timezone.utc).isoformat(),
  "ok": True,
  "session_prep": str(out / "session-prep.md"),
  "link_dry_run": str(out / "link-dry-run.json"),
  "wave": "2.2",
}
(out / "demo.json").write_text(json.dumps(payload, indent=2) + "\n")
print(json.dumps(payload, indent=2))
PY
echo "[OK] discovery theater demo → $OUT"
