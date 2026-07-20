#!/usr/bin/env bash
# tinyFolder-style drop → inbox → metabolism on-ramp (Brain Feed P0).
# Non-CLI users drop markdown into data-next/inbox; this stages + records.
# --living: also write living-enqueue.json aimed at CT101 distill (never starts
# a workstation overnight writer; refuses if gzmo-serve is active).
#
#   bash scripts/tinyfolder-drop.sh --demo
#   bash scripts/tinyfolder-drop.sh --demo --living
#   bash scripts/tinyfolder-drop.sh /path/to/note.md
#   bash scripts/tinyfolder-drop.sh --scan   # process pending drops already in inbox
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
INBOX="${GZMO_INBOX:-${TINYFOLDER_INBOX:-$DATA/inbox}}"
OUT="$DATA/tinyfolder"
MODE="file"
FILE=""
LIVING=0
for a in "$@"; do
  case "$a" in
    --demo) MODE="demo" ;;
    --scan) MODE="scan" ;;
    --living) LIVING=1 ;;
    *)
      if [[ -f "$a" ]]; then FILE="$a"; MODE="file"; fi
      ;;
  esac
done
mkdir -p "$INBOX" "$OUT" "$INBOX/processed"
# Keep check inbox in sync when using default paths
if [[ -z "${TINYFOLDER_INBOX:-}" ]]; then
  mkdir -p "$DATA/tinyfolder-inbox"
fi
export DATA INBOX OUT ROOT MODE FILE LIVING

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

# Soft metabolism hook + optional Brain Feed living-enqueue artifact.
import subprocess

queue = Path(os.environ["DATA"]) / "distill-queue"
queue.mkdir(parents=True, exist_ok=True)
qfile = queue / f"tinyfolder-{day}.jsonl"
for path in staged:
    row = {
        "ts": now.isoformat(),
        "source": "tinyfolder",
        "path": path,
        "advice": "Run `gzmo ingest-dir` or enable nightly ingest cron; then distill.",
        "living_queue": "gzmo:distill:pending",
    }
    with qfile.open("a", encoding="utf-8") as fh:
        fh.write(json.dumps(row) + "\n")

living = os.environ.get("LIVING", "0") == "1"
dual_writer = False
try:
    r = subprocess.run(
        ["systemctl", "--user", "is-active", "gzmo-serve.service"],
        capture_output=True, text=True, timeout=5,
    )
    if (r.stdout or "").strip() == "active":
        dual_writer = True
except Exception:
    pass

living_path = out / "living-enqueue.json"
if living:
    takeaway_lines = []
    for path in staged:
        try:
            body = Path(path).read_text(encoding="utf-8", errors="replace")
            if body.lstrip().startswith("---"):
                parts = body.split("---", 2)
                body = parts[2] if len(parts) > 2 else body
            line = " ".join(body.strip().split())[:280]
            if line:
                takeaway_lines.append(f"TinyFolderDrop: {line}")
        except Exception:
            pass
    living_payload = {
        "schema": "gzmo.brain_feed.tinyfolder_living/v1",
        "generated_at": now.isoformat(),
        "ok": (not dual_writer) and bool(staged),
        "dual_writer": dual_writer,
        "staged": staged,
        "living_distill_queue": "gzmo:distill:pending",
        "proposed_takeaways": takeaway_lines,
        "advice": (
            "tinyfolder_living_refused_dual_writer"
            if dual_writer
            else "tinyfolder_living_enqueue_ready — takeaway/ingest on living host only"
        ),
        "operator": [
            "Copy staged markdown to living host inbox OR",
            "ssh living: gzmo session close <sid> --takeaway 'TinyFolderDrop: …'",
            "Never start workstation gzmo serve while CT101 lives",
        ],
        "blocks_overnight_on_workstation": True,
    }
    living_path.write_text(json.dumps(living_payload, indent=2) + "\n", encoding="utf-8")
    mirror = Path(os.environ["DATA"]) / "tinyfolder-inbox"
    mirror.mkdir(parents=True, exist_ok=True)
    for path in staged:
        src_p = Path(path)
        dest = mirror / src_p.name
        if not dest.exists():
            try:
                dest.write_text(src_p.read_text(encoding="utf-8"), encoding="utf-8")
            except Exception:
                pass

payload = {
    "schema": "gzmo.tinyfolder.drop/v1",
    "generated_at": now.isoformat(),
    "ok": False if (living and dual_writer) else True,
    "inbox": str(inbox),
    "staged": staged,
    "count": len(staged),
    "queue": str(qfile),
    "living": living,
    "living_enqueue": str(living_path) if living else None,
    "dual_writer": dual_writer,
    "next": [
        "bash scripts/tinyfolder-drop.sh --demo --living",
        "bash scripts/brain-feed-check.sh",
    ],
    "note": "Filesystem on-ramp — --living never starts overnight writer here.",
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
            f"Living enqueue: {'yes' if living else 'no'}",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({
    "ok": payload["ok"],
    "staged": staged,
    "queue": str(qfile),
    "living": living,
    "living_enqueue": payload.get("living_enqueue"),
}, indent=2))
raise SystemExit(0 if payload["ok"] else 1)
PY
