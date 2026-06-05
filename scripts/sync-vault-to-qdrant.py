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


def load_facts(db_path: Path, source: str) -> list[dict]:
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    if source == "honeypot":
        rows = conn.execute(
            """
            SELECT id, content, embedding, confidence, decay_class,
                   source_file, promoted_at
            FROM honeypot
            WHERE embedding IS NOT NULL AND length(embedding) >= 4
              AND is_latest = 1
            """
        ).fetchall()
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
        rows = conn.execute(
            """
            SELECT id, content, embedding, half_life_days, confidence,
                   confirmation_count, decay_class, created_at, last_accessed_at,
                   source_file
            FROM semantic_vault
            WHERE embedding IS NOT NULL AND length(embedding) >= 4
            """
        ).fetchall()
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
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args()
    if args.collection is None:
        args.collection = "honeypot" if args.source == "honeypot" else DEFAULT_COLLECTION

    if not args.db.exists():
        print(f"[!] No vault at {args.db}", file=sys.stderr)
        sys.exit(1)

    points = load_facts(args.db, args.source)
    print(
        f"[*] {len(points)} facts ({args.source}) with {VECTOR_DIM}-dim embeddings in {args.db}"
    )

    if args.dry_run:
        return

    ensure_collection(args.url, args.collection)
    synced = 0
    for i in range(0, len(points), BATCH):
        batch = points[i : i + BATCH]
        upsert_batch(args.url, args.collection, batch)
        synced += len(batch)
        print(f"  upserted {synced}/{len(points)}")

    info = qdrant_request(args.url, "GET", f"/collections/{args.collection}")
    print(
        f"[OK] Qdrant {args.collection}: "
        f"{info['result']['points_count']} points "
        f"({info['result']['status']})"
    )


if __name__ == "__main__":
    main()
