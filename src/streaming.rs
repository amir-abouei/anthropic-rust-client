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
    ///
    /// Returns an error if the stream fails partway through.
    pub async fn text(mut self) -> crate::error::Result<String> {
        let mut buf = String::new();
        while let Some(event) = self.next().await {
            if let Some(t) = event?.as_text_delta() {
                buf.push_str(t);
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
        acc.into_message()
    }
}

impl Stream for MessageStream {
    type Item = crate::error::Result<StreamEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

// ── SSE parser ────────────────────────────────────────────────────────────────

/// Find the first occurrence of `needle` in `haystack`.
fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn parse_sse(
    response: reqwest::Response,
) -> impl Stream<Item = crate::error::Result<StreamEvent>> + Send + 'static {
    let byte_stream = response.bytes_stream();

    // The buffer holds raw bytes: a multi-byte UTF-8 character may be split
    // across HTTP chunks, so decoding is deferred until a full event is framed.
    stream::unfold(
        (byte_stream, Vec::<u8>::new()),
        |(mut byte_stream, mut buffer)| async move {
            loop {
                // If a complete SSE event is already in the buffer, parse it.
                if let Some(pos) = find_subslice(&buffer, b"\n\n") {
                    let event_bytes: Vec<u8> = buffer.drain(..pos + 2).collect();
                    let event_str = String::from_utf8_lossy(&event_bytes[..pos]);

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

                            return Some((result, (byte_stream, buffer)));
                        }
                        None => {
                            // Ping or event-only frame — skip and continue.
                            continue;
                        }
                    }
                }

                // Need more bytes.
                match byte_stream.next().await {
                    Some(Ok(chunk)) => {
                        buffer.extend_from_slice(&chunk);
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

#[cfg(test)]
mod tests {
    use super::find_subslice;

    #[test]
    fn find_subslice_basic() {
        assert_eq!(find_subslice(b"abc\n\ndef", b"\n\n"), Some(3));
        assert_eq!(find_subslice(b"no delimiter", b"\n\n"), None);
        assert_eq!(find_subslice(b"\n\nstart", b"\n\n"), Some(0));
    }

    #[test]
    fn framing_reconstructs_split_multibyte_char() {
        // "café—" with the 3-byte em-dash split across two appends must decode
        // correctly once the full frame is buffered.
        let full = "café—\n\n".as_bytes().to_vec();
        let split = 6; // mid em-dash sequence
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&full[..split]);
        buffer.extend_from_slice(&full[split..]);

        let pos = find_subslice(&buffer, b"\n\n").unwrap();
        let decoded = String::from_utf8_lossy(&buffer[..pos]);
        assert_eq!(decoded, "café—");
    }
}
