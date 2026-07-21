#!/usr/bin/env bash
# herdr close-ritual → living host takeaway enqueue (Brain Feed / Unpark s1).
# Demable SSH mirror of integrations/herdr-gzmo-metabolism session-close —
# no --now, dual-writer refuse, no memory-gym Cursor chat.
#
#   bash scripts/herdr-living-enqueue.sh
#   TAKEAWAY='…' bash scripts/herdr-living-enqueue.sh
#   HERDR_LIVING_ENQUEUE=1 bash integrations/herdr-gzmo-metabolism/scripts/session-close.sh --living --takeaway '…'
#
# Artifact: data-next/herdr-metabolism/living-enqueue.json
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/herdr-metabolism"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
TAKEAWAY="${TAKEAWAY:-}"
mkdir -p "$OUT"

if [[ -z "$TAKEAWAY" ]]; then
  TAKEAWAY="HerdrLivingEnqueue-$(date -u +%Y%m%dT%H%M%SZ)-$$: herdr close-ritual → living distill (no --now)"
fi

export OUT HOST GZMO_BIN TAKEAWAY ROOT
python3 - <<'PY'
import json, os, subprocess, uuid
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
host = os.environ["HOST"]
gzmo_bin = os.environ["GZMO_BIN"]
takeaway = os.environ["TAKEAWAY"].strip()
now = datetime.now(timezone.utc)
now_iso = now.strftime("%Y-%m-%dT%H:%M:%SZ")

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

applied = []
apply_error = None
session_has_takeaway = False
remote_session = None
sid = f"herdr-living-{uuid.uuid4().hex[:8]}"

if dual_writer:
    apply_error = "refused_dual_writer — stop workstation gzmo-serve before living enqueue"
elif not takeaway:
    apply_error = "empty_takeaway"
else:
    remote_session = f"/opt/gzmo/data/sessions/{sid}.json"
    sess = {
        "id": sid,
        "name": "herdr_living_enqueue",
        "created_at": now_iso,
        "last_active_at": now_iso,
        "messages": [
            {"role": "user", "content": "Herdr close-ritual living enqueue.", "is_meta": False},
            {"role": "assistant", "content": "Recording durable takeaway on living host.", "is_meta": False},
        ],
    }
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, f"cat > {remote_session}"],
        input=json.dumps(sess),
        text=True,
        capture_output=True,
    )
    if p.returncode != 0:
        apply_error = f"seed_session:{(p.stderr or p.stdout or '')[:200]}"
    else:
        cmd = (
            f"bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml "
            f"{gzmo_bin} session close {sid} --takeaway {json.dumps(takeaway)}'"
        )
        p2 = subprocess.run(
            ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
            capture_output=True,
            text=True,
        )
        if p2.returncode != 0:
            apply_error = f"session_close:{(p2.stderr or p2.stdout or '')[:300]}"
        else:
            applied.append({
                "session_id": sid,
                "takeaway": takeaway,
                "distill": "enqueue_only",
                "now_flag": False,
                "path": "herdr_session_close_living_ssh",
            })
            # Prove TAKEAWAY landed in remote session file
            p3 = subprocess.run(
                ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host,
                 f"grep -c '\\[TAKEAWAY\\]' {remote_session} || true"],
                capture_output=True, text=True,
            )
            try:
                session_has_takeaway = int((p3.stdout or "0").strip() or "0") > 0
            except ValueError:
                session_has_takeaway = "[TAKEAWAY]" in (p3.stdout or "")

ok = (not dual_writer) and bool(applied) and apply_error is None
if ok and not session_has_takeaway:
    # Soft: close succeeded; remote grep may race — still mark ok with note
    advice = "herdr_living_enqueue_ok — session close enqueued; TAKEAWAY grep soft"
else:
    advice = (
        "herdr_living_enqueue_ok — living takeaway enqueued (no --now)"
        if ok
        else f"herdr_living_enqueue_fail — {apply_error or 'unknown'}"
    )
    if ok:
        advice = "herdr_living_enqueue_ok — living takeaway enqueued (no --now)"

payload = {
    "schema": "gzmo.brain_feed.herdr_living_enqueue/v1",
    "generated_at": now.isoformat(),
    "ok": ok,
    "advice": advice,
    "dual_writer": dual_writer,
    "now_flag": False,
    "session_id": sid if applied else None,
    "remote_session": remote_session,
    "session_has_takeaway": session_has_takeaway,
    "takeaway": takeaway,
    "applied": applied,
    "apply_error": apply_error,
    "plugin_path": "integrations/herdr-gzmo-metabolism/scripts/session-close.sh",
    "operator": [
        "Same ritual as herdr gzmo.metabolism.session-close — aimed at living host",
        "Never pass --now while CT101 owns overnight",
        "Optional: HERDR_METABOLISM_LIVING=1 on plugin session-close --living",
    ],
    "doc": "docs/HERDR_METABOLISM.md",
}
(out / "living-enqueue.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "living-enqueue.md").write_text(
    "\n".join([
        "# herdr living enqueue",
        "",
        f"Ok: **{ok}**",
        f"Advice: {advice}",
        f"now_flag: false",
        f"session_has_takeaway: {session_has_takeaway}",
        f"session_id: {sid if applied else '—'}",
        "",
        "See docs/HERDR_METABOLISM.md",
        "",
    ]) + "\n",
    encoding="utf-8",
)
print(json.dumps({
    "ok": ok,
    "advice": advice,
    "session_id": sid if applied else None,
    "session_has_takeaway": session_has_takeaway,
    "dual_writer": dual_writer,
    "apply_error": apply_error,
}, indent=2))
raise SystemExit(0 if ok else 1)
PY
