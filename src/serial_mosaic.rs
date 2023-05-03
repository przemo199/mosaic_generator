use crate::mosaic_factory::MosaicBuilder;
use crate::MosaicFactory;

/// C-like mosaic implementation
#[derive(Clone, Copy, Debug)]
pub struct SerialMosaic;

impl MosaicBuilder for SerialMosaic {
    fn sum_tile_channels(&self, mosaic_factory: &MosaicFactory) -> Vec<u32> {
        let size = ((mosaic_factory.tiles_x * mosaic_factory.tiles_y)
            * mosaic_factory.image_data.channels as u32) as usize;
        let mut tile_sum: Vec<u32> = vec![0; size];

        for tile_y in 0..mosaic_factory.tiles_y {
            for tile_x in 0..mosaic_factory.tiles_x {
                let tile_sum_index = (tile_y * mosaic_factory.tiles_x + tile_x)
                    * mosaic_factory.image_data.channels as u32;
                let tile_index = (tile_y * mosaic_factory.tiles_x * mosaic_factory.tile_pixels
                    + tile_x * mosaic_factory.tile_side_length)
                    * mosaic_factory.image_data.channels as u32;
                for pixel_y in 0..mosaic_factory.tile_side_length {
                    for pixel_x in 0..mosaic_factory.tile_side_length {
                        let pixel_offset = (pixel_y * mosaic_factory.image_data.width
                            + pixel_x)
                            * mosaic_factory.image_data.channels as u32;
                        for channel in 0..mosaic_factory.image_data.channels {
                            let index = (tile_index + pixel_offset + channel as u32) as usize;
                            let pixel = mosaic_factory.image_data.data[index];
                            tile_sum[(tile_sum_index + channel as u32) as usize] += pixel as u32;
                        }
                    }
                }
            }
        }

        return tile_sum;
    }

    fn calc_tile_average(&self, mosaic_builder: &MosaicFactory, tile_sum: &[u32]) -> (Vec<u8>, Vec<u8>) {
        let mut global_sum: Vec<u128> = vec![0; mosaic_builder.image_data.channels as usize];
        let mut tile_average: Vec<u8> = vec![0; tile_sum.len()];

        let len = (mosaic_builder.tiles_y
            * mosaic_builder.tiles_x
            * mosaic_builder.image_data.channels as u32) as usize;
        for tile_channel in 0..len {
            let channel = (tile_channel % mosaic_builder.image_data.channels as usize) as u8;
            let tile_sum_channel = tile_sum[tile_channel];
            let tile_average_channel = (tile_sum_channel / mosaic_builder.tile_pixels) as u8;
            tile_average[tile_channel] = tile_average_channel;
            global_sum[channel as usize] += tile_average_channel as u128;
        }

        for channel in 0..mosaic_builder.image_data.channels {
            global_sum[channel as usize] /= (mosaic_builder.tiles_x * mosaic_builder.tiles_y) as u128;
        }
        let global_average: Vec<u8> = global_sum.into_iter().map(|x| x as u8).collect();
        return (tile_average, global_average);
    }

    fn create_mosaic(&self, mosaic_builder: &MosaicFactory, tile_average: &[u8]) -> Vec<u8> {
        let size = (mosaic_builder.image_data.width * mosaic_builder.image_data.height
            * mosaic_builder.image_data.channels as u32) as usize;
        let mut mosaic: Vec<u8> = vec![0; size];

        for tile_y in 0..mosaic_builder.tiles_y {
            for tile_x in 0..mosaic_builder.tiles_x {
                let tile_index = (tile_y * mosaic_builder.tiles_x + tile_x)
                    * mosaic_builder.image_data.channels as u32;
                let tile_offset = (tile_y * mosaic_builder.tiles_x * mosaic_builder.tile_pixels
                    + tile_x * mosaic_builder.tile_side_length)
                    * mosaic_builder.image_data.channels as u32;
                for pixel_y in 0..mosaic_builder.tile_side_length {
                    for pixel_x in 0..mosaic_builder.tile_side_length {
                        let pixel_offset = (pixel_y * mosaic_builder.image_data.width + pixel_x)
                            * mosaic_builder.image_data.channels as u32;
                        let pixel_location = (tile_offset + pixel_offset) as usize;
                        mosaic[pixel_location..pixel_location + mosaic_builder.image_data.channels as usize]
                            .copy_from_slice(&tile_average[tile_index as usize
                                ..(tile_index + (mosaic_builder.image_data.channels as u32)) as usize]);
                    }
                }
            }
        }

        return mosaic;
    }
}
