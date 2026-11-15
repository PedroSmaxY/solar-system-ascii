//! Photorealistic Titan Raytracing Shader:
//! - Direct zero-copy slice of authentic NASA Cassini 1024x512 RGB cylindrical map
//!   from `saturn_system_data.bin`.
//! - Fractional 4-sample bilinear interpolation without polar singularities.
//! - Titan Atmospheric Physics (1.45 bar dense N2/CH4 atmosphere):
//!   * Minnaert atmospheric limb darkening with k = 0.90.
//!   * Detached upper photochemical haze layer (Cassini high-altitude tholin aerosols ~300-500 km)
//!     with smooth volumetric forward/limb scattering.
//!   * Strictly pitch-black nightside when n_dot_l <= 0.0 (zero light leakage).
//! - Calibrated amber/orange photochemical tholin cloud deck and haze palette.

use crate::math::Vec3;
use crate::render::saturn_shader::{SATURN_SYSTEM_DATA, PIXELS, TEX_HEIGHT, TEX_WIDTH};
use std::f64::consts::PI;

pub const TITAN_RGB_OFFSET: usize = 16 + 3 * PIXELS + 4096; // 1_576_976
pub const TITAN_RGB_LEN: usize = 3 * PIXELS; // 1_572_864

/// Titan's surface atmospheric pressure in bar
pub const SURFACE_PRESSURE_BAR: f64 = 1.45;
/// Titan's atmospheric Minnaert limb darkening exponent (k = 0.90)
pub const MINNAERT_K: f64 = 0.90;
/// Characteristic amber / orange haze color for detached upper haze layer
pub const DETACHED_HAZE_COLOR: [f64; 3] = [225.0, 165.0, 65.0];

#[derive(Clone, Copy, Debug)]
pub struct TitanShader {
    titan_rgb: &'static [u8],
}

impl Default for TitanShader {
    fn default() -> Self {
        Self::new()
    }
}

impl TitanShader {
    /// Initialize TitanShader with zero-copy slice from `saturn_system_data.bin`.
    pub fn new() -> Self {
        assert_eq!(
            &SATURN_SYSTEM_DATA[0..8],
            b"ASTRO_SA",
            "Invalid saturn_system_data.bin header magic"
        );
        let width = u32::from_le_bytes([
            SATURN_SYSTEM_DATA[8],
            SATURN_SYSTEM_DATA[9],
            SATURN_SYSTEM_DATA[10],
            SATURN_SYSTEM_DATA[11],
        ]) as usize;
        let height = u32::from_le_bytes([
            SATURN_SYSTEM_DATA[12],
            SATURN_SYSTEM_DATA[13],
            SATURN_SYSTEM_DATA[14],
            SATURN_SYSTEM_DATA[15],
        ]) as usize;
        assert_eq!(width, TEX_WIDTH, "Unexpected texture width");
        assert_eq!(height, TEX_HEIGHT, "Unexpected texture height");
        assert!(
            SATURN_SYSTEM_DATA.len() >= TITAN_RGB_OFFSET + TITAN_RGB_LEN,
            "saturn_system_data.bin buffer too small for Titan dataset"
        );

        let titan_rgb = &SATURN_SYSTEM_DATA[TITAN_RGB_OFFSET..TITAN_RGB_OFFSET + TITAN_RGB_LEN];
        Self { titan_rgb }
    }

