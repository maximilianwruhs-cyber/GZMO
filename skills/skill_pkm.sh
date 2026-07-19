#!/usr/bin/env bash
# /pkm [category] — Deprecated: delegates to Rust-native Attractor Forge
set -euo pipefail
SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SKILLS_DIR")"
echo -e "\033[33mWarning: skills/skill_pkm.sh is deprecated. Routing to Rust-native /pkm skill (Pokemon Forge)...\033[0m" >&2
if [[ -x "$PROJECT_ROOT/target/release/gzmo" ]]; then
  exec "$PROJECT_ROOT/target/release/gzmo" chaos skill pkm "$@"
elif [[ -x "$PROJECT_ROOT/target/debug/gzmo" ]]; then
  exec "$PROJECT_ROOT/target/debug/gzmo" chaos skill pkm "$@"
fi
echo -e "\033[31mError: gzmo binary not found. Run: cargo build --release -p gzmo-cli\033[0m" >&2
exit 1
