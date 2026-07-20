# Portable GZMO core — RFC (Unpark Wave 4.4)

**Status:** RFC / inventory-first (2026-07-19)  
**Wave:** 4.4 — [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md)  
**Default:** against big-bang rewrite until this RFC is accepted in a dedicated PR

## Problem

CT101 living stack carries theatrical/ops baggage. A portable core would extract metabolism + MCP without host-specific paths.

## Non-goals (v0)

- Big-bang rewrite of `gzmo-core`  
- Moving living SoT off CT101 without ADR update  
- Bundling Arena/HSP/pantheon into the core crate

## Approach

1. **Inventory** top-level `gzmo-core` mods from `lib.rs` into product / living / theater_or_ops seams  
2. Demable: `bash scripts/portable-core-inventory.sh` → `data-next/portable-core/{latest.json,inventory.md}`  
3. **Define** a `gzmo-core` feature matrix (`product`, `living`) before any extract (sketched when Cargo has no `[features]` yet)  
4. **Prove** with existing gates: product-readiness without living features; living-readiness with them  

## Acceptance

- Written inventory table merged (`module_seams` in latest.json)  
- Feature flags sketched (no mandatory extract); **`hold_rewrite` default**  
- A+C gates still GREEN  

## Rejected for now

Full portable rewrite (`hold_rewrite` remains default until a follow-up RFC PR).
