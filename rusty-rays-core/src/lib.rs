mod tracer;
mod utils;

use utils::logger::ASYNC_LOGGER;

pub use tracer::{Model, Tracer};
pub use utils::logger::LOG;
pub use utils::*;

pub use slog::{debug, error, info, trace, warn};

/// It is important to call this function when exiting the program
pub fn shutdown_logger() {
    // flush the async logger - important that this runs
    if let Ok(mut guard) = ASYNC_LOGGER.async_guard.lock() {
        if let Some(guard) = guard.take() {
            drop(guard);
        }
    }
}
