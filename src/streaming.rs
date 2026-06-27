use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{stream, Stream, StreamExt};

use crate::{
    error::AnthropicError,
    types::stream::{MessageAccumulator, StreamEvent},
};

type BoxStream = Pin<Box<dyn Stream<Item = crate::error::Result<StreamEvent>> + Send + 'static>>;

/// A stream of [`StreamEvent`]s from a streaming Messages request.
pub struct MessageStream {
    inner: BoxStream,
}

impl MessageStream {
    pub(crate) fn new(response: reqwest::Response) -> Self {
        Self { inner: Box::pin(parse_sse(response)) }
    }

    /// Collect all text deltas into a `String`.
    pub async fn text(mut self) -> crate::error::Result<String> {
        let mut buf = String::new();
        while let Some(event) = self.next().await {
            if let Ok(ev) = event {
                if let Some(t) = ev.as_text_delta() {
                    buf.push_str(t);
                }
            }
        }
        Ok(buf)
    }

    /// Accumulate all events into a complete [`crate::types::message::Message`].
    pub async fn collect_message(
        mut self,
    ) -> crate::error::Result<crate::types::message::Message> {
        let mut acc = MessageAccumulator::default();
        while let Some(event) = self.next().await {
            acc.apply(&event?);
        }
        Ok(acc.into_message())
    }
}

impl Stream for MessageStream {
    type Item = crate::error::Result<StreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

// ── SSE parser ────────────────────────────────────────────────────────────────

fn parse_sse(
    response: reqwest::Response,
) -> impl Stream<Item = crate::error::Result<StreamEvent>> + Send + 'static {
    let byte_stream = response.bytes_stream();

    stream::unfold(
        (byte_stream, String::new()),
        |(mut byte_stream, mut buffer)| async move {
            loop {
                // If a complete SSE event is already in the buffer, parse it.
                if let Some(pos) = buffer.find("\n\n") {
                    let event_str = buffer[..pos].to_string();
                    let remaining = buffer[pos + 2..].to_string();

                    // Extract the `data:` field.
                    let data = event_str
                        .lines()
                        .find(|l| l.starts_with("data:"))
                        .map(|l| l["data:".len()..].trim().to_string());

                    match data {
                        Some(d) if d == "[DONE]" => return None,
                        Some(d) => {
                            let result = serde_json::from_str::<StreamEvent>(&d)
                                .map_err(AnthropicError::JsonError);

                            // Check for an in-band error event.
                            let result = result.and_then(|ev| match &ev {
                                StreamEvent::Error { error } => {
                                    Err(AnthropicError::StreamError(error.message.clone()))
                                }
                                _ => Ok(ev),
                            });

                            return Some((result, (byte_stream, remaining)));
                        }
                        None => {
                            // Ping or event-only line — skip and continue.
                            buffer = remaining;
                            continue;
                        }
                    }
                }

                // Need more bytes.
                match byte_stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.push_str(&String::from_utf8_lossy(&chunk));
                    }
                    Some(Err(e)) => {
                        return Some((Err(AnthropicError::HttpError(e)), (byte_stream, buffer)));
                    }
                    None => return None,
                }
            }
        },
    )
}
