#!/usr/bin/env python3
"""Mirror SQLite semantic_vault facts (with embeddings) into Qdrant LXC101."""

from __future__ import annotations

import argparse
import sqlite3
import struct
import sys
import uuid
from pathlib import Path

try:
    import urllib.request
    import json
except ImportError:
    sys.exit("stdlib only — no extra deps")

DEFAULT_DB = Path(__file__).resolve().parents[1] / "data" / "vault.db"
DEFAULT_URL = "http://192.168.31.202:6333"
DEFAULT_COLLECTION = "knowledge"
BATCH = 64
VECTOR_DIM = 1024


def decode_embed(blob: bytes) -> list[float]:
    if len(blob) % 4 != 0:
        return []
    return list(struct.unpack(f"<{len(blob) // 4}f", blob))


def qdrant_request(url: str, method: str, path: str, body: dict | None = None) -> dict:
    data = None
    headers = {"Content-Type": "application/json"}
    if body is not None:
        data = json.dumps(body).encode()
    req = urllib.request.Request(f"{url.rstrip('/')}{path}", data=data, headers=headers, method=method)
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read().decode())


def ensure_collection(url: str, name: str) -> None:
    try:
        info = qdrant_request(url, "GET", f"/collections/{name}")
        size = info["result"]["config"]["params"]["vectors"]["size"]
        if size != VECTOR_DIM:
            print(f"[!] Collection {name} has dim {size}, expected {VECTOR_DIM}", file=sys.stderr)
            sys.exit(1)
        return
    except Exception:
        pass
    qdrant_request(
        url,
        "PUT",
        f"/collections/{name}",
        {
            "vectors": {"size": VECTOR_DIM, "distance": "Cosine"},
        },
    )
    print(f"[*] Created collection {name} ({VECTOR_DIM}-dim cosine)")


def load_facts(
    db_path: Path,
    source: str,
    *,
    since: str | None = None,
    ids: set[str] | None = None,
) -> list[dict]:
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    if source == "honeypot":
        sql = """
            SELECT id, content, embedding, confidence, decay_class,
                   source_file, promoted_at
            FROM honeypot
            WHERE embedding IS NOT NULL AND length(embedding) >= 4
              AND is_latest = 1
            """
        params: list = []
        if since:
            sql += " AND promoted_at >= ?"
            params.append(since)
        if ids:
            placeholders = ",".join("?" for _ in ids)
            sql += f" AND id IN ({placeholders})"
            params.extend(ids)
        rows = conn.execute(sql, params).fetchall()
    elif source == "vault_filtered":
        rows = conn.execute(
            """
            SELECT id, content, embedding, half_life_days, confidence,
                   confirmation_count, decay_class, created_at, last_accessed_at,
                   source_file
            FROM semantic_vault
            WHERE embedding IS NOT NULL AND length(embedding) >= 4
              AND confidence >= 0.85
              AND source_file IS NOT NULL
              AND decay_class != 'Episodic'
            """
        ).fetchall()
    else:
        sql = """
            SELECT id, content, embedding, half_life_days, confidence,
                   confirmation_count, decay_class, created_at, last_accessed_at,
                   source_file
            FROM semantic_vault
            WHERE embedding IS NOT NULL AND length(embedding) >= 4
            """
        params = []
        if ids:
            placeholders = ",".join("?" for _ in ids)
            sql += f" AND id IN ({placeholders})"
            params.extend(ids)
        rows = conn.execute(sql, params).fetchall()
    conn.close()
    points = []
    for r in rows:
        vec = decode_embed(r["embedding"])
        if len(vec) != VECTOR_DIM:
            continue
        payload = {
            "content": r["content"],
            "confidence": r["confidence"],
            "decay_class": r["decay_class"],
            "source": "gzmo_honeypot" if source == "honeypot" else "gzmo_sqlite",
            "layer": source,
        }
        if source == "honeypot":
            payload["source_file"] = r["source_file"]
            payload["promoted_at"] = r["promoted_at"]
        else:
            payload["half_life_days"] = r["half_life_days"]
            payload["confirmation_count"] = r["confirmation_count"]
            payload["created_at"] = r["created_at"]
            payload["last_accessed_at"] = r["last_accessed_at"]
            if r["source_file"]:
                payload["source_file"] = r["source_file"]
        points.append(
            {
                "id": str(uuid.UUID(r["id"])),
                "vector": vec,
                "payload": payload,
            }
        )
    return points


