#![allow(dead_code)]
use crate::Result;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};
use tungstenite::Message;
use vlog::{v1, v3, verbose_log};

static MSG_COUNTER: AtomicUsize = AtomicUsize::new(1);

async fn get_next_id() -> usize {
    MSG_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// # Ask
///
/// Asks are sorted by price first such that lower prices are Ordering::Less than higher prices. If
/// prices are equal then they are sorted by quantity such that higher quantities are
/// Ordering::Less than lower quantities. This means that the following is true:
/// {price: 1.0, quantity: 100} < {price: 1.5, quantity: 1000} < {price: 1.5, quantity: 100}
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ask {
    pub platform: Platform,
    pub price: Decimal,
    pub quantity: Decimal,
}

impl Ord for Ask {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        // first sort by price...if our price is lower than the rhs price
        // then this returns Ordering::Less and our Ask will sort before
        // the rhs Ask. this puts the lowest price first
        let order = self.price.cmp(&rhs.price);

        // if the price is equal then we compare the quantities
        if order == std::cmp::Ordering::Equal {
            // if rhs quantity is 1000 and our quantity is 10000
            // this returns Ordering::Less which will result in our
            // Ask being sorted before the rhs Ask
            rhs.quantity.cmp(&self.quantity)
        } else {
            order
        }
    }
}

impl PartialOrd for Ask {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        if let Some(order) = self.price.partial_cmp(&rhs.price) {
            if order == std::cmp::Ordering::Equal {
                rhs.quantity.partial_cmp(&self.quantity)
            } else {
                Some(order)
            }
        } else {
            None
        }
    }
}

impl Default for Ask {
    fn default() -> Self {
        // a default Ask has a max price and zero quantity and should always be Ordering::Greater
        // than all other Asks
        Self {
            platform: Platform::default(),
            price: Decimal::MAX,
            quantity: Decimal::ZERO,
        }
    }
}

/// # Bid
///
/// Bids are sorted by price first such that higher prices are Ordering::Less than lower prices. If
/// prices are equal then they are sorted by quantity such that higher quantities are
/// Ordering::Less than lower quantities. This means that the following is true:
/// {price: 1.5, quantity: 1000} < {price: 1.5, quantity: 100} < {price: 1.0, quantity: 100}
///
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bid {
    pub platform: Platform,
    pub price: Decimal,
    pub quantity: Decimal,
}

impl Ord for Bid {
    fn cmp(&self, rhs: &Self) -> std::cmp::Ordering {
        // first sort by price...if the rhs price is lower than our price
        // then this returns Ordering::Less and the rhs Bid will sort before
        // our Bid. this puts the highest price first.
        let order = rhs.price.cmp(&self.price);

        // if the price is equal then we compare the quantities
        if order == std::cmp::Ordering::Equal {
            // if rhs quantity is 1000 and our quantity is 10000
            // this returns Ordering::Less which will result in our
            // Bid being sorted before the rhs Bid. this puts the
            // higher quantities first.
            rhs.quantity.cmp(&self.quantity)
        } else {
            order
        }
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, rhs: &Self) -> Option<std::cmp::Ordering> {
        if let Some(order) = rhs.price.partial_cmp(&self.price) {
            if order == std::cmp::Ordering::Equal {
                rhs.quantity.partial_cmp(&self.quantity)
            } else {
                Some(order)
            }
        } else {
            None
        }
    }
}

