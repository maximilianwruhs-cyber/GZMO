#!/usr/bin/env bash
# Portable GZMO core inventory — compare living binary vs gzmo-core-clean surface.
# Advice defaults to hold-rewrite (ADR-0003); packaging only if living binary blocks CE.
#
#   bash scripts/portable-core-inventory.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/portable-core"
BIN="${GZMO_BIN:-${CARGO_TARGET_DIR:-$HOME/github-clone/temp-bench/target}/release/gzmo}"
mkdir -p "$OUT"
export ROOT CLONE DATA OUT BIN

python3 - <<'PY'
import json, os, subprocess
from datetime import datetime, timezone
from pathlib import Path

root = Path(os.environ["ROOT"])
clone = Path(os.environ["CLONE"])
out = Path(os.environ["OUT"])
bin_path = Path(os.environ["BIN"])
now = datetime.now(timezone.utc).isoformat()


def git_sha(path: Path):
    try:
        r = subprocess.run(
            ["git", "-C", str(path), "rev-parse", "--short", "HEAD"],
            capture_output=True, text=True, timeout=5, check=False,
        )
        return r.stdout.strip() or None
    except Exception:
        return None


def crate_present(name: str) -> bool:
    return (root / name / "Cargo.toml").is_file()


clean = clone / "gzmo-core-clean"
sovereign = clone / "sovereign-agent"
living_ok = bin_path.is_file()
clean_ok = (clean / "Cargo.toml").is_file()

surface = {
    "living": {
        "gzmo_bin": str(bin_path) if living_ok else None,
        "sha": git_sha(root),
        "crates": {n: crate_present(n) for n in ("gzmo-core", "gzmo-cli", "gzmo-chaos")},
    },
    "siblings": {
        "gzmo-core-clean": {"path": str(clean), "sha": git_sha(clean), "present": clean_ok},
        "sovereign-agent": {
            "path": str(sovereign),
            "sha": git_sha(sovereign),
            "present": (sovereign / "Cargo.toml").is_file() or (sovereign / "README.md").is_file(),
        },
    },
}

# Packaging is blocked only if the living binary is missing while CE wants a pin.
advice = (
    "hold_rewrite — living gzmo binary present; extract only if CE packaging is blocked"
    if living_ok
    else "consider_extract — living binary missing; evaluate gzmo-core-clean surface"
)

payload = {
    "schema": "gzmo.portable-core.inventory/v1",
    "generated_at": now,
    "ok": True,
    "adr": "ADR-0003-one-instance-metabolism",
    "surface": surface,
    "advice": advice,
    "production": {
        "big_bang_rewrite": False,
        "note": "Inventory only — no crate split in this spike.",
    },
    "note": "Research spike — useful cleanup later; do not distract nightburst metabolism.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
(out / "latest.md").write_text(
    "\n".join(
        [
            "# Portable GZMO core inventory",
            "",
            f"Advice: **{advice}**",
            f"Living binary: `{bin_path if living_ok else 'missing'}`",
            f"gzmo-core-clean: {surface['siblings']['gzmo-core-clean']['sha'] or 'missing'}",
            "",
            payload["note"],
            "",
        ]
    ),
    encoding="utf-8",
)
print(json.dumps({"ok": True, "advice": advice, "living_ok": living_ok, "clean_ok": clean_ok}, indent=2))
PY
