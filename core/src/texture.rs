use image::{GenericImageView, ImageFormat, Pixel};

const TERRAIN_PNG: &[u8] = include_bytes!("../assets/terrain.png");

pub fn load_texture_from_terrain_png() -> (Vec<u8>, u32, u32) {
    let img = image::load_from_memory_with_format(TERRAIN_PNG, ImageFormat::Png)
        .expect("Failed to open image");

    let (width, height) = img.dimensions();

    let mut texels = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let channels = pixel.channels();
            texels.extend_from_slice(channels);
        }
    }
    (texels, width, height)
}
