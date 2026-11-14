//! 3D Deep Space Starfield with realistic stellar classification and twinkling.

use crate::math::Vec3;

pub struct Starfield;

impl Starfield {
    /// Sample deep space background for a normalized ray direction.
    /// Returns (RGB, character for ASCII ramp).
    #[inline(always)]
    pub fn sample(dir: Vec3, tick: u64) -> ([u8; 3], char) {
        // High-frequency spherical hash
        let qx = (dir.x * 240.0).floor() as i64;
        let qy = (dir.y * 240.0).floor() as i64;
        let qz = (dir.z * 240.0).floor() as i64;

        let mut h = (qx.wrapping_mul(73856093))
            ^ (qy.wrapping_mul(19349663))
            ^ (qz.wrapping_mul(83492791));
        h ^= h >> 13;
        h = h.wrapping_mul(0x5bd1e995);
        h ^= h >> 15;

        let hash_val = (h & 0xFFFF) as f64 / 65535.0;

        // Density threshold: only ~1.8% of sky directions contain visible stars
        if hash_val > 0.982 {
            let star_norm = (hash_val - 0.982) / 0.018; // 0..1 brightness
            let phase = ((h & 0xFF) as f64) * 0.1;
            let twinkle = 0.85 + 0.15 * ((tick as f64 * 0.15 + phase).sin());

            let brightness = (star_norm.powf(1.8) * twinkle).clamp(0.0, 1.0);

            // Spectral class based on hash
            let spec_type = (h >> 16) & 0x07;
            let (base_r, base_g, base_b) = match spec_type {
                0 | 1 => (180.0, 205.0, 255.0), // Blue-white O/B type
                2 | 3 | 4 => (255.0, 255.0, 250.0), // White/Yellow A/G type
                5 => (255.0, 220.0, 150.0),     // Orange K type
                _ => (255.0, 160.0, 120.0),     // Red M dwarf
            };

            let r = (base_r * brightness) as u8;
            let g = (base_g * brightness) as u8;
            let b = (base_b * brightness) as u8;

            let ch = if brightness > 0.75 {
                '*'
            } else if brightness > 0.45 {
                '·'
            } else {
                '.'
            };

            ([r, g, b], ch)
        } else {
            ([0, 0, 0], ' ')
        }
    }
}
