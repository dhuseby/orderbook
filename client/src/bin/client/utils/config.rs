use crate::utils::Result;
use serde::Deserialize;
use std::{
    convert::TryFrom,
    ffi::{OsStr, OsString},
    fs,
    path::PathBuf,
};

#[derive(Clone, Debug, Deserialize)]
pub struct Market {
    pub symbol: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub connect: String,
    pub token: String,
    pub markets: Vec<Market>,
}

impl TryFrom<&str> for Config {
    type Error = crate::utils::error::Error;

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
