---
type: entity
title: gzmo-daemon
created: 2026-06-08
updated: 2026-06-10
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---






# gzmo-daemon

Type: SYSTEM

## From [[obolus-vs-codium-extension-konzept-research-part2|obolus-vs-codium-extension-konzept-research-part2]] (2026-06-08)
- Chief of Staff
- The sovereign brain that evaluates logs, runs Obolus, and delegates to the cloud only when the absolute intelligence ceiling of local hardware is breached.
- Implement a deterministic "Triage Layer" in the GZMO Chief of Staff to route queries based on semantic complexity before hitting the RAG pipeline.
- AOS / GZMO: A system that evaluates tasks and natively "hot-swaps" AI models in and out of GPU VRAM based on real-time task complexity and Intel RAPL energy telemetry (Intelligence per Watt)
- local-first RAG daemon

## From [[gzmo-daemon-validation-audit-and-bun-migration-rep|gzmo-daemon-validation-audit-and-bun-migration-rep]] (2026-06-08)
- Daemon PID: 46902 (bun run index.ts — LÄUFT LIVE)
- Ollama: aktiv (PID 17601, 2 Runner geladen)
- Runs since 18h+ stable, tick=40350, no crashes.

## From [[refactoring-gzmo-daemon-for-native-bun-high-perfor|refactoring-gzmo-daemon-for-native-bun-high-perfor]] (2026-06-08)
- Being refactored for native Bun high-performance.
- Currently uses Node.js standard libraries.
- Will shift away from Node.js standard libraries.

## From [[the-gzmo-daemon-high-performance-bun-refactor|the-gzmo-daemon-high-performance-bun-refactor]] (2026-06-08)
- High-performance Bun refactor implemented.
- Runs alongside Ollama.
- Transitioned away from Node.js standard libraries.

## From [[drive-research-license-and-native-binding-analysis|drive-research-license-and-native-binding-analysis]] (2026-06-08)
- Prefix used for internal commandlets for data store autoconfiguration.
- Acronym heavily utilized within biological and genetic research software libraries.
- Refers to the Granzyme O (GZMO) gene found in camelid genomes.
- Genzyme Molecular Oncology.
- Commercial usage linked to pharmaceutical entities.
- Namespace saturation exists around this identifier.
- A complex, multi-modal system designed to execute critical infrastructure tasks.
- Operates across several distinct domains including internal commandlets, network routing management, and LLDP integrations.
- Relies on UDP pipelines for statistical data transport and maintains an HTTP server architecture.
- Resulting daemon artifact.
- Must interact with isolated subprocesses strictly through standard input/output (stdio) streams or isolated inter-process communication mechanisms.
- Must maintain absolute authority over its own systemd unit file deployment.
- Classifies packages attempting to circumvent systemd authority as Red.

## From [[gzmo-soul-merged-new-part2-micro06|gzmo-soul-merged-new-part2-micro06]] (2026-06-10)
- A local-first RAG daemon being improved for retrieval precision.
- Treated as a specialized Librarian Agent.
