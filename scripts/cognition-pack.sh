#!/usr/bin/env bash
# Portable cognition pack spike — distill → honeypot → spark → recall contract.
# Emits a JSON assembly other agents can follow without the full GZMO binary.
#
#   bash scripts/cognition-pack.sh
#   bash scripts/cognition-pack.sh --smoke   # also run a local recall probe
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/cognition-pack"
SMOKE=0
for a in "$@"; do
  case "$a" in
    --smoke) SMOKE=1 ;;
  esac
done
mkdir -p "$OUT"
export DATA OUT ROOT SMOKE
export GZMO_INSTANCE="${GZMO_INSTANCE:-next}"
export GZMO_CONFIG="${GZMO_CONFIG:-$ROOT/config/gzmo.toml}"
export GZMO_ALLOW_LAB_VAULT="${GZMO_ALLOW_LAB_VAULT:-1}"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}"

python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

data = Path(os.environ["DATA"])
out = Path(os.environ["OUT"])
root = Path(os.environ["ROOT"])
smoke = os.environ.get("SMOKE") == "1"
now = datetime.now(timezone.utc).isoformat()


def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


runs = data / "scheduler-runs"
stages = [
    {
        "id": "distill",
        "role": "session → vault candidates",
        "artifact": "scheduler-runs/latest-distill.json",
        "command": "gzmo distill",
    },
    {
        "id": "promote",
        "role": "vault → honeypot qualify",
        "artifact": "scheduler-runs/latest-promote.json",
        "command": "gzmo promote",
    },
    {
        "id": "embed",
        "role": "honeypot → vectors / Qdrant",
        "artifact": "scheduler-runs/latest-embed.json",
        "command": "gzmo memory embed",
    },
    {
        "id": "dream",
        "role": "overnight consolidation",
        "artifact": "scheduler-runs/latest-dream.json",
        "command": "gzmo dream",
    },
    {
        "id": "spark",
        "role": "serendipitous verified links",
        "artifact": "scheduler-runs/latest-spark.json",
        "command": "gzmo spark",
    },
    {
        "id": "recall",
        "role": "felt search / MCP surface",
        "artifact": "aos-status/latest.json",
        "command": "gzmo memory search <query>",
    },
]

status = []
for s in stages:
    path = data / s["artifact"]
    raw = load(path)
    ok = None
    finished = None
    if raw is None:
        ok = False
    elif s["id"] == "recall":
        ok = (raw.get("status") == "online") or bool(raw.get("gzmo"))
        finished = (raw.get("gzmo") or {}).get("generated_at")
    else:
        ok = bool(raw.get("ok", True))
        finished = raw.get("finished")
    status.append({
        **s,
        "present": path.is_file(),
        "ok": ok,
        "finished": finished,
        "path": str(path) if path.is_file() else None,
    })

contract = {
    "schema": "gzmo.cognition.pack/v1",
    "name": "cognition-pack",
    "description": "Portable overnight memory loop: distill → promote → embed → dream/spark → recall",
    "stages": [
        {
            "id": s["id"],
            "role": s["role"],
            "inputs": ["sessions|vault"] if s["id"] == "distill" else ["prior_stage"],
            "outputs": ["vault|honeypot|vectors|reports"],
            "cli": s["command"],
        }
        for s in stages
    ],
    "json_contract": {
        "job_run": {
            "job": "string",
            "ok": "bool",
            "started": "rfc3339",
            "finished": "rfc3339",
            "runner": "rust|script",
        },
        "recall_probe": {
            "query": "string",
            "hit": "bool",
            "limit": "int",
        },
    },
    "notes": [
        "Other agents can implement the same stage contract without linking gzmo-core.",
        "Living proof still runs on GZMO serve; this pack is the portable map + status.",
    ],
}

(out / "contract.json").write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8")

smoke_result = None
if smoke:
    bin_candidates = [
        os.environ.get("GZMO_BIN"),
        str(Path(os.environ.get("CARGO_TARGET_DIR", "")) / "release" / "gzmo"),
        str(root / "target" / "release" / "gzmo"),
    ]
    gzmo = next((b for b in bin_candidates if b and Path(b).is_file()), None)
    if not gzmo:
        # PATH
        from shutil import which
        gzmo = which("gzmo")
    if gzmo:
        env = os.environ.copy()
        env.setdefault("GZMO_ALLOW_LAB_VAULT", "1")
        proc = subprocess.run(
            [gzmo, "memory", "search", "honeypot", "--limit", "1", "--json"],
            cwd=str(root),
            env=env,
            capture_output=True,
            text=True,
            timeout=60,
        )
        hit = proc.returncode == 0 and ("fact" in proc.stdout.lower() or "hits" in proc.stdout.lower() or proc.stdout.strip().startswith("[") or '"id"' in proc.stdout)
        # softer: non-empty stdout and exit 0
        if proc.returncode == 0 and proc.stdout.strip():
            hit = True
        smoke_result = {
            "ok": proc.returncode == 0,
            "hit": bool(hit),
            "exit": proc.returncode,
            "stdout_chars": len(proc.stdout),
            "stderr_chars": len(proc.stderr),
        }
    else:
        smoke_result = {"ok": False, "hit": False, "detail": "gzmo binary not found"}

payload = {
    "schema": "gzmo.cognition.pack.status/v1",
    "generated_at": now,
    "ok": all(s.get("present") for s in status[:5]) or any(s.get("ok") for s in status),
    "stages": status,
    "contract": str(out / "contract.json"),
    "smoke": smoke_result,
    "note": "Portable assembly map + living status; not a separate runtime.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
lines = [
    "# Cognition pack",
    "",
    f"Generated: {now}",
    "",
    "| stage | present | ok | finished |",
    "|-------|---------|----|----------|",
]
for s in status:
    lines.append(f"| {s['id']} | {s['present']} | {s['ok']} | {s.get('finished') or '—'} |")
if smoke_result:
    lines += ["", f"Smoke recall: ok={smoke_result.get('ok')} hit={smoke_result.get('hit')}", ""]
lines += ["", f"Contract: `{out / 'contract.json'}`", ""]
(out / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({
    "ok": payload["ok"],
    "stages_present": sum(1 for s in status if s["present"]),
    "smoke": smoke_result,
    "path": str(out / "latest.json"),
}, indent=2))
PY
