use std::path::PathBuf;

use clap::{Parser, arg};

use rusty_rays_core::logger::{LOG, error, info, shutdown_logger, warn};
use rusty_rays_core::{Model, Tracer, write_render_to_file};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input file
    #[arg(short, long, required = true)]
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
            "reading input file from {} and writing output file to {}",
            _input_file.display(),
            _output_file.display()
        );

        match Model::from_file_path(_input_file) {
            Ok(model) => {
                info!(LOG, "initialized model from input file");
                let tracer = Tracer::new(model);
                let maybe_raw_pixel_colors = tracer.render();

                match maybe_raw_pixel_colors {
                    Ok(raw_pixel_colors) => {
                        info!(LOG, "tracer generated raw image data");
                        match write_render_to_file(&PathBuf::from(&_output_file), &raw_pixel_colors)
                        {
                            Ok(_) => {
                                info!(LOG, "wrote rendered image to {}", _output_file.display());
                            }
                            Err(error) => {
                                error!(LOG, "{}", error);
                            }
                        }
                    }
                    Err(error) => {
                        error!(LOG, "{}", error);
                    }
                }
            }
            Err(error) => {
                error!(LOG, "failed to instantiate model. error: {}", error);
            }
        }
    } else {
        warn!(LOG, "No functionality matching provided arguments. Exiting");
    }

    shutdown_logger();
}
