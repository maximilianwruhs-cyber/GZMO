# Würfel (Dice Loop) — FIFO and Isolation

Autopoietic `/dice` follow-up rolls scheduled by `gzmo-core/src/dice_loop.rs` and
fired from `daemon_cmd.rs` every 5 seconds when due.

## FIFO semantics

- One roll at a time: `mark_processing` sets in-flight state before skill dispatch.
- `schedule_from_roll` (inside `/dice --loop`) writes the next `fire_at` after completion.
- `max_chain_depth = 0` means unlimited chaining (see `DiceLoopConfig` docs).
- `PedagogySession.auto_triggers_enabled` and `[dice.loop].enabled` gate all fires.

## Synapse tagging

Each headless roll appends `chaos.dice_loop` with:

```json
{ "source": "wuerfel-cron", "headless": true, ... }
```

## Honeypot isolation

Facts from origins containing `wuerfel`, `dice_cascade`, or `wuerfel-cron` use
`container_tag = wuerfel-sandbox` (not default `obolus`).

## Kurator coordination

`kurator_monitor::record_dice_loop_fire` increments `dice_loops_seen` for session
`daemon`. When `dice_loops_seen >= [kurator].max_dice_loops_per_hour`, emits
`spawn.recommended` (human approval only; no autospawn).

## Bibliothek gate

Vault/KG promotion from dreams requires `[bibliothek].min_dream_cycles` successful
dream cycles (default 50). Würfel sandbox tags do not bypass this gate.
