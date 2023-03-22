use orderbook_server::{
    aggregator::Aggregator, config::Config, grpc::AggregatorServer, orderbook::*, Result,
};
use std::collections::HashMap;
use structopt::StructOpt;
use tokio::{signal, sync::watch};
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

#[tokio::main]
async fn main() -> Result<()> {
    // parse the command line flags
    let opt = Opt::from_args();

    // set the verbosity level
    set_verbosity_level(opt.verbosity);
    v3!("EXE - verbosity set to: {}", opt.verbosity);

    // create the watch channels for sending summaries for each market
    v3!("EXE - creating watch channels to connect aggregator to grpc server");
    let mut watch_rx: HashMap<String, watch::Receiver<Summary>> = HashMap::default();
    let mut watch_tx: HashMap<String, watch::Sender<Summary>> = HashMap::default();
    for market in &opt.config.markets {
        let (tx, rx) = watch::channel(Summary::default());
        watch_rx.insert(market.symbol.to_string(), rx);
        watch_tx.insert(market.symbol.to_string(), tx);
    }

    // create the data aggregator, it manages the sources of
    // market data from the different exchanages
    v1!("EXE - try_build aggregator");
    let mut aggregator = Aggregator::try_build(opt.config.clone(), watch_tx).await?;

    // construct the gRPC server using the aggregator service
    v1!(
        "EXE - starting gRPC server listening on {}",
        &opt.config.listen
    );
    let grpc = AggregatorServer::try_build(opt.config.clone(), watch_rx).await?;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                if grpc.shutdown().await.is_err() {
                    v3!("EXE - failed to shut down gRPC server");
                }
                if aggregator.shutdown().await.is_err() {
                    v3!("EXE - failed to shut down aggregator");
                } else {
                    // wait for the task to stop
                    v1!("EXE - waiting for aggregator to exit");
                    let _ = aggregator.join().await;
                }
                break;
            },
            // run the gRPC server
            _ = grpc.serve(opt.config.listen.clone()) => {}
        }
    }

    v1!("EXE - exiting");

    Ok(())
}
