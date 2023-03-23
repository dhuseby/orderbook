use anyhow;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    // i/o error
    #[error("i/o error")]
    IoError(#[from] std::io::Error),

    // toml decode error
    #[error("toml deserialize error")]
    TomlError(#[from] toml::de::Error),

    // Orderbook error
    #[error("orderbook client library error")]
    OrderbookClientError(#[from] orderbook_client::OrderbookError),

    // tokio error
    #[error("tokio join error {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),

    // ctrlc error
    #[error("failed to set up ctrl+c handler")]
    CtrlcError(#[from] ctrlc::Error),

    // invalid client state error
    #[error("invalid client state {0}, expecting {1}")]
    InvalidClientState(String, String),

    // unexpected response
    #[error("unexpected response {0} for state {1}")]
    UnexpectedResponse(String, String),

    // failed to connect
    #[error("failed to connect to server")]
    ConnectionFailed,

    // failed to shutdown
    #[error("failed to shutdown")]
    ShutdownFailed,
}

pub type Result<T> = anyhow::Result<T, Error>;
