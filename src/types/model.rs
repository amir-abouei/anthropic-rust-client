use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct CapabilitySupport {
    pub supported: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextManagementCapability {
    pub supported: bool,
    pub clear_thinking_20251015: Option<CapabilitySupport>,
    pub clear_tool_uses_20250919: Option<CapabilitySupport>,
    pub compact_20260112: Option<CapabilitySupport>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EffortCapability {
    pub supported: bool,
    pub low: Option<CapabilitySupport>,
    pub medium: Option<CapabilitySupport>,
    pub high: Option<CapabilitySupport>,
    pub xhigh: Option<CapabilitySupport>,
    pub max: Option<CapabilitySupport>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThinkingTypes {
    pub enabled: Option<CapabilitySupport>,
    pub adaptive: Option<CapabilitySupport>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ThinkingCapability {
    pub supported: bool,
    pub types: Option<ThinkingTypes>,
}

/// Full capability set for a model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelCapabilities {
    pub batch: Option<CapabilitySupport>,
    pub citations: Option<CapabilitySupport>,
    pub code_execution: Option<CapabilitySupport>,
    pub context_management: Option<ContextManagementCapability>,
    pub effort: Option<EffortCapability>,
    pub image_input: Option<CapabilitySupport>,
    pub pdf_input: Option<CapabilitySupport>,
    pub structured_outputs: Option<CapabilitySupport>,
    pub thinking: Option<ThinkingCapability>,
}

/// Metadata for a single Claude model.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub created_at: String,
    pub max_input_tokens: u64,
    pub max_tokens: u64,
    pub capabilities: Option<ModelCapabilities>,
}

/// Response from `GET /v1/models`.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelList {
    pub data: Vec<ModelInfo>,
    pub has_more: bool,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
}

/// Query parameters for listing models.
#[derive(Debug, Clone, Default)]
pub struct ListModelsParams {
    pub after_id: Option<String>,
    pub before_id: Option<String>,
    pub limit: Option<u32>,
}

impl ListModelsParams {
    pub fn after_id(mut self, id: impl Into<String>) -> Self {
        self.after_id = Some(id.into());
        self
    }

    pub fn before_id(mut self, id: impl Into<String>) -> Self {
        self.before_id = Some(id.into());
        self
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}
