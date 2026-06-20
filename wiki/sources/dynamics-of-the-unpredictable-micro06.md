---
type: source
title: dynamics-of-the-unpredictable-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# dynamics-of-the-unpredictable-micro06

Ingested source summary (2026-06-09).

## Entities
- [rodio](/entities/rodio.md) (TOOL)
- [wgpu::Queue](/entities/wgpu-queue.md) (TOOL)
- [wgpu::Instance](/entities/wgpu-instance.md) (TOOL)
- [WebGPU API](/entities/webgpu-api.md) (SYSTEM)
- [Vertex Shader](/entities/vertex-shader.md) (CONCEPT)
- [nalgebra](/entities/nalgebra.md) (TOOL)
- [Render Pipeline](/entities/render-pipeline.md) (CONCEPT)
- [rapier](/entities/rapier.md) (TOOL)
- [glam](/entities/glam.md) (TOOL)
- [Asset Conditioning Pipeline (ACP)](/entities/asset-conditioning-pipeline-acp.md) (SYSTEM)
- [wgpu::Adapter](/entities/wgpu-adapter.md) (TOOL)
- [gilrs (Game Input Library for Rust)](/entities/gilrs-game-input-library-for-rust.md) (TOOL)
- [WebGL](/entities/webgl.md) (SYSTEM)
- [Fragment Shader](/entities/fragment-shader.md) (CONCEPT)
- [glamx](/entities/glamx.md) (TOOL)
- [rapier2d](/entities/rapier2d.md) (TOOL)
- [winit](/entities/winit.md) (TOOL)
- [leafwing-input-manager](/entities/leafwing-input-manager.md) (TOOL)
- [cpal](/entities/cpal.md) (TOOL)
- [egui](/entities/egui.md) (TOOL)
- [Lua](/entities/lua.md) (LANGUAGE)
- [wasmtime](/entities/wasmtime.md) (TOOL)
- [extism](/entities/extism.md) (TOOL)
- [Apple's Metal](/entities/apple-s-metal.md) (SYSTEM)
- [Scene Graph](/entities/scene-graph.md) (CONCEPT)
- [Child(EntityId)](/entities/child-entityid.md) (CONCEPT)
- [Asset Manager](/entities/asset-manager.md) (SYSTEM)
- [mlua](/entities/mlua.md) (TOOL)
- [OpenGL](/entities/opengl.md) (SYSTEM)
- [naga](/entities/naga.md) (TOOL)
- [DirectX 12](/entities/directx-12.md) (SYSTEM)
- [wgpu::Surface](/entities/wgpu-surface.md) (TOOL)
- [Handle Pattern](/entities/handle-pattern.md) (CONCEPT)
- [WebGPU Shading Language (WGSL)](/entities/webgpu-shading-language-wgsl.md) (LANGUAGE)
- [rapier3d](/entities/rapier3d.md) (TOOL)
- [Vulkan](/entities/vulkan.md) (SYSTEM)
- [WebAssembly (Wasm)](/entities/webassembly-wasm.md) (SYSTEM)
- [Rhai](/entities/rhai.md) (LANGUAGE)
- [Python 3](/entities/python-3.md) (LANGUAGE)
- [wgpu::Device](/entities/wgpu-device.md) (TOOL)
- [Entity Component System (ECS)](/entities/entity-component-system-ecs.md) (CONCEPT)
- [kira](/entities/kira.md) (TOOL)
- [Parent(EntityId)](/entities/parent-entityid.md) (CONCEPT)

## Relations
- winit → USES → wgpu::Instance
- wgpu::Instance → USES → wgpu::Adapter
- wgpu::Instance → USES → wgpu::Device
- wgpu::Instance → USES → wgpu::Queue
- wgpu::Surface → USES → winit
- Render Pipeline → USES → WebGPU Shading Language (WGSL)
- naga → USES → WebGPU Shading Language (WGSL)
- Render Pipeline → USES → Vertex Shader
- Render Pipeline → USES → Fragment Shader
- glamx → USES → glam
- glamx → USES → nalgebra
- rapier → USES → Entity Component System (ECS)
- rapier → PART_OF → rapier2d
- rapier → PART_OF → rapier3d
- Asset Manager → USES → Handle Pattern
- Asset Conditioning Pipeline (ACP) → RELATED_TO → Asset Manager
- gilrs (Game Input Library for Rust) → RELATED_TO → winit
- leafwing-input-manager → USES → Entity Component System (ECS)
- rodio → USES → cpal
- kira → RELATED_TO → cpal
- Rhai → USES → Rust
- mlua → USES → Lua
- wasmtime → USES → WebAssembly (Wasm)
- extism → USES → WebAssembly (Wasm)
- Parent(EntityId) → USES → Entity Component System (ECS)
- Child(EntityId) → USES → Entity Component System (ECS)
