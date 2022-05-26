use crate::mosaic_builder::Mosaicable;
use crate::MosaicBuilder;
use std::sync::Mutex;

use rayon::prelude::*;

/// Parallel mosaic implementation
pub struct ParallelMosaicImpl;

impl Mosaicable for ParallelMosaicImpl {
    fn sum_tile_channels(&self, mosaic_builder: &MosaicBuilder<impl Mosaicable>) -> Vec<u32> {
        let tile_sum: Mutex<Vec<u32>> = Mutex::new(vec![
            0;
            ((mosaic_builder.tiles_x * mosaic_builder.tiles_y)
                * mosaic_builder.img_data.channels as u32)
                as usize
        ]);

        let tiles_x = mosaic_builder.tiles_x;
        let tiles_y = mosaic_builder.tiles_y;
        let channels = mosaic_builder.img_data.channels;
        let data = &mosaic_builder.img_data.data;
        let tile_pixels = mosaic_builder.tile_pixels;
        let width = mosaic_builder.img_data.width;
        let tile_side_length = mosaic_builder.tile_side_length;

        (0..tiles_y).into_par_iter().for_each(|tile_y| {
            for tile_x in 0..tiles_x {
                let tile_sum_index = (tile_y * tiles_x + tile_x) * channels as u32;
                let tile_index =
                    (tile_y * tiles_x * tile_pixels + tile_side_length as u32) * channels as u32;
                for pixel_y in 0..tile_side_length {
                    for pixel_x in 0..tile_side_length {
                        let pixel_offset =
                            (pixel_y as u32 * width + pixel_x as u32) * channels as u32;
                        for channel in 0..channels {
                            let pixel = data[(tile_index + pixel_offset + channel as u32) as usize];
                            let mut tile_sum_lock = tile_sum.lock().unwrap();
                            tile_sum_lock[(tile_sum_index + channel as u32) as usize] +=
                                pixel as u32;
                        }
                    }
                }
            }
        });

        return tile_sum.into_inner().unwrap();
    }

    fn calc_tile_average(
        &self,
        mosaic_builder: &MosaicBuilder<impl Mosaicable>,
        tile_sum: &[u32],
    ) -> (Vec<u8>, Vec<u8>) {
        let global_sum: Mutex<Vec<u128>> =
            Mutex::new(vec![0; mosaic_builder.img_data.channels as usize]);
        let tile_average: Mutex<Vec<u8>> = Mutex::new(vec![0; tile_sum.len()]);

        let tiles_x = mosaic_builder.tiles_x;
        let tiles_y = mosaic_builder.tiles_y;
        let channels = mosaic_builder.img_data.channels;
        let tile_pixels = mosaic_builder.tile_pixels;

        (0..tiles_y).into_par_iter().for_each(|tile_y| {
            for tile_x in 0..tiles_x {
                let tile_index = (tile_y * tiles_x + tile_x) * channels as u32;
                for channel in 0..channels {
                    let tile_sum_channel = tile_sum[(tile_index + channel as u32) as usize];
                    let tile_average_channel: u8 = (tile_sum_channel / tile_pixels) as u8;
                    let mut global_sum_lock = global_sum.lock().unwrap();
                    let mut tile_average_lock = tile_average.lock().unwrap();
                    tile_average_lock[(tile_index + channel as u32) as usize] =
                        tile_average_channel;
                    global_sum_lock[channel as usize] += tile_average_channel as u128;
                }
            }
        });

        let mut global_sum = global_sum.into_inner().unwrap();
        global_sum.iter_mut().for_each(|global_sum_channel| {
            *global_sum_channel /= (tiles_x * tiles_y) as u128;
        });

        let global_average: Vec<u8> = global_sum.into_iter().map(|x| x as u8).collect();
        return (tile_average.into_inner().unwrap(), global_average);
    }

    fn create_mosaic(
        &self,
        mosaic_builder: &MosaicBuilder<impl Mosaicable>,
        tile_average: &[u8],
    ) -> Vec<u8> {
        let mosaic: Mutex<Vec<u8>> = Mutex::new(vec![
            0;
            (mosaic_builder.img_data.width
                * mosaic_builder.img_data.height
                * mosaic_builder.img_data.channels as u32)
                as usize
        ]);

        let tiles_x = mosaic_builder.tiles_x;
        let tiles_y = mosaic_builder.tiles_y;
        let channels = mosaic_builder.img_data.channels;
        let tile_pixels = mosaic_builder.tile_pixels;
        let width = mosaic_builder.img_data.width;
        let tile_side_length = mosaic_builder.tile_side_length;

        (0..tiles_y).into_par_iter().for_each(|tile_y| {
            for tile_x in 0..tiles_x {
                let tile_index = (tile_y * tiles_x + tile_x) * channels as u32;
                let tile_offset = (tile_y * tiles_x * tile_pixels
                    + tile_x * tile_side_length as u32)
                    * channels as u32;
                for pixel_y in 0..tile_side_length {
                    for pixel_x in 0..tile_side_length {
                        let pixel_offset =
                            (pixel_y as u32 * width + pixel_x as u32) * channels as u32;
                        let pixel_location = (tile_offset + pixel_offset) as usize;
                        let mut mosaic_lock = mosaic.lock().unwrap();
                        mosaic_lock[pixel_location..pixel_location + channels as usize]
                            .copy_from_slice(
                                &tile_average[tile_index as usize
                                    ..(tile_index + (channels as u32)) as usize],
                            );
                    }
                }
            }
        });

        return mosaic.into_inner().unwrap();
    }
}
