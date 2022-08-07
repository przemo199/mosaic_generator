use crate::{ImageData, SerialMosaicImpl};
use image::{ImageFormat, ImageResult};
use std::ops::{AddAssign, Div};
use std::path::Path;
use std::time::{Duration, Instant};

pub trait MosaicBuilder: Sync {
    /// Adds together channels in pixels belonging to the same tile, each channel is summed to a separate value.
    fn sum_tile_channels(&self, _: &MosaicFactory) -> Vec<u32>;

    /// Calculates the average of each of the channels in a tile. Also calculates global image average.
    fn calc_tile_average(&self, _: &MosaicFactory, _: &[u32]) -> (Vec<u8>, Vec<u8>);

    /// Creates a mosaic from the tile averages.
    fn create_mosaic(&self, _: &MosaicFactory, _: &[u8]) -> Vec<u8>;
}

pub struct MosaicFactory {
    pub tile_side_length: u32,
    pub tile_pixels: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub img_data: ImageData,
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
            img_data: image_data,
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
            sum_tile_channels_time.add_assign(start.elapsed());

            let start = Instant::now();
            let tile_average = self.mosaic_builder.calc_tile_average(self, &tile_sum);
            calc_tile_average_time.add_assign(start.elapsed());

            let start = Instant::now();
            self.mosaic_builder.create_mosaic(self, &tile_average.0);
            create_mosaic_time.add_assign(start.elapsed());
        }

        return (
            sum_tile_channels_time.div(benchmark_runs),
            calc_tile_average_time.div(benchmark_runs),
            create_mosaic_time.div(benchmark_runs),
        );
    }

    pub fn save_mosaic<P: AsRef<Path>>(&self, output_img_path: P, img: &[u8]) -> ImageResult<()> {
        let extension = output_img_path.as_ref().extension();
        let format = extension.and_then(ImageFormat::from_extension);
        match format {
            Some(format) => {
                return image::save_buffer_with_format(
                    output_img_path.as_ref(),
                    img,
                    self.tiles_x * self.tile_side_length,
                    self.tiles_y * self.tile_side_length,
                    self.img_data.color,
                    format,
                );
            }
            None => panic!(
                "Missing or incorrect extension in output image path: {}",
                output_img_path.as_ref().display()
            ),
        }
    }

    pub fn generate_and_save_mosaic<P: AsRef<Path>>(&self, output_img_path: P) -> ImageResult<()> {
        let img = self.generate_mosaic();
        return self.save_mosaic(output_img_path, &img);
    }

    pub fn check_correctness(&self) -> (usize, usize, usize, usize) {
        let serial_stage1 = SerialMosaicImpl.sum_tile_channels(self);
        let serial_stage2 = SerialMosaicImpl.calc_tile_average(self, &serial_stage1);
        let serial_stage3 = SerialMosaicImpl.create_mosaic(self, &serial_stage2.0);
        let new_stage1 = self.mosaic_builder.sum_tile_channels(self);
        let new_stage2 = self.mosaic_builder.calc_tile_average(self, &serial_stage1);
        let new_stage3 = self.mosaic_builder.create_mosaic(self, &serial_stage2.0);

        fn calc_diff<U: PartialEq<W>, W: PartialEq<U>>(vec1: Vec<U>, vec2: Vec<W>) -> usize {
            return vec1.iter().zip(&vec2).filter(|(a, b)| a != b).count();
        }
        let stage1_diff = calc_diff(new_stage1, serial_stage1);
        let stage2_diff = calc_diff(new_stage2.0, serial_stage2.0);
        let global_average_diff = calc_diff(new_stage2.1, serial_stage2.1);
        let stage3_diff = calc_diff(new_stage3, serial_stage3);

        return (stage1_diff, stage2_diff, stage3_diff, global_average_diff);
    }
}
