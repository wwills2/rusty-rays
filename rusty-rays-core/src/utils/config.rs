/// The logger is dependent on this file
use crate::CONFIG_DIR_OVERRIDE;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use slog::Level;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::RwLock;

static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| RwLock::new(init_config()));

#[derive(Debug, PartialEq, Clone)]
pub struct Config {
    pub log_level: Level,
    pub log_files_dir: Option<PathBuf>,
    pub log_message_cache_overflow_limit: usize,
    pub max_render_threads: usize,
    pub loaded_from_file: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ParsedConfig {
    log_level: Option<String>,
    log_files_dir: Option<PathBuf>,
    log_message_cache_overflow_limit: Option<usize>,
    max_render_threads: Option<usize>,
}

static DEFAULT_CONFIG: Lazy<Config> = Lazy::new(|| {
    let log_dir: Option<PathBuf> = match dirs_next::cache_dir() {
        Some(mut user_cache_dir) => {
            let package_name = env!("CARGO_PKG_NAME");
            user_cache_dir.push(package_name);
            user_cache_dir.push("logs");

            Some(user_cache_dir)
        }
        None => {
            eprintln!("cannot find or access user cache directory.");
            None
        }
    };

    Config {
        log_level: Level::Info,
        log_files_dir: log_dir,
        log_message_cache_overflow_limit: 500_000,
        max_render_threads: num_cpus::get_physical(),
        loaded_from_file: false,
    }
});

impl Default for Config {
    fn default() -> Self {
        DEFAULT_CONFIG.clone()
    }
}

impl Config {
    pub fn get() -> Config {
        let config_read_result = CONFIG.read();
        if let Ok(config) = config_read_result {
            return config.clone();
        };

        eprintln!("failed to read in-memory config. attempting to read from file");
        init_config()
    }

    pub fn set(updated_config: Config) -> Result<(), String> {
        match CONFIG.write() {
            Ok(mut config) => {
                config.log_level = updated_config.log_level;
                config.log_message_cache_overflow_limit =
                    updated_config.log_message_cache_overflow_limit;
                config.max_render_threads = updated_config.max_render_threads;
                config.loaded_from_file = updated_config.loaded_from_file;

                write_config_to_file(updated_config)
            }
            Err(error) => Err(format!("failed to set config. Error: {}", error)),
        }
    }
}

fn init_config() -> Config {
    match get_config_file_content_string() {
        Ok(config_json) => parse_config_file_content(config_json),
        Err(error) => {
            eprintln!("failed to find or open config file. {}", error);

            let config = ParsedConfig {
                log_level: Some(DEFAULT_CONFIG.log_level.to_string()),
                log_message_cache_overflow_limit: Some(
                    DEFAULT_CONFIG.log_message_cache_overflow_limit,
                ),
                log_files_dir: DEFAULT_CONFIG.log_files_dir.clone(),
                max_render_threads: None, // set null
            };

            if let Err(create_file_error) = create_config_file(config) {
                eprintln!("failed to create config file. {}", create_file_error);
            };

            println!("using default config");

            DEFAULT_CONFIG.clone()
        }
    }
}

fn parse_config_file_content(config_json: String) -> Config {
    match serde_json::from_str::<ParsedConfig>(config_json.as_str()) {
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
            log_files_dir: parsed_config.log_files_dir,
            loaded_from_file: true,
        },
        Err(error) => {
            eprintln!("failed parse config json: {}", error);
            println!("using default config");

            DEFAULT_CONFIG.clone()
        }
    }
}

fn get_config_dir() -> Result<PathBuf, String> {
    if let Some(custom_config_dir) = CONFIG_DIR_OVERRIDE.get() {
        return Ok(custom_config_dir.clone());
    }

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
            config_dir.push("config.json");
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

fn write_config_to_file(config: Config) -> Result<(), String> {
    let writeable_config = ParsedConfig {
        log_level: Some(config.log_level.to_string()),
        log_message_cache_overflow_limit: Some(config.log_message_cache_overflow_limit),
        log_files_dir: DEFAULT_CONFIG.log_files_dir.clone(),
        max_render_threads: Some(config.max_render_threads),
    };

    let config_file_path = match get_config_file_path() {
        Ok(config_file_path) => config_file_path,
        Err(err_string) => {
            eprintln!("cannot open config file. Error: {}", err_string);
            return create_config_file(writeable_config);
        }
    };

    let config_file_contents = match serde_json::to_string_pretty(&writeable_config) {
        Ok(config_file_contents) => config_file_contents,
        Err(error) => return Err(format!("cannot serialize config. Error: {}", error)),
    };

    match fs::write(config_file_path, config_file_contents) {
        Ok(_) => Ok(()),
        Err(error) => Err(format!("cannot write config to file. Error: {}", error)),
    }
}

fn create_config_file(config: ParsedConfig) -> Result<(), String> {
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
            let config_contents_result = serde_json::to_string_pretty(&config);
            if config_contents_result.is_err() {
                return Err(format!(
                    "cannot serialize config. Error: {}",
                    config_contents_result.unwrap_err()
                ));
            }

            let config_file_contents = config_contents_result.unwrap();
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
