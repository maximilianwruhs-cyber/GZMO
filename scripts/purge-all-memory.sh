#!/usr/bin/env bash
# Memory reset: SQLite vault/honeypot/evidence, Neo4j, Qdrant, episodic ingest receipts.
# Use before slow wave re-ingest so the live store matches HEAD code (F1 evidence, v6 distill_dedup).
#
# Two modes:
#   FULL_PURGE    — vault.db reset; Neo4j *ingest provenance* strip; Qdrant honeypot+knowledge
#                   point delete; episodic ingest receipts archived.
#   NUCLEAR_PURGE — true clean slate. Everything in FULL_PURGE plus:
#                     * Neo4j full graph wipe (MATCH (n) DETACH DELETE n)
#                     * Qdrant knowledge_core collection also cleared
#                     * data/knowledge_core.db + candidates/export archived & removed
#                     * DREAMS.md archived & reset to a header stub
#                     * wiki/ archived & index/log reset to stubs
#                     * logs/migration-ingest-progress.txt removed
#                     * Redis distill queue (gzmo:distill:pending) flushed
#                     * Pi ~/.pi/agent/knowledge-state.json removed (forces full reindex)
#                     * data/Synapse/events.jsonl archived
#
# Usage:
#   ./scripts/purge-all-memory.sh --dry-run
#   ./scripts/purge-all-memory.sh --confirm FULL_PURGE
#   ./scripts/purge-all-memory.sh --dry-run --confirm NUCLEAR_PURGE   # preview nuclear counts
#   ./scripts/purge-all-memory.sh --confirm NUCLEAR_PURGE
#
# Stop gzmo daemon first (holds vault.db WAL):
#   pkill -f 'target/release/gzmo daemon' || true
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DRY_RUN=0
CONFIRM=""
NUCLEAR=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY_RUN=1 ;;
    --confirm) CONFIRM="${2:-}"; shift ;;
    *) echo "Unknown arg: $1" >&2; exit 1 ;;
  esac
  shift
done

case "$CONFIRM" in
  NUCLEAR_PURGE) NUCLEAR=1 ;;
  FULL_PURGE|"") : ;;
  *) echo "[!] Unknown --confirm token: $CONFIRM (expected FULL_PURGE or NUCLEAR_PURGE)" >&2; exit 1 ;;
esac

if [[ "$DRY_RUN" -eq 0 && "$CONFIRM" != "FULL_PURGE" && "$CONFIRM" != "NUCLEAR_PURGE" ]]; then
  echo "[!] Live purge requires: --confirm FULL_PURGE  (or --confirm NUCLEAR_PURGE for a true clean slate)" >&2
  echo "    Run with --dry-run first. Stop daemon before live purge." >&2
  exit 1
fi

MODE_LABEL="FULL"
[[ "$NUCLEAR" -eq 1 ]] && MODE_LABEL="NUCLEAR"

VAULT_DB="${GZMO_VAULT_DB:-$ROOT/data/vault.db}"
MEMORY_DIR="${GZMO_MEMORY_DIR:-$ROOT/memory}"
STAMP="$(date +%Y%m%d-%H%M%S)"
if [[ "$NUCLEAR" -eq 1 ]]; then
  BACKUP_DIR="$ROOT/data/backups/pre-nuclear-purge-$STAMP"
else
  BACKUP_DIR="$ROOT/data/backups/pre-full-purge-$STAMP"
fi

QDRANT_URL="${QDRANT_URL:-http://192.168.31.202:6333}"
QDRANT_COLLECTION="${QDRANT_COLLECTION:-knowledge}"
QDRANT_HONEYPOT_COLLECTION="${QDRANT_HONEYPOT_COLLECTION:-honeypot}"
QDRANT_CORE_COLLECTION="${QDRANT_CORE_COLLECTION:-knowledge_core}"

