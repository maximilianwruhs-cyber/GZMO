# Architecture Fix Handoff — 2026-07-29 11:38 CEST

**Status:** DRAFT — ready for implementation  
**Author:** GLaDOS (operator surface audit)  
**Audience:** Max (operator) + any future agent that takes over  
**USP context:** Brain Feed / nutrient / airgap living — OpenClaw is operator surface, CT101 metabolizes

---

## Executive Summary

The GZMO ecosystem is **architecturally sound in design** but has **three critical operational gaps** that prevent it from being reliable:

1. **CT101 vault only has 240 facts** (needs ≥10,000 for living instance) — the vault at `/opt/gzmo/data/vault.db` is a lab/lite vault, not the living vault
2. **Embedding/Rerank services are down** — `192.168.31.110:8081` serves a static frontend (not embedding API), `192.168.31.110:8082` is CLOSED
3. **pi-gzmo-memory.sh bridge is empty** — workstation memory bridge has no status

The **Brain Feed pipeline is GREEN** (12/12 checks pass), the **gzmo-daemon on CT101 is active**, and the **workstation gzmo-serve is correctly inactive**. The systemd timers are running. The core architecture is fine — the infrastructure is just not talking to itself.

---

## Current State — Full Audit

### ✅ What's Working

| Component | Status | Details |
|-----------|--------|---------|
| **Prime inference** | ✅ PASS | `:8000` responding, model `qwen3.6-35b-mtp` |
| **Redis (CT101)** | ✅ PASS | `PONG`, 8 keys, 1 with expiry |
| **Qdrant (CT101)** | ✅ PASS | 1 collection (`honeypot`) |
| **gzmo-daemon (CT101)** | ✅ active | Overnight metabolism running |
| **gzmo-serve (workstation)** | ✅ inactive | Correctly not dual-writing |
| **Brain Feed** | ✅ GREEN | 12/12 pass, 0 fail, 0 hold |
| **herdr takeaway** | ✅ PASS | living-proof HIT present |
| **Felt Use depth** | ✅ PASS | 102/107 recall≥3 (95.3%), 74 ripen dual-gate |
| **Serendipity** | ✅ PASS | 4 candidates, human-gated, no auto-apply |
| **OpenClaw cron** | ✅ 9 jobs | All enabled, scheduled correctly |
| **systemd timers** | ✅ 8 timers | Running on schedule |
| **Cargo build** | ✅ PASS | `gzmo-cli` builds clean |
| **Unit tests** | ✅ PASS | 255/257 passed, 2 ignored |
| **ADR doctrine** | ✅ intact | ADR-0003, ADR-0004, ADR-0005 all present |
| **Sync contract** | ✅ intact | `sync-openclaw-workspace.sh` works |

### ❌ What's Broken

#### Critical (P0)

| Issue | Impact | Evidence |
|-------|--------|----------|
| **CT101 vault only 240 facts** | Cannot verify living attach, cannot enqueue takeaways remotely, cannot check daemon health | `living-attach-check.sh` FAILs: "SSH living probe failed (host=ct101)" |
| **Embedding service down** | No vector embeddings for search/RAG | `192.168.31.110:8081` returns 404, not 1024-dim |
| **Rerank service down** | No reranking for recall quality | `192.168.31.110:8082` returns empty/no response |

#### Warning (P1)

| Issue | Impact | Evidence |
|-------|--------|----------|
| **pi-gzmo-memory.sh empty** | Workstation memory bridge has no status | `pi-gzmo-memory.sh status` returns empty |
| **Git dirty** | Uncommitted `reconcile-report.json` | `git status --short` shows 1 changed file |
| **Subagent extension missing** | Optional — no impact on core | `pi install npm:pi-subagents` not installed |

---

## Root Cause Analysis

### 1. CT101 Vault — Most Likely Causes

The `BatchMode=yes` SSH to `ct101` is failing. This is either:

- **Host key changed** — CT101 was rebuilt/reinstalled, and the workstation's `known_hosts` has the old key
- **SSH service down** on CT101 (Proxmox container issue)
- **Network path broken** — CT101 IP (`192.168.31.202`) is unreachable from workstation
- **SSH config missing** — no `Host ct101` entry in `~/.ssh/config`
- **BatchMode restriction** — password auth required but BatchMode rejects it

**Diagnostic priority:**
```bash
# 1. Is the host reachable?
ping -c 3 192.168.31.202

# 2. Is SSH responding?
ssh -o ConnectTimeout=5 -o BatchMode=no ct101 'echo alive'

# 3. Check known_hosts
grep ct101 ~/.ssh/known_hosts

# 4. Check SSH config
grep -A5 'Host ct101' ~/.ssh/config 2>/dev/null || echo "No ct101 entry in SSH config"
```

### 2. Embedding/Rerank Host — `192.168.31.110`

This is a different host (not CT101). Likely:

- **Service crashed** — embedding server process died
- **Host is offline** — the machine at `192.168.31.110` is powered off or disconnected
- **Port changed** — service moved to a different port

**Diagnostic priority:**
```bash
# 1. Is the host alive?
ping -c 3 192.168.31.110

# 2. What's actually listening?
nmap -p 8081,8082 192.168.31.110

# 3. Check if it's a Docker container
ssh 192.168.31.110 'docker ps 2>/dev/null || systemctl list-units --type=service 2>/dev/null'
```

### 3. pi-gzmo-memory.sh Empty

This is the workstation-side memory bridge. "Empty status" could mean:

