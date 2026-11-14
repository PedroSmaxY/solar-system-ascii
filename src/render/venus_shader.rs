//! Ultra-detailed Venus Shader with Dual-Layer Mode:
//! 1. Optical Atmospheric Mode (Default):
//!    - NASA Pioneer Venus & Akatsuki ultraviolet & visible cloud deck (1024x512)
//!    - Zonal atmospheric super-rotation in 4 days (~100 m/s westward winds)
//!    - Equatorial Y-chevron UV waves, horizontal jet bands, and bright polar hoods
//!    - Deep atmospheric twilight scattering & limb darkening
//! 2. Magellan Radar SAR Mode (Toggled via [V]):
//!    - NASA Magellan Synthetic Aperture Radar global reflectivity (1024x512)
//!    - Topographic relief displacement for Maxwell Montes, Aphrodite Terra, and shield volcanoes
//!    - Volcanic false-color bronze/golden basalt coloration

use crate::math::Vec3;
use std::f64::consts::PI;

pub struct VenusShader {
    atm_r: &'static [u8],          // 1024 * 512
    atm_g: &'static [u8],          // 1024 * 512
    atm_b: &'static [u8],          // 1024 * 512
    radar_data: &'static [u8],     // 1024 * 512
    elevation_data: &'static [u8], // 1024 * 512
    width: usize,
    height: usize,
    pub radar_mode: bool,
}

impl VenusShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/venus_data.bin");
        assert_eq!(&bytes[0..8], b"ASTRO_VN", "Invalid venus_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;

        let atm_r = &bytes[16..16 + pixels];
        let atm_g = &bytes[16 + pixels..16 + 2 * pixels];
        let atm_b = &bytes[16 + 2 * pixels..16 + 3 * pixels];
        let radar_data = &bytes[16 + 3 * pixels..16 + 4 * pixels];
        let elevation_data = &bytes[16 + 4 * pixels..16 + 5 * pixels];

        Self {
            atm_r,
            atm_g,
            atm_b,
            radar_data,
            elevation_data,
            width,
            height,
            radar_mode: false,
        }
    }

    /// Sample intrinsic optical atmospheric cloud deck RGB [0.0 .. 255.0] with bilinear filtering.
    #[inline(always)]
    pub fn sample_atmosphere(&self, lat_deg: f64, lon_deg: f64) -> [f64; 3] {
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

        [bilerp(self.atm_r), bilerp(self.atm_g), bilerp(self.atm_b)]
    }

    /// Sample intrinsic Magellan SAR radar reflectivity [0.0 .. 255.0] with bilinear filtering.
    #[inline(always)]
    pub fn sample_radar(&self, lat_deg: f64, lon_deg: f64) -> f64 {
        let u = ((lon_deg + 180.0) / 360.0).rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v = ((90.0 - lat_deg) / 180.0).clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u.floor() as usize;
        let y0 = v.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = u - u.floor();
        let fy = v - v.floor();

        let p00 = self.radar_data[y0 * self.width + x0] as f64;
        let p10 = self.radar_data[y0 * self.width + x1] as f64;
        let p01 = self.radar_data[y1 * self.width + x0] as f64;
        let p11 = self.radar_data[y1 * self.width + x1] as f64;

        let top = p00 * (1.0 - fx) + p10 * fx;
        let bot = p01 * (1.0 - fx) + p11 * fx;
        top * (1.0 - fy) + bot * fy
    }

    /// Sample topographic elevation from Magellan radar altimetry [0.0 .. 255.0] with bilinear filtering.
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

    /// Shade a surface/atmospheric point on Venus.
    pub fn shade(&self, local_hit: Vec3, local_sun: Vec3, local_view: Vec3, elapsed: f64) -> [u8; 3] {
        let lat_rad = local_hit.y.clamp(-1.0, 1.0).asin();
        let lat_deg = lat_rad * 180.0 / PI;
        let lon_rad = local_hit.z.atan2(local_hit.x);
        let lon_deg = lon_rad * 180.0 / PI;

        if self.radar_mode {
            // ==========================================
            // MAGELLAN RADAR SAR TOPOGRAPHY MODE
            // ==========================================
            let radar = self.sample_radar(lat_deg, lon_deg);

            // Compute local surface normal perturbation via Magellan elevation gradient
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

            let bump_scale = 0.0030;
            let grad_lon = (h_east - h_west) * bump_scale;
            let grad_lat = (h_north - h_south) * bump_scale;

            let perturbed_norm = (local_hit - t_lon * grad_lon - t_lat * grad_lat).normalize();

            let mu_0 = perturbed_norm.dot(local_sun).max(0.0);
            let mu = perturbed_norm.dot(local_view).max(0.0);

            if mu_0 <= 0.0 {
                return [0, 0, 0];
            }

            // Radar backscatter reflectance model
            let ls = if mu_0 + mu > 1e-5 { mu_0 / (mu_0 + mu) } else { 0.0 };
            let lambert = mu_0;
            let reflectance = 0.60 * ls * 2.0 + 0.40 * lambert;

            let norm_radar = radar / 255.0;
            let intensity = reflectance * norm_radar * 1.30;

            // Volcanic false-color golden-bronze basaltic palette (NASA Magellan standard)
            let r = (intensity * 255.0 * 1.08).clamp(0.0, 255.0);
            let g = (intensity * 205.0 * 0.98).clamp(0.0, 255.0);
            let b = (intensity * 115.0 * 0.82).clamp(0.0, 255.0);

            [r as u8, g as u8, b as u8]
        } else {
            // ==========================================
            // OPTICAL SUPER-ROTATING ATMOSPHERE MODE
            // ==========================================
            // 4-day zonal super-rotation (westward retrograde wind ~100 m/s)
            // Period: 4 days = 345,600 s. Scaled with simulation elapsed time.
            let super_rot_deg = (elapsed * (360.0 / (4.0 * 86_400.0))).rem_euclid(360.0);
            let sample_lon = (lon_deg - super_rot_deg).rem_euclid(360.0) - 180.0;

            // Sample real NASA Pioneer Venus / Akatsuki ultraviolet & visible cloud deck
            let [raw_r, raw_g, raw_b] = self.sample_atmosphere(lat_deg, sample_lon);

            // Atmospheric diffuse reflectance with limb darkening
            let mu_0 = local_hit.dot(local_sun);
            let mu = local_hit.dot(local_view).max(0.0);

            // Deep twilight penetration through cloud deck (smooth terminator gradient)
            let sun_factor = ((mu_0 + 0.12) / 1.12).clamp(0.0, 1.0);
            if sun_factor <= 0.0 {
                return [0, 0, 0];
            }

            // Limb darkening for thick scattering atmosphere (Chandrasekhar H-function approx)
            let limb_darkening = 0.42 + 0.58 * mu.powf(0.65);
            let lighting = sun_factor.powf(1.10) * limb_darkening;

            // Gentle golden warming near the terminator
            let term_warmth = (1.0 - sun_factor).powi(2);
            let r = (raw_r * lighting * 1.15).clamp(0.0, 255.0);
            let g = (raw_g * lighting * (1.10 - 0.12 * term_warmth)).clamp(0.0, 255.0);
            let b = (raw_b * lighting * (1.05 - 0.25 * term_warmth)).clamp(0.0, 255.0);

            [r as u8, g as u8, b as u8]
        }
    }
}
