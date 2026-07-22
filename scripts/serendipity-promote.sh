#!/usr/bin/env bash
# Brain Feed P0: serendipity / spark → takeaway candidates → living promote path.
# Dry-run by default (writes artifact only). Apply enqueues takeaway on living host
# (CT101) without --now and without starting a workstation overnight writer.
#
#   bash scripts/serendipity-promote.sh
#   SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh
#   SERENDIPITY_APPLY_LIMIT=3 SERENDIPITY_WEEKLY_CAP=3 SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh
#   bash scripts/serendipity-digest.sh && bash scripts/serendipity-promote.sh
# Filters horizon/local-intel theater (TurboQuant / 128k-on-32GB) out of living apply.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/serendipity"
HOST="${CT101_SSH_HOST:-ct101}"
GZMO_BIN="${CT101_GZMO_BIN:-/opt/gzmo/current/target/release/gzmo}"
APPLY="${SERENDIPITY_PROMOTE_APPLY:-0}"
APPLY_LIMIT="${SERENDIPITY_APPLY_LIMIT:-3}"
WEEKLY_CAP="${SERENDIPITY_WEEKLY_CAP:-3}"
mkdir -p "$OUT"

# Prefer living spark report; fall back to lab data-next
SPARK_RAW=""
if ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
  "test -f /opt/gzmo/data/spark/last-spark-report.json" 2>/dev/null; then
  SPARK_RAW="$(ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
    "cat /opt/gzmo/data/spark/last-spark-report.json" 2>/dev/null || true)"
fi
if [[ -z "$SPARK_RAW" && -f "$DATA/spark/last-spark-report.json" ]]; then
  SPARK_RAW="$(cat "$DATA/spark/last-spark-report.json")"
fi

export OUT APPLY HOST GZMO_BIN SPARK_RAW ROOT DATA APPLY_LIMIT WEEKLY_CAP
python3 - <<'PY'
import json, os, re, subprocess, uuid
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
apply = os.environ.get("APPLY", "0") == "1"
host = os.environ["HOST"]
gzmo_bin = os.environ["GZMO_BIN"]
raw = os.environ.get("SPARK_RAW") or ""
apply_limit = max(1, int(os.environ.get("APPLY_LIMIT") or 3))
weekly_cap = max(1, int(os.environ.get("WEEKLY_CAP") or 3))
now = datetime.now(timezone.utc)
stamp = now.strftime("%Y%m%dT%H%M%SZ")
iso_week = now.strftime("%G-W%V")

# Horizon / local-intel theater — never pump into living takeaways (ADR-0004 / opportunity horizon)
HORIZON_RE = re.compile(
    r"TurboQuant|llama-cpp-turboquant|256\s*K\s*context|32\s*GB\s*VRAM|"
    r"local.?intel|128k|256k",
    re.I,
)

spark = {}
if raw.strip():
    try:
        spark = json.loads(raw)
    except Exception as e:
        spark = {"parse_error": str(e)}

candidates = []
# Living report schema (gzmo.spark.report/v1): anchor_preview + anchor_id
preview = (spark.get("anchor_preview") or "").strip()
if preview:
    candidates.append({
        "source": "anchor_preview",
        "fact_id": spark.get("anchor_id"),
        "text": preview[:500],
        "promoted": spark.get("promoted"),
    })

# Prefer explicit promoted / hypothesis text (lab / richer reports)
for key in ("hypothesis", "link", "section", "summary"):
    v = spark.get(key)
    if isinstance(v, str) and len(v.strip()) > 20:
        candidates.append({"source": key, "text": v.strip()[:500]})

sel = spark.get("selection") or {}
anchor = sel.get("anchor") or {}
if isinstance(anchor, dict):
    content = (anchor.get("content") or "").strip()
    if content:
        candidates.append({
            "source": "anchor",
            "fact_id": anchor.get("id"),
            "text": content[:500],
        })

recent = sel.get("recent") or sel.get("candidates") or []
if isinstance(recent, list):
    for r in recent[:5]:
        if isinstance(r, dict):
            c = (r.get("content") or "").strip()
            if c:
                candidates.append({
                    "source": "nearby",
                    "fact_id": r.get("id"),
                    "text": c[:500],
                })

# Refractory field entries as soft nearby candidates
try:
    import subprocess as _sp
    ref_raw = _sp.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host,
         "cat /opt/gzmo/data/spark/refractory.json"],
        capture_output=True, text=True, timeout=15,
    )
    if ref_raw.returncode == 0 and ref_raw.stdout.strip():
        field = json.loads(ref_raw.stdout)
        for ent in (field.get("entries") or [])[:4]:
            if not isinstance(ent, dict):
                continue
            prev = (ent.get("preview") or "").strip()
            if prev:
                candidates.append({
                    "source": "refractory",
                    "fact_id": ent.get("id"),
                    "text": prev[:500],
                })
except Exception:
    pass

# Dedupe by normalized text
seen = set()
uniq = []
for c in candidates:
    norm = re.sub(r"\s+", " ", c["text"].lower())[:160]
    if norm in seen:
        continue
    seen.add(norm)
    uniq.append(c)
candidates = uniq[:6]

# USP filter: drop horizon/local-intel theater from the apply set
filtered_out = []
kept = []
for c in candidates:
    if HORIZON_RE.search(c.get("text") or ""):
        filtered_out.append({"reason": "horizon_local_intel", "text": c["text"][:120]})
    else:
        kept.append(c)
candidates = kept

