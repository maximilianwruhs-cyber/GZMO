---
type: source
title: architectural-analysis-of-the-openclaw-ai-plugin-s
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# architectural-analysis-of-the-openclaw-ai-plugin-s

Ingested source summary (2026-06-08).

## Entities
- [ACP Registry](/entities/acp-registry.md) (CONCEPT)
- [openclaw plugins install subsystem](/entities/openclaw-plugins-install-subsystem.md) (SYSTEM)
- [Discretionary Access Control (DAC) locking](/entities/discretionary-access-control-dac-locking.md) (CONCEPT)
- [basic-memory](/entities/basic-memory.md) (PROJECT)
- [sqlite-vec](/entities/sqlite-vec.md) (TOOL)
- [NVIDIA NemoClaw Docker implementation](/entities/nvidia-nemoclaw-docker-implementation.md) (ORGANIZATION)
- [Telegram](/entities/telegram.md) (SYSTEM)
- [PluginHookLlmInputEvent](/entities/pluginhookllminputevent.md) (CONCEPT)
- [PluginHookLlmOutputResult](/entities/pluginhookllmoutputresult.md) (CONCEPT)
- [OpenClawPluginApi](/entities/openclawpluginapi.md) (CONCEPT)
- [WhatsApp Baileys](/entities/whatsapp-baileys.md) (SYSTEM)
- [Ollama](/entities/ollama.md) (SYSTEM)
- [TypeScript](/entities/typescript.md) (CONCEPT)
- [Claude Code](/entities/claude-code.md) (TOOL)
- [OpenClaw AI agent gateway](/entities/openclaw-ai-agent-gateway.md) (SYSTEM)
- [Codex](/entities/codex.md) (SYSTEM)
- [AcpRuntimeBackend](/entities/acpruntimebackend.md) (CONCEPT)
- [package.json](/entities/package-json.md) (CONCEPT)
- [ClawHub](/entities/clawhub.md) (SYSTEM)
- [Native OpenClaw Plugins](/entities/native-openclaw-plugins.md) (CONCEPT)
- [openclaw.plugin.json](/entities/openclaw-plugin-json.md) (CONCEPT)
- [Zalo](/entities/zalo.md) (SYSTEM)
- [Zod Validation Pipeline](/entities/zod-validation-pipeline.md) (CONCEPT)
- [Copilot](/entities/copilot.md) (TOOL)
- [Plugin SDK](/entities/plugin-sdk.md) (TOOL)
- [Slack](/entities/slack.md) (SYSTEM)
- [PluginHookLlmOutputEvent](/entities/pluginhookllmoutputevent.md) (CONCEPT)
- [jiti](/entities/jiti.md) (TOOL)
- [OpenClaw AI Plugin S](/entities/openclaw-ai-plugin-s.md) (PROJECT)
- [acp-standard initiative](/entities/acp-standard-initiative.md) (PROJECT)
- [Compatible Bundles](/entities/compatible-bundles.md) (CONCEPT)
- [PluginHookLlmInputResult](/entities/pluginhookllminputresult.md) (CONCEPT)
- [UI Integration](/entities/ui-integration.md) (CONCEPT)
- [Matrix](/entities/matrix.md) (SYSTEM)
- [Agent Client Protocol (ACP)](/entities/agent-client-protocol-acp.md) (CONCEPT)
- [index.ts](/entities/index-ts.md) (CONCEPT)
- [Podman](/entities/podman.md) (SYSTEM)
- [JavaScript](/entities/javascript.md) (CONCEPT)
- [Cursor](/entities/cursor.md) (SYSTEM)
- [AI agents](/entities/ai-agents.md) (CONCEPT)
- [acpx plugin](/entities/acpx-plugin.md) (PROJECT)
- [OpenTelemetry](/entities/opentelemetry.md) (TOOL)
- [createPluginRuntimeStore](/entities/createpluginruntimestore.md) (CONCEPT)
- [lancedb](/entities/lancedb.md) (PROJECT)
- [Discord](/entities/discord.md) (SYSTEM)
- [Node.js](/entities/node-js.md) (SYSTEM)
- [Microsoft Teams](/entities/microsoft-teams.md) (SYSTEM)

## Relations
- OpenClaw AI Plugin S → PART_OF → OpenClaw AI agent gateway
- OpenClaw AI agent gateway → USES → AI agents
- OpenClaw AI agent gateway → USES → OpenClawPluginApi
- OpenClaw AI agent gateway → USES → ClawHub
- OpenClaw AI agent gateway → USES → Native OpenClaw Plugins
- OpenClaw AI agent gateway → USES → Compatible Bundles
- Compatible Bundles → RELATED_TO → Codex
- Compatible Bundles → RELATED_TO → Claude Code
- Compatible Bundles → RELATED_TO → Cursor
- Native OpenClaw Plugins → USES → package.json
- Native OpenClaw Plugins → USES → openclaw.plugin.json
- Native OpenClaw Plugins → USES → index.ts
- OpenClaw AI agent gateway → USES → jiti
- OpenClaw AI agent gateway → USES → Node.js
- Native OpenClaw Plugins → USES → TypeScript
- Native OpenClaw Plugins → USES → JavaScript
- OpenClawPluginApi → USES → PluginHookLlmInputEvent
- OpenClawPluginApi → USES → PluginHookLlmInputResult
- OpenClawPluginApi → USES → PluginHookLlmOutputEvent
- OpenClawPluginApi → USES → PluginHookLlmOutputResult
- OpenClaw AI agent gateway → USES → OpenTelemetry
- OpenClaw AI agent gateway → USES → Matrix
- OpenClaw AI agent gateway → USES → Microsoft Teams
- OpenClaw AI agent gateway → USES → Zalo
- OpenClaw AI agent gateway → USES → WhatsApp Baileys
- OpenClaw AI agent gateway → USES → Discord
- OpenClaw AI agent gateway → USES → Ollama
- OpenClawPluginApi → USES → Zod Validation Pipeline
- acpx plugin → USES → Codex
- acpx plugin → USES → Claude Code
- acpx plugin → USES → Copilot
- acpx plugin → USES → AcpRuntimeBackend
- acpx plugin → USES → Agent Client Protocol (ACP)
- acpx plugin → RELATED_TO → ACP Registry
- acpx plugin → RELATED_TO → acp-standard initiative
- openclaw plugins install subsystem → USES → NVIDIA NemoClaw Docker implementation
- openclaw plugins install subsystem → USES → Podman
- NVIDIA NemoClaw Docker implementation → USES → Discretionary Access Control (DAC) locking
- Plugin SDK → USES → createPluginRuntimeStore
- basic-memory → USES → sqlite-vec
- basic-memory → USES → lancedb
- lancedb → USES → sqlite-vec
- Plugin SDK → USES → Telegram
- Plugin SDK → USES → Slack
- Plugin SDK → USES → Discord
