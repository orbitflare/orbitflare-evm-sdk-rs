use orbitflare_evm_sdk::{
    Address, Filter, PolygonRpcClient, TransactionRequest, primitives::TxKind,
    rpc_types::BlockNumberOrTag,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let client = PolygonRpcClient::builder().build()?;

    let block_number = client.get_block_number().await?;
    println!(
        "chain id {} at block {block_number}",
        client.get_chain_id().await?
    );
    println!("gas price: {} wei", client.get_gas_price().await?);
    println!(
        "max priority fee: {} wei",
        client.max_priority_fee_per_gas().await?
    );

    let block = client
        .get_block_by_number(BlockNumberOrTag::Latest, true)
        .await?
        .expect("latest block");
    println!(
        "block {}: {} txs, gas used {}",
        block.header.number,
        block.transactions.len(),
        block.header.gas_used,
    );

    let fees = client
        .fee_history(5, BlockNumberOrTag::Latest, &[25.0, 50.0, 75.0])
        .await?;
    println!(
        "fee history: {} base-fee samples",
        fees.base_fee_per_gas.len()
    );

    let wpol: Address = "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270".parse()?;
    let call = TransactionRequest {
        to: Some(TxKind::Call(wpol)),
        input: alloy_input("0x95d89b41"),
        ..Default::default()
    };
    let symbol = client.call(&call).await?;
    println!("WPOL symbol() returned {} bytes", symbol.len());
    println!(
        "WPOL code size: {} bytes",
        client.get_code(wpol).await?.len()
    );

    let logs = client
        .get_logs(
            &Filter::new()
                .from_block(block_number.saturating_sub(5))
                .to_block(block_number),
        )
        .await?;
    println!("logs in last 6 blocks: {}", logs.len());

    if let Some(tx_hash) = block.transactions.hashes().next() {
        if let Some(tx) = client.get_transaction_by_hash(tx_hash).await? {
            println!("tx {tx_hash:#x} decoded ok (block {:?})", tx.block_number);
        }
        if let Some(receipt) = client.get_transaction_receipt(tx_hash).await? {
            use orbitflare_evm_sdk::ReceiptResponse;
            println!(
                "receipt: status={} gas_used={}",
                receipt.status(),
                receipt.gas_used,
            );
        }
    }

    Ok(())
}

fn alloy_input(hex: &str) -> alloy_rpc_types_eth::TransactionInput {
    let bytes: orbitflare_evm_sdk::Bytes = hex.parse().unwrap();
    alloy_rpc_types_eth::TransactionInput::new(bytes)
}
