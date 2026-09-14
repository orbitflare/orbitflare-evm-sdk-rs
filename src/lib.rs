#![allow(clippy::result_large_err)]

pub mod chain;
pub mod error;
pub mod retry;

#[cfg(feature = "rpc")]
mod credentials;
#[cfg(feature = "rpc")]
mod endpoint;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "ws")]
pub mod ws;

#[cfg(feature = "grpc")]
pub mod grpc;

pub use chain::{Bnb, Chain, HasGrpc, Polygon, Robinhood};
pub use error::{Error, Result};
pub use retry::RetryPolicy;

#[cfg(feature = "rpc")]
pub use rpc::{Block, NitroBlockExt, RpcClient, RpcClientBuilder, Transaction, TransactionReceipt};

#[cfg(feature = "ws")]
pub use ws::{WsClient, WsClientBuilder, WsSubscription};

#[cfg(feature = "grpc")]
pub use grpc::{PolygonGrpcClient, PolygonGrpcClientBuilder};

#[cfg(feature = "grpc")]
pub use orbitflare_polygon_proto as proto;

#[cfg(feature = "rpc")]
pub use alloy_network::{ReceiptResponse, TransactionResponse};

#[cfg(feature = "rpc")]
pub use alloy_primitives::{self as primitives, Address, B256, Bytes, U256};

#[cfg(feature = "rpc")]
pub use alloy_rpc_types_eth::{
    self as rpc_types, BlockNumberOrTag, FeeHistory, Filter, Header, Log, TransactionRequest,
};

#[cfg(feature = "rpc")]
pub type PolygonRpcClient = RpcClient<Polygon>;
#[cfg(feature = "rpc")]
pub type BnbRpcClient = RpcClient<Bnb>;
#[cfg(feature = "rpc")]
pub type RobinhoodRpcClient = RpcClient<Robinhood>;

#[cfg(feature = "ws")]
pub type PolygonWsClient = WsClient<Polygon>;
#[cfg(feature = "ws")]
pub type BnbWsClient = WsClient<Bnb>;
#[cfg(feature = "ws")]
pub type RobinhoodWsClient = WsClient<Robinhood>;
