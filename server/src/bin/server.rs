use orderbook_server::{config::Config, Result};
use structopt::StructOpt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{client_async_tls, tungstenite::protocol::Message};
use vlog::{set_verbosity_level, v3, verbose_log};

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

    Ok(())
}
