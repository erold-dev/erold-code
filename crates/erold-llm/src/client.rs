//! OpenAI API client
//!
//! HTTP client for the OpenAI Chat Completions API with streaming support.

use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

use crate::error::{LlmError, Result};
use crate::models::{ChatRequest, ChatResponse, ContentBlock, StopReason};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-4o";

/// LLM client for OpenAI API
#[derive(Debug, Clone)]
pub struct LlmClient {
    http: reqwest::Client,
    #[allow(dead_code)]
    api_key: String,
    model: String,
}

impl LlmClient {
    /// Create a new LLM client with default model
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_model(api_key, DEFAULT_MODEL)
    }

    /// Create a new LLM client with specific model
    ///
    /// # Errors
    /// Returns error if HTTP client creation fails
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Result<Self> {
        let api_key = api_key.into();

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {api_key}"))
                .map_err(|_| LlmError::InvalidResponse("Invalid API key format".into()))?,
        );

        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .default_headers(headers)
            .build()
            .map_err(LlmError::Http)?;

        Ok(Self {
            http,
            api_key,
            model: model.into(),
        })
    }

    /// Get the model name
    #[must_use]
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Set the model
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }

    /// Send a chat request and get a response
    ///
    /// # Errors
    /// Returns error on network failure, API error, or invalid response
    pub async fn chat(&self, mut request: ChatRequest) -> Result<ChatResponse> {
        // Ensure model is set
        if request.model.is_empty() {
            request.model = self.model.clone();
        }

        debug!(model = %request.model, messages = request.messages.len(), "Sending chat request");

        // Convert to OpenAI format
        let openai_request = request.to_openai_request();

        let response = self.http.post(OPENAI_API_URL).json(&openai_request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 429 {
                warn!("Rate limited by OpenAI API");
                return Err(LlmError::RateLimited { retry_after: 60 });
            }

            error!(status = %status, error = %error_text, "OpenAI API error");
            return Err(LlmError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let openai_response: OpenAIResponse = response.json().await?;
        let chat_response = ChatResponse::from_openai(openai_response);

        debug!(
            id = %chat_response.id,
            stop_reason = ?chat_response.stop_reason,
            input_tokens = chat_response.usage.input_tokens,
            output_tokens = chat_response.usage.output_tokens,
            "Received chat response"
        );

        Ok(chat_response)
    }

    /// Send a chat request with streaming response
    ///
    /// Returns a channel receiver for streaming events.
    ///
    /// # Errors
    /// Returns error on network failure or API error
    pub async fn chat_stream(
        &self,
        mut request: ChatRequest,
    ) -> Result<mpsc::Receiver<StreamUpdate>> {
        // Ensure model is set
        if request.model.is_empty() {
            request.model = self.model.clone();
        }

        debug!(model = %request.model, "Starting streaming chat request");

        let mut openai_request = request.to_openai_request();
        openai_request.stream = Some(true);

        let response = self.http.post(OPENAI_API_URL).json(&openai_request).send().await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();

            if status.as_u16() == 429 {
                return Err(LlmError::RateLimited { retry_after: 60 });
            }

            return Err(LlmError::Api {
                status: status.as_u16(),
                message: error_text,
            });
        }

        let (tx, rx) = mpsc::channel(100);

        // Spawn task to process SSE stream
        let stream = response.bytes_stream();
        tokio::spawn(async move {
            if let Err(e) = process_stream(stream, tx).await {
                error!(error = %e, "Error processing stream");
            }
        });

        Ok(rx)
    }
}

/// Updates from the streaming API
#[derive(Debug, Clone)]
pub enum StreamUpdate {
    /// Text delta
    Text(String),
    /// Tool use started
    ToolUseStart { id: String, name: String },
    /// Tool input delta (partial JSON)
    ToolInputDelta(String),
    /// Tool use completed with full input
    ToolUseComplete { id: String, name: String, input: serde_json::Value },
    /// Message completed
    Done { stop_reason: Option<StopReason> },
    /// Error occurred
    Error(String),
}

/// OpenAI API request format
#[derive(Debug, serde::Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, serde::Serialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<OpenAIContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum OpenAIContent {
    Text(String),
    Parts(Vec<OpenAIContentPart>),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum OpenAIContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ImageUrl {
    url: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAIFunctionCall,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, serde::Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunction,
}

