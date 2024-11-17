use std::process::exit;

use clap::error::ContextValue::Bool;
use clap::Parser;
use slog::{error, info, warn};

use crate::utils::logger;

mod tracer;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Starts Renderer Management Application
    #[arg(short, long, conflicts_with_all = & ["input_file", "output_file",  /* "cli_no_log" */])]
    start: bool,

    /// Path to the input file
    #[arg(short, long)]
    input_file: String,

    /// Path to the output file
    #[arg(short, long, requires = "input_file", default_value = "./")]
    output_file: String,
}

fn main() {
    let args = Args::parse();
    let logger = &logger::GLOBAL_LOGGER;

    if Some(&args.input_file).is_some() {
        info!(
            logger,
            "reading input file from {} and writing output file to {}",
            args.input_file,
            args.output_file
        );

        let model_result = tracer::tracer::Model::new(&args.input_file);
        match model_result {
            Err(error) => {
                error!(logger, "failed to read file, error: {}", error);
                exit(0);
            }
            _ => {}
        }

        let model = model_result.unwrap();
    } else if args.start {
        info!(logger, "when implemented this will start the application");
    } else {
        warn!(
            logger,
            "No functionality matching provided arguments. Exiting"
        );
    }
}