NEO4J_URL="${NEO4J_URL:-bolt://192.168.31.202:7687}"
NEO4J_USER="${NEO4J_USERNAME:-neo4j}"
NEO4J_PASS="${NEO4J_PASSWORD:-}"
NEO4J_DB="${NEO4J_DATABASE:-neo4j}"

REDIS_URL="${REDIS_URL:-redis://192.168.31.202:6379}"
REDIS_DISTILL_QUEUE="${REDIS_DISTILL_QUEUE:-gzmo:distill:pending}"
KNOWLEDGE_CORE_DB="${GZMO_KNOWLEDGE_CORE_DB:-$ROOT/data/knowledge_core.db}"
PI_KNOWLEDGE_STATE="${PI_KNOWLEDGE_STATE:-$HOME/.pi/agent/knowledge-state.json}"
SYNAPSE_BUS="${GZMO_SYNAPSE_BUS:-$ROOT/data/Synapse/events.jsonl}"

# Prefer the optional venv (neo4j + redis drivers) when present; fall back to system python3.
# Set up with: scripts/setup-optional-deps.sh
if [[ -x "$ROOT/scripts/.venv/bin/python" ]]; then
  PYBIN="$ROOT/scripts/.venv/bin/python"
else
  PYBIN="python3"
fi

if [[ -f "$ROOT/.env" ]]; then
  # shellcheck disable=SC1091
  set -a; source "$ROOT/.env"; set +a
  NEO4J_PASS="${NEO4J_PASSWORD:-$NEO4J_PASS}"
fi

if pgrep -f 'target/release/gzmo daemon' >/dev/null 2>&1; then
  if [[ "$DRY_RUN" -eq 0 ]]; then
    echo "[!] gzmo daemon is running — stop it first:" >&2
    echo "    pkill -TERM -f 'target/release/gzmo daemon'" >&2
    exit 1
  else
    echo "[WARN] gzmo daemon is running (purge would fail on live WAL)"
  fi
fi

echo "=== purge-all-memory [$MODE_LABEL] $(date -Is) ==="
echo "dry_run=$DRY_RUN nuclear=$NUCLEAR vault=$VAULT_DB"

# --- SQLite counts ---
if [[ -f "$VAULT_DB" ]]; then
  export VAULT_DB
  "$PYBIN" - <<'PY'
import os, sqlite3
db = os.environ["VAULT_DB"]
conn = sqlite3.connect(db)
tables = [
    ("semantic_vault", "SELECT COUNT(*) FROM semantic_vault"),
    ("honeypot", "SELECT COUNT(*) FROM honeypot"),
    ("honeypot_latest", "SELECT COUNT(*) FROM honeypot WHERE is_latest=1"),
    ("evidence", "SELECT COUNT(*) FROM evidence"),
    ("quarantine_vault", "SELECT COUNT(*) FROM quarantine_vault"),
]
for name, sql in tables:
    try:
        print(f"[*] SQLite {name}: {conn.execute(sql).fetchone()[0]}")
    except Exception as e:
        print(f"[*] SQLite {name}: skip ({e})")
conn.close()
PY
else
  echo "[*] SQLite vault.db not found (already empty)"
fi

# --- Neo4j counts ---
if [[ -n "$NEO4J_PASS" ]]; then
  export NEO4J_URL NEO4J_USER NEO4J_PASS NEO4J_DB
  "$PYBIN" - <<'PY' || echo "[WARN] Neo4j count failed"
import os, sys
try:
    from neo4j import GraphDatabase
except ImportError:
    print("[WARN] pip install neo4j for Neo4j purge")
    sys.exit(0)
