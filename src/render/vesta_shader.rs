//! Vesta Shader: High-fidelity NASA Dawn mission photographic & topographic shader
//! for protoplanet 4 Vesta.
//! - Triaxial ellipsoid morphology: semi-axes (286.3, 278.6, 223.2) km (sub-solar X, along-orbit Z, polar Y).
//! - NASA Dawn 1024x512 global controlled photographic photomosaic & topographic relief
//!   embedded directly from `assets/asteroid_belt_data.bin` (3rd and 4th layers).
//! - Topographic bump mapping capturing the massive Rheasilvia south polar impact basin
//!   (lat -75°, 500 km diameter, with 22 km central peak) and the Divalia Fossae equatorial trough system.
//! - Bilinear interpolation on 2x2 texel neighborhood for both albedo and relief without polar singularities.
//! - Lommel-Seeliger non-Lambertian scattering with Hapke opposition surge and complete pitch-black nightside.
//! - High geometric albedo ~0.42 calibrated to basaltic eucrite crust with warm basaltic tint:
//!   base sunlit albedo reaches [150, 142, 130] with peaks on bright crater rims.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const SEMI_AXIS_A: f64 = 286.3; // sub-solar X (km)
pub const SEMI_AXIS_B: f64 = 278.6; // along-orbit Z (km)
pub const SEMI_AXIS_C: f64 = 223.2; // polar Y (km)

pub const GEOMETRIC_ALBEDO: f64 = 0.42;

pub struct VestaShader {
    albedo_map: &'static [u8],    // 1024 * 512
    elevation_map: &'static [u8], // 1024 * 512
    width: usize,
    height: usize,
}

