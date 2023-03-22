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

    // tokio task join error
    #[error("tokio task join error {0}")]
    TaskJoinError(#[from] tokio::task::JoinError),

    // hex decode error
    #[error("failed to decode hex {0}")]
    HexDecodeError(#[from] hex::FromHexError),

    // uuid decode error
    #[error("failed to decode uuid {0}")]
    UuidDecodeError(#[from] uuid::Error),

    // tonic transport error
    #[error("tonic transport error {0}")]
    TonicTransportError(#[from] tonic::transport::Error),

    // tokio watch channel send error
    #[error("tokio watch channel send failed")]
    TokioWatchSendError,

    // tokio broadcast error
    #[error("failed to broadcast command")]
    TokioBroadcastError,

    // source command error
    #[error("failed to send command to source")]
    SourceCommandError,

    // shutdown error
    #[error("failed to signal shutdown")]
    ShutdownError,

    // source error
    #[error("no exchange defined for a source")]
    NoExchangeDefined,

    // invalid proxy error
    #[error("invalid proxy definition")]
    InvalidProxyDefined,

    // no shutdown channel error
    #[error("no shutdown channel provided")]
    NoShutdownChannel,

    // no command channel error
    #[error("no command broadcast channel provided")]
    NoCommandChannel,

    // no reconnect channel error
    #[error("no reconnect request channel provided")]
    NoReconnectChannel,

    // no data channel error
    #[error("no data channel provided")]
    NoDataChannel,

    // no join handle error
    #[error("tried to join when no join handle exists")]
    NoJoinHandle,

    // no oberon proof with the request
    #[error("no oberon proof was provided")]
    NoOberonProof,

    // proof timestamp is expired
    #[error("oberon proof timestamp has expired")]
    ExpiredOberonProof,

    // proof is invalid
    #[error("oberon proof is invalid")]
    InvalidOberonProof,

    // public key load failed
    #[error("failed to decode public key")]
    PublicKeyDecodeError,
}

pub type Result<T> = anyhow::Result<T, Error>;
