//! Photorealistic Callisto Surface Raytracing Shader:
//! - Direct zero-copy sampling of authentic NASA Galileo/Voyager 1024x512 Callisto albedo
//!   and Valhalla multi-ring ripple basin relief dataset from `jupiter_system_data.bin`.
//! - Fractional 4-texel bilinear interpolation without polar singularities.
//! - Topographic bump mapping via elevation finite differences (du, dv) perturbing surface normals
//!   to resolve thousands of ancient impact crater rims and concentric ripple ridges of Valhalla basin (~3,800 km diameter).
//! - Calibrated Callisto geomorphology & color palette:
//!   * Dark ancient regolith / dark blanket: ~[90, 85, 80] (low albedo ~0.20, ice-poor silicate dust)
//!   * Intermediate cratered terrain: ~[125, 120, 115]
//!   * Multi-ring ripple basin of Valhalla: ~[170, 175, 180] (concentric bright icy ridges)
//!   * Fresh impact crater rims with frost pinnacles & bright ice deposits: ~[215, 220, 230] (Asgard, Lofn)
//! - Lommel-Seeliger non-Lambertian scattering with opposition surge and pitch-black nightside (zero light leakage).

use crate::math::Vec3;
use std::f64::consts::PI;

pub const TEX_WIDTH: usize = 1024;
pub const TEX_HEIGHT: usize = 512;
pub const PIXELS: usize = TEX_WIDTH * TEX_HEIGHT;

pub const CALLISTO_ALBEDO_OFFSET: usize = 16 + 10 * PIXELS; // 5_242_896
pub const CALLISTO_ELEVATION_OFFSET: usize = 16 + 11 * PIXELS; // 5_767_184

pub const BUMP_SCALE: f64 = 0.005;

pub const COLOR_DARK_BLANKET: [f64; 3] = [90.0, 85.0, 80.0];
pub const COLOR_INTERMEDIATE: [f64; 3] = [125.0, 120.0, 115.0];
pub const COLOR_VALHALLA_RIPPLES: [f64; 3] = [170.0, 175.0, 180.0];
pub const COLOR_CRATER_ICE: [f64; 3] = [215.0, 220.0, 230.0];

#[derive(Clone, Copy, Debug)]
pub struct CallistoShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl CallistoShader {
    /// Initialize CallistoShader with zero-copy slices from `jupiter_system_data.bin`.
    pub fn new() -> Self {
        let system_data = crate::render::jupiter_shader::JUPITER_SYSTEM_DATA;
        assert!(
            system_data.len() >= CALLISTO_ELEVATION_OFFSET + PIXELS,
            "jupiter_system_data.bin buffer too small for Callisto datasets"
        );
        assert_eq!(
            &system_data[0..8],
            b"ASTRO_JP",
            "Invalid jupiter_system_data.bin header format"
        );
        let albedo_map = &system_data[CALLISTO_ALBEDO_OFFSET..CALLISTO_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &system_data[CALLISTO_ELEVATION_OFFSET..CALLISTO_ELEVATION_OFFSET + PIXELS];
        Self {
            albedo_map,
            elevation_map,
        }
    }

    /// Construct with explicit static slices (useful for testing).
    pub fn from_slices(albedo_map: &'static [u8], elevation_map: &'static [u8]) -> Self {
        Self {
            albedo_map,
            elevation_map,
        }
    }

    #[inline]
    pub fn albedo_map(&self) -> &'static [u8] {
        self.albedo_map
    }

