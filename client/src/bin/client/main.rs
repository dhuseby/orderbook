#![allow(dead_code)]
mod utils;
use crossbeam_channel::{bounded, select, Receiver, Sender};
use once_cell::sync::OnceCell;
use orderbook_client::{
    orderbook_client_initialize, orderbook_client_shutdown, OnOrderbookClientEvents, Orderbook,
    OrderbookResponse,
};
use std::{collections::HashMap, fmt, sync::Arc};
use structopt::StructOpt;
use tokio::sync::Mutex;
use utils::{
    config::{Config, Market},
    error::Error,
    Result,
};
use vlog::{set_verbosity_level, v1, v3, verbose_log};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "orderbook-server",
    version = "0.1",
    author = "David Huseby <dwh@linuxprogrammer.org>",
    about = "Orderbook aggregator service"
)]
struct Opt {
    /// Verbosity
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    verbosity: usize,

    /// Config file
    #[structopt(name = "config", parse(try_from_os_str = Config::try_from_os_str))]
    config: Config,
}

enum Responses {
    Init { id: u64, orderbook: Arc<Orderbook> },
    Completed { id: u64, resp: OrderbookResponse },
}

struct Cb {
    tx: Sender<Responses>,
}

impl OnOrderbookClientEvents for Cb {
    /// Called after initialization is complete
    fn initialized(&self, id: u64, orderbook: Arc<Orderbook>) {
        // NOTE: a channel is used to pass handling this state change in the local async runtime
        let _ = self.tx.send(Responses::Init { id, orderbook });
    }

    /// called when an operation completes
    fn completed(&self, id: u64, resp: OrderbookResponse) {
        // NOTE: a channel is used to pass handling this state change in the local async runtime
        let _ = self.tx.send(Responses::Completed { id, resp });
    }
}

// state machine for the client
enum Client {
    Created,
    Connecting {
        id: u64,
    },
    Subscribing {
        orderbook: Arc<Orderbook>,
        ids: HashMap<u64, String>,
        subs: HashMap<u64, String>,
    },
    ShuttingDown {
        id: u64,
    },
    Shutdown,
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Client::Created => write!(f, "Created"),
            Client::Connecting { .. } => write!(f, "Connecting"),
            Client::Subscribing { .. } => write!(f, "Subscribing"),
            Client::ShuttingDown { .. } => write!(f, "ShuttingDown"),
            Client::Shutdown => write!(f, "Shutdown"),
        }
    }
}

impl Client {
    pub fn new() -> Self {
        Self::Created
    }

    pub fn connect(
        &mut self,
        token: String,
        endpoint: String,
        callbacks: Box<dyn OnOrderbookClientEvents>,
    ) -> Result<()> {
        let id = orderbook_client_initialize(token, endpoint, callbacks)?;
        v3!("EXE - Created -> Connecting");
        *self = Self::Connecting { id };
        Ok(())
    }

    pub fn init(
        &mut self,
        rid: u64,
        orderbook: Arc<Orderbook>,
        markets: &Vec<Market>,
    ) -> Result<()> {
        match self {
            Self::Connecting { id } => {
                if *id == rid {
                    let mut ids: HashMap<u64, String> = HashMap::default();
                    for m in markets {
                        let id = orderbook.summary(m.symbol.clone())?;
                        v3!("EXE - subscribing {}: {}", id, m.symbol);
                        ids.insert(id, m.symbol.clone());
                    }

                    // Connecting -> Subscribing
                    v3!("EXE - Connecting -> Subscribing");
                    *self = Self::Subscribing {
                        orderbook: orderbook.clone(),
                        ids,
                        subs: HashMap::default(),
                    };
                    Ok(())
                } else {
                    Err(Error::ConnectionFailed)
                }
            }
            _ => Err(Error::InvalidClientState(
                format!("{}", self),
                "Connecting".to_string(),
            )),
        }
    }

