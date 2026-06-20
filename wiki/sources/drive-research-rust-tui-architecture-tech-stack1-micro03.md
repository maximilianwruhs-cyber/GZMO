---
type: source
title: drive-research-rust-tui-architecture-tech-stack1-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-rust-tui-architecture-tech-stack1-micro03

Ingested source summary (2026-06-09).

## Entities
- [Yazi](/entities/yazi.md) (SYSTEM)
- [Data Distribution Service (DDS)](/entities/data-distribution-service-dds.md) (CONCEPT)
- [asynchronous I/O](/entities/asynchronous-i-o.md) (CONCEPT)
- [helix-term](/entities/helix-term.md) (SYSTEM)
- [Rust](/entities/rust.md) (TOOL)
- [ratatui](/entities/ratatui.md) (TOOL)
- [client/server architecture](/entities/client-server-architecture.md) (CONCEPT)
- [Wasm runtime](/entities/wasm-runtime.md) (CONCEPT)
- [functional core](/entities/functional-core.md) (CONCEPT)
- [ropey crate](/entities/ropey-crate.md) (TOOL)
- [UI plugins](/entities/ui-plugins.md) (CONCEPT)
- [MVC (Model-View-Controller)](/entities/mvc-model-view-controller.md) (CONCEPT)
- [wasmtime](/entities/wasmtime.md) (TOOL)
- [Component-Based Architecture](/entities/component-based-architecture.md) (CONCEPT)
- [tokio](/entities/tokio.md) (TOOL)
- [crossterm](/entities/crossterm.md) (TOOL)
- [message bus](/entities/message-bus.md) (CONCEPT)

## Relations
- wasmtime → USES → UI plugins
- UI plugins → PART_OF → Wasm runtime
- Yazi → USES → asynchronous I/O
- Yazi → USES → client/server architecture
- client/server architecture → USES → Data Distribution Service (DDS)
- ratatui → USES → crossterm
- Component-Based Architecture → USES → tokio
- tokio → USES → Component-Based Architecture
- message bus → RELATED_TO → UI
- Rust → RELATED_TO → wasmtime
- Rust → RELATED_TO → helix-term
- Rust → RELATED_TO → Yazi
- Rust → RELATED_TO → ratatui
- Rust → RELATED_TO → tokio
