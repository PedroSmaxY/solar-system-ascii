//! Photorealistic Europa Raytracing Shader:
//! - Direct zero-copy sampling of authentic NASA Galileo/Juno 1024x512 albedo and relief maps.
//! - Bilinear interpolation with seamless longitude wrapping.
//! - Topographic bump mapping via finite differences across ice ridges and Conamara Chaos.
//! - Authentic Europa color palette: pristine ice plains, fractured chaos, reddish-brown lineae.
//! - Lommel-Seeliger non-Lambertian reflectance with opposition surge and pitch-black nightside.

use crate::math::Vec3;
use crate::render::jupiter_shader::{JUPITER_SYSTEM_DATA, PIXELS, TEX_HEIGHT, TEX_WIDTH};
use std::f64::consts::PI;

pub const EUROPA_ALBEDO_OFFSET: usize = 16 + 6 * PIXELS;
pub const EUROPA_ELEVATION_OFFSET: usize = 16 + 7 * PIXELS;

pub const COLOR_LINEAE: [f64; 3] = [145.0, 80.0, 55.0];
pub const COLOR_CHAOS: [f64; 3] = [205.0, 210.0, 220.0];
pub const COLOR_PLAINS: [f64; 3] = [230.0, 235.0, 245.0];

pub const BUMP_SCALE: f64 = 0.008;

pub struct EuropaShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl Default for EuropaShader {
    fn default() -> Self {
        Self::new()
    }
}