    pub fn completed(&mut self, rid: u64, resp: OrderbookResponse) -> Result<()> {
        match self {
            Self::Subscribing {
                ref mut ids,
                ref mut subs,
                ..
            } => match resp {
                OrderbookResponse::OrderbookSummary { summary } => {
                    if ids.contains_key(&rid) {
                        if let Some((_, symbol)) = ids.remove_entry(&rid) {
                            let _ = subs.insert(rid, symbol);
                        }
                    }
                    if let Some(symbol) = subs.get(&rid) {
                        v1!("{:>10} spread: {}", symbol, summary.spread);
                        for i in 0..10 {
                            let bid = if i < summary.bids.len() {
                                format!(
                                    "bid: {:>14} @ {:<18}",
                                    summary.bids[i].amount, summary.bids[i].price,
                                )
                            } else {
                                "".to_string()
                            };

                            let ask = if i < summary.asks.len() {
                                format!(
                                    "ask: {:>14} @ {:<18}",
                                    summary.asks[i].amount, summary.asks[i].price,
                                )
                            } else {
                                "".to_string()
                            };
                            v1!("\t{} {}", bid, ask);
                        }
                    }
                    Ok(())
                }
                OrderbookResponse::Failed { reason } => {
                    v1!("EXE - summary failed: {}", reason);
                    let _ = ids.remove(&rid);

                    if ids.len() == 0 {
                        v3!("EXE - ShuttingDown -> Shutdown");
                        *self = Self::Shutdown;
                    }
                    Ok(())
                }
                _ => Err(Error::UnexpectedResponse(
                    format!("{}", resp),
                    format!("{}", self),
                )),
            },
            Self::ShuttingDown { id } => match resp {
                OrderbookResponse::Failed { .. } | OrderbookResponse::Shutdown => {
                    if *id == rid {
                        v3!("EXE - ShuttingDown -> Shutdown");
                        *self = Self::Shutdown;
                        Ok(())
                    } else {
                        Err(Error::ShutdownFailed)
                    }
                }
                _ => Err(Error::UnexpectedResponse(
                    format!("{}", resp),
                    format!("{}", self),
                )),
            },
            _ => Err(Error::InvalidClientState(
                format!("{}", self),
                "Connecting".to_string(),
            )),
        }
    }

    pub fn shuttingdown(&mut self) -> Result<()> {
        let id = orderbook_client_shutdown()?;

        // Any -> ShuttingDown
        v3!("EXE - {} -> Shuttingdown", self);
        *self = Client::ShuttingDown { id };
        Ok(())
    }

    pub fn shutdown(&mut self) -> bool {
        match self {
            Self::Shutdown => true,
            _ => false,
        }
    }
}

// singleton of the client actor
static CLIENT: OnceCell<Mutex<Client>> = OnceCell::new();

// convenience function to get a reference to the client Mutex
fn global_client() -> &'static Mutex<Client> {
    CLIENT.get().unwrap()
}

// set up the client singleton
fn setup_client(client: Client) {
    if CLIENT.get().is_none() {
        if CLIENT.set(Mutex::new(client)).is_err() {
            v1!("client already initialized");
        }
    } else {
        let mut lock = CLIENT.get().unwrap().blocking_lock();
        *lock = client;
    }
}

fn shutdown_channel() -> Result<Receiver<()>> {
    let (tx, rx) = bounded::<()>(1);
    ctrlc::set_handler(move || {
        let _ = tx.send(());
    })?;

    Ok(rx)
}

fn main() -> Result<()> {
    // parse the command line flags
    let opt = Opt::from_args();

    // set the verbosity level
    set_verbosity_level(opt.verbosity);
    v3!("EXE - verbosity set to: {}", opt.verbosity);

    // set up the client state machine
    setup_client(Client::new());

    // create the channel for receiving Summary objects
    let (tx, rx) = bounded::<Responses>(16);
    let cb = Box::new(Cb { tx });

    // create the shutdown channel
    let shutdown_rx = shutdown_channel()?;

    // connect to the server and store the updated client
    {
        let mut c = global_client().blocking_lock();
        c.connect(opt.config.token, opt.config.connect, cb)?;
    }

    loop {
        select! {
            recv(shutdown_rx) -> _ => {
                let mut c = global_client().blocking_lock();
                if let Err(e) = c.shuttingdown() {
                    v1!("EXE - shutdown error: {}", e);
                }
            }
            recv(rx) -> resp => {
                if let Ok(r) = resp {
                    match r {
                        Responses::Init { id, orderbook } => {
                            let mut c = global_client().blocking_lock();
                            match c.init(id, orderbook, &opt.config.markets) {
                                Ok(()) => v3!("EXE - {}: initialized", id),
                                Err(e) => v1!("EXE - init failure: {}", e),
                            };
                        }
                        Responses::Completed { id, resp } => {
                            let mut c = global_client().blocking_lock();
                            match c.completed(id, resp) {
                                Ok(()) => v3!("EXE - {}: completed", id),
                                Err(e) => v1!("EXE - completed failure: {}", e),
                            };

                            // if the shutdown has completed, end the task
                            if c.shutdown() {
                                v1!("EXE - shutdown complete");
                                break;
                            }
                        }
                    }
                } else {
                    v1!("EXE - failed to receive message from callback");
                }
            }
        }
    }

    v1!("EXE - exiting");

    Ok(())
}
