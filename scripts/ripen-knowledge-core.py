#!/usr/bin/env python3
"""Ripen honeypot facts into the M5 knowledge_core.

Implements docs/archive/M5_KNOWLEDGE_CORE_CHARTER.md:

  Gate (§2)      residency >= 30d AND corroboration >= 3 (vault confirmation_count),
                 is_latest=1, verify_pass=1.
  Ripen (§3)     1. global dedup (content_norm)
                 2. contradiction resolution (is_latest=0 excluded; audit kept in provenance)
                 3. concept-card synthesis (one summary_md card per entity)
  Export (§4)    data/knowledge_core.db  (charter schema, exact)
  Exit (§5)      compaction ratio <= 10% of honeypot row count.

Operator sign-off (§2.2): default run is preview-only and writes a candidate
manifest. Committing the core DB requires --commit. --approve <manifest.json>
restricts the commit to operator-approved entity tags.

Gates are flags so the pipeline can be validated before the 3-month window:
strict charter run today yields ~0 cards (honeypot is days old), so use e.g.
  --min-age-days 0 --min-corroboration 1 --no-cap
for a v0 candidate core / preview.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_VAULT = ROOT / "data" / "vault.db"
DEFAULT_CORE = ROOT / "data" / "knowledge_core.db"
DEFAULT_MANIFEST = ROOT / "data" / "knowledge_core.candidates.json"
DEFAULT_EXPORT_MD = ROOT / "data" / "knowledge_core_export.md"

# Charter §2.1 defaults.
CHARTER_MIN_AGE_DAYS = 30
CHARTER_MIN_CORROBORATION = 3
# Charter §5.1 exit gate.
COMPACTION_MAX_RATIO = 0.10

TAG_RE = re.compile(r"^\s*\[([A-Z_]+):([^\]]+)\]\s*(.*)$", re.DOTALL)

CORE_SCHEMA = """
CREATE TABLE IF NOT EXISTS knowledge_core (
  id              TEXT PRIMARY KEY,
  entity_tag      TEXT NOT NULL,
  concept_name    TEXT NOT NULL,
  summary_md      TEXT NOT NULL,
  provenance_ids  TEXT NOT NULL,
  ripened_at      TEXT NOT NULL,
  version         INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX IF NOT EXISTS idx_knowledge_core_entity ON knowledge_core(entity_tag);
"""


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def parse_ts(value: str | None) -> datetime | None:
    if not value:
        return None
    try:
        ts = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None
    if ts.tzinfo is None:
        ts = ts.replace(tzinfo=timezone.utc)
    return ts


def age_days(promoted_at: str | None, now: datetime) -> int | None:
    ts = parse_ts(promoted_at)
    if ts is None:
        return None
    return (now - ts).days


def split_tag(content: str) -> tuple[str, str, str]:
    """Return (entity_tag, concept_name, body). Untagged → ('UNTAGGED', '<derived>', content)."""
    m = TAG_RE.match(content or "")
    if not m:
        head = (content or "").strip().split("\n", 1)[0][:48] or "unknown"
        return "UNTAGGED", head, (content or "").strip()
    type_, name, body = m.group(1), m.group(2).strip(), m.group(3).strip()
    return f"{type_}:{name}", name, body


def core_id(entity_tag: str) -> str:
    return "kc_" + hashlib.sha1(entity_tag.encode("utf-8")).hexdigest()[:16]


def load_candidates(conn: sqlite3.Connection, min_age: int, min_corrob: int, now: datetime):
    conn.row_factory = sqlite3.Row
    rows = conn.execute(
        """
        SELECT h.id, h.content, h.content_norm, h.confidence, h.recall_count,
               h.promoted_at, h.is_latest, h.verify_pass, h.decay_class,
               h.source_file, COALESCE(v.confirmation_count, 0) AS corroboration
        FROM honeypot h
        LEFT JOIN semantic_vault v ON h.vault_id = v.id
        """
    ).fetchall()
    total = len(rows)

    survivors = []
    for r in rows:
        if r["is_latest"] != 1:
            continue  # contradiction resolution: superseded rows never ripen
        if r["verify_pass"] != 1:
            continue
        if r["corroboration"] < min_corrob:
            continue
        ad = age_days(r["promoted_at"], now)
        if ad is None or ad < min_age:
            continue
        survivors.append(r)
    return total, survivors


def synthesize(survivors, now: datetime):
    """Group survivors by entity, dedup bodies, build concept cards."""
    groups: dict[str, dict] = {}
    for r in survivors:
        entity_tag, concept_name, body = split_tag(r["content"])
        g = groups.setdefault(
            entity_tag,
            {
                "entity_tag": entity_tag,
                "concept_name": concept_name,
                "bullets": [],
                "seen_norm": set(),
                "provenance_ids": [],
                "max_conf": 0.0,
                "sum_corrob": 0,
                "sum_recall": 0,
                "sources": set(),
            },
        )
        norm = (r["content_norm"] or body.lower()).strip()
        if norm not in g["seen_norm"] and body:  # §3.1 global dedup
            g["seen_norm"].add(norm)
            g["bullets"].append(body)
        g["provenance_ids"].append(r["id"])
        g["max_conf"] = max(g["max_conf"], float(r["confidence"] or 0.0))
        g["sum_corrob"] += int(r["corroboration"] or 0)
        g["sum_recall"] += int(r["recall_count"] or 0)
        if r["source_file"]:
            g["sources"].add(r["source_file"])

    cards = []
    for g in groups.values():
        fact_count = len(g["provenance_ids"])
        distinct = len(g["bullets"])
        # Importance: distinct knowledge density, trust, corroboration, usage.
        importance = distinct * 2 + g["max_conf"] * 3 + g["sum_corrob"] + g["sum_recall"]
        bullets = "\n".join(f"- {b}" for b in g["bullets"])
        footer = (
            f"\n\n_provenance: {fact_count} facts · {distinct} distinct · "
            f"max_conf {g['max_conf']:.2f} · corrob {g['sum_corrob']} · "
            f"ripened {now.date().isoformat()}_"
        )
        summary_md = f"## [{g['entity_tag']}]\n\n{bullets}{footer}"
        cards.append(
            {
                "id": core_id(g["entity_tag"]),
                "entity_tag": g["entity_tag"],
                "concept_name": g["concept_name"],
                "summary_md": summary_md,
                "provenance_ids": sorted(g["provenance_ids"]),
                "fact_count": fact_count,
                "distinct": distinct,
                "importance": round(importance, 3),
            }
        )
    cards.sort(key=lambda c: c["importance"], reverse=True)
    return cards


def apply_cap(cards, honeypot_total: int, cap: bool):
    if not cap:
        return cards, len(cards), None
    # Floor so the committed core stays at or under the <=10% exit gate.
    max_cards = max(1, int(honeypot_total * COMPACTION_MAX_RATIO))
    return cards[:max_cards], max_cards, COMPACTION_MAX_RATIO


def load_approved(path: Path | None) -> set[str] | None:
    if path is None:
        return None
    data = json.loads(path.read_text(encoding="utf-8"))
    approved = data.get("approved_entity_tags")
    if approved is None:
        # Fall back: treat every candidate in the manifest as approved.
        approved = [c["entity_tag"] for c in data.get("candidates", [])]
    return set(approved)


def write_manifest(path: Path, cards, meta) -> None:
    payload = {
        "generated_at": now_iso(),
        "meta": meta,
        "approved_entity_tags": [c["entity_tag"] for c in cards],
        "candidates": [
            {
                "id": c["id"],
                "entity_tag": c["entity_tag"],
                "concept_name": c["concept_name"],
                "fact_count": c["fact_count"],
                "distinct": c["distinct"],
                "importance": c["importance"],
                "provenance_ids": c["provenance_ids"],
            }
            for c in cards
        ],
    }
    path.write_text(json.dumps(payload, indent=2, ensure_ascii=False), encoding="utf-8")


def write_export_md(path: Path, cards, meta) -> None:
    lines = [
        "# Knowledge Core — what GZMO holds as ripened truth",
        "",
        f"_Generated {now_iso()} · {len(cards)} concept cards · "
        f"compaction {meta['compaction_ratio']:.1%} of {meta['honeypot_total']} honeypot facts_",
        "",
        "---",
        "",
    ]
    for c in cards:
        lines.append(c["summary_md"])
        lines.append("")
    path.write_text("\n".join(lines), encoding="utf-8")


def commit_core(core_db: Path, cards, now: datetime) -> dict:
    core_db.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(core_db)
    conn.executescript(CORE_SCHEMA)
    inserted = updated = unchanged = 0
    ripened = now.isoformat()
    for c in cards:
        existing = conn.execute(
            "SELECT summary_md, version FROM knowledge_core WHERE id=?", (c["id"],)
        ).fetchone()
        prov = json.dumps(c["provenance_ids"], ensure_ascii=False)
        if existing is None:
            conn.execute(
                "INSERT INTO knowledge_core "
                "(id, entity_tag, concept_name, summary_md, provenance_ids, ripened_at, version) "
                "VALUES (?,?,?,?,?,?,1)",
                (c["id"], c["entity_tag"], c["concept_name"], c["summary_md"], prov, ripened),
            )
            inserted += 1
        elif existing[0] != c["summary_md"]:
            conn.execute(
                "UPDATE knowledge_core SET entity_tag=?, concept_name=?, summary_md=?, "
                "provenance_ids=?, ripened_at=?, version=version+1 WHERE id=?",
                (c["entity_tag"], c["concept_name"], c["summary_md"], prov, ripened, c["id"]),
            )
            updated += 1
        else:
            unchanged += 1
    # Reconcile: the core is fully derived from the honeypot, so prune rows that
    # are no longer in the ripened set (dropped below cap, superseded, de-approved).
    keep = {c["id"] for c in cards}
    existing_ids = [r[0] for r in conn.execute("SELECT id FROM knowledge_core").fetchall()]
    removed = 0
    for rid in existing_ids:
        if rid not in keep:
            conn.execute("DELETE FROM knowledge_core WHERE id=?", (rid,))
            removed += 1
    conn.commit()
    conn.close()
    return {"inserted": inserted, "updated": updated, "unchanged": unchanged, "removed": removed}


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("--vault", type=Path, default=DEFAULT_VAULT, help="source honeypot/vault DB")
    p.add_argument("--core", type=Path, default=DEFAULT_CORE, help="output knowledge_core.db")
    p.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST, help="candidate manifest JSON")
    p.add_argument("--export-md", type=Path, default=DEFAULT_EXPORT_MD, help="human-readable card export")
    p.add_argument("--min-age-days", type=int, default=CHARTER_MIN_AGE_DAYS, help="honeypot residency gate")
    p.add_argument("--min-corroboration", type=int, default=CHARTER_MIN_CORROBORATION, help="confirmation_count gate")
    p.add_argument("--no-cap", action="store_true", help="disable the <=10%% compaction cap")
    p.add_argument("--commit", action="store_true", help="write knowledge_core.db + export (operator sign-off)")
    p.add_argument("--approve", type=Path, default=None, help="manifest of operator-approved entity tags")
    p.add_argument(
        "--sync-qdrant",
        action="store_true",
        help="after --commit, upsert knowledge_core collection to Qdrant (LXC101 :6333)",
    )
    args = p.parse_args()

    if not args.vault.exists():
        print(f"[!] no vault at {args.vault}", file=sys.stderr)
        return 1

    now = datetime.now(timezone.utc)
    conn = sqlite3.connect(args.vault)
    honeypot_total, survivors = load_candidates(conn, args.min_age_days, args.min_corroboration, now)
    conn.close()

    cards = synthesize(survivors, now)

    approved = load_approved(args.approve)
    if approved is not None:
        cards = [c for c in cards if c["entity_tag"] in approved]

    cards, max_cards, _ = apply_cap(cards, honeypot_total, cap=not args.no_cap)
    ratio = len(cards) / max(honeypot_total, 1)

    meta = {
        "honeypot_total": honeypot_total,
        "survivors_after_gate": len(survivors),
        "cards": len(cards),
        "compaction_ratio": round(ratio, 4),
        "cap_applied": None if args.no_cap else max_cards,
        "gates": {"min_age_days": args.min_age_days, "min_corroboration": args.min_corroboration},
        "charter_strict": args.min_age_days >= CHARTER_MIN_AGE_DAYS
        and args.min_corroboration >= CHARTER_MIN_CORROBORATION,
    }

    print(
        f"[*] honeypot={honeypot_total} survivors={len(survivors)} "
        f"cards={len(cards)} compaction={ratio:.1%} "
        f"(gate: age>={args.min_age_days}d corrob>={args.min_corroboration}"
        f"{', cap '+str(max_cards) if not args.no_cap else ', no-cap'})"
    )
    if meta["charter_strict"] and len(cards) == 0:
        print("[i] charter-strict gate empty — honeypot has not ripened yet (expected pre-M5).")
    if not args.no_cap and ratio > COMPACTION_MAX_RATIO + 1e-9:
        print(f"[!] compaction {ratio:.1%} exceeds {COMPACTION_MAX_RATIO:.0%} exit gate", file=sys.stderr)

    write_manifest(args.manifest, cards, meta)
    print(f"[+] wrote candidate manifest → {args.manifest}")

    if not args.commit:
        print("[i] preview only. Re-run with --commit (after operator review) to write the core DB.")
        return 0

    stats = commit_core(args.core, cards, now)
    write_export_md(args.export_md, cards, meta)
    print(
        f"[+] committed knowledge_core.db: +{stats['inserted']} new, "
        f"~{stats['updated']} reripened, ={stats['unchanged']} unchanged, "
        f"-{stats['removed']} pruned → {args.core}"
    )
    print(f"[+] wrote export → {args.export_md}")
    if args.sync_qdrant:
        import subprocess

        sync = Path(__file__).resolve().parent / "sync-knowledge-core-to-qdrant.py"
        print("[*] syncing knowledge_core → Qdrant …")
        rc = subprocess.call(
            [sys.executable, str(sync), "--vault", str(args.vault), "--core", str(args.core)],
        )
        if rc != 0:
            print("[!] Qdrant sync failed (is LXC101 :6333 reachable?)", file=sys.stderr)
            return rc
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
