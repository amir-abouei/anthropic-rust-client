# anthropic-rust-client

The `anthropic-rust-client` crate provides access to the [Anthropic Claude API](https://platform.claude.com/docs/en/api/overview) from Rust applications.

## Installation

```toml
[dependencies]
anthropic-rust-client = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Getting started

```rust
use anthropic_rust_client::{Client, CreateMessageRequest, Model};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Reads ANTHROPIC_API_KEY from environment by default
    let client = Client::new()?;

    let message = client
        .messages()
        .create(
            CreateMessageRequest::builder()
                .model(Model::claude_opus_4_8())
                .max_tokens(1024)
                .user("Hello, Claude!")
                .build()?,
        )
        .await?;

    println!("{}", message.text());
    Ok(())
}
```

Set your API key before running:

```sh
export ANTHROPIC_API_KEY=sk-ant-...
```

Or copy `.env.example` to `.env` and fill in your values:

```sh
cp .env.example .env
```

## Supported APIs

| API | Status |
|-----|--------|
| Messages (blocking + streaming) | ✅ GA |
| Token Counting | ✅ GA |
| Models | ✅ GA |
| Message Batches | ✅ GA |
| Files | ✅ Beta |

## Requirements

- Rust **1.75** or later (edition 2021)
- Tokio async runtime

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
