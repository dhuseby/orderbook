#![allow(dead_code)]
use crate::{platform::Platform, utils::*, Result};
use oberon::PublicKey;
use serde::Deserialize;
use std::{
    convert::TryFrom,
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
};
use url::Url;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq)]
pub struct Exchange {
    pub platform: Platform,
    #[serde(deserialize_with = "url_de_opt")]
    pub proxy: Option<Url>,
    #[serde(deserialize_with = "url_de")]
    pub url: Url,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Market {
    pub symbol: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub watchdog_timeout: Option<u64>,
    pub order_limit: Option<usize>,
    #[serde(deserialize_with = "pk_de")]
    pub public_key: PublicKey,
    pub exchanges: Vec<Exchange>,
    pub markets: Vec<Market>,
}

impl TryFrom<&str> for Config {
    type Error = crate::error::Error;

    fn try_from(value: &str) -> Result<Self> {
        let config: Self = toml::from_str(value)?;
        Ok(config)
    }
}

impl Config {
    pub fn try_from_os_str(ostr: &OsStr) -> std::result::Result<Self, OsString> {
        let pb = PathBuf::from(ostr);
        if let Ok(data) = fs::read_to_string(&pb) {
            if let Ok(config) = Self::try_from(data.as_str()) {
                return Ok(config);
            }
        }
        Err(ostr.to_os_string())
    }
}
