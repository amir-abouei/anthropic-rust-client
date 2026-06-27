//! Unit tests for SSE parsing and accumulation.

use anthropic_rust_client::types::stream::{
    ContentBlockDelta, ContentBlockStartData, MessageAccumulator, MessageDeltaData,
    MessageStartData, StreamEvent,
};
use anthropic_rust_client::{StopReason, Usage};

fn make_usage() -> Usage {
    Usage {
        input_tokens: 10,
        output_tokens: 5,
        cache_creation_input_tokens: None,
        cache_read_input_tokens: None,
        output_tokens_details: None,
        server_tool_use: None,
        service_tier: None,
    }
}

fn start_event(model: &str) -> StreamEvent {
    StreamEvent::MessageStart {
        message: MessageStartData {
            id: "msg_test".to_string(),
            message_type: "message".to_string(),
            role: "assistant".to_string(),
            model: model.to_string(),
            content: vec![],
            stop_reason: None,
            stop_sequence: None,
            usage: make_usage(),
        },
    }
}

// ── StreamEvent deserialization ───────────────────────────────────────────────

#[test]
fn deserialize_ping() {
    let ev: StreamEvent = serde_json::from_str(r#"{"type":"ping"}"#).unwrap();
    assert!(matches!(ev, StreamEvent::Ping));
}

#[test]
fn deserialize_message_stop() {
    let ev: StreamEvent = serde_json::from_str(r#"{"type":"message_stop"}"#).unwrap();
    assert!(matches!(ev, StreamEvent::MessageStop));
    assert!(ev.is_message_stop());
}

#[test]
fn deserialize_content_block_delta_text() {
    let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}"#;
    let ev: StreamEvent = serde_json::from_str(json).unwrap();
    assert_eq!(ev.as_text_delta(), Some("Hello"));
}

#[test]
fn deserialize_content_block_delta_thinking() {
    let json = r#"{"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"I think..."}}"#;
    let ev: StreamEvent = serde_json::from_str(json).unwrap();
    assert_eq!(ev.as_thinking_delta(), Some("I think..."));
}

#[test]
fn deserialize_content_block_start_text() {
    let json = r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#;
    let ev: StreamEvent = serde_json::from_str(json).unwrap();
    assert!(matches!(
        ev,
        StreamEvent::ContentBlockStart {
            content_block: ContentBlockStartData::Text { .. },
            ..
        }
    ));
}

#[test]
fn deserialize_content_block_start_tool_use() {
    let json = r#"{"type":"content_block_start","index":0,"content_block":{"type":"tool_use","id":"toolu_1","name":"get_weather"}}"#;
    let ev: StreamEvent = serde_json::from_str(json).unwrap();
    assert!(matches!(
        ev,
        StreamEvent::ContentBlockStart {
            content_block: ContentBlockStartData::ToolUse { .. },
            ..
        }
    ));
}

#[test]
fn deserialize_message_delta() {
    let json = r#"{"type":"message_delta","delta":{"stop_reason":"end_turn","stop_sequence":null},"usage":{"output_tokens":15}}"#;
    let ev: StreamEvent = serde_json::from_str(json).unwrap();
    assert!(matches!(ev, StreamEvent::MessageDelta { .. }));
    if let StreamEvent::MessageDelta { delta, .. } = &ev {
        assert_eq!(delta.stop_reason, Some(StopReason::EndTurn));
    }
}

#[test]
fn deserialize_error_event() {
    let json = r#"{"type":"error","error":{"type":"overloaded_error","message":"Overloaded"}}"#;
    let ev: StreamEvent = serde_json::from_str(json).unwrap();
    assert!(matches!(ev, StreamEvent::Error { .. }));
}

// ── MessageAccumulator ────────────────────────────────────────────────────────

#[test]
fn accumulator_builds_text_message() {
    let events = vec![
        start_event("claude-opus-4-8"),
        StreamEvent::ContentBlockStart {
            index: 0,
            content_block: ContentBlockStartData::Text { text: String::new() },
        },
        StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDelta::TextDelta { text: "Hello, ".to_string() },
        },
        StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDelta::TextDelta { text: "world!".to_string() },
        },
        StreamEvent::ContentBlockStop { index: 0 },
        StreamEvent::MessageDelta {
            delta: MessageDeltaData {
                stop_reason: Some(StopReason::EndTurn),
                stop_sequence: None,
            },
            usage: None,
        },
        StreamEvent::MessageStop,
    ];

    let mut acc = MessageAccumulator::default();
    for ev in &events {
        acc.apply(ev);
    }
    let msg = acc.into_message().unwrap();

    assert_eq!(msg.id, "msg_test");
    assert_eq!(msg.text(), "Hello, world!");
    assert_eq!(msg.stop_reason, Some(StopReason::EndTurn));
}