impl VestaShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/asteroid_belt_data.bin");
        assert_eq!(&bytes[0..8], b"ASTRO_AB", "Invalid asteroid_belt_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;

        // Layer 1: Ceres Albedo    (offset 16 .. 16 + pixels)
        // Layer 2: Ceres Elevation (offset 16 + pixels .. 16 + 2 * pixels)
        // Layer 3: Vesta Albedo    (offset 16 + 2 * pixels .. 16 + 3 * pixels)
        // Layer 4: Vesta Elevation (offset 16 + 3 * pixels .. 16 + 4 * pixels)
        let vesta_albedo = &bytes[16 + 2 * pixels..16 + 3 * pixels];
        let vesta_elevation = &bytes[16 + 3 * pixels..16 + 4 * pixels];

        Self {
            albedo_map: vesta_albedo,
            elevation_map: vesta_elevation,
            width,
            height,
        }
    }

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

    #[inline(always)]
    pub fn sample_elevation(&self, lat_rad: f64, lon_rad: f64) -> f64 {
        let u = (lon_rad + PI) / (2.0 * PI);
        let v = (PI * 0.5 - lat_rad) / PI;
        self.sample_bilinear(self.elevation_map, u, v)
    }

    pub fn sample_albedo(&self, lat_rad: f64, lon_rad: f64) -> [f64; 3] {
        let u = (lon_rad + PI) / (2.0 * PI);
        let v = (PI * 0.5 - lat_rad) / PI;
        let raw = self.sample_bilinear(self.albedo_map, u, v);

        // Calibrate raw NASA Dawn photomosaic brightness to geometric albedo ~0.42
        // Average raw pixel value in dataset is ~122.7.
        let lum = (raw / 122.7) * GEOMETRIC_ALBEDO;

        // Warm basaltic tint (eucrite basalt crust):
        // Base sunlit albedo reaches [150, 142, 130] with peaks on bright crater rims.
        let r = lum * (150.0 / 142.0);
        let g = lum * 1.0000;
        let b = lum * (130.0 / 142.0);

        [r, g, b]
    }

    pub fn surface_normal(&self, local_hit: Vec3) -> Vec3 {
        let inv_a = 1.0 / SEMI_AXIS_A;
        let inv_c = 1.0 / SEMI_AXIS_C;
        let inv_b = 1.0 / SEMI_AXIS_B;

        let base_norm = Vec3::new(
            local_hit.x * (inv_a * inv_a),
            local_hit.y * (inv_c * inv_c),
            local_hit.z * (inv_b * inv_b),
        )
        .normalize();

        let lat_rad = base_norm.y.clamp(-1.0, 1.0).asin();
        let lon_rad = base_norm.z.atan2(base_norm.x);
        let cos_lat = lat_rad.cos().max(1e-4);

        // Tangent vectors without polar singularities
        let t_lon = if base_norm.x.abs() > 1e-6 || base_norm.z.abs() > 1e-6 {
            Vec3::new(-base_norm.z, 0.0, base_norm.x).normalize()
        } else {
            Vec3::new(-lon_rad.sin(), 0.0, lon_rad.cos())
        };

        let t_lat = Vec3::new(
            -base_norm.y * lon_rad.cos(),
            cos_lat,
            -base_norm.y * lon_rad.sin(),
        )
        .normalize();

        let d_lon = 2.0 * PI / self.width as f64;
        let d_lat = PI / self.height as f64;

        let u_center = (lon_rad + PI) / (2.0 * PI);
        let u_east = ((lon_rad + d_lon) + PI) / (2.0 * PI);
        let u_west = ((lon_rad - d_lon) + PI) / (2.0 * PI);
        let v_center = (PI * 0.5 - lat_rad) / PI;
        let v_north = (PI * 0.5 - (lat_rad + d_lat)) / PI;
        let v_south = (PI * 0.5 - (lat_rad - d_lat)) / PI;

        let h_east = self.sample_bilinear(self.elevation_map, u_east, v_center);
        let h_west = self.sample_bilinear(self.elevation_map, u_west, v_center);
        let h_north = self.sample_bilinear(self.elevation_map, u_center, v_north);
        let h_south = self.sample_bilinear(self.elevation_map, u_center, v_south);

        let bump_scale = 0.0030;
        let grad_lon = (h_east - h_west) * bump_scale;
        let grad_lat = (h_north - h_south) * bump_scale;

        let perturbed = base_norm - t_lon * grad_lon - t_lat * grad_lat;
        let res = perturbed.normalize();
        if res.length_squared() > 1e-6 {
            res
        } else {
            base_norm
        }
    }

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

        // Opposition surge: 1.0 + 0.35 / (1.0 + 9.0 * half_alpha)
        let cos_alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let half_alpha = (alpha * 0.5).tan();
        let opposition_surge = 1.0 + 0.35 / (1.0 + 9.0 * half_alpha);

        let solar_irradiance = (1.0_f64 / 2.362).powi(2); // Vesta semi-major axis ~2.36 AU
        let intensity_scale = 2795.0;

        let base_norm = local_hit.normalize();
        let lat_rad = base_norm.y.clamp(-1.0, 1.0).asin();
        let lon_rad = base_norm.z.atan2(base_norm.x);
        let albedo = self.sample_albedo(lat_rad, lon_rad);

        let diffuse = ls * opposition_surge * solar_irradiance * intensity_scale;

        let r = (albedo[0] * diffuse).clamp(0.0, 255.0) as u8;
        let g = (albedo[1] * diffuse).clamp(0.0, 255.0) as u8;
        let b = (albedo[2] * diffuse).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}

