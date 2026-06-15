#!/usr/bin/env python3
"""Seed verified cognition-stack facts into vault + honeypot (manual origin).

Closes the honeypot gap for TurboQuant / Gemma 4 cutover / Prime model decisions
that live in repo docs but were never promoted through the pipeline.

After seeding, run:
  ./target/release/gzmo memory embed
  scripts/sync-vault-to-qdrant.py --source honeypot
"""

from __future__ import annotations

import argparse
import sqlite3
import sys
import uuid
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DB = ROOT / "data" / "vault.db"
SOURCE_FILE = "manual/cognition_stack_20260607.md"
ORIGIN = "manual"
CONTAINER = "obolus"
MIN_CONF = 0.95

# Verified from gzmo.toml, llama.cpp/prime-bench/*, GEMMA4_26B_PRIME.md (champion sweep 2026-06)
FACTS: list[tuple[str, str]] = [
    (
        "[SYSTEM:Prime] Production cognition on workstation port :8000 uses Gemma 4 26B-A4B-it MoE "
        "(alias gemma-4-26b-a4b-it, ctx 262144) via stock llama.cpp llama-server with dual RTX 5070 Ti "
        "layer-split and champion profile draft-mtp+ngram-mod, f16 KV, CUDA graphs off."
    ),
    (
        "[SYSTEM:Prime] VM200 Qwen2.5-Coder-7B on :8080 and Qwen2.5-1.5B librarian on :8083 were "
        "retired; extract, verify, dream, Spark, and session distill run on workstation Prime only."
    ),
    (
        "[CONCEPT:Prime-Champion-Profile] Gemma 4 26B champion sweep winner: draft-mtp+ngram-mod stacked "
        "spec, assistant gemma-4-26B-A4B-it-assistant-Q2_K.gguf, spec-draft-n-max=3, f16/f16 KV, "
        "GGML_CUDA_DISABLE_GRAPHS=1 — ~185 tok/s mtp-bench mean @ dual 5070 Ti."
    ),
    (
        "[CONCEPT:Gemma4-Chat-Template] Gemma 4 instruct uses gemma4 chat template via llama-server; "
        "legacy --chat-template gemma (Gemma 3) causes repetition/gibberish."
    ),
    (
        "[SYSTEM:Prime] gzmo.toml engine.local points at http://localhost:8000/v1 (gemma-4-26b-a4b-it); "
        "[librarian] is disabled and session distill routes to Prime via [routing.mappings] distill_*=local; "
        "sovereign FrankenMoE :8010 remains intentionally down."
    ),
    (
        "[CONCEPT:Cognition-Stack-Decision] Stack decision (2026-06): Gemma 4 26B-A4B @ 256K is "
        "production Prime (locked over Qwen3.6-35B 128K); ingest routes local_deterministic/local on "
        ":8000; do not bulk-ingest or swap Prime without eval gate."
    ),
    (
        "[SERVICE:Librarian] Session distill extract/summary/verify routes to Prime :8000 via gzmo.toml "
        "[routing.mappings] distill_*=local; [librarian] is disabled and VM200 :8083 Qwen 1.5B is retired."
    ),
    (
        "[CONCEPT:TurboQuant] TurboQuant KV cache fork (llama-cpp-turboquant) is not required for "
        "current Prime — stock llama.cpp + f16 KV achieves 256K on dual 5070 Ti with champion profile."
    ),
]


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def norm(content: str) -> str:
    return content.lower().strip()


def seed(db: Path, dry_run: bool) -> int:
    conn = sqlite3.connect(db)
    conn.row_factory = sqlite3.Row
    tables = {r[0] for r in conn.execute("SELECT name FROM sqlite_master WHERE type='table'")}
    if "honeypot" not in tables:
        print("[!] honeypot table missing — open vault with gzmo-core first", file=sys.stderr)
        return 1

    inserted_vault = inserted_hp = skipped = 0
    now = now_iso()

    for content in FACTS:
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
        f"[*] cognition seed: +{inserted_vault} vault, +{inserted_hp} honeypot, "
        f"={skipped} already present (source={SOURCE_FILE})"
    )
    if dry_run:
        print("[i] dry-run — no writes")
    elif inserted_hp:
        print("[i] next: ./target/release/gzmo memory embed && scripts/sync-vault-to-qdrant.py --source honeypot")
    return 0


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--db", type=Path, default=DEFAULT_DB)
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args()
    if not args.db.exists():
        print(f"[!] No vault at {args.db}", file=sys.stderr)
        return 1
    return seed(args.db, args.dry_run)


if __name__ == "__main__":
    raise SystemExit(main())
