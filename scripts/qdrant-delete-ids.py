#!/usr/bin/env python3
"""Delete Qdrant points by UUID (post-supersession hygiene).

  python3 scripts/qdrant-delete-ids.py --url http://localhost:6333 \
    --collection honeypot --ids-file /tmp/ids.txt
"""

from __future__ import annotations

import argparse
import json
import sys
import urllib.request


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--url", default="http://localhost:6333")
    p.add_argument("--collection", default="honeypot")
    p.add_argument("--ids-file", required=True, help="One UUID per line")
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args()

    ids = [
        line.strip()
        for line in open(args.ids_file, encoding="utf-8")
        if line.strip() and not line.startswith("#")
    ]
    if not ids:
        print("no ids", file=sys.stderr)
        return 1
    print(f"ids={len(ids)} collection={args.collection}")
    if args.dry_run:
        return 0

    body = json.dumps({"points": ids}).encode()
    req = urllib.request.Request(
        f"{args.url.rstrip('/')}/collections/{args.collection}/points/delete?wait=true",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        out = json.loads(resp.read().decode())
    print(json.dumps(out, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
