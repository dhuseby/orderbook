#![allow(dead_code)]
use crate::{
    api_proof::ApiProof,
    config::{Config, Exchange},
    error::Error,
    orderbook::*,
    platform::{Ask, Bid, Platform},
    source::Source,
    Result,
};
use futures_util::Stream;
use oberon::PublicKey;
use rust_decimal::prelude::*;
use std::{
    collections::{BTreeSet, HashMap},
    pin::Pin,
};
use tokio::{
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use vlog::{v1, v3, verbose_log};

// the Orderbook keeps the the latest asks and bids for each platform we're talking to and then
// whenever we receive an update we generate a new Summary that is the combination of all asks
// and bids that we send to clients
#[derive(Debug, Default)]
struct Orderbook {
    asks: HashMap<Platform, BTreeSet<Ask>>,
    bids: HashMap<Platform, BTreeSet<Bid>>,
}

impl Orderbook {
    fn spread(&self) -> Decimal {
        let mut best_ask = Ask::default();
        let mut best_bid = Bid::default();

        // get the best ask from all of the platforms
        for (p, v) in self.asks.iter() {
            v3!("checking {} best ask", p);
            if let Some(ask) = v.first() {
                v3!("  ask {}: {}", ask.platform, ask.price);
                if ask < &best_ask {
                    best_ask = ask.clone();
                }
                v3!("  best_ask {}: {}", best_ask.platform, best_ask.price);
            }
        }

        // get the best bid from all of the platforms
        for (p, v) in self.bids.iter() {
            v3!("checking {} best bid", p);
            if let Some(bid) = v.first() {
                v3!("  bid {}: {}", bid.platform, bid.price);
                if *bid < best_bid {
                    best_bid = bid.clone();
                }
                v3!("  best_bid {}: {}", best_bid.platform, best_bid.price);
            }
        }

        v3!(
            "spread: {} - {} = {}",
            best_ask.price,
            best_bid.price,
            best_ask.price - best_bid.price
        );

        // return the spread, this can sometimes be negative
        best_ask.price - best_bid.price
    }

    fn asks(&self, count: usize) -> Vec<Level> {
        let mut asks: BTreeSet<Ask> = BTreeSet::default();

        // merge all of the asks from all of the exchanges to sort them
        for v in self.asks.values() {
            let mut a = v.clone();
            asks.append(&mut a);
        }

        let mut out: Vec<Level> = Vec::with_capacity(count);

        // grab count number of asks and convert them to Level in order
        for a in asks.iter().take(count) {
            out.push(Level {
                exchange: format!("{}", a.platform),
                price: a.price.to_string(),
                amount: a.quantity.to_string(),
            });
        }
        out
    }

    fn bids(&self, count: usize) -> Vec<Level> {
        let mut bids: BTreeSet<Bid> = BTreeSet::default();

        // merge all of the bids from all of the exchanges to sort them
        for v in self.bids.values() {
            let mut b = v.clone();
            bids.append(&mut b);
        }

        let mut out: Vec<Level> = Vec::with_capacity(count);

        // grab count number of bids and covert them to Level in order
        for b in bids.iter().take(count) {
            out.push(Level {
                exchange: format!("{}", b.platform),
                price: b.price.to_string(),
                amount: b.quantity.to_string(),
            });
        }
        out
    }
}

#[derive(Debug)]
pub struct Aggregator {
    /// handle to the aggregator task
    handle: Option<JoinHandle<Result<()>>>,

    /// shutdown sender to tell our task to exit
    shutdown_tx: mpsc::Sender<()>,

    /// watch channels, one for each configured symbol pair
    watch_rx: HashMap<String, watch::Receiver<Summary>>,

    /// the public key used to check oberon api proofs
    public_key: PublicKey,
}

impl Aggregator {
    /// try to build an Aggregator and start its task
    pub async fn try_build(config: Config) -> Result<Self> {
        // create the watch channels for sending summaries for each market
        let mut watch_rx: HashMap<String, watch::Receiver<Summary>> = HashMap::default();
        let mut watch_tx: HashMap<String, watch::Sender<Summary>> = HashMap::default();
        for market in &config.markets {
            let (tx, rx) = watch::channel(Summary::default());
            watch_rx.insert(market.symbol.to_string(), rx);
            watch_tx.insert(market.symbol.to_string(), tx);
        }

        // create the shutdown channel
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // create the data channel from the sources
        let (data_tx, mut data_rx) = mpsc::channel(64);

        // create the restart channel from the sources
        let (restart_tx, mut restart_rx) = mpsc::channel(config.exchanges.len());

        /*
        // create the watchdog interval stream
        let mut watchdog = IntervalStream::new(interval(Duration::from_secs(
            config.watchdog_timeout.unwrap_or(5),
        )));
        */

        // the data structure for storing all of the data we generate Summary objects from
        let mut orderbooks: HashMap<String, Orderbook> = HashMap::default();

        // create the configured sources
        v1!("AGG - building sources");
        let mut sources: HashMap<Exchange, Source> = HashMap::default();
        for exchange in &config.exchanges {
            if !sources.contains_key(exchange) {
                if let Ok(mut source) =
                    Source::try_build(exchange.clone(), data_tx.clone(), restart_tx.clone()).await
                {
                    v3!("AGG - built {}", exchange.platform);
                    for market in &config.markets {
                        if source.subscribe(&market.symbol).await.is_err() {
                            v3!(
                                "AGG - failed to subscribe {} to {}",
                                exchange.platform,
                                market.symbol
                            );
                        }
                    }

                    // add the source to the map
                    sources.insert(exchange.clone(), source);
                }
            }
        }

        v3!("AGG - spawing aggregator task");
        let handle: JoinHandle<Result<()>> = tokio::spawn(async move {
            loop {
                tokio::select! {
                    // handles orders data coming in from sources
                    Some(orders) = data_rx.recv() => {
                        // create an empty order book if it isn't already in the orderbooks
                        if !orderbooks.contains_key(&orders.symbol) {
                            orderbooks.insert(orders.symbol.clone(), Orderbook::default());
                        }

                        let orderbook = orderbooks.get_mut(&orders.symbol).unwrap();

                        // create empty sorted lists of asks/bids if needed
                        if !orderbook.asks.contains_key(&orders.platform) {
                            orderbook.asks.insert(orders.platform.clone(), BTreeSet::default());
                        }
                        if !orderbook.bids.contains_key(&orders.platform) {
                            orderbook.bids.insert(orders.platform.clone(), BTreeSet::default());
                        }

                        // replace the sorted asks list with the latest
                        orderbook.asks.insert(orders.platform.clone(), orders.asks);

                        // replate the sorted bids list with the latest
                        orderbook.bids.insert(orders.platform.clone(), orders.bids);

                        // set the current value for the Summary on the watch channel
                        if let Some(tx) = watch_tx.get(&orders.symbol) {
                            let spread = orderbook.spread();
                            let summary = Summary {
                                spread: spread.to_string(),
                                bids: orderbook.bids(config.order_limit.unwrap_or(10)),
                                asks: orderbook.asks(config.order_limit.unwrap_or(10)),
                            };
                            if tx.send(summary).is_err() {
                                v3!("AGG - failed to send Summary for {}", &orders.symbol);
                            } else {
                                v1!("AGG - sent summary for {} spread: {}", &orders.symbol, spread);
                            }
                        }
                    }

                    // handles shutdown
                    _ = shutdown_rx.recv() => {
                        v1!("AGG - shutting down");
                        // send the shutdown signal to all of the sources
                        for (e, mut source) in sources.drain() {
                            v1!("AGG - shutting down {}", e.platform);
                            if source.shutdown().await.is_err() {
                                v3!("AGG - failed to shut down {}", e.platform);
                            } else {
                                v1!("AGG - joining {}", e.platform);
                                // wait for the task to stop
                                let _ = source.join().await;
                            }
                            v3!("AGG - {} has exited", e.platform);

                            // force drop the source to free up all of the associated
                            // resources and close the socket
                            drop(source);
                        }
                        v1!("AGG - all sources shut down");
                        return Ok(())
                    },

                    Some(e) = restart_rx.recv() => {
                        v1!("AGG - restarting {}", e.platform);
                        // remove the source from the sources hashmap
                        if let Some(mut source) = sources.remove(&e) {

                            // shut down the source
                            if source.shutdown().await.is_err() {
                                v3!("AGG - failed to shut down {}", e.platform);
                            } else {
                                // wait for the task to stop
                                v1!("AGG - waiting for {} to exit", e.platform);
                                let _ = source.join().await;
                            }

                            // force drop the source to free up all of the associated
                            // resources and close the socket
                            drop(source);
                        }

                        // recreate the source from the exchange
                        v1!("AGG - restarting {}", e.platform);
                        if let Ok(source) = Source::try_build(
                            e.clone(),
                            data_tx.clone(),
                            restart_tx.clone()
                        ).await {
                            v1!("AGG - restarted {}", e.platform);
                            // add the source to the hash map
                            sources.insert(e, source);
                        }
                    }

                    /*
                    // every time the watchdog ticks we check for any crashed sources and
                    // recreate them...
                    _ = watchdog.next() => {
                        // TODO: add some kind of atomic counter for each Exchange and add
                        // a limit on the number of times we try to re-create a Source before
                        // we start trying. Also should probably put an exponential delay in
                        // trying to recreate Sources so that we don't repeatedly hammer
                        // the source with connection attempts.

                        // async closures are still unstable so we can't use map().collect()
                        //
                        // check to see if any sources have crashed
                        v3!("AGG - watchdog timeout");
                        let mut crashed: Vec<Exchange> = Vec::default();
                        for (e, s) in sources.iter() {
                            if !s.running().await {
                                v3!("AGG - {} is not running", e.platform);
                                crashed.push(e.clone());
                            }
                        }

                        // remove and drop each crashed source and then recreate it
                        for e in crashed.iter() {

                            // remove and drop the source
                            if let Some(s) = sources.remove(&e) {
                                v3!("AGG - removing {}", e.platform);
                                drop(s);
                            }

                            // recreate the source from the exchange
                            v1!("AGG - restarting {} after crash", e.platform);
                            if let Ok(source) = Source::try_build(
                                e.clone(),
                                data_tx.clone(),
                                restart_tx.clone()
                            ).await {
                                v1!("AGG - restarted {} after crash", e.platform);
                                // add the source to the hash map
                                sources.insert(e.clone(), source);
                            }
                        }
                    }
                    */
                }
            }
        });

        Ok(Aggregator {
            handle: Some(handle),
            shutdown_tx,
            watch_rx,
            public_key: config.public_key.clone(),
        })
    }

    /// check to see if the Aggregator's task is still running
    pub async fn running(&self) -> bool {
        // this returns true if the receive end of the channel is closed/dropped which
        // will happen when the task exits
        self.shutdown_tx.is_closed()
    }

    /// wait for the Aggregator's task to end
    pub async fn join(&mut self) -> Result<()> {
        if let Some(handle) = self.handle.take() {
            handle.await?
        } else {
            Err(Error::NoJoinHandle)
        }
    }

    /// tell the Aggregator to shut down. This shuts down all of the Sources and cleans up all
    /// resources
    pub async fn shutdown(&self) -> Result<()> {
        self.shutdown_tx
            .send(())
            .await
            .map_err(|_| Error::ShutdownError)
    }

    /// returns the watch::Receiver for the Summary data for the specified symbol
    pub fn get_channel(&self, symbol: &str) -> Result<watch::Receiver<Summary>> {
        self.watch_rx
            .get(&symbol.to_string())
            .cloned()
            .ok_or(Error::NoDataChannel)
    }

    fn check_proof(&self, proof: &Option<OberonProof>) -> Result<()> {
        let proof: ApiProof = proof.as_ref().ok_or(Error::NoOberonProof)?.try_into()?;
        proof.verify(self.public_key)?;
        Ok(())
    }
}

type SummaryStream = Pin<Box<dyn Stream<Item = tonic::Result<Summary, Status>> + Send>>;
type ServiceResult<T> = tonic::Result<Response<T>, Status>;

#[tonic::async_trait]
impl orderbook_aggregator_server::OrderbookAggregator for Aggregator {
    type BookSummaryStream = SummaryStream;

    async fn book_summary(
        &self,
        request: Request<SummaryReq>,
    ) -> ServiceResult<Self::BookSummaryStream> {
        let request = request.into_inner();
        let (tx, rx) = mpsc::channel(64);
        let summary_receiver = ReceiverStream::new(rx);

        if self.check_proof(&request.proof).is_err() {
            return Ok(Response::new(
                Box::pin(summary_receiver) as Self::BookSummaryStream
            ));
        }

        let mut watch_rx = match self.get_channel(&request.symbol) {
            Ok(rx) => rx,
            Err(_) => {
                return Ok(Response::new(
                    Box::pin(summary_receiver) as Self::BookSummaryStream
                ));
            }
        };

        // spawn the task to take the updates from the Summary watch channel and feed them to the
        // gRPC client that requested the stream
        tokio::spawn(async move {
            tokio::select! {
                // this watches for the client to drop their end of the stream
                _ = tx.closed() => {
                    return;
                }
                // this watches for the Summary value to change
                _ = watch_rx.changed() => {
                    let summary = (*watch_rx.borrow_and_update()).clone();
                    let _ = tx.send(tonic::Result::<_, Status>::Ok(summary)).await;
                }

            }
        });
        Ok(Response::new(
            Box::pin(summary_receiver) as Self::BookSummaryStream
        ))
    }
}
