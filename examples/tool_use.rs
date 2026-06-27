use anthropic_rust_client::{
    Client, CreateMessageRequest, InputContentBlock, MessageContent, MessageParam, Model,
    OutputContentBlock, Role, Tool, ToolChoice, ToolResultContent,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    let weather_tool = Tool::custom(
        "get_weather",
        "Get the current temperature for a city.",
        json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "City name, e.g. 'Tokyo'"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit"
                }
            },
            "required": ["city"]
        }),
    );

    // ── Turn 1: send query, Claude may request tool use ───────────────────────
    let req = CreateMessageRequest::builder()
        .model(Model::claude_opus_4_8())
        .max_tokens(1024)
        .tool(weather_tool.clone())
        .tool_choice(ToolChoice::Auto)
        .user("What's the weather like in Tokyo right now?")
        .build()?;

    let response = client.messages().create(req).await?;

    if !response.wants_tool_use() {
        println!("No tool use requested: {}", response.text());
        return Ok(());
    }

    // ── Turn 2: provide tool results ──────────────────────────────────────────
    // Build the assistant turn from the response content.
    let assistant_blocks: Vec<InputContentBlock> = response
        .content
        .iter()
        .filter_map(|b| match b {
            OutputContentBlock::Text { text, .. } => {
                Some(InputContentBlock::text(text.clone()))
            }
            OutputContentBlock::ToolUse { id, name, input } => {
                Some(InputContentBlock::ToolUse {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                    cache_control: None,
                })
            }
            _ => None,
        })
        .collect();

    let mut messages: Vec<MessageParam> = vec![
        MessageParam::user("What's the weather like in Tokyo right now?"),
        MessageParam {
            role: Role::Assistant,
            content: MessageContent::Blocks(assistant_blocks),
        },
    ];

    for block in &response.content {
        if let OutputContentBlock::ToolUse { id, name, input } = block {
            println!("Claude called tool '{name}' with input: {input}");

            // Simulate the tool response.
            let result = json!({ "temperature": 22, "unit": "celsius", "condition": "cloudy" });

            messages.push(MessageParam {
                role: Role::User,
                content: MessageContent::Blocks(vec![
                    InputContentBlock::ToolResult {
                        tool_use_id: id.clone(),
                        content: Some(ToolResultContent::Text(result.to_string())),
                        is_error: None,
                        cache_control: None,
                    },
                ]),
            });
        }
    }

    let final_req = CreateMessageRequest::builder()
        .model(Model::claude_opus_4_8())
        .max_tokens(1024)
        .tool(weather_tool)
        .messages(messages)
        .build()?;

    let final_response = client.messages().create(final_req).await?;
    println!("\nFinal answer: {}", final_response.text());

    Ok(())
}
