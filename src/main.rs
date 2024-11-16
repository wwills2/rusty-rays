use clap::Parser;

mod tracer;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the input file
    #[arg(short, long)]
    input_file: String,

    /// Path to the output file
    #[arg(short, long, requires = "input_file", default_value = "./")]
    output_file: String,
}

fn main() {
    let args = Args::parse();

    println!(
        "reading input file from {} and writing output file to {}",
        args.input_file, args.output_file
    );

    let init_result = tracer::tracer::Model::new(&args.input_file);
    match init_result {
        Ok(_) => {
            println!("successfully read file");
        }
        Err(error) => {
            println!("failed to read file, error: {}", error);
        }
    }
}
