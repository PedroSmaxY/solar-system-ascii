//! 3D Deep Space Starfield with Authentic Celestial Sphere, Real Stars, Constellations, and Nebulae.
//! Slices directly from `assets/celestial_sphere_data.bin` (2048x1024 RGB equirectangular sky map).

use crate::math::Vec3;
use std::f64::consts::PI;

pub const CELESTIAL_DATA: &[u8] = include_bytes!("../../assets/celestial_sphere_data.bin");

pub const SKY_WIDTH: usize = 2048;
pub const SKY_HEIGHT: usize = 1024;
const TWO_PI: f64 = 2.0 * PI;

pub struct Starfield;

impl Starfield {
    /// Sample the authentic celestial sphere background for a normalized ray direction.
    /// Returns (RGB, character for ASCII ramp).
    #[inline(always)]
    pub fn sample(dir: Vec3, tick: u64) -> ([u8; 3], char) {
        // Equatorial spherical coordinates matching generate_celestial_data.py
        let v = (0.5 - dir.y.clamp(-1.0, 1.0).asin() / PI).clamp(0.0001, 0.9999);
        let u = dir.z.atan2(dir.x).rem_euclid(TWO_PI) / TWO_PI;

        let fx = u.rem_euclid(1.0) * (SKY_WIDTH as f64 - 1.0);
        let fy = v * (SKY_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % SKY_WIDTH;
        let y1 = (y0 + 1).min(SKY_HEIGHT - 1);

        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;

        let data = &CELESTIAL_DATA[16..];

        let idx00 = (y0 * SKY_WIDTH + x0) * 3;
        let idx10 = (y0 * SKY_WIDTH + x1) * 3;
        let idx01 = (y1 * SKY_WIDTH + x0) * 3;
        let idx11 = (y1 * SKY_WIDTH + x1) * 3;

        let r00 = data[idx00] as f64;
        let g00 = data[idx00 + 1] as f64;
        let b00 = data[idx00 + 2] as f64;

        let r10 = data[idx10] as f64;
        let g10 = data[idx10 + 1] as f64;
        let b10 = data[idx10 + 2] as f64;

        let r01 = data[idx01] as f64;
        let g01 = data[idx01 + 1] as f64;
        let b01 = data[idx01 + 2] as f64;

        let r11 = data[idx11] as f64;
        let g11 = data[idx11 + 1] as f64;
        let b11 = data[idx11 + 2] as f64;

        let r_top = r00 * (1.0 - tx) + r10 * tx;
        let r_bot = r01 * (1.0 - tx) + r11 * tx;
        let mut r = r_top * (1.0 - ty) + r_bot * ty;

        let g_top = g00 * (1.0 - tx) + g10 * tx;
        let g_bot = g01 * (1.0 - tx) + g11 * tx;
        let mut g = g_top * (1.0 - ty) + g_bot * ty;

        let b_top = b00 * (1.0 - tx) + b10 * tx;
        let b_bot = b01 * (1.0 - tx) + b11 * tx;
        let mut b = b_top * (1.0 - ty) + b_bot * ty;

        let lum = 0.299 * r + 0.587 * g + 0.114 * b;

        // Subtle twinkling on prominent stars
        if lum > 140.0 {
            let phase = ((x0 ^ y0) & 0x1F) as f64 * 0.2;
            let twinkle = 0.88 + 0.12 * ((tick as f64 * 0.18 + phase).sin());
            r = (r * twinkle).min(255.0);
            g = (g * twinkle).min(255.0);
            b = (b * twinkle).min(255.0);
        }

        let r_byte = r.clamp(0.0, 255.0) as u8;
        let g_byte = g.clamp(0.0, 255.0) as u8;
        let b_byte = b.clamp(0.0, 255.0) as u8;

        let ch = if lum > 150.0 {
            '*'
        } else if lum > 80.0 {
            '·'
        } else if lum > 30.0 {
            '.'
        } else {
            ' '
        };

        ([r_byte, g_byte, b_byte], ch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celestial_data_loading() {
        assert_eq!(&CELESTIAL_DATA[0..8], b"ASTRO_SK");
        let width = u32::from_le_bytes([CELESTIAL_DATA[8], CELESTIAL_DATA[9], CELESTIAL_DATA[10], CELESTIAL_DATA[11]]) as usize;
        let height = u32::from_le_bytes([CELESTIAL_DATA[12], CELESTIAL_DATA[13], CELESTIAL_DATA[14], CELESTIAL_DATA[15]]) as usize;
        assert_eq!(width, SKY_WIDTH);
        assert_eq!(height, SKY_HEIGHT);
        assert_eq!(CELESTIAL_DATA.len(), 16 + SKY_WIDTH * SKY_HEIGHT * 3);
    }

    #[test]
    fn test_celestial_sampling_continuous() {
        // Sample towards Sirius (RA ~6.75h, Dec -16.7°)
        let ra_rad = (6.752 / 24.0) * TWO_PI;
        let dec_rad = -16.716 * PI / 180.0;
        let dir = Vec3::new(
            dec_rad.cos() * ra_rad.cos(),
            dec_rad.sin(),
            dec_rad.cos() * ra_rad.sin(),
        ).normalize();

        let (rgb, ch) = Starfield::sample(dir, 0);
        assert!(rgb[0] > 100 || rgb[1] > 100 || rgb[2] > 100, "Sirius must be brightly rendered in skybox");
        assert!(ch == '*' || ch == '·', "Sirius should have a star character");
    }
}
