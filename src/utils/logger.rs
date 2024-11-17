use std::fs::{self, File};

use once_cell::sync::Lazy;
use slog::{Drain, Level, Logger, o};
use slog_async;
use slog_term;

pub static GLOBAL_LOGGER: Lazy<Logger> = Lazy::new(|| {
    let log_level = Level::Info;
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let filtered_console_drain = slog::LevelFilter::new(console_drain, log_level).fuse();

    let maybe_log_dir = dirs_next::cache_dir();
    let mut maybe_filtered_file_drain = None;

    if maybe_log_dir.is_some() {
        let mut log_dir = maybe_log_dir.unwrap();
        let package_name = env!("CARGO_PKG_NAME");
        log_dir.push(package_name);
        log_dir.push("application");
        log_dir.push("logs");

        match fs::create_dir_all(&log_dir) {
            Ok(_) => {
                let log_file_path = log_dir.join(format!(
                    "log_{}.log",
                    chrono::Local::now().format("%Y-%m-%d_%H-%M-%S")
                ));

                match File::create(&log_file_path) {
                    Ok(log_file) => {
                        let file_drain =
                            slog_term::CompactFormat::new(slog_term::PlainDecorator::new(log_file))
                                .build()
                                .fuse();
                        maybe_filtered_file_drain =
                            Some(slog::LevelFilter::new(file_drain, log_level).fuse());
                    }
                    Err(error) => {
                        eprintln!(
                            "Failed to create log file {:?}. Error: {}",
                            log_file_path.to_str(),
                            error
                        );
                    }
                };
            }
            Err(error) => {
                eprintln!(
                    "Logger failed to access users home cache directory. Error: {}",
                    error
                );
            }
        }
    }

    let drain = if maybe_filtered_file_drain.is_none() {
        slog_async::Async::new(filtered_console_drain)
            .build()
            .fuse()
    } else {
        let filtered_file_drain = maybe_filtered_file_drain.unwrap();
        slog_async::Async::new(
            slog::Duplicate::new(filtered_console_drain, filtered_file_drain).fuse(),
        )
        .build()
        .fuse()
    };
    Logger::root(drain, o!())
});
