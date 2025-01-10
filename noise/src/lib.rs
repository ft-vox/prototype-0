use std::f32;

const PERMUTATION_SIZE: usize = 256;

#[derive(Clone, Copy)]
pub struct NoiseLayer {
    frequency: f32,
    amplitude: f32,
}

impl NoiseLayer {
    pub fn new(frequency: f32, amplitude: f32) -> NoiseLayer {
        NoiseLayer {
            frequency,
            amplitude,
        }
    }
}

#[derive(Clone)]
pub struct Noise {
    layers: Vec<NoiseLayer>,
    permutation: [u8; PERMUTATION_SIZE * 2],
}

impl Noise {
    pub fn new(layers: &[NoiseLayer], seed: u64) -> Noise {
        let mut permutation = [0u8; PERMUTATION_SIZE * 2];
        let mut p: [u8; PERMUTATION_SIZE] = [0; PERMUTATION_SIZE];

        for (i, v) in p.iter_mut().enumerate() {
            *v = i as u8
        }

        let mut seed = seed;
        for i in (0..PERMUTATION_SIZE).rev() {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (seed % (i as u64 + 1)) as usize;
            p.swap(i, j);
        }

        permutation[..PERMUTATION_SIZE].copy_from_slice(&p[..PERMUTATION_SIZE]);
        permutation[PERMUTATION_SIZE..(PERMUTATION_SIZE + PERMUTATION_SIZE)]
            .copy_from_slice(&p[..PERMUTATION_SIZE]);

        Noise {
            layers: Vec::from(layers),
            permutation,
        }
    }

    fn fade(t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(t: f32, a: f32, b: f32) -> f32 {
        a + t * (b - a)
    }

    fn grad2(hash: u8, x: f32, y: f32) -> f32 {
        let h = hash & 3;
        let u = if h & 2 == 0 { x } else { -x };
        let v = if h & 1 == 0 { y } else { -y };
        u + v
    }

    pub fn noise2(&self, x: f32, y: f32) -> f32 {
        let mut total = 0.0;

        for layer in &self.layers {
            let xf = x * layer.frequency;
            let yf = y * layer.frequency;

            let xi = (xf.floor() as i32) & 255;
            let yi = (yf.floor() as i32) & 255;

            let xf = xf - xf.floor();
            let yf = yf - yf.floor();

            let u = Noise::fade(xf);
            let v = Noise::fade(yf);

            let a = self.permutation[xi as usize] as usize + yi as usize;
            let aa = self.permutation[a] as usize;
            let ab = self.permutation[a + 1] as usize;
            let b = self.permutation[xi as usize + 1] as usize + yi as usize;
            let ba = self.permutation[b] as usize;
            let bb = self.permutation[b + 1] as usize;

            let grad_aa = Noise::grad2(self.permutation[aa], xf, yf);
            let grad_ba = Noise::grad2(self.permutation[ba], xf - 1.0, yf);
            let grad_ab = Noise::grad2(self.permutation[ab], xf, yf - 1.0);
            let grad_bb = Noise::grad2(self.permutation[bb], xf - 1.0, yf - 1.0);

            let lerp_x1 = Noise::lerp(u, grad_aa, grad_ba);
            let lerp_x2 = Noise::lerp(u, grad_ab, grad_bb);
            let lerp_y = Noise::lerp(v, lerp_x1, lerp_x2);

            total += lerp_y * layer.amplitude;
        }

        total
    }

    fn grad3(hash: u8, x: f32, y: f32, z: f32) -> f32 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 {
            y
        } else if h == 12 || h == 14 {
            x
        } else {
            z
        };
        let u_sign = if h & 1 == 0 { u } else { -u };
        let v_sign = if h & 2 == 0 { v } else { -v };
        u_sign + v_sign
    }

    pub fn noise3(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut total = 0.0;

        for layer in &self.layers {
            let xf = x * layer.frequency;
            let yf = y * layer.frequency;
            let zf = z * layer.frequency;

            let xi = (xf.floor() as i32) & 255;
            let yi = (yf.floor() as i32) & 255;
            let zi = (zf.floor() as i32) & 255;

            let xf = xf - xf.floor();
            let yf = yf - yf.floor();
            let zf = zf - zf.floor();

            let u = Noise::fade(xf);
            let v = Noise::fade(yf);
            let w = Noise::fade(zf);

            let a = self.permutation[xi as usize] as usize + yi as usize;
            let aa = self.permutation[a] as usize + zi as usize;
            let ab = self.permutation[a + 1] as usize + zi as usize;
            let b = self.permutation[xi as usize + 1] as usize + yi as usize;
            let ba = self.permutation[b] as usize + zi as usize;
            let bb = self.permutation[b + 1] as usize + zi as usize;

            let grad_aa = Noise::grad3(self.permutation[aa], xf, yf, zf);
            let grad_ba = Noise::grad3(self.permutation[ba], xf - 1.0, yf, zf);
            let grad_ab = Noise::grad3(self.permutation[ab], xf, yf - 1.0, zf);
            let grad_bb = Noise::grad3(self.permutation[bb], xf - 1.0, yf - 1.0, zf);

            let grad_aa1 = Noise::grad3(self.permutation[aa + 1], xf, yf, zf - 1.0);
            let grad_ba1 = Noise::grad3(self.permutation[ba + 1], xf - 1.0, yf, zf - 1.0);
            let grad_ab1 = Noise::grad3(self.permutation[ab + 1], xf, yf - 1.0, zf - 1.0);
            let grad_bb1 = Noise::grad3(self.permutation[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0);

            let lerp_x1 = Noise::lerp(u, grad_aa, grad_ba);
            let lerp_x2 = Noise::lerp(u, grad_ab, grad_bb);
            let lerp_y1 = Noise::lerp(v, lerp_x1, lerp_x2);

            let lerp_x3 = Noise::lerp(u, grad_aa1, grad_ba1);
            let lerp_x4 = Noise::lerp(u, grad_ab1, grad_bb1);
            let lerp_y2 = Noise::lerp(v, lerp_x3, lerp_x4);

            total += Noise::lerp(w, lerp_y1, lerp_y2) * layer.amplitude;
        }

        total
    }
}
