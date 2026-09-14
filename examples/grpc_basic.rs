use orbitflare_evm_sdk::grpc::{BlockNumber, PolygonGrpcClient, ToAlloy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PolygonGrpcClient::builder().build()?;

    let header = client.header_by_number(BlockNumber::Latest).await?;
    println!(
        "latest block: {} (gas used {})",
        header.number, header.gas_used
    );

    let author = client.author(header.number).await?;
    println!("author: {author}");

    let start = header.number.saturating_sub(4);
    let blocks = client.block_info_in_batch(start, header.number).await?;
    for info in blocks {
        let number = info.header.as_ref().map(|h| h.number).unwrap_or_default();
        let author = info
            .author
            .as_ref()
            .map(|a| a.to_alloy().to_string())
            .unwrap_or_default();
        println!(
            "block {number} author {author} td {}",
            info.total_difficulty
        );
    }

    Ok(())
}
