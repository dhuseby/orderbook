#![allow(dead_code)]
use crate::{
    config::Exchange,
    error::Error,
    platform::{Orders, PlatformCmd},
    Result,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashSet;
use tokio::{net::TcpStream, sync::mpsc, task::JoinHandle};
use tokio_tungstenite::client_async_tls;
use vlog::{v1, v3, verbose_log};

#[derive(Debug)]
pub struct Source {
    /// handle to the source task
    handle: Option<JoinHandle<Result<()>>>,

    /// shutdown sender to tell our task to exit
    shutdown_tx: mpsc::Sender<()>,

    /// list of markets that the source is subscribed to
    markets: HashSet<String>,

    /// watch channel for the list of markets
    markets_tx: mpsc::UnboundedSender<HashSet<String>>,
}

impl Source {
    /// try to build a Source and connect to the exchange via a websocket
    pub async fn try_build(
        exchange: Exchange,
        data_tx: mpsc::Sender<Orders>,
        restart_tx: mpsc::Sender<Exchange>,
    ) -> Result<Self> {
        v3!("{} - try_build", exchange.platform);

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

                v1!(
                    "{} - proxying through {}:{}",
                    exchange.platform,
                    pr_host_str,
                    pr_port
                );
                let mut stream = TcpStream::connect((pr_host_str, pr_port)).await?;
                v1!(
                    "{} - connecting to {}:{}",
                    exchange.platform,
                    ex_host_str,
                    ex_port
                );
                async_socks5::connect(&mut stream, (ex_host_str, ex_port), None).await?;
                stream
            } else {
                v1!(
                    "{} - connecting to {}:{}",
                    exchange.platform,
                    ex_host_str,
                    ex_port
                );
                let stream = TcpStream::connect((ex_host_str, ex_port)).await?;
                stream
            }
        };

        v1!("{} - connected!", exchange.platform);

        // make the web socket connection
        let (mut ws_stream, ws_resp) = match client_async_tls(&exchange.url, stream).await {
            Ok((s, r)) => (s, r),
            Err(e) => {
                v3!("{} - connection failed...", exchange.platform);
                match e {
                    tokio_tungstenite::tungstenite::error::Error::Http(ref r) => {
                        if let Some(b) = r.body() {
                            if let Ok(s) = std::str::from_utf8(&b) {
                                v3!("{} - {}", exchange.platform, s);
                            }
                        }
                    }
                    _ => {
                        v3!("{} - unknown tungstenite error", exchange.platform)
                    }
                }
                return Err(Error::WebsocketError(e));
            }
        };

        v3!(
            "{} - ws connect status: {}",
            exchange.platform,
            ws_resp.status()
        );

        // connected...get platform specific setup messages to send
        if let Some(msg) = exchange.platform.connected().await {
            v3!("{} => {}", exchange.platform, msg);
            if ws_stream.send(msg).await.is_err() {
                v3!("{} - failed to send connection message", exchange.platform);
            }
        }

        // create the shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // create the markets update channel
        let (markets_tx, mut markets_rx) = mpsc::unbounded_channel::<HashSet<String>>();

        v3!("{} - spawing source task", exchange.platform);

        let handle: JoinHandle<Result<()>> = tokio::spawn(async move {
            v3!("{} - source task running", exchange.platform);
            let subscribed: HashSet<String> = HashSet::default();
            loop {
                tokio::select! {
                    // this handles shutdown notifications. if this task exits unexpectedly this
                    // receiver will be closed and can be used to detect the crash
                    _ = shutdown_rx.recv() => {
                        v1!("{} - shutting down", exchange.platform);
                        return Ok(());
                    },

                    // this handles updated market list
                    Some(markets) = markets_rx.recv() => {
                        // anything in markets not in subscribed are subscribes
                        let subscribes: HashSet<_> = markets.difference(&subscribed).collect();
                        for market in subscribes.iter() {
                            v1!("{} - subscribing to: {}", exchange.platform, market);
                            if let Ok(msg) = exchange.platform.subscribe(&market).await {
                                //v3!("-> {}", msg);
                                if ws_stream.send(msg).await.is_err() {
                                    v3!("{} - failed to send subscribe message", exchange.platform);
                                }
                            } else {
                                v3!("{} - failed to create subscribe message", exchange.platform);
                            }
                        }

                        // anything in subscribed and not in markets are unsubscribes
                        let unsubscribes: HashSet<_> = subscribed.difference(&markets).collect();
                        for market in unsubscribes.iter() {
                            v3!("{} - unsubscribing from: {}", exchange.platform, market);
                            if let Ok(msg) = exchange.platform.unsubscribe(&market).await {
                                if ws_stream.send(msg).await.is_err() {
                                    v3!("{} - failed to send unsubscribe message", exchange.platform);
                                }
                            } else {
                                v3!("{} - failed to create unsubscribe message", exchange.platform);
                            }
                        }
                    },

                    // this handles the incoming messages from the websocket
                    Some(res) = ws_stream.next() => {
                        match res {
                            Ok(msg) => {
                                //v3!("{} <= {}", exchange.platform, msg);
                                if let Some(cmd) = exchange.platform.process_msg(msg.to_text()?).await {
                                    match cmd {
                                        // send the data to the data aggregator
                                        PlatformCmd::Orders(ref orders) => {
                                            v3!("{} - {} orders", orders.platform, orders.symbol);
                                            if data_tx.send(orders.clone()).await.is_err() {
                                                v3!("{} - failed to send data to aggregator", exchange.platform);
                                            }
                                        }
                                        // send the reconnect request to the server
                                        PlatformCmd::RequestReconnect => {
                                            v3!("{} - restart requested", exchange.platform);
                                            if restart_tx.send(exchange.clone()).await.is_err() {
                                                v3!("{} - failed to set reconnect request", exchange.platform);
                                            }
                                        }
                                    }
                                }
                            },
                            Err(e) => {
                                v3!("{} - ws err: {:?}", exchange.platform, e);
                            },
                        }
                    }
                }
            }
        });

        Ok(Self {
            handle: Some(handle),
            shutdown_tx,
            markets: HashSet::default(),
            markets_tx,
        })
    }

    /// check if the Source's task is still running
    pub async fn running(&self) -> bool {
        // this returns true if the receive end of the channel is closed/dropped which
        // will happen when the task exits
        self.shutdown_tx.is_closed()
    }

    /// wait for the Source's task to end
    pub async fn join(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await?
        } else {
            Err(Error::NoJoinHandle)
        }
    }

    /// tell the Source task to shut down and release all resources
    pub async fn shutdown(&self) -> Result<()> {
        self.shutdown_tx
            .send(())
            .await
            .map_err(|_| Error::ShutdownError)
    }

    /// tell the Source to subscribe to the specified symbol
    pub async fn subscribe(&mut self, symbol: &str) -> Result<()> {
        if self.markets.insert(symbol.to_string()) {
            self.markets_tx
                .send(self.markets.clone())
                .map_err(|_| Error::TokioWatchSendError)
        } else {
            Ok(())
        }
    }

    /// tell the Source to unsubscribe from the specified symbol
    pub async fn unsubscribe(&mut self, symbol: &str) -> Result<()> {
        if self.markets.remove(symbol) {
            self.markets_tx
                .send(self.markets.clone())
                .map_err(|_| Error::TokioWatchSendError)
        } else {
            Ok(())
        }
    }
}
