use std::path::PathBuf;
use clap::{arg, Parser};
use slog::{error, info, warn};

use crate::tracer::model::Model;
use crate::tracer::{write, Tracer};
use crate::utils::logger::{ASYNC_LOGGER, LOG};

mod tracer;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Starts Renderer Management Application
    #[arg(short, long, conflicts_with_all = & ["input_file", "output_file"])]
    start: bool,

    /// Path to the input file
    #[arg(
        short,
        long,
        conflicts_with = "start",
        required_unless_present = "start"
    )]
    input_file: Option<PathBuf>,

    /// Path to the output file
    #[arg(short, long, requires = "input_file", default_value = Some("./render.bmp"))]
    output_file: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    info!(LOG, "welcome to rusty rays");

    if args.input_file.is_some() {
        let _input_file: PathBuf = args.input_file.unwrap();
        let _output_file: PathBuf = args.output_file.unwrap();

        info!(
            LOG,
            "reading input file from {} and writing output file to {}", _input_file.display(), _output_file.display()
        );

        match Model::new(_input_file.as_path()) {
            Ok(model) => {
                info!(LOG, "initialized model from input file");
                let tracer = Tracer::new(model);
                let maybe_raw_pixel_colors = tracer.render();

                match maybe_raw_pixel_colors {
                    Ok(raw_pixel_colors) => {
                        info!(LOG, "rendered raw image data");
                        match write(_output_file.as_path(), &raw_pixel_colors) {
                            Ok(_) => {
                                info!(LOG, "wrote rendered image to {}", _output_file.display());
                            },
                            Err(error) => {
                                error!(LOG, "{}", error);
                            }
                        }
                    },
                    Err(error) => {
                        error!(LOG, "{}", error);
                    }
                }
            }
            Err(error) => {
                error!(LOG, "failed to read file, error: {}", error);
            }
        }
    } else if args.start {
        info!(LOG, "when implemented this will start the application");
        todo!()
    } else {
        warn!(LOG, "No functionality matching provided arguments. Exiting");
    }

    // flush the async logger - important that this runs
    if let Ok(mut guard) = ASYNC_LOGGER.async_guard.lock() {
        if let Some(guard) = guard.take() {
            drop(guard);
        }
    }
}
