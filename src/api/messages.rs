use crate::{
    client::Client,
    error::Result,
    streaming::MessageStream,
    types::message::{
        CountTokensRequest, CreateMessageRequest, Message, TokenCountResponse,
    },
};

pub use crate::streaming::MessageStream as StreamHandle;

/// Messages API: `POST /v1/messages` and `POST /v1/messages/count_tokens`.
pub struct MessagesApi<'a> {
    client: &'a Client,
}

impl<'a> MessagesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Send a message and receive a complete [`Message`] response.
    pub async fn create(&self, mut request: CreateMessageRequest) -> Result<Message> {
        request.stream = None; // ensure no stream flag leaks in
        let resp = self
            .client
            .http
            .post(self.client.url("/v1/messages"))
            .json(&request)
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<Message>().await?)
    }

    /// Send a message and receive a [`MessageStream`] of server-sent events.
    pub async fn stream(&self, mut request: CreateMessageRequest) -> Result<MessageStream> {
        request.stream = Some(true);
        let resp = self
            .client
            .http
            .post(self.client.url("/v1/messages"))
            .json(&request)
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(MessageStream::new(resp))
    }

    /// Count the tokens a request would consume without actually sending it.
    pub async fn count_tokens(
        &self,
        request: CountTokensRequest,
    ) -> Result<TokenCountResponse> {
        let resp = self
            .client
            .http
            .post(self.client.url("/v1/messages/count_tokens"))
            .json(&request)
            .send()
            .await?;

        let resp = Client::check_response(resp).await?;
        Ok(resp.json::<TokenCountResponse>().await?)
    }
}
