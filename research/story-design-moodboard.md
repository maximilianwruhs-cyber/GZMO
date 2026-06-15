# `/story` Design Moodboard

We map GZMO's physical engine phases and derived LLM parameters to concrete narrative styles.

## Phase-Mood narrative mapping

| Phase | Voice / Style | Constraints & Focus | Example Output Direction |
|---|---|---|---|
| **Idle** (Tension < 30) | **Hemingway Sparse** (Calm / Reflective) | Concrete objects, flat declarations, minimal adjectives. Focus on quiet stillness. | *"The wrench. The oil. Nothing moved. The rain hit the tin roof. He waited."* |
| **Build** (Tension 30 - 70) | **Tension / rising pressure** (Urgent / Tightening) | Rising physical pressure, clockwork motifs, ticking clocks, breath tightening. | *"The gears did not slip. Every tooth held a fraction of the weight. Five more turns. A thread frayed."* |
| **Drop** (Tension > 70) | **Kafka Surreal** (Absurd / Disorienting) | Absurd bureaucracy, shifting geometry, body horror lite, rules that contradict. | *"The clerk insisted the permit was signed in a room that did not exist. The ceiling was lower today."* |

## Valence Tone Adjectives

We map `llm_valence` (ranges from -1.0 to 1.0) to emotional modifiers:

- **Valence > 0.3 (Positive / Reflective):** Warmth, acceptance, decay-as-peace.
- **Valence between -0.3 and 0.3 (Neutral):** Cold detachment, observation, mechanics.
- **Valence < -0.3 (Negative / Aggressive):** Friction, threat, heavy pressure, rot.
