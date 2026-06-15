#!/usr/bin/env python3
"""Emit a minimal Forum Romanum threaded chain for bus validation."""
from __future__ import annotations

import json
import os
import subprocess
import uuid
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUS = Path(os.environ.get("GZMO_SYNAPSE_BUS", ROOT / "data/Synapse/events.jsonl"))
LOCK = BUS.parent / f"{BUS.name}.lock"


def now_iso() -> str:
    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def emit(
    event_type: str,
    session_id: str,
    data: dict,
    reply_to: str | None = None,
) -> str:
    eid = str(uuid.uuid4())
    payload = {
        "id": eid,
        "event_type": event_type,
        "source": "pi_agent",
        "timestamp": now_iso(),
        "correlation_id": session_id,
        "data": data,
    }
    if reply_to:
        payload["reply_to"] = reply_to
    return json.dumps(payload, separators=(",", ":"))


def append_locked(line: str) -> None:
    BUS.parent.mkdir(parents=True, exist_ok=True)
    if not LOCK.exists():
        LOCK.write_text("\n")
    tmp = BUS.parent / f".forum-fixture-{os.getpid()}.line"
    tmp.write_text(line if line.endswith("\n") else line + "\n")
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
    marker = f"forum-fixture-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}"
    proposal_id = str(uuid.uuid4())

    a1 = emit("session_start", session_id, {"session_id": session_id, "marker": marker})
    append_locked(a1)
    a1_id = json.loads(a1)["id"]

    a2 = emit(
        "agent.message",
        session_id,
        {
            "session_id": session_id,
            "agent_id": "prometheus",
            "role": "proposer",
            "mode": "debate",
            "payload": {"text": "Proposal: tighten synapse gate defaults", "marker": marker},
        },
        reply_to=a1_id,
    )
    append_locked(a2)
    a2_id = json.loads(a2)["id"]

    a3 = emit(
        "proposal.created",
        session_id,
        {
            "session_id": session_id,
            "agent_id": "prometheus",
            "proposal_id": proposal_id,
            "title": "Synapse gate defaults",
            "body": "Enable gate in staging only",
            "status": "draft",
            "marker": marker,
        },
        reply_to=a2_id,
    )
    append_locked(a3)
    a3_id = json.loads(a3)["id"]

    a4 = emit(
        "agent.message",
        session_id,
        {
            "session_id": session_id,
            "agent_id": "epimetheus",
            "role": "critic",
            "mode": "debate",
            "payload": {"text": "Accept with monitoring", "marker": marker},
        },
        reply_to=a3_id,
    )
    append_locked(a4)
    a4_id = json.loads(a4)["id"]

    a5 = emit(
        "proposal.reviewed",
        session_id,
        {
            "session_id": session_id,
            "agent_id": "epimetheus",
            "proposal_id": proposal_id,
            "verdict": "accept",
            "comments": "fixture review",
            "marker": marker,
        },
        reply_to=a4_id,
    )
    append_locked(a5)

    print(f"MARKER={marker}")
    print(f"SESSION_ID={session_id}")
    print(f"PROPOSAL_ID={proposal_id}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
