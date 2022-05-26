use crate::ImageData;
use image::{ImageFormat, ImageResult};
use std::ops::{AddAssign, Div};
use std::path::Path;
use std::time::{Duration, Instant};

pub trait Mosaicable {
    /// Adds together channels in pixels belonging to the same tile, each channel is summed to a separate value.
    fn sum_tile_channels(mosaic_controller: &MosaicBuilder<impl Mosaicable>) -> Vec<u32>;
    /// Calculates the average of each of the channels in a tile. Also calculates global image average.
    fn calc_tile_average(
        mosaic_controller: &MosaicBuilder<impl Mosaicable>,
        tile_sum: &[u32],
    ) -> (Vec<u8>, Vec<u8>);
    /// Creates a mosaic from the tile averages.
    fn create_mosaic(
        mosaic_controller: &MosaicBuilder<impl Mosaicable>,
        tile_average: &[u8],
    ) -> Vec<u8>;
}

pub struct MosaicBuilder<T: Mosaicable> {
    pub tile_side_length: u32,
    pub tile_pixels: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub img_data: ImageData,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Mosaicable> MosaicBuilder<T> {
    pub fn new<U: Mosaicable, P: AsRef<Path>>(
        input_image_path: P,
        tile_side_length: u32,
    ) -> MosaicBuilder<U> {
        let image_data = ImageData::from_path(input_image_path, tile_side_length);
        return MosaicBuilder {
            tile_side_length,
            tile_pixels: tile_side_length * tile_side_length,
            tiles_x: image_data.width / tile_side_length as u32,
            tiles_y: image_data.height / tile_side_length as u32,
            img_data: image_data,
            _marker: std::marker::PhantomData,
        };
    }

    pub fn generate_mosaic(&self) -> Vec<u8> {
        let tile_sum = T::sum_tile_channels(self);
        let tile_average = T::calc_tile_average(self, &tile_sum);
        return T::create_mosaic(self, &tile_average.0);
    }

    pub fn benchmark(&self, benchmark_runs: u64) -> (Duration, Duration, Duration) {
        let mut sum_tile_channels_time = Duration::new(0, 0);
        let mut calc_tile_average_time = Duration::new(0, 0);
        let mut create_mosaic_time = Duration::new(0, 0);

        for _ in 0..benchmark_runs {
            let start = Instant::now();
            let tile_sum = T::sum_tile_channels(self);
            sum_tile_channels_time.add_assign(start.elapsed());

            let start = Instant::now();
            let tile_average = T::calc_tile_average(self, &tile_sum);
            calc_tile_average_time.add_assign(start.elapsed());

            let start = Instant::now();
            T::create_mosaic(self, &tile_average.0);
            create_mosaic_time.add_assign(start.elapsed());
        }

        return (
            sum_tile_channels_time.div(benchmark_runs as u32),
            calc_tile_average_time.div(benchmark_runs as u32),
            create_mosaic_time.div(benchmark_runs as u32),
        );
    }

    pub fn save_mosaic<P>(&self, output_img_path: P, img: &[u8]) -> ImageResult<()>
    where
        P: AsRef<Path>,
    {
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
                "Missing or incorrect extension on output image path: {}",
                output_img_path.as_ref().display()
            ),
        }
    }

    pub fn generate_and_save_mosaic<P>(&self, output_img_path: P) -> ImageResult<()>
    where
        P: AsRef<Path>,
    {
        let img = self.generate_mosaic();
        return self.save_mosaic(output_img_path, &img);
    }
}
