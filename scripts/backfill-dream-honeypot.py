#!/usr/bin/env python3
"""Backfill vault rows from dream consolidation into honeypot + FTS.

Dream used source_file=NULL until fixed in dreams.rs; this script:
1. Sets source_file to memory/YYYY-MM-DD.md for dream-shaped rows missing it.
2. Upserts qualifying rows into honeypot with origin verified_dream.
3. Rebuilds honeypot_fts (triggers are intentionally dropped).
"""

from __future__ import annotations

import argparse
import re
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_DB = Path(__file__).resolve().parents[1] / "data" / "vault.db"
MIN_CONFIDENCE = 0.85
DREAM_CONTENT = re.compile(r"^\[[A-Za-z_]+:[^\]]+\]")


def is_boilerplate(content: str) -> bool:
    lower = content.lower()
    return (
        "sources do not contain" in lower
        or "migration_id" in lower
        or "takeout drive" in lower
    )


def qualifies(row: sqlite3.Row) -> bool:
    if row["confidence"] < MIN_CONFIDENCE:
        return False
    if not row["source_file"]:
        return False
    sf_lower = row["source_file"].lower()
    if any(x in sf_lower for x in ["chat_history", "chat_session", "quelltext", "sources"]):
        return False
    lower = (row["content"] or "").lower()
    if lower.startswith("[relation:"):
        return False
    return not is_boilerplate(row["content"] or "")


def dream_source_for_created_at(created_at: str | None) -> str | None:
    if not created_at:
        return None
    day = created_at[:10]
    if len(day) != 10 or day[4] != "-":
        return None
    return f"memory/{day}.md"


def is_dream_shaped(content: str | None) -> bool:
    if not content:
        return False
    return bool(DREAM_CONTENT.match(content.strip()))


def rebuild_honeypot_fts(conn: sqlite3.Connection) -> None:
    conn.execute("DELETE FROM honeypot_fts")
    conn.execute(
        """
        INSERT INTO honeypot_fts(rowid, content, content_norm)
        SELECT rowid, content, content_norm FROM honeypot
        """
    )


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--db", type=Path, default=DEFAULT_DB)
    p.add_argument("--dry-run", action="store_true")
    p.add_argument(
        "--since",
        default="2026-06-04",
        help="Only vault rows created on/after this date (YYYY-MM-DD prefix match)",
    )
    args = p.parse_args()

    if not args.db.exists():
        raise SystemExit(f"[!] No vault at {args.db}")

    conn = sqlite3.connect(args.db)
    conn.row_factory = sqlite3.Row
    for trig in ("trg_honeypot_ai", "trg_honeypot_ad", "trg_honeypot_au"):
        conn.execute(f"DROP TRIGGER IF EXISTS {trig}")

    since_prefix = args.since if args.since.endswith("T") else args.since

    rows = conn.execute(
        """
        SELECT id, content, content_norm, embedding, confidence, decay_class, source_file, created_at
        FROM semantic_vault
        WHERE embedding IS NOT NULL AND length(embedding) >= 4
          AND confidence >= ?
          AND created_at >= ?
        """,
        (MIN_CONFIDENCE, since_prefix),
    ).fetchall()

    patched = 0
    for r in rows:
        if r["source_file"]:
            continue
        if not is_dream_shaped(r["content"]):
            continue
        src = dream_source_for_created_at(r["created_at"])
        if not src:
            continue
        patched += 1
        if not args.dry_run:
            conn.execute(
                "UPDATE semantic_vault SET source_file = ? WHERE id = ?",
                (src, r["id"]),
            )

    if not args.dry_run:
        conn.commit()
    print(f"[*] source_file patched on {patched} dream-shaped rows (since {args.since})")

    rows = conn.execute(
        """
        SELECT id, content, content_norm, embedding, confidence, decay_class, source_file, created_at
        FROM semantic_vault
        WHERE embedding IS NOT NULL AND length(embedding) >= 4
          AND created_at >= ?
        """,
        (since_prefix,),
    ).fetchall()

    candidates = [r for r in rows if qualifies(r) and is_dream_shaped(r["content"])]
    existing_hp = {r[0] for r in conn.execute("SELECT id FROM honeypot").fetchall()}
    new_ids = [r["id"] for r in candidates if r["id"] not in existing_hp]
    print(f"[*] dream honeypot candidates={len(candidates)} new_vs_honeypot={len(new_ids)}")

    if args.dry_run:
        conn.close()
        return

    inserted = 0
    for r in candidates:
        conn.execute(
            """
            INSERT INTO honeypot (
                id, vault_id, content, content_norm, embedding, origin, memory_type,
                verify_pass, confidence, decay_class, source_file, container_tag,
                promoted_at, is_latest, recall_count
            ) VALUES (?, ?, ?, ?, ?, 'verified_dream', 'fact', 1, ?, ?, ?, 'obolus', ?, 1, 0)
            ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                content_norm = excluded.content_norm,
                embedding = excluded.embedding,
                origin = 'verified_dream',
                confidence = MAX(confidence, excluded.confidence),
                source_file = excluded.source_file,
                promoted_at = excluded.promoted_at,
                is_latest = 1
            """,
            (
                r["id"],
                r["id"],
                r["content"],
                r["content_norm"] or (r["content"] or "").lower(),
                r["embedding"],
                r["confidence"],
                r["decay_class"] or "CuratedVault",
                r["source_file"],
                r["created_at"] or datetime.now(timezone.utc).isoformat(),
            ),
        )
        inserted += 1

    rebuild_honeypot_fts(conn)
    conn.commit()

    hp = conn.execute("SELECT COUNT(*) FROM honeypot WHERE is_latest=1").fetchone()[0]
    fts = conn.execute("SELECT COUNT(*) FROM honeypot_fts").fetchone()[0]
    conn.close()
    print(f"[+] honeypot upserted={inserted} honeypot_latest={hp} honeypot_fts={fts}")
    if hp != fts:
        print("[!] FTS parity mismatch after rebuild", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
