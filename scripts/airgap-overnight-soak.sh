#!/usr/bin/env bash
# Airgap overnight soak probe (G6) — prove local metabolism without public net.
# Does NOT pull ethernet itself (destructive). Checks local-first config + organ pulse.
#
#   bash scripts/airgap-overnight-soak.sh
#   AIRGAP_SOAK_STRICT=1 bash scripts/airgap-overnight-soak.sh  # fail if cloud engine required
#
# Artifact: data-next/airgap-overnight-soak/latest.{json,md}
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/airgap-overnight-soak"
HOST="${CT101_SSH_HOST:-ct101}"
STRICT="${AIRGAP_SOAK_STRICT:-0}"
mkdir -p "$OUT"

# Refuse dual-writer
SERVE="$(systemctl --user is-active gzmo-serve.service 2>/dev/null || true)"
SERVE="$(printf '%s\n' "$SERVE" | head -1)"
if [[ "$SERVE" == "active" ]]; then
  echo "REFUSE: gzmo-serve active on workstation" >&2
  exit 1
fi

export OUT HOST STRICT ROOT
python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
host = os.environ["HOST"]
strict = os.environ.get("STRICT", "0") == "1"
now = datetime.now(timezone.utc).isoformat()

def ssh(cmd: str, timeout=30) -> tuple[int, str]:
    p = subprocess.run(
        ["ssh", "-o", "ConnectTimeout=12", "-o", "BatchMode=yes", host, cmd],
        capture_output=True, text=True, timeout=timeout,
    )
    return p.returncode, (p.stdout or "") + (p.stderr or "")

checks = []
# Local embed URL should be LAN, not openrouter
rc, out_s = ssh("grep -E 'embeddings|engine_url|mode' /opt/gzmo/gzmo.toml | head -40")
checks.append({"id": "toml_probe", "ok": rc == 0, "detail": out_s[:500]})

rc2, organs = ssh(
    "ls /opt/gzmo/data/scheduler-runs/latest-*.json 2>/dev/null | wc -l"
)
n = 0
try:
    n = int("".join(c for c in organs if c.isdigit()) or "0")
except Exception:
    n = 0
checks.append({"id": "organ_receipts", "ok": n >= 5, "detail": f"latest_job_files={n}"})

# Prefer local mode language
cloudish = "openrouter.ai" in (checks[0].get("detail") or "").lower()
checks.append({
    "id": "cloud_engine_present",
    "ok": not (strict and cloudish),
    "detail": "openrouter in toml" if cloudish else "no openrouter snippet in probe",
    "hold_if_cloud": cloudish,
})

# Default route: living-readiness + brain-feed from workstation artifacts
for name in ("living-readiness", "brain-feed"):
    p = Path(os.environ["ROOT"]) / "data-next" / name / "latest.json"
    try:
        d = json.loads(p.read_text(encoding="utf-8"))
        ok = d.get("verdict") == "GREEN" or d.get("ok") is True
        checks.append({"id": name, "ok": ok, "detail": d.get("advice") or d.get("verdict")})
    except Exception as e:
        checks.append({"id": name, "ok": False, "detail": str(e)})

fail = [c for c in checks if not c["ok"]]
hold = any(c.get("hold_if_cloud") for c in checks)
verdict = "RED" if fail and not hold else ("HOLD" if hold or fail else "GREEN")
if hold and not fail:
    verdict = "HOLD"
advice = {
    "GREEN": "airgap_soak_probe_ok — local receipts + gates healthy; pull ethernet for full night proof",
    "HOLD": "airgap_soak_hold — cloud engine still in toml or partial gates; overnight ethernet-down still needed",
    "RED": "airgap_soak_fail — fix FAIL checks",
}[verdict]

payload = {
    "schema": "gzmo.airgap_overnight_soak/v1",
    "generated_at": now,
    "verdict": verdict,
    "ok": verdict == "GREEN",
    "advice": advice,
    "checks": checks,
    "operator_next": "Pull ethernet on living host overnight; morning re-run organ-trace + this script",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    f"# Airgap overnight soak — {verdict}\n\n{advice}\n", encoding="utf-8"
)
print(json.dumps({"verdict": verdict, "ok": verdict == "GREEN", "advice": advice}, indent=2))
raise SystemExit(0 if verdict != "RED" else 1)
PY
