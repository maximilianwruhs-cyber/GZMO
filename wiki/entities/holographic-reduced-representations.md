---
type: entity
title: "Holographic Reduced Representations (HRR)"
created: "2026-06-26"
updated: "2026-06-26"
status: draft
tags:
  - research
  - thema_009
  - anti-pattern
  - vector-symbolic
---

# Holographic Reduced Representations (HRR)

A Vector Symbolic Architecture (Plate, 2003) that binds symbols into fixed-width vectors via circular convolution and stores many associations via superposition. Approximately invertible and associative in theory.

## GZMO status: rejected

thema_009 (arXiv:2606.24948) evaluates HRR and its phase-only variant FHRR on FB15k-237 for zero-shot two-hop composition. Findings:

- Atomic single-hop MRR ~0.35 (competitive).
- Zero-shot two-hop composition at **chance** across all cleanup temperatures.
- Hop-1 intermediate recovered at MRR ~0.90, yet composition fails even with a verified-correct intermediate.
- Hop-2 facts retrieved at only 0.26–0.48× atomic baseline even as standalone queries.
- Lemma 4.1: FHRR softmax cleanup is not phase-equivariant (secondary, compounding failure).

**Rejected for GZMO** because: (1) the paper is a negative result for the target use case; (2) HRR capacity (~50 clean facts in 1024D) is far below GZMO's 22k+ honeypot points; (3) ARCH-DIR-001 zero-bloat prohibits net-new vector-symbolic machinery. Use [Verified Chain Recall](/entities/verified-chain-recall.md) instead.

## References

- [arXiv:2606.24948](https://arxiv.org/abs/2606.24948)
- [iamhero2709/holographic-memory](https://github.com/iamhero2709/holographic-memory)
- [shitijkarsolia/holomemory](https://github.com/shitijkarsolia/holomemory)
