//! LLM gateway abstractions for communicating with local vLLM.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::config;
use crate::types::Message;



/// Configuration for an OpenAI-compatible LLM endpoint.
/// Works with local (llama.cpp, Ollama, LM Studio) and cloud (OpenAI, Groq, Together) endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VllmConfig {
    pub base_url: String,
    pub model: String,
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
    /// API key for authenticated endpoints (Bearer token). Empty = no auth.
    #[serde(default)]
    pub api_key: String,
}

impl Default for VllmConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8000/v1".to_string(),
            model: "qwen3.6-27b".to_string(),
            temperature: 0.7,
            top_p: 0.9,
            max_tokens: 8192,
            api_key: String::new(),
        }
    }
}

impl From<crate::config::EngineProfileConfig> for VllmConfig {
    fn from(p: crate::config::EngineProfileConfig) -> Self {
        Self {
            base_url: p.url,
            model: p.model,
            temperature: p.temperature,
            top_p: p.top_p,
            max_tokens: p.max_tokens,
            api_key: p.api_key,
        }
    }
}

/// A tool declaration sent alongside the prompt to vLLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDeclaration {
    pub r#type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// A tool call returned by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub function_name: String,
    pub arguments: serde_json::Value,
}

/// The response from a single LLM inference step.
#[derive(Debug, Clone)]
pub enum LlmResponse {
    /// The model returned a text completion.
    Text(String),
    /// The model requested one or more tool calls.
    ToolCalls(Vec<ToolCall>),
}

/// Core trait for LLM communication. Implementations handle the HTTP
/// details of streaming, retries, and token counting.
#[async_trait]
pub trait LlmGateway: Send + Sync {
    /// Send a conversation to the local vLLM endpoint and receive a response.
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
    ) -> Result<LlmResponse>;

    /// Stream tokens from the LLM. Each chunk is yielded as it arrives.
    async fn complete_streaming(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<LlmResponse>;

    /// Request a structured JSON response constrained by a JSON Schema.
    /// Uses `response_format: { type: "json_schema" }` to enforce valid JSON
    /// at the decoding layer via GBNF grammar compilation.
    /// Returns the raw JSON string on success.
    async fn complete_structured(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
    ) -> Result<String>;

    /// Like [`complete_structured`](Self::complete_structured) but with an
    /// optional per-call temperature override. Useful when a single task (e.g.
    /// fact-checking) needs near-deterministic decoding regardless of the
    /// engine's configured default. The default implementation ignores the
    /// override and delegates, so alternative gateways need not implement it.
    async fn complete_structured_with_temp(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        _temperature: Option<f32>,
    ) -> Result<String> {
        self.complete_structured(messages, schema_name, json_schema).await
    }

    /// Structured completion with optional temperature and output token cap (shallow jobs).
    async fn complete_structured_bounded(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        self.complete_structured_with_temp(messages, schema_name, json_schema, temperature)
            .await
    }

    /// Set chaos-driven overrides for temperature and max_tokens.
    fn set_chaos_overrides(&self, _temperature: f32, _max_tokens: u32) {}

    /// Disable chaos overrides — revert to config values.
    fn clear_chaos_overrides(&self) {}

    /// Unstructured completion with optional per-call temperature / top_p overrides.
    async fn complete_with_persona(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
        _temperature: Option<f32>,
        _top_p: Option<f32>,
    ) -> Result<LlmResponse> {
        self.complete(messages, tools).await
    }
}

// ── Concrete Implementation ─────────────────────────────────────────


#[cfg(test)]
use crate::types::Role;

/// A concrete gateway that speaks OpenAI-compatible chat completions
/// to a local llama-server / TurboQuant endpoint.
///
/// Supports native tool calling via the `--jinja` flag on llama-server.
/// Enforces sequential tool execution (`parallel_tool_calls: false`)
/// to avoid known llama.cpp parallel parsing bugs.
pub struct TurboQuantGateway {
    config: VllmConfig,
    http: HttpClient,
    /// Optional chaos-driven overrides. Set by the PulseLoop each tick.
    /// When present, these override config.temperature and config.max_tokens.
    chaos_temperature: std::sync::atomic::AtomicU32,
    chaos_max_tokens: std::sync::atomic::AtomicU32,
    chaos_active: std::sync::atomic::AtomicBool,
}

// ── OpenAI-compatible request types ──────────────────────────────────

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<&'a ToolDeclaration>,
    /// Force sequential tool execution — parallel has known bugs in llama.cpp
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
    /// Let the model decide whether to call tools
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    /// llama-server: disable Qwen thinking trace for JSON/tool paths
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_format: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<&'a str>,
    /// For tool result messages, the ID of the tool call this responds to
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<&'a str>,
    /// For assistant messages that made tool calls
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<&'a Vec<crate::types::MessageToolCall>>,
}

// ── OpenAI-compatible response types ────────────────────────────────

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: ResponseMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ResponseMessage {
    content: Option<String>,
    /// Qwen3.x thinking models may put JSON here when `content` is empty.
    #[serde(default)]
    reasoning_content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ResponseToolCall>,
}

#[derive(Deserialize, Debug)]
struct ResponseToolCall {
    id: String,
    function: ResponseFunction,
}

