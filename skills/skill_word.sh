#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /word — Invent a brand new word with definition and example (Deprecated)
# Delegates to Rust-native word implementation
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SKILLS_DIR")"

echo -e "\033[33mWarning: skills/skill_word.sh is deprecated. Routing to Rust-native /word skill...\033[0m" >&2

# Check for release or debug binary
if [ -f "$PROJECT_ROOT/target/release/gzmo" ]; then
    exec "$PROJECT_ROOT/target/release/gzmo" chaos skill word "$@"
elif [ -f "$PROJECT_ROOT/target/debug/gzmo" ]; then
    exec "$PROJECT_ROOT/target/debug/gzmo" chaos skill word "$@"
else
    echo -e "\033[31mError: gzmo binary not found. Please compile the project first using 'cargo build --release'.\033[0m" >&2
    exit 1
fi

