use std::fs;
use std::path::Path;
use serde::{Deserialize};

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub token: String,
    pub base_url: String,
    pub project_id: String,
    pub issue_count: u32,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() != 2 {
            return Err("Usage: issue-creator path/to/config.toml");
        }

        let arg1 = args[1].clone();
        let filename = Path::new(arg1.as_str());
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