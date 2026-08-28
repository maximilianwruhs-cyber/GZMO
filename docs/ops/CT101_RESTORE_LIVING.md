# CT101 restore living — runbook (2026-07-17)

**Purpose:** Reverse the 2026-07-15/16 workstation cutover so CT101 is again the sole living metabolism brain.  
**Related:** [ADR-0003-one-instance-metabolism.md](../adr/ADR-0003-one-instance-metabolism.md), [CT101_BOUNDARY.md](./CT101_BOUNDARY.md), [PLACEMENT_DECISION.md](../PLACEMENT_DECISION.md)

## Non-goals

- Do **not** merge workstation `data-next/vault.db` into CT101
- Do **not** run `gzmo-serve` overnight alongside `gzmo-daemon`
- Do **not** graft lab `[assembly]` into CT101 `gzmo.toml`

## Checklist

### 1. Stop dual overnight writers (workstation)

```bash
systemctl --user stop gzmo-serve.service
systemctl --user disable gzmo-serve.service
systemctl --user is-active gzmo-serve.service gzmo-scheduler.service
# expect: inactive inactive
```

### 2. Confirm CT101 daemon

```bash
ssh pve "pct exec 101 -- systemctl is-active gzmo-daemon"
# expect: active
```

### 3. Operator SSH (`Host ct101` in `~/.ssh/config`)

Ensure workstation `~/.ssh/id_sidecar_proxmox.pub` is in CT101 `/root/.ssh/authorized_keys` (and `/home/maximilian/.ssh/authorized_keys` if needed):

```bash
ssh ct101 'hostname; whoami'
# expect: CT101 / root
```

Fallback if ProxyJump SSH fails: `ssh pve "pct exec 101 -- …"`.

### 4. Health gate

```bash
ssh ct101 'systemctl is-active gzmo-daemon'
ssh ct101 'docker ps --format "table {{.Names}}\t{{.Status}}"'
ssh ct101 'cd /opt/gzmo && GZMO_CONFIG=/opt/gzmo/gzmo.toml /opt/gzmo/current/target/release/gzmo health'
ssh ct101 'sqlite3 /opt/gzmo/data/vault.db "SELECT COUNT(*) FROM semantic_vault;"'
# or from workstation:
bash scripts/ct101-living-smoke.sh
```

Expect: daemon active; redis/qdrant/neo4j Up; health OK (honeypot↔Qdrant drift may WARN); vault ~60k facts.

Canonical deploy paths: [CT101_DEPLOY.md](./CT101_DEPLOY.md) (`/opt/gzmo/current` → release tree).

### 5. Living mentor + discovery attach

```bash
ssh ct101 'test -S /opt/gzmo/data/gzmo_mentor.sock && \
  GZMO_CONFIG=/opt/gzmo/gzmo.toml /opt/gzmo/current/target/release/gzmo mentor ping'
# expect: pong

# Pi discovery preflight requires this socket (not OpenRouter-only).
# Details: DISCOVERY_LIVING_WIRE.md
```

Expect: chaos-free mentor on dedicated daemon thread; Pi `gzmo_mentor_teach` uses the socket (no `fallback:"openrouter"` when daemon is up).

### 6. Policy docs

Confirm ADR-0003 / CT101_BOUNDARY / PLACEMENT / CORE_INSIGHT / GZMO_NEXT_RUNBOOK / `config/gzmo-next.toml` header describe CT101 as living and workstation as operator/lab.

## Success criteria

1. Workstation `gzmo-serve` stopped and disabled  
2. CT101 `gzmo-daemon` + sidecars active; vault still ~60k facts  
3. `ssh ct101` works from workstation  
4. Mentor socket answers `ping` at `/opt/gzmo/data/gzmo_mentor.sock`  
5. Docs say CT101 is living; workstation is operator/lab only  

## Topology

- **Living:** CT101 `.202` — `/opt/gzmo/`, `gzmo-daemon`, Docker sidecars, cloud-first cognition, mentor socket  
- **Operator:** Workstation `.184` — Prime `:8000`, CLI/Pi/Cursor  
- **Retrieval:** VM200 `.110` — embed/rerank `:8081`  
- **Discovery ops:** [DISCOVERY_LIVING_WIRE.md](../DISCOVERY_LIVING_WIRE.md)
