#!/usr/bin/env bash
# G8/L — honeypot FTS row parity; must not recreate trg_honeypot_* triggers.
set -eo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." >/dev/null 2>&1 && pwd)"
DB="${GZMO_VAULT_DB:-$ROOT/data/vault.db}"

if [[ ! -f "$DB" ]]; then
  echo "check-fts-sanity: missing $DB" >&2
  exit 2
fi

python3 - "$DB" <<'PY'
import sqlite3, sys

db = sys.argv[1]
c = sqlite3.connect(db)
hp = c.execute("SELECT COUNT(*) FROM honeypot WHERE is_latest=1").fetchone()[0]
fts = c.execute("SELECT COUNT(*) FROM honeypot_fts").fetchone()[0]
triggers = [
    r[0]
    for r in c.execute(
        "SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE 'trg_honeypot%'"
    ).fetchall()
]
print(f"honeypot_latest={hp}")
print(f"honeypot_fts={fts}")
print(f"trg_honeypot_triggers={triggers or '(none — expected)'}")
ok = hp == fts and len(triggers) == 0
print("PASS" if ok else "FAIL")
sys.exit(0 if ok else 1)
PY
