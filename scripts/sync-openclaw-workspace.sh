#!/usr/bin/env bash
# Sync GZMO ecosystem contract into ~/.openclaw/workspace (operator surface).
# Regenerates generated files; patches <!-- GZMO:ECOSYSTEM:BEGIN/END --> blocks.
# Never starts gzmo-serve. Never touches memory/*.md or MEMORY.md contents.
#
#   bash scripts/sync-openclaw-workspace.sh
#   OPENCLAW_WORKSPACE=~/.openclaw/workspace bash scripts/sync-openclaw-workspace.sh
#   OPENCLAW_PATCH_MODELS=1 bash scripts/sync-openclaw-workspace.sh
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC="$ROOT/config/openclaw-workspace"
WS="${OPENCLAW_WORKSPACE:-$HOME/.openclaw/workspace}"
BEGIN="<!-- GZMO:ECOSYSTEM:BEGIN -->"
END="<!-- GZMO:ECOSYSTEM:END -->"
[[ -d "$SRC" ]] || { echo "REFUSE: missing $SRC" >&2; exit 1; }
mkdir -p "$WS/bin" "$WS/memory"
chmod +x \
  "$ROOT/scripts/openclaw-takeaway.sh" \
  "$ROOT/scripts/pi-gzmo-mcp-serve.sh" \
  "$WS/bin/list-gzmo-crons.sh" 2>/dev/null || true

install -m 755 "$ROOT/scripts/openclaw-takeaway.sh" "$WS/bin/openclaw-takeaway.sh"
ln -sfn "$ROOT/scripts/openclaw-takeaway.sh" "$WS/bin/openclaw-takeaway-repo.sh"
if [[ ! -x "$WS/bin/list-gzmo-crons.sh" ]]; then
  cat >"$WS/bin/list-gzmo-crons.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
echo "=== OpenClaw cron (gateway) ==="
openclaw cron list 2>&1 || true
echo
echo "=== OpenClaw cron status ==="
openclaw cron status 2>&1 || true
echo
echo "=== Workstation systemd user timers (gzmo/okforge/vllm/openclaw) ==="
systemctl --user list-timers --all 2>&1 | rg -i 'gzmo-|okforge|vllm|openclaw' || true
echo
echo "=== Playbook ==="
echo "CRON_JOBS.md · GZMO_ECOSYSTEM_CRON.md · ECOSYSTEM.md · MODELS.md"
EOF
  chmod +x "$WS/bin/list-gzmo-crons.sh"
fi

# Full generated copies (always overwrite)
cp "$SRC/ECOSYSTEM.md" "$WS/ECOSYSTEM.md"
cp "$SRC/LIVING_ATTACH.md" "$WS/LIVING_ATTACH.md"
cp "$SRC/GZMO_ECOSYSTEM_CRON.md" "$WS/GZMO_ECOSYSTEM_CRON.md"
cp "$SRC/MODELS.md" "$WS/MODELS.md"

if [[ -d "$SRC/skills" ]]; then
  mkdir -p "$WS/skills"
  cp -a "$SRC/skills/." "$WS/skills/"
  find "$WS/skills" -type f -name 'run.sh' -exec chmod +x {} +
fi

mkdir -p "$HOME/.local/bin"
ln -sfn "$ROOT/scripts/openclaw-choose-character.sh" "$HOME/.local/bin/gzmo-character"
ln -sfn "$WS/skills/character/run.sh" "$WS/bin/character" 2>/dev/null || true
[[ -f "$WS/bin/character" ]] && chmod +x "$WS/bin/character"

for f in SOUL.md IDENTITY.md USER.md TOOLS.md AGENTS.md; do
  if [[ ! -f "$WS/$f" ]]; then
    if [[ -f "$SRC/$f" ]]; then cp "$SRC/$f" "$WS/$f"; else touch "$WS/$f"; fi
  fi
done

if [[ ! -f "$WS/TOOLS.local.md" ]]; then
  cat >"$WS/TOOLS.local.md" <<'EOF'
# TOOLS.local.md — personal / non-ecosystem notes

Synced TOOLS.md ecosystem block is overwritten by `sync-openclaw-workspace.sh`.
Put cameras, TTS, nicknames, and one-off host aliases here.

### Models

