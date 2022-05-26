mod cli;
mod image_data;
mod mosaic_builder;
mod parallel_mosaic_impl;
mod serial_mosaic_impl;
mod slow_parallel_mosaic_impl;

use crate::cli::{AlgorithmType, CliArgs};
use crate::image_data::ImageData;
use crate::mosaic_builder::{MosaicBuilder, MosaicFactory};
use crate::parallel_mosaic_impl::ParallelMosaicImpl;
use crate::serial_mosaic_impl::SerialMosaicImpl;
use crate::slow_parallel_mosaic_impl::SlowParallelMosaicImpl;

use clap::Parser;
use std::path::Path;

fn main() {
    let cli_args: CliArgs = CliArgs::parse();
    let input_image_path = Path::new(&cli_args.input_image_path);
    if !input_image_path.exists() || !input_image_path.is_file() {
        panic!("Input image does not exist: {}", input_image_path.display());
    }

    match cli_args.algorithm_type {
        AlgorithmType::Serial => run_workflow(
            &MosaicFactory::new(
                Path::new(input_image_path),
                SerialMosaicImpl,
                cli_args.tile_side_length,
            ),
            &cli_args,
        ),
        AlgorithmType::Parallel => run_workflow(
            &MosaicFactory::new(
                Path::new(input_image_path),
                ParallelMosaicImpl,
                cli_args.tile_side_length,
            ),
            &cli_args,
        ),
        AlgorithmType::SlowParallel => run_workflow(
            &MosaicFactory::new(
                Path::new(input_image_path),
                SlowParallelMosaicImpl,
                cli_args.tile_side_length,
            ),
            &cli_args,
        ),
    };
}

fn run_workflow(mosaic_factory: &MosaicFactory<impl MosaicBuilder>, cli_args: &CliArgs) {
    if cli_args.benchmark_runs > 0 {
        println!("Checking algorithm correctness...");
        let correctness_results = mosaic_factory.check_correctness();
        println!("Stage 1 incorrect: {:?}", correctness_results.0);
        println!("Stage 2 incorrect: {:?}", correctness_results.1);
        println!("Stage 3 incorrect: {:?}", correctness_results.2);
        println!("Global average incorrect: {:?}", correctness_results.3);
        println!("\nRunning benchmark...");
        let benchmark_results = mosaic_factory.benchmark(cli_args.benchmark_runs);
        println!("Stage 1: {:?}", benchmark_results.0);
        println!("Stage 2: {:?}", benchmark_results.1);
        println!("Stage 3: {:?}", benchmark_results.2);
        println!(
            "Total time: {:?}",
            benchmark_results.0 + benchmark_results.1 + benchmark_results.2
        );
    }

    if !cli_args.output_image_path.is_empty() {
        match mosaic_factory.generate_and_save_mosaic(&cli_args.output_image_path) {
            Ok(_) => {
                println!(
                    "Successfully generated and saved mosaic at: {}",
                    cli_args.output_image_path
                );
            }
            Err(_) => {
                panic!(
                    "Failed to generate and save mosaic at: {}",
                    cli_args.output_image_path
                );
            }
        }
    }
}
