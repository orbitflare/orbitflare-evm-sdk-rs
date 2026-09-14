mod convert;

pub use convert::{ToAlloy, ToProto};

use std::time::Duration;

use alloy_primitives::{Address, B256};
use orbitflare_polygon_proto::bor::bor_api_client::BorApiClient;
use orbitflare_polygon_proto::bor::{
    GetAuthorRequest, GetBlockByNumberRequest, GetBlockInfoInBatchRequest,
    GetHeaderByNumberRequest, GetRootHashRequest, GetStartBlockHeimdallSpanIdRequest,
    GetStartBlockHeimdallSpanIdResponse, GetTdByHashRequest, GetTdByNumberRequest,
    GetVoteOnHashRequest, ReceiptRequest,
};
use tonic::metadata::Ascii;
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};

use crate::{Error, Result};

pub use orbitflare_polygon_proto::bor::{Block, BlockInfo, Header, Log, Receipt};

const TOKEN_HEADER: &str = "x-token";

#[derive(Debug, Clone)]
pub enum BlockNumber {
    Latest,
    Number(u64),
    Raw(String),
}

impl BlockNumber {
    fn as_param(&self) -> String {
        match self {
            BlockNumber::Latest => "latest".to_string(),
            BlockNumber::Number(n) => format!("0x{n:x}"),
            BlockNumber::Raw(s) => s.clone(),
        }
    }
}

impl From<u64> for BlockNumber {
    fn from(n: u64) -> Self {
        BlockNumber::Number(n)
    }
}

#[derive(Clone)]
pub struct AuthInterceptor {
    token: Option<MetadataValue<Ascii>>,
}

impl Interceptor for AuthInterceptor {
    fn call(
        &mut self,
        mut request: tonic::Request<()>,
    ) -> std::result::Result<tonic::Request<()>, tonic::Status> {
        if let Some(token) = &self.token {
            request.metadata_mut().insert(TOKEN_HEADER, token.clone());
        }
        Ok(request)
    }
}

pub struct PolygonGrpcClientBuilder {
    url: Option<String>,
    api_key: Option<String>,
    timeout: Duration,
}

impl Default for PolygonGrpcClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PolygonGrpcClientBuilder {
    pub fn new() -> Self {
        Self {
            url: None,
            api_key: None,
            timeout: Duration::from_secs(30),
        }
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<PolygonGrpcClient> {
        let url = self
            .url
            .or_else(|| std::env::var("ORBITFLARE_GRPC_URL").ok())
            .filter(|u| !u.is_empty())
            .ok_or_else(|| {
                Error::Config(
                    "no gRPC URL provided. Pass .url() to the builder \
                     or set ORBITFLARE_GRPC_URL in your environment"
                        .to_string(),
                )
            })?;

        let token = self
            .api_key
            .or_else(|| std::env::var("ORBITFLARE_LICENSE_KEY").ok())
            .map(|key| {
                key.parse::<MetadataValue<Ascii>>()
                    .map_err(|_| Error::Auth("api key is not valid header ascii".to_string()))
            })
            .transpose()?;

        let mut endpoint = Endpoint::from_shared(url.clone())?.timeout(self.timeout);
        if url.starts_with("https://") {
            endpoint = endpoint.tls_config(ClientTlsConfig::new().with_native_roots())?;
        }
        let channel = endpoint.connect_lazy();

        let inner = BorApiClient::with_interceptor(channel, AuthInterceptor { token });
        Ok(PolygonGrpcClient { inner })
    }
}

#[derive(Clone)]
pub struct PolygonGrpcClient {
    inner: BorApiClient<InterceptedService<Channel, AuthInterceptor>>,
}

impl PolygonGrpcClient {
    pub fn builder() -> PolygonGrpcClientBuilder {
        PolygonGrpcClientBuilder::new()
    }