#[derive(Deserialize, Debug)]
struct ResponseFunction {
    name: String,
    /// Arguments can be either a JSON string (OpenAI spec) or a JSON object
    /// (when llama-server uses --tool-args-object). We handle both.
    arguments: serde_json::Value,
}

// ── Streaming SSE types ─────────────────────────────────────────────

/// Same as ChatRequest but with `stream: true`.
#[derive(Serialize)]
struct StreamChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<&'a ToolDeclaration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    parallel_tool_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a str>,
    stream: bool,
}

/// A single SSE chunk from the streaming response.
#[derive(Deserialize, Debug)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Deserialize, Debug)]
struct StreamChoice {
    delta: StreamDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct StreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<StreamToolCallDelta>>,
}

#[derive(Deserialize, Debug)]
struct StreamToolCallDelta {
    index: u32,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    function: Option<StreamFunctionDelta>,
}

#[derive(Deserialize, Debug)]
struct StreamFunctionDelta {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    arguments: Option<String>,
}

impl TurboQuantGateway {
    /// Create a new gateway with the given config.
    pub fn new(config: VllmConfig) -> Self {
        Self {
            config,
            http: HttpClient::new(),
            chaos_temperature: std::sync::atomic::AtomicU32::new(0),
            chaos_max_tokens: std::sync::atomic::AtomicU32::new(0),
            chaos_active: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Create a gateway with default Prime config (localhost:8000).
    pub fn default_local() -> Self {
        Self::new(VllmConfig::default())
    }

    /// Set chaos-driven overrides for temperature and max_tokens.
    /// Called by the REPL each time the PulseLoop broadcasts a new ChaosSnapshot.
    pub fn set_chaos_overrides(&self, temperature: f32, max_tokens: u32) {
        self.chaos_temperature.store(
            temperature.to_bits(),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.chaos_max_tokens.store(
            max_tokens,
            std::sync::atomic::Ordering::Relaxed,
        );
        self.chaos_active.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// Disable chaos overrides — revert to config values.
    pub fn clear_chaos_overrides(&self) {
        self.chaos_active.store(false, std::sync::atomic::Ordering::Relaxed);
    }



    /// Get the effective temperature (chaos override or config default).
    fn effective_temperature(&self) -> f32 {
        if self.chaos_active.load(std::sync::atomic::Ordering::Relaxed) {
            f32::from_bits(self.chaos_temperature.load(std::sync::atomic::Ordering::Relaxed))
        } else {
            self.config.temperature
        }
    }

    /// Get the effective max_tokens (chaos override or config default).
    fn effective_max_tokens(&self) -> u32 {
        if self.chaos_active.load(std::sync::atomic::Ordering::Relaxed) {
            let chaos_val = self.chaos_max_tokens.load(std::sync::atomic::Ordering::Relaxed);
            // Chaos caps generation length; profile max_tokens is the hard ceiling.
            chaos_val.clamp(256, self.config.max_tokens)
        } else {
            self.config.max_tokens
        }
    }

    /// Build the full endpoint URL for a given path.
    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// Build an authenticated POST request.
    /// Adds `Authorization: Bearer <key>` if an API key is configured.
    fn auth_post(&self, path: &str) -> reqwest::RequestBuilder {
        let builder = self.http.post(self.url(path));
        if self.config.api_key.is_empty() {
            builder
        } else {
            builder.bearer_auth(&self.config.api_key)
        }
    }

    /// Convert our Message type to the OpenAI chat message format.
    /// Properly handles tool_calls on assistant messages and tool_call_id on tool results.
    fn to_chat_messages(messages: &[Message]) -> Vec<ChatMessage<'_>> {
        messages
            .iter()
            .map(|m| {
                let content = if m.content.is_empty() && m.tool_calls.is_some() {
                    // OpenAI spec: assistant tool-call messages can omit content
                    None
                } else {
                    Some(m.content.as_str())
                };

                ChatMessage {
                    role: m.role.as_str(),
                    content,
                    tool_call_id: m.tool_call_id.as_deref(),
                    tool_calls: m.tool_calls.as_ref(),
                }
            })
            .collect()
    }

    /// Parse arguments from the response — handles both string and object forms.
    fn parse_arguments(args: serde_json::Value) -> serde_json::Value {
        match args {
            // Standard OpenAI spec: arguments is a JSON string
            serde_json::Value::String(s) => {
                serde_json::from_str(&s).unwrap_or(serde_json::Value::Object(Default::default()))
            }
            // --tool-args-object mode: arguments is already a JSON object
            obj @ serde_json::Value::Object(_) => obj,
            // Fallback
            other => other,
        }
    }
}

#[async_trait]
impl LlmGateway for TurboQuantGateway {
    async fn complete_with_persona(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
        temperature: Option<f32>,
        top_p: Option<f32>,
    ) -> Result<LlmResponse> {
        if temperature.is_none() && top_p.is_none() {
            return self.complete(messages, tools).await;
        }
        let mut cfg = self.config.clone();
        if let Some(t) = temperature {
            cfg.temperature = t;
        }
        if let Some(p) = top_p {
            cfg.top_p = p;
        }
        TurboQuantGateway::new(cfg).complete(messages, tools).await
    }

    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
    ) -> Result<LlmResponse> {
        let has_tools = !tools.is_empty();

        let (reasoning_format, chat_template_kwargs) = no_thinking_request_fields();
        let body = ChatRequest {
            model: &self.config.model,
            messages: Self::to_chat_messages(messages),
            temperature: self.effective_temperature(),
            top_p: self.config.top_p,
            max_tokens: self.effective_max_tokens(),
            tools: tools.iter().collect(),
            // Enforce sequential execution to avoid llama.cpp parallel bugs
            parallel_tool_calls: if has_tools { Some(false) } else { None },
            tool_choice: if has_tools { Some("auto") } else { None },
            reasoning_format: Some(reasoning_format),
            chat_template_kwargs: Some(chat_template_kwargs),
        };

        debug!(
            url = %self.url("chat/completions"),
            model = %self.config.model,
            tools = has_tools,
            "Sending completion request"
        );

        let resp = self
            .auth_post("chat/completions")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "TurboQuant returned HTTP {}: {}",
                status.as_u16(),
                error_body
            );
        }

        let chat_resp: ChatResponse = resp.json().await?;
        let choice = chat_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("TurboQuant returned empty choices array"))?;