#[derive(Debug, serde::Serialize)]
struct OpenAIFunction {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

/// OpenAI API response format
#[derive(Debug, serde::Deserialize)]
struct OpenAIResponse {
    id: String,
    choices: Vec<OpenAIChoice>,
    usage: OpenAIUsage,
    model: String,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAIResponseMessage {
    #[allow(dead_code)]
    role: String,
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

/// OpenAI streaming response
#[derive(Debug, serde::Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIStreamChoice>,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAIStreamChoice {
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAIStreamDelta {
    content: Option<String>,
    tool_calls: Option<Vec<OpenAIStreamToolCall>>,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAIStreamToolCall {
    index: usize,
    id: Option<String>,
    function: Option<OpenAIStreamFunction>,
}

#[derive(Debug, serde::Deserialize)]
struct OpenAIStreamFunction {
    name: Option<String>,
    arguments: Option<String>,
}

impl ChatRequest {
    /// Convert to OpenAI request format
    fn to_openai_request(&self) -> OpenAIRequest {
        let mut messages = Vec::new();

        // Add system message if present
        if let Some(ref system) = self.system {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: Some(OpenAIContent::Text(system.clone())),
                tool_calls: None,
                tool_call_id: None,
            });
        }

        // Convert messages
        for msg in &self.messages {
            let role = match msg.role {
                crate::models::Role::User => "user",
                crate::models::Role::Assistant => "assistant",
            };

            // Check if this is a tool result message
            let tool_results: Vec<_> = msg.content.iter().filter_map(|c| {
                if let ContentBlock::ToolResult { tool_use_id, content, .. } = c {
                    Some((tool_use_id.clone(), content.clone()))
                } else {
                    None
                }
            }).collect();

            if !tool_results.is_empty() {
                // Add tool results as separate messages
                for (tool_use_id, content) in tool_results {
                    messages.push(OpenAIMessage {
                        role: "tool".to_string(),
                        content: Some(OpenAIContent::Text(content)),
                        tool_calls: None,
                        tool_call_id: Some(tool_use_id),
                    });
                }
            } else {
                // Check for tool uses (assistant messages with tool calls)
                let tool_uses: Vec<_> = msg.content.iter().filter_map(|c| {
                    if let ContentBlock::ToolUse { id, name, input } = c {
                        Some(OpenAIToolCall {
                            id: id.clone(),
                            call_type: "function".to_string(),
                            function: OpenAIFunctionCall {
                                name: name.clone(),
                                arguments: serde_json::to_string(input).unwrap_or_default(),
                            },
                        })
                    } else {
                        None
                    }
                }).collect();

                // Get text content
                let text: String = msg.content.iter().filter_map(|c| {
                    if let ContentBlock::Text { text } = c {
                        Some(text.as_str())
                    } else {
                        None
                    }
                }).collect::<Vec<_>>().join("");

                if !tool_uses.is_empty() {
                    messages.push(OpenAIMessage {
                        role: role.to_string(),
                        content: if text.is_empty() { None } else { Some(OpenAIContent::Text(text)) },
                        tool_calls: Some(tool_uses),
                        tool_call_id: None,
                    });
                } else {
                    messages.push(OpenAIMessage {
                        role: role.to_string(),
                        content: Some(OpenAIContent::Text(text)),
                        tool_calls: None,
                        tool_call_id: None,
                    });
                }
            }
        }

        // Convert tools
        let tools = self.tools.as_ref().map(|tools| {
            tools.iter().map(|t| OpenAITool {
                tool_type: "function".to_string(),
                function: OpenAIFunction {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: t.input_schema.clone(),
                },
            }).collect()
        });

        OpenAIRequest {
            model: self.model.clone(),
            messages,
            max_tokens: Some(self.max_tokens),
            temperature: self.temperature,
            tools,
            stream: self.stream,
        }
    }
}

impl ChatResponse {
    /// Convert from OpenAI response format
    fn from_openai(response: OpenAIResponse) -> Self {
        let choice = response.choices.into_iter().next().unwrap_or(OpenAIChoice {
            message: OpenAIResponseMessage {
                role: "assistant".to_string(),
                content: None,
                tool_calls: None,
            },
            finish_reason: None,
        });

        let mut content = Vec::new();

        // Add text content if present
        if let Some(text) = choice.message.content {
            if !text.is_empty() {
                content.push(ContentBlock::Text { text });
            }
        }

        // Add tool calls if present
        if let Some(tool_calls) = choice.message.tool_calls {
            for tc in tool_calls {
                let input: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                    .unwrap_or(serde_json::Value::Null);
                content.push(ContentBlock::ToolUse {
                    id: tc.id,
                    name: tc.function.name,
                    input,
                });
            }
        }

        let stop_reason = match choice.finish_reason.as_deref() {
            Some("stop") => Some(StopReason::EndTurn),
            Some("length") => Some(StopReason::MaxTokens),
            Some("tool_calls") => Some(StopReason::ToolUse),
            _ => None,
        };

        ChatResponse {
            id: response.id,
            response_type: "message".to_string(),
            role: crate::models::Role::Assistant,
            content,
            model: response.model,
            stop_reason,
            usage: crate::models::Usage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
        }
    }
}

/// Process the SSE stream from OpenAI
async fn process_stream(
    mut stream: impl futures::Stream<Item = reqwest::Result<bytes::Bytes>> + Unpin,
    tx: mpsc::Sender<StreamUpdate>,
) -> Result<()> {
    let mut buffer = String::new();
    let mut current_tools: std::collections::HashMap<usize, (String, String, String)> = std::collections::HashMap::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        // Process complete lines
        while let Some(pos) = buffer.find('\n') {
            let line = buffer[..pos].trim().to_string();
            buffer = buffer[pos + 1..].to_string();

            if line.is_empty() {
                continue;
            }

            // Parse SSE event
            if let Some(data) = line.strip_prefix("data: ") {
                if data == "[DONE]" {
                    let _ = tx.send(StreamUpdate::Done { stop_reason: None }).await;
                    return Ok(());
                }

                match serde_json::from_str::<OpenAIStreamChunk>(data) {
                    Ok(chunk) => {
                        for choice in chunk.choices {
                            // Handle text content
                            if let Some(content) = choice.delta.content {
                                if !content.is_empty() {
                                    if tx.send(StreamUpdate::Text(content)).await.is_err() {
                                        return Ok(());
                                    }
                                }
                            }

                            // Handle tool calls
                            if let Some(tool_calls) = choice.delta.tool_calls {
                                for tc in tool_calls {
                                    if let Some(id) = tc.id {
                                        // New tool call starting
                                        let name = tc.function.as_ref()
                                            .and_then(|f| f.name.clone())
                                            .unwrap_or_default();
                                        current_tools.insert(tc.index, (id.clone(), name.clone(), String::new()));
                                        if tx.send(StreamUpdate::ToolUseStart { id, name }).await.is_err() {
                                            return Ok(());
                                        }
                                    }

                                    if let Some(ref func) = tc.function {
                                        if let Some(ref args) = func.arguments {
                                            if let Some((_, _, ref mut json_buf)) = current_tools.get_mut(&tc.index) {
                                                json_buf.push_str(args);
                                            }
                                            if tx.send(StreamUpdate::ToolInputDelta(args.clone())).await.is_err() {
                                                return Ok(());
                                            }
                                        }
                                    }
                                }
                            }

                            // Handle finish
                            if let Some(reason) = choice.finish_reason {
                                // Complete any pending tool calls
                                for (_, (id, name, json_buf)) in current_tools.drain() {
                                    let input = serde_json::from_str(&json_buf).unwrap_or(serde_json::Value::Null);
                                    let _ = tx.send(StreamUpdate::ToolUseComplete { id, name, input }).await;
                                }

                                let stop_reason = match reason.as_str() {
                                    "stop" => Some(StopReason::EndTurn),
                                    "length" => Some(StopReason::MaxTokens),
                                    "tool_calls" => Some(StopReason::ToolUse),
                                    _ => None,
                                };
                                let _ = tx.send(StreamUpdate::Done { stop_reason }).await;
                            }
                        }
                    }
                    Err(e) => {
                        debug!(error = %e, data = data, "Failed to parse stream event");
                    }
                }
            }
        }
    }

