//! Unit tests for serialization / deserialization of all public types.

use anthropic_rust_client::{
    CacheControl, CreateMessageRequest, ImageMediaType, ImageSource, InputContentBlock,
    MessageContent, MessageParam, Model, OutputContentBlock, Role, StopReason, SystemBlock,
    SystemPrompt, ThinkingConfig, Tool, ToolChoice, Usage,
};
use serde_json::json;

// ── Model ─────────────────────────────────────────────────────────────────────

#[test]
fn model_serializes_as_string() {
    let m = Model::claude_opus_4_8();
    let j = serde_json::to_value(&m).unwrap();
    assert_eq!(j, json!("claude-opus-4-8"));
}

#[test]
fn model_from_str() {
    let m: Model = "claude-sonnet-4-6".into();
    assert_eq!(m.0, "claude-sonnet-4-6");
}

#[test]
fn model_display() {
    assert_eq!(Model::claude_haiku_4_5().to_string(), "claude-haiku-4-5-20251001");
}

// ── Role ─────────────────────────────────────────────────────────────────────

#[test]
fn role_serialization() {
    assert_eq!(serde_json::to_value(Role::User).unwrap(), json!("user"));
    assert_eq!(serde_json::to_value(Role::Assistant).unwrap(), json!("assistant"));
}

