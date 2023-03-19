#![allow(dead_code)]
use crate::{platform::Platform, Result};
use serde::{Deserialize, Deserializer};
use std::{
    convert::TryFrom,
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
};
use url::Url;

#[derive(Clone, Debug, Deserialize)]
pub struct Exchange {
    pub platform: Platform,
    #[serde(deserialize_with = "url_de_opt")]
    pub proxy: Option<Url>,
    #[serde(deserialize_with = "url_de")]
    pub url: Url,
}

#[derive(Debug, Deserialize)]
pub struct Market {
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
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

fn url_de<'de, D>(deserializer: D) -> std::result::Result<Url, D::Error>
where
    D: Deserializer<'de>,
{
    // get the string representation...
    let s = String::deserialize(deserializer)?;
    // parse it into a Url
    Url::parse(&s).map_err(serde::de::Error::custom)
}

fn url_de_opt<'de, D>(deserializer: D) -> std::result::Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    // get the string representation...
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    // parse it into a Url
    match Url::parse(&s) {
        Ok(url) => Ok(Some(url)),
        Err(e) => Err(serde::de::Error::custom(e)),
    }
}
