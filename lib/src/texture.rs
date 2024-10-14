use image::{GenericImageView, Pixel};

pub fn create_texels(size: usize) -> Vec<u8> {
    (0..size * size)
        .map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            count
        })
        .collect()
}

pub fn load_texture_from_png(
    file_path: &str,
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
) -> Vec<u8> {
    let img = image::open(file_path).expect("Failed to open image");

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
