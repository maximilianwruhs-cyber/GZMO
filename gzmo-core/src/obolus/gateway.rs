//! Instrumented gateway wrapper — records Prime token usage to the Obolus ledger.

use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;

use crate::gateway::{
    LlmGateway, LlmResponse, ToolDeclaration, VllmConfig,
};
use crate::obolus::context::current_call_context;
use crate::obolus::ledger::{LedgerEntry, LedgerSource, ObolusLedger, TokenUsage};
use crate::types::Message;

/// True when this profile targets the local Prime port.
pub fn targets_prime(config: &VllmConfig, prime_port: u16) -> bool {
    config.base_url.contains(&format!(":{prime_port}"))
}

/// Wrap `inner` with instrumentation when analytics is enabled and URL hits Prime.
pub fn instrument_if_enabled(
    inner: Arc<dyn LlmGateway>,
    ledger: Option<Arc<ObolusLedger>>,
    prime_port: u16,
    profile: &VllmConfig,
    default_process: String,
    default_task_kind: Option<String>,
) -> Arc<dyn LlmGateway> {
    let Some(ledger) = ledger else {
        return inner;
    };
    if !targets_prime(profile, prime_port) {
        return inner;
    }
    Arc::new(InstrumentedGateway {
        inner,
        ledger,
        default_process,
        default_task_kind,
        model: profile.model.clone(),
    })
}

pub struct InstrumentedGateway {
    inner: Arc<dyn LlmGateway>,
    ledger: Arc<ObolusLedger>,
    default_process: String,
    default_task_kind: Option<String>,
    model: String,
}

impl InstrumentedGateway {
    fn record_call(&self, started: Instant, ok: bool, usage: Option<TokenUsage>) {
        let ctx = current_call_context();
        let process = ctx
            .as_ref()
            .map(|c| c.process.clone())
            .filter(|p| !p.is_empty())
            .unwrap_or_else(|| self.default_process.clone());
        let task_kind = ctx
            .as_ref()
            .and_then(|c| c.task_kind.clone())
            .or_else(|| self.default_task_kind.clone());
        let caller = ctx
            .as_ref()
            .map(|c| c.caller.clone())
            .filter(|c| !c.is_empty())
            .unwrap_or_else(|| "gateway".into());
        let (input_tokens, output_tokens, total_tokens) = match usage {
            Some(u) => (u.input_tokens, u.output_tokens, u.total_tokens),
            None => (0, 0, 0),
        };
        self.ledger.record(LedgerEntry {
            ts: Utc::now(),
            source: LedgerSource::Gateway,
            process,
            task_kind,
            caller,
            input_tokens,
            output_tokens,
            total_tokens,
            latency_ms: started.elapsed().as_millis() as u64,
            ok,
            model: Some(self.model.clone()),
            correlation_id: ctx.as_ref().and_then(|c| c.correlation_id.clone()),
            action_id: ctx.as_ref().and_then(|c| c.action_id.clone()),
            dedup_key: None,
        });
    }
}

#[async_trait]
impl LlmGateway for InstrumentedGateway {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
    ) -> Result<LlmResponse> {
        let started = Instant::now();
        match self.inner.complete(messages, tools).await {
            Ok(r) => {
                let usage = self.inner.take_last_usage();
                self.record_call(started, true, usage);
                Ok(r)
            }
            Err(e) => {
                self.record_call(started, false, None);
                Err(e)
            }
        }
    }

    async fn complete_streaming(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<LlmResponse> {
        let started = Instant::now();
        match self
            .inner
            .complete_streaming(messages, tools, on_chunk)
            .await
        {
            Ok(r) => {
                let usage = self.inner.take_last_usage();
                self.record_call(started, true, usage);
                Ok(r)
            }
            Err(e) => {
                self.record_call(started, false, None);
                Err(e)
            }
        }
    }

    async fn complete_structured(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
    ) -> Result<String> {
        let started = Instant::now();
        match self
            .inner
            .complete_structured(messages, schema_name, json_schema)
            .await
        {
            Ok(s) => {
                let usage = self.inner.take_last_usage();
                self.record_call(started, true, usage);
                Ok(s)
            }
            Err(e) => {
                self.record_call(started, false, None);
                Err(e)
            }
        }
    }

    async fn complete_structured_with_temp(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        temperature: Option<f32>,
    ) -> Result<String> {
        let started = Instant::now();
        match self
            .inner
            .complete_structured_with_temp(messages, schema_name, json_schema, temperature)
            .await
        {
            Ok(s) => {
                let usage = self.inner.take_last_usage();
                self.record_call(started, true, usage);
                Ok(s)
            }
            Err(e) => {
                self.record_call(started, false, None);
                Err(e)
            }
        }
    }

    async fn complete_structured_bounded(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let started = Instant::now();
        match self
            .inner
            .complete_structured_bounded(
                messages,
                schema_name,
                json_schema,
                temperature,
                max_tokens,
            )
            .await
        {
            Ok(s) => {
                let usage = self.inner.take_last_usage();
                self.record_call(started, true, usage);
                Ok(s)
            }
            Err(e) => {
                self.record_call(started, false, None);
                Err(e)
            }
        }
    }

    fn set_chaos_overrides(&self, temperature: f32, max_tokens: u32) {
        self.inner.set_chaos_overrides(temperature, max_tokens);
    }

    fn clear_chaos_overrides(&self) {
        self.inner.clear_chaos_overrides();
    }

    async fn complete_with_persona(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<LlmResponse> {
        let started = Instant::now();
        match self
            .inner
            .complete_with_persona(messages, tools, temperature, top_p)
            .await
        {
            Ok(r) => {
                let usage = self.inner.take_last_usage();
                self.record_call(started, true, usage);
                Ok(r)
            }
            Err(e) => {
                self.record_call(started, false, None);
                Err(e)
            }
        }
    }

    fn take_last_usage(&self) -> Option<TokenUsage> {
        self.inner.take_last_usage()
    }

    fn take_last_latency_ms(&self) -> Option<u64> {
        self.inner.take_last_latency_ms()
    }
}
