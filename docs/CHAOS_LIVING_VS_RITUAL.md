# Chaos: Living Park vs Chat/TUI Ritual

**Status:** Doctrine (2026-07-19)  
**Related:** [CHAOS_RHO_CONTROL_MODEL.md](./CHAOS_RHO_CONTROL_MODEL.md), [DISCOVERY_LIVING_WIRE.md](./DISCOVERY_LIVING_WIRE.md), plan scar `~/.cursor/plans/chaos_diagnosis_review_*.plan.md`

## Two modes (do not collapse)

| Mode | Where | Chaos / PulseLoop | Policy |
|------|-------|-------------------|--------|
| **Living metabolism** | CT101 `gzmo-daemon` | Chaos-free mentor path; tension oscillation **disabled** for discovery | Do not “fix” by editing JSON or re-enabling oscillation without operator intent |
| **Ritual / lab** | Workstation chat/TUI + `gzmo-next` | Opt-in PulseLoop; skills may emit `ChaosEvent` | TUI always runs PulseLoop when used; chat often has chaos **off** (`enabled_in_chat = false`) |

Deep ρ math and Thought Cabinet law remain valid engineering docs. Living CT101 deliberately runs **without** making chaos the overnight brain.

### C.0 feedback IPC boundary

The lab `chaos_feedback_inbox.jsonl` is drained only by the chat/TUI snapshot bridge into its
existing ritual `feedback_tx`. It is not daemon wiring: `daemon_cmd.rs` remains PulseLoop-free and
CT101 living KPI stays chaos-off.

## Critical scar: `CHAOS_STATE.json` is write-only

Nothing loads `CHAOS_STATE.json` into `PulseLoop` on boot. Boot uses `ChaosSnapshot::default()`. The file is **telemetry** rewritten every N ticks.

| Wrong remediation | Why it fails |
|-------------------|--------------|
| Edit JSON mutations / gravity then restart | Engine never reloads the file |
| Blame scheduler/distill for overwriting state | They do not write chaos JSON |
| Expect chat `/dice` to move phase with chaos off | Dead `feedback_tx` + static snapshot |

Tension is mostly **host-load smoothed** (HW blend). Skill deltas are small and wash out unless impulse/smoothing changes land.

## Operator checklist

1. Confirm surface: **TUI** (live engine) vs **chat** (often chaos-off) vs **daemon** (living park).
2. Watch the instance’s own state file (`data-next/CHAOS_STATE.json` on next; never assume legacy `data/`).
3. Do not treat chaos park on CT101 as a bug relative to shipped ρ docs — it is policy.
4. Implementation ideas (impulse field, `/transform` → `PersonaShift`) stay lab/TUI work — see chaos diagnosis plan.

## Links

- Shipped law: [CHAOS_RHO_CONTROL_MODEL.md](./CHAOS_RHO_CONTROL_MODEL.md)
- Capability tree: `docs/ct101-systems/60-chaos-engine/`
- Living wire: [DISCOVERY_LIVING_WIRE.md](./DISCOVERY_LIVING_WIRE.md)
