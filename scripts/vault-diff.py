#!/usr/bin/env python3
"""Compare two GZMO vault.db files (read-only). Stretch S3 vault-diff tooling."""
from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from pathlib import Path


def qcount(conn: sqlite3.Connection, sql: str) -> int:
    try:
        row = conn.execute(sql).fetchone()
        return int(row[0]) if row and row[0] is not None else 0
    except sqlite3.Error:
        return -1


def fact_ids(conn: sqlite3.Connection, limit: int = 5000) -> set[str]:
    try:
        rows = conn.execute(
            "SELECT id FROM semantic_vault LIMIT ?", (limit,)
        ).fetchall()
        return {str(r[0]) for r in rows if r and r[0] is not None}
    except sqlite3.Error:
        return set()


def sample_contents(conn: sqlite3.Connection, limit: int = 200) -> set[str]:
    try:
        rows = conn.execute(
            "SELECT content FROM semantic_vault WHERE content IS NOT NULL LIMIT ?",
            (limit,),
        ).fetchall()
        return {str(r[0]).strip() for r in rows if r and r[0]}
    except sqlite3.Error:
        return set()


def jaccard(a: set[str], b: set[str]) -> float:
    if not a and not b:
        return 1.0
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def summarize(path: Path) -> dict:
    conn = sqlite3.connect(f"file:{path}?mode=ro", uri=True)
    try:
        return {
            "path": str(path.resolve()),
            "bytes": path.stat().st_size if path.is_file() else 0,
            "semantic_vault": qcount(conn, "SELECT COUNT(*) FROM semantic_vault"),
            "honeypot_latest": qcount(
                conn, "SELECT COUNT(*) FROM honeypot WHERE is_latest=1"
            ),
            "honeypot_all": qcount(conn, "SELECT COUNT(*) FROM honeypot"),
            "facts_staging": qcount(conn, "SELECT COUNT(*) FROM facts"),
            "evidence": qcount(conn, "SELECT COUNT(*) FROM evidence"),
            "ids": fact_ids(conn),
            "contents": sample_contents(conn),
        }
    finally:
        conn.close()


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--left", required=True, help="First vault.db (e.g. data-next)")
    ap.add_argument("--right", required=True, help="Second vault.db (e.g. CT101 snapshot)")
    ap.add_argument("--json", action="store_true", help="Machine-readable output")
    args = ap.parse_args()

    left_path = Path(args.left)
    right_path = Path(args.right)
    if not left_path.is_file():
        print(f"FAIL: left vault missing: {left_path}", file=sys.stderr)
        return 2
    if not right_path.is_file():
        print(f"FAIL: right vault missing: {right_path}", file=sys.stderr)
        return 2

    left = summarize(left_path)
    right = summarize(right_path)
    id_overlap = len(left["ids"] & right["ids"])
    id_j = jaccard(left["ids"], right["ids"])
    content_j = jaccard(left["contents"], right["contents"])

    out = {
        "left": {k: v for k, v in left.items() if k not in ("ids", "contents")},
        "right": {k: v for k, v in right.items() if k not in ("ids", "contents")},
        "overlap": {
            "semantic_id_intersection": id_overlap,
            "semantic_id_jaccard": round(id_j, 4),
            "content_sample_jaccard": round(content_j, 4),
            "id_sample_size_left": len(left["ids"]),
            "id_sample_size_right": len(right["ids"]),
        },
    }

    if args.json:
        print(json.dumps(out, indent=2))
    else:
        print("=== vault-diff ===")
        for side in ("left", "right"):
            s = out[side]
            print(
                f"{side}: semantic={s['semantic_vault']} honeypot_latest={s['honeypot_latest']} "
                f"facts={s['facts_staging']} bytes={s['bytes']}"
            )
            print(f"  path={s['path']}")
        print(
            f"overlap: id_jaccard={out['overlap']['semantic_id_jaccard']} "
            f"content_jaccard={out['overlap']['content_sample_jaccard']} "
            f"id_intersection={out['overlap']['semantic_id_intersection']}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
