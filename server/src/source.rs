#![allow(dead_code)]
use crate::{
    config::{Exchange, Proxy},
    error::Error,
    Result,
};
use rust_decimal::prelude::*;
use tokio::sync::{broadcast, mpsc};

#[derive(Debug, Default)]
pub struct SourceBuilder {
    exchange: Option<Exchange>,
    proxy: Option<Proxy>,
    shutdown_rx: Option<broadcast::Receiver<bool>>,
    data_tx: Option<mpsc::Sender<Orders>>,
}

impl SourceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_exchange(mut self, exchange: Exchange) -> Self {
        self.exchange = Some(exchange);
        self
    }

    pub fn with_proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn with_shutdown_rx(mut self, shutdown_rx: broadcast::Receiver<bool>) -> Self {
        self.shutdown_rx = Some(shutdown_rx);
        self
    }

    pub fn with_data_tx(mut self, data_tx: mpsc::Sender<Orders>) -> Self {
        self.data_tx = Some(data_tx);
        self
    }

    pub async fn try_build(&self) -> Result<Source> {
        if self.exchange.is_none() {
            return Err(Error::NoExchangeDefined);
        }

        Ok(Source {})
    }
}

#[derive(Debug, Default)]
pub struct Order {
    price: Decimal,
    quantity: u128,
}

#[derive(Debug, Default)]
pub struct Orders {
    asks: Vec<Order>,
    bids: Vec<Order>,
}

#[derive(Debug, Default)]
pub struct Source {}
