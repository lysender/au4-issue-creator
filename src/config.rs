use std::{fs, path::PathBuf};
use std::path::Path;
use serde::Deserialize;
use clap::{Parser, Subcommand};

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub base_url: String,
    pub project_id: String,
    pub issue_count: u32,
}

impl Config {
    pub fn build(filename: &Path) -> Result<Config, &'static str> {
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
    #[arg(short, long, value_name = "FILE.toml")]
    pub config: PathBuf,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create issues into project specified in config file
    Create,

    /// Crawl all issues of the specified project
    CrawlIssues,

    /// Craw all issues from all visible projects
    CrawlAllIssues,
}