#[test]
fn accumulator_builds_tool_use_message() {
    let events = vec![
        start_event("claude-opus-4-8"),
        StreamEvent::ContentBlockStart {
            index: 0,
            content_block: ContentBlockStartData::ToolUse {
                id: "toolu_abc".to_string(),
                name: "get_weather".to_string(),
            },
        },
        StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDelta::InputJsonDelta {
                partial_json: r#"{"city": "#.to_string(),
            },
        },
        StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDelta::InputJsonDelta {
                partial_json: r#""Tokyo"}"#.to_string(),
            },
        },
        StreamEvent::ContentBlockStop { index: 0 },
        StreamEvent::MessageDelta {
            delta: MessageDeltaData {
                stop_reason: Some(StopReason::ToolUse),
                stop_sequence: None,
            },
            usage: None,
        },
        StreamEvent::MessageStop,
    ];

    let mut acc = MessageAccumulator::default();
    for ev in &events {
        acc.apply(ev);
    }
    let msg = acc.into_message().unwrap();

    assert!(msg.wants_tool_use());
    let tool_uses = msg.tool_uses();
    assert_eq!(tool_uses.len(), 1);

    use anthropic_rust_client::OutputContentBlock;
    if let OutputContentBlock::ToolUse { name, input, .. } = tool_uses[0] {
        assert_eq!(name, "get_weather");
        assert_eq!(input["city"], serde_json::json!("Tokyo"));
    }
}

#[test]
fn accumulator_builds_thinking_message() {
    let events = vec![
        start_event("claude-opus-4-8"),
        StreamEvent::ContentBlockStart {
            index: 0,
            content_block: ContentBlockStartData::Thinking { thinking: String::new() },
        },
        StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDelta::ThinkingDelta {
                thinking: "Let me think step by step...".to_string(),
            },
        },
        StreamEvent::ContentBlockDelta {
            index: 0,
            delta: ContentBlockDelta::SignatureDelta {
                signature: "sig123".to_string(),
            },
        },
        StreamEvent::ContentBlockStop { index: 0 },
        StreamEvent::ContentBlockStart {
            index: 1,
            content_block: ContentBlockStartData::Text { text: String::new() },
        },
        StreamEvent::ContentBlockDelta {
            index: 1,
            delta: ContentBlockDelta::TextDelta { text: "The answer is 42.".to_string() },
        },
        StreamEvent::ContentBlockStop { index: 1 },
        StreamEvent::MessageDelta {
            delta: MessageDeltaData {
                stop_reason: Some(StopReason::EndTurn),
                stop_sequence: None,
            },
            usage: None,
        },
        StreamEvent::MessageStop,
    ];

    let mut acc = MessageAccumulator::default();
    for ev in &events {
        acc.apply(ev);
    }
    let msg = acc.into_message().unwrap();

    assert_eq!(msg.thinking(), Some("Let me think step by step..."));
    assert_eq!(msg.text(), "The answer is 42.");
}
