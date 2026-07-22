#!/usr/bin/env bash
# O11 — honeypot / ripen visibility census (Experience F). Local-first.
#   bash scripts/honeypot-lifecycle-check.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/honeypot-lifecycle"
VAULT="${VAULT_PATH:-$DATA/vault.db}"
mkdir -p "$OUT"

python3 - "$VAULT" "$OUT" <<'PY'
import json, sqlite3, sys
from datetime import datetime, timezone
from pathlib import Path
vault, out = Path(sys.argv[1]), Path(sys.argv[2])
if not vault.is_file():
    print(json.dumps({"ok": False, "error": f"missing vault {vault}"})); raise SystemExit(1)
conn = sqlite3.connect(str(vault))
def q(sql):
    try:
        return conn.execute(sql).fetchone()[0]
    except Exception:
        return None
latest = q("SELECT COUNT(*) FROM honeypot WHERE is_latest=1")
total = q("SELECT COUNT(*) FROM honeypot")
ge1 = q("SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>=1")
ge3 = q("SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>=3")
# ripen dual-ish: conf>=0.9 and recall>=3 and origin in allowlist when columns exist
dual = q(
    "SELECT COUNT(*) FROM honeypot WHERE is_latest=1 AND recall_count>=3 "
    "AND confidence>=0.90 AND origin IN ('ingest','verified_dream','session_distill','honeypot')"
)
origins = {}
try:
    for o, c in conn.execute(
        "SELECT COALESCE(origin,'(null)'), COUNT(*) FROM honeypot WHERE is_latest=1 GROUP BY origin"
    ):
        origins[str(o)] = c
except Exception:
    pass
ok = bool(latest and latest > 0)
payload = {
    "schema": "gzmo.honeypot_lifecycle.check/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "ok": ok,
    "latest": latest,
    "total_rows": total,
    "recall_ge1": ge1,
    "recall_ge3": ge3,
    "ripen_dual_approx": dual,
    "origins": origins,
    "advice": "honeypot_lifecycle_ok — Experience F counts demable" if ok else "honeypot_lifecycle_fail",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
md = [
    "# Honeypot lifecycle",
    "",
    f"- latest: **{latest}**",
    f"- recall≥1 / ≥3: {ge1} / {ge3}",
    f"- ripen dual≈: {dual}",
    f"- origins: {origins}",
    "",
]
(out / "latest.md").write_text("\n".join(md) + "\n")
print(json.dumps(payload, indent=2))
raise SystemExit(0 if ok else 1)
PY
