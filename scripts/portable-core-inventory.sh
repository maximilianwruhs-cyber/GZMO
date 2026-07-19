#!/usr/bin/env bash
# Unpark Wave 4.4 demable: inventory product-A vs living-C modules (no rewrite).
#   bash scripts/portable-core-inventory.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/portable-core"
mkdir -p "$OUT"

export ROOT OUT
python3 - <<'PY'
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__import__("os").environ["ROOT"])
out = Path(__import__("os").environ["OUT"])
core = root / "gzmo-core" / "src"

productish = []
livingish = []
for p in sorted(core.rglob("*.rs")):
    rel = str(p.relative_to(root))
    text = p.read_text(encoding="utf-8", errors="ignore")[:4000].lower()
    name = p.name.lower()
    if any(k in name or k in rel for k in ("mcp", "vault", "honeypot", "session", "product")):
        productish.append(rel)
    if any(k in name or k in rel for k in ("daemon", "orchestr", "distill", "dream", "spark", "qdrant", "redis")):
        livingish.append(rel)

# Dedup keep first 40 each for readability
payload = {
    "schema": "gzmo.unpark.portable_core/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "wave": "4.4",
    "ok": True,
    "advice": "portable_core_inventory_ok — inventory only; hold_rewrite default",
    "product_candidate_files": productish[:40],
    "living_candidate_files": livingish[:40],
    "counts": {"product_candidates": len(productish), "living_candidates": len(livingish)},
    "rewrite": "hold_rewrite",
    "rfc": "docs/PORTABLE_GZMO_CORE_RFC.md",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n")
(out / "inventory.md").write_text(
    "# Portable core inventory\n\n"
    f"- Product-ish files: {len(productish)}\n"
    f"- Living-ish files: {len(livingish)}\n"
    "- Default: **hold_rewrite** — see PORTABLE_GZMO_CORE_RFC.md\n",
    encoding="utf-8",
)
print(json.dumps({"ok": True, "advice": payload["advice"], "counts": payload["counts"]}, indent=2))
PY
echo "[OK] portable core inventory → $OUT"
