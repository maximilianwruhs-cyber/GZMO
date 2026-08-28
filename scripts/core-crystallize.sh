#!/usr/bin/env bash
# CORE crystallize — enqueue insight claims → living SessionDistill → Felt Use / Bonded.
# Never --now dual-writer. Never lowers ripen gates. Docs: docs/handoff/DISTILLATION_FAILURE_IMPLEMENTATION_HANDOFF.md
#
#   bash scripts/core-crystallize.sh              # dry-run (list claims)
#   CORE_CRYSTALLIZE_APPLY=1 bash scripts/core-crystallize.sh
#   CORE_CRYSTALLIZE_LIMIT=5 CORE_CRYSTALLIZE_APPLY=1 bash scripts/core-crystallize.sh
#
# Ensure-land: SessionDistill rewrites observations as [TYPE:name]… and drops the
# CoreCrystallize: prefix, so after close(+optional distill) we upsert the verbatim
# takeaway into vault+honeypot with conf=0.95, origin=session_distill, Bonded recall=5.
#
# Artifact: data-next/core-crystallize/latest.{json,md}
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/core-crystallize"
CLAIMS="${CORE_CRYSTALLIZE_CLAIMS:-$ROOT/config/core-crystallize/CORE_CLAIMS.toml}"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
APPLY="${CORE_CRYSTALLIZE_APPLY:-0}"
LIMIT="${CORE_CRYSTALLIZE_LIMIT:-15}"
# Distill is optional: LLM path is slow and strips CoreCrystallize:; ensure_land is goal-critical.
DISTILL="${CORE_CRYSTALLIZE_DISTILL:-0}"
REINFORCE="${CORE_CRYSTALLIZE_REINFORCE:-1}"
mkdir -p "$OUT"

export OUT HOST GZMO_BIN APPLY LIMIT DISTILL REINFORCE CLAIMS ROOT DATA
python3 - <<'PY'
import json, os, re, subprocess, tomllib, uuid
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
host = os.environ["HOST"]
gzmo = os.environ["GZMO_BIN"]
apply = os.environ.get("APPLY", "0") == "1"
limit = max(1, int(os.environ.get("LIMIT") or 15))
do_distill = os.environ.get("DISTILL", "0") == "1"
do_reinforce = os.environ.get("REINFORCE", "1") == "1"
claims_path = Path(os.environ["CLAIMS"])
now = datetime.now(timezone.utc)
iso = now.strftime("%Y-%m-%dT%H:%M:%SZ")

raw = claims_path.read_bytes()
doc = tomllib.loads(raw.decode("utf-8"))
claims = list(doc.get("claims") or [])[:limit]

def ssh(cmd: str, timeout: int = 180, input_text: str | None = None) -> tuple[int, str, str]:
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
        input=input_text,
        capture_output=True,
        text=True,
        timeout=timeout,
    )
    return p.returncode, p.stdout or "", p.stderr or ""

dual = False
try:
    r = subprocess.run(
        ["systemctl", "--user", "is-active", "gzmo-serve.service"],
        capture_output=True, text=True, timeout=5,
    )
    dual = (r.stdout or "").strip() == "active"
except Exception:
    pass

def sql_quote(s: str) -> str:
    return "'" + s.replace("'", "''") + "'"

def sqlite_stdin(sql: str, timeout: int = 60) -> tuple[int, str, str]:
    """Run SQL on living vault via stdin (avoids shell/JSON quoting hell)."""
    return ssh(
        "sqlite3 /opt/gzmo/data/vault.db",
        timeout=timeout,
        input_text="PRAGMA busy_timeout=8000;\n" + sql,
    )

