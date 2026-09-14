use orbitflare_evm_sdk::PolygonWsClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = PolygonWsClient::builder().build().await?;

    let mut heads = client.new_heads_subscribe().await?;
    println!("subscribed to newHeads, waiting for blocks...");

    for _ in 0..5 {
        match heads.next().await {
            Some(header) => println!("block {} (gas used {})", header.number, header.gas_used),
            None => break,
        }
    }

    heads.unsubscribe().await;
    Ok(())
}
