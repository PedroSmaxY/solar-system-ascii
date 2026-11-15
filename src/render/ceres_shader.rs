//! Ceres Shader: High-fidelity NASA Dawn mission photographic & topographic shader
//! for the dwarf planet 1 Ceres.
//!
//! - Oblate spheroid geometry: equatorial radii a = b = 482.6 km, polar radius c = 445.6 km.
//! - 1024x512 NASA Dawn mission photographic photomosaic and elevation relief dataset.
//! - Bilinear interpolation with fractional weights (1 - tx) and (1 - ty).
//! - Calibrated albedo: dark carbonaceous chondrite regolith ([20, 21, 23], ~0.090) with
//!   bright sodium carbonate salt deposits in Occator Crater (Cerealia & Vinalia Faculae, [190, 195, 205], ~0.50).
//! - Topographic bump mapping via finite differences without polar singularities.
//! - Lommel-Seeliger non-Lambertian scattering with opposition surge and pitch-black nightside.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const SEMI_AXIS_A: f64 = 482.6; // Equatorial semi-major axis X (km)
pub const SEMI_AXIS_B: f64 = 482.6; // Equatorial semi-major axis Z (km)
pub const SEMI_AXIS_C: f64 = 445.6; // Polar semi-minor axis Y (km)

pub const MEAN_GEOMETRIC_ALBEDO: f64 = 0.090;
pub const OCCATOR_LON_DEG: f64 = -120.0;
pub const OCCATOR_LAT_DEG: f64 = 20.0;
pub const CEREALIA_FACULA_LON_DEG: f64 = -120.5;
pub const CEREALIA_FACULA_LAT_DEG: f64 = 19.53;
pub const AHUNA_MONS_LON_DEG: f64 = -44.0;
pub const AHUNA_MONS_LAT_DEG: f64 = -10.0;

pub struct CeresShader {
    albedo_map: &'static [u8],    // 1024 * 512
    elevation_map: &'static [u8], // 1024 * 512
    width: usize,
    height: usize,
}

