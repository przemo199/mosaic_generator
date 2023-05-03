use std::sync::Mutex;

use rayon::prelude::*;

use crate::mosaic_factory::MosaicBuilder;
use crate::MosaicFactory;

/// Slow parallel mosaic implementation
#[derive(Clone, Copy, Debug)]
pub struct SlowParallelMosaic;

impl MosaicBuilder for SlowParallelMosaic {
    fn sum_tile_channels(&self, mosaic_builder: &MosaicFactory) -> Vec<u32> {
        let size = ((mosaic_builder.tiles_x * mosaic_builder.tiles_y)
            * mosaic_builder.image_data.channels as u32) as usize;
        let tile_sum: Mutex<Vec<u32>> = Mutex::new(vec![0; size]);

        (0..mosaic_builder.tiles_y).into_par_iter().for_each(|tile_y| {
            for tile_x in 0..mosaic_builder.tiles_x {
                let tile_sum_index = (tile_y * mosaic_builder.tiles_x + tile_x)
                    * mosaic_builder.image_data.channels as u32;
                let tile_index = (tile_y * mosaic_builder.tiles_x * mosaic_builder.tile_pixels
                    + tile_x * mosaic_builder.tile_side_length)
                    * mosaic_builder.image_data.channels as u32;
                for pixel_y in 0..mosaic_builder.tile_side_length {
                    for pixel_x in 0..mosaic_builder.tile_side_length {
                        let pixel_offset = (pixel_y * mosaic_builder.image_data.width + pixel_x)
                            * mosaic_builder.image_data.channels as u32;
                        for channel in 0..mosaic_builder.image_data.channels {
                            let pixel = mosaic_builder.image_data.data
                                [(tile_index + pixel_offset + channel as u32) as usize];
                            let mut tile_sum_lock = tile_sum.lock().unwrap();
                            tile_sum_lock[(tile_sum_index + channel as u32) as usize] += pixel as u32;
                        }
                    }
                }
            }
        });

        return tile_sum.into_inner().unwrap();
    }

    fn calc_tile_average(
        &self,
        mosaic_builder: &MosaicFactory,
        tile_sum: &[u32],
    ) -> (Vec<u8>, Vec<u8>) {
        let global_sum: Mutex<Vec<u128>> =
            Mutex::new(vec![0; mosaic_builder.image_data.channels as usize]);
        let tile_average: Mutex<Vec<u8>> = Mutex::new(vec![0; tile_sum.len()]);

        (0..mosaic_builder.tiles_y).into_par_iter().for_each(|tile_y| {
            for tile_x in 0..mosaic_builder.tiles_x {
                let tile_index = (tile_y * mosaic_builder.tiles_x + tile_x)
                    * mosaic_builder.image_data.channels as u32;
                for channel in 0..mosaic_builder.image_data.channels {
                    let tile_sum_channel = tile_sum[(tile_index + channel as u32) as usize];
                    let tile_average_channel = (tile_sum_channel / mosaic_builder.tile_pixels) as u8;
                    let update_tile_average = || {
                        let mut tile_average_lock = tile_average.lock().unwrap();
                        tile_average_lock[(tile_index + channel as u32) as usize] = tile_average_channel;
                    };
                    let update_global_sum = || {
                        let mut global_sum_lock = global_sum.lock().unwrap();
                        global_sum_lock[channel as usize] += tile_average_channel as u128;
                    };
                    rayon::join(update_tile_average, update_global_sum);
                }
            }
        });

        let mut global_sum = global_sum.into_inner().unwrap();
        global_sum.iter_mut().for_each(|channel| {
            *channel /= (mosaic_builder.tiles_x * mosaic_builder.tiles_y) as u128;
        });

        let global_average: Vec<u8> = global_sum.into_iter().map(|x| x as u8).collect();
        return (tile_average.into_inner().unwrap(), global_average);
    }

    fn create_mosaic(&self, mosaic_builder: &MosaicFactory, tile_average: &[u8]) -> Vec<u8> {
        let size = (mosaic_builder.image_data.width * mosaic_builder.image_data.height
            * mosaic_builder.image_data.channels as u32) as usize;
        let mosaic: Mutex<Vec<u8>> = Mutex::new(vec![0; size]);

        (0..mosaic_builder.tiles_y).into_par_iter().for_each(|tile_y| {
            for tile_x in 0..mosaic_builder.tiles_x {
                let tile_index = (tile_y * mosaic_builder.tiles_x + tile_x)
                    * mosaic_builder.image_data.channels as u32;
                let tile_offset =
                    (tile_y * mosaic_builder.tiles_x * mosaic_builder.tile_pixels
                        + tile_x * mosaic_builder.tile_side_length)
                        * mosaic_builder.image_data.channels as u32;
                for pixel_y in 0..mosaic_builder.tile_side_length {
                    for pixel_x in 0..mosaic_builder.tile_side_length {
                        let pixel_offset = (pixel_y * mosaic_builder.image_data.width + pixel_x)
                            * mosaic_builder.image_data.channels as u32;
                        let pixel_location = (tile_offset + pixel_offset) as usize;
                        let mut mosaic_lock = mosaic.lock().unwrap();
                        mosaic_lock[pixel_location..pixel_location + mosaic_builder.image_data.channels as usize]
                            .copy_from_slice(&tile_average[tile_index as usize
                                ..(tile_index + (mosaic_builder.image_data.channels as u32)) as usize]);
                    }
                }
            }
        });

        return mosaic.into_inner().unwrap();
    }
}
