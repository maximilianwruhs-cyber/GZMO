---
type: entity
title: "arXiv Preprint Ecosystem"
created: "2026-06-20"
updated: "2026-06-20"
status: draft
sources: 1
tags:
  - research
  - arxiv
  - preprint
---

# arXiv Preprint Ecosystem

Multi-dimensional taxonomy of the arXiv preprint commons (thema_004).

## Core mechanics

- **Green Open-Access Self-Archiving** — authors upload pre/post-prints under open licenses
- **Immediate Unrefereed Dissemination** — time-stamped public distribution without journal peer review
- **Peer-Endorsement Curatorial Framework** — domain sponsors gate first-time submitters (2026 policy)
- **Automated syntactic screening** — plagiarism, spam, and format heuristics on ingest

## Data persistence

- Persistent IDs: arXiv ID + DOI with version history
- **OAI-PMH** at `oaipmh.arxiv.org` — metadata namespaces: `oai_dc`, `arXiv`, `arXivRaw`
- Bulk export mirror: `export.arxiv.org` (rate limit ~4 req/s)

## Interdisciplinary role

- LLM pre-training corpus (e.g. SlimPajama arXiv slice)
- Citation DAGs (SNAP cit-HepTh / cit-HepPh) for network science
- Overlay journals (Discrete Analysis, Quantum) decouple review from hosting

## GZMO links

- [Librarian Agent](/wiki/entities/librarian-agent.md) — epistemic retrieval
- [Open Knowledge Format](/wiki/entities/open-knowledge-format.md) — metadata bundles
- [arXiv Network Patterns](/wiki/entities/arxiv-network-patterns.md) — live skill design
