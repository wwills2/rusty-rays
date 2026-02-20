use clap::Parser;
use std::path::PathBuf;
use std::thread;

use rusty_rays_core::logger::{LOG, error, info, shutdown_logger};
use rusty_rays_core::{Model, Tracer, write_render_to_file};

use rusty_rays_core::CancellationToken;

#[cfg(unix)]
use signal_hook::consts::signal::SIGTSTP;
#[cfg(unix)]
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

            // Cancellation token shared with handlers.
            let cancel = CancellationToken::default();

            // Install signal/console handlers.
            let signals_thread = install_cancel_handlers(cancel.clone());

            // Run the render on a background thread so main can cleanly wait for completion/cancel.
            let progress_blocks = args.progress_blocks;
            let cancel_for_render = cancel.clone();

            let render_thread = thread::spawn(move || {
                // No event channel in the CLI for now; core will still log progress blocks itself.
                tracer.render(Some(cancel_for_render), None, Some(progress_blocks))
            });

            // Wait for render to finish (either completed or canceled)
            let maybe_raw_pixel_colors = render_thread.join().unwrap_or_else(|_error| {
                Err(rusty_rays_core::RenderError(
                    "render thread panicked".to_string(),
                ))
            });

            // If we spawned a Unix SIGTSTP listener thread, join it so we don't leave a dangling thread.
            if let Some(t) = signals_thread {
                let _ = t.join();
            }

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

fn install_cancel_handlers(cancel: CancellationToken) -> Option<std::thread::JoinHandle<()>> {
    // Cross-platform Ctrl+C (and Ctrl+Break on Windows).
    {
        let cancel_for_ctrlc = cancel.clone();
        ctrlc::set_handler(move || {
            info!(LOG, "received Ctrl+C (or Ctrl+Break). canceling render...");
            cancel_for_ctrlc.cancel();
        })
        .expect("failed to set Ctrl+C handler");
    }

    // Unix-only: also cancel on Ctrl+Z (SIGTSTP).
    #[cfg(unix)]
    {
        let cancel_for_sigstp = cancel.clone();
        return Some(thread::spawn(move || {
            // Catching SIGTSTP prevents the default "suspend process" behavior.
            let mut signals = Signals::new([SIGTSTP]).expect("failed to register SIGTSTP");
            for _sig in signals.forever() {
                info!(LOG, "received Ctrl+Z (SIGTSTP). canceling render...");
                cancel_for_sigstp.cancel();
                break;
            }
        }));
    }

    #[cfg(not(unix))]
    {
        None
    }
}