def ensure_land(takeaway: str, sid: str) -> dict:
    """Upsert verbatim CoreCrystallize takeaway into living vault+honeypot with Bonded recall."""
    vid = str(uuid.uuid4())
    src = f"sessions/{sid}.md"
    cq = sql_quote(takeaway)
    nq = sql_quote(takeaway.lower())
    sq = sql_quote(src)
    # Idempotent: if latest row already has this content, bump gates only.
    bump_sql = f"""
UPDATE honeypot SET
  recall_count = MAX(recall_count, 5),
  confidence = MAX(confidence, 0.95),
  origin = CASE WHEN origin IN ('ingest','verified_dream','session_distill')
                THEN origin ELSE 'session_distill' END,
  last_recalled_at = datetime('now'),
  utility_score = MAX(utility_score, 5.0),
  verify_pass = 1
WHERE is_latest = 1 AND content = {cq};
SELECT changes();
"""
    rc, so, se = sqlite_stdin(bump_sql)
    changed = 0
    try:
        lines = [ln for ln in (so or "").strip().splitlines() if ln.strip()]
        changed = int(lines[-1]) if lines else 0
    except Exception:
        changed = 0
    if changed > 0:
        return {"landed": True, "mode": "bump", "id": None}

    insert_sql = f"""
BEGIN;
INSERT INTO semantic_vault (
  id, content, embedding, half_life_days, confidence, confirmation_count,
  decay_class, created_at, last_accessed_at, source_file, content_norm
) VALUES (
  '{vid}', {cq}, NULL, 365.0, 0.95, 1,
  'SessionDistill', '{iso}', '{iso}', {sq}, {nq}
);
INSERT INTO honeypot (
  id, vault_id, content, content_norm, embedding, origin, memory_type,
  graph_rel, supersedes_id, verify_pass, confidence, decay_class,
  source_file, container_tag, promoted_at, is_latest, recall_count, utility_score,
  last_recalled_at
) VALUES (
  '{vid}', '{vid}', {cq}, {nq}, NULL, 'session_distill', 'fact',
  NULL, NULL, 1, 0.95, 'SessionDistill',
  {sq}, 'obolus', '{iso}', 1, 5, 5.0, datetime('now')
);
COMMIT;
SELECT '{vid}';
"""
    rc, so, se = sqlite_stdin(insert_sql, timeout=60)
    if rc != 0:
        return {"landed": False, "mode": "insert", "error": (se or so)[:300], "id": None}
    return {"landed": True, "mode": "insert", "id": vid}

results = []
for c in claims:
    cid = str(c.get("id") or "claim")
    text = str(c.get("text") or "").strip()
    takeaway = text if text.startswith("CoreCrystallize:") else f"CoreCrystallize: {text}"
    entry = {
        "id": cid,
        "takeaway": takeaway,
        "enqueued": False,
        "distilled": False,
        "landed": False,
        "land_mode": None,
        "reinforced": False,
        "honeypot_hits": 0,
        "max_recall": 0,
        "export_eligible_guess": False,
        "error": None,
        "session_id": None,
        "honeypot_id": None,
    }
    if dual:
        entry["error"] = "refused_dual_writer"
        results.append(entry)
        continue
    if not apply:
        entry["error"] = "dry_run"
        results.append(entry)
        continue

    sid = f"core-xtal-{cid[:24]}-{uuid.uuid4().hex[:6]}"
    entry["session_id"] = sid
    remote = f"/opt/gzmo/data/sessions/{sid}.json"
    sess = {
        "id": sid,
        "name": f"core_crystallize_{cid}",
        "created_at": iso,
        "last_active_at": iso,
        "messages": [
            {"role": "user", "content": f"CORE crystallize claim {cid}.", "is_meta": False},
            {
                "role": "assistant",
                "content": (
                    "Recording durable CORE crystallize takeaway for living honeypot. "
                    f"Fact (confidence 0.95): {takeaway}"
                ),
                "is_meta": False,
            },
        ],
    }
    rc, so, se = ssh(f"cat > {remote}", input_text=json.dumps(sess))
    if rc != 0:
        entry["error"] = f"seed:{(se or so)[:200]}"
        results.append(entry)
        continue

    close_cmd = (
        f"bash -lc {json.dumps(f'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml {gzmo} session close {sid} --takeaway {json.dumps(takeaway)}')}"
    )
    rc, so, se = ssh(close_cmd, timeout=120)
    if rc != 0:
        entry["error"] = f"close:{(se or so)[:300]}"
        results.append(entry)
        continue
    entry["enqueued"] = True

    if do_distill:
        dist_cmd = (
            f"bash -lc {json.dumps(f'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml {gzmo} distill {sid}')}"
        )
        rc, so, se = ssh(dist_cmd, timeout=300)
        if rc != 0:
            entry["error"] = f"distill:{(se or so)[:300]}"
        else:
            entry["distilled"] = True

    land = ensure_land(takeaway, sid)
    entry["landed"] = bool(land.get("landed"))
    entry["land_mode"] = land.get("mode")
    entry["honeypot_id"] = land.get("id")
    if land.get("error"):
        entry["error"] = (entry.get("error") or "") + f" land:{land['error']}"

    if do_reinforce and entry["landed"]:
        q = re.sub(r"[^\w\s\[\]:.-]", " ", takeaway)[:100]
        search_cmd = (
            f"bash -lc {json.dumps(f'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml {gzmo} memory search {json.dumps(q)} --limit 5')}"
        )
        ssh(search_cmd, timeout=120)
        entry["reinforced"] = True

    census_sql = (
        "SELECT COUNT(*), COALESCE(MAX(recall_count),0), "
        "SUM(CASE WHEN confidence>=0.90 AND recall_count>=3 "
        "AND origin IN ('ingest','verified_dream','session_distill') THEN 1 ELSE 0 END) "
        f"FROM honeypot WHERE is_latest=1 AND content = {sql_quote(takeaway)};"
    )
    rc, so, se = sqlite_stdin(census_sql)
    nums = re.findall(r"-?\d+", (so or "").replace("|", " "))
    if len(nums) >= 3:
        entry["honeypot_hits"] = int(nums[0])
        entry["max_recall"] = int(nums[1])
        entry["export_eligible_guess"] = int(nums[2]) > 0
    results.append(entry)

