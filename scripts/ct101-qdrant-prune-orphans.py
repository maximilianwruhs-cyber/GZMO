#!/usr/bin/env python3
"""Delete Qdrant honeypot points whose id is not in honeypot is_latest=1."""

from __future__ import annotations

import argparse
import json
import sqlite3
import urllib.request
from pathlib import Path


def qdrant(url: str, method: str, path: str, body: dict | None = None) -> dict:
    data = None if body is None else json.dumps(body).encode()
    req = urllib.request.Request(
        f"{url.rstrip('/')}{path}",
        data=data,
        headers={"Content-Type": "application/json"},
        method=method,
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        return json.loads(resp.read().decode())


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--db", type=Path, default=Path("/opt/gzmo/data/vault.db"))
    p.add_argument("--url", default="http://127.0.0.1:6333")
    p.add_argument("--collection", default="honeypot")
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args()

    conn = sqlite3.connect(args.db)
    keep = {r[0] for r in conn.execute("SELECT id FROM honeypot WHERE is_latest = 1")}
    print(f"keep={len(keep)}")

    offset = None
    orphans: list[str] = []
    while True:
        body: dict = {"limit": 256, "with_payload": False, "with_vector": False}
        if offset is not None:
            body["offset"] = offset
        res = qdrant(args.url, "POST", f"/collections/{args.collection}/points/scroll", body)[
            "result"
        ]
        pts = res.get("points") or []
        if not pts:
            break
        for pt in pts:
            pid = pt["id"]
            if pid not in keep:
                orphans.append(pid)
        offset = res.get("next_page_offset")
        if offset is None:
            break

    print(f"orphans={len(orphans)}")
    if args.dry_run or not orphans:
        info = qdrant(args.url, "GET", f"/collections/{args.collection}")
        print("points", info["result"]["points_count"])
        return 0

    for i in range(0, len(orphans), 64):
        batch = orphans[i : i + 64]
        qdrant(
            args.url,
            "POST",
            f"/collections/{args.collection}/points/delete?wait=true",
            {"points": batch},
        )
        print(f"deleted {i + len(batch)}/{len(orphans)}")

    info = qdrant(args.url, "GET", f"/collections/{args.collection}")
    print("points_after", info["result"]["points_count"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
