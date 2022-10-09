use crate::mosaic_factory::MosaicBuilder;
use crate::MosaicFactory;

use rayon::prelude::*;

/// Parallel mosaic implementation using rayon
pub struct ParallelMosaicImpl;

impl MosaicBuilder for ParallelMosaicImpl {
    fn sum_tile_channels(&self, mosaic_builder: &MosaicFactory) -> Vec<u32> {
        let mut tile_sum: Vec<u32> = vec![
            0;
            ((mosaic_builder.tiles_x * mosaic_builder.tiles_y)
                * mosaic_builder.img_data.channels as u32)
                as usize
        ];

        tile_sum
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, tile_sum_channel)| {
                let tile = (index / mosaic_builder.img_data.channels as usize) as u32;
                let channel = (index % mosaic_builder.img_data.channels as usize) as u8;

                let tile_x = tile % mosaic_builder.tiles_x;
                let tile_y = tile / mosaic_builder.tiles_x;

                let tile_pixel_start =
                    ((tile_y * mosaic_builder.tile_pixels * mosaic_builder.tiles_x
                        + tile_x * mosaic_builder.tile_side_length)
                        * mosaic_builder.img_data.channels as u32) as usize;

                let mut sum: u32 = 0;
                for pixel_y in 0..mosaic_builder.tile_side_length {
                    for pixel_x in 0..mosaic_builder.tile_side_length {
                        let pixel_index = (pixel_y * mosaic_builder.img_data.width + pixel_x)
                            * mosaic_builder.img_data.channels as u32;
                        sum += mosaic_builder.img_data.data
                            [tile_pixel_start + pixel_index as usize + channel as usize]
                            as u32;
                    }
                }

                *tile_sum_channel = sum;
            });

        return tile_sum;
    }

    fn calc_tile_average(
        &self,
        mosaic_builder: &MosaicFactory,
        tile_sum: &[u32],
    ) -> (Vec<u8>, Vec<u8>) {
        let mut global_sum: Vec<u128> = vec![0; mosaic_builder.img_data.channels as usize];
        let mut tile_average: Vec<u8> = vec![0; tile_sum.len()];

        let calc_tile_avg = || {
            tile_average
                .par_iter_mut()
                .enumerate()
                .for_each(|(index, tile_channel)| {
                    *tile_channel = (tile_sum[index] / mosaic_builder.tile_pixels) as u8;
                });
        };

        let calc_global_avg = || {
            global_sum
                .par_iter_mut()
                .enumerate()
                .for_each(|(index, global_sum_channel)| {
                    for tile in 0..((mosaic_builder.tiles_y * mosaic_builder.tiles_x) as usize) {
                        let channel = index;
                        let tile_sum_channel =
                            tile_sum[tile * mosaic_builder.img_data.channels as usize + channel];
                        let tile_average_channel =
                            (tile_sum_channel / mosaic_builder.tile_pixels) as u8;
                        *global_sum_channel += tile_average_channel as u128;
                    }
                });
        };

        rayon::join(calc_tile_avg, calc_global_avg);

        global_sum.iter_mut().for_each(|channel| {
            *channel /= (mosaic_builder.tiles_x * mosaic_builder.tiles_y) as u128;
        });
        let global_average: Vec<u8> = global_sum.into_iter().map(|x| x as u8).collect();
        return (tile_average, global_average);
    }

    fn create_mosaic(&self, mosaic_builder: &MosaicFactory, tile_average: &[u8]) -> Vec<u8> {
        let mut mosaic: Vec<u8> = vec![
            0;
            (mosaic_builder.img_data.width
                * mosaic_builder.img_data.height
                * mosaic_builder.img_data.channels as u32)
                as usize
        ];

        mosaic
            .par_chunks_mut(
                mosaic_builder.tile_side_length as usize
                    * mosaic_builder.img_data.channels as usize,
            )
            .enumerate()
            .for_each(|(index, chunk)| {
                let tile_x = index % mosaic_builder.tiles_x as usize;
                let tile_y =
                    index / (mosaic_builder.tiles_x * mosaic_builder.tile_side_length) as usize;
                let tile = (tile_y * mosaic_builder.tiles_x as usize + tile_x)
                    * mosaic_builder.img_data.channels as usize;
                let pixel_slice =
                    &tile_average[tile..tile + mosaic_builder.img_data.channels as usize];
                chunk
                    .chunks_mut(mosaic_builder.img_data.channels as usize)
                    .for_each(|pixel_chunk| {
                        pixel_chunk.copy_from_slice(pixel_slice);
                    });
            });

        return mosaic;
    }
}