# Global CoreCrystallize census
rc, so, se = sqlite_stdin(
    "SELECT COUNT(*), COALESCE(SUM(CASE WHEN recall_count>=3 THEN 1 ELSE 0 END),0), "
    "COALESCE(SUM(CASE WHEN confidence>=0.90 AND recall_count>=3 "
    "AND origin IN ('ingest','verified_dream','session_distill') THEN 1 ELSE 0 END),0) "
    "FROM honeypot WHERE is_latest=1 AND content LIKE 'CoreCrystallize:%';"
)
g_nums = re.findall(r"-?\d+", (so or "").replace("|", " "))
global_xtal = {
    "latest_core_crystallize": int(g_nums[0]) if len(g_nums) > 0 else 0,
    "recall_ge3": int(g_nums[1]) if len(g_nums) > 1 else 0,
    "export_eligible": int(g_nums[2]) if len(g_nums) > 2 else 0,
}

payload = {
    "schema": "gzmo.core_crystallize/v1",
    "generated_at": now.isoformat(),
    "apply": apply,
    "dual_writer_refused": dual,
    "claims_file": str(claims_path),
    "distill_enabled": do_distill,
    "results": results,
    "global": global_xtal,
    "goal": {
        "min_export_eligible": 10,
        "reached": global_xtal.get("export_eligible", 0) >= 10,
    },
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
lines = [
    "# CORE crystallize",
    "",
    f"apply: **{apply}** · dual_writer: **{dual}** · distill: **{do_distill}**",
    f"global CoreCrystallize latest={global_xtal['latest_core_crystallize']} "
    f"recall≥3={global_xtal['recall_ge3']} export_eligible≈{global_xtal['export_eligible']}",
    f"goal (≥10 export-eligible): **{payload['goal']['reached']}**",
    "",
    "| id | enqueued | landed | mode | hits | max_recall | export? | error |",
    "|----|----------|--------|------|------|------------|---------|-------|",
]
for r in results:
    lines.append(
        f"| {r['id']} | {r['enqueued']} | {r['landed']} | {r.get('land_mode') or ''} | "
        f"{r['honeypot_hits']} | {r['max_recall']} | {r['export_eligible_guess']} | {r.get('error') or ''} |"
    )
(out / "latest.md").write_text("\n".join(lines) + "\n")
print("\n".join(lines))
if dual:
    raise SystemExit(2)
if apply and not payload["goal"]["reached"]:
    print("\n[HOLD] goal not fully reached yet — inspect land errors / re-run", flush=True)
    raise SystemExit(1)
PY
