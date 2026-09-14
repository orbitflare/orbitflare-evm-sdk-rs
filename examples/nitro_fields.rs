use orbitflare_evm_sdk::{NitroBlockExt, RobinhoodRpcClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RobinhoodRpcClient::builder()
        .url("https://robinhood.rpc.orbitflare.com")
        .build()?;

    let block = client
        .get_block_by_number(client.block_tag(), false)
        .await?
        .ok_or("no block returned")?;

    println!("l2 block number: {}", block.header.number);
    println!("l1 block number: {:?}", block.l1_block_number());
    println!("send root:       {:?}", block.send_root());
    println!("send count:      {:?}", block.send_count());

    Ok(())
}
