use std::path::PathBuf;

use clap::{arg, Parser};

use rusty_rays_core::{
    deserialize_blob_to_raw_render, error, info, serialize_raw_render_to_blob, shutdown_logger, warn, write_render_to_file,
    Model, Tracer, LOG,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
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

                        // the next two steps for serialization and deserialization are not necessary
                        // this is just to test functionality
                        info!(LOG, "testing raw image data serialization");
                        let serialized_raw_image =
                            match serialize_raw_render_to_blob(&raw_pixel_colors) {
                                Ok(serialized_raw_image) => serialized_raw_image,
                                Err(error) => {
                                    error!(LOG, "failed to encode raw image. {}", error);
                                    return;
                                }
                            };

                        info!(LOG, "testing serialized raw image data deserialization");
                        let deserialized_raw_image =
                            match deserialize_blob_to_raw_render(&serialized_raw_image) {
                                Ok(deserialized_raw_image) => deserialized_raw_image,
                                Err(error) => {
                                    error!(LOG, "failed to decode raw image. {}", error);
                                    return;
                                }
                            };

                        match write_render_to_file(_output_file.as_path(), &deserialized_raw_image)
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
