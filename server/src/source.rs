#![allow(dead_code)]
use crate::{config::Exchange, error::Error, platform::PlatformCmd, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    net::TcpStream,
    sync::{broadcast, mpsc},
    task::JoinHandle,
};
use tokio_tungstenite::client_async_tls;
use vlog::{v3, verbose_log};

// an alias to the join handle for the task
pub type Source = JoinHandle<Result<()>>;

#[derive(Clone, Debug)]
pub enum SourceCmd {
    Shutdown,
    Subscribe(String),
    Unsubscribe(String),
}

#[derive(Debug, Default)]
pub struct SourceBuilder {
    exchange: Option<Exchange>,
    source_rx: Option<broadcast::Receiver<SourceCmd>>,
    platform_tx: Option<mpsc::Sender<PlatformCmd>>,
}

impl SourceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_exchange(mut self, exchange: Exchange) -> Self {
        self.exchange = Some(exchange);
        self
    }

    pub fn with_source_rx(mut self, source_rx: broadcast::Receiver<SourceCmd>) -> Self {
        self.source_rx = Some(source_rx);
        self
    }

    pub fn with_platform_tx(mut self, platform_tx: mpsc::Sender<PlatformCmd>) -> Self {
        self.platform_tx = Some(platform_tx);
        self
    }

    pub async fn try_build(mut self) -> Result<Source> {
        // get the exchange
        let exchange = match &self.exchange {
            Some(e) => e.clone(),
            None => return Err(Error::NoExchangeDefined),
        };
        v3!("attempting to ws to {:?}", exchange.platform);

        // get the exchange ToSocketAddrs
        let ex_host_str = exchange
            .url
            .host_str()
            .ok_or(Error::NoExchangeDefined)?
            .to_owned();
        let ex_port = exchange
            .url
            .port_or_known_default()
            .ok_or(Error::NoExchangeDefined)?;

        // create the socket
        let stream = {
            if let Some(proxy) = &exchange.proxy {
                let pr_host_str = proxy
                    .host_str()
                    .ok_or(Error::InvalidProxyDefined)?
                    .to_owned();
                let pr_port = proxy
                    .port_or_known_default()
                    .ok_or(Error::InvalidProxyDefined)?;

                v3!("proxying through {}:{}", pr_host_str, pr_port);
                let mut stream = TcpStream::connect((pr_host_str, pr_port)).await?;
                v3!("connecting to {}:{}", ex_host_str, ex_port);
                async_socks5::connect(&mut stream, (ex_host_str, ex_port), None).await?;
                stream
            } else {
                v3!("connecting to {}:{}", ex_host_str, ex_port);
                let stream = TcpStream::connect((ex_host_str, ex_port)).await?;
                stream
            }
        };

        v3!("connected!");

        // make the web socket connection
        let (mut ws_stream, ws_resp) = match client_async_tls(&exchange.url, stream).await {
            Ok((s, r)) => (s, r),
            Err(e) => {
                v3!("connection failed...");
                match e {
                    tokio_tungstenite::tungstenite::error::Error::Http(ref r) => {
                        if let Some(b) = r.body() {
                            if let Ok(s) = std::str::from_utf8(&b) {
                                v3!("{}", s);
                            }
                        }
                    }
                    _ => {
                        v3!("unknown tungstenite error")
                    }
                }
                return Err(Error::WebsocketError(e));
            }
        };

        v3!("ws connect status: {}", ws_resp.status());

        // get the shutdown_rx
        if self.source_rx.is_none() {
            return Err(Error::NoCommandChannel);
        }
        let mut source_rx = self.source_rx.take().unwrap();

        // get the platform_tx
        let platform_tx = match &self.platform_tx {
            Some(d) => d.clone(),
            None => return Err(Error::NoDataChannel),
        };

        // connected...do the platform callback to get any messages to send before subscribing
        if let Some(msg) = exchange.platform.connected().await {
            v3!("-> {}", msg);
            if ws_stream.send(msg).await.is_err() {
                v3!("failed to send connection message");
            }
        }

        v3!("spawing task");
        let handle: Source = tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(cmd) = source_rx.recv() => match cmd {
                        SourceCmd::Shutdown => {
                            v3!("shutting down");
                            return Ok(());
                        },
                        SourceCmd::Subscribe(symbol) => {
                            v3!("subscribing to: {}", symbol);
                            if let Ok(msg) = exchange.platform.subscribe(&symbol).await {
                                v3!("-> {}", msg);
                                if ws_stream.send(msg).await.is_err() {
                                    v3!("failed to send subscribe message");
                                }
                            } else {
                                v3!("failed to create subscribe message");
                            }
                        },
                        SourceCmd::Unsubscribe(symbol) => {
                            v3!("unsubscribing from: {}", symbol);
                            if let Ok(msg) = exchange.platform.unsubscribe(&symbol).await {
                                if ws_stream.send(msg).await.is_err() {
                                    v3!("failed to send unsubscribe message");
                                }
                            } else {
                                v3!("failed to create unsubscribe message");
                            }
                        },
                    },
                    Some(res) = ws_stream.next() => {
                        match res {
                            Ok(msg) => {
                                //v3!("<- {}", msg);
                                if let Some(cmd) = exchange.platform.process_msg(msg.to_text()?).await {
                                    if platform_tx.send(cmd).await.is_err() {
                                        v3!("failed to send data to aggregator");
                                    }
                                }
                            },
                            Err(e) => {
                                v3!("ws err: {:?}", e);
                            },
                        }
                    }
                }
            }
        });

        Ok(handle)
    }
}
