#!/usr/bin/env python3
"""
export-knowledge-core.py  —  M5: Export mature knowledge from honeypot.

Exports honeypot rows that meet maturity criteria into a standalone
knowledge_core.db. Only rows that are:
  - is_latest=1 (not superseded)
  - confidence >= 0.90
  - recall_count >= 3 (recalled at least 3 times)
  - origin in ('ingest', 'verified_dream', 'session_distill')

Writes `{output_parent}/ripen/latest.json` with gate diagnostics so overnight
"0 rows" is honest (starved recall vs empty core).

Usage:
  python3 scripts/export-knowledge-core.py [--db /opt/gzmo/data/vault.db]
                                           [--min-confidence 0.90]
                                           [--min-recall 3]
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from datetime import datetime, timezone
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

ORIGINS = ("ingest", "verified_dream", "session_distill")


def _count(conn: sqlite3.Connection, sql: str, params: tuple = ()) -> int:
    row = conn.execute(sql, params).fetchone()
    return int(row[0]) if row else 0


def export(
    vault_path: Path,
    output_path: Path,
    min_confidence: float,
    min_recall: int,
    limit: int | None = None,
) -> dict:
    src = sqlite3.connect(str(vault_path))
    src.row_factory = sqlite3.Row

    latest = _count(src, "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1")
    conf_ok = _count(
        src,
        "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1 AND confidence >= ?",
        (min_confidence,),
    )
    recall_ok = _count(
        src,
        "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1 AND recall_count >= ?",
        (min_recall,),
    )
    dual = _count(
        src,
        """SELECT COUNT(*) FROM honeypot
           WHERE is_latest = 1 AND confidence >= ? AND recall_count >= ?""",
        (min_confidence, min_recall),
    )
    nonzero_recall = _count(
        src,
        "SELECT COUNT(*) FROM honeypot WHERE is_latest = 1 AND recall_count > 0",
    )

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

    dual_origin = len(rows)
    if limit:
        rows = rows[:limit]

    dst = sqlite3.connect(str(output_path))
    dst.executescript(SCHEMA)
    before = _count(dst, "SELECT COUNT(*) FROM knowledge_core")

    dst.executemany(
        """INSERT OR REPLACE INTO knowledge_core
           (id, vault_id, content, content_norm, confidence, origin,
            memory_type, recall_count, container_tag, decay_class,
            source_file, promoted_at, exported_at)
           VALUES (?,?,?,?,?,?,?,?,?,?,?,?, datetime('now'))""",
        [tuple(r) for r in rows],
    )
    dst.commit()
    after = _count(dst, "SELECT COUNT(*) FROM knowledge_core")

    src.close()
    dst.close()

    exported = len(rows)
    if exported == 0 and nonzero_recall == 0:
        advice = (
            "starved_recall — no honeypot hits have recall_count>0; "
            "Felt Use / living search must run before ripen can emit"
        )
    elif exported == 0 and dual == 0:
        advice = (
            f"gate_miss — dual gate confidence>={min_confidence} AND "
            f"recall>={min_recall} matched 0 of {latest} latest"
        )
    elif exported == 0 and dual_origin == 0:
        advice = (
            f"origin_filter — {dual} passed dual gate but 0 in origins {ORIGINS}"
        )
    else:
        advice = f"exported_{exported} — core_rows {before}→{after}"

    return {
        "schema": "gzmo.ripen.export/v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "vault": str(vault_path),
        "output": str(output_path),
        "min_confidence": min_confidence,
        "min_recall": min_recall,
        "latest": latest,
        "nonzero_recall": nonzero_recall,
        "gate_confidence": conf_ok,
        "gate_recall": recall_ok,
        "gate_dual": dual,
        "gate_dual_origin": dual_origin,
        "exported": exported,
        "core_rows_before": before,
        "core_rows_after": after,
        "advice": advice,
        "ok": True,
        "emitted": exported > 0,
    }


def write_status(output_path: Path, payload: dict) -> Path:
    status_dir = output_path.parent / "ripen"
    status_dir.mkdir(parents=True, exist_ok=True)
    path = status_dir / "latest.json"
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    (status_dir / f"export-{stamp}.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )
    return path


def main() -> int:
    parser = argparse.ArgumentParser(description="Export mature knowledge from honeypot")
    parser.add_argument("--db", default=str(DEFAULT_DB), help="Path to vault.db")
    parser.add_argument("--output", default=str(DEFAULT_OUT), help="Output path")
    parser.add_argument("--min-confidence", type=float, default=0.90, help="Min confidence")
    parser.add_argument("--min-recall", type=int, default=3, help="Min recall count")
    parser.add_argument("--limit", type=int, default=None, help="Max rows")
    args = parser.parse_args()

    payload = export(
        Path(args.db),
        Path(args.output),
        args.min_confidence,
        args.min_recall,
        args.limit,
    )
    status = write_status(Path(args.output), payload)
    print(
        f"Exported {payload['exported']} rows to {args.output} "
        f"(core {payload['core_rows_before']}→{payload['core_rows_after']})"
    )
    print(
        f"gates: dual={payload['gate_dual']} dual+origin={payload['gate_dual_origin']} "
        f"nonzero_recall={payload['nonzero_recall']}"
    )
    print(f"advice: {payload['advice']}")
    print(f"status: {status}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
