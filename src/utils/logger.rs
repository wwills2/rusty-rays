use std::fs::{self, File};
use std::path::PathBuf;
use std::process::exit;

use once_cell::sync::Lazy;
use slog::{o, Drain, Level, Logger};
use slog_async;
use slog_term;

pub static GLOBAL_CLI_LOGGER: Lazy<Logger> = Lazy::new(|| {
    let log_level = Level::Info;
    let mut maybe_log_dir = dirs_next::cache_dir();
    if maybe_log_dir.is_none() {
        println!("Logger failed to access users home cache directory. Cannot start Application");
        println!("To run without logging use the '--cli-no-log' option");
        exit(10);
    }

    let mut log_dir = maybe_log_dir.unwrap();
    let package_name = env!("CARGO_PKG_NAME");
    log_dir.push(package_name);
    log_dir.push("cli");
    log_dir.push("logs");

    let log_dir_option = Option::from(log_dir);
    init_logger(log_dir_option, log_level)
});

pub static GLOBAL_APPLICATION_LOGGER: Lazy<Logger> = Lazy::new(|| {
    let log_level = Level::Info;
    let mut maybe_log_dir = dirs_next::cache_dir();
    if maybe_log_dir.is_none() {
        println!("Logger failed to access users home cache directory. Cannot start Application");
        exit(10);
    }

    let mut log_dir = maybe_log_dir.unwrap();
    let package_name = env!("CARGO_PKG_NAME");
    log_dir.push(package_name);
    log_dir.push("application");
    log_dir.push("logs");

    let log_dir_option = Option::from(log_dir);
    init_logger(log_dir_option, log_level)
});

pub static GLOBAL_CONSOLE_ONLY: Lazy<Logger> = Lazy::new(|| {
    let log_level = Level::Info;
    init_logger(None, log_level)
});

fn init_logger(maybe_log_dir: Option<PathBuf>, log_level: Level) -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let filtered_console_drain = slog::LevelFilter::new(console_drain, log_level).fuse();

    if maybe_log_dir.is_some() {
        let log_dir = maybe_log_dir.unwrap();
        match fs::create_dir_all(&log_dir) {
            Err(error) => {
                eprintln!(
                    "Failed to create log directory {:?}. Error: {}",
                    log_dir.to_str(),
                    error.to_string()
                );
                exit(11);
            }
            _ => {}
        }

        let log_file_path = log_dir.join(format!(
            "log_{}.log",
            chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
        ));
        let log_file = match File::create(&log_file_path) {
            Ok(log_file) => log_file,
            Err(error) => {
                eprintln!(
                    "Failed to create log file {:?}. Error: {}",
                    log_file_path.to_str(),
                    error.to_string()
                );
                exit(12);
            }
        };

        let file_drain = slog_term::CompactFormat::new(slog_term::PlainDecorator::new(log_file))
            .build()
            .fuse();
        let filtered_file_drain = slog::LevelFilter::new(file_drain, log_level).fuse();
        let drain = slog_async::Async::new(
            slog::Duplicate::new(filtered_console_drain, filtered_file_drain).fuse(),
        )
        .build()
        .fuse();

        return Logger::root(drain, o!());
    } else {
        let drain = slog_async::Async::new(filtered_console_drain)
            .build()
            .fuse();
        return Logger::root(drain, o!());
    }
}
