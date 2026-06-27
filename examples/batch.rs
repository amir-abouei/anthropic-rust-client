use anthropic_rust_client::{
    BatchRequestItem, Client, CreateBatchRequest, CreateMessageRequest, Model,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Build a batch of 3 independent requests.
    let items: Vec<BatchRequestItem> = ["Tokyo", "Paris", "New York"]
        .iter()
        .enumerate()
        .map(|(i, city)| {
            let req = CreateMessageRequest::builder()
                .model(Model::claude_haiku_4_5())
                .max_tokens(128)
                .user(format!("What is the population of {city}? Answer in one sentence."))
                .build()
                .expect("valid request");
            BatchRequestItem::new(format!("city-{i}"), req)
        })
        .collect();

    let batch = client
        .batches()
        .create(CreateBatchRequest::new(items))
        .await?;

    println!("Batch created: {} (status: {:?})", batch.id, batch.processing_status);
    println!("Batch expires at: {}", batch.expires_at);

    // In production, poll until batch.is_ended() then fetch results.
    // Here we just show the polling pattern:
    //
    // loop {
    //     let batch = client.batches().get(&batch.id).await?;
    //     if batch.is_ended() {
    //         let results = client.batches().results(&batch.id).await?;
    //         for result in results {
    //             println!("{}: {:?}", result.custom_id, result.result);
    //         }
    //         break;
    //     }
    //     tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    // }

    Ok(())
}
