use alloy_primitives::U256;
use orbitflare_evm_sdk::grpc::{BlockNumber, PolygonGrpcClient, ToAlloy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PolygonGrpcClient::builder().build()?;

    let header = client.header_by_number(BlockNumber::Latest).await?;
    println!(
        "latest header: number={} time={} gas_used={}/{} base_fee={}",
        header.number,
        header.time,
        header.gas_used,
        header.gas_limit,
        U256::from_be_slice(&header.base_fee),
    );
    if let Some(parent) = &header.parent_hash {
        println!("parent hash: {}", parent.to_alloy());
    }
    if let Some(state_root) = &header.state_root {
        println!("state root: {}", state_root.to_alloy());
    }

    let block = client.block_by_number(header.number).await?;
    let block_number = block.header.as_ref().map(|h| h.number).unwrap_or_default();
    println!("block_by_number({block_number}) ok");

    let author = client.author(header.number).await?;
    println!("author({}) = {author}", header.number);

    let td = client.td_by_number(header.number).await?;
    println!("td_by_number({}) = {td}", header.number);

    if let Some(parent) = &header.parent_hash {
        let parent_hash = parent.to_alloy();
        match client.td_by_hash(parent_hash).await {
            Ok(td) => println!("td_by_hash(parent) = {td}"),
            Err(e) => println!("td_by_hash(parent): {e}"),
        }
        match client.bor_block_receipt(parent_hash).await {
            Ok(receipt) => println!("bor_block_receipt(parent): logs={}", receipt.logs.len()),
            Err(e) => println!("bor_block_receipt(parent): {e}"),
        }
    }

    let start = header.number.saturating_sub(9);
    let root = client.root_hash(start, header.number).await?;
    println!("root_hash({start}..{}) = {root}", header.number);

    match client
        .vote_on_hash(start, header.number, root.clone(), "probe")
        .await
    {
        Ok(vote) => println!("vote_on_hash = {vote}"),
        Err(e) => println!("vote_on_hash: {e}"),
    }

    match client.start_block_heimdall_span_id(header.number).await {
        Ok(span) => println!(
            "heimdall span: start_block={} span_id={}",
            span.start_block, span.heimdall_span_id
        ),
        Err(e) => println!("heimdall span: not available on this endpoint ({e})"),
    }

    let blocks = client.block_info_in_batch(start, header.number).await?;
    println!("block_info_in_batch({start}..{}):", header.number);
    for info in &blocks {
        let number = info.header.as_ref().map(|h| h.number).unwrap_or_default();
        let author = info
            .author
            .as_ref()
            .map(|a| a.to_alloy().to_string())
            .unwrap_or_default();
        println!("  block {number} author {author}");
    }

    if let Ok(tx_hash) = std::env::var("EXAMPLE_TX_HASH") {
        let hash = tx_hash.parse()?;
        let receipt = client.transaction_receipt(hash).await?;
        println!(
            "receipt: status={} gas_used={} logs={}",
            receipt.status,
            receipt.gas_used,
            receipt.logs.len()
        );
    }

    Ok(())
}
