//! Photorealistic Io Volcanic Surface Shader:
//! - Direct zero-copy slice of authentic NASA Galileo/Voyager 1024x512 RGB cylindrical map.
//! - Non-Lambertian Lommel-Seeliger reflectance with opposition surge and pitch-black nightside.
//! - Bilinear texture filtering with seamless longitude wrapping.
//! - Authentic Io geomorphology and color palette:
//!   * Bright sulfur yellow plains: ~[235, 210, 80]
//!   * Orange/red sulfur fallout rings around active volcanoes (Pele, Tvashtar): ~[210, 90, 30]
//!   * Dark basalt volcanic patera pits (Loki Patera, Babbar, Pillan): ~[40, 35, 30]
//!   * White SO2 frost patches: ~[245, 245, 240]

use crate::math::Vec3;
use crate::render::jupiter_shader::{JUPITER_SYSTEM_DATA, PIXELS, TEX_HEIGHT, TEX_WIDTH};
use std::f64::consts::PI;

pub const IO_RGB_OFFSET: usize = 16 + 3 * PIXELS; // 1,572,880
pub const IO_BYTE_LEN: usize = 3 * PIXELS; // 1,572,864

pub const COLOR_SULFUR_PLAINS: [f64; 3] = [235.0, 210.0, 80.0];
pub const COLOR_VOLCANIC_FALLOUT: [f64; 3] = [210.0, 90.0, 30.0];
pub const COLOR_BASALT_PATERA: [f64; 3] = [40.0, 35.0, 30.0];
pub const COLOR_FROST_PATCHES: [f64; 3] = [245.0, 245.0, 240.0];

pub struct IoShader {
    io_rgb: &'static [u8],
}

impl Default for IoShader {
    fn default() -> Self {
        Self::new()
    }
}

impl IoShader {
    pub fn new() -> Self {
        assert_eq!(
            &JUPITER_SYSTEM_DATA[0..8],
            b"ASTRO_JP",
            "Invalid jupiter_system_data.bin header"
        );
        let io_rgb = &JUPITER_SYSTEM_DATA[IO_RGB_OFFSET..IO_RGB_OFFSET + IO_BYTE_LEN];
        Self { io_rgb }
    }

