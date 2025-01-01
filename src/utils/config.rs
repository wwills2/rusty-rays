use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize, Serializer};
use serde_json5;
use slog::Level;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

/// the logger is dependent on this file

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub log_level: Level,
    pub log_message_cache_overflow_limit: usize,
    pub loaded_from_file: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ParsedConfig {
    log_level: String,
    log_message_cache_overflow_limit: usize,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| match get_config_file_string() {
    Ok(config_json5) => match serde_json5::from_str::<ParsedConfig>(config_json5.as_str()) {
        Ok(parsed_config) => Config {
            log_level: match Level::from_str(parsed_config.log_level.as_str()) {
                Ok(log_level) => log_level,
                Err(_) => {
                    eprintln!(
                        "{} is not a valid log level (off, error, warn, info, debug, trace)",
                        parsed_config.log_level
                    );
                    println!("using \"INFO\" log level");
                    Level::Info
                }
            },
            log_message_cache_overflow_limit: parsed_config.log_message_cache_overflow_limit,
            loaded_from_file: true,
        },
        Err(error) => {
            eprintln!("failed parse config json5: {}", error);
            println!("using default config");

            DEFAULT_CONFIG.clone()
        }
    },
    Err(error) => {
        eprintln!("Failed to read config file: {}", error);
        println!("using default config");

        DEFAULT_CONFIG.clone()
    }
});

static DEFAULT_CONFIG: Lazy<Config> = Lazy::new(|| Config {
    log_level: Level::Info,
    log_message_cache_overflow_limit: 500_000,
    loaded_from_file: false,
});

fn get_config_file_path() -> Result<PathBuf, String> {
    match dirs::config_dir() {
        Some(mut config_dir) => {
            let package_name = env!("CARGO_PKG_NAME");
            config_dir.push(package_name);
            config_dir.push("config.json5");
            Ok(config_dir)
        }
        None => Err("cannot find or access user config directory.".to_string()),
    }
}

fn get_config_file_string() -> Result<String, String> {
    let config_file_path = get_config_file_path()?;
    match fs::read_to_string(config_file_path) {
        Ok(config_file_contents_string) => Ok(config_file_contents_string),
        Err(error) => Err(format!("cannot open config file. Error: {}", error)),
    }
}

fn create_config_file() -> Result<(), String> {
    let config_file_path = get_config_file_path()?;
    let create_file_result = fs::create_dir(&config_file_path);
    if create_file_result.is_err() {
        return Err(format!(
            "cannot create config file. Error: {}",
            create_file_result.unwrap_err()
        ));
    }

    match File::open(&config_file_path) {
        Ok(mut file) => {
            let comment = "this file is json5. comments are allowed, key quotes not needed";
            let config = ParsedConfig {
                log_level: DEFAULT_CONFIG.log_level.to_string(),
                log_message_cache_overflow_limit: DEFAULT_CONFIG.log_message_cache_overflow_limit,
            };
            let config_contents_result = serde_json5::to_string(&config);
            if config_contents_result.is_err() {
                return Err(format!(
                    "cannot serialize config. Error: {}",
                    config_contents_result.unwrap_err()
                ));
            }

            let config_file_contents = config_contents_result.unwrap();
            let _ignored = file.write_all(comment.as_bytes());
            match file.write_all(config_file_contents.as_bytes()) {
                Ok(_) => Ok(()),
                Err(error) => Err(format!("cannot write config to file. Error: {}", error)),
            }
        }
        Err(error) => Err(format!("cannot open config file. Error: {}", error)),
    }
}
