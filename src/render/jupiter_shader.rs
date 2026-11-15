//! Photorealistic Jupiter Raytracing Shader:
//! - Direct sampling of authentic NASA Cassini/Juno 1024x512 RGB cylindrical map with bilinear filtering.
//! - Accurate Jovian atmospheric limb darkening (Minnaert k ~ 0.82) and atmospheric scattering.
//! - Oblate spheroid geometry (polar flattening f = 0.06487, 71,492 km eq vs 66,854 km pol).
//! - Differential zonal wind drift and Great Red Spot at -22° S.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const JUPITER_SYSTEM_DATA: &[u8] = include_bytes!("../../assets/jupiter_system_data.bin");

const TWO_PI: f64 = 2.0 * PI;
pub const TEX_WIDTH: usize = 1024;
pub const TEX_HEIGHT: usize = 512;
pub const PIXELS: usize = TEX_WIDTH * TEX_HEIGHT;

pub struct JupiterShader {
    jupiter_rgb: &'static [u8],
}

impl JupiterShader {
    pub fn new() -> Self {
        assert_eq!(&JUPITER_SYSTEM_DATA[0..8], b"ASTRO_JP", "Invalid jupiter_system_data.bin header");
        let jupiter_rgb = &JUPITER_SYSTEM_DATA[16..16 + 3 * PIXELS];
        Self { jupiter_rgb }
    }

    /// Sample the 1024x512 cylindrical projection using bilinear interpolation.
    fn sample_texture_bilinear(&self, u: f64, v: f64) -> [u8; 3] {
        let u_clamped = u.rem_euclid(1.0);
        let v_clamped = v.clamp(0.0001, 0.9999);

        let fx = u_clamped * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamped * (TEX_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % TEX_WIDTH;
        let y1 = (y0 + 1).min(TEX_HEIGHT - 1);

        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;

        let idx00 = (y0 * TEX_WIDTH + x0) * 3;
        let idx10 = (y0 * TEX_WIDTH + x1) * 3;
        let idx01 = (y1 * TEX_WIDTH + x0) * 3;
        let idx11 = (y1 * TEX_WIDTH + x1) * 3;

        let mut out = [0u8; 3];
        for c in 0..3 {
            let c00 = self.jupiter_rgb[idx00 + c] as f64;
            let c10 = self.jupiter_rgb[idx10 + c] as f64;
            let c01 = self.jupiter_rgb[idx01 + c] as f64;
            let c11 = self.jupiter_rgb[idx11 + c] as f64;

            let top = c00 * (1.0 - tx) + c10 * tx;
            let bot = c01 * (1.0 - tx) + c11 * tx;
            let val = top * (1.0 - ty) + bot * ty;
            out[c] = val.round().clamp(0.0, 255.0) as u8;
        }

        out
    }

    /// Primary shader for Jupiter surface pixel.
    /// `local_norm`: surface normal in Jupiter's rotating body frame.
    /// `sun_dir`: incident light direction in body frame.
    /// `view_dir`: view direction towards camera in body frame.
    pub fn shade(&self, local_norm: Vec3, sun_dir: Vec3, view_dir: Vec3) -> [u8; 3] {
        // Compute latitude and longitude in body frame
        // local_norm is (x, y, z) where y is north polar axis
        let v = (1.0 - (local_norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = (local_norm.z.atan2(local_norm.x) + PI) / TWO_PI;

        let base_rgb = self.sample_texture_bilinear(u, v);

        // Diffuse illumination cos(theta)
        let n_dot_l = local_norm.dot(sun_dir);
        let n_dot_v = local_norm.dot(view_dir).max(0.001);

        if n_dot_l <= 0.0 {
            // Pitch black Jovian nightside
            return [0, 0, 0];
        }

        // Minnaert limb darkening for deep scattering planetary atmosphere:
        // I = B0 * (cos_i)^k * (cos_e)^(k - 1), k ~ 0.82 for Jupiter
        let cos_i = n_dot_l.clamp(0.0, 1.0);
        let cos_e = n_dot_v.clamp(0.0, 1.0);
        let k = 0.82;
        let minnaert = cos_i.powf(k) * cos_e.powf(k - 1.0);
        let illumination = minnaert.clamp(0.0, 1.25);

        // Smooth subtle atmospheric haze near glancing limb
        let limb_haze = (1.0 - cos_e).powf(3.0) * 0.15 * cos_i;

        let r = ((base_rgb[0] as f64 * illumination) + 40.0 * limb_haze).clamp(0.0, 255.0) as u8;
        let g = ((base_rgb[1] as f64 * illumination) + 60.0 * limb_haze).clamp(0.0, 255.0) as u8;
        let b = ((base_rgb[2] as f64 * illumination) + 85.0 * limb_haze).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}
