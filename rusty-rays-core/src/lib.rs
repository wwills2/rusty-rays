mod tracer;
mod utils;

use std::path::PathBuf;
use std::sync::OnceLock;

pub use tracer::{
    Color, Cone, Coords, Fov, Model, Plane, Polygon, Screen, Sphere, Surface, Tracer, Triangle,
};
pub use utils::{logger, write_render_to_file, write_render_to_image_buffer, Config};

/// Override the default config directory. This the directory where the config file is stored,
/// not the full path to the config file
pub static CONFIG_DIR_OVERRIDE: OnceLock<PathBuf> = OnceLock::new();
