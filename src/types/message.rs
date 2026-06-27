use serde::{Deserialize, Serialize};

use super::{
    common::{CacheControl, Metadata, Model, Role, StopReason, Usage},
    content::{
        ImageSource, InputContentBlock, MessageContent, OutputContentBlock, SystemPrompt,
    },
    tool::{Tool, ToolChoice},
};

// ── Thinking ──────────────────────────────────────────────────────────────────

/// Extended thinking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingConfig {
    Enabled {
        /// Minimum 1024; must be less than `max_tokens`.
        budget_tokens: u32,
    },
    Disabled,
    Adaptive {
        #[serde(skip_serializing_if = "Option::is_none")]
        budget_tokens: Option<u32>,
    },
}

impl ThinkingConfig {
    pub fn enabled(budget_tokens: u32) -> Self {
        Self::Enabled { budget_tokens }
    }

    pub fn adaptive() -> Self {
        Self::Adaptive { budget_tokens: None }
    }

    pub fn adaptive_with_budget(budget_tokens: u32) -> Self {
        Self::Adaptive { budget_tokens: Some(budget_tokens) }
    }
}

// ── Structured output ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputEffort {
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

/// Request structured (JSON schema) output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputConfig {
    JsonSchema {
        schema: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        effort: Option<OutputEffort>,
    },
}

// ── Request ───────────────────────────────────────────────────────────────────

/// A single turn in the conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageParam {
    pub role: Role,
    pub content: MessageContent,
}

impl MessageParam {
    pub fn user(content: impl Into<MessageContent>) -> Self {
        Self { role: Role::User, content: content.into() }
    }

    pub fn assistant(content: impl Into<MessageContent>) -> Self {
        Self { role: Role::Assistant, content: content.into() }
    }
}

/// Full request body for `POST /v1/messages`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateMessageRequest {
    pub model: Model,
    pub max_tokens: u32,
    pub messages: Vec<MessageParam>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stop_sequences: Vec<String>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_config: Option<OutputConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,

    /// Set to `true` internally by the streaming path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) stream: Option<bool>,
}

impl CreateMessageRequest {
    pub fn builder() -> CreateMessageRequestBuilder {
        CreateMessageRequestBuilder::default()
    }
}

// ── Builder ───────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub struct CreateMessageRequestBuilder {
    model: Option<Model>,
    max_tokens: Option<u32>,
    messages: Vec<MessageParam>,
    system: Option<SystemPrompt>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    stop_sequences: Vec<String>,
    tools: Vec<Tool>,
    tool_choice: Option<ToolChoice>,
    thinking: Option<ThinkingConfig>,
    metadata: Option<Metadata>,
    output_config: Option<OutputConfig>,
    service_tier: Option<String>,
    container: Option<String>,
}

