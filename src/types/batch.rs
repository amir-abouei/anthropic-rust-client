use serde::{Deserialize, Serialize};

use super::message::{CreateMessageRequest, Message};

/// A single item inside a message batch request.
#[derive(Debug, Clone, Serialize)]
pub struct BatchRequestItem {
    /// Developer-provided ID, unique within the batch.
    pub custom_id: String,
    /// The same parameters accepted by `POST /v1/messages` (minus `stream`).
    pub params: CreateMessageRequest,
}

impl BatchRequestItem {
    pub fn new(custom_id: impl Into<String>, params: CreateMessageRequest) -> Self {
        Self { custom_id: custom_id.into(), params }
    }
}

/// Request body for `POST /v1/messages/batches`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateBatchRequest {
    pub requests: Vec<BatchRequestItem>,
}

impl CreateBatchRequest {
    pub fn new(requests: Vec<BatchRequestItem>) -> Self {
        Self { requests }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BatchProcessingStatus {
    InProgress,
    Canceling,
    Ended,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchRequestCounts {
    pub processing: u32,
    pub succeeded: u32,
    pub errored: u32,
    pub canceled: u32,
    pub expired: u32,
}

/// A message batch object.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageBatch {
    pub id: String,
    #[serde(rename = "type")]
    pub batch_type: String,
    pub processing_status: BatchProcessingStatus,
    pub request_counts: BatchRequestCounts,
    pub ended_at: Option<String>,
    pub created_at: String,
    pub expires_at: String,
    pub cancel_initiated_at: Option<String>,
    pub results_url: Option<String>,
}

impl MessageBatch {
    /// Returns `true` if the batch has finished processing.
    pub fn is_ended(&self) -> bool {
        self.processing_status == BatchProcessingStatus::Ended
    }
}

/// List response for `GET /v1/messages/batches`.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageBatchList {
    pub data: Vec<MessageBatch>,
    pub has_more: bool,
    pub first_id: Option<String>,
    pub last_id: Option<String>,
}

// ── Batch results ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum BatchResultType {
    Succeeded { message: Message },
    Errored { error: BatchError },
    Canceled,
    Expired,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchError {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// One JSONL line from the batch results file.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchResult {
    pub custom_id: String,
    pub result: BatchResultType,
}

/// Parameters for listing batches.
#[derive(Debug, Clone, Default)]
pub struct ListBatchesParams {
    pub after_id: Option<String>,
    pub before_id: Option<String>,
    pub limit: Option<u32>,
}

impl ListBatchesParams {
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
