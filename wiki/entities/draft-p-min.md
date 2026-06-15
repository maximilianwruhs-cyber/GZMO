---
type: entity
title: --draft-p-min
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# --draft-p-min

Type: CONCEPT

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Modern tuning relies heavily on Min-P for a superior balance of determinism and creativity.
- Replaces legacy Top-K and Top-P samplers.
- Algorithm scales dynamically based on the confidence of the top token.
- Aggressively culls low-probability tokens, preventing hallucination of statistically improbable words.
- Allows creative branching across a wider array of low-probability but highly plausible tokens when the model is uncertain.
- Optimal base configuration includes --min-p 0.05 --temp 0.8.
- Imperative to completely disable Top-K and Top-P by setting them to 0 and 1.0 respectively when using Min-P.
- The Min-P algorithm replaces legacy Top-K and Top-P samplers.
- Sets the minimum probabilistic confidence threshold.
- Optimal base configuration is --min-p 0.05.
- Imperative to set Top-K and Top-P to 0 and 1.0 respectively when using --min-p.
- Sets the minimum probabilistic confidence threshold the draft model must meet to suggest a token.
- Tuning this parameter is highly sensitive.
- Values like 0.6 frequently outperform lower bounds by aggressively culling weak predictions before wasting the target model's verification cycles.
- On CUDA architectures, extremely high default values (e.g., 0.9) have proven detrimental, requiring manual tuning (e.g., 0.4 to 0.6) for optimal throughput.
