use serde::{Deserialize, Serialize};

/// Conversation role.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    User,
    Assistant,
}

/// A Claude model identifier.
///
/// Use the associated constants for well-known models, or [`Model::new`] for
/// any model string (including aliases like `"claude-opus-4-8"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Model(pub String);

impl Model {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    // ── Claude 4.x ────────────────────────────────────────────────────────────
    pub fn claude_opus_4_8() -> Self { Self::new("claude-opus-4-8") }
    pub fn claude_opus_4_6() -> Self { Self::new("claude-opus-4-6") }
    pub fn claude_sonnet_4_6() -> Self { Self::new("claude-sonnet-4-6") }
    pub fn claude_haiku_4_5() -> Self { Self::new("claude-haiku-4-5-20251001") }

    // ── Claude Fable ──────────────────────────────────────────────────────────
    pub fn claude_fable_5() -> Self { Self::new("claude-fable-5") }
}

impl From<&str> for Model {
    fn from(s: &str) -> Self { Self::new(s) }
}

impl From<String> for Model {
    fn from(s: String) -> Self { Self(s) }
}

impl std::fmt::Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Reason Claude stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    StopSequence,
    ToolUse,
    PauseTurn,
    Refusal,
}

/// Token usage breakdown for a request/response pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<OutputTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_tool_use: Option<ServerToolUseUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_tokens: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerToolUseUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_requests: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_fetch_requests: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Standard,
    Priority,
    Batch,
}

/// Prompt caching control attached to individual content blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    pub kind: CacheControlType,
    /// Optional TTL: `"5m"` or `"1h"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<String>,
}

impl CacheControl {
    pub fn ephemeral() -> Self {
        Self { kind: CacheControlType::Ephemeral, ttl: None }
    }

    pub fn ephemeral_with_ttl(ttl: impl Into<String>) -> Self {
        Self { kind: CacheControlType::Ephemeral, ttl: Some(ttl.into()) }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheControlType {
    Ephemeral,
}

/// External user ID passed to Anthropic for abuse detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

impl Metadata {
    pub fn user_id(id: impl Into<String>) -> Self {
        Self { user_id: Some(id.into()) }
    }
}
