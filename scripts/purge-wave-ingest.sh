#!/usr/bin/env bash
# Purge wave-scoped ingest footprint from vault, Neo4j, Qdrant, and episodic memory.
# Usage: ./purge-wave-ingest.sh wave_01_gzmo_obolus [--dry-run] [--confirm PURGE]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

WAVE="${1:-}"
DRY_RUN=0
CONFIRM=""

shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --confirm) CONFIRM="${2:-}"; shift ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
  shift
done

[[ -n "$WAVE" ]] || {
  echo "Usage: $0 <wave_prefix> [--dry-run] [--confirm PURGE]" >&2
  echo "  wave_prefix: e.g. wave_01_gzmo_obolus (matches source_file LIKE 'wave_01_%')" >&2
  exit 1
}

# Derive file prefix from wave name (wave_01_gzmo_obolus → wave_01_)
WAVE_NUM="$(echo "$WAVE" | grep -oE 'wave_[0-9]+' | head -1 || true)"
[[ -n "$WAVE_NUM" ]] || { echo "[!] Cannot derive wave prefix from: $WAVE" >&2; exit 1; }
FILE_PREFIX="${WAVE_NUM}_"

VAULT_DB="${GZMO_VAULT_DB:-$ROOT/data/vault.db}"
MEMORY_DIR="${GZMO_MEMORY_DIR:-$ROOT/memory}"
EPISODIC="$MEMORY_DIR/2026-06-02.md"
BACKUP="$ROOT/data/vault.db.pre-wave1-purge"
QDRANT_URL="${QDRANT_URL:-http://192.168.31.202:6333}"
QDRANT_COLLECTION="${QDRANT_COLLECTION:-knowledge}"
QDRANT_HONEYPOT_COLLECTION="${QDRANT_HONEYPOT_COLLECTION:-honeypot}"

# Neo4j from gzmo.toml env block (override via env)
NEO4J_URL="${NEO4J_URL:-bolt://192.168.31.202:7687}"
NEO4J_USER="${NEO4J_USERNAME:-neo4j}"
NEO4J_PASS="${NEO4J_PASSWORD:-}"
NEO4J_DB="${NEO4J_DATABASE:-neo4j}"

if [[ "$DRY_RUN" -eq 0 && "$CONFIRM" != "PURGE" ]]; then
  echo "[!] Destructive purge requires: --confirm PURGE" >&2
  echo "    Run with --dry-run first to preview counts." >&2
  exit 1
fi

if [[ "$DRY_RUN" -eq 0 && ! -f "$BACKUP" ]]; then
  echo "[!] Backup not found: $BACKUP" >&2
  echo "    cp data/vault.db data/vault.db.pre-wave1-purge" >&2
  exit 1
fi

echo "=== purge-wave-ingest $(date -Is) ==="
echo "wave=$WAVE file_prefix=$FILE_PREFIX dry_run=$DRY_RUN"
echo "vault=$VAULT_DB"

# --- SQLite: count / delete ---
VAULT_MATCH="source_file LIKE '${FILE_PREFIX}%' OR content LIKE '%[ingest:${FILE_PREFIX}%'"
# Fallback time window for rows ingested before source_file column existed
TIME_FALLBACK="created_at >= '2026-06-02T14:50:00' AND created_at <= '2026-06-02T16:35:00'"

