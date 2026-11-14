//! Ultra-detailed Earth Shader:
//! - Triple-layer cartography (NASA Blue Marble 4K satellite albedo + Natural Earth hydrology + NASA Black Marble night lights)
//! - Visible global rivers (Amazon, Paraná, Nile, Congo, Mississippi) and lakes (Great Lakes, Victoria, Titicaca)
//! - Real NASA satellite photographic colors across all continents (deserts, forests, savannas, mountains, glaciers)
//! - 100% unified physical lighting model: seamless twilight and smooth day-to-night gradient
//! - Ocean specular sun glint + gentle inland freshwater shimmer (25%)
//! - Global fluid dynamic cloud layer (Lagrangian cyclical advection & Coriolis drift)
//! - NASA Black Marble night lights web with dusk ignition and HDR Planckian color ramp
//! - Realistic Geomagnetic Auroras: tilted magnetic dipole (11.5°), episodic solar storms (~6 days), emerald green & violet curtains

use crate::math::Vec3;
use crate::render::fluid::FluidEngine;
use std::f64::consts::PI;

pub struct EarthShader {
    albedo_data: &'static [u8], // Layer 1: 1440 * 720 * 3
    mask_data: &'static [u8],   // Layer 2: 1440 * 720
    night_data: &'static [u8],  // Layer 3: 1440 * 720
    width: usize,
    height: usize,
}

// Magnetic dipole coordinates:
// North Magnetic Dipole: 80.5° N, -72.5° W (Northern Canada/Greenland)
const MAG_POLE_NORTH_LAT: f64 = 80.5 * PI / 180.0;
const MAG_POLE_NORTH_LON: f64 = -72.5 * PI / 180.0;
// South Magnetic Dipole: -64.5° S, 137.5° E (Wilkes Land, Antarctica)
const MAG_POLE_SOUTH_LAT: f64 = -64.5 * PI / 180.0;
const MAG_POLE_SOUTH_LON: f64 = 137.5 * PI / 180.0;

impl EarthShader {
    pub fn new() -> Self {
        let bytes = include_bytes!("../../assets/earth_data.bin");
        // Header: ASTRO_E3 (8 bytes) + W (4 bytes) + H (4 bytes)
        assert_eq!(&bytes[0..8], b"ASTRO_E3", "Invalid earth_data.bin header format");
        let width = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let height = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let pixels = width * height;
        let albedo_size = pixels * 3;

        let albedo_data = &bytes[16..16 + albedo_size];
        let mask_data = &bytes[16 + albedo_size..16 + albedo_size + pixels];
        let night_data = &bytes[16 + albedo_size + pixels..16 + albedo_size + 2 * pixels];

        Self {
            albedo_data,
            mask_data,
            night_data,
            width,
            height,
        }
    }

    /// Sample base intrinsic albedo [R, G, B] from NASA Blue Marble satellite data with bilinear filtering.
    pub fn sample_albedo(&self, lat_deg: f64, lon_deg: f64) -> [f64; 3] {
        let u = ((lon_deg + 180.0) / 360.0).rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v = ((90.0 - lat_deg) / 180.0).clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u.floor() as usize;
        let y0 = v.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = u - u.floor();
        let fy = v - v.floor();

        let idx00 = (y0 * self.width + x0) * 3;
        let idx10 = (y0 * self.width + x1) * 3;
        let idx01 = (y1 * self.width + x0) * 3;
        let idx11 = (y1 * self.width + x1) * 3;

        let mut rgb = [0.0; 3];
        for c in 0..3 {
            let p00 = self.albedo_data[idx00 + c] as f64;
            let p10 = self.albedo_data[idx10 + c] as f64;
            let p01 = self.albedo_data[idx01 + c] as f64;
            let p11 = self.albedo_data[idx11 + c] as f64;

            let top = p00 * (1.0 - fx) + p10 * fx;
            let bot = p01 * (1.0 - fx) + p11 * fx;
            rgb[c] = top * (1.0 - fy) + bot * fy;
        }
        rgb
    }

    /// Sample terrain / hydrology classification byte:
    /// 20: Deep ocean, 38: Continental shelf, 50: Inland freshwater river/lake, 145: Land, 250: Ice sheet.
    pub fn sample_mask(&self, lat_deg: f64, lon_deg: f64) -> u8 {
        let u = ((lon_deg + 180.0) / 360.0).rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v = ((90.0 - lat_deg) / 180.0).clamp(0.0, 1.0) * (self.height as f64 - 1.0);
        let x = u.round() as usize;
        let y = v.round() as usize;
        self.mask_data[y.min(self.height - 1) * self.width + x.min(self.width - 1)]
    }