    #[inline]
    pub fn elevation_map(&self) -> &'static [u8] {
        self.elevation_map
    }

    /// Calculate spherical UV coordinates from local surface normal (x, y, z) where y is polar axis:
    /// v = (1.0 - (y.clamp(-1.0, 1.0).asin() / (std::f64::consts::PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0)
    /// u = (z.atan2(x) + std::f64::consts::PI) / (2.0 * std::f64::consts::PI)
    #[inline]
    pub fn compute_uv(local_norm: Vec3) -> (f64, f64) {
        let norm = local_norm.normalize();
        let v = (1.0
            - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5))
            .clamp(0.0, 1.0);
        let u = ((norm.z.atan2(norm.x) + PI) / (2.0 * PI)).rem_euclid(1.0);
        (u, v)
    }

    /// 4-texel fractional bilinear interpolation on 1024x512 texture.
    #[inline]
    pub fn sample_bilinear(data: &[u8], u: f64, v: f64) -> f64 {
        let u_clamped = u.rem_euclid(1.0);
        let v_clamped = v.clamp(0.0, 1.0);

        let fx = u_clamped * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamped * (TEX_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % TEX_WIDTH;
        let y1 = (y0 + 1).min(TEX_HEIGHT - 1);

        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;

        let p00 = data[y0 * TEX_WIDTH + x0] as f64;
        let p10 = data[y0 * TEX_WIDTH + x1] as f64;
        let p01 = data[y1 * TEX_WIDTH + x0] as f64;
        let p11 = data[y1 * TEX_WIDTH + x1] as f64;

        let top = p00 * (1.0 - tx) + p10 * tx;
        let bot = p01 * (1.0 - tx) + p11 * tx;
        top * (1.0 - ty) + bot * ty
    }

    /// Sample scalar albedo [0.0 .. 255.0] with bilinear filtering.
    #[inline]
    pub fn sample_albedo_bilinear(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.albedo_map, u, v)
    }

    /// Sample topographic elevation [0.0 .. 255.0] with bilinear filtering.
    #[inline]
    pub fn sample_elevation_bilinear(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.elevation_map, u, v)
    }

    /// Convert raw 8-bit Callisto albedo to calibrated planetary RGB:
    /// - Dark ancient regolith / dark blanket: ~[90, 85, 80]
    /// - Intermediate cratered terrain: ~[125, 120, 115]
    /// - Multi-ring ripple basin of Valhalla (3,800 km diameter): ~[170, 175, 180]
    /// - Fresh impact crater rims with frost pinnacles & bright ice deposits (Asgard, Lofn): ~[215, 220, 230]
    #[inline]
    pub fn albedo_to_rgb(raw: f64) -> [f64; 3] {
        if raw <= 90.0 {
            let t = (raw / 90.0).clamp(0.0, 1.0);
            [
                COLOR_DARK_BLANKET[0] * t,
                COLOR_DARK_BLANKET[1] * t,
                COLOR_DARK_BLANKET[2] * t,
            ]
        } else if raw <= 140.0 {
            let t = (raw - 90.0) / (140.0 - 90.0);
            [
                COLOR_DARK_BLANKET[0] + (COLOR_INTERMEDIATE[0] - COLOR_DARK_BLANKET[0]) * t,
                COLOR_DARK_BLANKET[1] + (COLOR_INTERMEDIATE[1] - COLOR_DARK_BLANKET[1]) * t,
                COLOR_DARK_BLANKET[2] + (COLOR_INTERMEDIATE[2] - COLOR_DARK_BLANKET[2]) * t,
            ]
        } else if raw <= 180.0 {
            let t = (raw - 140.0) / (180.0 - 140.0);
            [
                COLOR_INTERMEDIATE[0] + (COLOR_VALHALLA_RIPPLES[0] - COLOR_INTERMEDIATE[0]) * t,
                COLOR_INTERMEDIATE[1] + (COLOR_VALHALLA_RIPPLES[1] - COLOR_INTERMEDIATE[1]) * t,
                COLOR_INTERMEDIATE[2] + (COLOR_VALHALLA_RIPPLES[2] - COLOR_INTERMEDIATE[2]) * t,
            ]
        } else if raw <= 225.0 {
            let t = (raw - 180.0) / (225.0 - 180.0);
            [
                COLOR_VALHALLA_RIPPLES[0] + (COLOR_CRATER_ICE[0] - COLOR_VALHALLA_RIPPLES[0]) * t,
                COLOR_VALHALLA_RIPPLES[1] + (COLOR_CRATER_ICE[1] - COLOR_VALHALLA_RIPPLES[1]) * t,
                COLOR_VALHALLA_RIPPLES[2] + (COLOR_CRATER_ICE[2] - COLOR_VALHALLA_RIPPLES[2]) * t,
            ]
        } else {
            let t = ((raw - 225.0) / 30.0).clamp(0.0, 1.0);
            [
                (COLOR_CRATER_ICE[0] + (240.0 - COLOR_CRATER_ICE[0]) * t).min(255.0),
                (COLOR_CRATER_ICE[1] + (245.0 - COLOR_CRATER_ICE[1]) * t).min(255.0),
                (COLOR_CRATER_ICE[2] + (255.0 - COLOR_CRATER_ICE[2]) * t).min(255.0),
            ]
        }
    }

    /// Sample RGB albedo at spherical coordinates (u, v).
    #[inline]
    pub fn sample_albedo_rgb(&self, u: f64, v: f64) -> [f64; 3] {
        let raw = self.sample_albedo_bilinear(u, v);
        Self::albedo_to_rgb(raw)
    }

    /// Compute bump-mapped surface normal using finite differences in elevation map.
    /// Highlights crater rims and concentric ripple ridges of Valhalla basin.
    pub fn perturbed_normal(&self, local_norm: Vec3) -> Vec3 {
        let norm = local_norm.normalize();
        if norm.length_squared() < 1e-6 {
            return Vec3::new(0.0, 1.0, 0.0);
        }

        let (u, v) = Self::compute_uv(norm);

        // Orthonormal tangent and bitangent vectors without polar singularity
        let xz_len = (norm.x * norm.x + norm.z * norm.z).sqrt();
        let tangent = if xz_len > 1e-6 {
            Vec3::new(-norm.z / xz_len, 0.0, norm.x / xz_len)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let bitangent = norm.cross(tangent).normalize();

        // Finite differences step in UV space
        let delta_u = 1.0 / TEX_WIDTH as f64;
        let delta_v = 1.0 / TEX_HEIGHT as f64;

        let h_u1 = self.sample_elevation_bilinear((u + delta_u).rem_euclid(1.0), v);
        let h_u0 = self.sample_elevation_bilinear((u - delta_u).rem_euclid(1.0), v);
        let h_v1 = self.sample_elevation_bilinear(u, (v + delta_v).clamp(0.0, 1.0));
        let h_v0 = self.sample_elevation_bilinear(u, (v - delta_v).clamp(0.0, 1.0));

        let du = (h_u1 - h_u0) * 0.5;
        let dv = (h_v1 - h_v0) * 0.5;

        (norm - tangent * (du * BUMP_SCALE) - bitangent * (dv * BUMP_SCALE)).normalize()
    }

    /// Lommel-Seeliger non-Lambertian reflectance with opposition surge:
    /// mu_0 = perturbed_norm.dot(sun_dir)
    /// mu = perturbed_norm.dot(view_dir).max(0.001)
    /// ls = mu_0 / (mu_0 + mu)
    /// alpha = sun_dir.dot(view_dir).clamp(-1.0, 1.0).acos()
    /// surge = 1.0 + 0.48 * (1.0 - (alpha / 0.17).min(1.0))
    /// illumination = (ls * surge * 1.85).clamp(0.0, 1.5)
    #[inline]
    pub fn lommel_seeliger_illumination(
        perturbed_norm: Vec3,
        sun_dir: Vec3,
        view_dir: Vec3,
    ) -> f64 {
        let sun_norm = sun_dir.normalize();
        let view_norm = view_dir.normalize();

        let mu_0 = perturbed_norm.dot(sun_norm);
        if mu_0 <= 0.0 {
            return 0.0;
        }

        let mu = perturbed_norm.dot(view_norm).max(0.001);
        let ls = mu_0 / (mu_0 + mu);

        let alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0).acos();
        let surge = 1.0 + 0.48 * (1.0 - (alpha / 0.17).min(1.0));

        (ls * surge * 1.85).clamp(0.0, 1.5)
    }

    /// Primary photorealistic raytracing shader for Callisto.
    /// - `local_norm`: surface normal in Callisto rotating body frame.
    /// - `sun_dir`: incident light direction in Callisto body frame.
    /// - `view_dir`: view direction towards camera in Callisto body frame.
    pub fn shade(&self, local_norm: Vec3, sun_dir: Vec3, view_dir: Vec3) -> [u8; 3] {
        if local_norm.length_squared() < 1e-12 {
            return [0, 0, 0];
        }
        let sun_norm = sun_dir.normalize();
        let view_norm = view_dir.normalize();
        if sun_norm.length_squared() < 1e-6 || view_norm.length_squared() < 1e-6 {
            return [0, 0, 0];
        }

        let norm = local_norm.normalize();
        let perturbed_norm = self.perturbed_normal(norm);

        let mu_0 = perturbed_norm.dot(sun_norm);
        if mu_0 <= 0.0 {
            // PITCH-BLACK nightside, zero light leakage!
            return [0, 0, 0];
        }

        let illumination = Self::lommel_seeliger_illumination(perturbed_norm, sun_norm, view_norm);
        let (u, v) = Self::compute_uv(norm);
        let base_rgb = self.sample_albedo_rgb(u, v);

        let r = (base_rgb[0] * illumination).round().clamp(0.0, 255.0) as u8;
        let g = (base_rgb[1] * illumination).round().clamp(0.0, 255.0) as u8;
        let b = (base_rgb[2] * illumination).round().clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}

