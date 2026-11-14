//! Ultra-detailed Venus Shader with Dual-Layer Mode:
//! 1. Optical Atmospheric Mode (Default):
//!    - Hyper-realistic sulfuric acid (H2SO4) cloud deck with 4-day atmospheric super-rotation
//!    - Planetary UV chevron 'Y-feature' equatorial wave pattern & polar vortices
//!    - High planetary albedo (~0.77) with creamy pale-gold Planckian coloration
//!    - Dense Mie & Rayleigh atmospheric twilight scattering
//! 2. Magellan Radar SAR Mode (Toggled via [V]):
//!    - NASA Magellan Synthetic Aperture Radar global reflectivity (1024x512)
//!    - Topographic relief displacement for Maxwell Montes, Aphrodite Terra, and shield volcanoes
//!    - Volcanic false-color bronze/golden basalt coloration

use crate::math::Vec3;
use std::f64::consts::PI;

pub struct VenusShader {
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

        let radar_data = &bytes[16..16 + pixels];
        let elevation_data = &bytes[16 + pixels..16 + 2 * pixels];

        Self {
            radar_data,
            elevation_data,
            width,
            height,
            radar_mode: false,
        }
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
            // 4-day zonal super-rotation (moving retrograde westward: negative lon)
            let super_rot = -(elapsed * 0.035);
            let rot_lon = (lon_rad + super_rot).rem_euclid(2.0 * PI) - PI;

            // Zonal wind shear generating the planetary UV chevron / horizontal 'Y-feature'
            // Equatorial winds are fastest (~100 m/s); mid-latitudes bend backwards
            let lat_bend = (lat_rad.abs() * 1.35).powf(1.6) * 0.45;
            let chevron_lon = rot_lon + lat_bend;

            // UV absorbing sulfur aerosol wave bands
            let y_feature = (chevron_lon * 2.0).cos() * (lat_rad * 3.5).cos().max(0.0).powi(2);
            let sub_bands = (lat_rad * 14.0 + (chevron_lon * 4.0).sin() * 0.3).sin() * 0.5 + 0.5;
            let micro_plumes = (lat_rad * 28.0 - chevron_lon * 6.0).cos() * 0.5 + 0.5;

            // Polar dipoles (swirling double-vortices at |lat| > 65°)
            let polar_mask = ((lat_rad.abs() - (65.0 * PI / 180.0)).max(0.0) / (25.0 * PI / 180.0)).powi(2);
            let polar_vortex = ((chevron_lon * 2.0 + elapsed * 0.05).sin() * 0.5 + 0.5) * polar_mask;

            // Atmospheric diffuse reflectance with limb darkening
            let mu_0 = local_hit.dot(local_sun);
            let mu = local_hit.dot(local_view).max(0.0);

            // Deep twilight penetration through cloud deck (smooth terminator gradient)
            let sun_factor = ((mu_0 + 0.12) / 1.12).clamp(0.0, 1.0);
            if sun_factor <= 0.0 {
                return [0, 0, 0];
            }

            // Limb darkening for thick scattering atmosphere (Chandrasekhar H-function approx)
            let limb_darkening = 0.40 + 0.60 * mu.sqrt();
            let lighting = sun_factor.powf(1.15) * limb_darkening;

            // Base sulfuric acid cloud albedo (~0.77 high optical thickness)
            // Irradiance at 0.723 AU is 1.91x Earth's
            let cloud_texture = 0.90 + 0.06 * y_feature + 0.03 * sub_bands + 0.02 * micro_plumes - 0.05 * polar_vortex;
            let total_lum = lighting * cloud_texture * 1.25;

            // Creamy yellowish-white Planckian palette:
            // Subsolar point is bright pale cream [250, 240, 215],
            // fading gently into golden amber [215, 185, 130] near terminator
            let r = (total_lum * 252.0).clamp(0.0, 255.0);
            let g = (total_lum * (240.0 - 45.0 * (1.0 - sun_factor))).clamp(0.0, 255.0);
            let b = (total_lum * (210.0 - 110.0 * (1.0 - sun_factor))).clamp(0.0, 255.0);

            [r as u8, g as u8, b as u8]
        }
    }
}
