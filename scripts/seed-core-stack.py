#!/usr/bin/env python3
"""Seed curated core-stack facts into vault + honeypot (manual origin).

Source of truth: docs/CORE_STACK_KNOWLEDGE.md. This script parses the
"Injected facts:" bullet lines from that document (every bullet whose text
starts with a [TYPE:Name] tag) and injects them directly into the vault +
honeypot as Structural, high-confidence, operator-curated facts.

This is the seed-cognition-stack.py pattern extended to the whole machine:
no LLM extraction, no migration-pile data — just authoritative self-knowledge.

After seeding, run:
  ./target/release/gzmo memory embed
  scripts/sync-vault-to-qdrant.py --source honeypot
"""

from __future__ import annotations

import argparse
import re
import sqlite3
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DB = ROOT / "data" / "vault.db"
DEFAULT_DOC = ROOT / "docs" / "CORE_STACK_KNOWLEDGE.md"
SOURCE_FILE = "manual/core_stack_20260607.md"
ORIGIN = "manual"
CONTAINER = "obolus"
MIN_CONF = 0.95

# A core fact is a markdown bullet whose content starts with a [TYPE:Name] tag.
# Card-field bullets (What:/How:/Use:/Why:/Related:) and headers never match.
FACT_RE = re.compile(r"^\s*-\s+(\[[A-Z][A-Za-z0-9]*:[^\]]+\].+)$")


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def norm(content: str) -> str:
    return content.lower().strip()


def extract_facts(doc: Path) -> list[str]:
    """Parse [TYPE:Name] fact bullets, skipping fenced code blocks."""
    facts: list[str] = []
    seen: set[str] = set()
    in_fence = False
    for raw in doc.read_text(encoding="utf-8").splitlines():
        stripped = raw.lstrip()
        if stripped.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        m = FACT_RE.match(raw)
        if not m:
            continue
        fact = " ".join(m.group(1).split())  # collapse whitespace
        key = norm(fact)
        if key in seen:
            continue
        seen.add(key)
        facts.append(fact)
    return facts


def mirror_embeddings(db: Path, source_file: str = SOURCE_FILE) -> int:
    """Copy semantic_vault embeddings into matching honeypot rows (post gzmo memory embed)."""
    conn = sqlite3.connect(db)
    n = conn.execute(
        """
        UPDATE honeypot SET embedding = (
            SELECT v.embedding FROM semantic_vault v WHERE v.id = honeypot.vault_id
        )
        WHERE source_file = ? AND vault_id IN (
            SELECT id FROM semantic_vault
            WHERE source_file = ? AND embedding IS NOT NULL AND length(embedding) >= 4
        )
        """,
        (source_file, source_file),
    ).rowcount
    conn.commit()
    conn.close()
    print(f"[+] mirrored {n} vault embeddings → honeypot (source={source_file})")
    return 0


def seed(db: Path, facts: list[str], dry_run: bool) -> int:
    conn = sqlite3.connect(db)
    conn.row_factory = sqlite3.Row
    tables = {r[0] for r in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")}
    if "honeypot" not in tables:
        print("[!] honeypot table missing — open vault with gzmo-core first", file=sys.stderr)
        return 1

    inserted_vault = inserted_hp = skipped = 0
    now = now_iso()

    for content in facts:
        cn = norm(content)
        existing = conn.execute(
            "SELECT id FROM honeypot WHERE content_norm = ? LIMIT 1", (cn,)
        ).fetchone()
        if existing:
            skipped += 1
            continue

        if dry_run:
            inserted_vault += 1
            inserted_hp += 1
            continue

        vid = str(uuid.uuid4())
        conn.execute(
            """
            INSERT INTO semantic_vault
                (id, content, embedding, half_life_days, confidence, confirmation_count,
                 decay_class, created_at, last_accessed_at, source_file, content_norm)
            VALUES (?, ?, ?, 365.0, ?, 1, 'Structural', ?, ?, ?, ?)
            """,
            (vid, content, b"", MIN_CONF, now, now, SOURCE_FILE, cn),
        )
        conn.execute(
            """
            INSERT INTO honeypot (
                id, vault_id, content, content_norm, embedding, origin, memory_type,
                verify_pass, confidence, decay_class, source_file, container_tag,
                promoted_at, is_latest, recall_count
            ) VALUES (?, ?, ?, ?, ?, ?, 'fact', 1, ?, 'Structural', ?, ?, ?, 1, 0)
            """,
            (
                vid,
                vid,
                content,
                cn,
                b"",
                ORIGIN,
                MIN_CONF,
                SOURCE_FILE,
                CONTAINER,
                now,
            ),
        )
        inserted_vault += 1
        inserted_hp += 1

    if not dry_run:
        conn.commit()
        conn.close()
        # vault backfill (gzmo memory embed) updates semantic_vault only — mirror to honeypot.
        conn = sqlite3.connect(db)
        n = conn.execute(
            """
            UPDATE honeypot SET embedding = (
                SELECT v.embedding FROM semantic_vault v WHERE v.id = honeypot.vault_id
            )
            WHERE source_file = ? AND vault_id IN (
                SELECT id FROM semantic_vault WHERE source_file = ?
            )
            """,
            (SOURCE_FILE, SOURCE_FILE),
        ).rowcount
        conn.commit()
        conn.close()
        if n:
            print(f"[+] mirrored {n} vault embeddings → honeypot")
    else:
        conn.close()

    print(
        f"[*] core-stack seed: +{inserted_vault} vault, +{inserted_hp} honeypot, "
        f"={skipped} already present (source={SOURCE_FILE})"
    )
    if dry_run:
        print("[i] dry-run — no writes")
    elif inserted_hp:
        print(
            "[i] next: ./target/release/gzmo memory embed && "
            "scripts/seed-core-stack.py --mirror-embeddings && "
            "scripts/sync-vault-to-qdrant.py --source honeypot"
        )
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--db", type=Path, default=DEFAULT_DB)
    p.add_argument("--doc", type=Path, default=DEFAULT_DOC, help="core knowledge markdown")
    p.add_argument("--dry-run", action="store_true")
    p.add_argument("--list", action="store_true", help="print parsed facts and exit")
    p.add_argument(
        "--mirror-embeddings",
        action="store_true",
        help="copy vault embeddings into honeypot after gzmo memory embed",
    )
    args = p.parse_args()

    if args.mirror_embeddings:
        if not args.db.exists():
            print(f"[!] No vault at {args.db}", file=sys.stderr)
            return 1
        return mirror_embeddings(args.db)

    if not args.doc.exists():
        print(f"[!] No core knowledge doc at {args.doc}", file=sys.stderr)
        return 1

    facts = extract_facts(args.doc)
    if not facts:
        print(f"[!] No [TYPE:Name] facts parsed from {args.doc}", file=sys.stderr)
        return 1

    if args.list:
        for f in facts:
            print(f)
        print(f"\n[*] {len(facts)} facts parsed from {args.doc}", file=sys.stderr)
        return 0

    print(f"[*] parsed {len(facts)} core facts from {args.doc}")

    if not args.db.exists():
        print(f"[!] No vault at {args.db}", file=sys.stderr)
        return 1
    return seed(args.db, facts, args.dry_run)


if __name__ == "__main__":
    raise SystemExit(main())
