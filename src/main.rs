use std::env;
use std::process;
use config::Config;

pub mod error;
pub mod config;
pub mod model;
pub mod run;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    if let Err(e) = run::run(config).await {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