driver = GraphDatabase.driver(os.environ["NEO4J_URL"], auth=(os.environ["NEO4J_USER"], os.environ["NEO4J_PASS"]))
with driver.session(database=os.environ.get("NEO4J_DB", "neo4j")) as s:
    prov = s.run("MATCH (e) WHERE any(obs IN e.observations WHERE obs CONTAINS '[provenance]') RETURN count(e) AS c").single()["c"]
    wave = s.run("MATCH (e) WHERE any(obs IN e.observations WHERE obs CONTAINS 'source=wave_') RETURN count(e) AS c").single()["c"]
    print(f"[*] Neo4j entities with [provenance] obs: {prov}")
    print(f"[*] Neo4j entities with source=wave_ obs: {wave}")
driver.close()
PY
else
  echo "[WARN] NEO4J_PASSWORD not set — Neo4j purge skipped"
fi

# Qdrant collections cleared: honeypot + knowledge always; knowledge_core only in nuclear mode.
QDRANT_COLLECTIONS=("$QDRANT_HONEYPOT_COLLECTION" "$QDRANT_COLLECTION")
[[ "$NUCLEAR" -eq 1 ]] && QDRANT_COLLECTIONS+=("$QDRANT_CORE_COLLECTION")

# --- Qdrant counts ---
for coll in "${QDRANT_COLLECTIONS[@]}"; do
  "$PYBIN" - <<PY || echo "[WARN] Qdrant $coll unreachable"
import json, os, urllib.request
url = os.environ.get("QDRANT_URL", "$QDRANT_URL").rstrip("/")
coll = "$coll"
try:
    d = json.load(urllib.request.urlopen(f"{url}/collections/{coll}", timeout=10))
    print(f"[*] Qdrant {coll}: {d['result']['points_count']} points")
except Exception as e:
    print(f"[WARN] Qdrant {coll}: {e}")
PY
done

