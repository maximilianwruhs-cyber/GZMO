#!/usr/bin/env python3
"""
export-knowledge-core.py  —  M5: Export mature knowledge from honeypot.

Exports honeypot rows that meet maturity criteria into a standalone
knowledge_core.db. Only rows that are:
  - is_latest=1 (not superseded)
  - confidence >= 0.90
  - recall_count >= 3 (recalled at least 3 times)
  - origin in ('ingest', 'verified_dream', 'session_distill')

Usage:
  python3 scripts/export-knowledge-core.py [--db /opt/gzmo/data/vault.db]
                                           [--min-confidence 0.90]
                                           [--min-recall 3]
"""

import sqlite3
import argparse
import sys
from pathlib import Path

DEFAULT_DB = Path("/opt/gzmo/data/vault.db")
DEFAULT_OUT = Path("/opt/gzmo/data/knowledge_core.db")

SCHEMA = """
CREATE TABLE IF NOT EXISTS knowledge_core (
    id              TEXT PRIMARY KEY,
    vault_id        TEXT NOT NULL,
    content         TEXT NOT NULL,
    content_norm    TEXT NOT NULL,
    confidence      REAL NOT NULL,
    origin          TEXT NOT NULL,
    memory_type     TEXT NOT NULL DEFAULT 'fact',
    recall_count    INTEGER NOT NULL DEFAULT 0,
    container_tag   TEXT NOT NULL DEFAULT 'obolus',
    decay_class     TEXT,
    source_file     TEXT,
    promoted_at     TEXT,
    exported_at     TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_kc_confidence ON knowledge_core(confidence DESC);
CREATE INDEX IF NOT EXISTS idx_kc_origin ON knowledge_core(origin);
"""


def export(
    vault_path: Path,
    output_path: Path,
    min_confidence: float,
    min_recall: int,
    limit: int | None = None,
) -> int:
    src = sqlite3.connect(str(vault_path))
    src.row_factory = sqlite3.Row

    rows = src.execute(
        """
        SELECT h.id, h.vault_id, h.content, h.content_norm, h.confidence,
               h.origin, h.memory_type, h.recall_count, h.container_tag,
               h.decay_class, h.source_file, h.promoted_at
        FROM honeypot h
        WHERE h.is_latest = 1
          AND h.confidence >= ?
          AND h.recall_count >= ?
          AND h.origin IN ('ingest', 'verified_dream', 'session_distill')
        ORDER BY h.confidence DESC, h.recall_count DESC
        """,
        (min_confidence, min_recall),
    ).fetchall()

    if limit:
        rows = rows[:limit]

    dst = sqlite3.connect(str(output_path))
    dst.executescript(SCHEMA)

    dst.executemany(
        """INSERT OR REPLACE INTO knowledge_core
           (id, vault_id, content, content_norm, confidence, origin,
            memory_type, recall_count, container_tag, decay_class,
            source_file, promoted_at, exported_at)
           VALUES (?,?,?,?,?,?,?,?,?,?,?,?, datetime('now'))""",
        [tuple(r) for r in rows],
    )
    dst.commit()

    src.close()
    dst.close()
    return len(rows)


def main():
    parser = argparse.ArgumentParser(description="Export mature knowledge from honeypot")
    parser.add_argument("--db", default=str(DEFAULT_DB), help="Path to vault.db")
    parser.add_argument("--output", default=str(DEFAULT_OUT), help="Output path")
    parser.add_argument("--min-confidence", type=float, default=0.90, help="Min confidence")
    parser.add_argument("--min-recall", type=int, default=3, help="Min recall count")
    parser.add_argument("--limit", type=int, default=None, help="Max rows")
    args = parser.parse_args()

    count = export(
        Path(args.db),
        Path(args.output),
        args.min_confidence,
        args.min_recall,
        args.limit,
    )
    print(f"Exported {count} rows to {args.output}")


if __name__ == "__main__":
    main()