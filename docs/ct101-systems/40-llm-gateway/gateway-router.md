# Subsystem — Gateway Router

**Source:** `gzmo-core/src/gateway.rs`  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

Resolves `TaskKind` → `Arc<dyn LlmGateway>` using the static Obolus routing table. Caches leaf gateways per engine profile name. Wraps background tasks in `FallbackGateway` when `cloud_first_background` is enabled.

---

## 2. How it works

### LlmGateway trait

```100:161:gzmo-core/src/gateway.rs
#[async_trait]
pub trait LlmGateway: Send + Sync {
    async fn complete(&self, messages: &[Message], tools: &[ToolDeclaration]) -> Result<LlmResponse>;
    async fn complete_streaming(&self, messages: &[Message], tools: &[ToolDeclaration], on_chunk: Box<dyn Fn(String) + Send>) -> Result<LlmResponse>;
    async fn complete_structured(&self, messages: &[Message], schema_name: &str, json_schema: serde_json::Value) -> Result<String>;
    async fn complete_structured_with_temp(..., temperature: Option<f32>) -> Result<String>;
    async fn complete_structured_bounded(..., max_tokens: Option<u32>) -> Result<String>;
    fn set_chaos_overrides(&self, _temperature: f32, _max_tokens: u32) {}
    fn clear_chaos_overrides(&self) {}
}
```

### TurboQuantGateway (OpenAI-compatible HTTP)

```175:183:gzmo-core/src/gateway.rs
pub struct TurboQuantGateway {
    config: VllmConfig,
    http: HttpClient,
    chaos_temperature: AtomicU32,
    chaos_max_tokens: AtomicU32,
    chaos_active: AtomicBool,
}
```

Structured JSON uses `response_format: json_schema`; OpenRouter gets `reasoning.effort`:

```939:946:gzmo-core/src/gateway.rs
fn openrouter_reasoning_for_config(config: &VllmConfig) -> Option<OpenRouterReasoning> {
    if !is_openrouter_endpoint(config) { return None; }
    let effort = config.reasoning_effort.as_ref().filter(|s| !s.is_empty())?;
    Some(OpenRouterReasoning { effort: effort.clone() })
}
```

### FallbackGateway

```972:1024:gzmo-core/src/gateway.rs
pub struct FallbackGateway {
    backends: Vec<(String, Arc<dyn LlmGateway>)>,
    task_label: String,
}

// complete(): try each backend in order, warn on llm_fallback, return first Ok
```

### GatewayRouter construction

```1190:1268:gzmo-core/src/gateway.rs
impl GatewayRouter {
    pub fn new(config: &config::GzmoConfig) -> Self {
        let cloud_first = config.routing.cloud_first_background && config.engine.cloud.is_some();
        // cloud leaf may wrap OpenRouter → Gemini fallback
        for &task in config::TaskKind::all() {
            let legacy_name = config.routing.resolve(task).to_string();
            let legacy_gw = build_leaf(&legacy_name, &mut leaves);
            let effective = match &cloud_gw {
                Some(cloud) if task.is_background() && legacy_name != "cloud" => {
                    Arc::new(FallbackGateway::new(task.to_string(), vec![
                        ("cloud".to_string(), Arc::clone(cloud)),
                        (legacy_name.clone(), legacy_gw),
                    ]))
                }
                _ => legacy_gw,
            };
            task_gateways.insert(task, effective);
        }
    }

    pub fn gateway(&self, task: config::TaskKind) -> &Arc<dyn LlmGateway> {
        self.task_gateways.get(&task).unwrap_or_else(|| self.leaves.get(&self.default_engine).expect(...))
    }
}
```

### Local/prime pin (no cloud loop)

```1276:1289:gzmo-core/src/gateway.rs
    fn resolve_profile_for_name(config: &config::GzmoConfig, name: &str) -> EngineProfileConfig {
        match name {
            "local" | "prime" => {
                config.engine.active_engine_for_mode(config::EngineMode::Local)
            }
            "cloud" => { /* engine.cloud profile */ }
```

---

## 3. Interfaces

| Interface | Location |
|-----------|----------|
| Router ctor | `GatewayRouter::new(&GzmoConfig)` |
| Task lookup | `router.gateway(TaskKind::DreamExtract)` |
| Profile lookup | `router.gateway_by_name("librarian")` |
| HTTP paths | `{base_url}/chat/completions`, `{base_url}/models` |
| Chaos overrides | PulseLoop → `set_chaos_overrides` on all backends |

---

## 4. THINKING nodes

> **THINKING — gateway.rs:FallbackGateway streaming**
> - *Reviewed:* Mutex-wrapped callback for multi-backend streaming retry.
> - *Insight:* Chat streaming excluded from cloud-first path in practice.
> - *Risk / limitation:* Mutex contention if streaming fallback ever enabled.
> - *Enhancement:* Dedicated non-fallback streaming gateway for Chat. [CT101-safe]

> **THINKING — gateway.rs:GatewayRouter cache**
> - *Reviewed:* Leaf gateways built once at daemon startup.
> - *Insight:* Config hot-reload requires daemon restart to pick up routing changes.
> - *Risk / limitation:* Handoff script applies fused config but daemon keeps old router until restart.
> - *Enhancement:* SIGHUP reload of GatewayRouter (GZMO-next only). [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| CT101 live | cloud_first → GLM 5.2 primary, Prime/librarian fallback |
| Config handoff | Lab-only 04:00 UTC fused config apply |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Log which fallback backend succeeded per task | [CT101-safe] |
| 2 | Router hot-reload on config change | [GZMO-next] |
| 3 | Circuit breaker after N cloud failures | [CT101-safe] |
| 4 | Request timeout per TaskKind | [CT101-safe] |
