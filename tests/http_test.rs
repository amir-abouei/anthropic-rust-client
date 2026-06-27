//! Integration tests for HTTP error handling using a mock server.

use anthropic_rust_client::{AnthropicError, Client, Config, CreateMessageRequest, Model};

fn make_client(base_url: &str) -> Client {
    Client::with_config(Config::builder("test-key").base_url(base_url).build()).unwrap()
}

// ── 401 authentication error ──────────────────────────────────────────────────

#[tokio::test]
async fn check_response_401_returns_auth_error() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/v1/messages")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"type":"error","error":{"type":"authentication_error","message":"Invalid API key"}}"#,
        )
        .create_async()
        .await;

    let client = make_client(&server.url());
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(10)
        .user("hello")
        .build()
        .unwrap();

    let err = client.messages().create(req).await.unwrap_err();
    assert!(matches!(err, AnthropicError::AuthError { .. }), "expected AuthError, got {err:?}");
}

// ── 429 rate limit with Retry-After header ────────────────────────────────────

#[tokio::test]
async fn check_response_429_parses_retry_after() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/v1/messages")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_header("retry-after", "30")
        .with_body(
            r#"{"type":"error","error":{"type":"rate_limit_error","message":"Rate limited"}}"#,
        )
        .create_async()
        .await;

    let client = make_client(&server.url());
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(10)
        .user("hello")
        .build()
        .unwrap();

    let err = client.messages().create(req).await.unwrap_err();
    match err {
        AnthropicError::RateLimitError { retry_after_secs, .. } => {
            assert_eq!(retry_after_secs, Some(30));
        }
        other => panic!("expected RateLimitError, got {other:?}"),
    }
}

// ── 500 with non-JSON body ────────────────────────────────────────────────────

#[tokio::test]
async fn check_response_non_json_error_body() {
    let mut server = mockito::Server::new_async().await;
    let _m = server
        .mock("POST", "/v1/messages")
        .with_status(500)
        .with_header("content-type", "text/plain")
        .with_body("Internal Server Error")
        .create_async()
        .await;

    let client = make_client(&server.url());
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(10)
        .user("hello")
        .build()
        .unwrap();

    let err = client.messages().create(req).await.unwrap_err();
    match err {
        AnthropicError::ApiError { status, message, .. } => {
            assert_eq!(status, 500);
            assert!(message.contains("Internal Server Error"));
        }
        other => panic!("expected ApiError, got {other:?}"),
    }
}

// ── Streaming SSE end-to-end (incl. non-ASCII text) ───────────────────────────

#[tokio::test]
async fn stream_collects_text_with_non_ascii() {
    let mut server = mockito::Server::new_async().await;
    let body = concat!(
        "event: message_start\n",
        r#"data: {"type":"message_start","message":{"id":"msg_1","type":"message","role":"assistant","model":"claude-haiku-4-5","content":[],"stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":0}}}"#,
        "\n\n",
        "event: content_block_start\n",
        r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
        "\n\n",
        "event: content_block_delta\n",
        r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"café—"}}"#,
        "\n\n",
        "event: content_block_stop\n",
        r#"data: {"type":"content_block_stop","index":0}"#,
        "\n\n",
        "event: message_stop\n",
        r#"data: {"type":"message_stop"}"#,
        "\n\n",
    );

    let _m = server
        .mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(body)
        .create_async()
        .await;

    let client = make_client(&server.url());
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(10)
        .user("hi")
        .build()
        .unwrap();

    let text = client.messages().stream(req).await.unwrap().text().await.unwrap();
    assert_eq!(text, "café—");
}

// ── Config debug redacts API key ──────────────────────────────────────────────

#[test]
fn config_debug_redacts_api_key() {
    let config = Config::builder("sk-secret-key-12345").build();
    let debug_str = format!("{config:?}");
    assert!(!debug_str.contains("sk-secret-key-12345"), "API key must not appear in Debug output");
    assert!(debug_str.contains("[REDACTED]"));
}
