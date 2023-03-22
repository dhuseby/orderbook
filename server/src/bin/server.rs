use orderbook_server::{aggregator::Aggregator, config::Config, Result};
use structopt::StructOpt;
use tokio::signal;
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

    // create the data aggregator, it manages the sources of
    // market data from the different exchanages
    v1!("EXE - try_build aggregator");
    let mut aggregator = Aggregator::try_build(opt.config.clone()).await?;

    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                if aggregator.shutdown().await.is_err() {
                    v3!("EXE - failed to shut down aggregator");
                } else {
                    // wait for the task to stop
                    v1!("EXE - waiting for aggregator to exit");
                    let _ = aggregator.join().await;
                }
                break;
            },
        }
    }

    v1!("EXE - exiting");

    Ok(())
}
