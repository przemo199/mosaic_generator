use std::path::Path;

use clap::Parser;

use crate::args::{AlgorithmType, Args};
use crate::image_data::ImageData;
use crate::mosaic_factory::{MosaicBuilder, MosaicFactory};
use crate::parallel_mosaic::ParallelMosaic;
use crate::serial_mosaic::SerialMosaic;
use crate::slow_parallel_mosaic::SlowParallelMosaic;

mod args;
mod image_data;
mod mosaic_factory;
mod parallel_mosaic;
mod serial_mosaic;
mod slow_parallel_mosaic;

fn main() {
    let cli_args: Args = Args::parse();
    let input_image_path = Path::new(&cli_args.input_image_path);
    if !input_image_path.exists() || !input_image_path.is_file() {
        panic!("Input image does not exist: {}", input_image_path.display());
    }

    let mosaic_builder: Box<dyn MosaicBuilder> = match cli_args.algorithm_type {
        AlgorithmType::Serial => Box::new(SerialMosaic),
        AlgorithmType::Parallel => Box::new(ParallelMosaic),
        AlgorithmType::SlowParallel => Box::new(SlowParallelMosaic),
    };

    let mosaic_factory = MosaicFactory::new(
        Path::new(input_image_path),
        mosaic_builder,
        cli_args.tile_side_length,
    );

    run_workflow(&mosaic_factory, &cli_args);
}

fn run_workflow(mosaic_factory: &MosaicFactory, cli_args: &Args) {
    if cli_args.benchmark_runs > 0 {
        println!("Checking algorithm correctness...");
        let correctness_results = mosaic_factory.check_correctness();
        println!("Stage 1 incorrect values: {:?}", correctness_results.0);
        println!("Stage 2 incorrect values: {:?}", correctness_results.1);
        println!("Stage 3 incorrect values: {:?}", correctness_results.2);
        println!("Global average incorrect values: {:?}", correctness_results.3);
        println!("\nBenchmarking...");
        let benchmark_results = mosaic_factory.benchmark(cli_args.benchmark_runs);
        println!("Stage 1 time: {:?}", benchmark_results.0);
        println!("Stage 2 time: {:?}", benchmark_results.1);
        println!("Stage 3 time: {:?}", benchmark_results.2);
        println!(
            "Total time: {:?}",
            benchmark_results.0 + benchmark_results.1 + benchmark_results.2
        );
    }

    match &cli_args.output_image_path {
        Some(path) => {
            match mosaic_factory.generate_and_save_mosaic(path) {
                Ok(_) => println!("Successfully generated and saved mosaic at: {}", path),
                Err(_) => panic!("Failed to save mosaic at: {}", path)
            }
        }
        None => println!("Result discarded, no output path provided")
    }
}
