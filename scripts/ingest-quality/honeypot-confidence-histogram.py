#!/usr/bin/env python3
"""Analyze vault confidence vs honeypot membership for F2 tuning.

Usage:
  ./scripts/ingest-quality/honeypot-confidence-histogram.py [--vault data/vault.db]
"""
from __future__ import annotations

import argparse
import json
import sqlite3
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
HONEYPOT_MIN = 0.85
SUGGEST_MIN = 0.82


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--vault", default=str(ROOT / "data" / "vault.db"))
    args = parser.parse_args()
    vault = Path(args.vault)
    if not vault.is_file():
        print(f"[FAIL] vault not found: {vault}", file=sys.stderr)
        return 1

    conn = sqlite3.connect(vault)
    rows = conn.execute(
        "SELECT sv.id, sv.confidence, sv.source_file, "
        "CASE WHEN h.id IS NOT NULL THEN 1 ELSE 0 END AS in_honeypot "
        "FROM semantic_vault sv "
        "LEFT JOIN honeypot h ON h.vault_id = sv.id"
    ).fetchall()
    conn.close()

    buckets = {
        "0.00-0.79": {"total": 0, "honeypot": 0},
        "0.80-0.84": {"total": 0, "honeypot": 0},
        "0.85-0.89": {"total": 0, "honeypot": 0},
        "0.90-1.00": {"total": 0, "honeypot": 0},
    }

    reject_reasons = {
        "low_confidence_band": 0,
        "missing_source": 0,
        "has_source": 0,
    }

    for _id, conf, source_file, in_hp in rows:
        c = float(conf or 0)
        if c < 0.80:
            key = "0.00-0.79"
        elif c < 0.85:
            key = "0.80-0.84"
        elif c < 0.90:
            key = "0.85-0.89"
        else:
            key = "0.90-1.00"
        buckets[key]["total"] += 1
        if in_hp:
            buckets[key]["honeypot"] += 1
        if c < HONEYPOT_MIN:
            reject_reasons["low_confidence_band"] += 1
        if not source_file or not str(source_file).strip():
            reject_reasons["missing_source"] += 1
        else:
            reject_reasons["has_source"] += 1

    band_80_84 = buckets["0.80-0.84"]["total"]
    band_80_84_hp = buckets["0.80-0.84"]["honeypot"]
    recommend_lower = False
    if band_80_84 > 0:
        hp_rate = band_80_84_hp / band_80_84
        if band_80_84 >= 100 and hp_rate >= 0.3:
            recommend_lower = True

    report = {
        "vault_rows": len(rows),
        "buckets": buckets,
        "reject_signals": reject_reasons,
        "current_honeypot_min": HONEYPOT_MIN,
        "recommend_lower_to": SUGGEST_MIN if recommend_lower else None,
        "recommendation": (
            f"Consider HONEYPOT_MIN_CONFIDENCE={SUGGEST_MIN} "
            f"({band_80_84} rows in 0.80-0.84, {hp_rate:.1%} already in honeypot)"
            if recommend_lower
            else "Keep HONEYPOT_MIN_CONFIDENCE=0.85 — insufficient evidence to lower"
        ),
    }

    print(json.dumps(report, indent=2))
    out = ROOT / "data" / "honeypot-confidence-report.json"
    out.write_text(json.dumps(report, indent=2) + "\n")
    print(f"\n[OK] wrote {out}")
    print(report["recommendation"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
