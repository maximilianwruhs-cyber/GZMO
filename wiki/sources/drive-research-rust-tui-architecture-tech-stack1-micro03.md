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
- [[yazi|Yazi]] (SYSTEM)
- [[data-distribution-service-dds|Data Distribution Service (DDS)]] (CONCEPT)
- [[asynchronous-i-o|asynchronous I/O]] (CONCEPT)
- [[helix-term|helix-term]] (SYSTEM)
- [[rust|Rust]] (TOOL)
- [[ratatui|ratatui]] (TOOL)
- [[client-server-architecture|client/server architecture]] (CONCEPT)
- [[wasm-runtime|Wasm runtime]] (CONCEPT)
- [[functional-core|functional core]] (CONCEPT)
- [[ropey-crate|ropey crate]] (TOOL)
- [[ui-plugins|UI plugins]] (CONCEPT)
- [[mvc-model-view-controller|MVC (Model-View-Controller)]] (CONCEPT)
- [[wasmtime|wasmtime]] (TOOL)
- [[component-based-architecture|Component-Based Architecture]] (CONCEPT)
- [[tokio|tokio]] (TOOL)
- [[crossterm|crossterm]] (TOOL)
- [[message-bus|message bus]] (CONCEPT)

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
