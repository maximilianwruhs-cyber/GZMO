#!/usr/bin/env bash
# Post-remediation verification matrix (docs/INFRASTRUCTURE_REMEDIATION_IMPLEMENTATION_GUIDE.md §10).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GZMO="${GZMO:-$ROOT/target/release/gzmo}"
PASS=0
FAIL=0
WARN=0

ok() { echo "[PASS] $*"; PASS=$((PASS + 1)); }
fail() { echo "[FAIL] $*"; FAIL=$((FAIL + 1)); }
warn() { echo "[WARN] $*"; WARN=$((WARN + 1)); }

echo "== GZMO remediation verify $(date -Iseconds) =="
echo "ROOT=$ROOT"
echo ""

# Host health
if "$ROOT/scripts/auto-health-check.sh" >/tmp/gzmo-remediation-health.txt 2>&1; then
  ok "auto-health-check.sh"
else
  fail "auto-health-check.sh (see /tmp/gzmo-remediation-health.txt)"
fi

# MCP
if "$ROOT/scripts/verify-mcp-json.sh" >/dev/null 2>&1; then
  ok "verify-mcp-json.sh"
else
  fail "verify-mcp-json.sh"
fi

# Graph validate
if "$GZMO" pedagogy graph validate "$ROOT/data/pedagogy/graphs/" >/dev/null 2>&1; then
  ok "pedagogy graph validate"
else
  fail "pedagogy graph validate"
fi

# Skill feedback + audit
if "$GZMO" chaos skill dice --json >/tmp/gzmo-remediation-dice.json 2>/dev/null; then
  ok "chaos skill dice --json"
  sleep 2
  if "$GZMO" chaos feedback-audit --tail 3 2>/dev/null | grep -q '"source"'; then
    ok "chaos feedback-audit has entries"
  else
    warn "chaos feedback-audit empty (daemon may need a tick)"
  fi
else
  fail "chaos skill dice"
fi

# Honeypot rejects CLI
if "$GZMO" honeypot rejects --tail 1 >/dev/null 2>&1; then
  ok "honeypot rejects CLI"
else
  fail "honeypot rejects CLI"
fi

# Low-tension config
if python3 -c "
import tomllib, pathlib
p = pathlib.Path('$ROOT/gzmo.toml')
cfg = tomllib.loads(p.read_text())
lt = cfg.get('pedagogy', {}).get('low_tension_dialogue', {})
assert lt.get('enabled') is True
assert float(lt.get('threshold', 0)) >= 18.0
print('threshold', lt.get('threshold'))
" 2>/dev/null; then
  ok "low_tension_dialogue config"
else
  fail "low_tension_dialogue config"
fi

# Container LAN forwards (host only)
if [[ -f "/.dockerenv" ]]; then
  warn "container-lan-forward (skip inside container)"
else
  if "$ROOT/scripts/container-lan-forward.sh" status 2>/dev/null | grep -q '\[UP\]'; then
    ok "container-lan-forward running"
  else
    warn "container-lan-forward not running (./scripts/container-lan-forward.sh start)"
  fi
fi

# Synapse event diversity (display filter inputs)
if python3 -c "
import json
from collections import Counter
path='$ROOT/data/Synapse/events.jsonl'
want={'mentor_teach','ingest_complete','spark_complete','chaos.dice_loop','chaos.feedback_drained','topic_shift_distill'}
seen=set()
with open(path) as f:
    for line in f:
        try:
            o=json.loads(line)
            if o.get('event_type') in want:
                seen.add(o['event_type'])
        except: pass
missing=want-seen
print('seen', sorted(seen))
if 'mentor_teach' in seen and 'ingest_complete' in seen:
    exit(0)
exit(1)
" 2>/dev/null; then
  ok "synapse has display event types"
else
  warn "some synapse display event types not yet seen (feedback_drained needs post-drain)"
fi

echo ""
echo "════════════════════════════════════════"
echo "PASS=$PASS  FAIL=$FAIL  WARN=$WARN"
echo "════════════════════════════════════════"
[[ "$FAIL" -eq 0 ]]