SQLITE_PY=$(cat <<'PY'
import os, sqlite3, sys
db_path = os.environ["VAULT_DB"]
file_prefix = os.environ["FILE_PREFIX"]
time_fallback = os.environ["TIME_FALLBACK"]
dry = os.environ.get("DRY_RUN", "0") == "1"

conn = sqlite3.connect(db_path)
cursor = conn.cursor()

query_cond = f"source_file LIKE '{file_prefix}%' OR ({time_fallback})"
try:
    count = cursor.execute(f"SELECT COUNT(*) FROM semantic_vault WHERE {query_cond}").fetchone()[0]
    hp_count = 0
    if cursor.execute(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='honeypot'"
    ).fetchone():
        hp_count = cursor.execute(
            f"SELECT COUNT(*) FROM honeypot WHERE source_file LIKE '{file_prefix}%'"
        ).fetchone()[0]
except Exception as e:
    print(f"[!] Error querying SQLite: {e}", file=sys.stderr)
    conn.close()
    sys.exit(1)

print(f"[*] SQLite semantic_vault rows to purge: {count} (source_file + time fallback)")
print(f"[*] SQLite honeypot rows to purge (source_file): {hp_count}")

if dry:
    conn.close()
    sys.exit(0)

try:
    ids = [row[0] for row in cursor.execute(f"SELECT id FROM semantic_vault WHERE {query_cond}").fetchall()]
    print(f"__PURGED_IDS__:" + " ".join(ids))
    cursor.execute(f"DELETE FROM semantic_vault WHERE {query_cond}")
    if cursor.execute(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='honeypot'"
    ).fetchone():
        cursor.execute(
            f"DELETE FROM honeypot WHERE source_file LIKE '{file_prefix}%'"
        )
        print("[+] SQLite: deleted wave honeypot rows")
    if cursor.execute(
        "SELECT 1 FROM sqlite_master WHERE type='table' AND name='evidence'"
    ).fetchone():
        cursor.execute(
            f"DELETE FROM evidence WHERE source_file LIKE '{file_prefix}%'"
        )
        print("[+] SQLite: deleted wave evidence rows")
        if cursor.execute(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='evidence_fts'"
        ).fetchone():
            cursor.execute(
                "DELETE FROM evidence_fts WHERE rowid NOT IN (SELECT rowid FROM evidence)"
            )
            print("[+] SQLite: pruned orphan evidence_fts rows")
    conn.commit()
    print("[+] SQLite: deleted wave vault rows")
    after = cursor.execute("SELECT COUNT(*) FROM semantic_vault").fetchone()[0]
    print(f"[*] SQLite semantic_vault count now: {after}")
except Exception as e:
    print(f"[!] Error modifying SQLite: {e}", file=sys.stderr)
    conn.close()
    sys.exit(1)
conn.close()
PY
)

export VAULT_DB FILE_PREFIX TIME_FALLBACK DRY_RUN
SQLITE_OUT=$(python3 -c "$SQLITE_PY")
echo "$SQLITE_OUT" | grep -v "__PURGED_IDS__" || true
PURGED_IDS=$(echo "$SQLITE_OUT" | grep "__PURGED_IDS__" | sed 's/__PURGED_IDS__://' || true)


# --- Neo4j: strip wave ingest observations ---
PROVENANCE_PATTERN="source=${FILE_PREFIX}"
echo "[*] Neo4j: purge observations matching '$PROVENANCE_PATTERN'"

NEO4J_PY=$(cat <<'PY'
import os, sys
try:
    from neo4j import GraphDatabase
except ImportError:
    print("[WARN] python neo4j driver not installed — skip Neo4j purge or: pip install neo4j", file=sys.stderr)
    sys.exit(0)

url = os.environ["NEO4J_URL"]
user = os.environ["NEO4J_USER"]
password = os.environ["NEO4J_PASS"]
database = os.environ.get("NEO4J_DB", "neo4j")
pattern = os.environ["PROVENANCE_PATTERN"]
dry = os.environ.get("DRY_RUN", "0") == "1"

driver = GraphDatabase.driver(url, auth=(user, password))
with driver.session(database=database) as session:
    count = session.run(
        "MATCH (e) WHERE any(obs IN e.observations WHERE obs CONTAINS $pat) RETURN count(e) AS c",
        pat=pattern,
    ).single()["c"]
    print(f"[*] Neo4j entities with wave ingest obs: {count}")
    if dry:
        driver.close()
        sys.exit(0)
    session.run(
        """
        MATCH (e)
        WHERE any(obs IN e.observations WHERE obs CONTAINS $pat)
        SET e.observations = [obs IN e.observations WHERE NOT obs CONTAINS $pat]
        """,
        pat=pattern,
    )
    deleted = session.run(
        """
        MATCH (e)
        WHERE e.observations IS NOT NULL AND size(e.observations) = 0
        DETACH DELETE e
        RETURN count(e) AS c
        """
    ).single()["c"]
    print(f"[+] Neo4j: removed {deleted} orphaned nodes")
driver.close()
PY
)

