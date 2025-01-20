use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json5;
use slog::Level;

/// the logger is dependent on this file

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub log_level: Level,
    pub log_message_cache_overflow_limit: usize,
    pub max_render_threads: usize,
    pub loaded_from_file: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ParsedConfig {
    log_level: Option<String>,
    log_message_cache_overflow_limit: Option<usize>,
    pub max_render_threads: Option<usize>,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| match get_config_file_content_string() {
    Ok(config_json5) => match serde_json5::from_str::<ParsedConfig>(config_json5.as_str()) {
        Ok(parsed_config) => Config {
            log_level: match parsed_config.log_level {
                Some(parsed_log_level) => match Level::from_str(parsed_log_level.as_str()) {
                    Ok(log_level) => log_level,
                    Err(_) => {
                        eprintln!(
                            "{} is not a valid log level (off, error, warn, info, debug, trace)",
                            parsed_log_level
                        );
                        println!("using {} log level", DEFAULT_CONFIG.log_level);
                        Level::Info
                    }
                },
                None => {
                    eprintln!("config file does not contain a log level");
                    println!("using {} log level", DEFAULT_CONFIG.log_level);
                    Level::Info
                }
            },
            log_message_cache_overflow_limit: parsed_config
                .log_message_cache_overflow_limit
                .unwrap_or_else(|| DEFAULT_CONFIG.log_message_cache_overflow_limit),
            max_render_threads: match parsed_config.max_render_threads {
                Some(max_threads) => {
                    if max_threads > 0 && max_threads <= DEFAULT_CONFIG.max_render_threads {
                        max_threads
                    } else {
                        DEFAULT_CONFIG.max_render_threads
                    }
                }
                None => DEFAULT_CONFIG.max_render_threads,
            },
            loaded_from_file: true,
        },
        Err(error) => {
            eprintln!("failed parse config json5: {}", error);
            println!("using default config");

            DEFAULT_CONFIG.clone()
        }
    },
    Err(error) => {
        eprintln!("failed to find or open config file. {}", error);

        match create_config_file() {
            Err(error) => {
                eprintln!("failed to create config file. {}", error);
            }
            _ => {}
        };

        println!("using default config");

        DEFAULT_CONFIG.clone()
    }
});

static DEFAULT_CONFIG: Lazy<Config> = Lazy::new(|| Config {
    log_level: Level::Info,
    log_message_cache_overflow_limit: 500_000,
    max_render_threads: num_cpus::get_physical(),
    loaded_from_file: false,
});

fn get_config_dir() -> Result<PathBuf, String> {
    match dirs_next::config_dir() {
        Some(mut config_dir) => {
            let package_name = env!("CARGO_PKG_NAME");
            config_dir.push(package_name);
            Ok(config_dir)
        }
        None => Err("cannot find or access user config directory.".to_string()),
    }
}
fn get_config_file_path() -> Result<PathBuf, String> {
    match get_config_dir() {
        Ok(mut config_dir) => {
            config_dir.push("config.json5");
            Ok(config_dir)
        }
        Err(err_string) => Err(err_string),
    }
}

fn get_config_file_content_string() -> Result<String, String> {
    let config_file_path = get_config_file_path()?;
    match fs::read_to_string(config_file_path) {
        Ok(config_file_contents_string) => Ok(config_file_contents_string),
        Err(error) => Err(format!("cannot open config file. Error: {}", error)),
    }
}

fn create_config_file() -> Result<(), String> {
    let config_dir_path: PathBuf = get_config_dir()?;
    let create_file_result = fs::create_dir_all(&config_dir_path);
    if create_file_result.is_err() {
        return Err(format!(
            "cannot create config file. Error: {}",
            create_file_result.unwrap_err()
        ));
    }

    let config_file_path = get_config_file_path()?;

    match File::create_new(&config_file_path) {
        Ok(mut file) => {
            let comment = "// this file is json5. comments are allowed, key quotes not needed\n";
            let config = ParsedConfig {
                log_level: Some(DEFAULT_CONFIG.log_level.to_string()),
                log_message_cache_overflow_limit: Some(
                    DEFAULT_CONFIG.log_message_cache_overflow_limit,
                ),
                max_render_threads: None, // set null
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
                Ok(_) => {
                    println!("created config file {}", config_file_path.to_str().unwrap());
                    Ok(())
                }
                Err(error) => Err(format!("cannot write config to file. Error: {}", error)),
            }
        }
        Err(error) => Err(format!("cannot open config file. Error: {}", error)),
    }
}