impl EuropaShader {
    pub fn new() -> Self {
        assert_eq!(
            &JUPITER_SYSTEM_DATA[0..8],
            b"ASTRO_JP",
            "Invalid jupiter_system_data.bin header"
        );
        let albedo_map = &JUPITER_SYSTEM_DATA[EUROPA_ALBEDO_OFFSET..EUROPA_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &JUPITER_SYSTEM_DATA[EUROPA_ELEVATION_OFFSET..EUROPA_ELEVATION_OFFSET + PIXELS];
        Self {
            albedo_map,
            elevation_map,
        }
    }

    /// Calculate spherical coordinates (u, v) from local normal (x, y, z) where y is polar axis:
    /// v = (1.0 - (y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0)
    /// u = (z.atan2(x) + PI) / (2.0 * PI)
    #[inline(always)]
    pub fn spherical_coords(norm: Vec3) -> (f64, f64) {
        let v = (1.0 - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = (norm.z.atan2(norm.x) + PI) / (2.0 * PI);
        (u, v)
    }

    /// 4-texel fractional bilinear interpolation on cylindrical map with seamless wrapping on u.
    #[inline(always)]
    pub fn sample_bilinear(data: &[u8], u: f64, v: f64) -> f64 {
        let u_clamped = u.rem_euclid(1.0);
        let v_clamped = v.clamp(0.0, 1.0);

        let fx = u_clamped * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamped * (TEX_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % TEX_WIDTH;
        let y1 = (y0 + 1).min(TEX_HEIGHT - 1);

        let tx = fx - fx.floor();
        let ty = fy - fy.floor();

        let p00 = data[y0 * TEX_WIDTH + x0] as f64;
        let p10 = data[y0 * TEX_WIDTH + x1] as f64;
        let p01 = data[y1 * TEX_WIDTH + x0] as f64;
        let p11 = data[y1 * TEX_WIDTH + x1] as f64;

        let top = p00 * (1.0 - tx) + p10 * tx;
        let bot = p01 * (1.0 - tx) + p11 * tx;
        top * (1.0 - ty) + bot * ty
    }

    #[inline(always)]
    pub fn sample_albedo(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.albedo_map, u, v)
    }

    #[inline(always)]
    pub fn sample_elevation(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.elevation_map, u, v)
    }

    /// Map raw 8-bit albedo to authentic Europa RGB palette:
    /// - Pristine water ice plains: ~[230, 235, 245]
    /// - Fractured ice blocks & Conamara Chaos: ~[205, 210, 220]
    /// - Reddish-brown Lineae (cycloid ridges, hydrated salts): ~[145, 80, 55]
    #[inline]
    pub fn albedo_to_rgb(raw: f64) -> [f64; 3] {
        if raw <= 195.0 {
            let t = ((raw - 160.0) / (195.0 - 160.0)).clamp(0.0, 1.0);
            let smooth_t = t * t * (3.0 - 2.0 * t);
            [
                COLOR_LINEAE[0] + (COLOR_CHAOS[0] - COLOR_LINEAE[0]) * smooth_t,
                COLOR_LINEAE[1] + (COLOR_CHAOS[1] - COLOR_LINEAE[1]) * smooth_t,
                COLOR_LINEAE[2] + (COLOR_CHAOS[2] - COLOR_LINEAE[2]) * smooth_t,
            ]
        } else {
            let t = ((raw - 195.0) / (220.0 - 195.0)).clamp(0.0, 1.0);
            let smooth_t = t * t * (3.0 - 2.0 * t);
            [
                COLOR_CHAOS[0] + (COLOR_PLAINS[0] - COLOR_CHAOS[0]) * smooth_t,
                COLOR_CHAOS[1] + (COLOR_PLAINS[1] - COLOR_CHAOS[1]) * smooth_t,
                COLOR_CHAOS[2] + (COLOR_PLAINS[2] - COLOR_CHAOS[2]) * smooth_t,
            ]
        }
    }

    /// Compute finite difference elevation gradient (du, dv) at (u, v).
    pub fn compute_elevation_gradient(&self, u: f64, v: f64) -> (f64, f64) {
        let du_step = 1.0 / TEX_WIDTH as f64;
        let dv_step = 1.0 / TEX_HEIGHT as f64;

        let h_east = self.sample_elevation(u + du_step, v);
        let h_west = self.sample_elevation(u - du_step, v);
        let h_south = self.sample_elevation(u, (v + dv_step).clamp(0.0, 1.0));
        let h_north = self.sample_elevation(u, (v - dv_step).clamp(0.0, 1.0));

        let du = (h_east - h_west) * 0.5;
        let dv = (h_south - h_north) * 0.5;
        (du, dv)
    }

    /// Perturb unit surface normal using topographic elevation gradient.
    pub fn surface_normal(&self, norm: Vec3) -> Vec3 {
        let (u, v) = Self::spherical_coords(norm);
        let (du, dv) = self.compute_elevation_gradient(u, v);

        // Construct tangent & bitangent vectors without polar singularities
        let xz_len = (norm.x * norm.x + norm.z * norm.z).sqrt();
        let tangent = if xz_len > 1e-6 {
            Vec3::new(-norm.z / xz_len, 0.0, norm.x / xz_len)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let bitangent = norm.cross(tangent);

        (norm - tangent * (du * BUMP_SCALE) - bitangent * (dv * BUMP_SCALE)).normalize()
    }

    #[inline(always)]
    pub fn perturbed_normal(&self, norm: Vec3) -> Vec3 {
        self.surface_normal(norm)
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

    /// Opposition surge retroreflective brightening: 1.0 + 0.40 * (1.0 - min(alpha / 0.15, 1.0)).
    #[inline(always)]
    pub fn opposition_surge(alpha: f64) -> f64 {
        1.0 + 0.40 * (1.0 - (alpha / 0.15).min(1.0))
    }

    /// Primary Europa photorealistic surface shader.
    /// `local_norm`: surface normal or hit vector from center in Europa's body frame.
    /// `sun_dir`: incident sunlight vector in body frame.
    /// `view_dir`: view direction towards camera in body frame.
    pub fn shade(&self, local_norm: Vec3, sun_dir: Vec3, view_dir: Vec3) -> [u8; 3] {
        if local_norm.length_squared() < 1e-12
            || sun_dir.length_squared() < 1e-12
            || view_dir.length_squared() < 1e-12
        {
            return [0, 0, 0];
        }

        let base_norm = local_norm.normalize();
        let sun_norm = sun_dir.normalize();
        let view_norm = view_dir.normalize();

        let perturbed_norm = self.surface_normal(base_norm);

        let mu_0 = perturbed_norm.dot(sun_norm);
        // Pitch-black nightside, zero light leakage
        if mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        let mu = perturbed_norm.dot(view_norm).max(0.001);
        let ls = mu_0 / (mu_0 + mu);

        let cos_alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let surge = Self::opposition_surge(alpha);

        let illumination = (ls * surge * 2.0).clamp(0.0, 1.6);

        let (u, v) = Self::spherical_coords(base_norm);
        let raw_albedo = self.sample_albedo(u, v);
        let base_rgb = Self::albedo_to_rgb(raw_albedo);

        let r = (base_rgb[0] * illumination).clamp(0.0, 255.0) as u8;
        let g = (base_rgb[1] * illumination).clamp(0.0, 255.0) as u8;
        let b = (base_rgb[2] * illumination).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_europa_slice_boundaries() {
        assert_eq!(JUPITER_SYSTEM_DATA.len(), 6_291_472);
        assert_eq!(EUROPA_ALBEDO_OFFSET, 16 + 6 * PIXELS);
        assert_eq!(EUROPA_ALBEDO_OFFSET, 3_145_744);
        assert_eq!(EUROPA_ELEVATION_OFFSET, 16 + 7 * PIXELS);
        assert_eq!(EUROPA_ELEVATION_OFFSET, 3_670_032);

        let shader = EuropaShader::new();
        assert_eq!(shader.albedo_map.len(), PIXELS);
        assert_eq!(shader.elevation_map.len(), PIXELS);
    }

    #[test]
    fn test_spherical_coords() {
        // Equator (x=1, y=0, z=0)
        let (u, v) = EuropaShader::spherical_coords(Vec3::new(1.0, 0.0, 0.0));
        assert!((u - 0.5).abs() < 1e-6);
        assert!((v - 0.5).abs() < 1e-6);

        // North Pole (y=1) -> v=0.0
        let (_u, v_north) = EuropaShader::spherical_coords(Vec3::new(0.0, 1.0, 0.0));
        assert!((v_north - 0.0).abs() < 1e-6);

        // South Pole (y=-1) -> v=1.0
        let (_u, v_south) = EuropaShader::spherical_coords(Vec3::new(0.0, -1.0, 0.0));
        assert!((v_south - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sample_bilinear() {
        let mut mock_data = vec![0u8; TEX_WIDTH * TEX_HEIGHT];
        // Set a 2x2 known cell at index 0,0
        mock_data[0] = 10;
        mock_data[1] = 20;
        mock_data[TEX_WIDTH] = 30;
        mock_data[TEX_WIDTH + 1] = 40;

        let u = 0.5 / (TEX_WIDTH as f64 - 1.0);
        let v = 0.5 / (TEX_HEIGHT as f64 - 1.0);
        let sampled = EuropaShader::sample_bilinear(&mock_data, u, v);
        assert!((sampled - 25.0).abs() < 1e-4);

        // Wrap test: u = 1.0 wraps to 0.0
        let sampled_wrap = EuropaShader::sample_bilinear(&mock_data, 1.0, 0.0);
        assert_eq!(sampled_wrap, mock_data[0] as f64);
    }

    #[test]
    fn test_bump_map_gradient_and_surface_normal() {
        let shader = EuropaShader::new();

        // Check equator
        let norm_eq = Vec3::new(1.0, 0.0, 0.0);
        let perturbed_eq = shader.surface_normal(norm_eq);
        assert!(!perturbed_eq.x.is_nan() && !perturbed_eq.y.is_nan() && !perturbed_eq.z.is_nan());
        assert!((perturbed_eq.length() - 1.0).abs() < 1e-5);

        // Check North Pole
        let norm_np = Vec3::new(0.0, 1.0, 0.0);
        let perturbed_np = shader.surface_normal(norm_np);
        assert!(!perturbed_np.x.is_nan() && !perturbed_np.y.is_nan() && !perturbed_np.z.is_nan());
        assert!((perturbed_np.length() - 1.0).abs() < 1e-5);

        // Check South Pole
        let norm_sp = Vec3::new(0.0, -1.0, 0.0);
        let perturbed_sp = shader.surface_normal(norm_sp);
        assert!(!perturbed_sp.x.is_nan() && !perturbed_sp.y.is_nan() && !perturbed_sp.z.is_nan());
        assert!((perturbed_sp.length() - 1.0).abs() < 1e-5);

        // Gradient calculation check
        let (du, dv) = shader.compute_elevation_gradient(0.5, 0.5);
        assert!(!du.is_nan() && !dv.is_nan());
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = EuropaShader::new();
        let norm = Vec3::new(1.0, 0.0, 0.0);

        // Day: Direct subsolar illumination at opposition
        let sun_day = Vec3::new(1.0, 0.0, 0.0);
        let view_day = Vec3::new(1.0, 0.0, 0.0);
        let color_day = shader.shade(norm, sun_day, view_day);
        assert!(
            color_day[0] > 100 && color_day[1] > 100 && color_day[2] > 100,
            "Day side should be brightly illuminated: got {:?}",
            color_day
        );

        // Night: Pitch black nightside
        let sun_night = Vec3::new(-1.0, 0.0, 0.0);
        let color_night = shader.shade(norm, sun_night, view_day);
        assert_eq!(
            color_night, [0, 0, 0],
            "Night side must be pitch-black [0, 0, 0]"
        );

        // Grazing night: sun pointing backwards
        let sun_grazing_night = Vec3::new(-0.2, 0.0, -0.98);
        let color_grazing = shader.shade(norm, sun_grazing_night, view_day);
        assert_eq!(
            color_grazing, [0, 0, 0],
            "Nightside past terminator must be pitch-black [0, 0, 0]"
        );
    }

    #[test]
    fn test_ice_vs_lineae_albedo() {
        // Reddish-brown Lineae
        let lineae = EuropaShader::albedo_to_rgb(160.0);
        assert!((lineae[0] - COLOR_LINEAE[0]).abs() < 1e-4);
        assert!((lineae[1] - COLOR_LINEAE[1]).abs() < 1e-4);
        assert!((lineae[2] - COLOR_LINEAE[2]).abs() < 1e-4);
        assert!(lineae[0] > lineae[1] && lineae[1] > lineae[2]); // reddish-brown

        // Conamara Chaos & fractured ice
        let chaos = EuropaShader::albedo_to_rgb(195.0);
        assert!((chaos[0] - COLOR_CHAOS[0]).abs() < 1e-4);
        assert!((chaos[1] - COLOR_CHAOS[1]).abs() < 1e-4);
        assert!((chaos[2] - COLOR_CHAOS[2]).abs() < 1e-4);

        // Pristine water ice plains
        let plains = EuropaShader::albedo_to_rgb(220.0);
        assert!((plains[0] - COLOR_PLAINS[0]).abs() < 1e-4);
        assert!((plains[1] - COLOR_PLAINS[1]).abs() < 1e-4);
        assert!((plains[2] - COLOR_PLAINS[2]).abs() < 1e-4);
        assert!(plains[2] >= plains[1] && plains[1] >= plains[0]); // icy blue-white

        // Luminance check: Plains must have much higher reflectance than Lineae
        let lum_lineae = 0.299 * lineae[0] + 0.587 * lineae[1] + 0.114 * lineae[2];
        let lum_plains = 0.299 * plains[0] + 0.587 * plains[1] + 0.114 * plains[2];
        assert!(lum_plains > lum_lineae * 2.0);
    }

    #[test]
    fn test_opposition_surge_and_lommel_seeliger() {
        assert_eq!(EuropaShader::lommel_seeliger(0.0, 1.0), 0.0);
        assert_eq!(EuropaShader::lommel_seeliger(-0.5, 0.8), 0.0);
        let ls = EuropaShader::lommel_seeliger(1.0, 1.0);
        assert!((ls - 0.5).abs() < 1e-4);

        let surge_0 = EuropaShader::opposition_surge(0.0);
        assert!((surge_0 - 1.40).abs() < 1e-6);

        let surge_half = EuropaShader::opposition_surge(0.075);
        assert!((surge_half - 1.20).abs() < 1e-6);

        let surge_out = EuropaShader::opposition_surge(0.15);
        assert!((surge_out - 1.00).abs() < 1e-6);

        let surge_large = EuropaShader::opposition_surge(1.0);
        assert!((surge_large - 1.00).abs() < 1e-6);
    }
}
