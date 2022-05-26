use crate::mosaic_builder::Mosaicable;
use crate::MosaicBuilder;
use crate::SerialMosaicImpl;

use rayon::prelude::*;

/// Parallel mosaic implementation using rayon.
pub struct Parallel2MosaicImpl;

impl Mosaicable for Parallel2MosaicImpl {
    fn sum_tile_channels(&self, mosaic_builder: &MosaicBuilder<impl Mosaicable>) -> Vec<u32> {
        let mut tile_sum: Vec<u32> = vec![
            0;
            ((mosaic_builder.tiles_x * mosaic_builder.tiles_y)
                * mosaic_builder.img_data.channels as u32)
                as usize
        ];

        let tiles_x = mosaic_builder.tiles_x;
        let channels = mosaic_builder.img_data.channels;
        let width = mosaic_builder.img_data.width;
        let data = &mosaic_builder.img_data.data;
        let tile_pixels = mosaic_builder.tile_pixels;
        let tile_side_length = mosaic_builder.tile_side_length;
        tile_sum
            .par_iter_mut()
            .enumerate()
            .for_each(|(index, tile_sum_channel)| {
                let tile = (index / channels as usize) as u32;
                let channel = (index % channels as usize) as u8;

                let tile_x = tile % tiles_x;
                let tile_y = tile / tiles_x;

                let tile_pixel_start =
                    (tile_y * tile_pixels * tiles_x + tile_x * tile_side_length) * channels as u32;

                let mut sum: u32 = 0;
                for pixel_y in 0..tile_side_length {
                    for pixel_x in 0..tile_side_length {
                        let pixel_index = (pixel_y * width + pixel_x) * channels as u32;
                        sum += data
                            [tile_pixel_start as usize + pixel_index as usize + channel as usize]
                            as u32;
                    }
                }

                *tile_sum_channel = sum;
            });

        return tile_sum;
    }

    fn calc_tile_average(
        &self,
        mosaic_builder: &MosaicBuilder<impl Mosaicable>,
        tile_sum: &[u32],
    ) -> (Vec<u8>, Vec<u8>) {
        return SerialMosaicImpl.calc_tile_average(mosaic_builder, tile_sum);
    }

    fn create_mosaic(
        &self,
        mosaic_builder: &MosaicBuilder<impl Mosaicable>,
        tile_average: &[u8],
    ) -> Vec<u8> {
        let mut mosaic: Vec<u8> = vec![
            0;
            (mosaic_builder.img_data.width
                * mosaic_builder.img_data.height
                * mosaic_builder.img_data.channels as u32)
                as usize
        ];

        let tiles_x = mosaic_builder.tiles_x;
        let channels = mosaic_builder.img_data.channels;
        let tile_side_length = mosaic_builder.tile_side_length;
        mosaic
            .par_chunks_mut(
                mosaic_builder.tile_side_length as usize
                    * mosaic_builder.img_data.channels as usize,
            )
            .enumerate()
            .for_each(|(index, chunk)| {
                let tile_x = index % tiles_x as usize;
                let tile_y = index / (tiles_x * tile_side_length) as usize;
                let tile = (tile_y * tiles_x as usize + tile_x) * channels as usize;
                let pixel_slice = &tile_average[tile..tile + channels as usize];
                chunk.chunks_mut(channels as usize).for_each(|pixel_chunk| {
                    pixel_chunk.copy_from_slice(pixel_slice);
                });
            });

        return mosaic;
    }
}
