use std::process;
use config::Config;
use config::Args;
use clap::Parser;

pub mod error;
pub mod config;
pub mod model;
pub mod run;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::build(args.config.as_str()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    if let Err(e) = run::run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
