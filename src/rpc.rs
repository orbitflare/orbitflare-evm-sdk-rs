use std::marker::PhantomData;
use std::time::Duration;

use alloy_network::{AnyReceiptEnvelope, AnyTxEnvelope};
use alloy_primitives::{Address, B256, Bytes, U64, U128};
use alloy_rpc_types_eth::{BlockNumberOrTag, FeeHistory, Filter, Log, TransactionRequest};
use reqwest::Client;
use serde_json::{Value, json};
use tracing::Instrument;

use crate::chain::Chain;
use crate::endpoint::EndpointSet;
use crate::error::{Error, Result, SanitizedUrl};
use crate::retry::RetryPolicy;

pub type Transaction = alloy_rpc_types_eth::Transaction<AnyTxEnvelope>;
pub type TransactionReceipt = alloy_rpc_types_eth::TransactionReceipt<AnyReceiptEnvelope<Log>>;
pub type Block = alloy_serde::WithOtherFields<alloy_rpc_types_eth::Block<Transaction>>;

pub trait NitroBlockExt {
    fn l1_block_number(&self) -> Option<u64>;
    fn send_root(&self) -> Option<B256>;
    fn send_count(&self) -> Option<u64>;
}

impl NitroBlockExt for Block {
    fn l1_block_number(&self) -> Option<u64> {
        other_u64(self, "l1BlockNumber")
    }

    fn send_root(&self) -> Option<B256> {
        self.other.get("sendRoot")?.as_str()?.parse().ok()
    }

    fn send_count(&self) -> Option<u64> {
        other_u64(self, "sendCount")
    }
}

fn other_u64(block: &Block, key: &str) -> Option<u64> {
    let v = block.other.get(key)?;
    match v.as_str() {
        Some(s) => u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok(),
        None => v.as_u64(),
    }
}

pub struct RpcClient<C: Chain> {
    http: Client,
    endpoints: EndpointSet,
    api_key: Option<String>,
    block_tag: BlockNumberOrTag,
    retry: RetryPolicy,
    _chain: PhantomData<fn() -> C>,
}

impl<C: Chain> RpcClient<C> {
    pub fn builder() -> RpcClientBuilder<C> {
        RpcClientBuilder::new()
    }

    pub fn chain_id_const() -> u64 {
        C::CHAIN_ID
    }

    pub fn block_tag(&self) -> BlockNumberOrTag {
        self.block_tag
    }