impl CreateMessageRequestBuilder {
    pub fn model(mut self, model: impl Into<Model>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn message(mut self, msg: MessageParam) -> Self {
        self.messages.push(msg);
        self
    }

    pub fn messages(mut self, msgs: Vec<MessageParam>) -> Self {
        self.messages.extend(msgs);
        self
    }

    /// Append a user text message.
    pub fn user(mut self, text: impl Into<String>) -> Self {
        self.messages.push(MessageParam::user(text.into()));
        self
    }

    /// Append a user message with text + image.
    pub fn user_with_image(
        mut self,
        text: impl Into<String>,
        image: ImageSource,
    ) -> Self {
        self.messages.push(MessageParam::user(vec![
            InputContentBlock::text(text.into()),
            InputContentBlock::image(image),
        ]));
        self
    }

    /// Append an assistant prefill message.
    pub fn assistant(mut self, text: impl Into<String>) -> Self {
        self.messages.push(MessageParam::assistant(text.into()));
        self
    }

    pub fn system(mut self, system: impl Into<SystemPrompt>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    pub fn top_p(mut self, p: f64) -> Self {
        self.top_p = Some(p);
        self
    }

    pub fn top_k(mut self, k: u32) -> Self {
        self.top_k = Some(k);
        self
    }

    pub fn stop_sequence(mut self, seq: impl Into<String>) -> Self {
        self.stop_sequences.push(seq.into());
        self
    }

    pub fn stop_sequences(mut self, seqs: Vec<String>) -> Self {
        self.stop_sequences.extend(seqs);
        self
    }

    pub fn tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools.extend(tools);
        self
    }

    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    pub fn thinking(mut self, config: ThinkingConfig) -> Self {
        self.thinking = Some(config);
        self
    }

    pub fn metadata(mut self, meta: Metadata) -> Self {
        self.metadata = Some(meta);
        self
    }

    pub fn output_config(mut self, config: OutputConfig) -> Self {
        self.output_config = Some(config);
        self
    }

    pub fn service_tier(mut self, tier: impl Into<String>) -> Self {
        self.service_tier = Some(tier.into());
        self
    }

    pub fn container(mut self, id: impl Into<String>) -> Self {
        self.container = Some(id.into());
        self
    }

    /// Also enables prompt caching with `cache_control` on the system prompt.
    pub fn system_with_cache(mut self, text: impl Into<String>) -> Self {
        use crate::types::content::SystemBlock;
        self.system = Some(SystemPrompt::Blocks(vec![SystemBlock::text_with_cache(text)]));
        self
    }

    pub fn cache_control(mut self, cc: CacheControl) -> Self {
        // Applies cache_control to the last message's last block (if applicable).
        if let Some(last) = self.messages.last_mut() {
            if let MessageContent::Blocks(ref mut blocks) = last.content {
                if let Some(last_block) = blocks.last_mut() {
                    match last_block {
                        InputContentBlock::Text { cache_control, .. } => {
                            *cache_control = Some(cc);
                        }
                        InputContentBlock::Image { cache_control, .. } => {
                            *cache_control = Some(cc);
                        }
                        InputContentBlock::Document { cache_control, .. } => {
                            *cache_control = Some(cc);
                        }
                        _ => {}
                    }
                }
            }
        }
        self
    }

    pub fn build(self) -> Result<CreateMessageRequest, crate::error::AnthropicError> {
        let model = self.model.ok_or_else(|| {
            crate::error::AnthropicError::ConfigError("model is required".to_string())
        })?;
        let max_tokens = self.max_tokens.ok_or_else(|| {
            crate::error::AnthropicError::ConfigError("max_tokens is required".to_string())
        })?;
        if self.messages.is_empty() {
            return Err(crate::error::AnthropicError::ConfigError(
                "at least one message is required".to_string(),
            ));
        }
        Ok(CreateMessageRequest {
            model,
            max_tokens,
            messages: self.messages,
            system: self.system,
            temperature: self.temperature,
            top_p: self.top_p,
            top_k: self.top_k,
            stop_sequences: self.stop_sequences,
            tools: self.tools,
            tool_choice: self.tool_choice,
            thinking: self.thinking,
            metadata: self.metadata,
            output_config: self.output_config,
            service_tier: self.service_tier,
            container: self.container,
            stream: None,
        })
    }
}

// ── Response ──────────────────────────────────────────────────────────────────

/// A complete message response from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: Role,
    pub model: String,
    pub content: Vec<OutputContentBlock>,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

impl Message {
    /// Concatenates all `Text` blocks into a single string.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|b| b.as_text())
            .collect::<Vec<_>>()
            .join("")
    }

    /// Returns the thinking content from the first `Thinking` block.
    pub fn thinking(&self) -> Option<&str> {
        self.content.iter().find_map(|b| b.as_thinking())
    }

    /// Returns all `ToolUse` blocks.
    pub fn tool_uses(&self) -> Vec<&OutputContentBlock> {
        self.content
            .iter()
            .filter(|b| matches!(b, OutputContentBlock::ToolUse { .. }))
            .collect()
    }

    /// Returns `true` if Claude wants to call at least one tool.
    pub fn wants_tool_use(&self) -> bool {
        self.stop_reason == Some(StopReason::ToolUse)
    }
}

// ── Token counting ────────────────────────────────────────────────────────────

/// Request body for `POST /v1/messages/count_tokens`.
#[derive(Debug, Clone, Serialize)]
pub struct CountTokensRequest {
    pub model: Model,
    pub messages: Vec<MessageParam>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<SystemPrompt>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
}

impl CountTokensRequest {
    pub fn new(model: impl Into<Model>, messages: Vec<MessageParam>) -> Self {
        Self {
            model: model.into(),
            messages,
            system: None,
            tools: Vec::new(),
            thinking: None,
            tool_choice: None,
        }
    }
}

/// Response from the token counting endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenCountResponse {
    pub input_tokens: u64,
}
