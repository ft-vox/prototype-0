use image::{GenericImageView, ImageFormat, Pixel};

const TERRAIN_PNG: &[u8] = include_bytes!("../assets/terrain.png");

pub fn load_texture_from_terrain_png(
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let img = image::load_from_memory_with_format(TERRAIN_PNG, ImageFormat::Png)
        .expect("Failed to open image");

    let (img_width, img_height) = img.dimensions();
    if start_x + width > img_width || start_y + height > img_height {
        panic!("Image size is out of the bounds.");
    }

    let mut texels = Vec::with_capacity((width * height * 4) as usize);
    for y in start_y..(start_y + height) {
        for x in start_x..(start_x + width) {
            let pixel = img.get_pixel(x, y);
            let channels = pixel.channels();
            texels.extend_from_slice(channels);
        }
    }
    texels
}