    pub async fn request(&self, method: &str, params: Value) -> Result<Value> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        self.execute(&body, method).await
    }

    pub async fn request_raw(&self, body: &str) -> Result<Value> {
        let parsed: Value = serde_json::from_str(body)?;
        let method = parsed
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("raw");
        self.execute(&parsed, method).await
    }

    pub async fn get_block_number(&self) -> Result<u64> {
        let r = self.request("eth_blockNumber", json!([])).await?;
        Ok(decode::<U64>(r, "block number")?.to())
    }

    pub async fn get_chain_id(&self) -> Result<u64> {
        let r = self.request("eth_chainId", json!([])).await?;
        Ok(decode::<U64>(r, "chain id")?.to())
    }

    pub async fn get_balance(&self, address: Address) -> Result<alloy_primitives::U256> {
        let r = self
            .request("eth_getBalance", json!([address, self.block_tag]))
            .await?;
        decode(r, "balance")
    }

    pub async fn get_transaction_count(&self, address: Address) -> Result<u64> {
        let r = self
            .request("eth_getTransactionCount", json!([address, self.block_tag]))
            .await?;
        Ok(decode::<U64>(r, "transaction count")?.to())
    }

    pub async fn get_gas_price(&self) -> Result<u128> {
        let r = self.request("eth_gasPrice", json!([])).await?;
        Ok(decode::<U128>(r, "gas price")?.to())
    }

    pub async fn max_priority_fee_per_gas(&self) -> Result<u128> {
        let r = self.request("eth_maxPriorityFeePerGas", json!([])).await?;
        Ok(decode::<U128>(r, "max priority fee")?.to())
    }

    pub async fn get_block_by_number(
        &self,
        block: impl Into<BlockNumberOrTag>,
        full_transactions: bool,
    ) -> Result<Option<Block>> {
        let r = self
            .request(
                "eth_getBlockByNumber",
                json!([block.into(), full_transactions]),
            )
            .await?;
        if r.is_null() {
            Ok(None)
        } else {
            Ok(Some(decode(r, "block")?))
        }
    }

    pub async fn get_block_by_hash(
        &self,
        hash: B256,
        full_transactions: bool,
    ) -> Result<Option<Block>> {
        let r = self
            .request("eth_getBlockByHash", json!([hash, full_transactions]))
            .await?;
        if r.is_null() {
            Ok(None)
        } else {
            Ok(Some(decode(r, "block")?))
        }
    }

    pub async fn get_transaction_by_hash(&self, hash: B256) -> Result<Option<Transaction>> {
        let r = self
            .request("eth_getTransactionByHash", json!([hash]))
            .await?;
        if r.is_null() {
            Ok(None)
        } else {
            Ok(Some(decode(r, "transaction")?))
        }
    }

    pub async fn get_transaction_receipt(&self, hash: B256) -> Result<Option<TransactionReceipt>> {
        let r = self
            .request("eth_getTransactionReceipt", json!([hash]))
            .await?;
        if r.is_null() {
            Ok(None)
        } else {
            Ok(Some(decode(r, "receipt")?))
        }
    }

    pub async fn get_logs(&self, filter: &Filter) -> Result<Vec<Log>> {
        let r = self.request("eth_getLogs", json!([filter])).await?;
        decode(r, "logs")
    }

    pub async fn get_code(&self, address: Address) -> Result<Bytes> {
        let r = self
            .request("eth_getCode", json!([address, self.block_tag]))
            .await?;
        decode(r, "code")
    }

    pub async fn get_storage_at(&self, address: Address, slot: B256) -> Result<B256> {
        let r = self
            .request("eth_getStorageAt", json!([address, slot, self.block_tag]))
            .await?;
        decode(r, "storage")
    }

    pub async fn call(&self, request: &TransactionRequest) -> Result<Bytes> {
        let r = self
            .request("eth_call", json!([request, self.block_tag]))
            .await?;
        decode(r, "call result")
    }

    pub async fn estimate_gas(&self, request: &TransactionRequest) -> Result<u64> {
        let r = self.request("eth_estimateGas", json!([request])).await?;
        Ok(decode::<U64>(r, "gas estimate")?.to())
    }

    pub async fn send_raw_transaction(&self, raw_tx: &[u8]) -> Result<B256> {
        let r = self
            .request(
                "eth_sendRawTransaction",
                json!([Bytes::copy_from_slice(raw_tx)]),
            )
            .await?;
        decode(r, "transaction hash")
    }

    pub async fn fee_history(
        &self,
        block_count: u64,
        newest_block: impl Into<BlockNumberOrTag>,
        reward_percentiles: &[f64],
    ) -> Result<FeeHistory> {
        let r = self
            .request(
                "eth_feeHistory",
                json!([
                    U64::from(block_count),
                    newest_block.into(),
                    reward_percentiles
                ]),
            )
            .await?;
        decode(r, "fee history")
    }

    async fn execute(&self, body: &Value, method: &str) -> Result<Value> {
        let mut last_err = None;
        let mut tried = 0;

        while tried < self.endpoints.len() {
            let idx = self.endpoints.pick();
            let url = self.endpoints.get(idx);
            let mut attempt = 0u32;
            tried += 1;

            loop {
                attempt += 1;
                let span = tracing::debug_span!(
                    "rpc",
                    chain_id = C::CHAIN_ID,
                    method,
                    endpoint = %SanitizedUrl(&url),
                    attempt,
                );

                match self.post(&url, body).instrument(span).await {
                    Ok(result) => {
                        self.endpoints.mark_success(idx);
                        return Ok(result);
                    }
                    Err(e) => {
                        let retryable = e.is_retryable();

                        if retryable && self.retry.has_attempts_left(attempt) {
                            let delay = e
                                .retry_after()
                                .unwrap_or_else(|| self.retry.delay_for_attempt(attempt));
                            tracing::warn!(
                                method,
                                attempt,
                                delay_ms = delay.as_millis() as u64,
                                error = %e,
                                "retrying",
                            );
                            tokio::time::sleep(delay).await;
                            continue;
                        }

                        self.endpoints.mark_failure(idx);
                        tracing::warn!(
                            method,
                            endpoint = %SanitizedUrl(&url),
                            error = %e,
                            "failing over",
                        );
                        last_err = Some(e.with_endpoint(&url));
                        break;
                    }
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| Error::Transport("all endpoints exhausted".to_string().into())))
    }

    fn resolve_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("ORBITFLARE_LICENSE_KEY").ok())
    }

    async fn post(&self, url: &str, body: &Value) -> Result<Value> {
        let auth_url = match self.resolve_api_key() {
            Some(key) => crate::credentials::apply_api_key(url, &key),
            None => url.to_string(),
        };

        let resp = self
            .http
            .post(&auth_url)
            .header("Content-Type", "application/json")
            .json(body)
            .send()
            .await?;

        let status = resp.status();

        if status.as_u16() == 429 {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(Duration::from_secs);
            let _ = resp.text().await;
            return Err(Error::RateLimited { retry_after });
        }

        let text = resp.text().await?;

        if status.is_server_error() {
            return Err(Error::transport(HttpError(status.as_u16(), text)));
        }

        if let Ok(parsed) = serde_json::from_str::<Value>(&text) {
            if let Some(error) = parsed.get("error") {
                let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
                let msg = error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown rpc error");
                return Err(Error::Rpc {
                    code,
                    message: msg.to_string(),
                });
            }
            if status.is_success() {
                return Ok(parsed["result"].clone());
            }
        }

        Err(Error::Rpc {
            code: -(status.as_u16() as i64),
            message: text.trim().to_string(),
        })
    }
}

