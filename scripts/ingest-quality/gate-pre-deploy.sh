#!/bin/bash
# gate-pre-deploy.sh  —  M4: Pre-deploy regression gate (lightweight)
# WARNING: runs before every daemon restart — must complete in < 2 seconds.
# Full eval is run separately via scripts/ingest-quality/replay-wave.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "=== M4 Pre-deploy Gate ==="

# 1. Check database exists and is readable
DB="/opt/gzmo/data/vault.db"
if [ ! -f "$DB" ]; then
    echo "❌ FAIL: vault.db not found at $DB"
    exit 1
fi
echo "✅ vault.db exists ($(du -h "$DB" | cut -f1))"

# 2. Quick honeypot ratio check (no LLM, pure SQLite)
python3 -c "
import sqlite3
c = sqlite3.connect('$DB')
v = c.execute('SELECT COUNT(*) FROM semantic_vault').fetchone()[0]
h = c.execute('SELECT COUNT(*) FROM honeypot').fetchone()[0]
ratio = h / v * 100 if v > 0 else 0
c.close()
print(f'[*] Honeypot ratio: {ratio:.1f}%')
if ratio > 90:
    exit(1)
" 2>&1 || {
    echo "❌ FAIL: Honeypot ratio > 90% — eligibility too loose"
    exit 1
}

echo "✅ Honeypot ratio in range"
echo "=== Gate PASSED ==="