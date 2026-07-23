# Workstation wipe → reattach (operator)

**Status:** Active (2026-07-23)  
**Scope:** Local workstation artefacts only. **CT101 is not wiped** — living vault/daemon stay on CT101.  
**USP:** nutrient · Brain Feed · airgap living — not ecosystem tourism  
**Doctrine:** [ADR-0003](./ADR-0003-one-instance-metabolism.md) · [ADR-0004](./ADR-0004-airgap-living-usp.md)  
**Happy path attach:** [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md) · Pi diet: [PI_PACKAGE_ALLOWLIST.md](./PI_PACKAGE_ALLOWLIST.md) · Pi upgrades: [PI_UPGRADE_RUNBOOK.md](./PI_UPGRADE_RUNBOOK.md)

This guide restores an **operator laptop/desktop** after OS reinstall or home wipe, then reattaches agents to living memory on CT101. It does **not** restore CT101, migrate vaults, or invent a local living claim.

---

## 1. What survived (nothing to restore on the vault)

| On CT101 | Status after workstation wipe |
|----------|-------------------------------|
| `/opt/gzmo/data/vault.db` (~60k facts reference) | Intact — single living vault |
| `gzmo-daemon` + sidecars (Redis/Qdrant/Neo4j) | Intact — overnight writer stays here |
| `/opt/gzmo/gzmo.toml`, `/opt/gzmo/.env`, `/opt/gzmo/current` | Intact |
| Mentor socket `/opt/gzmo/data/gzmo_mentor.sock` | Intact when daemon is up |

Workstation wipe loses **SSH keys, agent MCP configs, local clones, API keys, Pi packages** — not living metabolism. Do not treat a fresh `~/.gzmo` or repo `data-next/` as a recovery source for living.

---

## 2. Back up before wipe (if you still can)

Copy off-box (USB / other host). Prefer tarballs of dirs; never commit secrets into git.

### Must-have

| Artefact | Why |
|----------|-----|
| `~/.ssh/` (private keys + `config`) | `Host ct101` / ProxyJump; without this, living attach fails closed |
| Cursor MCP home — usually `~/.cursor/mcp.json` (and any project `.cursor/mcp.json` you rely on) | `gzmo-living` stanza |
| `~/.pi/agent/mcp.json` | Pi `pi-mcp-adapter` reads this, not only `settings.json` |
| Optional: `~/.pi/agent/settings.json` (or full `~/.pi/agent/`) | Package allowlist state; snapshot before Pi upgrades too — see [PI_UPGRADE_RUNBOOK.md](./PI_UPGRADE_RUNBOOK.md) |
| Local API keys / `.env` you used for OpenRouter, Neo4j lab, etc. | Re-export after wipe; living Neo4j password stays authoritative on CT101 `/opt/gzmo/.env` |
| Git credentials / SSH deploy keys / `gh` auth notes | Clone + PR workflow |
| Hermes MCP config (if used) — paste target for `gzmo-living` | Emit path: `scripts/emit-living-mcp-fragment.sh --format hermes` |

### Nice-to-have

| Artefact | Why |
|----------|-----|
| List of Pi packages / output of `bash scripts/pi-thin-diet.sh --check` | Rebuild thin core + chosen QoL from [PI_PACKAGE_ALLOWLIST.md](./PI_PACKAGE_ALLOWLIST.md) |
| HSP sibling clone notes (`~/github-clone/HSP`) if you use audio theater | Optional — not Brain Feed |
| Workstation lab notes: `data-next/` paths you care about as **lab** only | Never merge into living vault |
| `~/.config/mcp/mcp.json` if you used the global merge target from `install-shared-mcp.sh` | Shared MCP home |

### Secrets hygiene

Do **not** back up agent-home files that embed plaintext passwords into a shared drive without encryption. Policy: [AGENT_HOME_SECRETS.md](./AGENT_HOME_SECRETS.md). Prefer re-pulling `NEO4J_PASSWORD` from CT101 via `install-shared-mcp.sh` after wipe.

Quick snapshot (example):

```bash
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "/media/backup/ws-wipe-$STAMP"
cp -a ~/.ssh "/media/backup/ws-wipe-$STAMP/ssh"
[[ -f ~/.cursor/mcp.json ]] && cp -a ~/.cursor/mcp.json "/media/backup/ws-wipe-$STAMP/"
[[ -d ~/.pi/agent ]] && cp -a ~/.pi/agent "/media/backup/ws-wipe-$STAMP/pi-agent"
# add local .env paths you actually use (gitignored)
```

