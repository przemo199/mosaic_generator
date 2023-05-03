# mosaic generator

Mosaic generator is a tool build in Rust that turns provided image into a mosaic.

## Usage

```
Usage: mosaic_generator.exe [OPTIONS] <INPUT_IMAGE_PATH>

Arguments:
  <INPUT_IMAGE_PATH>  Path to a source image

Options:
  -a, --algorithm-type <ALGORITHM_TYPE>
          Type of algorithm to use in the image processing [default: serial] [possible values: serial, parallel, slow-parallel]
  -o, --output-image-path <OUTPUT_IMAGE_PATH>
          Path to save an output file
  -t, --tile-side-length <TILE_SIDE_LENGTH>
          Tile side length in pixels [default: 32]
  -b, --benchmark-runs <BENCHMARK_RUNS>
          Number of benchmarks iterations to run [default: 0]
  -h, --help
          Print help
  -V, --version
          Print version
```