    /// Access the underlying zero-copy slice of Titan RGB data.
    #[inline(always)]
    pub fn titan_rgb(&self) -> &'static [u8] {
        self.titan_rgb
    }

    /// Calculate spherical UV coordinates from local surface normal (x, y, z) where y is polar axis:
    /// - v = 0.0 at North Pole, 1.0 at South Pole
    /// - u in [0.0, 1.0) corresponding to longitude wrapped seamlessly
    #[inline(always)]
    pub fn spherical_coords(norm: Vec3) -> (f64, f64) {
        let n = norm.normalize();
        let v = (1.0 - (n.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = ((n.z.atan2(n.x) + PI) / (2.0 * PI)).rem_euclid(1.0);
        (u, v)
    }

    /// Sample the 1024x512 Titan RGB texture using fractional 4-sample bilinear interpolation.
    #[inline]
    pub fn sample_bilinear(&self, u: f64, v: f64) -> [u8; 3] {
        Self::sample_bilinear_rgb(self.titan_rgb, u, v)
    }

    /// 4-texel fractional bilinear interpolation on 1024x512 RGB cylindrical map with seamless wrapping on u.
    #[inline]
    pub fn sample_bilinear_rgb(data: &[u8], u: f64, v: f64) -> [u8; 3] {
        let u_clamped = u.rem_euclid(1.0);
        let v_clamped = v.clamp(0.0, 1.0);

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
            let c00 = data[idx00 + c] as f64;
            let c10 = data[idx10 + c] as f64;
            let c01 = data[idx01 + c] as f64;
            let c11 = data[idx11 + c] as f64;

            let top = c00 * (1.0 - tx) + c10 * tx;
            let bot = c01 * (1.0 - tx) + c11 * tx;
            let val = top * (1.0 - ty) + bot * ty;
            out[c] = val.round().clamp(0.0, 255.0) as u8;
        }

        out
    }

    /// Shade a surface/atmospheric point on Titan.
    ///
    /// Evaluates:
    /// - Strict nightside cut-off (`n_dot_l <= 0.0`) resulting in pitch-black `[0, 0, 0]`
    /// - Minnaert atmospheric limb darkening with k = 0.90 for thick scattering atmosphere
    /// - Volumetric detached upper haze forward scattering at glancing viewing angles
    pub fn shade(&self, local_norm: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        let norm = local_norm.normalize();
        let sun = local_sun.normalize();
        let view = local_view.normalize();

        let n_dot_l = norm.dot(sun);
        if n_dot_l <= 0.0 {
            // Strictly pitch-black nightside: zero light leakage
            return [0, 0, 0];
        }

        let n_dot_v = norm.dot(view).max(1e-4);

        let (u, v) = Self::spherical_coords(norm);
        let base_rgb = self.sample_bilinear(u, v);

        // Minnaert atmospheric limb darkening: B = B0 * (cos_i)^k * (cos_e)^(k - 1)
        let cos_i = n_dot_l.clamp(0.0, 1.0);
        let cos_e = n_dot_v.clamp(0.0, 1.0);
        let minnaert = (cos_i.powf(MINNAERT_K) * cos_e.powf(MINNAERT_K - 1.0)).clamp(0.0, 1.35);

        // Detached upper haze layer (Cassini high-altitude photochemical tholin haze ~300-500 km)
        // Strongest at glancing limb angles where optical path through the upper haze is longest
        let limb_angle = (1.0 - cos_e).clamp(0.0, 1.0);
        let haze_scattering = limb_angle.powf(2.5) * cos_i;
        let detached_haze_glow = haze_scattering * 0.28;

        let r = ((base_rgb[0] as f64 * minnaert) + DETACHED_HAZE_COLOR[0] * detached_haze_glow)
            .round()
            .clamp(0.0, 255.0) as u8;
        let g = ((base_rgb[1] as f64 * minnaert) + DETACHED_HAZE_COLOR[1] * detached_haze_glow)
            .round()
            .clamp(0.0, 255.0) as u8;
        let b = ((base_rgb[2] as f64 * minnaert) + DETACHED_HAZE_COLOR[2] * detached_haze_glow)
            .round()
            .clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titan_slice_boundaries() {
        assert_eq!(&SATURN_SYSTEM_DATA[0..8], b"ASTRO_SA");
        assert_eq!(SATURN_SYSTEM_DATA.len(), 7_344_144);
        assert_eq!(TITAN_RGB_OFFSET, 16 + 3 * PIXELS + 4096);
        assert_eq!(TITAN_RGB_OFFSET, 1_576_976);
        assert_eq!(TITAN_RGB_LEN, 3 * PIXELS);
        assert_eq!(TITAN_RGB_LEN, 1_572_864);
        assert_eq!(TITAN_RGB_OFFSET + TITAN_RGB_LEN, 3_149_840);

        let shader = TitanShader::new();
        assert_eq!(shader.titan_rgb().len(), TITAN_RGB_LEN);
        assert_eq!(
            &shader.titan_rgb()[..16],
            &SATURN_SYSTEM_DATA[TITAN_RGB_OFFSET..TITAN_RGB_OFFSET + 16]
        );
        assert_eq!(
            &shader.titan_rgb()[TITAN_RGB_LEN - 16..],
            &SATURN_SYSTEM_DATA[TITAN_RGB_OFFSET + TITAN_RGB_LEN - 16..TITAN_RGB_OFFSET + TITAN_RGB_LEN]
        );
    }

    #[test]
    fn test_sample_bilinear() {
        // 1. Synthetic 2x2 RGB test pattern
        let mut mock_data = vec![0u8; TEX_WIDTH * TEX_HEIGHT * 3];
        let p00_idx = 0;
        let p10_idx = 3;
        let p01_idx = TEX_WIDTH * 3;
        let p11_idx = (TEX_WIDTH + 1) * 3;

        mock_data[p00_idx..p00_idx + 3].copy_from_slice(&[10, 20, 30]);
        mock_data[p10_idx..p10_idx + 3].copy_from_slice(&[20, 30, 40]);
        mock_data[p01_idx..p01_idx + 3].copy_from_slice(&[30, 40, 50]);
        mock_data[p11_idx..p11_idx + 3].copy_from_slice(&[40, 50, 60]);

        let u_mid = 0.5 / (TEX_WIDTH as f64 - 1.0);
        let v_mid = 0.5 / (TEX_HEIGHT as f64 - 1.0);
        let sampled = TitanShader::sample_bilinear_rgb(&mock_data, u_mid, v_mid);
        assert_eq!(sampled, [25, 35, 45], "Bilinear midpoint should average the 4 samples");

        // 2. Longitude wrapping: u = 1.0 wraps seamlessly to u = 0.0
        let sampled_wrap0 = TitanShader::sample_bilinear_rgb(&mock_data, 0.0, 0.0);
        let sampled_wrap1 = TitanShader::sample_bilinear_rgb(&mock_data, 1.0, 0.0);
        assert_eq!(sampled_wrap0, sampled_wrap1);

        // 3. Real Titan dataset sampling
        let shader = TitanShader::new();
        let sample_at_0 = shader.sample_bilinear(0.0, 0.5);
        let sample_at_1 = shader.sample_bilinear(1.0, 0.5);
        assert_eq!(sample_at_0, sample_at_1, "u = 0.0 and u = 1.0 must sample identical meridian");

        // Latitude clamping at poles
        let sample_north = shader.sample_bilinear(0.3, 0.0);
        let sample_north_clamped = shader.sample_bilinear(0.3, -0.5);
        assert_eq!(sample_north, sample_north_clamped);

        let sample_south = shader.sample_bilinear(0.7, 1.0);
        let sample_south_clamped = shader.sample_bilinear(0.7, 1.5);
        assert_eq!(sample_south, sample_south_clamped);
    }

    #[test]
    fn test_atmospheric_amber_color_palette() {
        let shader = TitanShader::new();

        // 1. Physical atmospheric constants
        assert!((SURFACE_PRESSURE_BAR - 1.45).abs() < 1e-6, "Titan atmospheric pressure is 1.45 bar");
        assert!((MINNAERT_K - 0.90).abs() < 1e-6, "Minnaert limb darkening coefficient k = 0.90");

        // 2. Detached upper haze color shows strong amber/orange tholin dominance
        assert!(DETACHED_HAZE_COLOR[0] > DETACHED_HAZE_COLOR[1]);
        assert!(DETACHED_HAZE_COLOR[1] > DETACHED_HAZE_COLOR[2]);
        assert!(DETACHED_HAZE_COLOR[0] >= 200.0);

        // 3. Raw texture data across globe exhibits authentic amber/orange spectrum (R > G > B)
        let test_coords = [
            (0.1, 0.2), (0.25, 0.5), (0.5, 0.5), (0.75, 0.3), (0.9, 0.8), (0.33, 0.67)
        ];
        for (u, v) in test_coords {
            let rgb = shader.sample_bilinear(u, v);
            assert!(
                rgb[0] > rgb[1] && rgb[1] > rgb[2],
                "Titan photochemical haze must show R > G > B amber spectrum at ({}, {}): {:?}",
                u, v, rgb
            );
            assert!(
                rgb[0] >= 150,
                "Titan amber haze must have strong red reflectance: {:?}",
                rgb
            );
            assert!(
                rgb[2] <= 140,
                "Titan blue channel must be absorbed by atmospheric methane/tholins: {:?}",
                rgb
            );
        }

        // 4. Shaded dayside point preserves authentic warm amber coloration
        let subsolar_norm = Vec3::new(1.0, 0.0, 0.0);
        let sun = Vec3::new(1.0, 0.0, 0.0);
        let view = Vec3::new(1.0, 0.0, 0.0);
        let shaded = shader.shade(subsolar_norm, sun, view);
        assert!(shaded[0] > shaded[1] && shaded[1] > shaded[2], "Shaded daylight must be amber");
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = TitanShader::new();
        let norm = Vec3::new(1.0, 0.0, 0.0);

        // 1. Direct subsolar illumination (Dayside)
        let sun_day = Vec3::new(1.0, 0.0, 0.0);
        let view_day = Vec3::new(1.0, 0.0, 0.0);
        let color_day = shader.shade(norm, sun_day, view_day);
        assert!(
            color_day[0] >= 150 && color_day[1] >= 130 && color_day[2] >= 90,
            "Day side should be brightly illuminated with amber haze: got {:?}",
            color_day
        );

        // 2. Pitch black nightside: zero light leakage when sun is on opposite side (n_dot_l <= 0.0)
        let sun_night = Vec3::new(-1.0, 0.0, 0.0);
        let color_night = shader.shade(norm, sun_night, view_day);
        assert_eq!(
            color_night, [0, 0, 0],
            "Night side must be strictly pitch black [0, 0, 0]"
        );

        // 3. Grazing nightside / past terminator
        let sun_grazing_night = Vec3::new(-0.01, 0.0, 1.0);
        let color_grazing = shader.shade(norm, sun_grazing_night, view_day);
        assert_eq!(
            color_grazing, [0, 0, 0],
            "Grazing nightside past terminator must be strictly pitch black"
        );

        // 4. Perpendicular terminator boundary (n_dot_l == 0.0)
        let sun_perp = Vec3::new(0.0, 1.0, 0.0);
        let color_perp = shader.shade(norm, sun_perp, view_day);
        assert_eq!(
            color_perp, [0, 0, 0],
            "Terminator boundary n_dot_l = 0.0 must be pitch black"
        );

        // 5. Minnaert limb darkening and detached upper haze near the limb
        let angle_rad = 80.0_f64.to_radians();
        let view_limb = Vec3::new(angle_rad.cos(), angle_rad.sin(), 0.0).normalize();
        let color_limb = shader.shade(norm, sun_day, view_limb);
        assert!(
            color_limb[0] > 0 && color_limb[1] > 0,
            "Limb under daylight must be illuminated with detached haze: got {:?}",
            color_limb
        );
        // Amber hue is preserved at limb
        assert!(color_limb[0] > color_limb[1] && color_limb[1] > color_limb[2]);
    }
}
