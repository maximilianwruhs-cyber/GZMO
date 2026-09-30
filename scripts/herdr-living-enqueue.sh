#!/usr/bin/env bash
# herdr close-ritual → living host takeaway enqueue (Brain Feed / Unpark s1).
# LOCAL-FIRST since CUTOVER A (2026-09-30): writes to the local living root (~/.gzmo).
#   HERDR_LIVE_MODE=remote  → target a remote living host via SSH (reversible if one returns)
# No --now, dual-writer refuse, no memory-gym Cursor chat.
#
#   bash scripts/herdr-living-enqueue.sh
#   TAKEAWAY='…' bash scripts/herdr-living-enqueue.sh
# Artifact: data-next/herdr-metabolism/living-enqueue.json
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/herdr-metabolism"

# --- living host target (local-first, remote fallback) ---
LIVE_MODE="${HERDR_LIVE_MODE:-local}"
if [[ "$LIVE_MODE" == "remote" ]]; then
  HOST="${CT101_SSH_HOST:-ct101}"
  GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
  LIVING_ROOT="${CT101_LIVING_ROOT:-/opt/gzmo}"
else
  HOST="local"
  GZMO_BIN="${GZMO_LIVING_BIN:-$HOME/github-clone/GZMO/target/release/gzmo}"
  LIVING_ROOT="${GZMO_LIVING_ROOT:-$HOME/.gzmo}"
fi

TAKEAWAY="${TAKEAWAY:-}"
mkdir -p "$OUT"

if [[ -z "$TAKEAWAY" ]]; then
  TAKEAWAY="HerdrLivingEnqueue-$(date -u +%Y%m%dT%H%M%SZ)-$$: herdr close-ritual → living distill (no --now)"
fi

export OUT HOST GZMO_BIN LIVING_ROOT LIVE_MODE TAKEAWAY ROOT
python3 - <<'PY'
import json, os, subprocess, uuid, base64
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
host = os.environ["HOST"]
mode = os.environ.get("LIVE_MODE", "local")
gzmo_bin = os.environ["GZMO_BIN"]
living_root = os.environ["LIVING_ROOT"]
takeaway = os.environ["TAKEAWAY"].strip()
now = datetime.now(timezone.utc)
now_iso = now.strftime("%Y-%m-%dT%H:%M:%SZ")

def run_local(args, cwd=None, env_extra=None):
    e = dict(os.environ)
    if env_extra:
        e.update(env_extra)
    return subprocess.run(args, capture_output=True, text=True, timeout=90, cwd=cwd, env=e)

def run_remote(cmdline):
    return subprocess.run(["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmdline],
                          capture_output=True, text=True, timeout=120)

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
session_path_str = None
sid = f"herdr-living-{uuid.uuid4().hex[:8]}"

if dual_writer:
    apply_error = "refused_dual_writer — stop workstation gzmo-serve (lab) before living enqueue"
elif not takeaway:
    apply_error = "empty_takeaway"
else:
    session_path_str = f"{living_root}/data/sessions/{sid}.json"
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

    if mode == "local":
        sp = Path(session_path_str)
        sp.parent.mkdir(parents=True, exist_ok=True)
        sp.write_text(json.dumps(sess, separators=(",", ":")), encoding="utf-8")
        p = None  # local seed always ok unless write raised
        close_p = run_local(
            [gzmo_bin, "session", "close", sid, "--takeaway", takeaway],
            cwd=living_root,
            env_extra={"GZMO_CONFIG": f"{living_root}/gzmo.toml"},
        )
        p2 = close_p
    else:  # remote (reversible CT101-style)
        p = run_remote(f"cat > {session_path_str}")
        if p.returncode == 0 or True:
            pass
        # write session via ssh stdin
        pw = subprocess.run(["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, f"cat > {session_path_str}"],
                            input=json.dumps(sess), text=True, capture_output=True)
        if pw.returncode != 0:
            apply_error = f"seed_session:{(pw.stderr or pw.stdout or '')[:200]}"
        # shell-safe takeaway via base64 (no metachars in the remote command line)
        b64 = base64.b64encode(takeaway.encode("utf-8")).decode("ascii")
        cmd = (f"bash -lc 'cd {living_root} && GZMO_CONFIG={living_root}/gzmo.toml "
               f"{gzmo_bin} session close {sid} --takeaway \"$(echo {b64} | base64 -d)\"'")
        p2 = run_remote(cmd)

    if apply_error is None:
        if p2.returncode != 0:
            apply_error = f"session_close:{(p2.stderr or p2.stdout or '')[:300]}"
        else:
            applied.append({
                "session_id": sid,
                "takeaway": takeaway,
                "distill": "enqueue_only",
                "now_flag": False,
                "path": f"herdr_session_close_living_{mode}",
            })
            # Prove TAKEAWAY landed in the session file
            if mode == "local":
                try:
                    txt = Path(session_path_str).read_text(encoding="utf-8")
                    session_has_takeaway = "[TAKEAWAY]" in txt or takeaway[:40] in txt
                except Exception:
                    session_has_takeaway = False
            else:
                p3 = run_remote(f"grep -c '\\[TAKEAWAY\\]' {session_path_str} || true")
                try:
                    session_has_takeaway = int((p3.stdout or "0").strip() or "0") > 0
                except ValueError:
                    session_has_takeaway = "[TAKEAWAY]" in (p3.stdout or "")

ok = (not dual_writer) and bool(applied) and apply_error is None
if ok and not session_has_takeaway:
    advice = "herdr_living_enqueue_ok — session close enqueued; TAKEAWAY grep soft"
else:
    advice = ("herdr_living_enqueue_ok — living takeaway enqueued (no --now)" if ok
              else f"herdr_living_enqueue_fail — {apply_error or 'unknown'}")

payload = {
    "schema": "gzmo.brain_feed.herdr_living_enqueue/v1",
    "generated_at": now.isoformat(),
    "ok": ok,
    "mode": mode,
    "advice": advice,
    "dual_writer": dual_writer,
    "now_flag": False,
    "session_id": sid if applied else None,
    "remote_session": session_path_str,
    "session_has_takeaway": session_has_takeaway,
    "takeaway": takeaway,
    "applied": applied,
    "apply_error": apply_error,
    "plugin_path": "integrations/herdr-gzmo-metabolism/scripts/session-close.sh",
    "operator": [
        "Same ritual as herdr gzmo.metabolism.session-close — aimed at living host",
        "Never pass --now while the daemon owns overnight",
        "HERDR_LIVE_MODE=remote to target a remote living host (reversible)",
    ],
    "doc": "docs/HERDR_METABOLISM.md",
}
(out / "living-enqueue.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "living-enqueue.md").write_text(
    "\n".join([
        "# herdr living enqueue",
        "",
        f"Ok: **{ok}**",
        f"Mode: {mode}",
        f"Advice: {advice}",
        "now_flag: false",
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
    "mode": mode,
    "advice": advice,
    "session_id": sid if applied else None,
    "session_has_takeaway": session_has_takeaway,
    "dual_writer": dual_writer,
    "apply_error": apply_error,
}, indent=2))
raise SystemExit(0 if ok else 1)
PY
