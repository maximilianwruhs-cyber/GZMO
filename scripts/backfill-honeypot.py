#!/usr/bin/env python3
"""One-shot backfill: semantic_vault → honeypot (M2 rules, mirrors Rust qualifies_for_honeypot)."""

from __future__ import annotations

import argparse
import sqlite3
from datetime import datetime, timezone
from pathlib import Path

DEFAULT_DB = Path(__file__).resolve().parents[1] / "data" / "vault.db"
MIN_CONFIDENCE = 0.85


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


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--db", type=Path, default=DEFAULT_DB)
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args()

    if not args.db.exists():
        raise SystemExit(f"[!] No vault at {args.db}")

    conn = sqlite3.connect(args.db)
    conn.row_factory = sqlite3.Row
    # Broken FTS triggers (rowid vs TEXT id) cause SQL logic error on upsert — drop until M3 FTS wiring.
    for trig in ("trg_honeypot_ai", "trg_honeypot_ad", "trg_honeypot_au"):
        conn.execute(f"DROP TRIGGER IF EXISTS {trig}")
    tables = {r[0] for r in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")}
    if "honeypot" not in tables:
        raise SystemExit("[!] honeypot table missing — open vault with gzmo-core migration v3 first")

    rows = conn.execute(
        """
        SELECT id, content, content_norm, embedding, confidence, decay_class, source_file, created_at
        FROM semantic_vault
        WHERE embedding IS NOT NULL AND length(embedding) >= 4
        """
    ).fetchall()

    candidates = [r for r in rows if qualifies(r)]
    vault_n = conn.execute("SELECT COUNT(*) FROM semantic_vault").fetchone()[0]
    print(f"[*] vault_rows={vault_n} embedded_candidates={len(rows)} qualifies={len(candidates)}")

    if args.dry_run:
        ratio = len(candidates) / max(vault_n, 1)
        print(f"[*] dry-run ratio honeypot/vault would be {ratio:.1%}")
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
            ) VALUES (?, ?, ?, ?, ?, 'ingest_backfill', 'fact', 1, ?, ?, ?, 'obolus', ?, 1, 0)
            ON CONFLICT(id) DO UPDATE SET
                content = excluded.content,
                content_norm = excluded.content_norm,
                embedding = excluded.embedding,
                confidence = MAX(confidence, excluded.confidence),
                source_file = excluded.source_file,
                promoted_at = excluded.promoted_at
            """,
            (
                r["id"],
                r["id"],
                r["content"],
                r["content_norm"] or r["content"].lower(),
                r["embedding"],
                r["confidence"],
                r["decay_class"] or "CuratedVault",
                r["source_file"],
                r["created_at"] or datetime.now(timezone.utc).isoformat(),
            ),
        )
        inserted += 1
    conn.commit()

    hp = conn.execute("SELECT COUNT(*) FROM honeypot").fetchone()[0]
    conn.close()
    ratio = hp / max(vault_n, 1)
    print(f"[+] upserted {inserted} rows; honeypot_total={hp} ratio={ratio:.1%}")
    if ratio > 0.30:
        print("[!] ratio > 30% — consider tightening filters (see ANTIGRAVITY_STEP5 §7)", file=__import__("sys").stderr)


if __name__ == "__main__":
    main()
