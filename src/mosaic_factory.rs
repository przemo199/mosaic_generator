use crate::{ImageData, SerialMosaic};
use image::{ImageFormat, ImageResult};
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

pub trait MosaicBuilder: Sync {
    /// Adds together channels in pixels belonging to the same tile, each channel is summed to a separate value.
    fn sum_tile_channels(&self, mosaic_factory: &MosaicFactory) -> Vec<u32>;

    /// Calculates the average of each of the channels in a tile. Also calculates global image average.
    fn calc_tile_average(&self, mosaic_factory: &MosaicFactory, _: &[u32]) -> (Vec<u8>, Vec<u8>);

    /// Creates a mosaic from the tile averages.
    fn create_mosaic(&self, mosaic_factory: &MosaicFactory, _: &[u8]) -> Vec<u8>;
}

pub struct MosaicFactory {
    pub tile_side_length: u32,
    pub tile_pixels: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub image_data: ImageData,
    pub mosaic_builder: Box<dyn MosaicBuilder>,
}

impl MosaicFactory {
    pub fn new<P: AsRef<Path>>(
        input_image_path: P,
        mosaic_builder: Box<dyn MosaicBuilder>,
        tile_side_length: u32,
    ) -> MosaicFactory {
        let image_data = ImageData::from_path(input_image_path, tile_side_length);
        return MosaicFactory {
            tile_side_length,
            tile_pixels: tile_side_length * tile_side_length,
            tiles_x: image_data.width / tile_side_length,
            tiles_y: image_data.height / tile_side_length,
            image_data,
            mosaic_builder,
        };
    }

    pub fn generate_mosaic(&self) -> Vec<u8> {
        let tile_sum = self.mosaic_builder.sum_tile_channels(self);
        let average_results = self.mosaic_builder.calc_tile_average(self, &tile_sum);
        println!("Image global average: {:?}", average_results.1);
        return self.mosaic_builder.create_mosaic(self, &average_results.0);
    }

    pub fn benchmark(&self, benchmark_runs: u32) -> (Duration, Duration, Duration) {
        let mut sum_tile_channels_time = Duration::new(0, 0);
        let mut calc_tile_average_time = Duration::new(0, 0);
        let mut create_mosaic_time = Duration::new(0, 0);

        for _ in 0..benchmark_runs {
            let start = Instant::now();
            let tile_sum = self.mosaic_builder.sum_tile_channels(self);
            sum_tile_channels_time += start.elapsed();

            let start = Instant::now();
            let tile_average = self.mosaic_builder.calc_tile_average(self, &tile_sum);
            calc_tile_average_time += start.elapsed();

            let start = Instant::now();
            self.mosaic_builder.create_mosaic(self, &tile_average.0);
            create_mosaic_time += start.elapsed();
        }

        return (
            sum_tile_channels_time / benchmark_runs,
            calc_tile_average_time / benchmark_runs,
            create_mosaic_time / benchmark_runs,
        );
    }

    fn prepare_file<P: AsRef<Path>>(file_name: &P) -> io::Result<()> {
        let path = Path::new(file_name.as_ref());
        let prefix = path
            .parent()
            .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
        return std::fs::create_dir_all(prefix);
    }

    pub fn save_mosaic<P: AsRef<Path>>(&self, output_img_path: &P, img: &[u8]) -> ImageResult<()> {
        Self::prepare_file(output_img_path)?;

        let extension = output_img_path.as_ref().extension();
        let format = extension.and_then(ImageFormat::from_extension);
        match format {
            Some(format) => {
                return image::save_buffer_with_format(
                    output_img_path.as_ref(),
                    img,
                    self.tiles_x * self.tile_side_length,
                    self.tiles_y * self.tile_side_length,
                    self.image_data.color,
                    format,
                );
            }
            None => panic!(
                "Missing or incorrect extension in output image path: {}",
                output_img_path.as_ref().display()
            ),
        }
    }

    pub fn generate_and_save_mosaic<P: AsRef<Path>>(&self, output_img_path: &P) -> ImageResult<()> {
        let img = self.generate_mosaic();
        return self.save_mosaic(&output_img_path, &img);
    }

    pub fn check_correctness(&self) -> (usize, usize, usize, usize) {
        let serial_stage_1 = SerialMosaic.sum_tile_channels(self);
        let serial_stage_2 = SerialMosaic.calc_tile_average(self, &serial_stage_1);
        let serial_stage_3 = SerialMosaic.create_mosaic(self, &serial_stage_2.0);
        let new_stage_1 = self.mosaic_builder.sum_tile_channels(self);
        let new_stage_2 = self.mosaic_builder.calc_tile_average(self, &serial_stage_1);
        let new_stage_3 = self.mosaic_builder.create_mosaic(self, &serial_stage_2.0);

        fn calc_diff<U: PartialEq<W>, W: PartialEq<U>>(vec_1: Vec<U>, vec_2: Vec<W>) -> usize {
            return vec_1.iter().zip(&vec_2).filter(|(a, b)| a != b).count();
        }
        let stage_1_diff = calc_diff(new_stage_1, serial_stage_1);
        let stage_2_diff = calc_diff(new_stage_2.0, serial_stage_2.0);
        let global_average_diff = calc_diff(new_stage_2.1, serial_stage_2.1);
        let stage_3_diff = calc_diff(new_stage_3, serial_stage_3);

        return (
            stage_1_diff,
            stage_2_diff,
            stage_3_diff,
            global_average_diff,
        );
    }
}
