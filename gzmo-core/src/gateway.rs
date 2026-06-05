//! LLM gateway abstractions for communicating with local vLLM.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use tracing::debug;

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
            base_url: "http://localhost:1234/v1".to_string(),
            model: "gemma-4-E4B-it-Q4_K_M.gguf".to_string(),
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

    /// Create a gateway with default TurboQuant config (localhost:1234).
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
            // Use whichever is larger: config or chaos (don't truncate below config)
            chaos_val.max(self.config.max_tokens / 2)
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

// ── Obolus Gateway Router ──────────────────────────────────────────────

/// Resolves `TaskKind` → `Arc<dyn LlmGateway>` using the static routing table.
///
/// Caches gateways per engine profile name to avoid redundant construction.
/// All gateways are created once at construction time and reused for every
/// task invocation.
pub struct GatewayRouter {
    /// Pre-built gateways keyed by engine profile name.
    gateways: HashMap<String, Arc<dyn LlmGateway>>,
    /// Routing configuration (task → profile name).
    routing: config::RoutingConfig,
}

impl GatewayRouter {
    /// Create a new router from the loaded config.
    /// Builds and caches all gateways referenced by the routing table.
    pub fn new(config: &config::GzmoConfig) -> Self {
        let mut gateways = HashMap::new();

        // Build gateways for every unique profile referenced in the routing table
        for mapping in config.routing.mappings.values() {
            if gateways.contains_key(mapping) {
                continue;
            }
            let profile = Self::resolve_profile_for_name(config, mapping);
            let vllm = VllmConfig::from(profile);
            gateways.insert(
                mapping.clone(),
                Arc::new(TurboQuantGateway::new(vllm)) as Arc<dyn LlmGateway>,
            );
        }

        // Always include the default (active engine) gateway
        if !gateways.contains_key(&config.routing.default_engine) {
            let active = config.engine.active_engine();
            gateways.insert(
                config.routing.default_engine.clone(),
                Arc::new(TurboQuantGateway::new(VllmConfig::from(active)))
                    as Arc<dyn LlmGateway>,
            );
        }

        Self {
            gateways,
            routing: config.routing.clone(),
        }
    }

    /// Resolve a specific engine profile by name.
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
            "local" => config.engine.active_engine(),
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
        let profile_name = self.routing.resolve(task);
        self.gateways
            .get(profile_name)
            .unwrap_or_else(|| {
                // Safety: the default engine gateway is always present
                self.gateways
                    .get(&self.routing.default_engine)
                    .expect("default gateway must exist")
            })
    }

    /// Get a gateway by engine profile name (for manual routing).
    pub fn gateway_by_name(&self, name: &str) -> Option<&Arc<dyn LlmGateway>> {
        self.gateways.get(name)
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
