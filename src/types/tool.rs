use serde::{Deserialize, Deserializer, Serialize, Serializer, ser::SerializeMap};

use super::common::CacheControl;

// ── Tool choice ───────────────────────────────────────────────────────────────

/// Controls which tool(s) Claude may call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// Claude decides whether to use a tool (default).
    Auto,
    /// Claude must call at least one tool.
    Any,
    /// No tool calls allowed.
    None,
    /// Claude must call the named tool.
    Tool { name: String },
}

// ── Versioned server-tool type strings ───────────────────────────────────────

const CUSTOM_TYPE: &str = "custom";
const WEB_SEARCH_TYPE: &str = "web_search_20260209";
const WEB_FETCH_TYPE: &str = "web_fetch_20260309";
const CODE_EXECUTION_TYPE: &str = "code_execution_20260521";
const BASH_TYPE: &str = "bash_20250124";
const STR_REPLACE_EDITOR_TYPE: &str = "str_replace_editor_20250728";
const MEMORY_TYPE: &str = "memory_20250818";

// ── Tool definitions ──────────────────────────────────────────────────────────

/// A custom (user-defined) tool.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomTool {
    pub name: String,
    pub description: Option<String>,
    /// JSON Schema describing the tool's `input` object.
    pub input_schema: serde_json::Value,
    pub cache_control: Option<CacheControl>,
    /// Require strict schema validation.
    pub strict: Option<bool>,
    /// Defer loading until first invocation.
    pub defer_loading: Option<bool>,
}

impl CustomTool {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: Some(description.into()),
            input_schema,
            cache_control: None,
            strict: None,
            defer_loading: None,
        }
    }
}

impl Serialize for CustomTool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", CUSTOM_TYPE)?;
        m.serialize_entry("name", &self.name)?;
        if let Some(d) = &self.description { m.serialize_entry("description", d)?; }
        m.serialize_entry("input_schema", &self.input_schema)?;
        if let Some(cc) = &self.cache_control { m.serialize_entry("cache_control", cc)?; }
        if let Some(s) = self.strict { m.serialize_entry("strict", &s)?; }
        if let Some(d) = self.defer_loading { m.serialize_entry("defer_loading", &d)?; }
        m.end()
    }
}

/// Web search server tool.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebSearchTool {
    pub name: Option<String>,
    pub max_uses: Option<u32>,
    pub allowed_domains: Option<Vec<String>>,
    pub blocked_domains: Option<Vec<String>>,
    pub user_location: Option<UserLocation>,
    pub cache_control: Option<CacheControl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLocation {
    #[serde(rename = "type")]
    pub kind: String,
    pub country: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub timezone: Option<String>,
}

impl UserLocation {
    pub fn approximate(country: impl Into<String>) -> Self {
        Self {
            kind: "approximate".to_string(),
            country: Some(country.into()),
            city: None,
            region: None,
            timezone: None,
        }
    }
}

impl Serialize for WebSearchTool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", WEB_SEARCH_TYPE)?;
        if let Some(n) = &self.name { m.serialize_entry("name", n)?; }
        if let Some(mu) = self.max_uses { m.serialize_entry("max_uses", &mu)?; }
        if let Some(ad) = &self.allowed_domains { m.serialize_entry("allowed_domains", ad)?; }
        if let Some(bd) = &self.blocked_domains { m.serialize_entry("blocked_domains", bd)?; }
        if let Some(ul) = &self.user_location { m.serialize_entry("user_location", ul)?; }
        if let Some(cc) = &self.cache_control { m.serialize_entry("cache_control", cc)?; }
        m.end()
    }
}

/// Web fetch server tool.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct WebFetchTool {
    pub allowed_domains: Option<Vec<String>>,
    pub blocked_domains: Option<Vec<String>>,
    pub cache_control: Option<CacheControl>,
}

impl Serialize for WebFetchTool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", WEB_FETCH_TYPE)?;
        if let Some(ad) = &self.allowed_domains { m.serialize_entry("allowed_domains", ad)?; }
        if let Some(bd) = &self.blocked_domains { m.serialize_entry("blocked_domains", bd)?; }
        if let Some(cc) = &self.cache_control { m.serialize_entry("cache_control", cc)?; }
        m.end()
    }
}

/// Code execution server tool (Python sandbox).
#[derive(Debug, Clone, Default, Deserialize)]
pub struct CodeExecutionTool {
    pub cache_control: Option<CacheControl>,
}

