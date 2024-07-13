use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CLIArgs {
    /// Path to a source image
    pub input_image_path: String,

    /// Type of algorithm to use in the image processing
    #[arg(value_enum, default_value = "serial")]
    pub algorithm_type: AlgorithmType,

    /// Path to save an output file
    #[arg(short, long)]
    pub output_image_path: Option<String>,

    /// Tile side length in pixels
    #[arg(short, long, default_value = "32")]
    pub tile_side_length: u32,

    /// Number of benchmarks iterations to run
    #[arg(short, long)]
    pub benchmark_runs: Option<u32>,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum AlgorithmType {
    Serial,
    Parallel,
    SlowParallel,
}
