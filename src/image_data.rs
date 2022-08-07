use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView, ImageError};
use std::path::Path;

#[derive(Debug)]
pub struct ImageData {
    pub width: u32,
    pub height: u32,
    pub channels: u8,
    pub data: Vec<u8>,
    pub color: image::ColorType,
}

impl ImageData {
    pub fn new(img: &DynamicImage, tile_side_length: u32) -> ImageData {
        let cropped_img = crop_image(img, tile_side_length);
        return ImageData {
            width: cropped_img.width(),
            height: cropped_img.height(),
            channels: image_channels(&cropped_img),
            data: cropped_img.as_bytes().to_vec(),
            color: cropped_img.color(),
        };
    }

    pub fn from_path<P: AsRef<Path>>(path: P, tile_side_length: u32) -> ImageData {
        let img = load_image(path.as_ref());
        return ImageData::new(&img, tile_side_length);
    }

    fn get_pixel_channel(&self, x: u32, y: u32, channel: u8) -> Option<&u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = ((y * self.width + x) * self.channels as u32) as usize;
        Some(&self.data[index + channel as usize])
    }

    fn get_pixel_channel_mut(&mut self, x: u32, y: u32, channel: u8) -> Option<&mut u8> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = ((y * self.width + x) * self.channels as u32) as usize;
        Some(&mut self.data[index + channel as usize])
    }
}

pub fn crop_image(img: &DynamicImage, tile_side_length: u32) -> DynamicImage {
    let (original_width, original_height) = img.dimensions();
    let new_width = (original_width / tile_side_length) * tile_side_length;
    let new_height = (original_height / tile_side_length) * tile_side_length;
    let margin_x = (original_width - new_width) / 2;
    let margin_y = (original_height - new_height) / 2;
    return img.crop_imm(margin_x, margin_y, new_width, new_height);
}

pub fn image_channels(img: &DynamicImage) -> u8 {
    return img.color().channel_count();
}

pub fn load_image<P: AsRef<Path>>(image_path: P) -> DynamicImage {
    fn local_load_image(image_path: &Path) -> Result<DynamicImage, ImageError> {
        return ImageReader::open(image_path)?.decode();
    }

    let img = local_load_image(image_path.as_ref());
    match img {
        Ok(img) => {
            return img;
        }
        Err(e) => {
            panic!("Error loading image: {}", e);
        }
    }
}
