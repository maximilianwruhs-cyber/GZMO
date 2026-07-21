#!/usr/bin/env bash
# Organ trace — which metabolism / soft-fail jobs left evidence.
#
# Lab (default): data-next/scheduler-runs
# Living:        SSH CT101 /opt/gzmo/data/scheduler-runs + satellite organs
#
#   bash scripts/organ-trace.sh
#   bash scripts/organ-trace.sh --living
#
# Never starts an overnight writer. Dual-writer note if workstation gzmo-serve active.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="${DATA}/organ-trace"
HOST="${CT101_SSH_HOST:-ct101}"
REMOTE_DATA="${KEEP_QUALITY_DATA_DIR:-/opt/gzmo/data}"
REMOTE_RUNS="${REMOTE_DATA}/scheduler-runs"
LIVING=0
for a in "$@"; do
  case "$a" in
    --living) LIVING=1 ;;
    -h|--help)
      echo "Usage: $0 [--living]"
      exit 0
      ;;
  esac
done
mkdir -p "$OUT"

DUAL=0
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  DUAL=1
fi

LOCAL_RUNS="$DATA/scheduler-runs"
MIRROR="$OUT/living-scheduler-runs"
mkdir -p "$MIRROR"

if [[ "$LIVING" == "1" ]]; then
  # Mirror living scheduler-runs if present (may be empty on daemon-only hosts)
  ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
    "test -d '$REMOTE_RUNS' && ls '$REMOTE_RUNS'/latest-*.json 2>/dev/null | head -40" \
    >"$OUT/remote-list.txt" 2>/dev/null || true
  if [[ -s "$OUT/remote-list.txt" ]]; then
    while read -r remote_path; do
      [[ -z "$remote_path" ]] && continue
      base="$(basename "$remote_path")"
      scp -o ConnectTimeout=12 -o BatchMode=yes \
        "${HOST}:${remote_path}" "$MIRROR/$base" >/dev/null 2>&1 || true
    done <"$OUT/remote-list.txt"
  fi
  # Also pull stamped job runs if any
  ssh -o ConnectTimeout=12 -o BatchMode=yes "$HOST" \
    "ls '$REMOTE_RUNS'/*-*.json 2>/dev/null | head -80" \
    >"$OUT/remote-stamped.txt" 2>/dev/null || true
  if [[ -s "$OUT/remote-stamped.txt" ]]; then
    while read -r remote_path; do
      [[ -z "$remote_path" ]] && continue
      base="$(basename "$remote_path")"
      scp -o ConnectTimeout=12 -o BatchMode=yes \
        "${HOST}:${remote_path}" "$MIRROR/$base" >/dev/null 2>&1 || true
    done <"$OUT/remote-stamped.txt"
  fi
  RUNS_DIR="$MIRROR"
  SOURCE="living"
else
  RUNS_DIR="$LOCAL_RUNS"
  SOURCE="lab"
fi

export OUT RUNS_DIR SOURCE LIVING DUAL HOST REMOTE_DATA ROOT
python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
runs = Path(os.environ["RUNS_DIR"])
source = os.environ["SOURCE"]
living = os.environ.get("LIVING") == "1"
dual = os.environ.get("DUAL") == "1"
host = os.environ["HOST"]
remote_data = os.environ["REMOTE_DATA"]
now = datetime.now(timezone.utc).isoformat()

ORGANS = {
    "distill": "session-distill",
    "promote": "honeypot-gate / vault-promote",
    "embed": "embeddings + qdrant-sync",
    "dream": "dream-append / REM",
    "spark": "spark-link",
    "wiki": "wiki-okforge-push",
    "dream-compact": "dreams_md compact",
    "watchdog": "missed-run watchdog",
    "learning-loop": "dream+spark night ring",
}

CORE = ["distill", "promote", "embed", "dream", "spark"]

fired = []
if runs.is_dir():
    for path in sorted(runs.glob("latest-*.json")):
        job = path.name.removeprefix("latest-").removesuffix(".json")
        try:
            r = json.loads(path.read_text(encoding="utf-8"))
        except Exception:
            continue
        if job == "watchdog":
            fired.append({
                "job": job,
                "organ": ORGANS.get(job, job),
                "ok": r.get("ok", not r.get("stale", True)),
                "finished": r.get("checked_at") or r.get("finished"),
                "detail": r.get("detail") or r.get("advice"),
                "source": "scheduler-runs",
            })
            continue
        if job == "learning-loop":
            fired.append({
                "job": job,
                "organ": ORGANS.get(job, job),
                "ok": bool(r.get("complete") or r.get("ok")),
                "finished": r.get("updated_at") or r.get("finished"),
                "detail": f"night_id={r.get('night_id')}",
                "source": "scheduler-runs",
            })
            continue
        fired.append({
            "job": job,
            "organ": ORGANS.get(job, job),
            "ok": bool(r.get("ok")),
            "finished": r.get("finished") or r.get("updated_at"),
            "runner": r.get("runner") or r.get("script"),
            "error": r.get("error"),
            "source": "scheduler-runs",
        })

