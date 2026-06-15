#!/usr/bin/env python3
"""Emit a minimal Pi synapse session for live bus validation (flock-compatible)."""
from __future__ import annotations

import json
import os
import subprocess
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUS = Path(os.environ.get("GZMO_SYNAPSE_BUS", ROOT / "data/Synapse/events.jsonl"))
LOCK = BUS.parent / f"{BUS.name}.lock"


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def emit(event_type: str, session_id: str, data: dict | None = None, reply_to: str | None = None) -> str:
    eid = str(uuid.uuid4())
    payload = {
        "id": eid,
        "event_type": event_type,
        "source": "pi_agent",
        "timestamp": now_iso(),
        "correlation_id": session_id,
        "data": {"session_id": session_id, **(data or {})},
    }
    if reply_to:
        payload["reply_to"] = reply_to
    return json.dumps(payload, separators=(",", ":"))


def append_locked(line: str) -> None:
    BUS.parent.mkdir(parents=True, exist_ok=True)
    if not LOCK.exists():
        LOCK.write_text("\n")
    tmp = BUS.parent / f".synapse-live-{os.getpid()}.line"
    tmp.write_text(line if line.endswith("\n") else line + "\n", encoding="utf-8")
    cmd = f"cat '{tmp}' >> '{BUS}' && rm -f '{tmp}'"
    try:
        subprocess.run(["flock", "-x", str(LOCK), "bash", "-lc", cmd], check=True, timeout=15)
    except (subprocess.CalledProcessError, FileNotFoundError):
        with BUS.open("a", encoding="utf-8") as f:
            f.write(line if line.endswith("\n") else line + "\n")
    finally:
        if tmp.exists():
            tmp.unlink(missing_ok=True)


def main() -> int:
    session_id = str(uuid.uuid4())
    tool_call_id = str(uuid.uuid4())
    marker = f"infra-live-test-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"

    lines = [
        emit("session_start", session_id, {"reason": "infra-live-test", "marker": marker}),
        emit(
            "skill.invoke",
            session_id,
            {"skill": "calculate", "args": "2+3*4", "toolCallId": tool_call_id},
        ),
        emit(
            "skill.complete",
            session_id,
            {"skill": "calculate", "toolCallId": tool_call_id, "duration_ms": 42},
        ),
        emit(
            "quest_complete",
            session_id,
            {
                "turnIndex": 0,
                "messageText": "calculate live test",
                "inputTokens": 100,
                "outputTokens": 50,
                "toolResultsCount": 1,
                "marker": marker,
            },
        ),
        emit("session_end", session_id, {"reason": "infra-live-test", "marker": marker}),
    ]

    for line in lines:
        append_locked(line)

    print(f"MARKER={marker}")
    print(f"SESSION_ID={session_id}")
    print(f"BUS={BUS}")
    print(f"APPENDED={len(lines)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
