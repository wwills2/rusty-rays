use clap::Parser;
use std::path::PathBuf;
use std::thread;

use rusty_rays_core::logger::{error, info, shutdown_logger, LOG};
use rusty_rays_core::{write_render_to_file, CancellationToken, Model, Tracer};

// Add this dependency in Cargo.toml:
// signal-hook = "0.3"
use signal_hook::consts::signal::{SIGINT, SIGTERM, SIGTSTP};
use signal_hook::iterator::Signals;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input file
    #[arg(short, long, required = true)]
    input_file: PathBuf,

    /// Path to the output file
    #[arg(short, long, default_value = "./render.bmp")]
    output_file: PathBuf,

    /// Number of progress blocks across the whole render.
    /// Example: 10 => ~10% steps, 100 => ~1% steps, 0 => disable progress events/log blocks.
    #[arg(long, default_value_t = 10)]
    progress_blocks: usize,
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

            // Cancellation token shared with signal handler.
            let cancel = CancellationToken::default();

            // Spawn a small signal-listener thread that cancels the render on Ctrl+Z (SIGTSTP),
            // Ctrl+C (SIGINT), or termination (SIGTERM).
            let cancel_for_signals = cancel.clone();
            let signals_thread = thread::spawn(move || {
                // Note: catching SIGTSTP prevents the default "suspend process" behavior.
                let mut signals =
                    Signals::new([SIGTSTP, SIGINT, SIGTERM]).expect("failed to register signals");
                for sig in signals.forever() {
                    match sig {
                        SIGTSTP => {
                            info!(LOG, "received Ctrl+Z (SIGTSTP). canceling render...");
                        }
                        SIGINT => {
                            info!(LOG, "received Ctrl+C (SIGINT). canceling render...");
                        }
                        SIGTERM => {
                            info!(LOG, "received SIGTERM. canceling render...");
                        }
                        _ => {}
                    }
                    cancel_for_signals.cancel();
                    break;
                }
            });

            // Run the render on a background thread so main can respond to signals and then
            // exit only after render returns.
            let progress_blocks = args.progress_blocks;
            let cancel_for_render = cancel.clone();

            let render_thread = thread::spawn(move || {
                // No event channel in the CLI for now; core will still log progress blocks itself.
                tracer.render(Some(cancel_for_render), None, Some(progress_blocks))
            });

            // Wait for render to finish (either completed or canceled)
            let maybe_raw_pixel_colors = render_thread.join().unwrap_or_else(|error| {
                Err(rusty_rays_core::RenderError(
                    "render thread panicked".to_string(),
                ))
            });

            // Make sure the signal thread stops (if render finished normally, we still want to exit cleanly)
            // Dropping the process will end it anyway, but joining avoids dangling threads in some environments.
            let _ = signals_thread.join();

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
                    // Your core returns RenderError("canceled") on cancel — treat that as a normal exit.
                    if error.to_string().contains("canceled") {
                        info!(LOG, "render canceled. exiting.");
                    } else {
                        error!(LOG, "{}", error);
                    }
                }
            }
        }
        Err(error) => {
            error!(LOG, "failed to instantiate model. error: {}", error);
        }
    }

    shutdown_logger();
}
