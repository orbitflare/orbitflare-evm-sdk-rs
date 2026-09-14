use orbitflare_evm_sdk::{Address, PolygonRpcClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PolygonRpcClient::builder().build()?;

    println!("chain id: {}", client.get_chain_id().await?);
    println!("block number: {}", client.get_block_number().await?);
    println!("gas price: {} wei", client.get_gas_price().await?);

    let wmatic: Address = "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270".parse()?;
    println!(
        "WPOL balance of self: {}",
        client.get_balance(wmatic).await?
    );

    Ok(())
}
