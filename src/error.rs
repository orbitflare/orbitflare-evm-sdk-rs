#[cfg(feature = "rpc")]
use std::fmt;
use std::time::Duration;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Transport(Box<dyn std::error::Error + Send + Sync>),

    #[error("rpc error {code}: {message}")]
    Rpc { code: i64, message: String },

    #[error("rate limited{}", match .retry_after {
        Some(d) => format!(" (retry after {}s)", d.as_secs()),
        None => String::new(),
    })]
    RateLimited { retry_after: Option<Duration> },

    #[error("auth: {0}")]
    Auth(String),

    #[error("request timed out")]
    Timeout,

    #[error("stream closed: {0}")]
    Stream(String),

    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    Serialization(String),

    #[error("missing field in response: {0}")]
    MissingField(&'static str),

    #[cfg(feature = "grpc")]
    #[error("grpc: {0}")]
    Grpc(Box<tonic::Status>),
}

impl Error {
    pub fn is_retryable(&self) -> bool {
        match self {
            Error::Transport(_) | Error::Timeout | Error::RateLimited { .. } => true,
            Error::Rpc { code, .. } => matches!(code, -32005),
            Error::Stream(_) => true,
            #[cfg(feature = "grpc")]
            Error::Grpc(status) => matches!(
                status.code(),
                tonic::Code::Unavailable
                    | tonic::Code::ResourceExhausted
                    | tonic::Code::DeadlineExceeded
            ),
            _ => false,
        }
    }

    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Error::RateLimited { retry_after } => *retry_after,
            _ => None,
        }
    }

    #[cfg(feature = "rpc")]
    pub(crate) fn transport(err: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::Transport(Box::new(err))
    }

    #[cfg(feature = "rpc")]
    pub fn with_endpoint(self, endpoint: &str) -> Self {
        let endpoint = sanitize_url(endpoint);
        match self {
            Error::Transport(e) => Error::Transport(format!("[{endpoint}] {e}").into()),
            Error::Rpc { code, message } => Error::Rpc {
                code,
                message: format!("[{endpoint}] {message}"),
            },
            other => other,
        }
    }
}

#[cfg(feature = "rpc")]
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_timeout() {
            Error::Timeout
        } else {
            Error::Transport(Box::new(e))
        }
    }
}

#[cfg(feature = "rpc")]
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}

#[cfg(feature = "grpc")]
impl From<tonic::Status> for Error {
    fn from(s: tonic::Status) -> Self {
        Error::Grpc(Box::new(s))
    }
}

#[cfg(feature = "grpc")]
impl From<tonic::transport::Error> for Error {
    fn from(e: tonic::transport::Error) -> Self {
        Error::Transport(Box::new(e))
    }
}

#[cfg(feature = "rpc")]
pub(crate) fn sanitize_url(s: &str) -> String {
    let mut result = s.to_string();
    for needle in ["?api_key=", "&api_key="] {
        while let Some(start) = result.find(needle) {
            let value_start = start + needle.len();
            let end = result[value_start..]
                .find(|c: char| c == '&' || c == ')' || c.is_whitespace())
                .map(|i| i + value_start)
                .unwrap_or(result.len());
            result = format!("{}{}", &result[..start], &result[end..]);
        }
    }
    result
}

#[cfg(feature = "rpc")]
pub(crate) struct SanitizedUrl<'a>(pub &'a str);

#[cfg(feature = "rpc")]
impl fmt::Display for SanitizedUrl<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&sanitize_url(self.0))
    }
}
