#!/usr/bin/env bash
# Intelligence-per-Watt router spike — policy + Arena/AOS signals → route advice.
#
#   bash scripts/ipw-route.sh
#   bash scripts/ipw-route.sh --task chat
#   bash scripts/ipw-route.sh --task heavy_bench
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
POLICY="${GZMO_IPW_POLICY:-$ROOT/config/ipw-router.policy.toml}"
TASK="chat"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --task) TASK="${2:-chat}"; shift 2 ;;
    *) shift ;;
  esac
done
OUT="$DATA/ipw-router"
mkdir -p "$OUT"
export DATA POLICY OUT TASK ROOT

python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

data = Path(os.environ["DATA"])
policy_path = Path(os.environ["POLICY"])
out = Path(os.environ["OUT"])
task = os.environ.get("TASK", "chat")
now = datetime.now(timezone.utc).isoformat()


def load_json(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


def parse_toml_lite(text: str) -> dict:
    """Minimal TOML subset reader for this policy file (no external dep)."""
    root: dict = {}
    section = root
    for raw in text.splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            key = line[1:-1].strip()
            section = root.setdefault(key, {})
            continue
        if "=" not in line:
            continue
        k, v = line.split("=", 1)
        k, v = k.strip(), v.strip()
        if v.startswith('"') and v.endswith('"'):
            val: object = v[1:-1]
        elif v.lower() in ("true", "false"):
            val = v.lower() == "true"
        else:
            try:
                val = float(v) if "." in v else int(v)
            except ValueError:
                val = v
        section[k] = val
    return root


policy = parse_toml_lite(policy_path.read_text(encoding="utf-8")) if policy_path.is_file() else {}
thr = policy.get("thresholds") or {}
routes = policy.get("routes") or {}
failover = policy.get("failover") or {}

z_floor = float(thr.get("z_floor", 0.75))
watts_ceiling = float(thr.get("watts_ceiling", 120.0))
cloud_enabled = bool(failover.get("cloud_enabled", False))
if os.environ.get("GZMO_IPW_CLOUD", "").strip() in ("1", "true", "yes", "on"):
    cloud_enabled = True

arena = load_json(data / "arena" / "latest.json") or {}
aos = load_json(data / "aos-status" / "latest.json") or {}
z = arena.get("z")
if z is None:
    z = aos.get("z_score")
z = float(z) if z is not None else None
watts = arena.get("watts_avg") or aos.get("energy_avg")
watts = float(watts) if watts is not None else None

preferred = routes.get(task) or routes.get("chat") or "local_prime"
route = preferred
reasons = [f"task_class={task} → policy {preferred}"]
ceiling_breach = watts is not None and watts > watts_ceiling
weak_z = z is not None and z < z_floor

if task in ("overnight", "heavy_bench") or (weak_z and task not in ("recall",)):
    route = routes.get("overnight") or "local_metabolism"
    reasons.append(f"route overnight/metabolism (z={z}, floor={z_floor})")

if ceiling_breach:
    if cloud_enabled:
        route = routes.get("failover") or "cloud_openrouter"
        reasons.append(f"watts {watts} > ceiling {watts_ceiling} → cloud failover")
    else:
        route = "local_throttle"
        reasons.append(
            f"watts {watts} > ceiling {watts_ceiling} but cloud disabled — throttle local"
        )

payload = {
    "schema": "gzmo.ipw.route/v1",
    "generated_at": now,
    "ok": True,
    "task_class": task,
    "route": route,
    "preferred_policy": preferred,
    "signals": {
        "arena_z": z,
        "watts_avg": watts,
        "z_floor": z_floor,
        "watts_ceiling": watts_ceiling,
        "cloud_enabled": cloud_enabled,
        "ceiling_breach": ceiling_breach,
        "weak_z": weak_z,
    },
    "reasons": reasons,
    "policy_path": str(policy_path),
    "note": "Sibling advice only — serve/chat may read; metabolism jobs never blocked.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# IpW route advice",
            "",
            f"Task: `{task}` → **{route}**",
            f"z={z} · watts={watts} · cloud={cloud_enabled}",
            "",
            *[f"- {r}" for r in reasons],
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({k: payload[k] for k in ("task_class", "route", "signals", "reasons")}, indent=2))
print(f"Wrote {out / 'latest.json'}")
PY