#[test]
fn role_deserialization() {
    let r: Role = serde_json::from_str(r#""user""#).unwrap();
    assert_eq!(r, Role::User);
}

// ── StopReason ────────────────────────────────────────────────────────────────

#[test]
fn stop_reason_roundtrip() {
    for (variant, expected) in [
        (StopReason::EndTurn, "end_turn"),
        (StopReason::MaxTokens, "max_tokens"),
        (StopReason::ToolUse, "tool_use"),
        (StopReason::PauseTurn, "pause_turn"),
    ] {
        let serialized = serde_json::to_value(&variant).unwrap();
        assert_eq!(serialized, json!(expected));

        let deserialized: StopReason = serde_json::from_str(&format!("\"{expected}\"")).unwrap();
        assert_eq!(deserialized, variant);
    }
}

// ── CacheControl ──────────────────────────────────────────────────────────────

#[test]
fn cache_control_ephemeral() {
    let cc = CacheControl::ephemeral();
    let j = serde_json::to_value(&cc).unwrap();
    assert_eq!(j["type"], json!("ephemeral"));
    assert_eq!(j.get("ttl"), None);
}

#[test]
fn cache_control_with_ttl() {
    let cc = CacheControl::ephemeral_with_ttl("1h");
    let j = serde_json::to_value(&cc).unwrap();
    assert_eq!(j["ttl"], json!("1h"));
}

// ── ImageSource ───────────────────────────────────────────────────────────────

#[test]
fn image_source_base64_serializes() {
    let src = ImageSource::base64(ImageMediaType::Jpeg, "abc123");
    let j = serde_json::to_value(&src).unwrap();
    assert_eq!(j["type"], json!("base64"));
    assert_eq!(j["media_type"], json!("image/jpeg"));
    assert_eq!(j["data"], json!("abc123"));
}

#[test]
fn image_source_url_serializes() {
    let src = ImageSource::url("https://example.com/img.png");
    let j = serde_json::to_value(&src).unwrap();
    assert_eq!(j["type"], json!("url"));
    assert_eq!(j["url"], json!("https://example.com/img.png"));
}

// ── InputContentBlock ─────────────────────────────────────────────────────────

#[test]
fn text_block_serializes() {
    let b = InputContentBlock::text("hello");
    let j = serde_json::to_value(&b).unwrap();
    assert_eq!(j["type"], json!("text"));
    assert_eq!(j["text"], json!("hello"));
    assert_eq!(j.get("cache_control"), None);
}

#[test]
fn text_block_with_cache_serializes() {
    let b = InputContentBlock::text_with_cache("hello");
    let j = serde_json::to_value(&b).unwrap();
    assert_eq!(j["cache_control"]["type"], json!("ephemeral"));
}

#[test]
fn tool_result_block_serializes() {
    let b = InputContentBlock::tool_result("toolu_abc", "42 degrees");
    let j = serde_json::to_value(&b).unwrap();
    assert_eq!(j["type"], json!("tool_result"));
    assert_eq!(j["tool_use_id"], json!("toolu_abc"));
}

#[test]
fn tool_error_block_has_is_error_true() {
    let b = InputContentBlock::tool_error("toolu_abc", "not found");
    let j = serde_json::to_value(&b).unwrap();
    assert_eq!(j["is_error"], json!(true));
}

// ── MessageContent (untagged union) ───────────────────────────────────────────

#[test]
fn message_content_text_roundtrip() {
    let mc = MessageContent::Text("hi".to_string());
    let j = serde_json::to_value(&mc).unwrap();
    assert_eq!(j, json!("hi"));

    let deserialized: MessageContent = serde_json::from_value(json!("hi")).unwrap();
    assert!(matches!(deserialized, MessageContent::Text(_)));
}

#[test]
fn message_content_blocks_roundtrip() {
    let mc = MessageContent::Blocks(vec![InputContentBlock::text("hello")]);
    let j = serde_json::to_value(&mc).unwrap();
    assert!(j.is_array());
}

// ── SystemPrompt ──────────────────────────────────────────────────────────────

#[test]
fn system_prompt_text_serializes_as_string() {
    let sp: SystemPrompt = "Be helpful.".into();
    let j = serde_json::to_value(&sp).unwrap();
    assert_eq!(j, json!("Be helpful."));
}

#[test]
fn system_prompt_blocks_serializes_as_array() {
    let sp = SystemPrompt::Blocks(vec![SystemBlock::text("Be helpful.")]);
    let j = serde_json::to_value(&sp).unwrap();
    assert!(j.is_array());
    assert_eq!(j[0]["type"], json!("text"));
}

// ── ThinkingConfig ────────────────────────────────────────────────────────────

#[test]
fn thinking_enabled_serializes() {
    let t = ThinkingConfig::enabled(8000);
    let j = serde_json::to_value(&t).unwrap();
    assert_eq!(j["type"], json!("enabled"));
    assert_eq!(j["budget_tokens"], json!(8000));
}

#[test]
fn thinking_disabled_serializes() {
    let t = ThinkingConfig::Disabled;
    let j = serde_json::to_value(&t).unwrap();
    assert_eq!(j["type"], json!("disabled"));
}

#[test]
fn thinking_adaptive_serializes() {
    let t = ThinkingConfig::adaptive();
    let j = serde_json::to_value(&t).unwrap();
    assert_eq!(j["type"], json!("adaptive"));
    assert_eq!(j.get("budget_tokens"), None); // omitted when None
}

// ── ToolChoice ────────────────────────────────────────────────────────────────

#[test]
fn tool_choice_auto() {
    let j = serde_json::to_value(ToolChoice::Auto).unwrap();
    assert_eq!(j["type"], json!("auto"));
}

#[test]
fn tool_choice_tool() {
    let j = serde_json::to_value(ToolChoice::Tool { name: "my_tool".to_string() }).unwrap();
    assert_eq!(j["type"], json!("tool"));
    assert_eq!(j["name"], json!("my_tool"));
}

// ── Tool ─────────────────────────────────────────────────────────────────────

#[test]
fn custom_tool_serializes() {
    let t = Tool::custom("search", "Search the web", json!({"type": "object", "properties": {}}));
    let j = serde_json::to_value(&t).unwrap();
    assert_eq!(j["type"], json!("custom"));
    assert_eq!(j["name"], json!("search"));
    assert_eq!(j["description"], json!("Search the web"));
}

#[test]
fn web_search_tool_serializes() {
    let t = Tool::web_search();
    let j = serde_json::to_value(&t).unwrap();
    assert_eq!(j["type"], json!("web_search_20260209"));
}

#[test]
fn code_execution_tool_serializes() {
    let t = Tool::code_execution();
    let j = serde_json::to_value(&t).unwrap();
    assert_eq!(j["type"], json!("code_execution_20260521"));
}

#[test]
fn bash_tool_serializes() {
    let t = Tool::bash();
    let j = serde_json::to_value(&t).unwrap();
    assert_eq!(j["type"], json!("bash_20250124"));
}

// ── CreateMessageRequest builder ──────────────────────────────────────────────

#[test]
fn builder_requires_model() {
    let result = CreateMessageRequest::builder()
        .max_tokens(100)
        .user("hi")
        .build();
    assert!(result.is_err());
}

#[test]
fn builder_requires_max_tokens() {
    let result = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .user("hi")
        .build();
    assert!(result.is_err());
}

#[test]
fn builder_requires_messages() {
    let result = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(100)
        .build();
    assert!(result.is_err());
}

#[test]
fn builder_minimal_request_serializes() {
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(256)
        .user("Hello!")
        .build()
        .unwrap();

    let j = serde_json::to_value(&req).unwrap();
    assert_eq!(j["model"], json!("claude-haiku-4-5-20251001"));
    assert_eq!(j["max_tokens"], json!(256));
    assert_eq!(j["messages"][0]["role"], json!("user"));
    assert_eq!(j["messages"][0]["content"], json!("Hello!"));

    // Optional fields must be absent.
    assert_eq!(j.get("stream"), None);
    assert_eq!(j.get("system"), None);
    assert_eq!(j.get("tools"), None);
    assert_eq!(j.get("temperature"), None);
}

#[test]
fn builder_full_request_serializes() {
    let req = CreateMessageRequest::builder()
        .model(Model::claude_opus_4_8())
        .max_tokens(4096)
        .system("You are a helpful assistant.")
        .user("What is 2+2?")
        .temperature(0.5)
        .top_p(0.9)
        .stop_sequence("STOP")
        .tool(Tool::web_search())
        .tool_choice(ToolChoice::Auto)
        .thinking(ThinkingConfig::enabled(2048))
        .build()
        .unwrap();

    let j = serde_json::to_value(&req).unwrap();
    assert_eq!(j["temperature"], json!(0.5));
    assert_eq!(j["top_p"], json!(0.9));
    assert_eq!(j["stop_sequences"][0], json!("STOP"));
    assert_eq!(j["tools"][0]["type"], json!("web_search_20260209"));
    assert_eq!(j["tool_choice"]["type"], json!("auto"));
    assert_eq!(j["thinking"]["type"], json!("enabled"));
    assert_eq!(j["thinking"]["budget_tokens"], json!(2048));
}

#[test]
fn builder_multi_turn_conversation() {
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(256)
        .user("My name is Alice.")
        .assistant("Nice to meet you, Alice!")
        .user("What's my name?")
        .build()
        .unwrap();

    let j = serde_json::to_value(&req).unwrap();
    let msgs = j["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 3);
    assert_eq!(msgs[0]["role"], json!("user"));
    assert_eq!(msgs[1]["role"], json!("assistant"));
    assert_eq!(msgs[2]["role"], json!("user"));
}

#[test]
fn cache_control_promotes_plain_text_message() {
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(100)
        .user("hello")
        .cache_control(CacheControl::ephemeral())
        .build()
        .unwrap();

    let j = serde_json::to_value(&req).unwrap();
    let content = &j["messages"][0]["content"];
    assert!(content.is_array());
    assert_eq!(content[0]["type"], json!("text"));
    assert_eq!(content[0]["text"], json!("hello"));
    assert_eq!(content[0]["cache_control"]["type"], json!("ephemeral"));
}

#[test]
fn cache_control_preserves_existing_blocks() {
    // Regression: applying cache_control to a message that already has blocks
    // must not discard those blocks.
    let req = CreateMessageRequest::builder()
        .model(Model::claude_haiku_4_5())
        .max_tokens(100)
        .message(MessageParam::user(vec![
            InputContentBlock::text("block one"),
            InputContentBlock::text("block two"),
        ]))
        .cache_control(CacheControl::ephemeral())
        .build()
        .unwrap();

    let j = serde_json::to_value(&req).unwrap();
    let content = j["messages"][0]["content"].as_array().unwrap();
    assert_eq!(content.len(), 2, "existing blocks must be preserved");
    assert_eq!(content[0]["text"], json!("block one"));
    assert_eq!(content[1]["text"], json!("block two"));
    assert_eq!(content[1]["cache_control"]["type"], json!("ephemeral"));
    assert_eq!(content[0].get("cache_control"), None);
}

// ── OutputContentBlock helpers ────────────────────────────────────────────────

#[test]
fn output_block_as_text() {
    let b = OutputContentBlock::Text { text: "hi".to_string(), citations: None };
    assert_eq!(b.as_text(), Some("hi"));

    let b2 = OutputContentBlock::Thinking {
        thinking: "hmm".to_string(),
        signature: "sig".to_string(),
    };
    assert_eq!(b2.as_text(), None);
}

#[test]
fn output_block_as_thinking() {
    let b = OutputContentBlock::Thinking {
        thinking: "deep thought".to_string(),
        signature: "sig".to_string(),
    };
    assert_eq!(b.as_thinking(), Some("deep thought"));
}

// ── Usage deserialization ─────────────────────────────────────────────────────

#[test]
fn usage_deserializes_partial() {
    let j = json!({ "input_tokens": 10, "output_tokens": 25 });
    let u: Usage = serde_json::from_value(j).unwrap();
    assert_eq!(u.input_tokens, 10);
    assert_eq!(u.output_tokens, 25);
    assert!(u.cache_creation_input_tokens.is_none());
}

#[test]
fn usage_deserializes_with_cache_fields() {
    let j = json!({
        "input_tokens": 100,
        "output_tokens": 50,
        "cache_creation_input_tokens": 80,
        "cache_read_input_tokens": 20
    });
    let u: Usage = serde_json::from_value(j).unwrap();
    assert_eq!(u.cache_creation_input_tokens, Some(80));
    assert_eq!(u.cache_read_input_tokens, Some(20));
}

// ── Message helpers ───────────────────────────────────────────────────────────

#[test]
fn message_text_concatenates_blocks() {
    use anthropic_rust_client::types::message::Message;
    let msg = Message {
        id: "msg_123".to_string(),
        message_type: "message".to_string(),
        role: Role::Assistant,
        model: "claude-opus-4-8".to_string(),
        content: vec![
            OutputContentBlock::Text { text: "Hello ".to_string(), citations: None },
            OutputContentBlock::Text { text: "world".to_string(), citations: None },
        ],
        stop_reason: Some(StopReason::EndTurn),
        stop_sequence: None,
        usage: Usage {
            input_tokens: 5,
            output_tokens: 2,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            output_tokens_details: None,
            server_tool_use: None,
            service_tier: None,
        },
    };

    assert_eq!(msg.text(), "Hello world");
    assert!(!msg.wants_tool_use());
}

#[test]
fn message_wants_tool_use() {
    use anthropic_rust_client::types::message::Message;
    let msg = Message {
        id: "msg_456".to_string(),
        message_type: "message".to_string(),
        role: Role::Assistant,
        model: "claude-opus-4-8".to_string(),
        content: vec![
            OutputContentBlock::ToolUse {
                id: "toolu_1".to_string(),
                name: "get_weather".to_string(),
                input: json!({"city": "Tokyo"}),
            },
        ],
        stop_reason: Some(StopReason::ToolUse),
        stop_sequence: None,
        usage: Usage {
            input_tokens: 20,
            output_tokens: 10,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            output_tokens_details: None,
            server_tool_use: None,
            service_tier: None,
        },
    };

    assert!(msg.wants_tool_use());
    assert_eq!(msg.tool_uses().len(), 1);
}