# Living satellite evidence when scheduler-runs thin/empty
satellites = []
if living:
    probes = [
        ("spark", f"{remote_data}/spark/last-spark-report.json", "spark refractory / last report"),
        ("night-lymph", f"{remote_data}/night-lymph/latest.json", "night lymph"),
        ("immune", f"{remote_data}/immune/latest.json", "immune plan"),
        ("ripen", f"{remote_data}/ripen/latest.json", "honeypot ripen"),
        ("dreams-md", "/opt/gzmo/DREAMS.md", "DREAMS.md mtime (dream organ soft)"),
    ]
    for job, remote_path, organ in probes:
        p = subprocess.run(
            ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host,
             f"stat -c '%Y %s' {remote_path} 2>/dev/null || echo MISSING"],
            capture_output=True, text=True, timeout=20,
        )
        raw = (p.stdout or "").strip()
        if not raw or raw.startswith("MISSING"):
            satellites.append({
                "job": job,
                "organ": organ,
                "ok": False,
                "finished": None,
                "detail": "missing",
                "source": "living-satellite",
            })
            continue
        parts = raw.split()
        try:
            mtime = int(parts[0])
            size = int(parts[1]) if len(parts) > 1 else 0
        except Exception:
            mtime, size = 0, 0
        finished = datetime.fromtimestamp(mtime, tz=timezone.utc).isoformat() if mtime else None
        satellites.append({
            "job": job,
            "organ": organ,
            "ok": size > 0,
            "finished": finished,
            "detail": f"bytes={size}",
            "source": "living-satellite",
        })

order = ["distill", "promote", "embed", "dream", "spark", "wiki", "dream-compact", "watchdog", "learning-loop"]
fired.sort(key=lambda x: order.index(x["job"]) if x["job"] in order else 99)

core_present = {f["job"] for f in fired if f["job"] in CORE}
missed_core = [j for j in CORE if j not in core_present]

advice = "organ_trace_ok"
ok = True
if dual:
    advice = "organ_trace_hold — dual_writer (gzmo-serve active); do not start second overnight brain"
    ok = True  # observation still valid; do not flip RED for tracing
if living and not fired and any(s.get("ok") for s in satellites):
    advice = (
        "organ_trace_living_satellites_only — scheduler-runs empty/missing; "
        "daemon organs present; start gzmo serve for typed metabolism records"
    )
elif living and missed_core:
    advice = (
        f"organ_trace_living_soft_miss — missing core latest-*: {','.join(missed_core)}"
    )
elif living and fired:
    advice = f"organ_trace_living_ok — scheduler jobs={len(fired)}"
elif not living and fired:
    advice = f"organ_trace_lab_ok — jobs={len(fired)}"
elif not fired:
    advice = "organ_trace_empty — no scheduler-runs latest-* yet"

payload = {
    "schema": "gzmo.organ-trace/v2",
    "generated_at": now,
    "ok": ok,
    "advice": advice,
    "source": source,
    "living": living,
    "dual_writer": dual,
    "runs_dir": str(runs),
    "organs_fired": len(fired),
    "ok_count": sum(1 for f in fired if f.get("ok")),
    "jobs": fired,
    "satellites": satellites,
    "missed_core": missed_core,
    "doc": "docs/STACK_OPPORTUNITY_MAP.md",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
if living:
    (out / "living-latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

lines = [
    f"# Organ trace — {now}",
    "",
    f"Source: **{source}** · dual_writer={dual}",
    f"Advice: {advice}",
    "",
    f"Scheduler jobs: **{len(fired)}** (ok={payload['ok_count']})",
    "",
    "| Job | Organ | Result | Last | Source |",
    "|-----|-------|--------|------|--------|",
]
for f in fired:
    mark = "OK" if f.get("ok") else "FAIL/STALE"
    lines.append(
        f"| {f['job']} | {f['organ']} | {mark} | {f.get('finished') or '—'} | {f.get('source')} |"
    )
if satellites:
    lines += ["", "## Living satellites", "", "| Job | Organ | Result | Last |", "|-----|-------|--------|------|"]
    for s in satellites:
        mark = "OK" if s.get("ok") else "MISSING"
        lines.append(
            f"| {s['job']} | {s['organ']} | {mark} | {s.get('finished') or '—'} |"
        )
if missed_core:
    lines += ["", f"Soft miss (core latest-*): {', '.join(missed_core)}", ""]
lines.append("")
(out / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({
    "ok": ok,
    "advice": advice,
    "source": source,
    "organs_fired": len(fired),
    "ok_count": payload["ok_count"],
    "missed_core": missed_core,
    "satellites_ok": sum(1 for s in satellites if s.get("ok")),
    "dual_writer": dual,
    "path": str(out / "latest.json"),
}, indent=2))
raise SystemExit(0 if ok else 1)
PY
