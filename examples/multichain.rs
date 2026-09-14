use orbitflare_evm_sdk::{BnbRpcClient, PolygonRpcClient, RobinhoodRpcClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let poly = PolygonRpcClient::builder()
        .url("https://ams.poly.rpc.orbitflare.com")
        .build()?;
    println!(
        "polygon:   chain {} block {}",
        poly.get_chain_id().await?,
        poly.get_block_number().await?
    );

    let bnb = BnbRpcClient::builder()
        .url("https://bsc.rpc.orbitflare.com")
        .build()?;
    println!(
        "bnb:       chain {} block {}",
        bnb.get_chain_id().await?,
        bnb.get_block_number().await?
    );

    let rh = RobinhoodRpcClient::builder()
        .url("https://robinhood.rpc.orbitflare.com")
        .build()?;
    println!(
        "robinhood: chain {} block {}",
        rh.get_chain_id().await?,
        rh.get_block_number().await?
    );

    Ok(())
}
