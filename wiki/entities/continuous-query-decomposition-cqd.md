---
type: entity
title: "Continuous Query Decomposition (CQD)"
created: "2026-06-26"
updated: "2026-06-26"
status: draft
tags:
  - research
  - thema_009
  - knowledge-graph
  - compositional
---

# Continuous Query Decomposition (CQD)

Arakelyan et al., ICLR 2021 (Outstanding Paper). Answers complex first-order queries on incomplete KGs by **decomposing into atomic sub-queries**, scoring each with a pretrained neural link predictor, and aggregating via t-norms/t-conorms. Two solvers: CQD-CO (gradient) and CQD-Beam (combinatorial, with explain logs).

## GZMO relevance

CQD is the **positive architectural analogue** for [Verified Chain Recall](/entities/verified-chain-recall.md): decompose a two-hop query into atomic honeypot lookups, verify intermediates against Neo4j, aggregate. Unlike holographic binding, CQD-style decomposition works because each atom is an independent retrieval, not a superposed interference. VCR's `A via REL1 mid via REL2 B` hints mirror CQD-Beam explain logs.

## Reference

- [Complex Query Answering with Neural Link Predictors](https://openreview.net/forum?id=Mos9F9kDwkz)
- [uclnlp/cqd](https://github.com/uclnlp/cqd)
