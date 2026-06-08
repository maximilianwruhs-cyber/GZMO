#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════════
# /stabilize — Stabilizes the chaos engine attractor by decreasing rho
# ═══════════════════════════════════════════════════════════════════

SKILLS_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SKILLS_DIR/_llm_helper.sh"

print_box "STABILIZE" "Attractor stabilized. Lorenz ρ mod decreased by 1.0" "🌀" "$C_GREEN"