takeaways = []
for c in candidates[:apply_limit]:
    # Frame as serendipity link takeaway (operator-reviewable)
    prefix = "SerendipityLink"
    if c.get("fact_id"):
        body = f"{prefix}: {c['text'][:280]} (from {c['source']} id={c['fact_id']})"
    else:
        body = f"{prefix}: {c['text'][:320]} (from {c['source']})"
    takeaways.append(body)

# Weekly apply cap (ISO week) — O10 honesty
week_applies = 0
week_log = out / "weekly-apply-log.jsonl"
if week_log.is_file():
    for line in week_log.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except Exception:
            continue
        if row.get("iso_week") == iso_week:
            week_applies += int(row.get("applied_count") or 0)

dual_writer = False
try:
    r = subprocess.run(
        ["systemctl", "--user", "is-active", "gzmo-serve.service"],
        capture_output=True, text=True, timeout=5,
    )
    if (r.stdout or "").strip() == "active":
        dual_writer = True
except Exception:
    pass

applied = []
apply_error = None
if apply and week_applies >= weekly_cap:
    apply_error = f"weekly_cap_hit — already {week_applies}/{weekly_cap} applies in {iso_week}"
elif apply and takeaways and not dual_writer:
    sid = f"serendipity-promote-{uuid.uuid4().hex[:8]}"
    remote_sess = f"/opt/gzmo/data/sessions/{sid}.json"
    now_iso = now.strftime("%Y-%m-%dT%H:%M:%SZ")
    sess = {
        "id": sid,
        "name": "serendipity_promote",
        "created_at": now_iso,
        "last_active_at": now_iso,
        "messages": [
            {"role": "user", "content": "Serendipity promote-back session.", "is_meta": False},
            {"role": "assistant", "content": "Recording verified spark links as takeaways.", "is_meta": False},
        ],
    }
    # Seed session
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, f"cat > {remote_sess}"],
        input=json.dumps(sess), text=True, capture_output=True,
    )
    if p.returncode != 0:
        apply_error = f"seed_session:{p.stderr[:200]}"
    else:
        # Cap ≤ APPLY_LIMIT takeaways in one session close
        combined = " | ".join(takeaways[:apply_limit])
        cmd = (
            f"bash -lc 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml "
            f"{gzmo_bin} session close {sid} --takeaway {json.dumps(combined)}'"
        )
        p2 = subprocess.run(
            ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
            capture_output=True, text=True,
        )
        if p2.returncode == 0:
            applied.append({
                "session_id": sid,
                "takeaway": combined,
                "distill": "enqueue_only",
                "takeaway_n": min(len(takeaways), apply_limit),
            })
            with week_log.open("a", encoding="utf-8") as f:
                f.write(json.dumps({
                    "generated_at": now.isoformat(),
                    "iso_week": iso_week,
                    "applied_count": 1,
                    "takeaway_n": min(len(takeaways), apply_limit),
                    "session_id": sid,
                }, separators=(",", ":")) + "\n")
            week_applies += 1
        else:
            apply_error = f"session_close:{(p2.stderr or p2.stdout)[:300]}"
elif apply and dual_writer:
    apply_error = "refused_dual_writer — stop gzmo-serve before applying to living host"
elif apply and not takeaways:
    apply_error = "no_candidates_after_usp_filter" if filtered_out else "no_candidates"

advice = "serendipity_promote_dry_run_ok"
if dual_writer:
    advice = "serendipity_promote_hold — dual_writer active"
elif not takeaways and not apply:
    advice = (
        "serendipity_promote_hold — 0 USP candidates after horizon filter"
        if filtered_out
        else "serendipity_promote_hold — no spark candidates (run spark / digest first)"
    )
elif apply and applied:
    advice = (
        f"serendipity_promote_applied — takeaway enqueued on living host "
        f"(week {iso_week} applies={week_applies}/{weekly_cap})"
    )
elif apply and apply_error:
    advice = f"serendipity_promote_apply_failed — {apply_error}"

ok = (not dual_writer) and (bool(takeaways) or not apply) and (apply_error is None if apply else True)
# dry-run with zero candidates is HOLD-ok for check (ok=True, advice hold)
if not takeaways and not apply:
    ok = True

payload = {
    "schema": "gzmo.brain_feed.serendipity_promote/v1",
    "generated_at": now.isoformat(),
    "ok": ok,
    "dry_run": not apply,
    "advice": advice,
    "dual_writer": dual_writer,
    "candidate_count": len(candidates),
    "candidates": candidates,
    "filtered_out": filtered_out,
    "apply_limit": apply_limit,
    "weekly_cap": weekly_cap,
    "iso_week": iso_week,
    "week_applies": week_applies,
    "takeaways": takeaways,
    "applied": applied,
    "apply_error": apply_error,
    "spark_promoted": spark.get("promoted"),
    "spark_date": spark.get("date"),
    "next": [
        "bash scripts/serendipity-digest.sh",
        "Review data-next/serendipity/promote-latest.json takeaways (≤3; horizon filtered)",
        "SERENDIPITY_PROMOTE_APPLY=1 bash scripts/serendipity-promote.sh  # human-gated",
        "bash scripts/serendipity-weekly-check.sh",
    ],
}
path = out / f"promote-{stamp}.json"
path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "promote-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
print(json.dumps({
    "ok": ok,
    "advice": advice,
    "dry_run": not apply,
    "candidates": len(candidates),
    "filtered_out": len(filtered_out),
    "week_applies": week_applies,
    "path": str(path),
}, indent=2))
raise SystemExit(0 if ok else 1)
PY
