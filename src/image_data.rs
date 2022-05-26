use image::DynamicImage;

#[derive(Debug)]
struct ImageData {
    width: u32,
    height: u32,
    channels: u8,
    data: DynamicImage,
}

impl ImageData {
    fn new(width: u32, height: u32, channels: u8, data: DynamicImage) -> ImageData {
        ImageData {
            width,
            height,
            channels,
            data,
        }
    }
}