- The script exists but has no configured endpoints
- The MCP server isn't registered in OpenClaw config
- The bridge was never initialized after setup

**Diagnostic priority:**
```bash
# Check if the script exists and what it does
cat ~/github-clone/GZMO/scripts/pi-gzmo-memory.sh

# Check OpenClaw MCP server config
openclaw mcp show 2>/dev/null || true
```

---

## Implementation Plan

### Phase 1: Restore Connectivity (P0) — Estimated: 30-60 min

#### Step 1.1: Fix CT101 SSH

```bash
# Diagnose
ping -c 3 192.168.31.202
ssh -o ConnectTimeout=5 ct101 'systemctl is-active gzmo-daemon'

# If key mismatch:
ssh-keygen -R ct102  # remove old key
# Then reconnect (accept new key)
ssh ct101 'echo connected'

# If SSH config missing, add:
# Host ct101
#   HostName 192.168.31.202
#   User gzmo
#   IdentityFile ~/.ssh/id_ed25519
#   BatchMode yes
```

**Verify:**
```bash
bash ~/github-clone/GZMO/scripts/living-attach-check.sh
# Expected: PASS: living attach proof
```

#### Step 1.2: Restore Embedding/Rerank Services

```bash
# Diagnose host 192.168.31.110
ping -c 3 192.168.31.110

# If host is alive, check what's running:
ssh 192.168.31.110 'systemctl status embedding-service 2>/dev/null || docker ps'

# If service is down, restart:
# ssh 192.168.31.110 'systemctl restart embedding-service'
# or docker restart

# If host is dead, power on / reconnect
```

**Verify:**
```bash
curl -s http://192.168.31.110:8081/v1/embeddings?model=test
# Expected: 200 with embeddings array

curl -s http://192.168.31.110:8082/v1/rerank
# Expected: 200 with rerank response
```

### Phase 2: Restore Workstation Memory Bridge (P1) — Estimated: 15-30 min

#### Step 2.1: Fix pi-gzmo-memory.sh

```bash
# Check what the script expects
cat ~/github-clone/GZMO/scripts/pi-gzmo-memory.sh

# Check OpenClaw MCP server config
openclaw mcp show gzmo-memory 2>/dev/null || echo "Not registered"

# If MCP not registered, register it:
# openclaw mcp add gzmo-memory --stdio "bash scripts/pi-gzmo-memory.sh"
```

#### Step 2.2: Commit Git Dirty State

```bash
cd ~/github-clone/GZMO
git diff reconcile-report.json  # review
git add reconcile-report.json
git commit -m "chore: reconcile report"
git push
```

### Phase 3: Verify Full Stack (P0) — Estimated: 15 min

Run the full verification suite:

```bash
# 1. Living attach
bash ~/github-clone/GZMO/scripts/living-attach-check.sh

# 2. Brain Feed
bash ~/github-clone/GZMO/scripts/brain-feed-check.sh

# 3. Auto health (quick)
bash ~/github-clone/GZMO/scripts/auto-health-check.sh

# 4. Verify cron jobs
bash ~/openclaw/workspace/bin/list-gzmo-crons.sh

# 5. Verify systemd timers
systemctl --user list-timers --all | rg -i 'gzmo-|okforge'
```

**Expected final state:**
- `living-attach-check.sh` → PASS
- `brain-feed-check.sh` → GREEN (12/12)
- `auto-health-check.sh` → all PASS
- All 8 systemd timers active
- All 9 OpenClaw cron jobs enabled

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| CT101 Proxmox container issue | Medium | High | Check Proxmox web UI, container status |
| Embedding host permanently offline | Low | Medium | Consider running embeddings locally on CT101 |
| SSH key rotation needed | High | Medium | Document in `TOOLS.local.md` |
| Brain Feed degrades while embeds are down | Medium | Medium | Felt Use depth is still 95% — current knowledge is intact |

---

## What NOT to Do

1. **Do NOT start `gzmo-serve` on workstation** — dual-writer violation (ADR-0003)
2. **Do NOT curl upsert into Qdrant** — bypasses Brain Feed pipeline
3. **Do NOT set `GZMO_PRODUCT=1`** — breaks living attach contract
4. **Do NOT set `GZMO_ALLOW_LAB_VAULT=1`** — silences fact floor check
5. **Do NOT run `session close --now` on CT101** — CT101 owns metabolism
6. **Do NOT rebuild CT101** unless absolutely necessary — preserves vault state

---

## Future Improvements (Post-Fix)

1. **Add health check alerting** — when CT101 SSH or embedding services go down, get notified
2. **Document embedding host** — what's at `192.168.31.110`? Who manages it?
3. **Add backup verification** — CT101 vault backups should be verified, not just assumed
4. **Consider local embeddings** — if the embedding host is unreliable, run one on CT101
5. **Add systemd watchdog** — `gzmo-daemon` should have `Restart=always` and `RestartSec=10`
6. **Document SSH key rotation** — so this doesn't happen again after rebuilds

---

## Quick Reference

```bash
# Fix CT101 SSH
ping -c 3 192.168.31.202
ssh-keygen -R ct101
ssh ct101 'echo alive'

# Fix embeddings
ping -c 3 192.168.31.110
ssh 192.168.31.110 'systemctl status embedding-service'

# Verify everything
bash ~/github-clone/GZMO/scripts/living-attach-check.sh
bash ~/github-clone/GZMO/scripts/brain-feed-check.sh
bash ~/github-clone/GZMO/scripts/auto-health-check.sh
```

---

*The cake is a lie. But the fix is real. 🎂*
