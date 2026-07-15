# Subsystem — LXC Host Provisioning

**Source:** `swap/scripts/setup_lxc101.sh`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Automates CT101 bootstrap from the Proxmox host: installs Docker CE inside LXC container **101**, deploys the GraphRAG database sidecar stack to `/opt/database-cluster`, and starts Redis, Qdrant, and Neo4j. Enables one-shot homelab provisioning without Ansible/Terraform on the hot path.

---

## 2. How it works

The script runs on PVE and uses `pct exec 101` for every in-container step:

```11:14:swap/scripts/setup_lxc101.sh
# 1. Update and install dependencies
echo ">>> [1/5] Installing base packages inside LXC 101..."
pct exec $CONTAINER_ID -- apt-get update
pct exec $CONTAINER_ID -- apt-get install -y ca-certificates curl gnupg
```

Docker GPG key and apt repo are resolved from the container's architecture and Debian codename:

```23:29:swap/scripts/setup_lxc101.sh
echo ">>> [3/5] Resolving architecture and codename..."
ARCH=$(pct exec $CONTAINER_ID -- dpkg --print-architecture | tr -d '\r\n')
CODENAME=$(pct exec $CONTAINER_ID -- sh -c '. /etc/os-release && echo $VERSION_CODENAME' | tr -d '\r\n')
echo "Resolved: Arch=$ARCH, Codename=$CODENAME"

echo ">>> Appending Docker repositories to sources.list.d..."
pct exec $CONTAINER_ID -- sh -c "echo 'deb [arch=$ARCH signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian $CODENAME stable' > /etc/apt/sources.list.d/docker.list"
```

Compose is piped from a host-side temp file into the container:

```41:47:swap/scripts/setup_lxc101.sh
echo ">>> [6/6] Injecting GraphRAG Database Stack and launching..."
pct exec $CONTAINER_ID -- mkdir -p /opt/database-cluster
cat "$COMPOSE_SRC" | pct exec $CONTAINER_ID -- sh -c "cat > /opt/database-cluster/docker-compose.yml"

echo ">>> Launching Neo4j, Qdrant, and Redis containers..."
pct exec $CONTAINER_ID -- sh -c "cd /opt/database-cluster && docker compose up -d"
```

---

## 3. Interfaces

| Interface | Value |
|-----------|-------|
| Container ID | `101` (constant) |
| Compose source on PVE | `/tmp/db-compose.yml` (must be copied from `swap/templates/database-cluster-compose.yml` before run) |
| In-container compose path | `/opt/database-cluster/docker-compose.yml` |
| CT101 LAN IP | `192.168.31.202` |
| Ops entry | `ssh pve "pct exec 101 -- …"` |

---

## 4. THINKING nodes

> **THINKING — setup_lxc101.sh:container bootstrap**
> - *Reviewed:* Steps 1–5 install Docker CE; step 6 injects compose and runs `docker compose up -d`.
> - *Insight:* Provisioning is entirely imperative shell — no idempotency checks beyond Docker's own restart policy.
> - *Risk / limitation:* `COMPOSE_SRC=/tmp/db-compose.yml` must exist on PVE; script does not copy the template itself.
> - *Enhancement:* Auto-copy from repo path or accept `--compose` flag. [CT101-safe]

> **THINKING — setup_lxc101.sh:pct exec pattern**
> - *Reviewed:* All commands go through `pct exec $CONTAINER_ID`.
> - *Insight:* Host never SSHes into CT101 directly; Proxmox is the only ops bridge.
> - *Risk / limitation:* Requires root on PVE; no rollback if compose fails mid-flight.
> - *Enhancement:* Pre-flight check that container 101 is running. [CT101-safe]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| GZMO-next workstation | Same script pattern reusable for a dev LXC; not needed on workstation (native Docker) |
| CT101 production | Frozen — re-run only for disaster recovery or sidecar upgrades |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Copy compose template automatically from repo | [CT101-safe] |
| 2 | Verify sidecar health after `docker compose up -d` | [CT101-safe] |
| 3 | Pin image digests in compose for reproducible deploys | [CT101-safe] |
| 4 | Terraform/Pulumi wrapper for LXC + compose | [GZMO-next] |