fn decode<T: serde::de::DeserializeOwned>(value: Value, what: &str) -> Result<T> {
    serde_json::from_value(value).map_err(|e| Error::Serialization(format!("decoding {what}: {e}")))
}

pub struct RpcClientBuilder<C: Chain> {
    url: Option<String>,
    fallbacks: Vec<String>,
    api_key: Option<String>,
    block_tag: BlockNumberOrTag,
    retry: RetryPolicy,
    timeout: Duration,
    _chain: PhantomData<fn() -> C>,
}

impl<C: Chain> Default for RpcClientBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Chain> RpcClientBuilder<C> {
    pub fn new() -> Self {
        Self {
            url: None,
            fallbacks: Vec::new(),
            api_key: None,
            block_tag: BlockNumberOrTag::Latest,
            retry: RetryPolicy::default(),
            timeout: Duration::from_secs(30),
            _chain: PhantomData,
        }
    }

    pub fn url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    pub fn urls(mut self, urls: &[&str]) -> Self {
        if let Some((first, rest)) = urls.split_first() {
            self.url = Some(first.to_string());
            self.fallbacks = rest.iter().map(|s| s.to_string()).collect();
        }
        self
    }

    pub fn fallback_url(mut self, url: &str) -> Self {
        self.fallbacks.push(url.to_string());
        self
    }

    pub fn fallback_urls(mut self, urls: &[&str]) -> Self {
        self.fallbacks.extend(urls.iter().map(|s| s.to_string()));
        self
    }

    pub fn api_key(mut self, key: &str) -> Self {
        self.api_key = Some(key.to_string());
        self
    }

    pub fn block_tag(mut self, block_tag: impl Into<BlockNumberOrTag>) -> Self {
        self.block_tag = block_tag.into();
        self
    }

    pub fn retry(mut self, policy: RetryPolicy) -> Self {
        self.retry = policy;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<RpcClient<C>> {
        let url = self
            .url
            .or_else(|| std::env::var("ORBITFLARE_RPC_URL").ok())
            .filter(|u| !u.is_empty())
            .ok_or_else(|| {
                Error::Config(
                    "no RPC URL provided. Pass .url() to the builder \
                     or set ORBITFLARE_RPC_URL in your environment"
                        .to_string(),
                )
            })?;

        let http = Client::builder()
            .timeout(self.timeout)
            .build()
            .map_err(Error::transport)?;

        let endpoints = EndpointSet::new(&url, &self.fallbacks);

        Ok(RpcClient {
            http,
            endpoints,
            api_key: self.api_key,
            block_tag: self.block_tag,
            retry: self.retry,
            _chain: PhantomData,
        })
    }
}

#[derive(Debug)]
struct HttpError(u16, String);

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP {}: {}", self.0, self.1.trim())
    }
}

impl std::error::Error for HttpError {}
