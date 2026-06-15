#!/usr/bin/env bash
# /dice [D6|D20] — Deprecated: delegates to Rust-native dice roll
set -euo pipefail
SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SKILLS_DIR")"
echo -e "\033[33mWarning: skills/skill_dice.sh is deprecated. Routing to Rust-native /dice skill...\033[0m" >&2
if [[ -x "$PROJECT_ROOT/target/release/gzmo" ]]; then
  exec "$PROJECT_ROOT/target/release/gzmo" chaos skill dice "$@"
elif [[ -x "$PROJECT_ROOT/target/debug/gzmo" ]]; then
  exec "$PROJECT_ROOT/target/debug/gzmo" chaos skill dice "$@"
fi
echo -e "\033[31mError: gzmo binary not found. Run: cargo build --release -p gzmo-cli\033[0m" >&2
exit 1
