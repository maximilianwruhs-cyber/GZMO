---
type: entity
title: clawhip
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# clawhip

Type: ROUTER

## From [[ai-research-part6-micro04|ai-research-part6-micro04]] (2026-06-09)
- Dedicated, asynchronous event and notification router
- Keeps monitoring data and webhooks outside the coding agent's working memory

## From [[ai-research-part6-micro05|ai-research-part6-micro05]] (2026-06-09)
- A decoupled event router that outsources communication from the active work context.
- Acts as a dedicated event router, decoupling communication from logical execution.
- Formats events for external sinks like Discord/Slack.

## From [[ai-research-part8-micro07|ai-research-part8-micro07]] (2026-06-09)
- It handles operational telemetry, formatting, and webhooks externally.
- It shards memory to keep the LLM context window pristine.
