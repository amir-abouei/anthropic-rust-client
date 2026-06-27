use crate::{
    client::Client,
    error::Result,
    types::model::{ListModelsParams, ModelInfo, ModelList},
};

/// Models API: `GET /v1/models` and `GET /v1/models/{model_id}`.
pub struct ModelsApi<'a> {
    client: &'a Client,
}

impl<'a> ModelsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all available models (most-recently-released first).
    pub async fn list(&self, params: ListModelsParams) -> Result<ModelList> {
        let mut req = self.client.http.get(self.client.url("/v1/models"));

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
        Ok(resp.json::<ModelList>().await?)
    }

    /// Retrieve a specific model (also resolves aliases).
    pub async fn get(&self, model_id: &str) -> Result<ModelInfo> {
        let resp = self
            .client
            .http
            .get(self.client.url(&format!("/v1/models/{model_id}")))
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<ModelInfo>().await?)
    }
}
