#![allow(dead_code)]
use crate::{api_proof::ApiProof, config::Config, error::Error, orderbook::*, Result};
use futures_util::{FutureExt, Stream};
use oberon::PublicKey;
use std::{collections::HashMap, net::SocketAddr, pin::Pin};
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use vlog::{v1, v3, verbose_log};

type SummaryStream = Pin<Box<dyn Stream<Item = tonic::Result<Summary, Status>> + Send>>;
type ServiceResult<T> = tonic::Result<Response<T>, Status>;

#[derive(Debug)]
pub struct AggregatorServer {
    /// watch channels, one for each configured symbol pair
    watch_rx: HashMap<String, watch::Receiver<Summary>>,

    /// the public key used to check oberon api proofs
    public_key: PublicKey,

    /// shutdown sender to tell our task to exit
    shutdown_tx: watch::Sender<()>,
}

impl AggregatorServer {
    pub async fn try_build(
        config: Config,
        watch_rx: HashMap<String, watch::Receiver<Summary>>,
    ) -> Result<Self> {
        // create the shutdown channel
        let (shutdown_tx, _) = watch::channel(());

        Ok(Self {
            watch_rx,
            public_key: config.public_key.clone(),
            shutdown_tx,
        })
    }

    /// Consumes this server and runs the gRPC server
    pub async fn serve(&self, addr: SocketAddr) -> Result<()> {
        v1!("gRPC - server started");
        let _ = tonic::transport::Server::builder()
            .add_service(orderbook_aggregator_server::OrderbookAggregatorServer::new(
                Inner {
                    watch_rx: self.watch_rx.clone(),
                    public_key: self.public_key.clone(),
                    shutdown_rx: self.get_shutdown_rx(),
                },
            ))
            .serve_with_shutdown(addr, self.get_shutdown_rx().changed().map(|_| ()))
            .await;

        Ok(())
    }

    /// tell the Aggregator to shut down. This shuts down all of the Sources and cleans up all
    /// resources
    pub async fn shutdown(&self) -> Result<()> {
        v1!("gRPC - sending shutdown signal to streaming tasks and tonic server");
        self.shutdown_tx.send(()).map_err(|_| Error::ShutdownError)
    }

    fn get_shutdown_rx(&self) -> watch::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
}

#[derive(Clone, Debug)]
struct Inner {
    /// watch channels, one for each configured symbol pair
    watch_rx: HashMap<String, watch::Receiver<Summary>>,

    /// the public key used to check oberon api proofs
    public_key: PublicKey,

    /// shutdown sender to tell our task to exit
    shutdown_rx: watch::Receiver<()>,
}

#[tonic::async_trait]
impl orderbook_aggregator_server::OrderbookAggregator for Inner {
    type BookSummaryStream = SummaryStream;

    async fn book_summary(
        &self,
        request: Request<SummaryReq>,
    ) -> ServiceResult<Self::BookSummaryStream> {
        let request = request.into_inner();
        let (tx, rx) = mpsc::channel(64);
        let summary_receiver = ReceiverStream::new(rx);

        v3!("gRPC - checking Oberon proof");
        if self.check_proof(&request.proof).is_err() {
            return Ok(Response::new(
                Box::pin(summary_receiver) as Self::BookSummaryStream
            ));
        }
        v3!("gRPC - valid Oberon proof");

        let mut watch_rx = match self.get_channel(&request.symbol) {
            Ok(rx) => rx,
            Err(_) => {
                return Ok(Response::new(
                    Box::pin(summary_receiver) as Self::BookSummaryStream
                ));
            }
        };
        v3!("gRPC - got a receive channel");

        let mut shutdown_rx = self.get_shutdown_rx();
        v3!("gRPC - got a shutdown rx channel");

        // spawn the task to take the updates from the Summary watch channel and feed them to the
        // gRPC client that requested the stream
        tokio::spawn(async move {
            tokio::select! {
                // this watches for the shutdown broadcast
                _ = shutdown_rx.changed() => {
                    v1!("gRPC - sending task shutting down");
                    return;
                },
                // this watches for the client to drop their end of the stream
                _ = tx.closed() => {
                    v3!("gRPC - summary sender closing");
                    return;
                }
                // this watches for the Summary value to change
                _ = watch_rx.changed() => {
                    let summary = (*watch_rx.borrow_and_update()).clone();
                    let _ = tx.send(tonic::Result::<_, Status>::Ok(summary)).await;
                    v1!("gRPC - summary sent to client");
                }

            }
        });
        Ok(Response::new(
            Box::pin(summary_receiver) as Self::BookSummaryStream
        ))
    }
}

impl Inner {
    /// returns the watch::Receiver for the Summary data for the specified symbol
    pub fn get_channel(&self, symbol: &str) -> Result<watch::Receiver<Summary>> {
        self.watch_rx
            .get(&symbol.to_string())
            .cloned()
            .ok_or(Error::NoDataChannel)
    }

    fn get_shutdown_rx(&self) -> watch::Receiver<()> {
        self.shutdown_rx.clone()
    }

    fn check_proof(&self, proof: &Option<OberonProof>) -> Result<()> {
        let proof: ApiProof = proof.as_ref().ok_or(Error::NoOberonProof)?.try_into()?;
        proof.verify(self.public_key)?;
        Ok(())
    }
}
