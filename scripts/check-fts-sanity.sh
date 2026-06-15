#!/usr/bin/env bash
# G8/L — honeypot FTS index parity (join-based, matches ensure_honeypot_fts_synced).
# Must not recreate trg_honeypot_* triggers.
# Usage: check-fts-sanity.sh [--repair]
set -eo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
DB="${GZMO_VAULT_DB:-$ROOT/data/vault.db}"
REPAIR=0
[[ "${1:-}" == "--repair" ]] && REPAIR=1

if [[ ! -f "$DB" ]]; then
  echo "check-fts-sanity: missing $DB" >&2
  exit 2
fi

python3 - "$DB" "$REPAIR" <<'PY'
import sqlite3, sys

db, repair = sys.argv[1], sys.argv[2] == "1"
c = sqlite3.connect(db)

hp = c.execute("SELECT COUNT(*) FROM honeypot WHERE is_latest=1").fetchone()[0]
fts_total = c.execute("SELECT COUNT(*) FROM honeypot_fts").fetchone()[0]
fts_latest = c.execute(
    """SELECT COUNT(*) FROM honeypot_fts f
       JOIN honeypot h ON f.rowid = h.rowid
       WHERE h.is_latest = 1"""
).fetchone()[0]
fts_stale = c.execute(
    """SELECT COUNT(*) FROM honeypot_fts f
       JOIN honeypot h ON f.rowid = h.rowid
       WHERE h.is_latest = 0"""
).fetchone()[0]
fts_orphan = c.execute(
    """SELECT COUNT(*) FROM honeypot_fts f
       LEFT JOIN honeypot h ON f.rowid = h.rowid
       WHERE h.rowid IS NULL"""
).fetchone()[0]
triggers = [
    r[0]
    for r in c.execute(
        "SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE 'trg_honeypot%'"
    ).fetchall()
]

drift = fts_latest != hp or fts_stale > 0 or fts_orphan > 0
if drift and repair and hp > 0:
    c.execute("DELETE FROM honeypot_fts")
    c.execute(
        """INSERT INTO honeypot_fts(rowid, content, content_norm)
           SELECT rowid, content, content_norm FROM honeypot WHERE is_latest = 1"""
    )
    c.commit()
    fts_total = c.execute("SELECT COUNT(*) FROM honeypot_fts").fetchone()[0]
    fts_latest = hp
    fts_stale = 0
    fts_orphan = 0
    print("[repair] rebuilt honeypot_fts from is_latest=1 rows")

print(f"honeypot_latest={hp}")
print(f"honeypot_fts={fts_total} (indexed_latest={fts_latest} stale={fts_stale} orphan={fts_orphan})")
print(f"trg_honeypot_triggers={triggers or '(none — expected)'}")
ok = fts_latest == hp and fts_stale == 0 and fts_orphan == 0 and len(triggers) == 0
print("PASS" if ok else "FAIL")
sys.exit(0 if ok else 1)
PY