impl Serialize for CodeExecutionTool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", CODE_EXECUTION_TYPE)?;
        if let Some(cc) = &self.cache_control { m.serialize_entry("cache_control", cc)?; }
        m.end()
    }
}

/// Bash tool for shell command execution.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BashTool {
    pub cache_control: Option<CacheControl>,
}

impl Serialize for BashTool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", BASH_TYPE)?;
        if let Some(cc) = &self.cache_control { m.serialize_entry("cache_control", cc)?; }
        m.end()
    }
}

/// Text editor tool.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct StrReplaceEditorTool {
    pub cache_control: Option<CacheControl>,
}

impl Serialize for StrReplaceEditorTool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", STR_REPLACE_EDITOR_TYPE)?;
        if let Some(cc) = &self.cache_control { m.serialize_entry("cache_control", cc)?; }
        m.end()
    }
}

/// Persistent memory tool.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MemoryTool {
    pub cache_control: Option<CacheControl>,
}

impl Serialize for MemoryTool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("type", MEMORY_TYPE)?;
        if let Some(cc) = &self.cache_control { m.serialize_entry("cache_control", cc)?; }
        m.end()
    }
}

/// All available tool types.
#[derive(Debug, Clone)]
pub enum Tool {
    Custom(CustomTool),
    WebSearch(WebSearchTool),
    WebFetch(WebFetchTool),
    CodeExecution(CodeExecutionTool),
    Bash(BashTool),
    StrReplaceEditor(StrReplaceEditorTool),
    Memory(MemoryTool),
}

impl Tool {
    /// Convenience: create a custom (user-defined) tool.
    pub fn custom(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self::Custom(CustomTool::new(name, description, input_schema))
    }

    /// Convenience: add web search capability.
    pub fn web_search() -> Self {
        Self::WebSearch(WebSearchTool::default())
    }

    /// Convenience: add web fetch capability.
    pub fn web_fetch() -> Self {
        Self::WebFetch(WebFetchTool::default())
    }

    /// Convenience: add Python code execution.
    pub fn code_execution() -> Self {
        Self::CodeExecution(CodeExecutionTool::default())
    }

    /// Convenience: add bash tool.
    pub fn bash() -> Self {
        Self::Bash(BashTool::default())
    }

    /// Convenience: add text editor tool.
    pub fn str_replace_editor() -> Self {
        Self::StrReplaceEditor(StrReplaceEditorTool::default())
    }

    /// Convenience: add persistent memory tool.
    pub fn memory() -> Self {
        Self::Memory(MemoryTool::default())
    }
}

impl Serialize for Tool {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Custom(t) => t.serialize(s),
            Self::WebSearch(t) => t.serialize(s),
            Self::WebFetch(t) => t.serialize(s),
            Self::CodeExecution(t) => t.serialize(s),
            Self::Bash(t) => t.serialize(s),
            Self::StrReplaceEditor(t) => t.serialize(s),
            Self::Memory(t) => t.serialize(s),
        }
    }
}

impl<'de> Deserialize<'de> for Tool {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(d)?;
        let type_str = value
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| D::Error::missing_field("type"))?;

        match type_str {
            CUSTOM_TYPE => serde_json::from_value(value)
                .map(Tool::Custom)
                .map_err(D::Error::custom),
            WEB_SEARCH_TYPE => serde_json::from_value(value)
                .map(Tool::WebSearch)
                .map_err(D::Error::custom),
            WEB_FETCH_TYPE => serde_json::from_value(value)
                .map(Tool::WebFetch)
                .map_err(D::Error::custom),
            CODE_EXECUTION_TYPE => serde_json::from_value(value)
                .map(Tool::CodeExecution)
                .map_err(D::Error::custom),
            BASH_TYPE => serde_json::from_value(value)
                .map(Tool::Bash)
                .map_err(D::Error::custom),
            STR_REPLACE_EDITOR_TYPE => serde_json::from_value(value)
                .map(Tool::StrReplaceEditor)
                .map_err(D::Error::custom),
            MEMORY_TYPE => serde_json::from_value(value)
                .map(Tool::Memory)
                .map_err(D::Error::custom),
            other => Err(D::Error::unknown_variant(
                other,
                &[
                    CUSTOM_TYPE,
                    WEB_SEARCH_TYPE,
                    WEB_FETCH_TYPE,
                    CODE_EXECUTION_TYPE,
                    BASH_TYPE,
                    STR_REPLACE_EDITOR_TYPE,
                    MEMORY_TYPE,
                ],
            )),
        }
    }
}
