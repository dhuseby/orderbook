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

    // source error
    #[error("no exchange defined for a source")]
    NoExchangeDefined,
}

pub type Result<T> = anyhow::Result<T, Error>;