Canonical sheet is synced **MODELS.md** (vLLM dual-Blackwell Prime).
Do not reintroduce llama.cpp primary here.
EOF
fi

python3 - "$WS" "$SRC" "$BEGIN" "$END" <<'PY'
import re, sys
from pathlib import Path
ws, src, begin, end = Path(sys.argv[1]), Path(sys.argv[2]), sys.argv[3], sys.argv[4]

def load_block(name: str) -> str:
    raw = (src / name).read_text(encoding="utf-8")
    raw = raw.replace(begin, "").replace(end, "")
    return raw.strip() + "\n"

def patch(path: Path, block: str, insert_after: str | None = None) -> None:
    text = path.read_text(encoding="utf-8") if path.is_file() else ""
    chunk = f"{begin}\n{block.rstrip()}\n{end}\n"
    text = re.sub(re.escape(begin) + r".*?" + re.escape(end) + r"\s*", "", text, flags=re.S)
    text = text.replace(begin, "").replace(end, "")
    if insert_after and insert_after in text:
        text = text.replace(insert_after, insert_after + "\n\n" + chunk, 1)
    else:
        text = text.rstrip() + "\n\n" + chunk
    path.write_text(text if text.endswith("\n") else text + "\n", encoding="utf-8")
    print(f"patched {path.name}")

patch(ws / "AGENTS.md", load_block("AGENTS.ecosystem.md"), insert_after="## Session Startup")
patch(ws / "TOOLS.md", load_block("TOOLS.ecosystem.md"))
patch(ws / "SOUL.md", load_block("SOUL.ecosystem.md"))
patch(ws / "IDENTITY.md", load_block("IDENTITY.ecosystem.md"))
patch(ws / "USER.md", load_block("USER.ecosystem.md"))
print("marker patch complete")
PY

python3 - <<'PY'
import json, os, sys
from pathlib import Path
cfg_path = Path.home() / ".openclaw/openclaw.json"
want = "vllm/Qwen/Qwen3.6-35B-A3B"
patch = os.environ.get("OPENCLAW_PATCH_MODELS", "0") == "1"
if not cfg_path.is_file():
    print("[models] no openclaw.json — skip")
    raise SystemExit(0)
cfg = json.loads(cfg_path.read_text(encoding="utf-8"))
primary = cfg.get("agents", {}).get("defaults", {}).get("model", {}).get("primary")
print(f"[models] openclaw primary={primary!r}")
if primary != want:
    print(f"[models] WARN expected primary={want!r}", file=sys.stderr)
    if patch:
        d = cfg.setdefault("agents", {}).setdefault("defaults", {})
        prev_fb = (d.get("model") or {}).get("fallbacks") or ["openrouter/deepseek/deepseek-v4-flash"]
        d["model"] = {"primary": want, "fallbacks": prev_fb}
        mm = d.setdefault("models", {})
        mm.pop("llamacpp/qwen3.8-27b-abliterated", None)
        mm[want] = {"alias": "local"}
        providers = cfg.setdefault("models", {}).setdefault("providers", {})
        vllm = {
            "baseUrl": "http://127.0.0.1:8000/v1",
            "api": "openai-completions",
            "apiKey": "vllm-local",
            "models": [{
                "id": "Qwen/Qwen3.6-35B-A3B",
                "name": "Qwen3.6-35B-A3B-AWQ",
                "reasoning": True,
                "input": ["text"],
                "cost": {"input": 0, "output": 0, "cacheRead": 0, "cacheWrite": 0},
                "contextWindow": 32768,
                "maxTokens": 4096,
            }],
        }
        providers["vllm"] = vllm
        providers["llamacpp"] = dict(vllm)
        bak = cfg_path.with_suffix(cfg_path.suffix + ".bak-sync-models")
        bak.write_text(json.dumps(json.loads(cfg_path.read_text()), indent=2) + "\n", encoding="utf-8")
        # re-read was pre-mutation already in cfg
        cfg_path.write_text(json.dumps(cfg, indent=2) + "\n", encoding="utf-8")
        print(f"[models] PATCHED primary → {want} (backup {bak.name})")
    else:
        print("[models] set OPENCLAW_PATCH_MODELS=1 to auto-fix", file=sys.stderr)
