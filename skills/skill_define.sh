#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /define [term] — Definition, pronunciation (IPA), and etymology (Deprecated)
# Delegates to Rust-native define implementation
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SKILLS_DIR")"

echo -e "\033[33mWarning: skills/skill_define.sh is deprecated. Routing to Rust-native /define skill...\033[0m" >&2

# Check for release or debug binary
if [ -f "$PROJECT_ROOT/target/release/gzmo" ]; then
    exec "$PROJECT_ROOT/target/release/gzmo" chaos skill define "$@"
elif [ -f "$PROJECT_ROOT/target/debug/gzmo" ]; then
    exec "$PROJECT_ROOT/target/debug/gzmo" chaos skill define "$@"
else
    echo -e "\033[31mError: gzmo binary not found. Please compile the project first using 'cargo build --release'.\033[0m" >&2
    exit 1
fi