    /// Sample night lights intensity (0..1.0) with bilinear filtering.
    pub fn sample_night_lights(&self, lat_deg: f64, lon_deg: f64) -> f64 {
        let u = ((lon_deg + 180.0) / 360.0).rem_euclid(1.0) * (self.width as f64 - 1.0);
        let v = ((90.0 - lat_deg) / 180.0).clamp(0.0, 1.0) * (self.height as f64 - 1.0);

        let x0 = u.floor() as usize;
        let y0 = v.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);

        let fx = u - u.floor();
        let fy = v - v.floor();

        let p00 = self.night_data[y0 * self.width + x0] as f64;
        let p10 = self.night_data[y0 * self.width + x1] as f64;
        let p01 = self.night_data[y1 * self.width + x0] as f64;
        let p11 = self.night_data[y1 * self.width + x1] as f64;

        let top = p00 * (1.0 - fx) + p10 * fx;
        let bot = p01 * (1.0 - fx) + p11 * fx;
        (top * (1.0 - fy) + bot * fy) / 255.0
    }

    /// Calculate shaded surface color for point on Earth.
    pub fn shade(
        &self,
        local_hit: Vec3, // Point on unit sphere in Earth's local coordinate frame
        world_norm: Vec3,
        sun_dir: Vec3,
        view_dir: Vec3,
        tick: u64,
        elapsed: f64,
    ) -> [u8; 3] {
        let lat_rad = local_hit.y.clamp(-1.0, 1.0).asin();
        let lat_deg = lat_rad * 180.0 / PI;
        let lon_rad = local_hit.z.atan2(local_hit.x);
        let lon_deg = lon_rad * 180.0 / PI;

        // Terrain & hydrology classification
        let mask = self.sample_mask(lat_deg, lon_deg);
        let is_inland_water = mask == 50;
        let is_water = mask <= 50;

        // Intrinsic surface albedo sampled directly from NASA Blue Marble 4K satellite photography
        let albedo = self.sample_albedo(lat_deg, lon_deg);

        // ========================================================
        // 1. PHYSICAL DAY-TO-NIGHT TERMINATOR LIGHTING
        // Seamless smoothstep transition without artificial orange ground halo
        // ========================================================
        let n_dot_l = world_norm.dot(sun_dir);

        // Smooth continuous transition from night (-0.18) to day (+0.22)
        let twilight_t = ((n_dot_l + 0.18) / 0.40).clamp(0.0, 1.0);
        let day_weight = twilight_t * twilight_t * (3.0 - 2.0 * twilight_t); // Smoothstep curve
        let night_weight = 1.0 - day_weight;

        // Direct sunlight diffuse term
        let direct_sun = n_dot_l.max(0.0).powf(0.92);

        // Surface lighting: seamlessly blends direct sunlight into natural starlight/airglow
        let night_ambient_scale = 0.22; // 22% natural starlight/airglow illumination
        let surface_light = day_weight * (direct_sun * 0.92 + 0.08) + night_weight * night_ambient_scale;

        let mut ground_col = [
            albedo[0] * surface_light,
            albedo[1] * surface_light,
            albedo[2] * surface_light,
        ];

        // Ocean & Inland Water Specular Highlight (Sun Glint):
        // Open oceans receive full specular sun glint (240.0).
        // Inland rivers and lakes receive subtle, gentle freshwater glint (60.0 = 25%) to prevent white clutter.
        if is_water && day_weight > 0.01 {
            let halfway = (sun_dir + view_dir).normalize();
            let n_dot_h = world_norm.dot(halfway).max(0.0);
            let specular = n_dot_h.powf(32.0) * day_weight;
            let spec_intensity = if is_inland_water { 60.0 } else { 240.0 };
            ground_col[0] += specular * spec_intensity;
            ground_col[1] += specular * (spec_intensity * 0.95);
            ground_col[2] += specular * (spec_intensity * 0.83);
        }

        // ========================================================
        // 2. FLUID DYNAMIC CLOUD LAYER (Powered by FluidEngine)
        // Lagrangian cyclical advection & Coriolis drift
        // ========================================================
        let (cloud_density, cloud_alpha) = FluidEngine::sample_earth_clouds(local_hit, elapsed);

        // Soft cloud shadow cast onto terrain under sunlight
        if direct_sun > 0.05 && cloud_density > 0.05 {
            let shadow_hit = Vec3::new(
                local_hit.x - sun_dir.x * 0.025,
                local_hit.y - sun_dir.y * 0.025,
                local_hit.z - sun_dir.z * 0.025,
            ).normalize();
            let (s_dens, _) = FluidEngine::sample_earth_clouds(shadow_hit, elapsed);
            let s_occ = (s_dens * 0.32 * direct_sun).clamp(0.0, 0.35);
            ground_col[0] *= 1.0 - s_occ;
            ground_col[1] *= 1.0 - s_occ;
            ground_col[2] *= 1.0 - s_occ;
        }

        // Dynamic cloud illumination (Mie multiple scattering):
        // Water droplets and ice crystals undergo intense forward and multiple internal scattering,
        // maintaining pure, radiant white albedo across illuminated angles (eliminating muddy gray smoke).
        let mie_scatter = direct_sun.powf(0.55) * 0.72 + 0.28;
        let cloud_day = [255.0 * mie_scatter, 255.0 * mie_scatter, 255.0 * mie_scatter];
        let cloud_night = [50.0, 68.0, 92.0];

        let cloud_col = [
            cloud_night[0] * night_weight + cloud_day[0] * day_weight,
            cloud_night[1] * night_weight + cloud_day[1] * day_weight,
            cloud_night[2] * night_weight + cloud_day[2] * day_weight,
        ];

        // Composite clouds over surface
        let mut final_col = [
            ground_col[0] * (1.0 - cloud_alpha) + cloud_col[0] * cloud_alpha,
            ground_col[1] * (1.0 - cloud_alpha) + cloud_col[1] * cloud_alpha,
            ground_col[2] * (1.0 - cloud_alpha) + cloud_col[2] * cloud_alpha,
        ];

        // ========================================================
        // 3. HUMAN CIVILIZATION NIGHT LIGHTS (NASA Black Marble Grid)
        // Photometric HDR Planckian color ramp & dusk ignition curve
        // Core reaches pure white-hot [255, 255, 245], suburbs gold, rural amber
        // ========================================================
        let dusk_t = ((0.08 - n_dot_l) / 0.16).clamp(0.0, 1.0);
        let city_activation = dusk_t * dusk_t * (3.0 - 2.0 * dusk_t);

        if city_activation > 0.01 && mask >= 38 {
            let night_density = self.sample_night_lights(lat_deg, lon_deg);
            if night_density > 0.01 {
                let tick_f = tick as f64;
                let cell_hash = ((lat_deg * 37.0).floor() + (lon_deg * 53.0).floor()).abs();
                let pulse = 0.92 + 0.08 * ((tick_f * 0.22 + cell_hash).sin());

                let s = (night_density * city_activation * pulse).clamp(0.0, 1.0);
                let cloud_filter = 1.0 - cloud_density * 0.40;

                // Photometric HDR Planckian color ramp
                let (base_r, base_g, base_b) = if s > 0.65 {
                    let t = (s - 0.65) / 0.35;
                    (
                        255.0,
                        210.0 + 45.0 * t,
                        85.0 + 160.0 * t,
                    )
                } else if s > 0.20 {
                    let t = (s - 0.20) / 0.45;
                    (
                        215.0 + 40.0 * t,
                        135.0 + 75.0 * t,
                        35.0 + 50.0 * t,
                    )
                } else {
                    let t = (s / 0.20).clamp(0.0, 1.0);
                    (
                        215.0 * t,
                        135.0 * t,
                        35.0 * t,
                    )
                };

                let light_r = base_r * cloud_filter;
                let light_g = base_g * cloud_filter;
                let light_b = base_b * cloud_filter;

                final_col[0] = final_col[0].max(light_r);
                final_col[1] = final_col[1].max(light_g);
                final_col[2] = final_col[2].max(light_b);
            }
        }

        // ========================================================
        // 4. REALISTIC GEOMAGNETIC AURORAS
        // - Centered on real tilted magnetic dipole (North: 80.5° N, -72.5° W; South: -64.5° S, 137.5° E)
        // - Magnetic azimuth (psi) coordinate system: multi-arc serpentine drapery folding along the ring
        // - Smooth continuous twilight extinction (day_fade) eliminating any razor-blade cutoff line
        // - Episodic Solar Storm Recurrence: calm baseline for days, erupting every ~6.2 days for 6 hours
        // ========================================================
        let twilight_fade = ((0.15 - n_dot_l) / 0.30).clamp(0.0, 1.0);
        let day_fade = twilight_fade * twilight_fade * (3.0 - 2.0 * twilight_fade); // C^1 continuous smoothstep

        if day_fade > 0.01 {
            let (pole_lat, pole_lon) = if lat_rad >= 0.0 {
                (MAG_POLE_NORTH_LAT, MAG_POLE_NORTH_LON)
            } else {
                (MAG_POLE_SOUTH_LAT, MAG_POLE_SOUTH_LON)
            };

            // Spherical trigonometry for geomagnetic latitude lambda_m
            let delta_lon = lon_rad - pole_lon;
            let sin_mag_lat = lat_rad.sin() * pole_lat.sin()
                + lat_rad.cos() * pole_lat.cos() * delta_lon.cos();
            let mag_lat_deg = sin_mag_lat.clamp(-1.0, 1.0).asin().abs() * 180.0 / PI;

            // Magnetic azimuth psi around the magnetic pole [-PI..PI]
            let y_psi = lat_rad.cos() * delta_lon.sin();
            let x_psi = pole_lat.cos() * lat_rad.sin() - pole_lat.sin() * lat_rad.cos() * delta_lon.cos();
            let psi = y_psi.atan2(x_psi);

            // Solar Storm Cycle:
            // ~6.2 days period = 535,680 seconds.
            // Storm window lasts ~28,800 seconds (~8 hours = 5.3% of cycle).
            let storm_period = 535_680.0;
            let storm_phase = (elapsed / storm_period).fract();
            let is_storm_window = storm_phase < 0.054;

            let solar_activity = if is_storm_window {
                let storm_t = (storm_phase / 0.054) * PI;
                0.05 + 0.95 * storm_t.sin().powf(1.4)
            } else {
                0.03 // Quiet calm baseline
            };

            if solar_activity > 0.08 {
                let t_aurora = elapsed * 0.35;
                // Serpentine undulating drapery in magnetic azimuth psi
                let wave = (psi * 5.0 + t_aurora * 0.5).sin() * 1.6 + (psi * 11.0 - t_aurora * 0.8).cos() * 0.9;
                let peak_lat = (72.0 - 10.0 * solar_activity) + wave;

                // Multi-arc concentric auroral curtains (Gaussian profiles)
                let d1 = (mag_lat_deg - peak_lat).abs();
                let d2 = (mag_lat_deg - (peak_lat - 3.2)).abs();
                let arc1 = (-d1 * d1 / (2.0 * 1.6 * 1.6)).exp();
                let arc2 = (-d2 * d2 / (2.0 * 1.3 * 1.3)).exp() * 0.72;

                // Vertical magnetic field ray striations along field lines
                let field_rays = ((psi * 36.0 - t_aurora * 0.4).sin() * 0.5 + 0.5).powf(1.5);
                let drapery = (arc1 + arc2) * (0.62 + 0.38 * field_rays);

                let aurora_energy = (solar_activity * drapery * day_fade).clamp(0.0, 1.0);

                if aurora_energy > 0.015 {
                    // Spectral emission:
                    // Atomic oxygen 557.7 nm emerald green [75, 255, 135]
                    // Nitrogen / high-altitude oxygen 630 nm violet [185, 75, 235]
                    let violet_fringe = (arc1 * 0.90).powi(3);
                    let aur_r = (75.0 * (1.0 - violet_fringe) + 185.0 * violet_fringe) * aurora_energy * 0.88;
                    let aur_g = (255.0 * (1.0 - violet_fringe) + 75.0 * violet_fringe) * aurora_energy * 0.88;
                    let aur_b = (135.0 * (1.0 - violet_fringe) + 235.0 * violet_fringe) * aurora_energy * 0.88;

                    final_col[0] = (final_col[0] + aur_r).min(255.0);
                    final_col[1] = (final_col[1] + aur_g).min(255.0);
                    final_col[2] = (final_col[2] + aur_b).min(255.0);
                }
            }
        }

        [
            final_col[0].clamp(0.0, 255.0) as u8,
            final_col[1].clamp(0.0, 255.0) as u8,
            final_col[2].clamp(0.0, 255.0) as u8,
        ]
    }
}
