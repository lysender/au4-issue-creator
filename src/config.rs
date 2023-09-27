use std::fs;
use std::path::Path;
use serde::Deserialize;
use clap::Parser;

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub base_url: String,
    pub project_id: String,
    pub issue_count: u32,
}

impl Config {
    pub fn build(config_file: &str) -> Result<Config, &'static str> {
        let filename = Path::new(config_file);
        let toml_string = match fs::read_to_string(filename) {
            Ok(str) => str,
            Err(_) => {
                return Err("Unable to read config file.");
            }
        };

        let config: Config = match toml::from_str(toml_string.as_str()) {
            Ok(value) => value,
            Err(err) => {
                println!("{:?}", err);
                return Err("Unable to parse config file.");
            }
        };

        if config.issue_count == 0 || config.issue_count > 100 {
            return Err("Issue count must be between 1 to 100");
        }

        Ok(config)
    }
}


/// CLI tool to create issues into a project
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// TOML configuration file
    #[arg(short, long)]
    pub config: String,
}
