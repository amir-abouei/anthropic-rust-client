use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

use crate::{
    api::{batches::BatchesApi, files::FilesApi, messages::MessagesApi, models::ModelsApi},
    config::Config,
    error::{AnthropicError, ApiErrorBody, Result},
};

/// The main Anthropic API client.
///
/// Create with [`Client::new`] (reads `ANTHROPIC_API_KEY` from environment) or
/// [`Client::with_config`] for full control.
///
/// ```no_run
/// use anthropic_rust_client::Client;
///
/// let client = Client::new().unwrap();
/// ```
#[derive(Clone)]
pub struct Client {
    pub(crate) http: reqwest::Client,
    pub(crate) config: Config,
}

impl Client {
    /// Create a client using `ANTHROPIC_API_KEY` from the environment.
    pub fn new() -> Result<Self> {
        let config = Config::from_env()?;
        Self::with_config(config)
    }

    /// Create a client from an explicit API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        let config = Config::builder(api_key).build();
        Self::with_config(config)
    }

    /// Create a client from a fully-specified [`Config`].
    pub fn with_config(config: Config) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&config.api_key)
                .map_err(|e| AnthropicError::ConfigError(e.to_string()))?,
        );
        headers.insert(
            "anthropic-version",
            HeaderValue::from_str(&config.api_version)
                .map_err(|e| AnthropicError::ConfigError(e.to_string()))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // Caller-supplied default headers.
        for (k, v) in &config.default_headers {
            let name = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                .map_err(|e| AnthropicError::ConfigError(e.to_string()))?;
            let value = HeaderValue::from_str(v)
                .map_err(|e| AnthropicError::ConfigError(e.to_string()))?;
            headers.insert(name, value);
        }

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(AnthropicError::HttpError)?;

        Ok(Self { http, config })
    }

    // ── Sub-API accessors ──────────────────────────────────────────────────────

    pub fn messages(&self) -> MessagesApi<'_> { MessagesApi::new(self) }
    pub fn models(&self) -> ModelsApi<'_> { ModelsApi::new(self) }
    pub fn batches(&self) -> BatchesApi<'_> { BatchesApi::new(self) }
    pub fn files(&self) -> FilesApi<'_> { FilesApi::new(self) }

    // ── Internal helpers ───────────────────────────────────────────────────────

    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}{path}", self.config.base_url)
    }

    /// Parse the response, returning an error for non-2xx status codes.
    pub(crate) async fn check_response(
        response: reqwest::Response,
    ) -> Result<reqwest::Response> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status().as_u16();
        let bytes = response
            .bytes()
            .await
            .unwrap_or_default();

        let (message, error_type) =
            if let Ok(body) = serde_json::from_slice::<ApiErrorBody>(&bytes) {
                (body.error.message, Some(body.error.error_type))
            } else {
                (
                    String::from_utf8_lossy(&bytes).to_string(),
                    None,
                )
            };

        Err(match status {
            401 => AnthropicError::AuthError { message },
            429 => AnthropicError::RateLimitError { message, retry_after_secs: None },
            _ => AnthropicError::ApiError { status, message, error_type },
        })
    }

    /// Set a beta header on a request builder.
    pub(crate) fn with_beta(
        &self,
        req: reqwest::RequestBuilder,
        beta: &str,
    ) -> reqwest::RequestBuilder {
        req.header("anthropic-beta", beta)
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("base_url", &self.config.base_url)
            .field("api_version", &self.config.api_version)
            .finish_non_exhaustive()
    }
}
