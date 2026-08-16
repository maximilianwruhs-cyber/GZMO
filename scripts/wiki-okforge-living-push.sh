#!/usr/bin/env bash
# Read-only dump of living honeypot facts → local OKForge OKCP (no vault write).
# Never starts gzmo-serve. Refuses if workstation serve is active.
#
#   bash scripts/wiki-okforge-living-push.sh
#   bash scripts/wiki-okforge-living-push.sh --dry-run
#   bash scripts/wiki-okforge-living-push.sh --limit 20
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/okforge-observatory"
HOST="${CT101_SSH_HOST:-ct101}"
VAULT_DB="${CT101_VAULT_DB:-/opt/gzmo/data/vault.db}"
LIMIT="${WIKI_PUSH_LIMIT:-40}"
ORIGIN="${WIKI_PUSH_ORIGIN:-living}"
DRY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run) DRY=1; shift ;;
    --limit) LIMIT="$2"; shift 2 ;;
    --origin) ORIGIN="$2"; shift 2 ;;
    *) echo "unknown flag: $1" >&2; exit 2 ;;
  esac
done

mkdir -p "$OUT"
DUMP="$OUT/living-honeypot-drafts.json"

serve="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
serve="$(printf '%s\n' "$serve" | head -1)"
if [[ "$serve" == "active" ]]; then
  echo "[!] refused_dual_writer — stop workstation gzmo-serve" >&2
  exit 1
fi

ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" 'true' >/dev/null
ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" python3 - "$VAULT_DB" "$LIMIT" <<'PY' >"$DUMP"
import json, sqlite3, sys
vault, limit = sys.argv[1], int(sys.argv[2])
conn = sqlite3.connect(f"file:{vault}?mode=ro", uri=True)
cur = conn.cursor()
rows = []
try:
    cur.execute(
        "SELECT id, content FROM honeypot WHERE is_latest = 1 ORDER BY rowid DESC LIMIT ?",
        (limit,),
    )
    rows = [{"id": i, "content": c} for i, c in cur.fetchall()]
except sqlite3.Error:
    cur.execute(
        "SELECT id, content FROM semantic_vault ORDER BY created_at DESC LIMIT ?",
        (limit,),
    )
    rows = [{"id": i, "content": c} for i, c in cur.fetchall()]
print(json.dumps({"facts": rows}, ensure_ascii=False))
PY

n="$(python3 -c "import json;print(len(json.load(open('$DUMP')).get('facts') or []))")"
if [[ "$n" -eq 0 ]]; then
  echo "[!] living dump empty — no honeypot/vault facts" >&2
  exit 1
fi

BIN="${GZMO_BIN:-}"
if [[ -z "$BIN" ]]; then
  if [[ -x "$ROOT/target/release/gzmo" ]]; then
    BIN="$ROOT/target/release/gzmo"
  elif command -v gzmo >/dev/null 2>&1; then
    BIN="$(command -v gzmo)"
  else
    echo "[!] gzmo binary not found" >&2
    exit 1
  fi
fi

args=(wiki push --from-json "$DUMP" --origin "$ORIGIN" --limit "$LIMIT" --meta "$DATA/wiki-push-latest.json")
[[ "$DRY" == "1" ]] && args+=(--dry-run)
exec "$BIN" "${args[@]}"
