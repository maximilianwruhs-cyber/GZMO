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
- [[bol-manifest-checkpoints|BoL Manifest Checkpoints]] (CONCEPT)
- [[hermes-system|Hermes System]] (SYSTEM)
- [[complemento-carta-porte|Complemento Carta Porte]] (CONCEPT)
- [[sqlite|SQLite]] (TOOL)
- [[checkpoint-manager|Checkpoint Manager]] (SYSTEM)
- [[unified-checkpointing|Unified Checkpointing]] (CONCEPT)
- [[checkpoint-and-snapshot-infrastructure|Checkpoint and Snapshot Infrastructure]] (SYSTEM)
- [[ace-camir|ACE CAMIR]] (SYSTEM)
- [[logistics-manifest|Logistics Manifest]] (CONCEPT)
- [[rollback|/rollback]] (TOOL)
- [[three-tier-memory-architecture|Three-Tier Memory Architecture]] (SYSTEM)
- [[cron-jobs|Cron-jobs]] (SYSTEM)
- [[hermes-agent|Hermes Agent]] (SYSTEM)
- [[bill-of-lading-bol-manifest|Bill of Lading (BoL) Manifest]] (CONCEPT)
- [[git|Git]] (TOOL)
- [[sap-crm|SAP CRM]] (SYSTEM)
- [[microsoft-sql-server|Microsoft SQL Server]] (SYSTEM)
- [[ai-agent|AI Agent]] (SYSTEM)

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
