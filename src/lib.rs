//! # anthropic-rust-client
//!
//! Comprehensive async Rust SDK for the [Anthropic Claude API](https://platform.claude.com/docs/en/api/overview).
//!
//! ## Supported APIs
//!
//! | API | Endpoints |
//! |-----|-----------|
//! | **Messages** | `POST /v1/messages` (blocking + streaming) |
//! | **Token Counting** | `POST /v1/messages/count_tokens` |
//! | **Models** | `GET /v1/models`, `GET /v1/models/{id}` |
//! | **Message Batches** | Full CRUD + results download |
//! | **Files** *(beta)* | Upload, list, get, download, delete |
//!
//! ## Quick start
//!
//! ```no_run
//! use anthropic_rust_client::{Client, types::message::CreateMessageRequest, types::common::Model};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new()?; // reads ANTHROPIC_API_KEY
//!
//!     let message = client
//!         .messages()
//!         .create(
//!             CreateMessageRequest::builder()
//!                 .model(Model::claude_opus_4_8())
//!                 .max_tokens(1024)
//!                 .user("Hello, Claude!")
//!                 .build()?,
//!         )
//!         .await?;
//!
//!     println!("{}", message.text());
//!     Ok(())
//! }
//! ```

pub mod api;
pub mod client;
pub mod config;
pub mod error;
pub mod streaming;
pub mod types;

// ── Top-level re-exports ──────────────────────────────────────────────────────

pub use client::Client;
pub use config::{Config, ConfigBuilder};
pub use error::{AnthropicError, Result};
pub use streaming::MessageStream;

// Commonly used types re-exported for convenience.
pub use types::{
    batch::{BatchRequestItem, BatchResult, CreateBatchRequest, MessageBatch},
    common::{CacheControl, Metadata, Model, Role, StopReason, Usage},
    content::{
        Citation, CitationsConfig, DocumentMediaType, DocumentSource, ImageMediaType, ImageSource,
        InputContentBlock, MessageContent, OutputContentBlock, SystemBlock, SystemPrompt,
        ToolResultContent,
    },
    file::{DeletedFile, FileList, FileMetadata, ListFilesParams},
    message::{
        CountTokensRequest, CreateMessageRequest, CreateMessageRequestBuilder, Message,
        MessageParam, OutputConfig, OutputEffort, ThinkingConfig, TokenCountResponse,
    },
    model::{ListModelsParams, ModelInfo, ModelList},
    stream::{
        AccumulatedBlock, ContentBlockDelta, ContentBlockStartData, MessageAccumulator,
        MessageDeltaData, StreamEvent,
    },
    tool::{
        BashTool, CodeExecutionTool, CustomTool, MemoryTool, StrReplaceEditorTool, Tool,
        ToolChoice, UserLocation, WebFetchTool, WebSearchTool,
    },
};
