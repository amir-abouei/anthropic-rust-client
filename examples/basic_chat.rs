use anthropic_rust_client::{Client, CreateMessageRequest, Model};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    let message = client
        .messages()
        .create(
            CreateMessageRequest::builder()
                .model(Model::claude_opus_4_8())
                .max_tokens(1024)
                .user("What is the capital of France? Answer in one sentence.")
                .build()?,
        )
        .await?;

    println!("Response: {}", message.text());
    println!("Tokens used: {} in, {} out", message.usage.input_tokens, message.usage.output_tokens);

    Ok(())
}
