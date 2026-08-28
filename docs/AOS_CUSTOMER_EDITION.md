# AOS Customer Edition (Unpark Wave 4.1)

**Status:** Sketch / Later packaging (2026-07-19)  
**Wave:** 4.1 — [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md)  
**On top of:** living appliance — the only product ([ADR-0007](./adr/ADR-0007-one-product-living.md))

## Intent

One-curl (or near one-curl) Ubuntu path that stands up:

```text
Prime (local LLM) + gzmo-daemon + Redis + Qdrant + Neo4j + OKForge attach
```

Clients attach via `gzmo-living`. `install-gzmo.sh` → `~/.gzmo` is not a complete GZMO ([ADR-0007](./adr/ADR-0007-one-product-living.md)).

## Non-goals

| Not | Why |
|-----|-----|
| Default incomplete `~/.gzmo` | Not the product (ADR-0007) |
| Two overnight writers | ADR-0003 |
| Arena as required CE component | Wave 3 lab only |
| Pi as primary UX | Optional glass |

## Golden path (draft)

1. Install docker + compose  
2. `bash scripts/living-appliance-up.sh` (or CT101 pin promote)  
3. Install/release `gzmo` binary; point `GZMO_CONFIG` at living home  
4. Enable `gzmo-daemon` (systemd)  
5. Optional: OKForge / Observatory attach docs  
6. Verify: `bash scripts/living-readiness-gate.sh` + `bash scripts/ct101-living-appliance-smoke.sh`

## Exit criteria

- Separate install doc (this file) linked from [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md)  
- A stranger path still passes `product-readiness-gate.sh` without Redis/Qdrant/Neo4j  
- Demable CE smoke: `bash scripts/aos-ce-smoke.sh` → `data-next/aos-ce/{latest,golden-path}.json` (does not overwrite `~/.gzmo`)
