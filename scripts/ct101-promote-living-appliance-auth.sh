#!/usr/bin/env bash
# Unwrap CT101 live-secrets HOLD: move inline NEO4J_AUTH out of compose into .env.
# Does not recreate sidecars unless --recreate. Never prints the password.
# Also syncs NEO4J_PASSWORD into /opt/gzmo/.env when missing.
#
#   bash scripts/ct101-promote-living-appliance-auth.sh
#   bash scripts/ct101-promote-living-appliance-auth.sh --recreate   # apply compose env change
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOST="${CT101_SSH_HOST:-ct101}"
CLUSTER="${CT101_DATABASE_CLUSTER:-/opt/database-cluster}"
RECREATE=0
[[ "${1:-}" == "--recreate" ]] && RECREATE=1

echo "[*] promote living appliance auth on $HOST ($CLUSTER)"

ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
  "CLUSTER=$(printf '%q' "$CLUSTER") RECREATE=$(printf '%q' "$RECREATE") bash -s" <<'REMOTE'
set -euo pipefail
CLUSTER="${CLUSTER:?}"
cd "$CLUSTER"

ts="$(date -u +%Y%m%dT%H%M%SZ)"
cp -a docker-compose.yml "docker-compose.yml.bak.${ts}"

# Capture auth from running container (preferred) or existing compose literal.
AUTH="$(
  docker inspect sidecar-neo4j --format '{{range .Config.Env}}{{println .}}{{end}}' 2>/dev/null \
    | grep '^NEO4J_AUTH=' | head -1 | cut -d= -f2- || true
)"
if [[ -z "$AUTH" ]]; then
  AUTH="$(grep -E '^\s*- NEO4J_AUTH=neo4j/' docker-compose.yml | head -1 | sed -E 's/.*NEO4J_AUTH=//' | tr -d '"' || true)"
fi
[[ -n "$AUTH" && "$AUTH" == neo4j/* ]] || { echo "[!] could not resolve NEO4J_AUTH" >&2; exit 1; }

umask 077
printf 'NEO4J_AUTH=%s\n' "$AUTH" > .env
chmod 600 .env
echo "[OK] wrote $CLUSTER/.env (mode 600, value not printed)"

# Ensure daemon env has password half
PASS="${AUTH#neo4j/}"
if [[ -f /opt/gzmo/.env ]]; then
  if grep -qE '^NEO4J_PASSWORD=' /opt/gzmo/.env; then
    # replace in place without echo
    tmp="$(mktemp)"
    grep -vE '^NEO4J_PASSWORD=' /opt/gzmo/.env >"$tmp" || true
    printf 'NEO4J_PASSWORD=%s\n' "$PASS" >>"$tmp"
    chmod 600 "$tmp"
    mv "$tmp" /opt/gzmo/.env
    chown maximilian:maximilian /opt/gzmo/.env 2>/dev/null || true
    echo "[OK] refreshed NEO4J_PASSWORD in /opt/gzmo/.env"
  else
    printf 'NEO4J_PASSWORD=%s\n' "$PASS" >> /opt/gzmo/.env
    echo "[OK] appended NEO4J_PASSWORD to /opt/gzmo/.env"
  fi
fi

# Rewrite compose: inline NEO4J_AUTH → ${NEO4J_AUTH:?…}
python3 - <<'PY'
from pathlib import Path
import re
p = Path("docker-compose.yml")
text = p.read_text(encoding="utf-8")
text2, n = re.subn(
    r"^(\s*- NEO4J_AUTH=)neo4j/\S+",
    r"\1${NEO4J_AUTH:?set NEO4J_AUTH in .env}",
    text,
    count=1,
    flags=re.M,
)
if n == 0 and "${NEO4J_AUTH" not in text:
    raise SystemExit("no inline NEO4J_AUTH line to rewrite")
p.write_text(text2, encoding="utf-8")
print(f"[OK] compose auth style → env substitution (replacements={n})")
PY

# Pin qdrant tag to running version (avoid downgrade)
ver="$(curl -fsS http://127.0.0.1:6333/ | python3 -c 'import json,sys; print(json.load(sys.stdin).get("version",""))')"
if [[ -n "$ver" ]]; then
  python3 - <<PY
from pathlib import Path
import re
p = Path("docker-compose.yml")
text = p.read_text(encoding="utf-8")
text2, n = re.subn(
    r"(^\s+image:\s*)qdrant/qdrant:\S+",
    r"\1qdrant/qdrant:v${ver}",
    text,
    count=1,
    flags=re.M,
)
if n == 0:
    text2, n = re.subn(
        r"(^\s+image:\s*)qdrant/qdrant\s*$",
        r"\1qdrant/qdrant:v${ver}",
        text,
        count=1,
        flags=re.M,
    )
p.write_text(text2, encoding="utf-8")
print(f"[OK] live qdrant image → qdrant/qdrant:v${ver} (replacements={n})")
PY
fi

if [[ "${RECREATE}" == "1" ]]; then
  echo "[*] docker compose up -d (neo4j recreate to pick env file)"
  docker compose config >/dev/null
  docker compose up -d
  echo "[OK] compose up done"
else
  echo "[*] skipped recreate — run with --recreate to apply container env from .env"
  echo "    (compose file + .env are already migrated; pin-check can PASS secrets style)"
fi

# Prove we did not leave a literal in compose
if grep -E 'NEO4J_AUTH=neo4j/' docker-compose.yml >/dev/null; then
  echo "[!] still have inline NEO4J_AUTH literal" >&2
  exit 1
fi
echo "[OK] no inline NEO4J_AUTH literal in compose"
REMOTE

echo ""
echo "Next:"
echo "  bash scripts/ct101-sync-living-appliance.sh   # refresh staged pin (qdrant tag)"
echo "  bash scripts/ct101-living-appliance-pin-check.sh"
echo "  # optional apply: ssh $HOST 'cd $CLUSTER && docker compose up -d'"