---

## 3. After wipe — ordered restore

### 3.1 OS basics → git → SSH to CT101

1. Install git, `ssh`, Python 3, and your agent hosts (Cursor and/or Pi; Hermes optional).
2. Restore `~/.ssh` (keys + `config`). Expect `Host ct101` (and ProxyJump/`pve` if that was your topology). Key name historically: `id_sidecar_proxmox` — see [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md).
3. Prove SSH before any MCP work:

```bash
ssh -o BatchMode=yes -o ConnectTimeout=8 ct101 'hostname; whoami; test -f /opt/gzmo/data/vault.db && echo vault_ok'
# expect: CT101 / root (or your ops user) + vault_ok
```

Fallback if ProxyJump fails: `ssh pve "pct exec 101 -- …"` — still not a reason to stand up a local living vault.

### 3.2 Clone GZMO

```bash
mkdir -p ~/github-clone
git clone git@github.com:maximilianwruhs-cyber/GZMO.git ~/github-clone/GZMO
cd ~/github-clone/GZMO
```

Workstation is **operator/lab**. Do not enable overnight `gzmo-serve` while CT101 claims writer ([ADR-0003](./ADR-0003-one-instance-metabolism.md)).

### 3.3 Living MCP attach (prove ~60k facts)

Fail-closed kit — never starts `gzmo-serve`:

```bash
cd ~/github-clone/GZMO

# 1) Prove living (SSH → CT101 vault path + fact floor + dual_writer=false)
bash scripts/living-attach-check.sh

# 2) Merge gzmo-living into Cursor + Pi (+ global) MCP homes
bash scripts/install-shared-mcp.sh

# 3) External hosts (Hermes, etc.): emit + paste — do not invent hand-rolled SSH
bash scripts/emit-living-mcp-fragment.sh --format hermes   # or --format json
# Dry-run examples: docs/examples/hermes-gzmo-living.yaml
```

**Pass signals** (from [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md)):

| Proof | Pass |
|-------|------|
| Vault path | `/opt/gzmo/data/vault.db` (not `~/.gzmo`, not workstation `data-next/`) |
| Fact floor | `vault_facts` ≥ 10k; CT101 reference **~60k** |
| Dual-writer | workstation `gzmo-serve` **inactive** |
| MCP label | server name **`gzmo-living`** (not only `gzmo-memory`) |

### 3.4 Cursor vs Pi vs Hermes

| Host | Config | Restore path |
|------|--------|--------------|
| **Cursor** | `~/.cursor/mcp.json` | `install-shared-mcp.sh` merges `gzmo-living` → `scripts/pi-gzmo-mcp-serve.sh` |
| **Pi** | `~/.pi/agent/mcp.json` **required** (+ packages in `settings.json`) | Same installer for MCP; then thin diet (§3.5) |
| **Hermes** | Foreign YAML/JSON | `emit-living-mcp-fragment.sh --format hermes` → paste under `mcp_servers.gzmo-living` |

Lite product path (`gzmo-memory` / `~/.gzmo`) is a **separate** stranger bootstrap — [PRODUCT_MCP.md](./PRODUCT_MCP.md). Do not use `GZMO_ALLOW_LAB_VAULT=1` or `GZMO_PRODUCT=1` while claiming living.

### 3.5 Pi thin diet (core, then optional QoL)

```bash
bash scripts/pi-thin-diet.sh --apply-core --dry-run
bash scripts/pi-thin-diet.sh --apply-core
bash scripts/pi-thin-diet.sh --check
```

Optional QoL only from the allowlist (not a store shopping list):

```bash
bash scripts/pi-thin-diet.sh --apply-recommended --with spark,plan,ask,permissions,web,skillful
# see docs/PI_PACKAGE_ALLOWLIST.md for flags
```

Core doctrine: one MCP adapter story, one `gzmo-pi` source (git **or** npm), optional `hsp-pi`, one `pi-subagents`. Deny competing memory packages (`pi-memory`, `pi-hermes-memory`, …).

### 3.6 HSP optional

Audio / metabolism theater only — **not** living GREEN, **not** Brain Feed:

