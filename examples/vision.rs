use anthropic_rust_client::{
    Client, CreateMessageRequest, ImageMediaType, ImageSource, InputContentBlock, Model,
};
use base64::{Engine as _, engine::general_purpose};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // ── Example 1: image from URL ─────────────────────────────────────────────
    let response = client
        .messages()
        .create(
            CreateMessageRequest::builder()
                .model(Model::claude_opus_4_8())
                .max_tokens(1024)
                .user_with_image(
                    "Describe what you see in this image in one paragraph.",
                    ImageSource::url(
                        "https://upload.wikimedia.org/wikipedia/commons/thumb/4/47/PNG_transparency_demonstration_1.png/280px-PNG_transparency_demonstration_1.png",
                    ),
                )
                .build()?,
        )
        .await?;

    println!("URL image description:\n{}\n", response.text());

    // ── Example 2: image from base64 ─────────────────────────────────────────
    // In practice you'd read this from a file; here we use a minimal 1×1 PNG.
    let tiny_png_bytes: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82,
        0, 0, 0, 1, 0, 0, 0, 1, 8, 2, 0, 0, 0, 144, 119, 83, 222, 0,
        0, 0, 12, 73, 68, 65, 84, 8, 215, 99, 248, 15, 0, 0, 1, 1, 0,
        5, 24, 213, 78, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    let b64 = general_purpose::STANDARD.encode(tiny_png_bytes);

    let blocks = vec![
        InputContentBlock::image(ImageSource::base64(ImageMediaType::Png, b64)),
        InputContentBlock::text("What color is the dominant pixel in this image?"),
    ];

    let response = client
        .messages()
        .create(
            CreateMessageRequest::builder()
                .model(Model::claude_opus_4_8())
                .max_tokens(256)
                .message(anthropic_rust_client::MessageParam::user(blocks))
                .build()?,
        )
        .await?;

    println!("Base64 image response:\n{}", response.text());

    Ok(())
}
