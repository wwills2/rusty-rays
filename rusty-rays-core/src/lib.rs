mod tracer;
mod utils;

use std::path::PathBuf;
use std::sync::OnceLock;
use utils::ASYNC_LOGGER;

pub use tracer::{Model, Tracer};
pub use utils::{
    deserialize_blob_to_raw_render, serialize_raw_render_to_blob, write_render_to_file, Config, LOG,
};

pub use slog::{debug, error, info, trace, warn, Level};

/// Override the default config directory. This the directory where the config file is stored,
/// not the full path to the config file
pub static CONFIG_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();

/// It is important to call this function when exiting the program
pub fn shutdown_logger() {
    // flush the async logger - important that this runs
    if let Ok(mut guard) = ASYNC_LOGGER.async_guard.lock() {
        if let Some(guard) = guard.take() {
            drop(guard);
        }
    }
}
