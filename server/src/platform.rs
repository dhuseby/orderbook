#![allow(dead_code)]
use crate::Result;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};
use tungstenite::Message;
use vlog::{v3, verbose_log};

static MSG_COUNTER: AtomicUsize = AtomicUsize::new(1);

async fn get_next_id() -> usize {
    MSG_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug)]
pub struct Order {
    price: Decimal,
    quantity: Decimal,
}

#[derive(Debug)]
pub enum PlatformCmd {
    Orders {
        platform: Platform,
        symbol: String,
        asks: Vec<Order>,
        bids: Vec<Order>,
    },
    RequestReconnect,
}

impl fmt::Display for PlatformCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlatformCmd::Orders {
                platform,
                symbol,
                ref asks,
                ref bids,
            } => {
                write!(
                    f,
                    "{} - {} // asks: {}, bids: {}",
                    platform,
                    symbol,
                    asks.len(),
                    bids.len()
                )
                /*
                let _ = writeln!(f, "{} - {}:", platform, symbol);
                let _ = writeln!(f, "asks:");
                let _: Vec<_> = asks
                    .iter()
                    .map(|a| writeln!(f, "{} @ {}", a.quantity, a.price))
                    .collect();
                let _ = writeln!(f, "bids:");
                let _: Vec<_> = bids
                    .iter()
                    .map(|b| writeln!(f, "{} @ {}", b.quantity, b.price))
                    .collect();
                writeln!(f, "")
                */
            }
            PlatformCmd::RequestReconnect => {
                write!(f, "reconnect requested")
            }
        }
    }
}

// Orders from both Binance and Bitstamp are an array of 2 decimal values; the first is the price
// and the second is the quantity
type ArrOrder = [Decimal; 2];

#[derive(Debug, Deserialize, Serialize)]
pub struct BinanceOrdersData {
    #[serde(rename = "lastUpdateId")]
    last_update_id: usize,
    bids: Vec<ArrOrder>,
    asks: Vec<ArrOrder>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BitstampOrdersData {
    bids: Vec<ArrOrder>,
    asks: Vec<ArrOrder>,
    timestamp: String,
    microtimestamp: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BitstampChannelData {
    channel: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum WsMsg {
    // deserialized from the ws server
    BinanceOrders {
        stream: String,
        data: BinanceOrdersData,
    },
    BinanceResponse {
        result: serde_json::Value,
        id: usize,
    },
    BitstampOrders {
        event: String, // must be "data"
        channel: String,
        data: BitstampOrdersData,
    },
    BitstampResponse {
        event: String,
        channel: String,
        data: serde_json::Value,
    },

    // serialized to ws server
    BinanceSubUnsub {
        method: String, // must be "SUBSCRIBE" or "UNSUBSCRIBE"
        params: Vec<serde_json::Value>,
        id: usize,
    },
    BitstampSubUnsub {
        event: String, // must be "bts:subscribe" or "bts:unsubscribe"
        data: BitstampChannelData,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Binance,
    Bitstamp,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Platform::Binance => write!(f, "Binance"),
            Platform::Bitstamp => write!(f, "Bitstamp"),
        }
    }
}

impl Platform {
    pub async fn process_msg(&self, msg: &str) -> Option<PlatformCmd> {
        //NOTE: deserialization is notoriously slow
        let m = match serde_json::from_str::<WsMsg>(msg) {
            Ok(m) => m,
            Err(_) => return None,
        };

        match m {
            WsMsg::BinanceOrders {
                stream,
                data:
                    BinanceOrdersData {
                        ref bids, ref asks, ..
                    },
            } => {
                //NOTE: this may be too much copying for very high perf. going with a naive
                //solution for now until performace is measured as insufficient
                Some(PlatformCmd::Orders {
                    platform: self.clone(),
                    symbol: stream.split("@").next().unwrap_or("unknown").to_string(),
                    bids: bids
                        .into_iter()
                        .map(|b| Order {
                            price: b[0],
                            quantity: b[1],
                        })
                        .collect(),
                    asks: asks
                        .into_iter()
                        .map(|a| Order {
                            price: a[0],
                            quantity: a[1],
                        })
                        .collect(),
                })
            }
            WsMsg::BitstampOrders {
                channel,
                data:
                    BitstampOrdersData {
                        ref bids, ref asks, ..
                    },
                ..
            } => {
                //NOTE: this may be too much copying for very high perf. going with a naive
                //solution for now until performace is measured as insufficient
                Some(PlatformCmd::Orders {
                    platform: self.clone(),
                    symbol: channel
                        .split("_")
                        .skip(2)
                        .next()
                        .unwrap_or("unknown")
                        .to_string(),
                    bids: bids
                        .into_iter()
                        .map(|b| Order {
                            price: b[0],
                            quantity: b[1],
                        })
                        .collect(),
                    asks: asks
                        .into_iter()
                        .map(|a| Order {
                            price: a[0],
                            quantity: a[1],
                        })
                        .collect(),
                })
            }
            WsMsg::BitstampResponse { event, .. } => match event.as_str() {
                "bts:subscription_succeeded" => {
                    v3!("subscription succeeded");
                    None
                }
                "bts:request_reconnect" => Some(PlatformCmd::RequestReconnect),
                _ => {
                    v3!("received bitstamp event: {}", event);
                    None
                }
            },
            _ => return None,
        }
    }

    pub async fn connected(&self) -> Option<Message> {
        match self {
            Self::Binance => {
                let msg = WsMsg::BinanceSubUnsub {
                    method: "SET_PROPERTY".to_string(),
                    params: vec![
                        serde_json::Value::String("combined".to_string()),
                        serde_json::Value::Bool(true),
                    ],
                    id: get_next_id().await,
                };
                if let Ok(s) = serde_json::to_string(&msg) {
                    Some(Message::text(&s))
                } else {
                    None
                }
            }
            Self::Bitstamp => None,
        }
    }

    pub async fn subscribe(&self, symbol: &str) -> Result<Message> {
        match self {
            Self::Binance => self.sub_unsub("SUBSCRIBE", symbol).await,
            Self::Bitstamp => self.sub_unsub("bts:subscribe", symbol).await,
        }
    }

    pub async fn unsubscribe(&self, symbol: &str) -> Result<Message> {
        match self {
            Self::Binance => self.sub_unsub("UNSUBSCRIBE", symbol).await,
            Self::Bitstamp => self.sub_unsub("bts:unsubscribe", symbol).await,
        }
    }

    async fn sub_unsub(&self, cmd: &str, symbol: &str) -> Result<Message> {
        let msg = match self {
            Self::Binance => WsMsg::BinanceSubUnsub {
                method: cmd.to_string(),
                params: vec![
                    serde_json::Value::String(format!("{}@depth20", symbol)),
                    serde_json::Value::String(format!("{}@100ms", symbol)),
                ],
                id: get_next_id().await,
            },
            Self::Bitstamp => WsMsg::BitstampSubUnsub {
                event: cmd.to_string(),
                data: BitstampChannelData {
                    channel: format!("order_book_{}", symbol),
                },
            },
        };
        let s = serde_json::to_string(&msg)?;
        Ok(Message::text(&s))
    }
}
