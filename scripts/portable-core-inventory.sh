#!/usr/bin/env bash
# Unpark Wave 4.4 demable: inventory product-A vs living-C modules (no rewrite).
# Classifies gzmo-core top-level mods from lib.rs + path seams (not filename keywords alone).
#
#   bash scripts/portable-core-inventory.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/portable-core"
mkdir -p "$OUT"

export ROOT OUT
python3 - <<'PY'
import json
import re
from datetime import datetime, timezone
from pathlib import Path

root = Path(__import__("os").environ["ROOT"])
out = Path(__import__("os").environ["OUT"])
lib = root / "gzmo-core" / "src" / "lib.rs"
cargo = root / "gzmo-core" / "Cargo.toml"
core = root / "gzmo-core" / "src"

# RFC seam buckets (module names from lib.rs)
PRODUCT = {
    "mcp",
    "memory",
    "session",
    "session_distill",
    "platform_memory",
    "platform_search",
    "mentor_client",
    "types",
    "config",
    "gateway",
    "health",
    "identity",
    "text_util",
    "tools",
    "wiki",
    "wiki_md",
}
LIVING = {
    "daemon",
    "orchestrator",
    "metabolism",
    "dreams",
    "dreams_md",
    "spark",
    "spark_schedule",
    "ingest",
    "ingest_prep",
    "watcher",
    "cron",
    "synapse",
    "synapse_reader",
    "kg_reconcile",
    "stealth",
    "dice_loop",
}
THEATER = {
    "skills",
    "pedagogy",
    "assembly",
    "observatory_board",
    "okforge_client",
    "wiki_okf",
    "workflow_skills",
    "ecosystem_status",
    "scanner",
    "context",
    "agent_loop",
    "agent_session",
    "subagent",
}

mods = []
if lib.is_file():
    for m in re.findall(r"(?m)^pub mod (\w+);", lib.read_text(encoding="utf-8")):
        mods.append(m)

features = []
if cargo.is_file():
    text = cargo.read_text(encoding="utf-8")
    in_feat = False
    for line in text.splitlines():
        if line.strip() == "[features]":
            in_feat = True
            continue
        if in_feat:
            if line.startswith("["):
                break
            if "=" in line and not line.strip().startswith("#"):
                features.append(line.split("=", 1)[0].strip())

seams = []
for name in mods:
    path = core / name
    kind = "shared"
    if name in PRODUCT:
        kind = "product"
    elif name in LIVING:
        kind = "living"
    elif name in THEATER:
        kind = "theater_or_ops"
    elif (core / f"{name}.rs").is_file() or path.is_dir():
        # Heuristic fallback for unclassified mods
        blob = name.lower()
        if any(k in blob for k in ("mcp", "vault", "session", "memory")):
            kind = "product"
        elif any(k in blob for k in ("daemon", "dream", "spark", "distill", "ingest")):
            kind = "living"
    exists = (core / f"{name}.rs").is_file() or path.is_dir()
    seams.append({"module": name, "seam": kind, "path_exists": exists})

by = {"product": [], "living": [], "theater_or_ops": [], "shared": []}
for s in seams:
    by.setdefault(s["seam"], []).append(s["module"])

payload = {
    "schema": "gzmo.unpark.portable_core/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "4.4",
    "ok": True,
    "advice": "portable_core_inventory_ok — module seams from lib.rs; hold_rewrite default",
    "source": "gzmo-core/src/lib.rs",
    "cargo_features": features,
    "features_sketched": ["product", "living"] if not features else features,
    "module_seams": seams,
    "counts": {k: len(v) for k, v in by.items()},
    "product_modules": by.get("product", []),
    "living_modules": by.get("living", []),
    "theater_or_ops_modules": by.get("theater_or_ops", []),
    "shared_modules": by.get("shared", []),
    "rewrite": "hold_rewrite",
    "rfc": "docs/PORTABLE_GZMO_CORE_RFC.md",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")

lines = [
    "# Portable core inventory",
    "",
    f"Generated: {payload['generated_at']}",
    f"Source: `{payload['source']}`",
    f"Cargo features declared: `{features or 'none — sketch product/living'}`",
    f"Rewrite: **{payload['rewrite']}**",
    "",
    "| Seam | Count | Modules |",
    "|------|------:|---------|",
]
for k in ("product", "living", "theater_or_ops", "shared"):
    mods_k = by.get(k, [])
    lines.append(f"| {k} | {len(mods_k)} | {', '.join(mods_k) or '—'} |")
lines += [
    "",
    "Default remains **hold_rewrite**. See [PORTABLE_GZMO_CORE_RFC.md](../../docs/PORTABLE_GZMO_CORE_RFC.md).",
    "",
]
(out / "inventory.md").write_text("\n".join(lines), encoding="utf-8")
print(
    json.dumps(
        {
            "ok": True,
            "advice": payload["advice"],
            "counts": payload["counts"],
            "rewrite": payload["rewrite"],
            "modules": len(seams),
        },
        indent=2,
    )
)
if not by.get("product") or not by.get("living"):
    raise SystemExit("inventory missing product or living seam modules")
if payload["rewrite"] != "hold_rewrite":
    raise SystemExit("rewrite must remain hold_rewrite")
PY
echo "[OK] portable core inventory → $OUT"