def upsert_batch(url: str, collection: str, batch: list[dict]) -> None:
    qdrant_request(
        url,
        "PUT",
        f"/collections/{collection}/points?wait=true",
        {"points": batch},
    )


def prune_honeypot_orphans(url: str, collection: str, db_path: Path, *, dry_run: bool) -> int:
    """Delete Qdrant points whose id is not a current (`is_latest=1`) honeypot row.

    Temporal Validity / GPM: upserting latest does not remove superseded vectors.
    """
    conn = sqlite3.connect(db_path)
    keep = {str(uuid.UUID(r[0])) for r in conn.execute("SELECT id FROM honeypot WHERE is_latest = 1")}
    conn.close()

    orphans: list[str] = []
    offset = None
    while True:
        body: dict = {"limit": 256, "with_payload": False, "with_vector": False}
        if offset is not None:
            body["offset"] = offset
        res = qdrant_request(url, "POST", f"/collections/{collection}/points/scroll", body)[
            "result"
        ]
        pts = res.get("points") or []
        if not pts:
            break
        for pt in pts:
            pid = str(pt["id"])
            if pid not in keep:
                orphans.append(pid)
        offset = res.get("next_page_offset")
        if offset is None:
            break

    print(f"[*] qdrant orphans (not is_latest=1): {len(orphans)}")
    if dry_run or not orphans:
        return len(orphans)
    for i in range(0, len(orphans), BATCH):
        batch = orphans[i : i + BATCH]
        qdrant_request(
            url,
            "POST",
            f"/collections/{collection}/points/delete?wait=true",
            {"points": batch},
        )
        print(f"  pruned {i + len(batch)}/{len(orphans)}")
    return len(orphans)


def main() -> None:
    p = argparse.ArgumentParser(description="Sync GZMO vault embeddings to Qdrant")
    p.add_argument("--db", type=Path, default=DEFAULT_DB)
    p.add_argument("--url", default=DEFAULT_URL)
    p.add_argument("--collection", default=None)
    p.add_argument(
        "--source",
        choices=("honeypot", "vault", "vault_filtered"),
        default="honeypot",
        help="SQLite table/query (default: honeypot M2)",
    )
    p.add_argument(
        "--since",
        default=None,
        help="ISO timestamp — only honeypot rows with promoted_at >= since (incremental)",
    )
    p.add_argument(
        "--ids",
        default=None,
        help="Comma-separated fact UUIDs to upsert (incremental)",
    )
    p.add_argument("--dry-run", action="store_true")
    p.add_argument(
        "--no-prune",
        action="store_true",
        help="Skip deleting Qdrant points that are no longer is_latest=1 (honeypot only)",
    )
    args = p.parse_args()
    if args.collection is None:
        args.collection = "honeypot" if args.source == "honeypot" else DEFAULT_COLLECTION

    if not args.db.exists():
        print(f"[!] No vault at {args.db}", file=sys.stderr)
        sys.exit(1)

    id_set = None
    if args.ids:
        id_set = {x.strip() for x in args.ids.split(",") if x.strip()}

    points = load_facts(args.db, args.source, since=args.since, ids=id_set)
    print(
        f"[*] {len(points)} facts ({args.source}) with {VECTOR_DIM}-dim embeddings in {args.db}"
        + (f" since={args.since}" if args.since else "")
        + (f" ids={len(id_set)}" if id_set else "")
    )

    if args.dry_run:
        if args.source == "honeypot" and not args.no_prune:
            try:
                prune_honeypot_orphans(args.url, args.collection, args.db, dry_run=True)
            except Exception as e:
                print(f"[!] prune dry-run skipped: {e}", file=sys.stderr)
        return

    ensure_collection(args.url, args.collection)
    synced = 0
    for i in range(0, len(points), BATCH):
        batch = points[i : i + BATCH]
        upsert_batch(args.url, args.collection, batch)
        synced += len(batch)
        print(f"  upserted {synced}/{len(points)}")

    if args.source == "honeypot" and not args.no_prune:
        prune_honeypot_orphans(args.url, args.collection, args.db, dry_run=False)

    info = qdrant_request(args.url, "GET", f"/collections/{args.collection}")
    print(
        f"[OK] Qdrant {args.collection}: "
        f"{info['result']['points_count']} points "
        f"({info['result']['status']})"
    )


if __name__ == "__main__":
    main()
