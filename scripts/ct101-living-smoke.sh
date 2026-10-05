#!/usr/bin/env bash
# DEPRECATED (CUTOVER A, 2026-09-30) — kept for old call sites & muscle memory.
# Real implementation: scripts/living-smoke.sh (local-first, SSH fallback).
exec bash "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/living-smoke.sh" "$@"