    pub async fn header_by_number(&self, number: impl Into<BlockNumber>) -> Result<Header> {
        let request = GetHeaderByNumberRequest {
            number: number.into().as_param(),
        };
        let response = self.inner.clone().header_by_number(request).await?;
        response
            .into_inner()
            .header
            .ok_or(Error::MissingField("header"))
    }

    pub async fn block_by_number(&self, number: impl Into<BlockNumber>) -> Result<Block> {
        let request = GetBlockByNumberRequest {
            number: number.into().as_param(),
        };
        let response = self.inner.clone().block_by_number(request).await?;
        response
            .into_inner()
            .block
            .ok_or(Error::MissingField("block"))
    }

    pub async fn transaction_receipt(&self, hash: B256) -> Result<Receipt> {
        let request = ReceiptRequest {
            hash: Some(hash.to_proto()),
        };
        let response = self.inner.clone().transaction_receipt(request).await?;
        response
            .into_inner()
            .receipt
            .ok_or(Error::MissingField("receipt"))
    }

    pub async fn bor_block_receipt(&self, hash: B256) -> Result<Receipt> {
        let request = ReceiptRequest {
            hash: Some(hash.to_proto()),
        };
        let response = self.inner.clone().bor_block_receipt(request).await?;
        response
            .into_inner()
            .receipt
            .ok_or(Error::MissingField("receipt"))
    }

    pub async fn author(&self, number: impl Into<BlockNumber>) -> Result<Address> {
        let request = GetAuthorRequest {
            number: number.into().as_param(),
        };
        let response = self.inner.clone().get_author(request).await?;
        let author = response
            .into_inner()
            .author
            .ok_or(Error::MissingField("author"))?;
        Ok(author.to_alloy())
    }

    pub async fn td_by_hash(&self, hash: B256) -> Result<u64> {
        let request = GetTdByHashRequest {
            hash: Some(hash.to_proto()),
        };
        let response = self.inner.clone().get_td_by_hash(request).await?;
        Ok(response.into_inner().total_difficulty)
    }

    pub async fn td_by_number(&self, number: impl Into<BlockNumber>) -> Result<u64> {
        let request = GetTdByNumberRequest {
            number: number.into().as_param(),
        };
        let response = self.inner.clone().get_td_by_number(request).await?;
        Ok(response.into_inner().total_difficulty)
    }

    pub async fn root_hash(&self, start_block: u64, end_block: u64) -> Result<String> {
        let request = GetRootHashRequest {
            start_block_number: start_block,
            end_block_number: end_block,
        };
        let response = self.inner.clone().get_root_hash(request).await?;
        Ok(response.into_inner().root_hash)
    }

    pub async fn vote_on_hash(
        &self,
        start_block: u64,
        end_block: u64,
        hash: impl Into<String>,
        milestone_id: impl Into<String>,
    ) -> Result<bool> {
        let request = GetVoteOnHashRequest {
            start_block_number: start_block,
            end_block_number: end_block,
            hash: hash.into(),
            milestone_id: milestone_id.into(),
        };
        let response = self.inner.clone().get_vote_on_hash(request).await?;
        Ok(response.into_inner().response)
    }

    pub async fn block_info_in_batch(
        &self,
        start_block: u64,
        end_block: u64,
    ) -> Result<Vec<BlockInfo>> {
        let request = GetBlockInfoInBatchRequest {
            start_block_number: start_block,
            end_block_number: end_block,
        };
        let response = self.inner.clone().get_block_info_in_batch(request).await?;
        Ok(response.into_inner().blocks)
    }

    pub async fn start_block_heimdall_span_id(
        &self,
        start_block: u64,
    ) -> Result<GetStartBlockHeimdallSpanIdResponse> {
        let request = GetStartBlockHeimdallSpanIdRequest { start_block };
        let response = self
            .inner
            .clone()
            .get_start_block_heimdall_span_id(request)
            .await?;
        Ok(response.into_inner())
    }
}
