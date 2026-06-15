# `/story` Skill Audit

An audit of the current story skill, the card skill, generative infrastructure, and the chaos engine to identify integration vectors.

## Codebase Archaeology Summary

| File | Role & Observations | Integration Vector for `/story` |
|------|---------------------|--------------------------------|
| `gzmo-core/src/skills/story.rs` | Current baseline (50 lines). Thin LLM wrapper. Keyword defaults to `"chaos"`. Emits `ChaosEvent::StoryGenerated`. | Needs complete rewrite to construct `StoryBrief` and use chaos variables. |
| `gzmo-core/src/skills/card.rs` | Forges MTG cards. Uses `chaos_index` to pick color, rarity, card type. Structures system prompt. | Excellent reference for using dynamic constraints derived from the snapshot. |
| `gzmo-core/src/skills/generative.rs` | Shared LLM quality-gate / persona infrastructure. Sets persona overrides. | We can use `try_generative` with dynamic system prompts and user prompts. |
| `gzmo-chaos/src/pulse.rs` | Heartbeat (174 BPM). Broadcasts `ChaosSnapshot`. | Target coordinates: `tick`, `phase`, `llm_valence`, `rho_effective`, `thoughts_incubating`. |
| `gzmo-chaos/src/thoughts.rs` | Thought Cabinet. Story has 40-tick incubation and +0.5 $\rho$ crystallization impulse. | Need to pull incubating thought previews. `ChaosSnapshot` lacks this today. |
| `gzmo-chaos/src/feedback.rs` | Emits events. `StoryGenerated` has `text: String`. | We can keep it simple or optionally extend it. |
| `gzmo-cli/src/chaos_skill_cmd.rs` | CLI entry point. Loads snapshot from `CHAOS_STATE.json`. | Loads state once. Stale state issue when daemon runs. |
| `skills/skill_story.sh` | Shell parity script. Hardcoded temperature 0.85. | Must deprecate shell skill or map the dynamic logic there too. |
| `gzmo-core/src/skills/persona.rs` | Active transform state. | Persona parameters (temp, top_p, prompt) already wire into `generative.rs`. |

## Integration Matrix

| Field/Pattern | Used by card? | Used by story? | Recommendation |
|---|---|---|---|
| `chaos_index` | yes | no | Adopt for secondary choices if needed. |
| `llm_valence` | no | no | Map to mood/tone axis in dynamic prompts. |
| `phase` (Idle/Build/Drop) | display only | no | Map to narrative mode (Hemingway vs Kafka). |
| `incubating thoughts` | no | no | Extract previews to feed narrative continuation. |
| `nonce` | no | no | Generate using tick/hash/counter to guarantee prompt variation. |
| `anti-repeat ledger` | no | no | Implement file-based rolling hash ledger to reject exact repeats. |
