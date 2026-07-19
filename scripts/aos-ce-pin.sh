#!/usr/bin/env bash
# AOS Customer Edition golden-path pin — inventory living workstation versions.
# Sibling suggestion only; does not mutate AOS-Customer-Edition or live config.
#
#   bash scripts/aos-ce-pin.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLONE="${GZMO_CLONE_ROOT:-$(dirname "$ROOT")}"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/aos-ce"
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
data = Path(os.environ["DATA"])
bin_path = Path(os.environ["BIN"])
now = datetime.now(timezone.utc).isoformat()


def git_sha(path: Path) -> str | None:
    if not (path / ".git").exists() and not path.is_dir():
        return None
    try:
        r = subprocess.run(
            ["git", "-C", str(path), "rev-parse", "--short", "HEAD"],
            capture_output=True,
            text=True,
            timeout=5,
            check=False,
        )
        return r.stdout.strip() or None
    except Exception:
        return None


def load(p: Path):
    try:
        return json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        return None


repos = {
    "GZMO": git_sha(root),
    "AOS-Customer-Edition": git_sha(clone / "AOS-Customer-Edition"),
    "AOS": git_sha(clone / "AOS"),
    "Obolus": git_sha(clone / "Obolus"),
    "okforge": git_sha(clone / "okforge") or git_sha(clone / "OKForge"),
    "little-tools-lab": git_sha(clone / "little-tools-lab"),
    "escape-loop-bench": git_sha(clone / "escape-loop-bench"),
}

gzmo_ver = None
if bin_path.is_file():
    try:
        r = subprocess.run([str(bin_path), "--version"], capture_output=True, text=True, timeout=10)
        gzmo_ver = (r.stdout or r.stderr or "").strip().splitlines()[0] if (r.stdout or r.stderr) else None
    except Exception:
        gzmo_ver = None

arena = load(data / "arena" / "latest.json") or {}
gate = load(data / "concept-gate" / "latest.json") or {}
cognition = load(data / "cognition-pack" / "latest.json") or {}

payload = {
    "schema": "gzmo.aos-ce.pin/v1",
    "generated_at": now,
    "role": "golden_path_pin",
    "host_hint": "living workstation (nightburst lab)",
    "pins": {
        "repos": repos,
        "gzmo_binary": str(bin_path) if bin_path.is_file() else None,
        "gzmo_version": gzmo_ver,
        "instance": os.environ.get("GZMO_INSTANCE", "next"),
        "config_hint": "config/gzmo.toml → gzmo-next (operator local)",
    },
    "living_signals": {
        "arena_champion": arena.get("champion"),
        "arena_energy_source": arena.get("energy_source"),
        "concept_gate": gate.get("verdict"),
        "cognition_pack_ok": cognition.get("ok"),
    },
    "install_hint": "curl -fsSL …/AOS-Customer-Edition/…/bootstrap.sh | bash  # pin SHAs in CE after proof",
    "advice": (
        "ready_to_pin — copy repo SHAs into AOS-Customer-Edition deploy pins when demable"
        if repos.get("AOS-Customer-Edition") and repos.get("GZMO")
        else "hold — missing CE or GZMO checkout"
    ),
    "note": "Spike only — does not rewrite CE ansible/bootstrap. Human promotes pins.",
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
lines = [
    "# AOS Customer Edition golden-path pin",
    "",
    f"Generated: {now}",
    f"Advice: **{payload['advice']}**",
    "",
    "## Repo SHAs",
    "",
]
for k, v in repos.items():
    lines.append(f"- `{k}`: `{v or 'missing'}`")
lines += [
    "",
    f"gzmo: {gzmo_ver or 'n/a'}",
    f"Arena champion: {arena.get('champion')} · energy={arena.get('energy_source')}",
    "",
    payload["note"],
    "",
]
(out / "latest.md").write_text("\n".join(lines), encoding="utf-8")
print(json.dumps({"ok": True, "advice": payload["advice"], "repos": repos}, indent=2))
PY