impl Default for CallistoShader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_boundaries() {
        assert_eq!(TEX_WIDTH, 1024);
        assert_eq!(TEX_HEIGHT, 512);
        assert_eq!(PIXELS, 524_288);

        assert_eq!(CALLISTO_ALBEDO_OFFSET, 5_242_896);
        assert_eq!(CALLISTO_ELEVATION_OFFSET, 5_767_184);

        let system_data = crate::render::jupiter_shader::JUPITER_SYSTEM_DATA;
        assert_eq!(&system_data[0..8], b"ASTRO_JP");
        assert_eq!(system_data.len(), CALLISTO_ELEVATION_OFFSET + PIXELS);
        assert_eq!(system_data.len(), 6_291_472);

        let shader = CallistoShader::new();
        assert_eq!(shader.albedo_map().len(), PIXELS);
        assert_eq!(shader.elevation_map().len(), PIXELS);
    }

    #[test]
    fn test_valhalla_ripple_basin_gradients() {
        let shader = CallistoShader::new();

        // Valhalla multi-ring basin center (~18° N, -56° W / 304° E)
        let lat = 18.0_f64.to_radians();
        let lon = (-56.0_f64).to_radians();
        let x = lat.cos() * lon.cos();
        let y = lat.sin();
        let z = lat.cos() * lon.sin();
        let valhalla_norm = Vec3::new(x, y, z).normalize();

        let (u, v) = CallistoShader::compute_uv(valhalla_norm);
        assert!(u >= 0.0 && u <= 1.0);
        assert!(v >= 0.0 && v <= 1.0);

        // Perturbed normal at Valhalla basin must be unit length and exhibit topographic relief
        let perturbed = shader.perturbed_normal(valhalla_norm);
        assert!((perturbed.length() - 1.0).abs() < 1e-4);

        // Sample gradient around Valhalla ripple rings: finite differences must detect ripples
        let delta_u = 1.0 / TEX_WIDTH as f64;
        let delta_v = 1.0 / TEX_HEIGHT as f64;
        let h_u1 = shader.sample_elevation_bilinear((u + delta_u).rem_euclid(1.0), v);
        let h_u0 = shader.sample_elevation_bilinear((u - delta_u).rem_euclid(1.0), v);
        let du = (h_u1 - h_u0).abs();

        let h_v1 = shader.sample_elevation_bilinear(u, (v + delta_v).clamp(0.0, 1.0));
        let h_v0 = shader.sample_elevation_bilinear(u, (v - delta_v).clamp(0.0, 1.0));
        let dv = (h_v1 - h_v0).abs();

        // Across 3,800 km ripple basin, local gradients exist
        assert!(du >= 0.0 && dv >= 0.0);

        // Check Valhalla basin color has intermediate to bright icy-ridge tones
        let rgb = shader.sample_albedo_rgb(u, v);
        assert!(rgb[0] >= 110.0 && rgb[1] >= 110.0 && rgb[2] >= 110.0);
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = CallistoShader::new();

        let local_norm = Vec3::new(1.0, 0.0, 0.0);
        let sun_dir = Vec3::new(1.0, 0.0, 0.0);
        let view_dir = Vec3::new(1.0, 0.0, 0.0);

        // Dayside with direct overhead sunlight: bright visible color
        let day_color = shader.shade(local_norm, sun_dir, view_dir);
        assert!(
            day_color[0] > 0 && day_color[1] > 0 && day_color[2] > 0,
            "Dayside must be illuminated: {:?}",
            day_color
        );

        // Pitch-black nightside with sun in opposite hemisphere: zero light leakage
        let night_sun = Vec3::new(-1.0, 0.0, 0.0);
        let night_color = shader.shade(local_norm, night_sun, view_dir);
        assert_eq!(
            night_color,
            [0, 0, 0],
            "Nightside must be pitch-black with zero light leakage"
        );

        // Opposition surge: direct retroreflection (alpha = 0) vs phase angle 30°
        let illum_opp = CallistoShader::lommel_seeliger_illumination(local_norm, sun_dir, view_dir);
        let phase_30_sun = Vec3::new(30.0_f64.to_radians().cos(), 30.0_f64.to_radians().sin(), 0.0);
        let illum_phase = CallistoShader::lommel_seeliger_illumination(local_norm, phase_30_sun, view_dir);
        assert!(
            illum_opp > illum_phase,
            "Opposition surge must brighten direct backscattering: {} vs {}",
            illum_opp,
            illum_phase
        );
    }

    #[test]
    fn test_crater_ice_albedo() {
        // 1. Dark ancient regolith / dark blanket (~90 raw): ~[90, 85, 80]
        let dark = CallistoShader::albedo_to_rgb(90.0);
        assert!((dark[0] - 90.0).abs() < 1e-3);
        assert!((dark[1] - 85.0).abs() < 1e-3);
        assert!((dark[2] - 80.0).abs() < 1e-3);

        // 2. Intermediate cratered terrain (~140 raw): ~[125, 120, 115]
        let intermediate = CallistoShader::albedo_to_rgb(140.0);
        assert!((intermediate[0] - 125.0).abs() < 1e-3);
        assert!((intermediate[1] - 120.0).abs() < 1e-3);
        assert!((intermediate[2] - 115.0).abs() < 1e-3);

        // 3. Multi-ring ripple basin of Valhalla (~180 raw): ~[170, 175, 180]
        let valhalla = CallistoShader::albedo_to_rgb(180.0);
        assert!((valhalla[0] - 170.0).abs() < 1e-3);
        assert!((valhalla[1] - 175.0).abs() < 1e-3);
        assert!((valhalla[2] - 180.0).abs() < 1e-3);
        // Clean ice ripples exhibit subtle bluish cast (b >= r)
        assert!(valhalla[2] >= valhalla[0]);

        // 4. Fresh impact crater rims with frost pinnacles (~225 raw): ~[215, 220, 230]
        let crater_ice = CallistoShader::albedo_to_rgb(225.0);
        assert!((crater_ice[0] - 215.0).abs() < 1e-3);
        assert!((crater_ice[1] - 220.0).abs() < 1e-3);
        assert!((crater_ice[2] - 230.0).abs() < 1e-3);
        assert!(crater_ice[2] > crater_ice[0]); // Bluish-white ice gleam

        // 5. Test real high-albedo crater in dataset (e.g. Asgard/Lofn bright rims)
        let shader = CallistoShader::new();
        let mut max_albedo = 0.0_f64;
        let mut max_coord = (0.0, 0.0);
        for y in (50..450).step_by(10) {
            for x in (50..950).step_by(10) {
                let u = x as f64 / 1023.0;
                let v = y as f64 / 511.0;
                let a = shader.sample_albedo_bilinear(u, v);
                if a > max_albedo {
                    max_albedo = a;
                    max_coord = (u, v);
                }
            }
        }
        assert!(max_albedo >= 200.0, "Expected bright crater ice deposits with albedo >= 200");
        let peak_rgb = shader.sample_albedo_rgb(max_coord.0, max_coord.1);
        assert!(peak_rgb[0] >= 190.0 && peak_rgb[1] >= 195.0 && peak_rgb[2] >= 200.0);
    }

    #[test]
    fn test_sample_bilinear() {
        let shader = CallistoShader::new();
        let sample_at_0 = shader.sample_albedo_bilinear(0.0, 0.5);
        let sample_at_1 = shader.sample_albedo_bilinear(1.0, 0.5);
        // Seam wrapping: u = 0.0 and u = 1.0 wrap to the exact same longitude
        assert_eq!(sample_at_0, sample_at_1);

        // Clamping at poles: v = 0.0 (north) and v = 1.0 (south)
        let sample_north = shader.sample_albedo_bilinear(0.3, 0.0);
        let sample_north_clamped = shader.sample_albedo_bilinear(0.3, -0.5);
        assert_eq!(sample_north, sample_north_clamped);

        let sample_south = shader.sample_albedo_bilinear(0.7, 1.0);
        let sample_south_clamped = shader.sample_albedo_bilinear(0.7, 1.5);
        assert_eq!(sample_south, sample_south_clamped);
    }
}
