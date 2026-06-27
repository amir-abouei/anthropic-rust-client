use serde::Deserialize;

use super::{
    common::{StopReason, Usage},
    content::OutputContentBlock,
    message::Message,
};

/// All server-sent events emitted during a streaming response.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    MessageStart {
        message: MessageStartData,
    },
    ContentBlockStart {
        index: u32,
        content_block: ContentBlockStartData,
    },
    ContentBlockDelta {
        index: u32,
        delta: ContentBlockDelta,
    },
    ContentBlockStop {
        index: u32,
    },
    MessageDelta {
        delta: MessageDeltaData,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<StreamDeltaUsage>,
    },
    MessageStop,
    Ping,
    Error {
        error: StreamError,
    },
}

impl StreamEvent {
    /// Extract text from a `ContentBlockDelta` / `TextDelta`, or `None`.
    pub fn as_text_delta(&self) -> Option<&str> {
        if let Self::ContentBlockDelta {
            delta: ContentBlockDelta::TextDelta { text },
            ..
        } = self
        {
            Some(text)
        } else {
            None
        }
    }

    /// Extract thinking delta text, or `None`.
    pub fn as_thinking_delta(&self) -> Option<&str> {
        if let Self::ContentBlockDelta {
            delta: ContentBlockDelta::ThinkingDelta { thinking },
            ..
        } = self
        {
            Some(thinking)
        } else {
            None
        }
    }

    /// Returns `true` for the final `message_stop` event.
    pub fn is_message_stop(&self) -> bool {
        matches!(self, Self::MessageStop)
    }
}

/// Partial message sent with the `message_start` event.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageStartData {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub model: String,
    pub content: Vec<serde_json::Value>,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
}

/// The initial state of a content block sent with `content_block_start`.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockStartData {
    Text { text: String },
    Thinking { thinking: String },
    ToolUse { id: String, name: String },
}

/// An incremental update for a content block.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlockDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    InputJsonDelta { partial_json: String },
    SignatureDelta { signature: String },
}

/// Message-level update (stop reason) sent with `message_delta`.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaData {
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
}

/// Token usage update sent alongside `message_delta`.
#[derive(Debug, Clone, Deserialize)]
pub struct StreamDeltaUsage {
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

// ── Accumulator ───────────────────────────────────────────────────────────────

/// Accumulates [`StreamEvent`]s into a complete [`Message`].
///
/// Used internally by [`crate::api::messages::MessageStream::collect_message`].
#[derive(Debug, Default)]
pub struct MessageAccumulator {
    pub id: Option<String>,
    pub model: Option<String>,
    pub stop_reason: Option<StopReason>,
    pub stop_sequence: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub blocks: Vec<AccumulatedBlock>,
}

#[derive(Debug)]
pub enum AccumulatedBlock {
    Text { text: String },
    Thinking { thinking: String, signature: String },
    ToolUse { id: String, name: String, partial_json: String },
}

impl MessageAccumulator {
    pub fn apply(&mut self, event: &StreamEvent) {
        match event {
            StreamEvent::MessageStart { message } => {
                self.id = Some(message.id.clone());
                self.model = Some(message.model.clone());
                self.input_tokens = message.usage.input_tokens;
                self.output_tokens = message.usage.output_tokens;
            }
            StreamEvent::ContentBlockStart { index, content_block } => {
                let idx = *index as usize;
                while self.blocks.len() <= idx {
                    self.blocks.push(AccumulatedBlock::Text { text: String::new() });
                }
                match content_block {
                    ContentBlockStartData::Text { text } => {
                        self.blocks[idx] = AccumulatedBlock::Text { text: text.clone() };
                    }
                    ContentBlockStartData::Thinking { thinking } => {
                        self.blocks[idx] = AccumulatedBlock::Thinking {
                            thinking: thinking.clone(),
                            signature: String::new(),
                        };
                    }
                    ContentBlockStartData::ToolUse { id, name } => {
                        self.blocks[idx] = AccumulatedBlock::ToolUse {
                            id: id.clone(),
                            name: name.clone(),
                            partial_json: String::new(),
                        };
                    }
                }
            }
            StreamEvent::ContentBlockDelta { index, delta } => {
                if let Some(block) = self.blocks.get_mut(*index as usize) {
                    match (block, delta) {
                        (AccumulatedBlock::Text { text }, ContentBlockDelta::TextDelta { text: d }) => {
                            text.push_str(d);
                        }
                        (
                            AccumulatedBlock::Thinking { thinking, .. },
                            ContentBlockDelta::ThinkingDelta { thinking: d },
                        ) => {
                            thinking.push_str(d);
                        }
                        (
                            AccumulatedBlock::Thinking { signature, .. },
                            ContentBlockDelta::SignatureDelta { signature: d },
                        ) => {
                            signature.push_str(d);
                        }
                        (
                            AccumulatedBlock::ToolUse { partial_json, .. },
                            ContentBlockDelta::InputJsonDelta { partial_json: d },
                        ) => {
                            partial_json.push_str(d);
                        }
                        _ => {}
                    }
                }
            }
            StreamEvent::MessageDelta { delta, usage } => {
                self.stop_reason = delta.stop_reason.clone();
                self.stop_sequence = delta.stop_sequence.clone();
                if let Some(u) = usage {
                    self.output_tokens = u.output_tokens;
                }
            }
            _ => {}
        }
    }

    pub fn into_message(self) -> Message {
        use crate::types::common::{Role, Usage};

        let content = self
            .blocks
            .into_iter()
            .map(|b| match b {
                AccumulatedBlock::Text { text } => {
                    OutputContentBlock::Text { text, citations: None }
                }
                AccumulatedBlock::Thinking { thinking, signature } => {
                    OutputContentBlock::Thinking { thinking, signature }
                }
                AccumulatedBlock::ToolUse { id, name, partial_json } => {
                    let input = serde_json::from_str(&partial_json)
                        .unwrap_or(serde_json::Value::Null);
                    OutputContentBlock::ToolUse { id, name, input }
                }
            })
            .collect();

        Message {
            id: self.id.unwrap_or_default(),
            message_type: "message".to_string(),
            role: Role::Assistant,
            model: self.model.unwrap_or_default(),
            content,
            stop_reason: self.stop_reason,
            stop_sequence: self.stop_sequence,
            usage: Usage {
                input_tokens: self.input_tokens,
                output_tokens: self.output_tokens,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
                output_tokens_details: None,
                server_tool_use: None,
                service_tier: None,
            },
        }
    }
}