- Pi package: `npm:hsp-pi` (via `--apply-core` optional / allowlist)
- Sibling: `~/github-clone/HSP` + demos in [HSP_DEMO.md](./HSP_DEMO.md) (`hsp-emit-demo.sh`, optional `hsp ping`)

Skip entirely if you do not use sonification.

### 3.7 ADR-0003 — no workstation overnight writer

While CT101 holds the living claim:

```bash
systemctl --user stop gzmo-serve.service 2>/dev/null || true
systemctl --user disable gzmo-serve.service 2>/dev/null || true
systemctl --user is-active gzmo-serve.service   # expect: inactive
```

**Never** enable workstation `gzmo-serve` “to make attach work.” Attach is MCP/`mcp-serve` over the SSH wrapper — not a second overnight writer. Explicit living-host moves use `scripts/living-host-mutex.sh` and are out of scope for a wipe restore.

---

## 4. Verification ladder

Run in order; stop on first FAIL.

| Step | Command / action | Expect |
|------|------------------|--------|
| 1 | `ssh ct101 'test -f /opt/gzmo/data/vault.db && echo ok'` | `ok` |
| 2 | `bash scripts/living-attach-check.sh` | Exit 0; vault under `/opt/gzmo/data/`; facts ≥ 10k (~60k); `dual_writer=false` |
| 3 | In Cursor/Pi: tools on **`gzmo-living`** → `gzmo_memory_status` | `vault_path` → `/opt/gzmo/data/vault.db`; large `vault_facts` |
| 4 | Optional: `bash scripts/organ-trace.sh --living` | Lab artefact under `data-next/organ-trace/` (soft miss OK); never dual-writer |
| 5 | Optional: `bash scripts/ct101-living-smoke.sh` | Daemon/sidecars healthy on CT101 |
| 6 | Pi: `bash scripts/pi-thin-diet.sh --check` | No deny-list / duplicate gzmo-pi fights |

Partial env-only probe (no SSH): `bash scripts/living-attach-check.sh --local-only` — **not** vault proof; re-run full check when CT101 is reachable.

---

## 5. Do not

| Don’t | Why |
|-------|-----|
| Merge lab `~/.gzmo` into living / CT101 vault | Lab/lite ≠ living; corrupts the USP story |
| Dual-write (workstation `gzmo-serve` + CT101 daemon) | Violates ADR-0003 |
| Claim overnight soak GREEN from one attach probe | Craft first; no fake soaks |
| Restore living from workstation `data-next/` or a tiny local vault | Wrong vault; attach-check should refuse |
| Set `GZMO_ALLOW_LAB_VAULT=1` or `GZMO_PRODUCT=1` “to make living work” | False-positive / product marker; attach kit FAILs |
| Hand-roll `ssh … gzmo mcp-serve` without living `GZMO_CONFIG=/opt/gzmo/gzmo.toml` | Wrong HOME/config scars on CT101 — use emit + `pi-gzmo-mcp-serve.sh` |
| Install Pi competing memory packages for “recall” | Living memory is MCP `gzmo-living` |

---

## 6. If CT101 is unreachable

That is **not** this guide.

- Do **not** invent a local vault as living.
- Do **not** point agents at `data-next/`, `~/.gzmo`, or a fresh `gzmo init` and call it CT101 living.
- Fix SSH / LAN / LXC first ([CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md) for CT101-side living restore; [REBOOT_STARTUP.md](./REBOOT_STARTUP.md) for cold-start).
- Until attach-check passes against `/opt/gzmo/data/vault.db`, treat memory tools as **offline** — lite `gzmo-memory` is a separate product path, not a substitute claim.

---

## Related

- [EXTERNAL_LIVING_ATTACH.md](./EXTERNAL_LIVING_ATTACH.md) — DO / NEVER + emit/check kit  
- [MCP_LOCAL_ATTACH.md](./MCP_LOCAL_ATTACH.md) — brand stdio attach contract  
- [PI_GZMO_MEMORY_INTEGRATION.md](./PI_GZMO_MEMORY_INTEGRATION.md) — ops SSH living  
- [CT101_RESTORE_LIVING.md](./CT101_RESTORE_LIVING.md) — CT101 living claim restore (different problem)  
- [GZMO_NEXT_RUNBOOK.md](./GZMO_NEXT_RUNBOOK.md) — workstation lab / next instance  
- Skill: `skills/workflows/living-attach/SKILL.md`
