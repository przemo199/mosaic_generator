mod cli;
mod config;
mod image_data;
mod mosaic;
mod mosaic_controller;

use crate::cli::CliArgs;
use crate::image_data::ImageData;
use crate::mosaic::BasicMosaicAlgorithm;
use crate::mosaic_controller::MosaicBuilder;

use clap::Parser;
use image::ImageFormat;
use std::path::Path;

fn main() {
    let cli_args: CliArgs = CliArgs::parse();
    let input_image_path = Path::new(&cli_args.input_image_path);
    if !input_image_path.exists() || !input_image_path.is_file() {
        panic!("Input image does not exist: {}", input_image_path.display());
    }

    let img_data = ImageData::from_path(Path::new(input_image_path));
    let mosaic_algorithm = BasicMosaicAlgorithm::new(img_data);
    let mosaic_factory = MosaicBuilder::new(mosaic_algorithm);

    if cli_args.benchmark {
        println!("Running benchmark...");
        println!(
            "{:?}",
            mosaic_factory.benchmark(config::BENCHMARK_RUNS as u64)
        );
    }

    if !cli_args.output_image_path.is_empty() {
        let extension = Path::new(&cli_args.output_image_path).extension();
        match extension {
            Some(ext) => {
                let extension = ext;
            }
            None => panic!(
                "Missing extension on output image path: {}",
                cli_args.output_image_path
            ),
        }
        let output_file_format =
            ImageFormat::from_extension(Path::new(&cli_args.output_image_path).extension());
    }
}
