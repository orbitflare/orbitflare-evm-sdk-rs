use orbitflare_evm_sdk::{Address, Chain, RpcClient};

struct Base;

impl Chain for Base {
    const CHAIN_ID: u64 = 8453;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::<Base>::builder()
        .url("https://mainnet.base.org")
        .build()?;

    println!("declared chain id: {}", RpcClient::<Base>::chain_id_const());
    println!("reported chain id: {}", client.get_chain_id().await?);
    println!("block number:      {}", client.get_block_number().await?);
    println!("gas price:         {} wei", client.get_gas_price().await?);

    let weth: Address = "0x4200000000000000000000000000000000000006".parse()?;
    println!(
        "WETH code size:    {} bytes",
        client.get_code(weth).await?.len()
    );

    Ok(())
}
