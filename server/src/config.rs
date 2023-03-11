#![allow(dead_code)]
use crate::Result;
use serde::Deserialize;
use std::{
    convert::TryFrom,
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
};

#[derive(Debug, Deserialize)]
pub struct Setup {
    proxy: bool,
}

#[derive(Debug, Deserialize)]
pub struct Proxy {
    url: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Binance,
    Bitstamp,
}

#[derive(Debug, Deserialize)]
pub struct Exchange {
    platform: Platform,
    url: String,
    port: u16,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    proxy: Proxy,
    exchanges: Vec<Exchange>,
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
