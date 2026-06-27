use anthropic_rust_client::{Client, CreateMessageRequest, Model};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    let mut stream = client
        .messages()
        .stream(
            CreateMessageRequest::builder()
                .model(Model::claude_sonnet_4_6())
                .max_tokens(512)
                .user("Count from 1 to 10, one number per line.")
                .build()?,
        )
        .await?;

    print!("Streaming: ");
    while let Some(event) = stream.next().await {
        let event = event?;
        if let Some(text) = event.as_text_delta() {
            print!("{text}");
        }
        if event.is_message_stop() {
            break;
        }
    }
    println!("\nDone.");

    Ok(())
}