impl CeresShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/asteroid_belt_data.bin");
        assert_eq!(&bytes[0..8], b"ASTRO_AB", "Invalid asteroid_belt_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;

        let albedo_map = &bytes[16..16 + pixels];
        let elevation_map = &bytes[16 + pixels..16 + 2 * pixels];

        Self {
            albedo_map,
            elevation_map,
            width,
            height,
        }
    }

    /// Lommel-Seeliger non-Lambertian reflectance: mu_0 / (mu_0 + mu).
    #[inline(always)]
    pub fn lommel_seeliger(mu_0: f64, mu: f64) -> f64 {
        if mu_0 <= 0.0 {
            0.0
        } else if mu_0 + mu > 1e-9 {
            mu_0 / (mu_0 + mu)
        } else {
            0.0
        }
    }

    /// Bilinear texture sampling with fractional weights (1 - tx) and (1 - ty).
    #[inline(always)]
    pub fn sample_bilinear(&self, data: &[u8], u: f64, v: f64) -> f64 {
        let u_wrap = u.rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v_clamp = v.clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u_wrap.floor() as usize;
        let y0 = v_clamp.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let tx = u_wrap - u_wrap.floor();
        let ty = v_clamp - v_clamp.floor();

        let p00 = data[y0 * self.width + x0] as f64;
        let p10 = data[y0 * self.width + x1] as f64;
        let p01 = data[y1 * self.width + x0] as f64;
        let p11 = data[y1 * self.width + x1] as f64;

        let top = p00 * (1.0 - tx) + p10 * tx;
        let bot = p01 * (1.0 - tx) + p11 * tx;
        top * (1.0 - ty) + bot * ty
    }

    /// Sample intrinsic Ceres albedo RGB from NASA Dawn dataset:
    /// - Mean regolith: ~0.090 albedo, charcoal ([75, 78, 85])
    /// - Occator Crater salt deposits (Cerealia & Vinalia Faculae): ~0.50 albedo, gleam ([235, 240, 255])
    pub fn sample_albedo(&self, lat_rad: f64, lon_rad: f64) -> [f64; 3] {
        let u = (lon_rad + PI) / (2.0 * PI);
        let v = (PI * 0.5 - lat_rad) / PI;
        let raw = self.sample_bilinear(self.albedo_map, u, v);

        // Calibrate raw NASA Dawn photomosaic (nominal regolith ~64, Occator peak ~176)
        if raw <= 64.0 {
            let factor = (raw / 64.0).max(0.0);
            [75.0 * factor, 78.0 * factor, 85.0 * factor]
        } else {
            let t = ((raw - 64.0) / (176.0 - 64.0)).clamp(0.0, 1.0);
            [
                75.0 + (235.0 - 75.0) * t,
                78.0 + (240.0 - 78.0) * t,
                85.0 + (255.0 - 85.0) * t,
            ]
        }
    }

    /// Topographic bump mapped surface normal on the oblate spheroid without polar singularities.
    pub fn surface_normal(&self, local_hit: Vec3) -> Vec3 {
        let inv_a = 1.0 / SEMI_AXIS_A;
        let inv_b = 1.0 / SEMI_AXIS_B;
        let inv_c = 1.0 / SEMI_AXIS_C;

        let base_norm = Vec3::new(
            local_hit.x * (inv_a * inv_a),
            local_hit.y * (inv_c * inv_c),
            local_hit.z * (inv_b * inv_b),
        )
        .normalize();

        let lat_rad = base_norm.y.clamp(-1.0, 1.0).asin();
        let lon_rad = base_norm.z.atan2(base_norm.x);
        let cos_lat = lat_rad.cos().max(1e-4);

        // Avoid polar singularities when constructing tangent vectors
        let xz_len = (base_norm.x * base_norm.x + base_norm.z * base_norm.z).sqrt();
        let t_lon = if xz_len > 1e-6 {
            Vec3::new(-base_norm.z / xz_len, 0.0, base_norm.x / xz_len)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        let t_lat = Vec3::new(
            -base_norm.y * lon_rad.cos(),
            cos_lat,
            -base_norm.y * lon_rad.sin(),
        )
        .normalize();

        // Finite differences with d_lon = 2*PI/1024, d_lat = PI/512
        let d_lon = 2.0 * PI / self.width as f64;
        let d_lat = PI / self.height as f64;

        let u = (lon_rad + PI) / (2.0 * PI);
        let v = (PI * 0.5 - lat_rad) / PI;

        let u_east = ((lon_rad + d_lon) + PI) / (2.0 * PI);
        let u_west = ((lon_rad - d_lon) + PI) / (2.0 * PI);
        let v_north = (PI * 0.5 - (lat_rad + d_lat)) / PI;
        let v_south = (PI * 0.5 - (lat_rad - d_lat)) / PI;

        let h_east = self.sample_bilinear(self.elevation_map, u_east, v);
        let h_west = self.sample_bilinear(self.elevation_map, u_west, v);
        let h_north = self.sample_bilinear(self.elevation_map, u, v_north);
        let h_south = self.sample_bilinear(self.elevation_map, u, v_south);

        let bump_scale = 0.0035;
        let grad_lon = (h_east - h_west) * bump_scale;
        let grad_lat = (h_north - h_south) * bump_scale;

        (base_norm - t_lon * grad_lon - t_lat * grad_lat).normalize()
    }

    /// Complete shading for Ceres point:
    /// - Topographic normal perturbation
    /// - Lommel-Seeliger scattering
    /// - Opposition surge
    /// - Complete pitch-black nightside when mu_0 <= 0.0
    pub fn shade(&self, local_hit: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        if local_hit.length_squared() < 1e-12 {
            return [0, 0, 0];
        }

        let sun_norm = local_sun.normalize();
        let view_norm = local_view.normalize();
        if sun_norm.length_squared() < 1e-6 || view_norm.length_squared() < 1e-6 {
            return [0, 0, 0];
        }

        let norm = self.surface_normal(local_hit);
        let mu_0 = norm.dot(sun_norm);

        // Complete pitch-black nightside when mu_0 <= 0.0
        if mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        let mu = norm.dot(view_norm).max(0.0);
        let ls = Self::lommel_seeliger(mu_0, mu);

        let cos_alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let half_alpha = (alpha * 0.5).tan();
        let opposition_surge = 1.0 + 0.35 / (1.0 + 9.0 * half_alpha);

        // Subsolar opposition reference: ls(1, 1) = 0.5, surge(0) = 1.35, product = 0.675
        let lighting = (ls * opposition_surge) / 0.675;

        let inv_a = 1.0 / SEMI_AXIS_A;
        let inv_b = 1.0 / SEMI_AXIS_B;
        let inv_c = 1.0 / SEMI_AXIS_C;
        let base_norm = Vec3::new(
            local_hit.x * (inv_a * inv_a),
            local_hit.y * (inv_c * inv_c),
            local_hit.z * (inv_b * inv_b),
        )
        .normalize();

        let lat_rad = base_norm.y.clamp(-1.0, 1.0).asin();
        let lon_rad = base_norm.z.atan2(base_norm.x);
        let albedo = self.sample_albedo(lat_rad, lon_rad);

        let r = (albedo[0] * lighting).clamp(0.0, 255.0) as u8;
        let g = (albedo[1] * lighting).clamp(0.0, 255.0) as u8;
        let b = (albedo[2] * lighting).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceres_dataset_load() {
        let shader = CeresShader::new();
        assert_eq!(shader.width, 1024);
        assert_eq!(shader.height, 512);
        assert_eq!(shader.albedo_map.len(), 1024 * 512);
        assert_eq!(shader.elevation_map.len(), 1024 * 512);
    }

    #[test]
    fn test_sample_bilinear() {
        static GRID: [u8; 4] = [10u8, 20, 30, 40];
        // Custom 2x2 mock shader
        let mock_shader = CeresShader {
            albedo_map: &GRID,
            elevation_map: &GRID,
            width: 2,
            height: 2,
        };
        let val = mock_shader.sample_bilinear(&GRID, 0.5, 0.5);
        // (10 * 0.5 + 20 * 0.5) * 0.5 + (30 * 0.5 + 40 * 0.5) * 0.5 = 15 * 0.5 + 35 * 0.5 = 25.0
        assert!((val - 25.0).abs() < 1e-4);
    }

    #[test]
    fn test_lommel_seeliger_and_nightside() {
        assert_eq!(CeresShader::lommel_seeliger(0.0, 1.0), 0.0);
        assert_eq!(CeresShader::lommel_seeliger(-0.5, 0.8), 0.0);
        let ls = CeresShader::lommel_seeliger(1.0, 1.0);
        assert!((ls - 0.5).abs() < 1e-6);

        let shader = CeresShader::new();
        // Point on surface with sun pointing away (night side)
        let hit = Vec3::new(482.6, 0.0, 0.0);
        let sun = Vec3::new(-1.0, 0.0, 0.0); // Opposing sun
        let view = Vec3::new(1.0, 0.0, 0.0);
        let night_rgb = shader.shade(hit, sun, view);
        assert_eq!(night_rgb, [0, 0, 0], "Night side must be pitch-black [0, 0, 0]");
    }

    #[test]
    fn test_occator_crater_faculae_and_base_regolith() {
        let shader = CeresShader::new();
        let facula_lon = CEREALIA_FACULA_LON_DEG * PI / 180.0;
        let facula_lat = CEREALIA_FACULA_LAT_DEG * PI / 180.0;

        let occ_albedo = shader.sample_albedo(facula_lat, facula_lon);
        // Cerealia / Vinalia Faculae gleam brightly
        assert!(occ_albedo[0] > 200.0, "Cerealia Facula red > 200: got {}", occ_albedo[0]);
        assert!(occ_albedo[1] > 200.0, "Cerealia Facula green > 200: got {}", occ_albedo[1]);
        assert!(occ_albedo[2] > 220.0, "Cerealia Facula blue > 220: got {}", occ_albedo[2]);

        // Base regolith at 0 lon, 0 lat
        let reg_albedo = shader.sample_albedo(0.0, 0.0);
        assert!((reg_albedo[0] - 75.0).abs() < 15.0, "Base regolith R around 75: got {}", reg_albedo[0]);
        assert!((reg_albedo[1] - 78.0).abs() < 15.0, "Base regolith G around 78: got {}", reg_albedo[1]);
        assert!((reg_albedo[2] - 85.0).abs() < 15.0, "Base regolith B around 85: got {}", reg_albedo[2]);
    }

    #[test]
    fn test_polar_singularity_free_normal() {
        let shader = CeresShader::new();
        // North pole hit
        let hit_north = Vec3::new(0.0, SEMI_AXIS_C, 0.0);
        let norm_north = shader.surface_normal(hit_north);
        assert!(!norm_north.x.is_nan() && !norm_north.y.is_nan() && !norm_north.z.is_nan());
        assert!((norm_north.length() - 1.0).abs() < 1e-4);

        // South pole hit
        let hit_south = Vec3::new(0.0, -SEMI_AXIS_C, 0.0);
        let norm_south = shader.surface_normal(hit_south);
        assert!(!norm_south.x.is_nan() && !norm_south.y.is_nan() && !norm_south.z.is_nan());
        assert!((norm_south.length() - 1.0).abs() < 1e-4);
    }
}
