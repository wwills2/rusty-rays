mod tracer;
mod utils;

use std::path::PathBuf;
use std::sync::OnceLock;

pub use tracer::{Color, Cone, Model, Plane, Polygon, Sphere, Tracer, Triangle};
pub use utils::{
    deserialize_blob_to_raw_render, logger, serialize_raw_render_to_blob, write_render_to_file,
    Config,
};

/// Override the default config directory. This the directory where the config file is stored,
/// not the full path to the config file
pub static CONFIG_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();
