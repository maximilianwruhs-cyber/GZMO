---
type: source
title: drive-research-hermes-system-untersuchung-und-erweiterung
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-hermes-system-untersuchung-und-erweiterung

Ingested source summary (2026-06-08).

## Entities
- [BoL Manifest Checkpoints](/entities/bol-manifest-checkpoints.md) (CONCEPT)
- [Hermes System](/entities/hermes-system.md) (SYSTEM)
- [Complemento Carta Porte](/entities/complemento-carta-porte.md) (CONCEPT)
- [SQLite](/entities/sqlite.md) (TOOL)
- [Checkpoint Manager](/entities/checkpoint-manager.md) (SYSTEM)
- [Unified Checkpointing](/entities/unified-checkpointing.md) (CONCEPT)
- [Checkpoint and Snapshot Infrastructure](/entities/checkpoint-and-snapshot-infrastructure.md) (SYSTEM)
- [ACE CAMIR](/entities/ace-camir.md) (SYSTEM)
- [Logistics Manifest](/entities/logistics-manifest.md) (CONCEPT)
- [/rollback](/entities/rollback.md) (TOOL)
- [Three-Tier Memory Architecture](/entities/three-tier-memory-architecture.md) (SYSTEM)
- [Cron-jobs](/entities/cron-jobs.md) (SYSTEM)
- [Hermes Agent](/entities/hermes-agent.md) (SYSTEM)
- [Bill of Lading (BoL) Manifest](/entities/bill-of-lading-bol-manifest.md) (CONCEPT)
- [Git](/entities/git.md) (TOOL)
- [SAP CRM](/entities/sap-crm.md) (SYSTEM)
- [Microsoft SQL Server](/entities/microsoft-sql-server.md) (SYSTEM)
- [AI Agent](/entities/ai-agent.md) (SYSTEM)

## Relations
- Hermes System → USES → Checkpoint and Snapshot Infrastructure
- Checkpoint and Snapshot Infrastructure → USES → BoL Manifest Checkpoints
- BoL Manifest Checkpoints → USES → /rollback
- BoL Manifest Checkpoints → PART_OF → Checkpoint Manager
- Checkpoint Manager → USES → Git
- Hermes System → RELATED_TO → Bill of Lading (BoL) Manifest
- Checkpoint and Snapshot Infrastructure → RELATED_TO → Bill of Lading (BoL) Manifest
- Hermes System → RELATED_TO → Unified Checkpointing
- Hermes System → USES → Three-Tier Memory Architecture
- Three-Tier Memory Architecture → USES → SQLite
- Checkpoint Manager → RELATED_TO → Bill of Lading (BoL) Manifest
- Unified Checkpointing → RELATED_TO → Git
- Unified Checkpointing → RELATED_TO → SQLite
- Microsoft SQL Server → RELATED_TO → Bill of Lading (BoL) Manifest
- SAP CRM → RELATED_TO → Bill of Lading (BoL) Manifest
- Hermes Agent → USES → SQLite
- Hermes Agent → USES → Git
- Hermes Agent → RELATED_TO → Unified Checkpointing
- Unified Checkpointing → PART_OF → BoL Manifest Checkpoints
- BoL Manifest Checkpoints → RELATED_TO → AI Agent
- AI Agent → USES → Logistics Manifest
- Complemento Carta Porte → RELATED_TO → Logistics Manifest
- ACE CAMIR → RELATED_TO → Logistics Manifest