# --- Episodic ingest receipt lines ---
if [[ -d "$MEMORY_DIR" ]]; then
  n=$(grep -rc '\[ingest:' "$MEMORY_DIR"/*.md 2>/dev/null | awk -F: '{s+=$2} END{print s+0}' || echo 0)
  echo "[*] Episodic [ingest: lines across memory/*.md: $n"
fi

# --- Nuclear-only previews ---
if [[ "$NUCLEAR" -eq 1 ]]; then
  if [[ -n "$NEO4J_PASS" ]]; then
    export NEO4J_URL NEO4J_USER NEO4J_PASS NEO4J_DB
    "$PYBIN" - <<'PY' || echo "[WARN] Neo4j total count failed"
import os, sys
try:
    from neo4j import GraphDatabase
except ImportError:
    print("[WARN] pip install neo4j for Neo4j purge")
    sys.exit(0)
driver = GraphDatabase.driver(os.environ["NEO4J_URL"], auth=(os.environ["NEO4J_USER"], os.environ["NEO4J_PASS"]))
with driver.session(database=os.environ.get("NEO4J_DB", "neo4j")) as s:
    nodes = s.run("MATCH (n) RETURN count(n) AS c").single()["c"]
    rels = s.run("MATCH ()-[r]->() RETURN count(r) AS c").single()["c"]
    print(f"[*] Neo4j TOTAL nodes: {nodes}, relations: {rels} (NUCLEAR wipes ALL)")
driver.close()
PY
  fi
  if [[ -f "$KNOWLEDGE_CORE_DB" ]]; then
    python3 -c "
import sqlite3
try:
    c=sqlite3.connect('$KNOWLEDGE_CORE_DB')
    print('[*] knowledge_core.db cards:', c.execute('SELECT COUNT(*) FROM knowledge_core').fetchone()[0])
except Exception as e:
    print('[*] knowledge_core.db: skip (%s)' % e)
" || true
  else
    echo "[*] knowledge_core.db: not present"
  fi
  if [[ -f "$ROOT/DREAMS.md" ]]; then
    echo "[*] DREAMS.md lines: $(wc -l < "$ROOT/DREAMS.md")"
  fi
  if [[ -d "$ROOT/wiki" ]]; then
    echo "[*] wiki/*.md files: $(find "$ROOT/wiki" -name '*.md' | wc -l)"
  fi
  [[ -f "$PI_KNOWLEDGE_STATE" ]] && echo "[*] Pi knowledge-state.json: present (will be removed)"
  [[ -f "$SYNAPSE_BUS" ]] && echo "[*] Synapse events.jsonl lines: $(wc -l < "$SYNAPSE_BUS")"
fi

if [[ "$DRY_RUN" -eq 1 ]]; then
  echo "=== DRY RUN complete — no changes made ($MODE_LABEL) ==="
  exit 0
fi

mkdir -p "$BACKUP_DIR"
echo "[*] Backup dir: $BACKUP_DIR"

if [[ -f "$VAULT_DB" ]]; then
  cp -a "$VAULT_DB" "$BACKUP_DIR/vault.db"
  [[ -f "${VAULT_DB}-wal" ]] && cp -a "${VAULT_DB}-wal" "$BACKUP_DIR/" || true
  [[ -f "${VAULT_DB}-shm" ]] && cp -a "${VAULT_DB}-shm" "$BACKUP_DIR/" || true
  rm -f "$VAULT_DB" "${VAULT_DB}-wal" "${VAULT_DB}-shm"
  echo "[+] SQLite: removed vault.db (backup in $BACKUP_DIR)"
fi

# Fresh vault on next gzmo open (migrations v1–v6, empty tables)
unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli -q 2>/dev/null || true
if [[ -x "$ROOT/target/release/gzmo" ]]; then
  RUST_LOG=error "$ROOT/target/release/gzmo" memory status >/dev/null 2>&1 || true
  echo "[+] SQLite: recreated empty vault via gzmo memory status"
  python3 -c "
import sqlite3
c=sqlite3.connect('$VAULT_DB')
print('[*] user_version', c.execute('PRAGMA user_version').fetchone()[0])
print('[*] semantic_vault', c.execute('SELECT COUNT(*) FROM semantic_vault').fetchone()[0])
"
fi

# Neo4j: FULL_PURGE strips ingest provenance; NUCLEAR_PURGE wipes the whole graph.
if [[ -n "$NEO4J_PASS" ]]; then
  export NEO4J_URL NEO4J_USER NEO4J_PASS NEO4J_DB NEO4J_BACKUP_DIR="$BACKUP_DIR" GZMO_NUCLEAR="$NUCLEAR"
  "$PYBIN" - <<'PY' || echo "[WARN] Neo4j purge failed"
import json, os
from neo4j import GraphDatabase
nuclear = os.environ.get("GZMO_NUCLEAR") == "1"
driver = GraphDatabase.driver(os.environ["NEO4J_URL"], auth=(os.environ["NEO4J_USER"], os.environ["NEO4J_PASS"]))
with driver.session(database=os.environ.get("NEO4J_DB", "neo4j")) as s:
    if nuclear:
        # Export a JSON snapshot of all nodes before the wipe (graph is irreversible otherwise).
        backup_dir = os.environ.get("NEO4J_BACKUP_DIR", ".")
        os.makedirs(backup_dir, exist_ok=True)
        rows = s.run("MATCH (n) RETURN labels(n) AS labels, properties(n) AS props")
        nodes = [{"labels": r["labels"], "props": r["props"]} for r in rows]
        with open(os.path.join(backup_dir, "neo4j-nodes.json"), "w") as fh:
            json.dump(nodes, fh, default=str, ensure_ascii=False, indent=0)
        deleted = s.run("MATCH (n) DETACH DELETE n RETURN count(n) AS c").single()["c"]
        print(f"[+] Neo4j: NUCLEAR wipe removed {deleted} nodes (snapshot: neo4j-nodes.json)")
    else:
        s.run("""
            MATCH (e)
            WHERE any(obs IN e.observations WHERE obs CONTAINS '[provenance]' OR obs CONTAINS 'source=wave_')
            SET e.observations = [obs IN e.observations WHERE NOT (obs CONTAINS '[provenance]' OR obs CONTAINS 'source=wave_')]
        """)
        deleted = s.run("""
            MATCH (e)
            WHERE e.observations IS NOT NULL AND size(e.observations) = 0
            DETACH DELETE e
            RETURN count(e) AS c
        """).single()["c"]
        print(f"[+] Neo4j: removed {deleted} orphaned nodes after provenance strip")
driver.close()
PY
fi

# Qdrant: delete all points (honeypot + knowledge; + knowledge_core in nuclear mode)
for coll in "${QDRANT_COLLECTIONS[@]}"; do
  "$PYBIN" - <<PY || echo "[WARN] Qdrant delete $coll failed"
import json, os, urllib.request
url = os.environ.get("QDRANT_URL", "$QDRANT_URL").rstrip("/")
coll = "$coll"
body = json.dumps({"filter": {"must": [{"key": "id", "match": {"any": []}}]}}).encode()
# Delete all points via filter match-all workaround: scroll + delete by id
ids = []
offset = None
while True:
    payload = {"limit": 256, "with_payload": False, "with_vector": False}
    if offset:
        payload["offset"] = offset
    req = urllib.request.Request(
        f"{url}/collections/{coll}/points/scroll",
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = json.load(resp)
    except Exception as e:
        print(f"[WARN] scroll {coll}: {e}")
        break
    pts = data.get("result", {}).get("points", [])
    if not pts:
        break
    ids.extend(str(p["id"]) for p in pts)
    offset = data.get("result", {}).get("next_page_offset")
    if not offset:
        break
if not ids:
    print(f"[*] Qdrant {coll}: already empty")
else:
    batch = 128
    for i in range(0, len(ids), batch):
        chunk = ids[i:i+batch]
        req = urllib.request.Request(
            f"{url}/collections/{coll}/points/delete",
            data=json.dumps({"points": chunk}).encode(),
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with urllib.request.urlopen(req, timeout=120) as resp:
            print(f"[+] Qdrant {coll}: deleted batch {i//batch+1} ({len(chunk)} ids) status={resp.status}")
PY
done

# Episodic: archive memory dir, start fresh daily stub
if [[ -d "$MEMORY_DIR" ]]; then
  cp -a "$MEMORY_DIR" "$BACKUP_DIR/memory"
  today="$(date +%Y-%m-%d)"
  find "$MEMORY_DIR" -maxdepth 1 -name '*.md' -type f ! -name "Vault_*.md" -delete 2>/dev/null || true
  echo "# Episodic log — reset $STAMP" > "$MEMORY_DIR/$today.md"
  echo "[+] Episodic: archived to $BACKUP_DIR/memory; fresh $today.md"
fi

# Eval artifacts: archive stale report (dry-run baseline ≠ live store)
EVAL_BACKUP="$BACKUP_DIR/ingest-quality-reports"
mkdir -p "$EVAL_BACKUP"
for f in report.json pipeline-lock.json; do
  if [[ -f "$ROOT/scripts/ingest-quality/$f" ]]; then
    cp -a "$ROOT/scripts/ingest-quality/$f" "$EVAL_BACKUP/"
  fi
done
if [[ -d "$ROOT/scripts/ingest-quality/reports" ]]; then
  cp -a "$ROOT/scripts/ingest-quality/reports" "$EVAL_BACKUP/"
fi
echo "[+] Eval artifacts archived to $EVAL_BACKUP (report.json left in place — re-run replay after re-ingest)"

# ─── Nuclear-only: derived stores, cognition history, queues, Pi state ───
if [[ "$NUCLEAR" -eq 1 ]]; then
  echo ""
  echo "--- NUCLEAR extensions ---"

  # M5 mature core (derived) — archive + remove
  for f in "$KNOWLEDGE_CORE_DB" "$ROOT/data/knowledge_core.candidates.json" "$ROOT/data/knowledge_core_export.md"; do
    if [[ -f "$f" ]]; then
      cp -a "$f" "$BACKUP_DIR/$(basename "$f")"
      rm -f "$f"
      echo "[+] knowledge_core: archived & removed $(basename "$f")"
    fi
  done

  # DREAMS.md — archive + reset to header stub
  if [[ -f "$ROOT/DREAMS.md" ]]; then
    cp -a "$ROOT/DREAMS.md" "$BACKUP_DIR/DREAMS.md"
    printf '# Dream Consolidation — reset %s\n' "$STAMP" > "$ROOT/DREAMS.md"
    echo "[+] DREAMS.md: archived & reset to header stub"
  fi

  # wiki/ — archive + reset index/log stubs (emit-only layer; rebuilt by WikiEngine)
  if [[ -d "$ROOT/wiki" ]]; then
    cp -a "$ROOT/wiki" "$BACKUP_DIR/wiki"
    find "$ROOT/wiki" -maxdepth 1 -name '*.md' -type f -delete 2>/dev/null || true
    printf '# GZMO Wiki Index — reset %s\n' "$STAMP" > "$ROOT/wiki/index.md"
    printf '# GZMO Wiki Log — reset %s\n' "$STAMP" > "$ROOT/wiki/log.md"
    echo "[+] wiki/: archived & index/log reset to stubs"
  fi

  # Stale migration progress
  if [[ -f "$ROOT/logs/migration-ingest-progress.txt" ]]; then
    cp -a "$ROOT/logs/migration-ingest-progress.txt" "$BACKUP_DIR/" 2>/dev/null || true
    rm -f "$ROOT/logs/migration-ingest-progress.txt"
    echo "[+] logs: removed stale migration-ingest-progress.txt"
  fi

  # Synapse bus — archive (append-only; never consumed for state, but reset for clean slate)
  if [[ -f "$SYNAPSE_BUS" ]]; then
    cp -a "$SYNAPSE_BUS" "$BACKUP_DIR/Synapse-events.jsonl"
    : > "$SYNAPSE_BUS"
    echo "[+] Synapse: archived & truncated events.jsonl"
  fi

  # Redis distill queue — flush
  "$PYBIN" - <<PY || echo "[WARN] Redis distill queue flush skipped"
import os
try:
    import redis  # type: ignore
except ImportError:
    print("[WARN] pip install redis to flush distill queue (gzmo:distill:pending)")
else:
    try:
        r = redis.Redis.from_url(os.environ.get("REDIS_URL", "$REDIS_URL"))
        n = r.delete("$REDIS_DISTILL_QUEUE")
        print(f"[+] Redis: deleted distill queue key (existed={n})")
    except Exception as e:
        print(f"[WARN] Redis flush: {e}")
PY

  # Pi reindex fingerprints — remove to force a full Pi KB reindex on next run
  if [[ -f "$PI_KNOWLEDGE_STATE" ]]; then
    cp -a "$PI_KNOWLEDGE_STATE" "$BACKUP_DIR/pi-knowledge-state.json" 2>/dev/null || true
    rm -f "$PI_KNOWLEDGE_STATE"
    echo "[+] Pi: removed knowledge-state.json (forces full reindex)"
  fi
fi

echo ""
echo "=== $MODE_LABEL PURGE complete ==="
echo "Backup: $BACKUP_DIR"
if [[ "$NUCLEAR" -eq 1 ]]; then
  echo "Next: curate wave-1 source files, then:"
  echo "      ./scripts/run-curated-ingest.sh --manifest scripts/ingest-quality/curated-wave-01.manifest --dry-run"
  echo "      see docs/MIGRATION_INGEST_RUNBOOK.md (curation-first rebuild)"
else
  echo "Next: ./scripts/slow-reingest-wave.sh --dry-run"
  echo "      ./scripts/slow-reingest-wave.sh --batch 3 --sleep 60"
fi
