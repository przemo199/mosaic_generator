use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to a source image
    pub input_image_path: String,

    /// Type of algorithm to use in the image processing
    #[clap(arg_enum, default_value = "serial")]
    pub algorithm_type: AlgorithmType,

    /// Path to save an output file
    #[clap(short, long, default_value = "")]
    pub output_image_path: String,

    /// Tile side length in pixels
    #[clap(short, long, default_value = "32")]
    pub tile_side_length: u32,

    /// Number of benchmarks iterations to run
    #[clap(short, long, default_value = "0")]
    pub benchmark_runs: u32,
}

#[derive(clap::ArgEnum, Clone, Debug)]
pub enum AlgorithmType {
    Serial,
    Parallel,
    SlowParallel,
}
