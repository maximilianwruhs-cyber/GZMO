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
- [[rodio|rodio]] (TOOL)
- [[wgpu-queue|wgpu::Queue]] (TOOL)
- [[wgpu-instance|wgpu::Instance]] (TOOL)
- [[webgpu-api|WebGPU API]] (SYSTEM)
- [[vertex-shader|Vertex Shader]] (CONCEPT)
- [[nalgebra|nalgebra]] (TOOL)
- [[render-pipeline|Render Pipeline]] (CONCEPT)
- [[rapier|rapier]] (TOOL)
- [[glam|glam]] (TOOL)
- [[asset-conditioning-pipeline-acp|Asset Conditioning Pipeline (ACP)]] (SYSTEM)
- [[wgpu-adapter|wgpu::Adapter]] (TOOL)
- [[gilrs-game-input-library-for-rust|gilrs (Game Input Library for Rust)]] (TOOL)
- [[webgl|WebGL]] (SYSTEM)
- [[fragment-shader|Fragment Shader]] (CONCEPT)
- [[glamx|glamx]] (TOOL)
- [[rapier2d|rapier2d]] (TOOL)
- [[winit|winit]] (TOOL)
- [[leafwing-input-manager|leafwing-input-manager]] (TOOL)
- [[cpal|cpal]] (TOOL)
- [[egui|egui]] (TOOL)
- [[lua|Lua]] (LANGUAGE)
- [[wasmtime|wasmtime]] (TOOL)
- [[extism|extism]] (TOOL)
- [[apple-s-metal|Apple's Metal]] (SYSTEM)
- [[scene-graph|Scene Graph]] (CONCEPT)
- [[child-entityid|Child(EntityId)]] (CONCEPT)
- [[asset-manager|Asset Manager]] (SYSTEM)
- [[mlua|mlua]] (TOOL)
- [[opengl|OpenGL]] (SYSTEM)
- [[naga|naga]] (TOOL)
- [[directx-12|DirectX 12]] (SYSTEM)
- [[wgpu-surface|wgpu::Surface]] (TOOL)
- [[handle-pattern|Handle Pattern]] (CONCEPT)
- [[webgpu-shading-language-wgsl|WebGPU Shading Language (WGSL)]] (LANGUAGE)
- [[rapier3d|rapier3d]] (TOOL)
- [[vulkan|Vulkan]] (SYSTEM)
- [[webassembly-wasm|WebAssembly (Wasm)]] (SYSTEM)
- [[rhai|Rhai]] (LANGUAGE)
- [[python-3|Python 3]] (LANGUAGE)
- [[wgpu-device|wgpu::Device]] (TOOL)
- [[entity-component-system-ecs|Entity Component System (ECS)]] (CONCEPT)
- [[kira|kira]] (TOOL)
- [[parent-entityid|Parent(EntityId)]] (CONCEPT)

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
