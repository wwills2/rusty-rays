use clap::{arg, Parser};
use slog::{error, info, warn};

use crate::tracer::model;
use crate::tracer::model::Model;
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
    input_file: Option<String>,

    /// Path to the output file
    #[arg(short, long, requires = "input_file", default_value = Some("./render"))]
    output_file: Option<String>,
}

fn main() {
    let args = Args::parse();

    info!(LOG, "welcome to rusty rays");

    if args.input_file.is_some() {
        let _input_file = args.input_file.unwrap();
        let _output_file = args.output_file.unwrap();

        info!(
            LOG,
            "reading input file from {} and writing output file to {}", _input_file, _output_file
        );

        match Model::new(&_input_file) {
            Ok(model) => {
                info!(LOG, "initialized model from input file");
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

    // flush the async logger
    if let Ok(mut guard) = ASYNC_LOGGER.async_guard.lock() {
        if let Some(guard) = guard.take() {
            drop(guard);
        }
    }
}
