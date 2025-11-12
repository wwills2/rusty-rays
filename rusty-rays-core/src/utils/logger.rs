use std::fs::{self, File};
use std::sync::Mutex;

use crate::utils::Config;
use once_cell::sync::Lazy;
use slog::{Drain, Logger, o};
use slog_async;
use slog_async::AsyncGuard;
use slog_term;

pub use slog::{Level, debug, error, info, trace, warn};

struct AsyncLoggerWithGuard {
    pub logger: Logger,
    pub async_guard: Mutex<Option<AsyncGuard>>,
}

pub static LOG: Lazy<&Logger> = Lazy::new(|| &ASYNC_LOGGER.logger);

static ASYNC_LOGGER: Lazy<AsyncLoggerWithGuard> = Lazy::new(|| {
    let log_level = Config::get().log_level;
    let log_channel_size = Config::get().log_message_cache_overflow_limit;
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let filtered_console_drain = slog::LevelFilter::new(console_drain, log_level).fuse();

    let mut maybe_filtered_file_drain = None;

    if let Some(log_dir) = Config::get().log_files_dir {
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

    let (async_drain, async_guard) = if let Some(filtered_file_drain) = maybe_filtered_file_drain {
        slog_async::Async::new(
            slog::Duplicate::new(filtered_console_drain, filtered_file_drain).fuse(),
        )
        .chan_size(log_channel_size)
        .build_with_guard()
    } else {
        slog_async::Async::new(filtered_console_drain)
            .chan_size(log_channel_size)
            .build_with_guard()
    };

    AsyncLoggerWithGuard {
        logger: Logger::root(async_drain.fuse(), o!()),
        async_guard: Mutex::new(Some(async_guard)),
    }
});

/// It is important to call this function when exiting the program
pub fn shutdown_logger() {
    // flush the async logger - important that this runs
    if let Ok(mut guard) = ASYNC_LOGGER.async_guard.lock() {
        if let Some(guard) = guard.take() {
            drop(guard);
        }
    }
}
