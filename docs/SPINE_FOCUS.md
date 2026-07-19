# Spine focus — two pillars

**Status:** Active (2026-07-19)  
**Supersedes for product direction:** expanding the opportunity-map zoo  
**Audience:** Operator deciding what to strengthen next

## Verdict

Stop expanding Arena / HSP / € / AOS / research stubs. Concentrate on two pillars:

1. **Living overnight metabolism** — chat → distill → vault/honeypot → recall you can feel  
2. **Product memory MCP** — `gzmo memory mcp` / `mcp-serve` as the attach surface for other agents

Everything else stays parked as nightburst spikes unless it directly protects or demos those pillars.

## Living vault owner

| Role | Host | Path | Overnight writer |
|------|------|------|------------------|
| **Living production** | **CT101** | `/opt/gzmo/` + `gzmo-daemon` | Yes — sole writer |
| **Operator / lab** | Workstation | `GZMO/data-next/` | No overnight `gzmo serve` while CT101 lives |

This is already ADR-0003 / [CT101_BOUNDARY.md](CT101_BOUNDARY.md). Nightburst artifacts under `data-next/` prove compressed recipes; they do **not** replace CT101 as the living brain.

## Keep / Park / Later

See the top of [STACK_OPPORTUNITY_MAP.md](STACK_OPPORTUNITY_MAP.md).

## Demable check (≈5 minutes)

```bash
# Product MCP cold path (no CT101 / LAN required)
./scripts/verify-product-mcp.sh

# Spine demo: product MCP + lab recall-proof + owner doctrine
bash scripts/spine-demo.sh
# → data-next/spine-demo/latest.{json,md}
```

Stranger test: can you show (1) MCP search/status works on a fresh `gzmo init` dir, and (2) a recall-proof log where seeded facts stuck after metabolism?

## Related

- [PRODUCT_MCP.md](PRODUCT_MCP.md)
- [GZMO_NEXT_RUNBOOK.md](GZMO_NEXT_RUNBOOK.md)
- [CORE_INSIGHT.md](CORE_INSIGHT.md)
- [ADR-0003-one-instance-metabolism.md](ADR-0003-one-instance-metabolism.md)