        debug!(
            finish_reason = ?choice.finish_reason,
            has_tool_calls = !choice.message.tool_calls.is_empty(),
            has_content = choice.message.content.is_some(),
            "Got completion response"
        );

        // Check for tool calls — either via finish_reason or presence of tool_calls array
        let is_tool_call = choice
            .finish_reason
            .as_deref()
            .map(|r| r == "tool_calls" || r == "stop")
            .unwrap_or(false)
            && !choice.message.tool_calls.is_empty();

        if is_tool_call || !choice.message.tool_calls.is_empty() {
            let calls = choice
                .message
                .tool_calls
                .into_iter()
                .map(|tc| ToolCall {
                    id: tc.id,
                    function_name: tc.function.name,
                    arguments: Self::parse_arguments(tc.function.arguments),
                })
                .collect();
            Ok(LlmResponse::ToolCalls(calls))
        } else {
            Ok(LlmResponse::Text(assistant_visible_text(
                choice.message.content,
                choice.message.reasoning_content,
            )))
        }
    }

    async fn complete_streaming(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<LlmResponse> {
        use futures_util::StreamExt;
        use reqwest_eventsource::{Event, EventSource};

        let has_tools = !tools.is_empty();

        // Build the same request body but with stream: true
        let body = StreamChatRequest {
            model: &self.config.model,
            messages: Self::to_chat_messages(messages),
            temperature: self.effective_temperature(),
            top_p: self.config.top_p,
            max_tokens: self.effective_max_tokens(),
            tools: tools.iter().collect(),
            parallel_tool_calls: if has_tools { Some(false) } else { None },
            tool_choice: if has_tools { Some("auto") } else { None },
            stream: true,
        };

        debug!(
            url = %self.url("chat/completions"),
            model = %self.config.model,
            stream = true,
            "Sending streaming completion request"
        );

        let builder = self
            .auth_post("chat/completions")
            .json(&body);

        let mut es = EventSource::new(builder)?;

        // Accumulators
        let mut text_buf = String::new();
        // tool_calls: map from index -> (id, name, arguments_json)
        let mut tool_map: std::collections::BTreeMap<u32, (String, String, String)> =
            std::collections::BTreeMap::new();
        let mut finish_reason: Option<String> = None;

        while let Some(event) = es.next().await {
            match event {
                Ok(Event::Open) => {
                    debug!("SSE stream opened");
                }
                Ok(Event::Message(msg)) => {
                    if msg.data == "[DONE]" {
                        break;
                    }

                    let chunk: StreamChunk = match serde_json::from_str(&msg.data) {
                        Ok(c) => c,
                        Err(e) => {
                            debug!(error = %e, data = %msg.data, "Failed to parse SSE chunk");
                            continue;
                        }
                    };

                    for choice in chunk.choices {
                        // Capture finish_reason
                        if let Some(fr) = choice.finish_reason {
                            finish_reason = Some(fr);
                        }

                        // Stream text content immediately
                        if let Some(content) = choice.delta.content {
                            text_buf.push_str(&content);
                            on_chunk(content);
                        }

                        // Buffer tool call deltas silently
                        if let Some(tc_deltas) = choice.delta.tool_calls {
                            for tc in tc_deltas {
                                let entry = tool_map
                                    .entry(tc.index)
                                    .or_insert_with(|| (String::new(), String::new(), String::new()));

                                if let Some(id) = tc.id {
                                    entry.0 = id;
                                }
                                if let Some(func) = tc.function {
                                    if let Some(name) = func.name {
                                        entry.1 = name;
                                    }
                                    if let Some(args) = func.arguments {
                                        entry.2.push_str(&args);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(reqwest_eventsource::Error::StreamEnded) => break,
                Err(e) => {
                    es.close();
                    anyhow::bail!("SSE stream error: {}", e);
                }
            }
        }
        es.close();

        // Determine response type
        let has_tool_calls = !tool_map.is_empty();
        let is_tool_finish = finish_reason.as_deref() == Some("tool_calls");

        if has_tool_calls || is_tool_finish {
            let calls = tool_map
                .into_values()
                .map(|(id, name, args_str)| {
                    let arguments = Self::parse_arguments(
                        serde_json::Value::String(args_str),
                    );
                    ToolCall { id, function_name: name, arguments }
                })
                .collect();
            Ok(LlmResponse::ToolCalls(calls))
        } else {
            Ok(LlmResponse::Text(text_buf))
        }
    }

    async fn complete_structured(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
    ) -> Result<String> {
        self.structured_request(
            messages,
            schema_name,
            &json_schema,
            self.effective_temperature(),
            None,
        )
        .await
    }

    async fn complete_structured_with_temp(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        temperature: Option<f32>,
    ) -> Result<String> {
        let temp = temperature.unwrap_or_else(|| self.effective_temperature());
        self.structured_request(messages, schema_name, &json_schema, temp, None)
            .await
    }

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
            .await
    }

    fn set_chaos_overrides(&self, temperature: f32, max_tokens: u32) {
        self.set_chaos_overrides(temperature, max_tokens);
    }

    fn clear_chaos_overrides(&self) {
        self.clear_chaos_overrides();
    }
}

impl TurboQuantGateway {
    /// Shared implementation for structured (JSON-schema constrained) requests.
    /// Accepts an explicit temperature so callers can pin determinism per-call.
    async fn structured_request(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: &serde_json::Value,
        temperature: f32,
        max_tokens_override: Option<u32>,
    ) -> Result<String> {
        let max_tokens = max_tokens_override
            .map(|cap| cap.min(self.config.max_tokens))
            .unwrap_or_else(|| self.effective_max_tokens());
        // Prime/llama-server: reasoning_format + json_schema triggers sampler init failure (HTTP 400).
        // Disabling thinking via chat_template_kwargs alone is sufficient for structured calls.
        let (_, chat_template_kwargs) = no_thinking_request_fields();
        let body = StructuredChatRequest {
            model: &self.config.model,
            messages: Self::to_chat_messages(messages),
            temperature,
            top_p: self.config.top_p,
            max_tokens,
            response_format: ResponseFormatSpec {
                r#type: "json_schema",
                json_schema: JsonSchemaSpec {
                    name: schema_name,
                    strict: true,
                    schema: json_schema,
                },
            },
            reasoning_format: None,
            chat_template_kwargs: Some(chat_template_kwargs),
        };

        debug!(
            url = %self.url("chat/completions"),
            schema = %schema_name,
            temperature,
            "Sending structured completion request"
        );

        let resp = self
            .auth_post("chat/completions")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            let error_body = resp.text().await.unwrap_or_default();
            anyhow::bail!(
                "TurboQuant structured output returned HTTP {}: {}",
                status.as_u16(),
                error_body
            );
        }

        let chat_resp: ChatResponse = resp.json().await?;
        let choice = chat_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("TurboQuant returned empty choices array"))?;

        let body = extract_structured_json_body(
            choice.message.content.as_deref(),
            choice.message.reasoning_content.as_deref(),
        );
        if body.is_empty() {
            anyhow::bail!(
                "Structured completion returned empty JSON (finish_reason={:?})",
                choice.finish_reason
            );
        }
        Ok(body)
    }
}

/// Prefer `content`, then `reasoning_content`, then brace-balanced JSON extraction.
pub(crate) fn extract_structured_json_body(
    content: Option<&str>,
    reasoning_content: Option<&str>,
) -> String {
    for text in [content, reasoning_content] {
        if let Some(t) = text {
            let trimmed = t.trim();
            if json_payload_usable(trimmed) {
                return trimmed.to_string();
            }
            if let Some(extracted) = extract_balanced_json_object(trimmed) {
                return extracted;
            }
        }
    }
    content.unwrap_or_default().trim().to_string()
}

fn json_payload_usable(s: &str) -> bool {
    (s.starts_with('{') && s.ends_with('}')) || (s.starts_with('[') && s.ends_with(']'))
}

/// Extract the first `{…}` object from a longer thinking trace.
fn extract_balanced_json_object(text: &str) -> Option<String> {
    let start = text.find('{')?;
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    for (i, &b) in bytes.iter().enumerate().skip(start) {
        match b {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let slice = &text[start..=i];
                    if json_payload_usable(slice) {
                        return Some(slice.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse JSON from model output; repair common truncation from token limits.
pub fn parse_json_lenient<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T> {
    let trimmed = raw.trim();
    if let Ok(v) = serde_json::from_str(trimmed) {
        return Ok(v);
    }
    if let Some(extracted) = extract_balanced_json_object(trimmed) {
        if let Ok(v) = serde_json::from_str(&extracted) {
            return Ok(v);
        }
        let repaired = repair_truncated_json_object(&extracted);
        if let Ok(v) = serde_json::from_str(&repaired) {
            return Ok(v);
        }
    }
    let repaired = repair_truncated_json_object(trimmed);
    serde_json::from_str(&repaired).map_err(|e| {
        anyhow::anyhow!("JSON parse failed: {e}\nRaw: {trimmed}\nRepaired: {repaired}")
    })
}

/// Close an unterminated string and missing `}`/`]` braces (best-effort).
fn repair_truncated_json_object(s: &str) -> String {
    let mut out = s.trim().to_string();
    if out.is_empty() {
        return "{}".to_string();
    }
    if !out.starts_with('{') && !out.starts_with('[') {
        return out;
    }
    let in_string = out
        .chars()
        .fold((false, false), |(in_str, escape), c| {
            if escape {
                return (in_str, false);
            }
            if c == '\\' && in_str {
                return (true, true);
            }
            if c == '"' {
                return (!in_str, false);
            }
            (in_str, false)
        })
        .0;
    if in_string {
        out.push('"');
    }
    let open_brace = out.chars().filter(|&c| c == '{').count();
    let close_brace = out.chars().filter(|&c| c == '}').count();
    for _ in close_brace..open_brace {
        out.push('}');
    }
    let open_bracket = out.chars().filter(|&c| c == '[').count();
    let close_bracket = out.chars().filter(|&c| c == ']').count();
    for _ in close_bracket..open_bracket {
        out.push(']');
    }
    out
}

// ── Structured output request types ─────────────────────────────────

/// Chat request with `response_format` for grammar-constrained JSON output.
#[derive(Serialize)]
struct StructuredChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    temperature: f32,
    top_p: f32,
    max_tokens: u32,
    response_format: ResponseFormatSpec<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reasoning_format: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<serde_json::Value>,
}

/// Prefer normal assistant `content`, then Qwen `reasoning_content` when content is empty.
pub fn assistant_visible_text(content: Option<String>, reasoning_content: Option<String>) -> String {
    let c = content.unwrap_or_default();
    if !c.trim().is_empty() {
        return c;
    }
    reasoning_content.unwrap_or_default()
}

fn no_thinking_request_fields() -> (&'static str, serde_json::Value) {
    (
        "none",
        serde_json::json!({ "enable_thinking": false }),
    )
}

#[derive(Serialize)]
struct ResponseFormatSpec<'a> {
    r#type: &'a str,
    json_schema: JsonSchemaSpec<'a>,
}

#[derive(Serialize)]
struct JsonSchemaSpec<'a> {
    name: &'a str,
    strict: bool,
    schema: &'a serde_json::Value,
}

// ── Fallback Gateway ───────────────────────────────────────────────────

/// A gateway that tries an ordered list of backends, falling back to the next
/// on any error (connection refused, HTTP 4xx/5xx, timeout, empty output).
///
/// Used for cloud-first background routing: the primary is the cloud profile
/// (OpenRouter) and the fallback is the task's legacy local/librarian profile.
/// Failover is on transport/HTTP/gateway errors only — verifier rejection and
/// other task-logic decisions happen in the engine layer above the gateway and
/// are never seen here.
pub struct FallbackGateway {
    /// Ordered (profile label, gateway) pairs. First is primary.
    backends: Vec<(String, Arc<dyn LlmGateway>)>,
    /// Human-readable task label for logs (e.g. "distill_extract").
    task_label: String,
}

impl FallbackGateway {
    /// Build a fallback chain. Requires at least one backend.
    pub fn new(task_label: impl Into<String>, backends: Vec<(String, Arc<dyn LlmGateway>)>) -> Self {
        assert!(
            !backends.is_empty(),
            "FallbackGateway requires at least one backend"
        );
        Self {
            backends,
            task_label: task_label.into(),
        }
    }

    /// Label of the next backend after index `i`, for logging.
    fn next_label(&self, i: usize) -> &str {
        self.backends
            .get(i + 1)
            .map(|(l, _)| l.as_str())
            .unwrap_or("<none>")
    }
}

#[async_trait]
impl LlmGateway for FallbackGateway {
    async fn complete(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
    ) -> Result<LlmResponse> {
        let mut last_err = None;
        for (i, (label, gw)) in self.backends.iter().enumerate() {
            match gw.complete(messages, tools).await {
                Ok(r) => return Ok(r),
                Err(e) => {
                    warn!(
                        task = %self.task_label,
                        from = %label,
                        to = %self.next_label(i),
                        error = %e,
                        "llm_fallback (complete)"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("backends is non-empty"))
    }

    async fn complete_streaming(
        &self,
        messages: &[Message],
        tools: &[ToolDeclaration],
        on_chunk: Box<dyn Fn(String) + Send>,
    ) -> Result<LlmResponse> {
        // The callback can only be moved once; share it across attempts behind a
        // mutex so each backend gets a fresh forwarding closure. Streaming is not
        // on the cloud-first background path (chat is excluded), so contention is
        // not a concern in practice.
        let shared: Arc<Mutex<Box<dyn Fn(String) + Send>>> = Arc::new(Mutex::new(on_chunk));
        let mut last_err = None;
        for (i, (label, gw)) in self.backends.iter().enumerate() {
            let cb = Arc::clone(&shared);
            let forward: Box<dyn Fn(String) + Send> = Box::new(move |s: String| {
                if let Ok(f) = cb.lock() {
                    f(s);
                }
            });
            match gw.complete_streaming(messages, tools, forward).await {
                Ok(r) => return Ok(r),
                Err(e) => {
                    warn!(
                        task = %self.task_label,
                        from = %label,
                        to = %self.next_label(i),
                        error = %e,
                        "llm_fallback (streaming)"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("backends is non-empty"))
    }

    async fn complete_structured(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
    ) -> Result<String> {
        let mut last_err = None;
        for (i, (label, gw)) in self.backends.iter().enumerate() {
            match gw
                .complete_structured(messages, schema_name, json_schema.clone())
                .await
            {
                Ok(r) => return Ok(r),
                Err(e) => {
                    warn!(
                        task = %self.task_label,
                        from = %label,
                        to = %self.next_label(i),
                        error = %e,
                        "llm_fallback (structured)"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("backends is non-empty"))
    }

    async fn complete_structured_with_temp(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        temperature: Option<f32>,
    ) -> Result<String> {
        let mut last_err = None;
        for (i, (label, gw)) in self.backends.iter().enumerate() {
            match gw
                .complete_structured_with_temp(
                    messages,
                    schema_name,
                    json_schema.clone(),
                    temperature,
                )
                .await
            {
                Ok(r) => return Ok(r),
                Err(e) => {
                    warn!(
                        task = %self.task_label,
                        from = %label,
                        to = %self.next_label(i),
                        error = %e,
                        "llm_fallback (structured_with_temp)"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("backends is non-empty"))
    }

    async fn complete_structured_bounded(
        &self,
        messages: &[Message],
        schema_name: &str,
        json_schema: serde_json::Value,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<String> {
        let mut last_err = None;
        for (i, (label, gw)) in self.backends.iter().enumerate() {
            match gw
                .complete_structured_bounded(
                    messages,
                    schema_name,
                    json_schema.clone(),
                    temperature,
                    max_tokens,
                )
                .await
            {
                Ok(r) => return Ok(r),
                Err(e) => {
                    warn!(
                        task = %self.task_label,
                        from = %label,
                        to = %self.next_label(i),
                        error = %e,
                        "llm_fallback (structured_bounded)"
                    );
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.expect("backends is non-empty"))
    }

    fn set_chaos_overrides(&self, temperature: f32, max_tokens: u32) {
        for (_, gw) in &self.backends {
            gw.set_chaos_overrides(temperature, max_tokens);
        }
    }

    fn clear_chaos_overrides(&self) {
        for (_, gw) in &self.backends {
            gw.clear_chaos_overrides();
        }
    }
}

// ── Obolus Gateway Router ──────────────────────────────────────────────

/// Resolves `TaskKind` → `Arc<dyn LlmGateway>` using the static routing table.
///
/// Caches gateways per engine profile name to avoid redundant construction.
/// All gateways are created once at construction time and reused for every
/// task invocation.
pub struct GatewayRouter {
    /// Effective gateway per task kind (already wrapped for cloud-first when enabled).
    task_gateways: HashMap<config::TaskKind, Arc<dyn LlmGateway>>,
    /// Leaf gateways keyed by engine profile name (for `gateway_by_name`).
    leaves: HashMap<String, Arc<dyn LlmGateway>>,
    /// Default engine profile name (safety fallback).
    default_engine: String,
}

impl GatewayRouter {
    /// Create a new router from the loaded config.
    ///
    /// Builds one effective gateway per [`config::TaskKind`]. When
    /// `[routing] cloud_first_background = true` and the task is a background
    /// kind (everything except `Chat`), the effective gateway is a
    /// [`FallbackGateway`] that tries the cloud profile first and falls back to
    /// the task's legacy profile (the value from `[routing.mappings]`).
    pub fn new(config: &config::GzmoConfig) -> Self {
        let mut leaves: HashMap<String, Arc<dyn LlmGateway>> = HashMap::new();

        // Helper: get-or-build a leaf TurboQuant gateway for a profile name.
        let build_leaf = |name: &str, leaves: &mut HashMap<String, Arc<dyn LlmGateway>>| {
            if let Some(gw) = leaves.get(name) {
                return Arc::clone(gw);
            }
            let profile = Self::resolve_profile_for_name(config, name);
            let gw: Arc<dyn LlmGateway> =
                Arc::new(TurboQuantGateway::new(VllmConfig::from(profile)));
            leaves.insert(name.to_string(), Arc::clone(&gw));
            gw
        };

        let cloud_first =
            config.routing.cloud_first_background && config.engine.cloud.is_some();

        // Build the cloud leaf once, optionally wrapping OpenRouter -> Gemini
        // when `[engine.cloud] fallback_*` (or GZMO_GEMINI_KEY) is configured.
        let cloud_gw: Option<Arc<dyn LlmGateway>> = if cloud_first {
            let primary = build_leaf("cloud", &mut leaves);
            let composed = match config.engine.cloud_fallback() {
                Some(fb) => {
                    let gemini: Arc<dyn LlmGateway> =
                        Arc::new(TurboQuantGateway::new(VllmConfig::from(fb)));
                    Arc::new(FallbackGateway::new(
                        "cloud",
                        vec![
                            ("openrouter".to_string(), primary),
                            ("gemini".to_string(), gemini),
                        ],
                    )) as Arc<dyn LlmGateway>
                }
                None => primary,
            };
            Some(composed)
        } else {
            None
        };

        // Build the effective gateway for every task kind.
        let mut task_gateways: HashMap<config::TaskKind, Arc<dyn LlmGateway>> = HashMap::new();
        for &task in config::TaskKind::all() {
            let legacy_name = config.routing.resolve(task).to_string();
            let legacy_gw = build_leaf(&legacy_name, &mut leaves);

            let effective = match &cloud_gw {
                Some(cloud) if task.is_background() && legacy_name != "cloud" => {
                    Arc::new(FallbackGateway::new(
                        task.to_string(),
                        vec![
                            ("cloud".to_string(), Arc::clone(cloud)),
                            (legacy_name.clone(), legacy_gw),
                        ],
                    )) as Arc<dyn LlmGateway>
                }
                _ => legacy_gw,
            };
            task_gateways.insert(task, effective);
        }

        // Ensure the default engine leaf exists as a safety fallback.
        let default_engine = config.routing.default_engine.clone();
        build_leaf(&default_engine, &mut leaves);

        Self {
            task_gateways,
            leaves,
            default_engine,
        }
    }

    /// Resolve a specific engine profile by name.
    ///
    /// `local`/`prime` are pinned to `[engine.local]` (Prime) regardless of
    /// `active_mode`, so a cloud-first fallback to "local" never loops back to
    /// the cloud engine when the operator has `/mode cloud` active.
    fn resolve_profile_for_name(
        config: &config::GzmoConfig,
        name: &str,
    ) -> config::EngineProfileConfig {
        // Check inline profiles first
        if let Some(inline) = config.routing.get_profile(name) {
            return inline.clone();
        }

        // Fall back to standard engine sections
        match name {
            "local" | "prime" => {
                config.engine.active_engine_for_mode(config::EngineMode::Local)
            }
            "cloud" => {
                if let Some(ref cloud) = config.engine.cloud {
                    config::EngineProfileConfig {
                        provider: cloud.provider.clone(),
                        url: cloud.url.clone(),
                        model: cloud.model.clone(),
                        api_key: cloud.api_key.clone(),
                        temperature: cloud.temperature,
                        top_p: cloud.top_p,
                        max_tokens: cloud.max_tokens,
                    }
                } else {
                    config.engine.active_engine()
                }
            }
            "sovereign" => config
                .engine
                .sovereign
                .clone()
                .unwrap_or_else(|| config.engine.active_engine()),
            "librarian" => config.librarian.to_engine_profile(),
            _ => {
                tracing::warn!(
                    profile = name,
                    "Unknown routing profile — falling back to active engine"
                );
                config.engine.active_engine()
            }
        }
    }

    /// Resolve the gateway for a specific task kind.
    pub fn gateway(&self, task: config::TaskKind) -> &Arc<dyn LlmGateway> {
        self.task_gateways.get(&task).unwrap_or_else(|| {
            // Safety: the default engine leaf is always present
            self.leaves
                .get(&self.default_engine)
                .expect("default gateway must exist")
        })
    }

    /// Get a leaf gateway by engine profile name (for manual routing).
    pub fn gateway_by_name(&self, name: &str) -> Option<&Arc<dyn LlmGateway>> {
        self.leaves.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_json_from_reasoning_trace() {
        let reasoning = "Let me verify.\n{\"supported\":true,\"confidence\":0.9,\"evidence_anchor\":\"foo bar baz\",\"evidence_recent\":\"x y z twelve\"}";
        let out = extract_structured_json_body(None, Some(reasoning));
        assert!(out.contains("\"supported\":true"));
    }

    #[test]
    fn parse_json_lenient_repairs_truncated_string() {
        let broken = r#"{"internal_analysis":"short","anchor_label":"A","recent_label":"B","connection":"Link text that got cut off mid"#;
        let v: serde_json::Value = parse_json_lenient(broken).expect("repaired JSON");
        assert_eq!(v["anchor_label"], "A");
    }

    // ── FallbackGateway tests ──────────────────────────────────────────

    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Mock gateway that records call count and optionally always errors.
    struct MockGateway {
        label: &'static str,
        fail: bool,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl LlmGateway for MockGateway {
        async fn complete(
            &self,
            _messages: &[Message],
            _tools: &[ToolDeclaration],
        ) -> Result<LlmResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                anyhow::bail!("mock {} failed", self.label);
            }
            Ok(LlmResponse::Text(self.label.to_string()))
        }

        async fn complete_streaming(
            &self,
            _messages: &[Message],
            _tools: &[ToolDeclaration],
            on_chunk: Box<dyn Fn(String) + Send>,
        ) -> Result<LlmResponse> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                anyhow::bail!("mock {} failed", self.label);
            }
            on_chunk(self.label.to_string());
            Ok(LlmResponse::Text(self.label.to_string()))
        }

        async fn complete_structured(
            &self,
            _messages: &[Message],
            _schema_name: &str,
            _json_schema: serde_json::Value,
        ) -> Result<String> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                anyhow::bail!("mock {} failed", self.label);
            }
            Ok(self.label.to_string())
        }
    }

    fn mock(label: &'static str, fail: bool) -> (Arc<dyn LlmGateway>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        let gw: Arc<dyn LlmGateway> = Arc::new(MockGateway {
            label,
            fail,
            calls: Arc::clone(&calls),
        });
        (gw, calls)
    }

    #[tokio::test]
    async fn fallback_returns_primary_on_success() {
        let (a, ca) = mock("a", false);
        let (b, cb) = mock("b", false);
        let gw = FallbackGateway::new("t", vec![("a".into(), a), ("b".into(), b)]);
        match gw.complete(&[], &[]).await.unwrap() {
            LlmResponse::Text(t) => assert_eq!(t, "a"),
            _ => panic!("expected text"),
        }
        assert_eq!(ca.load(Ordering::SeqCst), 1);
        assert_eq!(cb.load(Ordering::SeqCst), 0, "fallback must not run when primary succeeds");
    }

    #[tokio::test]
    async fn fallback_skips_failed_primary() {
        let (a, ca) = mock("a", true);
        let (b, cb) = mock("b", false);
        let gw = FallbackGateway::new("t", vec![("a".into(), a), ("b".into(), b)]);
        match gw.complete(&[], &[]).await.unwrap() {
            LlmResponse::Text(t) => assert_eq!(t, "b"),
            _ => panic!("expected text"),
        }
        assert_eq!(ca.load(Ordering::SeqCst), 1);
        assert_eq!(cb.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn fallback_all_fail_returns_error() {
        let (a, ca) = mock("a", true);
        let (b, cb) = mock("b", true);
        let gw = FallbackGateway::new("t", vec![("a".into(), a), ("b".into(), b)]);
        assert!(gw.complete(&[], &[]).await.is_err());
        assert_eq!(ca.load(Ordering::SeqCst), 1);
        assert_eq!(cb.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn fallback_structured_bounded_delegates_through_chain() {
        // bounded -> with_temp -> structured (default delegation) must still fail over.
        let (a, ca) = mock("a", true);
        let (b, cb) = mock("b", false);
        let gw = FallbackGateway::new("t", vec![("a".into(), a), ("b".into(), b)]);
        let out = gw
            .complete_structured_bounded(&[], "s", serde_json::json!({}), Some(0.1), Some(64))
            .await
            .unwrap();
        assert_eq!(out, "b");
        assert_eq!(ca.load(Ordering::SeqCst), 1);
        assert_eq!(cb.load(Ordering::SeqCst), 1);
    }

    // ── Routing semantics tests ────────────────────────────────────────

    #[test]
    fn chat_is_not_background() {
        assert!(!config::TaskKind::Chat.is_background());
        assert!(!config::TaskKind::PedagogyInternal.is_background());
        assert!(config::TaskKind::DreamExtract.is_background());
        assert!(config::TaskKind::Daemon.is_background());
    }

    #[test]
    fn legacy_local_pinned_to_prime_under_cloud_mode() {
        let mut cfg = config::GzmoConfig::default();
        cfg.engine.local = Some(config::EngineProfileConfig {
            url: "http://prime:8000/v1".into(),
            ..Default::default()
        });
        cfg.engine.cloud = Some(config::CloudEngineConfig {
            provider: "openrouter".into(),
            url: "http://cloud/v1".into(),
            model: "nemotron".into(),
            api_key: "k".into(),
            temperature: 0.4,
            top_p: 0.95,
            max_tokens: 100,
            fallback_provider: None,
            fallback_url: None,
            fallback_model: None,
            fallback_api_key: None,
        });
        // Operator switched chat to cloud — legacy fallback "local" must still be Prime.
        cfg.engine.active_mode = config::EngineMode::Cloud;
        let prof = GatewayRouter::resolve_profile_for_name(&cfg, "local");
        assert_eq!(prof.url, "http://prime:8000/v1");
        let prof_prime = GatewayRouter::resolve_profile_for_name(&cfg, "prime");
        assert_eq!(prof_prime.url, "http://prime:8000/v1");
    }

    #[tokio::test]
    #[ignore = "requires live llama-server on VllmConfig URL"]
    async fn test_turboquant_live() {
        let gw = TurboQuantGateway::default_local();

        let messages = vec![
            Message {
                role: Role::System,
                content: "You are a sovereign AI. Respond extremely concisely.".to_string(),
                is_meta: false, tool_calls: None, tool_call_id: None,
            },
            Message {
                role: Role::User,
                content: "What is 2+2? Reply with the number only.".to_string(),
                is_meta: false, tool_calls: None, tool_call_id: None,
            },
        ];

        let result = gw.complete(&messages, &[]).await;
        match &result {
            Ok(LlmResponse::Text(t)) => println!("\n[TurboQuant] {}\n", t),
            Ok(LlmResponse::ToolCalls(tc)) => println!("\n[TurboQuant] Tool calls: {:?}\n", tc),
            Err(e) => println!("\n[TurboQuant Error] {:?}\n", e),
        }
        assert!(
            result.is_ok(),
            "TurboQuant completion failed: {:?}",
            result.err()
        );
    }
}