impl Default for VestaShader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vesta_dataset_loading() {
        let shader = VestaShader::new();
        assert_eq!(shader.width, 1024);
        assert_eq!(shader.height, 512);
        assert_eq!(shader.albedo_map.len(), 1024 * 512);
        assert_eq!(shader.elevation_map.len(), 1024 * 512);
    }

    #[test]
    fn test_bilinear_filtering() {
        let test_data = [10u8, 20, 30, 40];
        // 2x2 synthetic check
        let dummy_shader = VestaShader {
            albedo_map: &[],
            elevation_map: &[],
            width: 2,
            height: 2,
        };
        // Top-left: u=0.0, v=0.0 -> 10.0
        let val_tl = dummy_shader.sample_bilinear(&test_data, 0.0, 0.0);
        assert!((val_tl - 10.0).abs() < 1e-4);

        // Center: u=0.5, v=0.5 -> (10+20+30+40)/4 = 25.0
        let val_c = dummy_shader.sample_bilinear(&test_data, 0.5, 0.5);
        assert!((val_c - 25.0).abs() < 1e-4);
    }

    #[test]
    fn test_surface_normal_triaxial() {
        let shader = VestaShader::new();
        // Point along semi-axis a (X = 286.3)
        let norm_x = shader.surface_normal(Vec3::new(SEMI_AXIS_A, 0.0, 0.0));
        assert!(norm_x.x > 0.95);
        assert!((norm_x.length() - 1.0).abs() < 1e-4);

        // Point along polar axis c (Y = 223.2)
        let norm_y = shader.surface_normal(Vec3::new(0.0, SEMI_AXIS_C, 0.0));
        assert!(norm_y.y > 0.95);
        assert!((norm_y.length() - 1.0).abs() < 1e-4);

        // Point along semi-axis b (Z = 278.6)
        let norm_z = shader.surface_normal(Vec3::new(0.0, 0.0, SEMI_AXIS_B));
        assert!(norm_z.z > 0.95);
        assert!((norm_z.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_albedo_warm_basaltic_tint() {
        let shader = VestaShader::new();
        let albedo = shader.sample_albedo(0.0, 0.0);
        // Basaltic eucrite tint: R > G > B
        assert!(albedo[0] > albedo[1], "Red channel must exceed Green for warm basalt");
        assert!(albedo[1] > albedo[2], "Green channel must exceed Blue for warm basalt");
        // Check geometric albedo scale ~0.42
        assert!(albedo[1] >= 0.25 && albedo[1] <= 0.60, "Albedo ~0.42 expected");
    }

    #[test]
    fn test_lommel_seeliger_and_nightside() {
        let shader = VestaShader::new();
        let local_hit = Vec3::new(SEMI_AXIS_A, 0.0, 0.0);
        let sun_facing = Vec3::new(1.0, 0.0, 0.0);
        let view_facing = Vec3::new(1.0, 0.0, 0.0);

        // Sunlit side
        let color_day = shader.shade(local_hit, sun_facing, view_facing);
        assert!(color_day[0] >= 140, "Base sunlit red should reach ~150: got {}", color_day[0]);
        assert!(color_day[1] >= 135, "Base sunlit green should reach ~142: got {}", color_day[1]);
        assert!(color_day[2] >= 120, "Base sunlit blue should reach ~130: got {}", color_day[2]);

        // Complete pitch-black nightside
        let sun_opposite = Vec3::new(-1.0, 0.0, 0.0);
        let color_night = shader.shade(local_hit, sun_opposite, view_facing);
        assert_eq!(color_night, [0, 0, 0], "Nightside must be pitch-black [0, 0, 0]");
    }

    #[test]
    fn test_polar_singularity_free() {
        let shader = VestaShader::new();
        // Exact North pole
        let norm_np = shader.surface_normal(Vec3::new(0.0, SEMI_AXIS_C, 0.0));
        assert!(!norm_np.x.is_nan() && !norm_np.y.is_nan() && !norm_np.z.is_nan());
        assert!((norm_np.length() - 1.0).abs() < 1e-4);

        // Exact South pole (Rheasilvia basin center)
        let norm_sp = shader.surface_normal(Vec3::new(0.0, -SEMI_AXIS_C, 0.0));
        assert!(!norm_sp.x.is_nan() && !norm_sp.y.is_nan() && !norm_sp.z.is_nan());
        assert!((norm_sp.length() - 1.0).abs() < 1e-4);
    }
}
