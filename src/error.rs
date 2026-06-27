use thiserror::Error;

#[derive(Debug, Error)]
pub enum AnthropicError {
    #[error("API error {status}: {message}")]
    ApiError {
        status: u16,
        message: String,
        error_type: Option<String>,
    },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Rate limit exceeded: {message}")]
    RateLimitError {
        message: String,
        retry_after_secs: Option<u64>,
    },

    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("File error: {0}")]
    FileError(String),
}

pub type Result<T> = std::result::Result<T, AnthropicError>;

/// Parsed Anthropic API error body.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct ApiErrorBody {
    #[serde(rename = "type")]
    pub _type: String,
    pub error: ApiErrorDetail,
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ApiErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}
