use crate::{
    client::Client,
    error::{AnthropicError, Result},
    types::batch::{
        BatchResult, CreateBatchRequest, ListBatchesParams, MessageBatch, MessageBatchList,
    },
};

const BETA: &str = "message-batches-2024-09-24";

/// Message Batches API.
pub struct BatchesApi<'a> {
    client: &'a Client,
}

impl<'a> BatchesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    fn base_req(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        self.client
            .with_beta(
                self.client.http.request(method, self.client.url(path)),
                BETA,
            )
    }

    /// Submit a batch of message requests for async processing.
    pub async fn create(&self, request: CreateBatchRequest) -> Result<MessageBatch> {
        let resp = self
            .base_req(reqwest::Method::POST, "/v1/messages/batches")
            .json(&request)
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<MessageBatch>().await?)
    }

    /// Retrieve a single batch by ID.
    pub async fn get(&self, batch_id: &str) -> Result<MessageBatch> {
        let resp = self
            .base_req(
                reqwest::Method::GET,
                &format!("/v1/messages/batches/{batch_id}"),
            )
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<MessageBatch>().await?)
    }

    /// List all batches, newest first.
    pub async fn list(&self, params: ListBatchesParams) -> Result<MessageBatchList> {
        let mut req =
            self.base_req(reqwest::Method::GET, "/v1/messages/batches");

        if let Some(id) = params.after_id {
            req = req.query(&[("after_id", id)]);
        }
        if let Some(id) = params.before_id {
            req = req.query(&[("before_id", id)]);
        }
        if let Some(l) = params.limit {
            req = req.query(&[("limit", l.to_string())]);
        }

        let resp = req.send().await?;
        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<MessageBatchList>().await?)
    }

    /// Cancel an in-progress batch (returns the updated batch).
    pub async fn cancel(&self, batch_id: &str) -> Result<MessageBatch> {
        let resp = self
            .base_req(
                reqwest::Method::POST,
                &format!("/v1/messages/batches/{batch_id}/cancel"),
            )
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<MessageBatch>().await?)
    }

    /// Download and parse JSONL results for a completed batch.
    ///
    /// Fails if the batch has not yet ended (check `batch.is_ended()` first).
    pub async fn results(&self, batch_id: &str) -> Result<Vec<BatchResult>> {
        let resp = self
            .base_req(
                reqwest::Method::GET,
                &format!("/v1/messages/batches/{batch_id}/results"),
            )
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        let text = resp.text().await?;

        // Results are JSONL (one JSON object per line).
        text.lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| {
                serde_json::from_str::<BatchResult>(line).map_err(AnthropicError::JsonError)
            })
            .collect()
    }
}
