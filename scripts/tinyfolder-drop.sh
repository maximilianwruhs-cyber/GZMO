#!/usr/bin/env bash
# tinyFolder-style drop → inbox → metabolism on-ramp (nightburst spike).
# Non-CLI users drop markdown into data-next/inbox; this stages + records.
#
#   bash scripts/tinyfolder-drop.sh --demo
#   bash scripts/tinyfolder-drop.sh /path/to/note.md
#   bash scripts/tinyfolder-drop.sh --scan   # process pending drops already in inbox
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
INBOX="${GZMO_INBOX:-$DATA/inbox}"
OUT="$DATA/tinyfolder"
MODE="file"
FILE=""
for a in "$@"; do
  case "$a" in
    --demo) MODE="demo" ;;
    --scan) MODE="scan" ;;
    *)
      if [[ -f "$a" ]]; then FILE="$a"; MODE="file"; fi
      ;;
  esac
done
mkdir -p "$INBOX" "$OUT" "$INBOX/processed"
export DATA INBOX OUT ROOT MODE FILE

python3 - <<'PY'
import json, os, shutil, uuid
from datetime import datetime, timezone
from pathlib import Path

inbox = Path(os.environ["INBOX"])
out = Path(os.environ["OUT"])
mode = os.environ.get("MODE", "file")
src = os.environ.get("FILE") or ""
now = datetime.now(timezone.utc)
stamp = now.strftime("%Y%m%dT%H%M%SZ")
day = now.strftime("%Y-%m-%d")


def write_drop(body: str, title: str) -> Path:
    name = f"drop-{stamp}-{uuid.uuid4().hex[:6]}.md"
    path = inbox / name
    fm = (
        "---\n"
        f"status: pending\n"
        f"action: ingest\n"
        f"source: tinyfolder\n"
        f"title: {title}\n"
        f"dropped_at: {now.isoformat()}\n"
        "---\n\n"
    )
    path.write_text(fm + body.strip() + "\n", encoding="utf-8")
    return path


staged = []
if mode == "demo":
    p = write_drop(
        "TinyFolder demo drop: remember that drop-folder notes should reach "
        "distill/ingest overnight without requiring the CLI chat surface.",
        "tinyfolder-demo",
    )
    staged.append(str(p))
elif mode == "file" and src:
    src_p = Path(src)
    body = src_p.read_text(encoding="utf-8")
    if not body.lstrip().startswith("---"):
        p = write_drop(body, src_p.stem)
    else:
        p = inbox / f"drop-{stamp}-{src_p.name}"
        shutil.copy2(src_p, p)
    staged.append(str(p))
elif mode == "scan":
    for p in sorted(inbox.glob("*.md")):
        if p.name.startswith("_"):
            continue
        text = p.read_text(encoding="utf-8")
        if "status: pending" in text or "status:pending" in text.replace(" ", ""):
            staged.append(str(p))
else:
    raise SystemExit("usage: tinyfolder-drop.sh --demo | --scan | <file.md>")

# Soft metabolism hook: append takeaway lines into a session-shaped queue file
# (serve distill / ingest-dir can pick up inbox; we only stage + advise).
queue = Path(os.environ["DATA"]) / "distill-queue"
queue.mkdir(parents=True, exist_ok=True)
qfile = queue / f"tinyfolder-{day}.jsonl"
for path in staged:
    row = {
        "ts": now.isoformat(),
        "source": "tinyfolder",
        "path": path,
        "advice": "Run `gzmo ingest-dir` or enable nightly ingest cron; then distill.",
    }
    with qfile.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row) + "\n")

payload = {
    "schema": "gzmo.tinyfolder.drop/v1",
    "generated_at": now.isoformat(),
    "ok": True,
    "inbox": str(inbox),
    "staged": staged,
    "count": len(staged),
    "queue": str(qfile),
    "next": [
        "gzmo ingest-dir  # if ingest CLI available",
        "gzmo distill     # metabolize after ingest/session",
        "bash scripts/cognition-pack.sh --smoke",
    ],
    "note": "Filesystem on-ramp spike — does not enable continuous watcher (ADR ingest off by default).",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# tinyFolder drop",
            "",
            f"Staged: {len(staged)} → `{inbox}`",
            *[f"- `{p}`" for p in staged],
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "staged": staged, "queue": str(qfile)}, indent=2))
PY
