use orderbook_server::{
    config::Config,
    error::Error,
    platform::PlatformCmd,
    source::{Source, SourceBuilder, SourceCmd},
    Result,
};
use std::sync::atomic::{AtomicUsize, Ordering};
use structopt::StructOpt;
use tokio::{
    signal,
    sync::{broadcast, mpsc},
};
use vlog::{set_verbosity_level, v3, verbose_log};

static CMD_COUNTER: AtomicUsize = AtomicUsize::new(1);

async fn get_next_num() -> usize {
    CMD_COUNTER.fetch_add(1, Ordering::SeqCst)
}

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
    v3!("verbosity set to: {}", opt.verbosity);

    println!("{:?}", opt.config);

    let (source_tx, _) = broadcast::channel::<SourceCmd>(64);
    let (platform_tx, mut platform_rx) = mpsc::channel::<PlatformCmd>(64);
    let mut sources: Vec<Source> = Vec::with_capacity(2);

    // fire up web sockets for each exchange
    for exchange in &opt.config.exchanges {
        sources.push(
            SourceBuilder::new()
                .with_exchange(exchange.clone())
                .with_source_rx(source_tx.subscribe()) // subscribe creates a Receiver
                .with_platform_tx(platform_tx.clone())
                .try_build()
                .await?,
        );
    }

    for market in &opt.config.markets {
        // send a subscribe message to all sources
        let _ = source_tx
            .send(SourceCmd::Subscribe(market.symbol.clone()))
            .map_err(|_| Error::TokioBroadcastError);
    }

    loop {
        tokio::select! {
            Some(cmd) = platform_rx.recv() => {
                v3!("{} {}", get_next_num().await, cmd);
            },
            _ = signal::ctrl_c() => {
                break;
            },
        }
    }

    // tell the sources to shut down
    let _ = source_tx
        .send(SourceCmd::Shutdown)
        .map_err(|_| Error::TokioBroadcastError);

    // wait on all of the source tasks
    futures::future::join_all(sources).await;

    Ok(())
}