if [[ -n "$NEO4J_PASS" ]]; then
  export NEO4J_URL NEO4J_USER NEO4J_PASS NEO4J_DB PROVENANCE_PATTERN
  export DRY_RUN="$DRY_RUN"
  python3 -c "$NEO4J_PY" || echo "[WARN] Neo4j purge step failed"
else
  echo "[WARN] NEO4J_PASSWORD not set — skipping Neo4j purge (export from gzmo.toml mcp_servers.env)"
fi

# --- Qdrant: delete purged UUIDs then full sync ---
if [[ "$DRY_RUN" -eq 0 && -n "${PURGED_IDS:-}" ]]; then
  echo "[*] Qdrant: deleting purged vault point IDs..."
  python3 <<PY || echo "[WARN] Qdrant point delete failed — run full sync"
import json, os, urllib.request
url = os.environ.get("QDRANT_URL", "$QDRANT_URL").rstrip("/")
collection = os.environ.get("QDRANT_COLLECTION", "$QDRANT_COLLECTION")
ids = """$PURGED_IDS""".strip().split()
if not ids:
    print("[*] No vault IDs to delete from Qdrant")
    raise SystemExit(0)
batch = 64
for i in range(0, len(ids), batch):
    chunk = ids[i:i+batch]
    body = json.dumps({"points": chunk}).encode()
    req = urllib.request.Request(
        f"{url}/collections/{collection}/points/delete",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    with urllib.request.urlopen(req, timeout=60) as resp:
        print(f"[+] Qdrant delete batch {i//batch + 1}: {resp.status}")
hp_coll = os.environ.get("QDRANT_HONEYPOT_COLLECTION", "honeypot")
for i in range(0, len(ids), batch):
    chunk = ids[i:i+batch]
    body = json.dumps({"points": chunk}).encode()
    req = urllib.request.Request(
        f"{url}/collections/{hp_coll}/points/delete",
        data=body,
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            print(f"[+] Qdrant honeypot delete {hp_coll} batch {i//batch + 1}: {resp.status}")
    except Exception as e:
        print(f"[WARN] Qdrant {hp_coll} delete: {e}")
PY
  export QDRANT_HONEYPOT_COLLECTION
  if [[ -x "$ROOT/scripts/sync-vault-to-qdrant.sh" ]]; then
    echo "[*] Qdrant full sync..."
    (cd "$ROOT" && ./scripts/sync-vault-to-qdrant.sh) || echo "[WARN] Qdrant sync failed"
  fi
else
  echo "[*] Qdrant: skip delete (dry-run or no IDs)"
fi

# --- Episodic: remove [ingest:wave_01_ lines ---
if [[ -f "$EPISODIC" ]]; then
  EPISODIC_MATCH=$(grep -c "\[ingest:${FILE_PREFIX}" "$EPISODIC" 2>/dev/null || echo 0)
  echo "[*] Episodic ingest lines to remove: $EPISODIC_MATCH"
  if [[ "$DRY_RUN" -eq 0 && "$EPISODIC_MATCH" -gt 0 ]]; then
    cp "$EPISODIC" "${EPISODIC}.pre-purge"
    grep -v "\[ingest:${FILE_PREFIX}" "$EPISODIC" > "${EPISODIC}.tmp" && mv "${EPISODIC}.tmp" "$EPISODIC"
    echo "[+] Episodic: stripped wave ingest lines"
  fi
else
  echo "[*] Episodic file not found: $EPISODIC"
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "=== DRY RUN complete — no changes made ==="
else
  echo "=== PURGE complete ==="
fi
