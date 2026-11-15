//! Deimos Shader: High-fidelity USGS / Viking photographic & topographic shader
//! for Mars' outer moon Deimos.
//! - Triaxial ellipsoid morphology: semi-axes (7.5, 6.1, 5.5) km (sub-Mars X, along-orbit Z, polar Y).
//! - 1024x512 USGS controlled photographic photomosaic with bilinear interpolation.
//! - Thick regolith blanket smoothing out small craters, with bright faculae around Swift and Voltaire craters.
//! - Lommel-Seeliger non-Lambertian scattering with opposition surge and pitch-black nightside.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const SEMI_AXIS_A: f64 = 7.5; // sub-Mars X
pub const SEMI_AXIS_B: f64 = 6.1; // along-orbit Z
pub const SEMI_AXIS_C: f64 = 5.5; // polar Y

pub const GEOMETRIC_ALBEDO: f64 = 0.080;

pub struct DeimosShader {
    albedo_map: &'static [u8],    // 1024 * 512
    elevation_map: &'static [u8], // 1024 * 512
    width: usize,
    height: usize,
}

impl DeimosShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/mars_moons_data.bin");
        assert_eq!(&bytes[0..8], b"ASTRO_MN", "Invalid mars_moons_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;

        let deimos_albedo = &bytes[16 + 2 * pixels..16 + 3 * pixels];
        let deimos_elevation = &bytes[16 + 3 * pixels..16 + 4 * pixels];

        Self {
            albedo_map: deimos_albedo,
            elevation_map: deimos_elevation,
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
    fn sample_bilinear(&self, data: &[u8], u: f64, v: f64) -> f64 {
        let u_wrap = u.rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v_clamp = v.clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u_wrap.floor() as usize;
        let y0 = v_clamp.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = u_wrap - u_wrap.floor();
        let fy = v_clamp - v_clamp.floor();

        let p00 = data[y0 * self.width + x0] as f64;
        let p10 = data[y0 * self.width + x1] as f64;
        let p01 = data[y1 * self.width + x0] as f64;
        let p11 = data[y1 * self.width + x1] as f64;

        let top = p00 * (1.0 - fx) + p10 * fx;
        let bot = p01 * (1.0 - fx) + p11 * fx;
        top * (1.0 - fy) + bot * fy
    }

    pub fn sample_albedo(&self, lat_rad: f64, lon_rad: f64) -> [f64; 3] {
        let u = (lon_rad + PI) / (2.0 * PI);
        let v = (PI * 0.5 - lat_rad) / PI;
        let raw = self.sample_bilinear(self.albedo_map, u, v);

        // Calibrate raw USGS photomosaic brightness to geometric albedo ~0.080
        let lum = (raw / 95.0) * GEOMETRIC_ALBEDO;

        // Warm reddish D-type asteroid tint
        let r = lum * 1.15;
        let g = lum * 1.00;
        let b = lum * 0.88;

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

        let t_lon = Vec3::new(-base_norm.z, 0.0, base_norm.x).normalize();
        let t_lat = Vec3::new(
            -base_norm.y * lon_rad.cos(),
            cos_lat,
            -base_norm.y * lon_rad.sin(),
        ).normalize();

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

        let bump_scale = 0.0028;
        let grad_lon = (h_east - h_west) * bump_scale;
        let grad_lat = (h_north - h_south) * bump_scale;

        (base_norm - t_lon * grad_lon - t_lat * grad_lat).normalize()
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

        if mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        let mu = norm.dot(view_norm).max(0.0);
        let ls = Self::lommel_seeliger(mu_0, mu);

        let cos_alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let half_alpha = (alpha * 0.5).tan();
        let opposition_surge = 1.0 + 0.35 / (1.0 + 9.0 * half_alpha);

        let solar_irradiance = (1.0_f64 / 1.524).powi(2);
        let intensity_scale = 2200.0;

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
