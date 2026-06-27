use serde::{Deserialize, Serialize};

use super::common::CacheControl;

// ── Image ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageMediaType {
    #[serde(rename = "image/jpeg")]
    Jpeg,
    #[serde(rename = "image/png")]
    Png,
    #[serde(rename = "image/gif")]
    Gif,
    #[serde(rename = "image/webp")]
    Webp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    Base64 {
        media_type: ImageMediaType,
        data: String,
    },
    Url {
        url: String,
    },
    File {
        file_id: String,
    },
}

impl ImageSource {
    pub fn base64(media_type: ImageMediaType, data: impl Into<String>) -> Self {
        Self::Base64 { media_type, data: data.into() }
    }

    pub fn url(url: impl Into<String>) -> Self {
        Self::Url { url: url.into() }
    }

    pub fn file(file_id: impl Into<String>) -> Self {
        Self::File { file_id: file_id.into() }
    }
}

// ── Document ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentMediaType {
    #[serde(rename = "application/pdf")]
    Pdf,
    #[serde(rename = "text/plain")]
    PlainText,
    #[serde(rename = "text/html")]
    Html,
    #[serde(rename = "text/markdown")]
    Markdown,
    #[serde(rename = "application/vnd.openxmlformats-officedocument.wordprocessingml.document")]
    Docx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationsConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocumentSource {
    Base64 {
        media_type: DocumentMediaType,
        data: String,
    },
    Url {
        url: String,
    },
    Text {
        data: String,
    },
    Content {
        content: Vec<InputContentBlock>,
    },
    File {
        file_id: String,
    },
}

// ── Input content blocks (what you send to the API) ───────────────────────────

/// A content block that appears inside a `messages` array entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputContentBlock {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Image {
        source: ImageSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    Document {
        source: DocumentSource,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        context: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<CitationsConfig>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolResult {
        tool_use_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<ToolResultContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

impl InputContentBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into(), cache_control: None }
    }

    pub fn text_with_cache(text: impl Into<String>) -> Self {
        Self::Text { text: text.into(), cache_control: Some(CacheControl::ephemeral()) }
    }

    pub fn image(source: ImageSource) -> Self {
        Self::Image { source, cache_control: None }
    }

    pub fn document(source: DocumentSource) -> Self {
        Self::Document { source, title: None, context: None, citations: None, cache_control: None }
    }

    pub fn tool_result(tool_use_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(ToolResultContent::Text(content.into())),
            is_error: None,
            cache_control: None,
        }
    }

    pub fn tool_error(tool_use_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: Some(ToolResultContent::Text(error.into())),
            is_error: Some(true),
            cache_control: None,
        }
    }
}

/// Content of a tool_result block: either a plain string or rich blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Blocks(Vec<InputContentBlock>),
}

/// The `content` field of a MessageParam: either a plain string or a list of blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<InputContentBlock>),
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self { Self::Text(s.to_string()) }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self { Self::Text(s) }
}

impl From<Vec<InputContentBlock>> for MessageContent {
    fn from(v: Vec<InputContentBlock>) -> Self { Self::Blocks(v) }
}

/// The `system` field: a plain string or an array of cacheable text blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    Text(String),
    Blocks(Vec<SystemBlock>),
}

impl From<&str> for SystemPrompt {
    fn from(s: &str) -> Self { Self::Text(s.to_string()) }
}

impl From<String> for SystemPrompt {
    fn from(s: String) -> Self { Self::Text(s) }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SystemBlock {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
}

impl SystemBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into(), cache_control: None }
    }

    pub fn text_with_cache(text: impl Into<String>) -> Self {
        Self::Text { text: text.into(), cache_control: Some(CacheControl::ephemeral()) }
    }
}

// ── Output content blocks (what the API returns) ──────────────────────────────

/// A content block in the API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutputContentBlock {
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        citations: Option<Vec<Citation>>,
    },
    Thinking {
        thinking: String,
        signature: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ServerToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    WebSearchToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
    WebFetchToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
    CodeExecutionToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
    ContainerUpload {
        file_id: String,
    },
}

impl OutputContentBlock {
    /// Returns the text content if this is a `Text` block.
    pub fn as_text(&self) -> Option<&str> {
        if let Self::Text { text, .. } = self { Some(text) } else { None }
    }

    /// Returns the thinking content if this is a `Thinking` block.
    pub fn as_thinking(&self) -> Option<&str> {
        if let Self::Thinking { thinking, .. } = self { Some(thinking) } else { None }
    }
}

/// A citation pointing back to a source document or search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    #[serde(rename = "type")]
    pub citation_type: String,
    #[serde(flatten)]
    pub location: serde_json::Value,
}