else:
    prov = (cfg.get("models", {}).get("providers", {}) or {}).get("vllm") or {}
    if prov.get("apiKey") and prov.get("apiKey") != "vllm-local":
        print(f"[models] WARN vllm apiKey={prov.get('apiKey')!r} expected vllm-local", file=sys.stderr)
PY

python3 - "$WS" <<'PY'
import json, subprocess, sys
from pathlib import Path
ws = Path(sys.argv[1])
jobs = []
err = ""
try:
    raw = subprocess.check_output(["openclaw", "cron", "list", "--json"], stderr=subprocess.DEVNULL, text=True, timeout=30)
    data = json.loads(raw)
    if isinstance(data, list):
        jobs = data
    elif isinstance(data, dict):
        jobs = data.get("jobs") or data.get("items") or []
except Exception as e:
    err = f"(openclaw cron list failed: {e})"
try:
    timers = subprocess.check_output(["systemctl", "--user", "list-timers", "--all"], text=True, stderr=subprocess.DEVNULL, timeout=30)
    keep = [ln for ln in timers.splitlines() if any(x in ln.lower() for x in ("gzmo", "okforge", "vllm", "openclaw"))]
    timers_s = "\n".join(keep) if keep else timers[:2000]
except Exception:
    timers_s = "(systemctl list-timers unavailable)"
lines = [
    "# CRON_JOBS.md — snapshot",
    "",
    "Generated by `sync-openclaw-workspace.sh`. Refresh by re-running sync.",
    "",
    "Also read `ECOSYSTEM.md` + `GZMO_ECOSYSTEM_CRON.md` + `MODELS.md`.",
    "",
    f"## OpenClaw gateway jobs ({len(jobs)})",
    "",
]
if err:
    lines += [err, ""]
for j in jobs:
    if not isinstance(j, dict):
        continue
    sched = j.get("schedule") or {}
    expr = sched.get("expr") or sched.get("kind")
    tz = sched.get("tz") or ""
    payload = j.get("payload") or {}
    lines += [
        f"### {j.get('name')}",
        f"- id: `{j.get('id')}`",
        f"- declaration: `{j.get('declarationKey')}`",
        f"- schedule: `{expr}` {tz}".rstrip(),
        f"- enabled: {j.get('enabled')}",
        f"- payload: {payload.get('kind')}",
        f"- delivery: {(j.get('delivery') or {}).get('mode')}",
        "",
    ]
lines += ["## Workstation systemd timers (gzmo/okforge/vllm/openclaw)", "", "```", timers_s.rstrip() or "(none)", "```", ""]
(ws / "CRON_JOBS.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
(ws / ".cron-jobs.json").unlink(missing_ok=True)
print(f"wrote CRON_JOBS.md ({len(jobs)} openclaw jobs)")
PY

if [[ "${HEARTBEAT_ENABLE:-0}" != "1" ]]; then
  cat >"$WS/HEARTBEAT.md" <<'EOF'
<!-- Heartbeat template; comments-only content prevents scheduled heartbeat API calls. -->
<!-- Ecosystem digests use OpenClaw cron + systemd timers — see ECOSYSTEM.md / CRON_JOBS.md -->
# Keep this file empty (or with only comments) to skip heartbeat API calls.
EOF
fi

cat >"$WS/README.md" <<EOF
# OpenClaw workspace (GZMO-aligned)

| File | Owner |
|------|-------|
| \`ECOSYSTEM.md\` / \`MODELS.md\` | **synced** — start here |
| \`LIVING_ATTACH.md\` / \`CRON_JOBS.md\` / \`GZMO_ECOSYSTEM_CRON.md\` | **synced** |
| \`AGENTS.md\` / \`TOOLS.md\` / \`SOUL.md\` / \`IDENTITY.md\` / \`USER.md\` | hybrid (markers synced) |
| \`TOOLS.local.md\` / \`memory/\` / \`MEMORY.md\` | local only |
| \`HEARTBEAT.md\` | comments-only by default |

Sync: \`bash $ROOT/scripts/sync-openclaw-workspace.sh\`  
Contract: \`$ROOT/docs/OPENCLAW_WORKSPACE_CONTRACT.md\`
EOF

echo "[OK] synced OpenClaw workspace → $WS"
ls -1 "$WS"/*.md | sed 's|.*/||' | sort
