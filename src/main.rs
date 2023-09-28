use std::process;
use config::Commands;
use config::Config;
use config::Args;
use clap::Parser;

use crate::error::Result;

pub mod error;
pub mod config;
pub mod model;
pub mod crawler;
pub mod run;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let config = Config::build(args.config.as_path()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    if let Err(e) = run_command(args, config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}

async fn run_command(args: Args, config: Config) -> Result<()> {
    match args.command {
        Commands::Create => {
            run::run(config).await?;
            Ok(())
        },
    }
}

