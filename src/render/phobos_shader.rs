//! Phobos Shader: High-fidelity USGS / Viking / Mars Express photographic & topographic shader
//! for Mars' inner moon Phobos.
//! - Triaxial ellipsoid morphology: semi-axes (13.0, 11.4, 9.1) km (sub-Mars X, along-orbit Z, polar Y).
//! - 1024x512 USGS controlled photographic photomosaic with bilinear interpolation.
//! - Topographic relief bump gradient rendering Stickney crater (9 km), Hall, Roche, and fracture grooves.
//! - Lommel-Seeliger non-Lambertian scattering with opposition surge and pitch-black nightside.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const SEMI_AXIS_A: f64 = 13.0; // sub-Mars X
pub const SEMI_AXIS_B: f64 = 11.4; // along-orbit Z
pub const SEMI_AXIS_C: f64 = 9.1;  // polar Y

pub const BASE_ALBEDO: f64 = 0.071;
pub const STICKNEY_LON_RAD: f64 = -50.0 * PI / 180.0;
pub const STICKNEY_LAT_RAD: f64 = 0.0;
pub const STICKNEY_RADIUS_RAD: f64 = 0.65;
pub const STICKNEY_DEPTH_KM: f64 = 1.5;

pub struct PhobosShader {
    albedo_map: &'static [u8],    // 1024 * 512
    elevation_map: &'static [u8], // 1024 * 512
    width: usize,
    height: usize,
}

impl PhobosShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/mars_moons_data.bin");
        assert_eq!(&bytes[0..8], b"ASTRO_MN", "Invalid mars_moons_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;

        let phobos_albedo = &bytes[16..16 + pixels];
        let phobos_elevation = &bytes[16 + pixels..16 + 2 * pixels];

        Self {
            albedo_map: phobos_albedo,
            elevation_map: phobos_elevation,
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

    pub fn stickney_elevation(&self, lat_rad: f64, lon_rad: f64) -> f64 {
        let cos_d = (lat_rad.sin() * STICKNEY_LAT_RAD.sin()
            + lat_rad.cos() * STICKNEY_LAT_RAD.cos() * (lon_rad - STICKNEY_LON_RAD).cos())
            .clamp(-1.0, 1.0);
        let dist = cos_d.acos();
        let rho = dist / STICKNEY_RADIUS_RAD;

        let bowl = if rho < 1.0 {
            -STICKNEY_DEPTH_KM * (1.0 - rho * rho)
        } else {
            0.0
        };

        let rim = 0.35 * (-((rho - 1.0) / 0.18).powi(2)).exp();
        bowl + rim
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

        // Calibrate raw USGS photomosaic brightness to geometric albedo ~0.071
        let lum = (raw / 95.0) * BASE_ALBEDO;

        // Stickney blue unit: subtle bluish ejecta around Stickney crater (-50° lon, 0° lat)
        let cos_s = (lat_rad.cos() * (lon_rad - STICKNEY_LON_RAD).cos()).clamp(-1.0, 1.0);
        let dist_s = cos_s.acos();
        let stickney_mask = (-((dist_s - 0.70) / 0.35).powi(2)).exp();

        let r = lum * (1.02 - 0.08 * stickney_mask);
        let g = lum * 1.00;
        let b = lum * (0.98 + 0.16 * stickney_mask);

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

        let bump_scale = 0.0035;
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