    Ok(())
}

/// Builder for constructing chat sessions
pub struct ChatSession {
    client: LlmClient,
    messages: Vec<crate::models::Message>,
    system: Option<String>,
    tools: Vec<crate::models::Tool>,
    max_tokens: u32,
    temperature: Option<f32>,
}

impl ChatSession {
    /// Create a new chat session
    #[must_use]
    pub fn new(client: LlmClient) -> Self {
        Self {
            client,
            messages: Vec::new(),
            system: None,
            tools: Vec::new(),
            max_tokens: 4096,
            temperature: None,
        }
    }

    /// Set system prompt
    #[must_use]
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Add a tool
    #[must_use]
    pub fn tool(mut self, tool: crate::models::Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add multiple tools
    #[must_use]
    pub fn tools(mut self, tools: impl IntoIterator<Item = crate::models::Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    /// Set max tokens
    #[must_use]
    pub fn max_tokens(mut self, max: u32) -> Self {
        self.max_tokens = max;
        self
    }

    /// Set temperature
    #[must_use]
    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Add a user message
    pub fn user(&mut self, text: impl Into<String>) {
        self.messages.push(crate::models::Message::user(text));
    }

    /// Add an assistant message
    pub fn assistant(&mut self, text: impl Into<String>) {
        self.messages.push(crate::models::Message::assistant(text));
    }

    /// Add a message
    pub fn message(&mut self, msg: crate::models::Message) {
        self.messages.push(msg);
    }

    /// Add tool results
    pub fn tool_results(&mut self, results: Vec<ContentBlock>) {
        self.messages.push(crate::models::Message::tool_results(results));
    }

    /// Get the messages
    #[must_use]
    pub fn messages(&self) -> &[crate::models::Message] {
        &self.messages
    }

    /// Send a message and get response
    ///
    /// # Errors
    /// Returns error on API failure
    pub async fn send(&mut self, text: impl Into<String>) -> Result<ChatResponse> {
        self.user(text);
        self.complete().await
    }

    /// Complete the conversation
    ///
    /// # Errors
    /// Returns error on API failure
    pub async fn complete(&mut self) -> Result<ChatResponse> {
        let mut request = ChatRequest::new(self.client.model.clone(), self.messages.clone());

        if let Some(ref system) = self.system {
            request = request.system(system.clone());
        }

        if !self.tools.is_empty() {
            request = request.tools(self.tools.clone());
        }

        request = request.max_tokens(self.max_tokens);

        if let Some(temp) = self.temperature {
            request = request.temperature(temp);
        }

        let response = self.client.chat(request).await?;

        // Add assistant response to history
        self.messages.push(response.to_message());

        Ok(response)
    }

    /// Complete with streaming
    ///
    /// # Errors
    /// Returns error on API failure
    pub async fn complete_stream(&mut self) -> Result<mpsc::Receiver<StreamUpdate>> {
        let mut request = ChatRequest::new(self.client.model.clone(), self.messages.clone())
            .stream();

        if let Some(ref system) = self.system {
            request = request.system(system.clone());
        }

        if !self.tools.is_empty() {
            request = request.tools(self.tools.clone());
        }

        request = request.max_tokens(self.max_tokens);

        if let Some(temp) = self.temperature {
            request = request.temperature(temp);
        }

        self.client.chat_stream(request).await
    }

    /// Clear message history
    pub fn clear(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = LlmClient::new("test-key").unwrap();
        assert_eq!(client.model(), DEFAULT_MODEL);
    }

    #[test]
    fn test_client_with_model() {
        let client = LlmClient::with_model("test-key", "gpt-4o-mini").unwrap();
        assert_eq!(client.model(), "gpt-4o-mini");
    }

    #[test]
    fn test_chat_session_builder() {
        let client = LlmClient::new("test-key").unwrap();
        let session = ChatSession::new(client)
            .system("You are helpful")
            .max_tokens(1000)
            .temperature(0.5);

        assert_eq!(session.system, Some("You are helpful".to_string()));
        assert_eq!(session.max_tokens, 1000);
        assert_eq!(session.temperature, Some(0.5));
    }
}
