use anthropic_rust_client::{Client, CreateMessageRequest, Model, ThinkingConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    let response = client
        .messages()
        .create(
            CreateMessageRequest::builder()
                .model(Model::claude_opus_4_8())
                .max_tokens(16000)
                .thinking(ThinkingConfig::enabled(10000))
                .user("Solve: A train travels 120 km at 60 km/h, then 80 km at 40 km/h. \
                       What is the average speed for the entire journey?")
                .build()?,
        )
        .await?;

    if let Some(thinking) = response.thinking() {
        println!("=== Extended Thinking ===\n{thinking}\n");
    }

    println!("=== Answer ===\n{}", response.text());
    println!(
        "\nTokens: {} input, {} output ({} thinking)",
        response.usage.input_tokens,
        response.usage.output_tokens,
        response
            .usage
            .output_tokens_details
            .as_ref()
            .and_then(|d| d.thinking_tokens)
            .unwrap_or(0),
    );

    Ok(())
}
