use crate::error::{AnthropicError, Result};

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const DEFAULT_API_VERSION: &str = "2023-06-01";
const DEFAULT_TIMEOUT_SECS: u64 = 600;

/// Configuration for the Anthropic API client.
#[derive(Debug, Clone)]
pub struct Config {
    pub(crate) api_key: String,
    pub(crate) base_url: String,
    pub(crate) api_version: String,
    pub(crate) timeout_secs: u64,
    pub(crate) _max_retries: u32,
    pub(crate) default_headers: Vec<(String, String)>,
}

impl Config {
    pub fn builder(api_key: impl Into<String>) -> ConfigBuilder {
        ConfigBuilder::new(api_key)
    }

    /// Build config from environment variables.
    ///
    /// | Variable | Required | Default |
    /// |---|---|---|
    /// | `ANTHROPIC_API_KEY` | yes | — |
    /// | `ANTHROPIC_BASE_URL` | no | `https://api.anthropic.com` |
    /// | `ANTHROPIC_API_VERSION` | no | `2023-06-01` |
    /// | `ANTHROPIC_TIMEOUT` | no | `600` |
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").map_err(|_| {
            AnthropicError::ConfigError(
                "ANTHROPIC_API_KEY environment variable not set".to_string(),
            )
        })?;

        let mut builder = Self::builder(api_key);

        if let Ok(url) = std::env::var("ANTHROPIC_BASE_URL") {
            builder = builder.base_url(url);
        }
        if let Ok(version) = std::env::var("ANTHROPIC_API_VERSION") {
            builder = builder.api_version(version);
        }
        if let Ok(timeout) = std::env::var("ANTHROPIC_TIMEOUT") {
            let secs = timeout.parse::<u64>().map_err(|_| {
                AnthropicError::ConfigError(
                    "ANTHROPIC_TIMEOUT must be a positive integer (seconds)".to_string(),
                )
            })?;
            builder = builder.timeout_secs(secs);
        }

        Ok(builder.build())
    }
}

/// Builder for [`Config`].
#[derive(Debug)]
pub struct ConfigBuilder {
    api_key: String,
    base_url: String,
    api_version: String,
    timeout_secs: u64,
    max_retries: u32,
    default_headers: Vec<(String, String)>,
}

impl ConfigBuilder {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            api_version: DEFAULT_API_VERSION.to_string(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            max_retries: 2,
            default_headers: Vec::new(),
        }
    }

    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    pub fn api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    pub fn timeout_secs(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((key.into(), value.into()));
        self
    }

    pub fn build(self) -> Config {
        Config {
            api_key: self.api_key,
            base_url: self.base_url,
            api_version: self.api_version,
            timeout_secs: self.timeout_secs,
            _max_retries: self.max_retries,
            default_headers: self.default_headers,
        }
    }
}
