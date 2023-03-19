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

    // url parse error
    #[error("url parse error: {0}")]
    UrlError(#[from] url::ParseError),

    // async_socks5 error
    #[error("socks protocol error {0}")]
    SocksError(#[from] async_socks5::Error),

    // tungstenite error
    #[error("websocket error {0}")]
    WebsocketError(#[from] tokio_tungstenite::tungstenite::Error),

    // serde json error
    #[error("serde deserialization error {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    // tokio broadcast error
    #[error("failed to broadcast command")]
    TokioBroadcastError,

    // source error
    #[error("no exchange defined for a source")]
    NoExchangeDefined,

    // invalid proxy error
    #[error("invalid proxy definition")]
    InvalidProxyDefined,

    // no command channel error
    #[error("no command broadcast channel provided")]
    NoCommandChannel,

    // no data channel error
    #[error("no data channel provided")]
    NoDataChannel,
}

pub type Result<T> = anyhow::Result<T, Error>;
