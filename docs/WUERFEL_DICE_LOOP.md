# Würfel (Dice Loop) — Lab Scheduling

`gzmo-core/src/dice_loop.rs` persists the next autopoietic `/dice` follow-up
after an enabled `/dice` roll. The loop is opt-in: `[dice.loop].enabled` defaults
to `false`.

## Scheduling semantics

- `/dice d6 --loop` and `/dice d20 --json` parse the die while ignoring mode tokens.
- A roll maps to a delay between `min_minutes` and `max_minutes`, then writes
  `data/dice_loop_state.json`.
- A natural 1 cancels the pending state by default (`cancel_on_nat_1 = true`).
- `max_chain_depth = 0` permits unlimited chaining; any positive value is a cap.

## Intentional boundary

Only scheduling lands in core. Main deliberately has no daemon fire path:
`daemon_cmd.rs` does not inspect due state or dispatch headless `/dice` ticks.
This remains a lab/opt-in future step so CT101 living KPIs stay chaos-free.
