#!/usr/bin/env bash
# Obolus Forge mutation spike — losers get sibling mutation proposals; winners pinned.
# Never overwrites live gzmo-next.toml / fused engine map.
#
#   bash scripts/obolus-forge-mutate.sh
#   FORGE_Z_FLOOR=0.85 bash scripts/obolus-forge-mutate.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
ARENA="$DATA/arena"
OUT="$ARENA/forge"
export DATA ARENA OUT
export FORGE_Z_FLOOR="${FORGE_Z_FLOOR:-0.85}"
export FORGE_EPS="${FORGE_EPS:-1e-6}"
export FORGE_MAD_K="${FORGE_MAD_K:-1.0}"
mkdir -p "$OUT" "$ARENA/history"

python3 - <<'PY'
import json, os, hashlib
from datetime import datetime, timezone
from pathlib import Path

arena = Path(os.environ["ARENA"])
out = Path(os.environ["OUT"])
hist = arena / "history"
z_floor = float(os.environ.get("FORGE_Z_FLOOR", "0.85"))
eps = float(os.environ.get("FORGE_EPS", "1e-6"))
mad_k = float(os.environ.get("FORGE_MAD_K", "1.0"))
now = datetime.now(timezone.utc).isoformat()


def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


def median(xs):
    v = sorted(xs)
    n = len(v)
    if n == 0:
        return 0.0
    if n % 2 == 1:
        return v[n // 2]
    return (v[n // 2 - 1] + v[n // 2]) / 2.0


def mad(xs):
    if not xs:
        return 0.0
    m = median(xs)
    return median([abs(x - m) for x in xs])


def accept_delta(samples, epsilon, k):
    mad_part = (k * mad(samples)) if len(samples) >= 2 else 0.0
    return max(max(epsilon, 0.0), mad_part)


runs = []
for p in sorted(hist.glob("arena-*.json")):
    r = load(p)
    if r:
        runs.append(r)
latest = load(arena / "latest.json")
if latest and not any(r.get("finished") == latest.get("finished") for r in runs):
    runs.append(latest)

if not runs:
    payload = {
        "schema": "gzmo.obolus.forge/v1",
        "generated_at": now,
        "ok": False,
        "detail": "no arena runs — bash scripts/arena-night.sh first",
        "mutations": [],
        "pinned": [],
    }
    (out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
    print(json.dumps(payload, indent=2))
    raise SystemExit(0)

# Rank by z (desc); pin champion + near-ties (MAD); mutate clear losers.
ranked = sorted(runs, key=lambda r: float(r.get("z") or 0.0), reverse=True)
champion = ranked[0]
zs = [float(r.get("z") or 0.0) for r in ranked]
floor = accept_delta(zs, eps, mad_k)
champ_z = float(champion.get("z") or 0.0)
pinned = []
mutations = []

for i, r in enumerate(ranked):
    eng = r.get("champion") or r.get("engine") or "unknown"
    z = float(r.get("z") or 0.0)
    q = float(r.get("quality") or 0.0)
    e = float(r.get("efficiency") or 0.0)
    joules = r.get("joules")
    entry = {
        "engine": eng,
        "z": z,
        "quality": q,
        "efficiency": e,
        "joules": joules,
        "finished": r.get("finished"),
        "rank": i + 1,
    }
    near_champion = (champ_z - z) <= floor
    if i == 0 or near_champion or z >= z_floor:
        pinned.append({
            **entry,
            "action": "pin",
            "near_champion": near_champion,
            "accept_delta": floor,
        })
        continue

    # Deterministic mutation recipe from metrics (no live LLM required for spike).
    reasons = []
    mut = {"prompt_delta": [], "size_hint": "unchanged", "temp_delta": 0.0}
    if q < 0.9:
        reasons.append("low_quality")
        mut["prompt_delta"].append("Tighten recall instructions; prefer honeypot citations.")
        mut["temp_delta"] = -0.1
    if e < 0.75:
        reasons.append("low_efficiency")
        mut["prompt_delta"].append("Prefer shorter answers; cap max_tokens for overnight jobs.")
        mut["size_hint"] = "smaller_or_quantized"
    if (champ_z - z) > floor:
        reasons.append("below_mad_floor")
        mut["prompt_delta"].append("Raise faithfulness check before claiming recall hits.")
    if z < z_floor:
        reasons.append("below_z_floor")
    if not reasons:
        reasons.append("rank_loser")
        mut["prompt_delta"].append("Re-run Arena with RAPL if estimate-only.")

    seed = f"{eng}:{z}:{r.get('finished')}"
    mid = hashlib.sha256(seed.encode()).hexdigest()[:12]
    mutations.append({
        **entry,
        "action": "mutate",
        "mutation_id": mid,
        "reasons": reasons,
        "mutation": mut,
        "accept_delta": floor,
        "note": "Sibling proposal only — human promote via assemble/fuse; never auto-overwrite live config.",
    })

# Always ensure champion is pinned even if below floor (best of breed).
champ_eng = champion.get("champion") or champion.get("engine")
if not any(p.get("engine") == champ_eng for p in pinned):
    pinned.insert(0, {
        "engine": champ_eng,
        "z": champ_z,
        "quality": float(champion.get("quality") or 0),
        "efficiency": float(champion.get("efficiency") or 0),
        "joules": champion.get("joules"),
        "finished": champion.get("finished"),
        "rank": 1,
        "action": "pin",
        "near_champion": True,
        "accept_delta": floor,
    })

# Sole / all-pinned but below absolute floor → still propose next-gen improvement.
if not mutations and champ_z < z_floor:
    eng = champ_eng
    z = champ_z
    seed = f"improve:{eng}:{z}:{champion.get('finished')}"
    mid = hashlib.sha256(seed.encode()).hexdigest()[:12]
    mutations.append({
        "engine": eng,
        "z": z,
        "quality": float(champion.get("quality") or 0),
        "efficiency": float(champion.get("efficiency") or 0),
        "joules": champion.get("joules"),
        "finished": champion.get("finished"),
        "rank": 1,
        "action": "mutate",
        "mutation_id": mid,
        "reasons": ["below_z_floor", "improve_champion"],
        "mutation": {
            "prompt_delta": [
                "Wire RAPL for true joules; keep recall faithfulness gate.",
                "Trim overnight max_tokens; prefer honeypot citations.",
            ],
            "size_hint": "unchanged_or_quantized",
            "temp_delta": -0.05,
        },
        "accept_delta": floor,
        "note": "Sibling proposal only — human promote via assemble/fuse; never auto-overwrite live config.",
    })

payload = {
    "schema": "gzmo.obolus.forge/v1",
    "generated_at": now,
    "ok": True,
    "z_floor": z_floor,
    "noise_epsilon": eps,
    "mad_k": mad_k,
    "accept_delta": floor,
    "arena_runs": len(runs),
    "champion": champion.get("champion") or champion.get("engine"),
    "champion_z": champ_z,
    "pinned": pinned,
    "mutations": mutations,
    "suggestion_toml": str(out / "mutation-suggestion.toml"),
    "note": "Forge: pin champion + MAD-near ties; mutate clear losers. No live config write; human pin required.",
}

(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

champ = payload["champion"]
lines = [
    "# Obolus Forge mutation suggestion (sibling — not live config)",
    f"# Generated {now}",
    f"# z_floor = {z_floor}  accept_delta(MAD) = {floor}",
    "",
    "[forge.champion]",
    f'engine_label = "{champ}"',
    f"z = {payload['champion_z']}",
    "pinned = true",
    "",
]
for m in mutations:
    mid = m["mutation_id"]
    lines += [
        f"[forge.mutations.{mid}]",
        f'engine_label = "{m["engine"]}"',
        f"z = {m['z']}",
        f"reasons = {json.dumps(m['reasons'])}",
        f"temp_delta = {m['mutation']['temp_delta']}",
        f'size_hint = "{m["mutation"]["size_hint"]}"',
        "prompt_delta = [",
    ]
    for p in m["mutation"]["prompt_delta"]:
        lines.append(f'  "{p}",')
    lines += ["]", ""]

(out / "mutation-suggestion.toml").write_text("\n".join(lines) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Obolus Forge mutations",
            "",
            f"Generated: {now}",
            f"Champion: **{champ}** (z={payload['champion_z']})",
            f"accept_delta (MAD): {floor}",
            f"Pinned: {len(pinned)} · Mutations proposed: {len(mutations)}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({
    "ok": payload["ok"],
    "champion": payload["champion"],
    "champion_z": payload["champion_z"],
    "accept_delta": floor,
    "pinned": len(pinned),
    "mutations": len(mutations),
    "out": str(out / "latest.json"),
}, indent=2))
PY
