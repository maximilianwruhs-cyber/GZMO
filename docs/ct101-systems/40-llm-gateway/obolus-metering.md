# Subsystem — Obolus Metering & Task Routing

**Source:** `gzmo-core/src/gateway.rs` (`GatewayRouter`, `FallbackGateway`), `gzmo-core/src/config.rs` (`TaskKind`, `RoutingConfig`)  
**Parent:** [SYSTEM.md](./SYSTEM.md)

---

## 1. Capability

**Obolus** (Economy Organ) implements cost-aware LLM routing: separate extract vs verify gateways per cognition pipeline, cloud-first background routing with automatic fallback to local/librarian profiles, and chat isolation on Prime quality path. Token/power ledger lives in `data/Obolus/` (JSONL) — gateway enforces routing; ledger records spend.

---

## 2. How it works

### Per-engine task binding at daemon startup

Each cognition subsystem gets distinct extract and verify gateways:

```102:110:gzmo-cli/src/daemon_cmd.rs
    let router = GatewayRouter::new(config);
    let dream_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamExtract));
    let dream_verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::DreamVerify));
    let spark_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::SparkHypothesis));
    let spark_verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::SparkVerify));
    let ingest_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::IngestExtract));
    let ingest_verify_gateway: Arc<dyn LlmGateway> = Arc::clone(router.gateway(TaskKind::IngestVerify));
```

Distill triple routing:

```169:179:gzmo-cli/src/daemon_cmd.rs
    let distill_engine = Arc::new(SessionDistillEngine::new(
        ...
        Arc::clone(router.gateway(TaskKind::DistillExtract)),
        Arc::clone(router.gateway(TaskKind::DistillVerify)),
        config.session_distill.librarian_summary
            .then(|| Arc::clone(router.gateway(TaskKind::DistillSummary)))
            .filter(|_| config.librarian.enabled),
        ...
    ));
```

### Cloud-first background economy

```1213:1256:gzmo-core/src/gateway.rs
        let cloud_first =
            config.routing.cloud_first_background && config.engine.cloud.is_some();
        // ...
            let effective = match &cloud_gw {
                Some(cloud) if task.is_background() && legacy_name != "cloud" => {
                    Arc::new(FallbackGateway::new(
                        task.to_string(),
                        vec![
                            ("cloud".to_string(), Arc::clone(cloud)),
                            (legacy_name.clone(), legacy_gw),
                        ],
                    ))
                }
                _ => legacy_gw,
            };
```

**Chat exclusion:** `TaskKind::Chat.is_background()` returns false — interactive sessions never auto-route cloud-first.

### Verify decoupling (quality gate)

Dream/Spark/Ingest/Distill engines use `new_with_verify(extract_gw, verify_gw, ...)`:
- **Extract** — can map to librarian (cheap, fast)
- **Verify** — can map to cloud GLM (accurate, higher cost)

Example CT101 mapping pattern:
| Task | Typical profile |
|------|-----------------|
| `dream_extract` | librarian |
| `dream_verify` | cloud |
| `spark_hypothesis` | librarian |
| `spark_verify` | cloud |
| `chat` | local/prime (always) |

### Fallback telemetry

```1012:1019:gzmo-core/src/gateway.rs
                    warn!(
                        task = %self.task_label,
                        from = %label,
                        to = %self.next_label(i),
                        error = %e,
                        "llm_fallback (complete)"
                    );
```

Failover triggers on transport/HTTP/gateway errors only — verifier rejection is handled in `KgPromoter` above the gateway.

### Structured output bounds (shallow jobs)

```694:704:gzmo-core/src/gateway.rs
    async fn complete_structured_bounded(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let temp = temperature.unwrap_or_else(|| self.effective_temperature());
        self.structured_request(messages, schema_name, &json_schema, temp, max_tokens)
```

Cognition pipelines pin low temperature on verify passes via `complete_structured_with_temp`.

---

## 3. Interfaces

| Interface | Purpose |
|-----------|---------|
| `[routing] cloud_first_background` | Enable cloud→legacy FallbackGateway |
| `[routing.mappings]` | Task kind → profile name |
| `data/Obolus/*.jsonl` | Token/power ledger (downstream of calls) |
| Synapse | Indirect — no per-token events yet |
| OpenRouter | Bearer auth from env; `reasoning.effort` for GLM 5.2 |

---

## 4. THINKING nodes

> **THINKING — GatewayRouter:task_gateways map**
> - *Reviewed:* One effective gateway pre-built per TaskKind at startup.
> - *Insight:* Obolus decision is O(1) hash lookup at call time — zero routing overhead.
> - *Risk / limitation:* Cannot dynamically shift load without restart.
> - *Enhancement:* Runtime routing table swap behind ArcSwap. [GZMO-next]

> **THINKING — extract/verify split**
> - *Reviewed:* Separate TaskKinds for extract vs verify in every pipeline.
> - *Insight:* Cost optimization — 80% of tokens on cheap extract, 20% on cloud verify.
> - *Risk / limitation:* Misconfigured mapping (both cloud) doubles API cost.
> - *Enhancement:* Config validation: verify profile must differ from extract or warn. [CT101-safe]

> **THINKING — FallbackGateway scope**
> - *Reviewed:* Failover on HTTP/transport errors only.
> - *Insight:* Empty JSON or verifier reject does NOT trigger fallback — prevents cost runaway.
> - *Risk / limitation:* Cloud returns 200 with garbage → no fallback, pipeline fails.
> - *Enhancement:* Schema-validation failure optional secondary fallback. [GZMO-next]

---

## 5. Advancement

| Lab / GZMO-next | Mapping |
|-----------------|---------|
| CT101 live | cloud_first + GLM 5.2 xhigh for verify-heavy background |
| Obolus ledger | Record model, task, tokens per structured call |
| GZMO-next | Dynamic routing by queue depth and budget cap |

---

## 6. Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Wire gateway calls to Obolus JSONL ledger | [CT101-safe] |
| 2 | Config lint: extract≠verify profile recommendation | [CT101-safe] |
| 3 | Daily cloud spend cap with auto local fallback | [CT101-safe] |
| 4 | TaskKind-level max_tokens defaults in routing | [GZMO-next] |
| 5 | Budget-aware scheduler (defer spark when over cap) | [GZMO-next] |
