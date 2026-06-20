---
type: entity
title: NousResearch/hermes-agent
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# NousResearch/hermes-agent

Type: ORGANIZATION

## From [drive-research-hermes-agent-prompt-builder-analysis](/entities/drive-research-hermes-agent-prompt-builder-analysis.md) (2026-06-08)
- The GitHub repository for the Hermes Agent.
- Contains various files and documentation related to the project.

## From [drive-research-hermes-session-storage-migration-analysis](/entities/drive-research-hermes-session-storage-migration-analysis.md) (2026-06-08)
- Sends Tool-Progress-Indicators to the chat.
- Has a configuration `cleanup_progress: true` to delete progress messages.
- Architecture requires implementation of an asynchronous delete() method on DiscordAdapter.
- Uses SQLite database for metadata, routing telemetry, and session lineage.
- Content memory was based on flat text files.
- Framework is being evaluated against other approaches.
- Architecture has evolved from JSONL files to a relational SQLite structure (state.db).
- Positioned as a robust, cost-effective, and learnable agent framework.
- Developed by Nous Research.
- Features a closed learning loop, persistent procedural memory, and autonomous skill generation.
- Evolved from stateless models to stateful frameworks capable of complex, multi-stage tasks.
- Uses a hybrid approach in data storage, combining historical file formats with modern relational database structures.
- Integrates SQLite for metadata, token counts, billing information, and full-text searches.
- Supports session segmentation for long conversations to prevent context drift.
- Employs a three-tier storage architecture (Hot, Warm, Cold).
- Integrates Segment Anything Model (SAM) and SAM 2.
- Has extensive computer-use capabilities, primarily for macOS environments.
- Requires strict sandbox environments due to extensive access rights.
- Architecture is being compared to the Gobii-Framework.
- Aims to equip autonomous agents with tools for browser control and long-term memory.
- Uses SQLite database for global metadata and session management.
- Agent interacts indirectly with memory files.
- Database persistence uses continuous writing in WAL mode.
- Uses strict prompting for hallucination prevention.
- Adapting Gobii's paradigm could lead to new use cases.
- Evolution marks a milestone in persistent, autonomous AI systems.
- Core architecture is SQLite.
- Interfaces to external gateways and TUI environments need stricter regulation.
- GitHub repository for the Hermes Agent.
- Contains code and documentation for the Hermes Agent.
