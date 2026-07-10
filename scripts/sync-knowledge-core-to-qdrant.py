#!/usr/bin/env python3
"""Sync M5 knowledge_core concept cards to Qdrant (collection: knowledge_core).

Vectors are derived from honeypot provenance embeddings (mean pool).
Payload: entity_tag, concept_name, summary_md excerpt, provenance_ids, layer=knowledge_core.
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import struct
import sys
import uuid
from pathlib import Path

try:
    import urllib.request
except ImportError:
    sys.exit("stdlib only — no extra deps")

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_VAULT = ROOT / "data" / "vault.db"
DEFAULT_CORE = ROOT / "data" / "knowledge_core.db"
DEFAULT_URL = "http://192.168.31.202:6333"
DEFAULT_COLLECTION = "knowledge_core"
BATCH = 32
VECTOR_DIM = 1024


def decode_embed(blob: bytes) -> list[float]:
    if len(blob) % 4 != 0:
        return []
    return list(struct.unpack(f"<{len(blob) // 4}f", blob))


def mean_vector(vectors: list[list[float]]) -> list[float] | None:
    if not vectors:
        return None
    dim = len(vectors[0])
    if any(len(v) != dim for v in vectors):
        return None
    acc = [0.0] * dim
    for v in vectors:
        for i, x in enumerate(v):
            acc[i] += x
    n = float(len(vectors))
    return [x / n for x in acc]


def stable_point_id(core_id: str) -> str:
    return str(uuid.uuid5(uuid.NAMESPACE_URL, f"gzmo:knowledge_core:{core_id}"))


def qdrant_request(url: str, method: str, path: str, body: dict | None = None) -> dict:
    data = None
    headers = {"Content-Type": "application/json"}
    if body is not None:
        data = json.dumps(body).encode()
    req = urllib.request.Request(f"{url.rstrip('/')}{path}", data=data, headers=headers, method=method)
    with urllib.request.urlopen(req, timeout=60) as resp:
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
        {"vectors": {"size": VECTOR_DIM, "distance": "Cosine"}},
    )
    print(f"[*] Created collection {name} ({VECTOR_DIM}-dim cosine)")


def load_honeypot_embeddings(vault: Path) -> dict[str, list[float]]:
    conn = sqlite3.connect(vault)
    rows = conn.execute(
        "SELECT id, embedding FROM honeypot WHERE embedding IS NOT NULL AND length(embedding) >= 4"
    ).fetchall()
    conn.close()
    out: dict[str, list[float]] = {}
    for fid, blob in rows:
        vec = decode_embed(blob)
        if len(vec) == VECTOR_DIM:
            out[fid] = vec
    return out


def load_core_points(core_db: Path, vault: Path) -> list[dict]:
    if not core_db.exists():
        return []
    embeds = load_honeypot_embeddings(vault)
    conn = sqlite3.connect(core_db)
    conn.row_factory = sqlite3.Row

    # Detect schema: concept-card style (entity_tag) vs flat-fact style (content)
    cols = [r[1] for r in conn.execute("PRAGMA table_info(knowledge_core)").fetchall()]
    has_concept_schema = "entity_tag" in cols

    if has_concept_schema:
        rows = conn.execute(
            "SELECT id, entity_tag, concept_name, summary_md, provenance_ids, version FROM knowledge_core"
        ).fetchall()
    else:
        rows = conn.execute(
            "SELECT id, content, content_norm, confidence, origin, memory_type, recall_count, container_tag FROM knowledge_core"
        ).fetchall()

    conn.close()

    points: list[dict] = []
    skipped = 0
    for r in rows:
        if has_concept_schema:
            prov = json.loads(r["provenance_ids"] or "[]")
            vecs = [embeds[p] for p in prov if p in embeds]
            vec = mean_vector(vecs)
            if vec is None:
                skipped += 1
                continue
            excerpt = (r["summary_md"] or "")[:2000]
            points.append({
                "id": stable_point_id(r["id"]),
                "vector": vec,
                "payload": {
                    "core_id": r["id"],
                    "entity_tag": r["entity_tag"],
                    "concept_name": r["concept_name"],
                    "summary_md": excerpt,
                    "provenance_count": len(prov),
                    "version": r["version"],
                    "layer": "knowledge_core",
                    "source": "gzmo_knowledge_core",
                },
            })
        else:
            # Flat fact schema — no provenance_ids to derive vectors, skip true embedding
            content = r["content"] or ""
            content_norm = r["content_norm"] or ""
            points.append({
                "id": stable_point_id(r["id"]),
                "vector": [0.0] * VECTOR_DIM,  # zero vector — real embed from ingest pipeline
                "payload": {
                    "core_id": r["id"],
                    "entity_tag": content_norm[:120],
                    "concept_name": content[:200],
                    "summary_md": content[:2000],
                    "confidence": r["confidence"],
                    "origin": r["origin"],
                    "recall_count": r["recall_count"],
                    "layer": "knowledge_core",
                    "source": "gzmo_knowledge_core",
                },
            })
    if skipped:
        print(f"[!] skipped {skipped} cards without embeddable provenance", file=sys.stderr)
    return points


def upsert_batch(url: str, collection: str, batch: list[dict]) -> None:
    qdrant_request(url, "PUT", f"/collections/{collection}/points?wait=true", {"points": batch})


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--vault", type=Path, default=DEFAULT_VAULT)
    p.add_argument("--core", type=Path, default=DEFAULT_CORE)
    p.add_argument("--url", default=DEFAULT_URL)
    p.add_argument("--collection", default=DEFAULT_COLLECTION)
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args()

    if not args.vault.exists():
        print(f"[!] No vault at {args.vault}", file=sys.stderr)
        return 1
    if not args.core.exists():
        print(f"[!] No knowledge_core at {args.core} — run ripen-knowledge-core.py --commit first", file=sys.stderr)
        return 1

    points = load_core_points(args.core, args.vault)
    print(f"[*] {len(points)} knowledge_core cards with pooled vectors")

    if args.dry_run or not points:
        return 0 if points or args.dry_run else 1

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
        f"{info['result']['points_count']} points ({info['result']['status']})"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
