#!/usr/bin/env bash
# tinyFolder-style drop → inbox → metabolism on-ramp (Brain Feed P0).
# Non-CLI users drop markdown into data-next/inbox; this stages + records.
# --living: write living-enqueue.json aimed at CT101 distill (never starts
# a workstation overnight writer; refuses if gzmo-serve is active).
# --apply-takeaway: with --living, SSH session close --takeaway on living host
# (enqueue only, no --now). Also: TINYFOLDER_APPLY_TAKEAWAY=1
#
#   bash scripts/tinyfolder-drop.sh --demo
#   bash scripts/tinyfolder-drop.sh --demo --living
#   bash scripts/tinyfolder-drop.sh --demo --living --apply-takeaway
#   bash scripts/tinyfolder-drop.sh /path/to/note.md --living --apply-takeaway
#   bash scripts/tinyfolder-drop.sh --scan
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
INBOX="${GZMO_INBOX:-${TINYFOLDER_INBOX:-$DATA/inbox}}"
OUT="$DATA/tinyfolder"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
MODE="file"
FILE=""
LIVING=0
APPLY=0
if [[ "${TINYFOLDER_APPLY_TAKEAWAY:-0}" == "1" ]]; then
  APPLY=1
fi
for a in "$@"; do
  case "$a" in
    --demo) MODE="demo" ;;
    --scan) MODE="scan" ;;
    --living) LIVING=1 ;;
    --apply-takeaway) APPLY=1; LIVING=1 ;;
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
export DATA INBOX OUT ROOT MODE FILE LIVING APPLY HOST GZMO_BIN

python3 - <<'PY'
import json, os, shutil, subprocess, uuid
from datetime import datetime, timezone
from pathlib import Path

inbox = Path(os.environ["INBOX"])
out = Path(os.environ["OUT"])
mode = os.environ.get("MODE", "file")
src = os.environ.get("FILE") or ""
living = os.environ.get("LIVING", "0") == "1"
apply = os.environ.get("APPLY", "0") == "1"
host = os.environ.get("HOST", "ct101")
gzmo_bin = os.environ.get("GZMO_BIN", "/opt/gzmo/current/target/release/gzmo")
now = datetime.now(timezone.utc)
stamp = now.strftime("%Y%m%dT%H%M%SZ")
day = now.strftime("%Y-%m-%d")


def write_drop(body: str, title: str, stable_name: str = None) -> Path:
    if stable_name:
        name = stable_name  # idempotent: overwrite in place, never accumulate
    else:
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
    # Stable name: the Brain Feed gate calls this every 30 min — a fresh
    # timestamped file per run is how 571 identical demo drops accumulated
    # in 30 days. One file, overwritten in place.
    p = write_drop(
        "TinyFolder demo drop: remember that drop-folder notes should reach "
        "distill/ingest overnight without requiring the CLI chat surface.",
        "tinyfolder-demo",
        stable_name="drop-tinyfolder-demo.md",
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
    for path in sorted(inbox.glob("*.md")):
        if path.name.startswith("_") or path.name == "README.md":
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        if "status: pending" in text or "status:pending" in text.replace(" ", ""):
            staged.append(str(path))
else:
    raise SystemExit(
        "usage: tinyfolder-drop.sh --demo | --scan | <file.md> "
        "[--living] [--apply-takeaway]"
    )

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

applied = []
apply_error = None
if apply and living and takeaway_lines and not dual_writer:
    sid = f"tinyfolder-apply-{uuid.uuid4().hex[:8]}"
    remote_sess = f"/opt/gzmo/data/sessions/{sid}.json"
    now_iso = now.strftime("%Y-%m-%dT%H:%M:%SZ")
    sess = {
        "id": sid,
        "name": "tinyfolder_living_apply",
        "created_at": now_iso,
        "last_active_at": now_iso,
        "messages": [
            {"role": "user", "content": "TinyFolder living takeaway enqueue.", "is_meta": False},
            {"role": "assistant", "content": "Recording drop-folder note as takeaway.", "is_meta": False},
        ],
    }
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, f"cat > {remote_sess}"],
        input=json.dumps(sess),
        text=True,
        capture_output=True,
    )
    if p.returncode != 0:
        apply_error = f"seed_session:{(p.stderr or p.stdout or '')[:200]}"
    else:
        combined = " | ".join(takeaway_lines[:3])
        cmd = (
            f"bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml "
            f"{gzmo_bin} session close {sid} --takeaway {json.dumps(combined)}'"
        )
        p2 = subprocess.run(
            ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
            capture_output=True,
            text=True,
        )
        if p2.returncode == 0:
            applied.append({"session_id": sid, "takeaway": combined, "distill": "enqueue_only"})
        else:
            apply_error = f"session_close:{(p2.stderr or p2.stdout or '')[:300]}"
