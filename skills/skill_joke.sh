#!/usr/bin/env bash
# /joke [topic] — Deprecated: delegates to Rust-native Attractor Comedy
set -euo pipefail
SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SKILLS_DIR")"
echo -e "\033[33mWarning: skills/skill_joke.sh is deprecated. Routing to Rust-native /joke skill...\033[0m" >&2
if [[ -x "$PROJECT_ROOT/target/release/gzmo" ]]; then
  exec "$PROJECT_ROOT/target/release/gzmo" chaos skill joke "$@"
elif [[ -x "$PROJECT_ROOT/target/debug/gzmo" ]]; then
  exec "$PROJECT_ROOT/target/debug/gzmo" chaos skill joke "$@"
fi
echo -e "\033[31mError: gzmo binary not found. Run: cargo build --release -p gzmo-cli\033[0m" >&2
exit 1
