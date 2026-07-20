#!/usr/bin/env bash
# Takeaway as side-effect — reminder + surface check (never starts a memory-gym chat).
# Emits data-next/takeaway-side-effect/latest.{json,md}
#
#   bash scripts/takeaway-side-effect-remind.sh
#   bash scripts/takeaway-side-effect-remind.sh --install-hook   # optional local git hook
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DATA="${GZMO_DATA_NEXT:-$ROOT/data-next}"
OUT="$DATA/takeaway-side-effect"
mkdir -p "$OUT"

INSTALL_HOOK=0
for arg in "$@"; do
  case "$arg" in
    --install-hook) INSTALL_HOOK=1 ;;
    -h|--help)
      echo "Usage: $0 [--install-hook]"
      exit 0
      ;;
  esac
done

pass=0
fail=0
hold=0
declare -a ROWS=()

row() {
  local status="$1" name="$2" detail="$3"
  ROWS+=("$status|$name|$detail")
  case "$status" in
    PASS) pass=$((pass + 1)) ;;
    FAIL) fail=$((fail + 1)) ;;
    HOLD) hold=$((hold + 1)) ;;
  esac
  echo "[$status] $name — $detail"
}

echo "=== Takeaway side-effect remind ==="

if [[ -f "$ROOT/.github/pull_request_template.md" ]] \
  && grep -q "Living takeaway" "$ROOT/.github/pull_request_template.md"; then
  row PASS "pr-template" ".github/pull_request_template.md reminds one living takeaway"
else
  row FAIL "pr-template" "missing living-takeaway PR checklist"
fi

if [[ -f "$ROOT/docs/BRAIN_FEED.md" ]] \
  && grep -qi "side-effect" "$ROOT/docs/BRAIN_FEED.md"; then
  row PASS "brain-feed-doctrine" "BRAIN_FEED.md documents takeaway as side-effect"
else
  row FAIL "brain-feed-doctrine" "BRAIN_FEED.md missing side-effect section"
fi

if [[ -f "$ROOT/docs/HERDR_METABOLISM.md" ]] \
  && grep -qi "side-effect\|piggyback\|memory gym" "$ROOT/docs/HERDR_METABOLISM.md"; then
  row PASS "herdr-doctrine" "HERDR_METABOLISM.md documents piggyback close-ritual"
else
  row FAIL "herdr-doctrine" "HERDR_METABOLISM.md missing piggyback doctrine"
fi

HOOK_SRC="$ROOT/scripts/hooks/post-commit-takeaway-remind"
if [[ -x "$HOOK_SRC" ]]; then
  row PASS "hook-script" "scripts/hooks/post-commit-takeaway-remind present"
else
  row FAIL "hook-script" "missing scripts/hooks/post-commit-takeaway-remind"
fi

HOOK_DST="$ROOT/.git/hooks/post-commit"
if [[ "$INSTALL_HOOK" == "1" ]]; then
  if [[ -d "$ROOT/.git/hooks" ]]; then
    # Non-destructive: only install if absent or already our stub
    if [[ ! -e "$HOOK_DST" ]] || grep -q "takeaway-side-effect" "$HOOK_DST" 2>/dev/null; then
      cp "$HOOK_SRC" "$HOOK_DST"
      chmod +x "$HOOK_DST"
      row PASS "hook-installed" "local .git/hooks/post-commit → takeaway remind (non-blocking)"
    else
      row HOLD "hook-installed" "existing post-commit present — leave untouched; run remind manually"
    fi
  else
    row HOLD "hook-installed" "not a git checkout with .git/hooks"
  fi
elif [[ -e "$HOOK_DST" ]] && grep -q "takeaway-side-effect" "$HOOK_DST" 2>/dev/null; then
  row PASS "hook-installed" "local post-commit remind already linked"
else
  row HOLD "hook-installed" "optional — bash scripts/takeaway-side-effect-remind.sh --install-hook"
fi

# Soft: never claim a takeaway was written; this is reminder infrastructure only
row PASS "no-memory-gym" "this script never opens Cursor chat or runs session close"

export OUT pass fail hold
set +e
python3 - <<'PY'
import json, os
from datetime import datetime, timezone
from pathlib import Path

out = Path(os.environ["OUT"])
pass_n = int(os.environ["pass"])
fail_n = int(os.environ["fail"])
hold_n = int(os.environ["hold"])
verdict = "GREEN" if fail_n == 0 else "RED"
advice = (
    "takeaway_side_effect_ready — remind surfaces present; piggyback on real work only"
    if verdict == "GREEN"
    else "takeaway_side_effect_hold — fix FAIL rows"
)
payload = {
    "schema": "gzmo.brain_feed.takeaway_side_effect/v1",
    "generated_at": datetime.now(timezone.utc).isoformat(),
    "verdict": verdict,
    "ok": fail_n == 0,
    "advice": advice,
    "auto_session": False,
    "counts": {"pass": pass_n, "fail": fail_n, "hold": hold_n},
    "doc": "docs/BRAIN_FEED.md",
    "operator": [
        "End real work with one gzmo session close --takeaway (or herdr close-ritual)",
        "Prefer living host enqueue; no --now while CT101 owns overnight",
        "Do not start a second agent chat whose only job is memory",
    ],
}
(out / "latest.json").write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
md = [
    "# Takeaway side-effect remind",
    "",
    f"Verdict: **{verdict}**",
    "",
    f"- Advice: {advice}",
    "- Auto-session: **false**",
    "",
    "```bash",
    "gzmo session close --takeaway '…one durable fact…'",
    "# or: herdr plugin pane open --plugin gzmo.metabolism --entrypoint close-ritual",
    "```",
    "",
    "See docs/BRAIN_FEED.md · docs/HERDR_METABOLISM.md",
    "",
]
(out / "latest.md").write_text("\n".join(md) + "\n", encoding="utf-8")
print(json.dumps({"verdict": verdict, "advice": advice, "pass": pass_n, "fail": fail_n, "hold": hold_n}, indent=2))
raise SystemExit(0 if fail_n == 0 else 1)
PY
GATE_EXIT=$?
set -e

echo "=== takeaway-side-effect done (exit $GATE_EXIT) ==="
exit "$GATE_EXIT"
