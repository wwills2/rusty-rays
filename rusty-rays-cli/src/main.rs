use clap::Parser;
use std::path::PathBuf;

use rusty_rays_core::logger::{LOG, error, info, shutdown_logger};
use rusty_rays_core::{Model, Tracer, write_render_to_file};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input file
    #[arg(short, long, required = true)]
    input_file: PathBuf,

    /// Path to the output file
    #[arg(short, long, default_value = "./render.bmp")]
    output_file: PathBuf,
}

fn main() {
    let args = Args::parse();

    info!(LOG, "welcome to rusty rays");

    let a_input_file: PathBuf = args.input_file;
    let a_output_file: PathBuf = args.output_file;

    info!(
        LOG,
        "reading input file from {} and writing output file to {}",
        a_input_file.display(),
        a_output_file.display()
    );

    match Model::from_file_path(a_input_file) {
        Ok(model) => {
            info!(LOG, "initialized model from input file");
            let tracer = Tracer::new(model);
            let maybe_raw_pixel_colors = tracer.render();

            match maybe_raw_pixel_colors {
                Ok(raw_pixel_colors) => {
                    info!(LOG, "tracer generated raw image data");
                    match write_render_to_file(&PathBuf::from(&a_output_file), &raw_pixel_colors) {
                        Ok(_) => {
                            info!(LOG, "wrote rendered image to {}", a_output_file.display());
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

    shutdown_logger();
}