elif apply and dual_writer:
    apply_error = "refused_dual_writer — stop gzmo-serve before applying to living host"
elif apply and not takeaway_lines:
    apply_error = "no_takeaways"
elif apply and not living:
    apply_error = "apply_requires_living"

living_path = out / "living-enqueue.json"
if living:
    if dual_writer:
        advice = "tinyfolder_living_refused_dual_writer"
    elif apply and applied:
        advice = "tinyfolder_living_takeaway_applied — enqueued on living host (no --now)"
    elif apply and apply_error:
        advice = f"tinyfolder_living_apply_failed — {apply_error}"
    else:
        advice = "tinyfolder_living_enqueue_ready — takeaway/ingest on living host only"
    living_ok = (not dual_writer) and bool(staged) and (apply_error is None if apply else True)
    living_payload = {
        "schema": "gzmo.brain_feed.tinyfolder_living/v1",
        "generated_at": now.isoformat(),
        "ok": living_ok,
        "dual_writer": dual_writer,
        "staged": staged,
        "living_distill_queue": "gzmo:distill:pending",
        "proposed_takeaways": takeaway_lines,
        "apply_takeaway": apply,
        "applied": applied,
        "apply_error": apply_error,
        "advice": advice,
        "operator": [
            "Dry-run: bash scripts/tinyfolder-drop.sh --demo --living",
            "Apply: bash scripts/tinyfolder-drop.sh --demo --living --apply-takeaway",
            "Or: TINYFOLDER_APPLY_TAKEAWAY=1 bash scripts/tinyfolder-drop.sh --living <file.md>",
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

# Living on-ramp: copy staged real drops to CT101 inbox (idempotent), then move
# the local copy to processed/ — CT101 owns metabolism from there. Demo mode skips.
sync_living = os.environ.get("TINYFOLDER_SYNC_LIVING", "1") == "1"
living_inbox = os.environ.get("CT101_TINYFOLDER_INBOX", "/opt/gzmo/data/inbox")
synced, sync_errors = [], []
if sync_living and mode in ("file", "scan"):
    for path in staged:
        p = Path(path)
        try:
            body_text = p.read_text(encoding="utf-8")
        except Exception as e:
            sync_errors.append(f"{p.name}:read:{e}")
            continue
        if "title: tinyfolder-demo" in body_text:
            continue  # gate warm-up spam never feeds the living host
        cmd = f"test -e {living_inbox}/{p.name} || cat > {living_inbox}/{p.name}"
        try:
            r = subprocess.run(
                ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
                input=body_text,
                text=True,
                capture_output=True,
                timeout=30,
            )
            if r.returncode == 0:
                (inbox / "processed" / p.name).parent.mkdir(parents=True, exist_ok=True)
                p.rename(inbox / "processed" / p.name)
                synced.append(str(p))
            else:
                sync_errors.append(f"{p.name}:{(r.stderr or r.stdout or '')[:120]}")
        except Exception as e:  # keep the local pending file for retry
            sync_errors.append(f"{p.name}:{e}")

ok = False if (living and dual_writer) else True
if apply and apply_error:
    ok = False

payload = {
    "schema": "gzmo.tinyfolder.drop/v1",
    "generated_at": now.isoformat(),
    "ok": ok,
    "inbox": str(inbox),
    "staged": staged,
    "count": len(staged),
    "synced_to_living": synced,
    "sync_errors": sync_errors,
    "queue": str(qfile),
    "living": living,
    "apply_takeaway": apply,
    "applied": applied,
    "apply_error": apply_error,
    "living_enqueue": str(living_path) if living else None,
    "dual_writer": dual_writer,
    "next": [
        "bash scripts/tinyfolder-drop.sh --demo --living",
        "bash scripts/tinyfolder-drop.sh --demo --living --apply-takeaway",
        "bash scripts/brain-feed-check.sh",
    ],
    "note": "Filesystem on-ramp — --living/--apply-takeaway never starts overnight writer here.",
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
            f"Apply takeaway: {'yes' if apply else 'no'}",
            f"Applied: {len(applied)}",
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
    "apply_takeaway": apply,
    "applied": applied,
    "apply_error": apply_error,
    "living_enqueue": payload.get("living_enqueue"),
}, indent=2))
raise SystemExit(0 if payload["ok"] else 1)
PY
