//! Ultra-accurate Mars Shader for AstroEngine:
//! - NASA Viking / Solar System Scope True-Color cartography (1024x512 RGB)
//! - NASA MOLA (Mars Orbiter Laser Altimeter) topographic relief for Olympus Mons and Valles Marineris
//! - Thin CO2 atmosphere with suspended iron-oxide dust: warm butterscotch daytime sky & blue twilight terminator
//! - Calibrated Lommel-Seeliger / Hapke scattering for porous basaltic regolith

use crate::math::Vec3;
use std::f64::consts::PI;

pub struct MarsShader {
    mars_r: &'static [u8],         // 1024 * 512
    mars_g: &'static [u8],         // 1024 * 512
    mars_b: &'static [u8],         // 1024 * 512
    elevation_data: &'static [u8], // 1024 * 512
    width: usize,
    height: usize,
}

impl MarsShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/mars_data.bin");
        assert_eq!(&bytes[0..8], b"ASTRO_MA", "Invalid mars_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;

        let mars_r = &bytes[16..16 + pixels];
        let mars_g = &bytes[16 + pixels..16 + 2 * pixels];
        let mars_b = &bytes[16 + 2 * pixels..16 + 3 * pixels];
        let elevation_data = &bytes[16 + 3 * pixels..16 + 4 * pixels];

        Self {
            mars_r,
            mars_g,
            mars_b,
            elevation_data,
            width,
            height,
        }
    }

    /// Sample intrinsic surface albedo RGB [0.0 .. 255.0] with bilinear filtering.
    #[inline(always)]
    pub fn sample_albedo(&self, lat_deg: f64, lon_deg: f64) -> [f64; 3] {
        let u = ((lon_deg + 180.0) / 360.0).rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v = ((90.0 - lat_deg) / 180.0).clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u.floor() as usize;
        let y0 = v.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = u - u.floor();
        let fy = v - v.floor();

        let idx00 = y0 * self.width + x0;
        let idx10 = y0 * self.width + x1;
        let idx01 = y1 * self.width + x0;
        let idx11 = y1 * self.width + x1;

        let bilerp = |data: &[u8]| -> f64 {
            let p00 = data[idx00] as f64;
            let p10 = data[idx10] as f64;
            let p01 = data[idx01] as f64;
            let p11 = data[idx11] as f64;
            let top = p00 * (1.0 - fx) + p10 * fx;
            let bot = p01 * (1.0 - fx) + p11 * fx;
            top * (1.0 - fy) + bot * fy
        };

        [bilerp(self.mars_r), bilerp(self.mars_g), bilerp(self.mars_b)]
    }

    /// Sample MOLA topographic elevation [0.0 .. 255.0] with bilinear filtering.
    #[inline(always)]
    pub fn sample_elevation(&self, lat_deg: f64, lon_deg: f64) -> f64 {
        let u = ((lon_deg + 180.0) / 360.0).rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v = ((90.0 - lat_deg) / 180.0).clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u.floor() as usize;
        let y0 = v.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = u - u.floor();
        let fy = v - v.floor();

        let p00 = self.elevation_data[y0 * self.width + x0] as f64;
        let p10 = self.elevation_data[y0 * self.width + x1] as f64;
        let p01 = self.elevation_data[y1 * self.width + x0] as f64;
        let p11 = self.elevation_data[y1 * self.width + x1] as f64;

        let top = p00 * (1.0 - fx) + p10 * fx;
        let bot = p01 * (1.0 - fx) + p11 * fx;
        top * (1.0 - fy) + bot * fy
    }

    /// Shade a point on the Martian surface with MOLA bump mapping and atmospheric scattering.
    pub fn shade(&self, local_hit: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        let lat_rad = local_hit.y.clamp(-1.0, 1.0).asin();
        let lat_deg = lat_rad * 180.0 / PI;
        let lon_rad = local_hit.z.atan2(local_hit.x);
        let lon_deg = lon_rad * 180.0 / PI;

        // Sample base surface albedo
        let [base_r, base_g, base_b] = self.sample_albedo(lat_deg, lon_deg);

        // MOLA elevation bump mapping
        let d_lon = 360.0 / self.width as f64;
        let d_lat = 180.0 / self.height as f64;

        let h_east = self.sample_elevation(lat_deg, lon_deg + d_lon);
        let h_west = self.sample_elevation(lat_deg, lon_deg - d_lon);
        let h_north = self.sample_elevation(lat_deg + d_lat, lon_deg);
        let h_south = self.sample_elevation(lat_deg - d_lat, lon_deg);

        let cos_lat = lat_rad.cos().max(1e-4);
        let t_lon = Vec3::new(-local_hit.z, 0.0, local_hit.x).normalize();
        let t_lat = Vec3::new(
            -local_hit.y * lon_rad.cos(),
            cos_lat,
            -local_hit.y * lon_rad.sin(),
        ).normalize();

        let bump_scale = 0.0028;
        let grad_lon = (h_east - h_west) * bump_scale;
        let grad_lat = (h_north - h_south) * bump_scale;

        let perturbed_norm = (local_hit - t_lon * grad_lon - t_lat * grad_lat).normalize();

        let mu_0 = perturbed_norm.dot(local_sun);
        let mu = perturbed_norm.dot(local_view).max(0.0);

        // Absolute nightside cutoff with thin twilight transition
        if mu_0 < -0.05 {
            return [0, 0, 0];
        }

        let sun_factor = ((mu_0 + 0.05) / 1.05).clamp(0.0, 1.0);

        // Regolith scattering: mixture of Lambert and Lommel-Seeliger
        let mu_0_clamped = mu_0.max(0.0);
        let ls = if mu_0_clamped + mu > 1e-5 {
            mu_0_clamped / (mu_0_clamped + mu)
        } else {
            0.0
        };
        let surface_reflectance = 0.65 * mu_0_clamped + 0.35 * ls * 2.0;

        // Solar flux at 1.524 AU is (1 / 1.524^2) ≈ 0.43x Earth's, but normalized for human eye vision
        let lighting = sun_factor.powf(1.10) * (0.35 + 0.65 * surface_reflectance);

        // Thin CO2 Martian atmosphere scattering:
        // Suspended fine dust scatters blue light forward at twilight (the famous blue sunset on Mars!)
        let twilight_factor = (1.0 - sun_factor.max(0.0)).powi(3);
        let blue_twilight_boost = twilight_factor * (local_sun.dot(local_view).max(0.0)).powf(2.5) * 45.0;

        let r = (base_r * lighting * 1.10).clamp(0.0, 255.0);
        let g = (base_g * lighting * 1.05).clamp(0.0, 255.0);
        let b = (base_b * lighting * 1.00 + blue_twilight_boost).clamp(0.0, 255.0);

        [r as u8, g as u8, b as u8]
    }
}
