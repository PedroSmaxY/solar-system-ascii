//! Ultra-accurate Moon Shader powered by NASA LRO (Lunar Reconnaissance Orbiter) data:
//! - NASA LROC WAC 1024x512 global photographic albedo (Maria, Highlands, Crater Rays)
//! - NASA LOLA laser altimeter global elevation & relief for dynamic terminator shadows
//! - Lommel-Seeliger & Hapke lunar-Lambert photometric model with opposition effect
//! - Synchronous tidal locking alignment (Near Side Sinus Medii at Lon 0°, Lat 0°)

use crate::math::Vec3;
use std::f64::consts::PI;

pub struct MoonShader {
    albedo_data: &'static [u8],    // 1024 * 512
    elevation_data: &'static [u8], // 1024 * 512
    width: usize,
    height: usize,
}

impl MoonShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/moon_data.bin");
        assert_eq!(&bytes[0..8], b"ASTRO_M1", "Invalid moon_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;

        let albedo_data = &bytes[16..16 + pixels];
        let elevation_data = &bytes[16 + pixels..16 + 2 * pixels];

        Self {
            albedo_data,
            elevation_data,
            width,
            height,
        }
    }

    /// Sample intrinsic photographic albedo [0.0 .. 255.0] with bilinear filtering.
    #[inline(always)]
    pub fn sample_albedo(&self, lat_deg: f64, lon_deg: f64) -> f64 {
        let u = ((lon_deg + 180.0) / 360.0).rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v = ((90.0 - lat_deg) / 180.0).clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u.floor() as usize;
        let y0 = v.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = u - u.floor();
        let fy = v - v.floor();

        let p00 = self.albedo_data[y0 * self.width + x0] as f64;
        let p10 = self.albedo_data[y0 * self.width + x1] as f64;
        let p01 = self.albedo_data[y1 * self.width + x0] as f64;
        let p11 = self.albedo_data[y1 * self.width + x1] as f64;

        let top = p00 * (1.0 - fx) + p10 * fx;
        let bot = p01 * (1.0 - fx) + p11 * fx;
        top * (1.0 - fy) + bot * fy
    }

    /// Sample topographic elevation [0.0 .. 255.0] with bilinear filtering.
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

    /// Shade a surface point on the Moon using NASA photographic albedo, LOLA relief,
    /// and Lommel-Seeliger / Hapke photometric reflectance.
    pub fn shade(&self, local_hit: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        let lat_rad = local_hit.y.clamp(-1.0, 1.0).asin();
        let lat_deg = lat_rad * 180.0 / PI;
        let lon_rad = local_hit.z.atan2(local_hit.x);
        let lon_deg = lon_rad * 180.0 / PI;

        // Intrinsic NASA photographic albedo
        let albedo = self.sample_albedo(lat_deg, lon_deg);

        // Compute local surface normal perturbation via LOLA elevation gradient
        let d_lon = 360.0 / self.width as f64;
        let d_lat = 180.0 / self.height as f64;

        let h_east = self.sample_elevation(lat_deg, lon_deg + d_lon);
        let h_west = self.sample_elevation(lat_deg, lon_deg - d_lon);
        let h_north = self.sample_elevation(lat_deg + d_lat, lon_deg);
        let h_south = self.sample_elevation(lat_deg - d_lat, lon_deg);

        let dh_dx = (h_east - h_west) * 0.5;
        let dh_dy = (h_north - h_south) * 0.5;

        // Tangent & bitangent on unit sphere
        let cos_lat = lat_rad.cos().max(0.02);
        let tangent = Vec3::new(-lon_rad.sin(), 0.0, lon_rad.cos());
        let bitangent = Vec3::new(-lat_rad.sin() * lon_rad.cos(), cos_lat, -lat_rad.sin() * lon_rad.sin());

        // Bump scale calibrated for crisp, dramatic crater relief at terminator
        let bump_scale = 0.008;
        let perturbed_normal = (local_hit + (tangent * (dh_dx / cos_lat) + bitangent * dh_dy) * bump_scale).normalize();

        // Photometric incidence and emission angles
        let mu_0 = perturbed_normal.dot(local_sun);
        let mu = perturbed_normal.dot(local_view).max(0.05);

        // Phase angle alpha between sun and view direction
        let cos_alpha = local_sun.dot(local_view).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();

        // Lunar Opposition Surge (retroreflective brightening when alpha ~ 0)
        let half_alpha = (alpha * 0.5).tan();
        let opposition_surge = 1.0 + 0.30 / (1.0 + 8.0 * half_alpha);

        // Lommel-Seeliger & Lunar-Lambert reflectance
        let l_factor = 0.65; // Lunar regolith scattering constant
        let lommel_seeliger = if mu_0 > 0.0 {
            2.0 * l_factor * (mu_0 / (mu + mu_0)) + (1.0 - l_factor) * mu_0
        } else {
            0.0
        };

        // Smooth lunar terminator transition
        let terminator = ((mu_0 + 0.08) / 0.16).clamp(0.0, 1.0);
        let day_light = lommel_seeliger * opposition_surge * terminator;
        let night_ambient = 0.12; // Earthshine and cosmic starlight baseline

        let final_lum = albedo * (day_light * 0.90 + night_ambient);

        // Neutral achromatic lunar silver/gray tone
        let r = final_lum.clamp(0.0, 255.0) as u8;
        let g = (final_lum * 0.98).clamp(0.0, 255.0) as u8;
        let b = (final_lum * 0.95).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}
