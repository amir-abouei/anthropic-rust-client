use bytes::Bytes;

use crate::{
    client::Client,
    error::Result,
    types::file::{DeletedFile, FileList, FileMetadata, ListFilesParams},
};

const BETA: &str = "files-api-2025-04-14";

/// Files API (beta): upload, list, download, and delete files.
pub struct FilesApi<'a> {
    client: &'a Client,
}

impl<'a> FilesApi<'a> {
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

    /// Upload a file from raw bytes.
    ///
    /// `filename` is the name reported to the API; `mime_type` must match the
    /// file content (e.g. `"application/pdf"`, `"text/plain"`).
    pub async fn upload(
        &self,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        data: impl Into<Bytes>,
    ) -> Result<FileMetadata> {
        let filename = filename.into();
        let mime_type = mime_type.into();
        let data = data.into();

        // Use Part::stream so the Bytes buffer is not copied into a Vec.
        // reqwest::multipart sets the correct Content-Type header automatically,
        // overriding the client-level application/json default.
        let part = reqwest::multipart::Part::stream(data)
            .file_name(filename)
            .mime_str(&mime_type)
            .map_err(|e| crate::error::AnthropicError::FileError(e.to_string()))?;

        let form = reqwest::multipart::Form::new().part("file", part);

        let resp = self
            .base_req(reqwest::Method::POST, "/v1/files")
            .multipart(form)
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<FileMetadata>().await?)
    }

    /// List uploaded files.
    pub async fn list(&self, params: ListFilesParams) -> Result<FileList> {
        let mut req = self.base_req(reqwest::Method::GET, "/v1/files");

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
        Ok(resp.json::<FileList>().await?)
    }

    /// Retrieve metadata for a single file.
    pub async fn get(&self, file_id: &str) -> Result<FileMetadata> {
        let resp = self
            .base_req(reqwest::Method::GET, &format!("/v1/files/{file_id}"))
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<FileMetadata>().await?)
    }

    /// Download the raw bytes of a file.
    pub async fn download(&self, file_id: &str) -> Result<Bytes> {
        let resp = self
            .base_req(
                reqwest::Method::GET,
                &format!("/v1/files/{file_id}/content"),
            )
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.bytes().await?)
    }

    /// Permanently delete a file.
    pub async fn delete(&self, file_id: &str) -> Result<DeletedFile> {
        let resp = self
            .base_req(reqwest::Method::DELETE, &format!("/v1/files/{file_id}"))
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<DeletedFile>().await?)
    }
}
