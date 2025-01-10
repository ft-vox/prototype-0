use ft_vox_prototype_0_noise::{Noise, NoiseLayer};

use png::Encoder;
use std::{env, fs::File, io::BufWriter};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 10 || (args.len() - 8) % 2 != 0 {
        eprintln!("usage: noise_test <output_path> <image_size> <top_left.x> <top_left.y> <bottom_right.x> <bottom_right.y> <seed> <layer1.frequency> <layer1.amplitude> [<layer2.frequency> <layer2.amplitude> [...]]");
        return;
    }

    let output_path: &String = &args[1];
    let image_size: usize = args[2].parse().expect("Invalid image_size");
    let top_left_x: f32 = args[3].parse().expect("Invalid top_left.x");
    let top_left_y: f32 = args[4].parse().expect("Invalid top_left.y");
    let bottom_right_x: f32 = args[5].parse().expect("Invalid bottom_right.x");
    let bottom_right_y: f32 = args[6].parse().expect("Invalid bottom_right.y");
    let seed: u64 = args[7].parse().expect("Invalid seed");

    let mut layers = Vec::new();
    for i in (8..args.len()).step_by(2) {
        let frequency: f32 = args[i].parse().expect("Invalid layer frequency");
        let amplitude: f32 = args[i + 1].parse().expect("Invalid layer amplitude");
        layers.push(NoiseLayer::new(frequency, amplitude));
    }

    let noise = Noise::new(&layers, seed);
    let mut img_data = vec![0u8; image_size * image_size];

    for y in 0..image_size {
        for x in 0..image_size {
            let nx = top_left_x + (bottom_right_x - top_left_x) * (x as f32) / (image_size as f32);
            let ny = top_left_y + (bottom_right_y - top_left_y) * (y as f32) / (image_size as f32);
            let value = (noise.noise2(nx, ny) * 0.5 + 0.5) * 255.0;
            let clamped_value = value.clamp(0.0, 255.0) as u8;
            img_data[y * image_size + x] = clamped_value;
        }
    }

    let file = File::create(output_path).expect("Unable to create PNG file");
    let mut w = BufWriter::new(file);

    let mut encoder = Encoder::new(&mut w, image_size as u32, image_size as u32);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().expect("Unable to write PNG header");
    writer
        .write_image_data(&img_data)
        .expect("Unable to write PNG data");
}
