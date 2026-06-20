---
type: source
title: obolus-vs-codium-extension-konzept-research-part1-micro04
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# obolus-vs-codium-extension-konzept-research-part1-micro04

Ingested source summary (2026-06-09).

## Entities
- [WebSocketDisconnect](/entities/websocketdisconnect.md) (CONCEPT)
- [ConnectionManager](/entities/connectionmanager.md) (SYSTEM)
- [WebSockets](/entities/websockets.md) (CONCEPT)
- [Starlette](/entities/starlette.md) (TOOL)
- [FastAPI](/entities/fastapi.md) (TOOL)
- [HTMLResponse](/entities/htmlresponse.md) (TOOL)
- [WebSocketException](/entities/websocketexception.md) (CONCEPT)

## Relations
- WebSockets → RELATED_TO → FastAPI
- FastAPI → USES → WebSockets
- FastAPI → USES → Starlette
- FastAPI → USES → HTMLResponse
- WebSocketException → RELATED_TO → WebSockets
- WebSocketDisconnect → RELATED_TO → WebSockets
- ConnectionManager → USES → WebSocket
