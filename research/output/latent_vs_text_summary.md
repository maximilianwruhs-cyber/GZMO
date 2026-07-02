## Text-based MAS baseline (mas_text_baseline.json)

- **four_agent**: latency=2556ms, tokens=1469, success=100%
- **single**: latency=1649ms, tokens=442, success=100%
- **two_agent**: latency=1137ms, tokens=503, success=100%

## Latent bridge run (mas_latent_real_compare.json)

- **recursive_mas**: latency=7468ms, tokens=0, success=100%
- **single**: latency=1614ms, tokens=432, success=100%
- **two_agent**: latency=1171ms, tokens=503, success=100%

**Handoff token overhead (2-agent vs single)**: 61 tokens

**Latency ratio (two_agent / recursive_mas)**: 0.16x