impl Default for Bid {
    fn default() -> Self {
        // a default Bid has a zero price and zero quantity and should always be Ordering::Greater
        // than all other Bids
        Self {
            platform: Platform::default(),
            price: Decimal::ZERO,
            quantity: Decimal::ZERO,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Orders {
    pub platform: Platform,
    pub symbol: String,
    pub asks: BTreeSet<Ask>,
    pub bids: BTreeSet<Bid>,
}

#[derive(Debug)]
pub enum PlatformCmd {
    Orders(Orders),
    RequestReconnect,
}

impl fmt::Display for PlatformCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlatformCmd::Orders(ref orders) => {
                write!(
                    f,
                    "{} - {} // asks: {}, bids: {}",
                    orders.platform,
                    orders.symbol,
                    orders.asks.len(),
                    orders.bids.len()
                )
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

#[derive(Clone, Debug, Default, Deserialize, Eq, Hash, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    #[default]
    Binance,
    Bitstamp,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Platform::Binance => write!(f, "binance"),
            Platform::Bitstamp => write!(f, "bitstamp"),
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
                let orders = Orders {
                    platform: self.clone(),
                    symbol: stream.split("@").next().unwrap_or("unknown").to_string(),
                    bids: bids
                        .into_iter()
                        .map(|b| Bid {
                            platform: self.clone(),
                            price: b[0],
                            quantity: b[1],
                        })
                        .collect(),
                    asks: asks
                        .into_iter()
                        .map(|a| Ask {
                            platform: self.clone(),
                            price: a[0],
                            quantity: a[1],
                        })
                        .collect(),
                };
                Some(PlatformCmd::Orders(orders))
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
                let orders = Orders {
                    platform: self.clone(),
                    symbol: channel
                        .split("_")
                        .skip(2)
                        .next()
                        .unwrap_or("unknown")
                        .to_string(),
                    bids: bids
                        .into_iter()
                        .map(|b| Bid {
                            platform: self.clone(),
                            price: b[0],
                            quantity: b[1],
                        })
                        .collect(),
                    asks: asks
                        .into_iter()
                        .map(|a| Ask {
                            platform: self.clone(),
                            price: a[0],
                            quantity: a[1],
                        })
                        .collect(),
                };
                Some(PlatformCmd::Orders(orders))
            }
            WsMsg::BitstampResponse { event, .. } => match event.as_str() {
                "bts:subscription_succeeded" => {
                    v1!("subscription succeeded");
                    None
                }
                // this will request that this source get shut down and re-connected
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ask_sorting() {
        let mut sorted = BTreeSet::new();

        let first = Ask {
            platform: Platform::default(),
            price: Decimal::from_str("1.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        let second = Ask {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("1000.0").unwrap(),
        };

        let third = Ask {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        sorted.insert(&second);
        sorted.insert(&third);
        sorted.insert(&first);

        assert_eq!(Some(&first), sorted.pop_first());
        assert_eq!(Some(&second), sorted.pop_first());
        assert_eq!(Some(&third), sorted.pop_first());
    }

    #[test]
    fn bid_sorting() {
        let mut sorted = BTreeSet::new();

        let first = Bid {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("1000.0").unwrap(),
        };

        let second = Bid {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        let third = Bid {
            platform: Platform::default(),
            price: Decimal::from_str("1.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        sorted.insert(&second);
        sorted.insert(&third);
        sorted.insert(&first);

        assert_eq!(Some(&first), sorted.pop_first());
        assert_eq!(Some(&second), sorted.pop_first());
        assert_eq!(Some(&third), sorted.pop_first());
    }

    #[test]
    fn ask_cmp() {
        let def = Ask::default();

        let first = Ask {
            platform: Platform::default(),
            price: Decimal::from_str("1.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        let second = Ask {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("1000.0").unwrap(),
        };

        let third = Ask {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        assert!(first < second);
        assert!(second < third);
        assert!(first < third);
        assert!(first < def);
        assert!(second < def);
        assert!(third < def);
    }

    #[test]
    fn bid_cmp() {
        let def = Bid::default();

        let first = Bid {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("1000.0").unwrap(),
        };

        let second = Bid {
            platform: Platform::default(),
            price: Decimal::from_str("1.5").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        let third = Bid {
            platform: Platform::default(),
            price: Decimal::from_str("1.0").unwrap(),
            quantity: Decimal::from_str("100.0").unwrap(),
        };

        assert!(first < second);
        assert!(second < third);
        assert!(first < third);
        assert!(first < def);
        assert!(second < def);
        assert!(third < def);
    }
}
