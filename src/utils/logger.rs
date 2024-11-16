use std::env::current_dir;
use std::fs::{self, File};
use std::path::PathBuf;

use dirs::home_dir;
use once_cell::sync::Lazy;
use slog::{Drain, Level, Logger, o};
use slog_async;
use slog_term;

pub static GLOBAL_CLI_LOGGER: Lazy<Logger> = Lazy::new(|| {
    let log_level = Level::Info;
    let mut log_dir = current_dir().expect("Failed to determine current directory");
    log_dir.push("logs");

    return init_logger(log_dir, log_level);
});

pub static GLOBAL_APPLICATION_LOGGER: Lazy<Logger> = Lazy::new(|| {
    let log_level = Level::Info;
    let mut log_dir = home_dir().expect("Could not determine home directory");
    log_dir.push(".rusty-rays");
    log_dir.push("logs");

    return init_logger(log_dir, log_level);
});

fn init_logger(log_dir: PathBuf, log_level: Level) -> Logger {
    fs::create_dir_all(&log_dir).expect("Failed to create log directory");

    let log_file_path = log_dir.join(format!("log_{}.log", chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")));
    let log_file = File::create(log_file_path).expect("Failed to create log file");

    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let file_drain = slog_term::CompactFormat::new(slog_term::PlainDecorator::new(log_file)).build().fuse();

    let filtered_console_drain = slog::LevelFilter::new(console_drain, log_level).fuse();
    let filtered_file_drain = slog::LevelFilter::new(file_drain, log_level).fuse();

    let drain = slog_async::Async::new(slog::Duplicate::new(filtered_console_drain, filtered_file_drain).fuse()).build().fuse();

    return Logger::root(drain, o!());
}


