# External living attach (agents)

**Status:** Agent-facing safe attach kit (2026-07-22)  
**USP:** nutrient · Brain Feed · airgap living — not ecosystem tourism  
**Doctrine:** [ADR-0003](./ADR-0003-one-instance-metabolism.md) · [ADR-0004](./ADR-0004-airgap-living-usp.md) · [MCP_LOCAL_ATTACH.md](./MCP_LOCAL_ATTACH.md)  
**Research:** [living-external-attach-plug-and-play-2026-07-22.md](../research/living-external-attach-plug-and-play-2026-07-22.md)

Living attach is **not** “any stdio `gzmo mcp-serve`.” It is a labeled contract: server name `gzmo-living` + living `GZMO_CONFIG` (or the official SSH wrapper) + vault proof.

## DO THIS (happy path)

```bash
# 1) Prove living (fail-closed; read-only; never starts gzmo-serve)
bash scripts/living-attach-check.sh

# 2) Emit one stanza (stdout) — do not invent hand-rolled SSH
bash scripts/emit-living-mcp-fragment.sh --format hermes
# or JSON:
bash scripts/emit-living-mcp-fragment.sh --format json

# 3) Paste under mcp_servers.gzmo-living (Hermes) or mcpServers["gzmo-living"] (Cursor-shaped JSON)
# Repo-owned dry-run examples (safe): docs/examples/hermes-gzmo-living.yaml
```

**Ops SSH (operator LAN):** fragment command = `scripts/pi-gzmo-mcp-serve.sh` (sets remote `GZMO_CONFIG=/opt/gzmo/gzmo.toml`).  
**Airgap on-box (USP):** local `gzmo mcp-serve` with living `GZMO_CONFIG` — prefer this when the agent runs on the living host.

Cursor/Pi operators can still run `bash scripts/install-shared-mcp.sh` (merges known JSON homes). External hosts: **emit + paste**; do not thrash foreign configs blindly.

### Attach truth (not folklore)

| Proof | Pass |
|-------|------|
| `gzmo_memory_status` / CLI `memory status --json` | `vault_path` under living data (CT101: `/opt/gzmo/data/vault.db`) |
| Fact floor | `vault_facts` ≥ 10k (CT101 reference ~60k) |
| Dual-writer | workstation `gzmo-serve` **inactive** while CT101 owns overnight |
| MCP label | server name **`gzmo-living`** |

`"connection closed: initialized request"` after `timeout 5 … mcp-serve` is a **truncated handshake**, not living attach. MCP serve does **not** require `~/.gzmo/SOUL.md`.

## NEVER DO THAT

| Action | Why |
|--------|-----|
| Keep only `gzmo-memory` while claiming living | Lite/lab path (`~/.gzmo`, often ~hundreds of facts) |
| Set `GZMO_ALLOW_LAB_VAULT=1` “to make living work” | Silences ≥10k refuse; Hermes false-positive |
| Set `GZMO_PRODUCT=1` on living | Product/lite marker; attach-check FAILs |
| Hand-roll `ssh … gzmo mcp-serve` without `GZMO_CONFIG=/opt/gzmo/gzmo.toml` | Wrong HOME/config scars on CT101 |
| Enable workstation `gzmo-serve` / second overnight writer | Violates ADR-0003 single writer |
| Rewrite CT101 code via the attach path | Attach is read/search; metabolism stays on CT101 daemon |
| Public / WAN HTTP MCP as the living SKU | Out of brand (ADR-0004) |
| Claim overnight soak GREEN from one attach probe | Craft first; no fake soaks |

## Guardrails baked into this kit

- `living-attach-check.sh` — exits non-zero on lab vault, PRODUCT/LAB-ALLOW conflict, dual-writer, SSH misconfig, low fact count  
- `emit-living-mcp-fragment.sh` — refuses to emit if PRODUCT or LAB-ALLOW set; emits **only** `gzmo-living`  
- `pi-gzmo-mcp-serve.sh` — refuses PRODUCT / LAB-ALLOW; only remote `mcp-serve` (never starts local `gzmo-serve`)

## Related

- [WORKSTATION_WIPE_RESTORE.md](./WORKSTATION_WIPE_RESTORE.md) — after local wipe: backup list + ordered reattach (CT101 vault stays)  
- [PI_GZMO_MEMORY_INTEGRATION.md](./PI_GZMO_MEMORY_INTEGRATION.md) — ops SSH living  
- [AIRGAP_LIVING.md](./AIRGAP_LIVING.md) — on-box USP  
- [PRODUCT_MCP.md](./PRODUCT_MCP.md) — lite `gzmo-memory` stranger path (separate)  
- Skill: `skills/workflows/living-attach/SKILL.md`
