<p align="center">
  <img src="https://raw.githubusercontent.com/orbitflare/orbitflare-evm-sdk-rs/main/assets/banner.png" alt="orbitflare-evm-sdk" width="100%">
</p>

<p align="center">
  <a href="https://crates.io/crates/orbitflare-evm-sdk"><img src="https://img.shields.io/crates/v/orbitflare-evm-sdk.svg?style=flat-square&color=3CAB9C&labelColor=041815" alt="crates.io"></a>
  <a href="https://docs.orbitflare.com/sdk/overview"><img src="https://img.shields.io/badge/docs-orbitflare.com-3CAB9C?style=flat-square&labelColor=041815" alt="Documentation"></a>
</p>

# orbitflare-evm-sdk

Rust SDK for OrbitFlare's EVM chains by [OrbitFlare](https://orbitflare.com) - RPC, WebSocket, and gRPC clients for Polygon, BNB Smart Chain, and Robinhood Chain, plus any EVM chain you define yourself.

Built on [alloy](https://github.com/alloy-rs/alloy) types (`Address`, `U256`, `B256`, `Filter`, `TransactionRequest`, typed blocks, transactions, receipts, and logs), with OrbitFlare's own transport: endpoint failover, retry with backoff, and self-healing WebSocket subscriptions.

## Supported chains

| Chain | Chain ID | Alias |
|---|---|---|
| Polygon | 137 | `PolygonRpcClient`, `PolygonWsClient` |
| BNB Smart Chain | 56 | `BnbRpcClient`, `BnbWsClient` |
| Robinhood Chain | 4663 | `RobinhoodRpcClient`, `RobinhoodWsClient` |
| Your own | any | `RpcClient<C>` where `impl Chain for C` |

The clients are generic over the chain (`RpcClient<C>`, `WsClient<C>`); the aliases above are convenience shorthands. gRPC is currently Polygon only (Bor).

## Install

```bash
cargo add orbitflare-evm-sdk
```

Only the RPC client is enabled by default. Enable what you need:

```bash
cargo add orbitflare-evm-sdk --features ws
cargo add orbitflare-evm-sdk --features grpc
cargo add orbitflare-evm-sdk --features all
```

Or in your `Cargo.toml`:

```toml
[dependencies]
orbitflare-evm-sdk = { version = "0.1.0", features = ["all"] }
```

## RPC

```rust
use orbitflare_evm_sdk::{primitives::address, PolygonRpcClient, Result};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let client = PolygonRpcClient::builder()
        .url("https://ams.poly.rpc.orbitflare.com")
        .api_key("ORBIT-...")
        .build()?;

    let block = client.get_block_number().await?;
    let gas_price = client.get_gas_price().await?;
    let balance = client
        .get_balance(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .await?;

    let syncing = client.request("eth_syncing", json!([])).await?;

    Ok(())
}
```

There are no baked-in default endpoints: set the URL with `.url()` on the builder or via an environment variable (see [Endpoints](#endpoints)), and the API key with `.api_key()` or `ORBITFLARE_LICENSE_KEY`.

### Custom chains

Any EVM chain works: implement `Chain` for a marker type and point a generic `RpcClient<C>` at its URL. `CHAIN_ID` is metadata (available as `RpcClient::<C>::chain_id_const()`); it does not gate requests.

```rust
use orbitflare_evm_sdk::{Chain, RpcClient};

struct Base;

impl Chain for Base {
    const CHAIN_ID: u64 = 8453;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RpcClient::<Base>::builder()
        .url("https://mainnet.base.org")
        .build()?;

    println!("{}", client.get_block_number().await?);
    Ok(())
}
```

### Typed helpers

All addresses, hashes, and quantities are alloy types. Common ones are re-exported at the crate root; the full crates are available as `orbitflare_evm_sdk::primitives` (alloy-primitives) and `orbitflare_evm_sdk::rpc_types` (alloy-rpc-types-eth).

| Method | Returns |
|---|---|
| `get_block_number()` | `u64` |
| `get_chain_id()` | `u64` |
| `get_balance(Address)` | `U256` wei |
| `get_transaction_count(Address)` | `u64` nonce |
| `get_gas_price()` | `u128` wei |
| `max_priority_fee_per_gas()` | `u128` wei |
| `get_block_by_number(impl Into<BlockNumberOrTag>, full_txs)` | `Option<Block>` |
| `get_block_by_hash(B256, full_txs)` | `Option<Block>` |
| `get_transaction_by_hash(B256)` | `Option<Transaction>` |
| `get_transaction_receipt(B256)` | `Option<TransactionReceipt>` |
| `get_logs(&Filter)` | `Vec<Log>` |
| `get_code(Address)` | `Bytes` |
| `get_storage_at(Address, B256)` | `B256` |
| `call(&TransactionRequest)` | `Bytes` |
| `estimate_gas(&TransactionRequest)` | `u64` |
| `send_raw_transaction(&[u8])` | `B256` tx hash |
| `fee_history(blocks, newest, percentiles)` | `FeeHistory` |
| `request(method, params)` | Any RPC method by name (`serde_json::Value`) |
| `request_raw(body)` | Raw JSON-RPC body string |

State queries (`get_balance`, `call`, ...) use the client's block tag, set via `.block_tag(BlockNumberOrTag::Finalized)` on the builder (default: `Latest`).

### Log filters

`get_logs` takes alloy's `Filter` directly:

```rust
use orbitflare_evm_sdk::{primitives::{address, b256}, BlockNumberOrTag, Filter};

let transfer_topic =
    b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

let filter = Filter::new()
    .from_block(60_000_000u64)
    .to_block(BlockNumberOrTag::Latest)
    .address(address!("0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270"))
    .event_signature(transfer_topic);

let logs = client.get_logs(&filter).await?;
```

### Sending transactions

Build and sign with alloy (`alloy-signer`, `alloy-network`), then broadcast through the SDK:

```rust
let hash = client.send_raw_transaction(&signed_tx_rlp).await?;
let receipt = client.get_transaction_receipt(hash).await?;
```

## WebSocket

```rust
use orbitflare_evm_sdk::{RobinhoodWsClient, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let client = RobinhoodWsClient::builder()
        .url("wss://robinhood.rpc.orbitflare.com")
        .build()
        .await?;

    let mut heads = client.new_heads_subscribe().await?;

    while let Some(head) = heads.next().await {
        println!("{}", head.number);
    }

    Ok(())
}
```

Subscriptions are typed: `new_heads_subscribe()` yields `Header`, `logs_subscribe(&Filter)` yields `Log`, `new_pending_transactions_subscribe()` yields `B256`. All auto-resubscribe on reconnect. `next_raw()` returns the raw `serde_json::Value` payload if you need it.

## gRPC (Polygon Bor)

Polygon exposes a Bor gRPC interface for low-overhead block, header, and receipt access. Enable the `grpc` feature.

```rust
use orbitflare_evm_sdk::grpc::{BlockNumber, PolygonGrpcClient, ToAlloy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = PolygonGrpcClient::builder()
        .url("http://your-bor-grpc-endpoint:3131")
        .api_key("ORBIT-...")
        .build()?;

    let header = client.header_by_number(BlockNumber::Latest).await?;
    let author = client.author(header.number).await?;
    let batch = client.block_info_in_batch(header.number - 4, header.number).await?;

    Ok(())
}
```

| Method | Returns |
|---|---|
| `header_by_number(impl Into<BlockNumber>)` | `Header` |
| `block_by_number(impl Into<BlockNumber>)` | `Block` |
| `transaction_receipt(B256)` | `Receipt` |
| `bor_block_receipt(B256)` | `Receipt` |
| `author(impl Into<BlockNumber>)` | `Address` |
| `td_by_hash(B256)` / `td_by_number(...)` | `u64` total difficulty |
| `root_hash(start, end)` | `String` |
| `block_info_in_batch(start, end)` | `Vec<BlockInfo>` |

gRPC runs over plaintext HTTP/2. Authenticate with a token via `.api_key()` (sent as `x-token`) or use IP whitelisting. H160/H256 values convert to alloy `Address`/`B256` with the `ToAlloy` trait.

## Arbitrum / Nitro block fields

Robinhood Chain is Arbitrum Nitro, so its blocks carry extra fields (`l1BlockNumber`, `sendRoot`, `sendCount`) that standard EVM block types drop. The SDK preserves them and exposes typed accessors via `NitroBlockExt`.

```rust
use orbitflare_evm_sdk::{NitroBlockExt, RobinhoodRpcClient};

let client = RobinhoodRpcClient::builder()
    .url("https://robinhood.rpc.orbitflare.com")
    .build()?;

let block = client
    .get_block_by_number(client.block_tag(), false)
    .await?
    .unwrap();

let l1 = block.l1_block_number();
let send_root = block.send_root();
let send_count = block.send_count();
```

Any other non-standard field is still available on `block.other`. Arbitrum precompiles (ArbSys, ArbGasInfo, ...) are reachable through `call()` like any contract.

## Authentication

The API key is appended to the endpoint URL as `?api_key=<key>`. Resolution order is `.api_key()` on the builder, then the `ORBITFLARE_LICENSE_KEY` environment variable.

To use a third-party provider or a different auth format, bake the full authenticated URL into `.url()` and do not call `.api_key()`:

```rust
let client = RpcClient::<Base>::builder()
    .url("https://your-provider.example/v2/YOUR_KEY_IN_PATH")
    .build()?;
```

Header-based auth is not applied by the SDK; put the credential in the URL.

## Endpoint failover

All clients support multiple endpoints with automatic failover and health tracking.

```rust
let client = PolygonRpcClient::builder()
    .url("https://ams.poly.rpc.orbitflare.com")
    .fallback_urls(&["https://your-secondary-endpoint"])
    .build()?;
```

Failing endpoints are quarantined with exponential cooldown (10s, 20s, 40s, max 60s) and automatically retried once the cooldown expires. Healthy endpoints are always preferred.

## Retry

RPC calls retry on transient errors (5xx, 429, connection resets, JSON-RPC error code -32005) with exponential backoff before failing over to the next endpoint. 429 responses with a `Retry-After` header are respected.

```rust
use orbitflare_evm_sdk::RetryPolicy;
use std::time::Duration;

let client = PolygonRpcClient::builder()
    .url("https://ams.poly.rpc.orbitflare.com")
    .retry(RetryPolicy {
        initial_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(15),
        multiplier: 2.0,
        max_attempts: 5,
    })
    .build()?;
```

The WebSocket client uses active ping/pong to detect dead connections (default: ping every 10s, 3 missed pongs before reconnect, both configurable via `.ping_interval_secs()` and `.max_missed_pongs()`). It reconnects automatically and re-subscribes all active subscriptions after reconnecting.

## Endpoints

There are no default endpoints. Set the URL per client with `.url()`, or via an environment variable. Resolution order: `.url()` on the builder, then the environment variable.

| Chain | RPC | WebSocket |
|---|---|---|
| Polygon | `https://ams.poly.rpc.orbitflare.com` | `wss://ams.poly.rpc.orbitflare.com` |
| BNB Smart Chain | `https://bsc.rpc.orbitflare.com` | `wss://bsc.rpc.orbitflare.com` |
| Robinhood Chain | `https://robinhood.rpc.orbitflare.com` | `wss://robinhood.rpc.orbitflare.com` |

Use the exact endpoints from your OrbitFlare dashboard.

## Environment variables

| Variable | Used by | Purpose |
|---|---|---|
| `ORBITFLARE_LICENSE_KEY` | RPC, WebSocket, gRPC | API key appended to endpoint URLs (`x-token` for gRPC) |
| `ORBITFLARE_RPC_URL` | RPC | Endpoint used if `.url()` is not called |
| `ORBITFLARE_WS_URL` | WebSocket | Endpoint used if `.url()` is not called |
| `ORBITFLARE_GRPC_URL` | gRPC | Endpoint used if `.url()` is not called |

## Feature flags

| Feature | Enables |
|---|---|
| `rpc` (default) | HTTP JSON-RPC client |
| `ws` | WebSocket subscription client |
| `grpc` | Polygon Bor gRPC client |
| `all` | everything above |
