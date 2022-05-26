# mosaic generator

Mosaic generator is a tool build in Rust that turns a given image into a mosaic.

## Usage

```
USAGE:
    mosaic_generator.exe [OPTIONS] <INPUT_IMAGE_PATH> [ALGORITHM_TYPE]

ARGS:
    <INPUT_IMAGE_PATH>    Path to a source image
    <ALGORITHM_TYPE>      Type of algorithm to use in the image processing [default: serial]
                          [possible values: serial, parallel, slow-parallel]

OPTIONS:
    -b, --benchmark-runs <BENCHMARK_RUNS>
            Number of benchmarks iterations to run [default: 0]

    -h, --help
            Print help information

    -o, --output-image-path <OUTPUT_IMAGE_PATH>
            Path to save an output file [default: ]

    -t, --tile-side-length <TILE_SIDE_LENGTH>
            Tile side length in pixels [default: 32]

    -V, --version
            Print version information
```
