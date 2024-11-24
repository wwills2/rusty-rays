use std::mem::needs_drop;
use std::process::exit;
use std::ptr::drop_in_place;
use std::thread::sleep;
use std::time::Duration;

use clap::{arg, Parser};
use clap::error::ContextValue::Bool;
use slog::{error, info, Logger, warn};

use crate::tracer::model;
use crate::utils::logger;
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
        let present_input_file = args.input_file.unwrap();
        let present_output_file = args.output_file.unwrap();

        info!(
            LOG,
            "reading input file from {} and writing output file to {}",
            present_input_file,
            present_output_file
        );

        let model_result = model::Model::new(&present_input_file);
        match model_result {
            Err(error) => {
                error!(LOG, "failed to read file, error: {}", error);
                exit(0);
            }
            _ => {}
        }

        let model = model_result.unwrap();
    } else if args.start {
        info!(LOG, "when implemented this will start the application");
        todo!()
    } else {
        warn!(LOG, "No functionality matching provided arguments. Exiting");
    }

    if let Ok(mut guard) = ASYNC_LOGGER.async_guard.lock() {
        if let Some(guard) = guard.take() {
            drop(guard);
        }
    }
}