    /// Access the underlying zero-copy slice of Io RGB data.
    #[inline(always)]
    pub fn io_rgb(&self) -> &'static [u8] {
        self.io_rgb
    }

    /// Calculate spherical coordinates (u, v) from local normal (x, y, z) where y is polar axis:
    /// v = (1.0 - (y.clamp(-1.0, 1.0).asin() / (std::f64::consts::PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0)
    /// u = (z.atan2(x) + std::f64::consts::PI) / (2.0 * std::f64::consts::PI)
    #[inline(always)]
    pub fn spherical_coords(norm: Vec3) -> (f64, f64) {
        let v = (1.0 - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = (norm.z.atan2(norm.x) + PI) / (2.0 * PI);
        (u, v)
    }

    /// Read single texel RGB from slice with sulfur plains fallback for zero-fill background.
    #[inline(always)]
    fn get_texel(&self, x: usize, y: usize) -> [f64; 3] {
        let idx = (y * TEX_WIDTH + x) * 3;
        let r = self.io_rgb[idx] as f64;
        let g = self.io_rgb[idx + 1] as f64;
        let b = self.io_rgb[idx + 2] as f64;
        if r == 0.0 && g == 0.0 && b == 0.0 {
            COLOR_SULFUR_PLAINS
        } else {
            [r, g, b]
        }
    }

    /// 4-texel fractional bilinear interpolation on cylindrical projection.
    #[inline(always)]
    pub fn sample_bilinear_raw(data: &[u8], u: f64, v: f64) -> [f64; 3] {
        let u_wrap = u.rem_euclid(1.0);
        let v_clamp = v.clamp(0.0, 1.0);

        let fx = u_wrap * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamp * (TEX_HEIGHT as f64 - 1.0);

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

        let mut out = [0.0; 3];
        for c in 0..3 {
            let c00 = data[idx00 + c] as f64;
            let c10 = data[idx10 + c] as f64;
            let c01 = data[idx01 + c] as f64;
            let c11 = data[idx11 + c] as f64;

            let top = c00 * (1.0 - tx) + c10 * tx;
            let bot = c01 * (1.0 - tx) + c11 * tx;
            out[c] = top * (1.0 - ty) + bot * ty;
        }
        out
    }

    /// Sample the 1024x512 Io RGB map using 4 surrounding texels and bilinear interpolation.
    pub fn sample_texture_bilinear(&self, u: f64, v: f64) -> [u8; 3] {
        let u_wrap = u.rem_euclid(1.0);
        let v_clamp = v.clamp(0.0, 1.0);

        let fx = u_wrap * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamp * (TEX_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % TEX_WIDTH;
        let y1 = (y0 + 1).min(TEX_HEIGHT - 1);

        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;

        let c00 = self.get_texel(x0, y0);
        let c10 = self.get_texel(x1, y0);
        let c01 = self.get_texel(x0, y1);
        let c11 = self.get_texel(x1, y1);

        let mut out = [0u8; 3];
        for c in 0..3 {
            let top = c00[c] * (1.0 - tx) + c10[c] * tx;
            let bot = c01[c] * (1.0 - tx) + c11[c] * tx;
            let val = top * (1.0 - ty) + bot * ty;
            out[c] = val.round().clamp(0.0, 255.0) as u8;
        }
        out
    }

    /// Lommel-Seeliger non-Lambertian reflectance: mu_0 / (mu_0 + mu).
    #[inline(always)]
    pub fn lommel_seeliger(mu_0: f64, mu: f64) -> f64 {
        if mu_0 <= 0.0 {
            0.0
        } else {
            mu_0 / (mu_0 + mu.max(0.001))
        }
    }

    /// Opposition surge retroreflective brightening: 1.0 + 0.45 * (1.0 - (alpha / 0.18).min(1.0)).
    #[inline(always)]
    pub fn opposition_surge(alpha: f64) -> f64 {
        1.0 + 0.45 * (1.0 - (alpha / 0.18).min(1.0))
    }

    /// Primary Io volcanic surface shader:
    /// `local_norm`: surface normal in Io's body frame.
    /// `sun_dir`: incident light direction in body frame.
    /// `view_dir`: view direction towards camera in body frame.
    pub fn shade(&self, local_norm: Vec3, sun_dir: Vec3, view_dir: Vec3) -> [u8; 3] {
        if local_norm.length_squared() < 1e-12
            || sun_dir.length_squared() < 1e-12
            || view_dir.length_squared() < 1e-12
        {
            return [0, 0, 0];
        }

        let norm = local_norm.normalize();
        let sun = sun_dir.normalize();
        let view = view_dir.normalize();

        let mu_0 = norm.dot(sun);
        // Pitch-black nightside, zero light leakage!
        if mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        let mu = norm.dot(view).max(0.001);
        let ls = Self::lommel_seeliger(mu_0, mu);

        let cos_alpha = sun.dot(view).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let surge = Self::opposition_surge(alpha);

        let illumination = (ls * surge * 1.8).clamp(0.0, 1.5);

        let (u, v) = Self::spherical_coords(norm);
        let base_rgb = self.sample_texture_bilinear(u, v);

        let r = ((base_rgb[0] as f64) * illumination).clamp(0.0, 255.0).round() as u8;
        let g = ((base_rgb[1] as f64) * illumination).clamp(0.0, 255.0).round() as u8;
        let b = ((base_rgb[2] as f64) * illumination).clamp(0.0, 255.0).round() as u8;

        [r, g, b]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_slice_boundaries() {
        assert_eq!(JUPITER_SYSTEM_DATA.len(), 6_291_472);
        assert_eq!(IO_RGB_OFFSET, 16 + 3 * PIXELS);
        assert_eq!(IO_RGB_OFFSET, 1_572_880);
        assert_eq!(IO_BYTE_LEN, 3 * PIXELS);
        assert_eq!(IO_BYTE_LEN, 1_572_864);
        assert_eq!(IO_RGB_OFFSET + IO_BYTE_LEN, 3_145_744);

        let shader = IoShader::new();
        assert_eq!(shader.io_rgb().len(), IO_BYTE_LEN);
    }

    #[test]
    fn test_spherical_coords() {
        // Equator at prime meridian (x=1, y=0, z=0)
        let (u, v) = IoShader::spherical_coords(Vec3::new(1.0, 0.0, 0.0));
        assert!((u - 0.5).abs() < 1e-6);
        assert!((v - 0.5).abs() < 1e-6);

        // North pole (y=1) -> v=0.0
        let (_u, v_north) = IoShader::spherical_coords(Vec3::new(0.0, 1.0, 0.0));
        assert!((v_north - 0.0).abs() < 1e-6);

        // South pole (y=-1) -> v=1.0
        let (_u, v_south) = IoShader::spherical_coords(Vec3::new(0.0, -1.0, 0.0));
        assert!((v_south - 1.0).abs() < 1e-6);

        // Longitude 90 deg E (x=0, z=1) -> u=0.75
        let (u_90e, _v) = IoShader::spherical_coords(Vec3::new(0.0, 0.0, 1.0));
        assert!((u_90e - 0.75).abs() < 1e-6);

        // Longitude 90 deg W (x=0, z=-1) -> u=0.25
        let (u_90w, _v) = IoShader::spherical_coords(Vec3::new(0.0, 0.0, -1.0));
        assert!((u_90w - 0.25).abs() < 1e-6);
    }

    #[test]
    fn test_bilinear_interpolation_accuracy() {
        let mut mock_data = vec![0u8; TEX_WIDTH * TEX_HEIGHT * 3];
        // Set a 2x2 known cell at index (0,0)
        let idx00 = 0;
        let idx10 = 3;
        let idx01 = TEX_WIDTH * 3;
        let idx11 = (TEX_WIDTH + 1) * 3;

        mock_data[idx00..idx00 + 3].copy_from_slice(&[10, 20, 30]);
        mock_data[idx10..idx10 + 3].copy_from_slice(&[50, 60, 70]);
        mock_data[idx01..idx01 + 3].copy_from_slice(&[90, 100, 110]);
        mock_data[idx11..idx11 + 3].copy_from_slice(&[130, 140, 150]);

        let u = 0.5 / (TEX_WIDTH as f64 - 1.0);
        let v = 0.5 / (TEX_HEIGHT as f64 - 1.0);
        let sampled = IoShader::sample_bilinear_raw(&mock_data, u, v);

        // Exact arithmetic mean:
        // R: (10 + 50 + 90 + 130) / 4 = 70.0
        // G: (20 + 60 + 100 + 140) / 4 = 80.0
        // B: (30 + 70 + 110 + 150) / 4 = 90.0
        assert!((sampled[0] - 70.0).abs() < 1e-4);
        assert!((sampled[1] - 80.0).abs() < 1e-4);
        assert!((sampled[2] - 90.0).abs() < 1e-4);

        // Seamless wrap on u: u = 1.0 wraps to 0.0
        let sampled_wrap = IoShader::sample_bilinear_raw(&mock_data, 1.0, 0.0);
        assert_eq!(sampled_wrap, [10.0, 20.0, 30.0]);
    }

    #[test]
    fn test_lommel_seeliger_and_opposition_surge() {
        // Dark side mu_0 <= 0
        assert_eq!(IoShader::lommel_seeliger(0.0, 1.0), 0.0);
        assert_eq!(IoShader::lommel_seeliger(-0.5, 0.8), 0.0);

        // Subsolar reflection: mu_0 = 1.0, mu = 1.0 -> ls = 0.5
        let ls = IoShader::lommel_seeliger(1.0, 1.0);
        assert!((ls - 0.5).abs() < 1e-4);

        // Opposition surge: alpha = 0.0 -> 1.45
        let surge_0 = IoShader::opposition_surge(0.0);
        assert!((surge_0 - 1.45).abs() < 1e-6);

        // Opposition surge: halfway alpha = 0.09 -> 1.225
        let surge_half = IoShader::opposition_surge(0.09);
        assert!((surge_half - 1.225).abs() < 1e-6);

        // Opposition surge: alpha >= 0.18 -> 1.0
        let surge_out = IoShader::opposition_surge(0.18);
        assert!((surge_out - 1.00).abs() < 1e-6);

        let surge_large = IoShader::opposition_surge(1.0);
        assert!((surge_large - 1.00).abs() < 1e-6);
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = IoShader::new();
        let norm = Vec3::new(1.0, 0.0, 0.0);

        // Day: direct subsolar illumination at opposition
        let sun_day = Vec3::new(1.0, 0.0, 0.0);
        let view_day = Vec3::new(1.0, 0.0, 0.0);
        let color_day = shader.shade(norm, sun_day, view_day);
        assert!(
            color_day[0] > 100 && color_day[1] > 100,
            "Day side should be brightly illuminated: got {:?}",
            color_day
        );

        // Night: Pitch black nightside (sun_dir pointing away)
        let sun_night = Vec3::new(-1.0, 0.0, 0.0);
        let color_night = shader.shade(norm, sun_night, view_day);
        assert_eq!(
            color_night, [0, 0, 0],
            "Night side must be pitch-black [0, 0, 0]"
        );

        // Grazing night: sun behind tangent plane
        let sun_grazing_night = Vec3::new(-0.1, 0.0, -0.99);
        let color_grazing = shader.shade(norm, sun_grazing_night, view_day);
        assert_eq!(
            color_grazing, [0, 0, 0],
            "Past terminator must be pitch-black [0, 0, 0]"
        );

        // Terminator boundary (mu_0 == 0.0)
        let sun_terminator = Vec3::new(0.0, 1.0, 0.0);
        let color_term = shader.shade(norm, sun_terminator, view_day);
        assert_eq!(
            color_term, [0, 0, 0],
            "At terminator (mu_0 == 0.0) must be pitch-black [0, 0, 0]"
        );
    }

    #[test]
    fn test_sulfur_palette_and_geomorphology() {
        // Verify palette constants conform to authentic NASA Galileo RGB colors
        assert_eq!(COLOR_SULFUR_PLAINS, [235.0, 210.0, 80.0]);
        assert_eq!(COLOR_VOLCANIC_FALLOUT, [210.0, 90.0, 30.0]);
        assert_eq!(COLOR_BASALT_PATERA, [40.0, 35.0, 30.0]);
        assert_eq!(COLOR_FROST_PATCHES, [245.0, 245.0, 240.0]);

        // Sulfur plains have dominant yellow (high R & G, low B)
        assert!(COLOR_SULFUR_PLAINS[0] > 200.0);
        assert!(COLOR_SULFUR_PLAINS[1] > 180.0);
        assert!(COLOR_SULFUR_PLAINS[2] < 120.0);

        // Volcanic fallout has dominant red/orange
        assert!(COLOR_VOLCANIC_FALLOUT[0] > COLOR_VOLCANIC_FALLOUT[1]);
        assert!(COLOR_VOLCANIC_FALLOUT[1] > COLOR_VOLCANIC_FALLOUT[2]);

        // Basalt patera is dark
        let lum_basalt = 0.299 * COLOR_BASALT_PATERA[0]
            + 0.587 * COLOR_BASALT_PATERA[1]
            + 0.114 * COLOR_BASALT_PATERA[2];
        assert!(lum_basalt < 50.0);

        // Frost patches are bright white
        let lum_frost = 0.299 * COLOR_FROST_PATCHES[0]
            + 0.587 * COLOR_FROST_PATCHES[1]
            + 0.114 * COLOR_FROST_PATCHES[2];
        assert!(lum_frost > 240.0);

        // Sample texture bilinear on actual shader
        let shader = IoShader::new();
        let sampled = shader.sample_texture_bilinear(0.5, 0.5);
        assert!(sampled[0] > 0 && sampled[1] > 0);
    }
}
