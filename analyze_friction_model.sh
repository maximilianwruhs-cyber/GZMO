#!/bin/bash
# Remediation for R1: Friction Model Analysis Script
# Purpose: Extract and log friction/crystallization telemetry to identify correlation with chaos.val, tension, or rho_mod.

LOG_DIR="gzmo_skills/scripts/discovery-remediations/mechanics-session-final-2026-06-16T14-57-29Z-061747"
mkdir -p "$LOG_DIR"
OUTPUT_FILE="$LOG_DIR/friction_analysis_$(date +%Y%m%d_%H%M%S).log"

echo "--- Friction Model Analysis Report ---" > "$OUTPUT_FILE"
echo "Timestamp: $(date)" >> "$OUTPUT_FILE"

# 1. Count total relevant events
COUNT=$(grep "chaos.dice_loop" /home/maximilian-wruhs/gzmo_skills/data/Synapse/events.jsonl | \
        jq -c 'select(.payload.friction != null or .payload.crystallize != null)' | wc -l)
echo "[INFO] Found $COUNT relevant events." >> "$OUTPUT_FILE"

# 2. Extract correlation data
echo "[INFO] Extracting correlation data (tick, friction, chaos_val, tension, rho_mod)..." >> "$OUTPUT_FILE"
grep "chaos.dice_loop" /home/maximilian-wruhs/gzmo_skills/data/Synapse/events.jsonl | \
jq -c 'select(.payload.friction != null or .payload.crystallize != null) | {tick: .tick, friction: .payload.friction, chaos_val: .payload.chaos_val, tension: .payload.tension, rho_mod: .payload.rho_mod}' >> "$OUTPUT_FILE"

echo "--- Analysis complete. Log written to $OUTPUT_FILE ---" >> "$OUTPUT_FILE"
