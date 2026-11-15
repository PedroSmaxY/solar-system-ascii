//! Photorealistic Rhea Surface Raytracing Shader:
//! - Direct zero-copy sampling of authentic Cassini Rhea 1024x512 albedo and relief datasets from `saturn_system_data.bin`.
//! - Fractional 4-texel bilinear interpolation without polar singularities.
//! - Topographic bump mapping via elevation finite differences perturbing surface normals
//!   to resolve ancient impact crater rims (e.g. Tirawa, Mamaldi basins) and wispy fault scarps.
//! - Calibrated Rhea geomorphology & color palette:
//!   * Ancient saturated ice-rock regolith: ~[215, 220, 225]
//!   * Darker crater floors / contaminated regolith: ~[170, 175, 180]
//!   * Wispy terrain (ice fracture canyons in trailing hemisphere) & fresh ejecta: ~[245, 250, 255]
//! - Lommel-Seeliger non-Lambertian scattering with opposition surge and pitch-black nightside.

use crate::math::Vec3;
use crate::render::saturn_shader::SATURN_SYSTEM_DATA;
use std::f64::consts::PI;

pub const TEX_WIDTH: usize = 1024;
pub const TEX_HEIGHT: usize = 512;
pub const PIXELS: usize = TEX_WIDTH * TEX_HEIGHT;

pub const RHEA_ALBEDO_OFFSET: usize = 16 + 12 * PIXELS + 4096; // 6_295_568
pub const RHEA_ELEVATION_OFFSET: usize = 16 + 13 * PIXELS + 4096; // 6_819_856

pub const RHEA_RADIUS_KM: f64 = 763.8;
pub const BUMP_SCALE: f64 = 0.006;

pub const COLOR_BASE_REGOLITH: [f64; 3] = [215.0, 220.0, 225.0];
pub const COLOR_DARK_CRATER_FLOOR: [f64; 3] = [170.0, 175.0, 180.0];
pub const COLOR_WISPY_BRIGHT_ICE: [f64; 3] = [245.0, 250.0, 255.0];

#[derive(Clone, Copy, Debug)]
pub struct RheaShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl RheaShader {
    /// Initialize RheaShader with zero-copy slices from `saturn_system_data.bin`.
    pub fn new() -> Self {
        assert!(
            SATURN_SYSTEM_DATA.len() >= RHEA_ELEVATION_OFFSET + PIXELS,
            "saturn_system_data.bin buffer too small for Rhea datasets"
        );
        assert_eq!(
            &SATURN_SYSTEM_DATA[0..8],
            b"ASTRO_SA",
            "Invalid saturn_system_data.bin header format"
        );
        let albedo_map = &SATURN_SYSTEM_DATA[RHEA_ALBEDO_OFFSET..RHEA_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &SATURN_SYSTEM_DATA[RHEA_ELEVATION_OFFSET..RHEA_ELEVATION_OFFSET + PIXELS];
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

    /// Calculate spherical UV coordinates from local surface normal (x, y, z) where y is polar axis.
    #[inline]
    pub fn compute_uv(local_norm: Vec3) -> (f64, f64) {
        let norm = local_norm.normalize();
        let v = (1.0
            - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5))
            .clamp(0.0, 1.0);
        let u = ((norm.z.atan2(norm.x) + PI) / (2.0 * PI)).rem_euclid(1.0);
        (u, v)
    }

    /// 4-texel fractional bilinear interpolation on cylindrical projection with seamless wrapping on u.
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

    /// Convert raw 8-bit Rhea albedo to calibrated planetary RGB:
    /// - Ancient heavily-cratered ice-rock regolith: ~[215, 220, 225]
    /// - Older contaminated regolith / darker crater floors: ~[170, 175, 180]
    /// - Wispy terrain (ice fracture canyons in trailing hemisphere) & fresh ejecta: ~[245, 250, 255]
    #[inline]
    pub fn albedo_to_rgb(raw: f64) -> [f64; 3] {
        if raw <= 205.0 {
            let t = ((raw - 146.0) / (205.0 - 146.0)).clamp(0.0, 1.0);
            [
                COLOR_DARK_CRATER_FLOOR[0] + (COLOR_BASE_REGOLITH[0] - COLOR_DARK_CRATER_FLOOR[0]) * t,
                COLOR_DARK_CRATER_FLOOR[1] + (COLOR_BASE_REGOLITH[1] - COLOR_DARK_CRATER_FLOOR[1]) * t,
                COLOR_DARK_CRATER_FLOOR[2] + (COLOR_BASE_REGOLITH[2] - COLOR_DARK_CRATER_FLOOR[2]) * t,
            ]
        } else {
            let t = ((raw - 205.0) / (255.0 - 205.0)).clamp(0.0, 1.0);
            [
                COLOR_BASE_REGOLITH[0] + (COLOR_WISPY_BRIGHT_ICE[0] - COLOR_BASE_REGOLITH[0]) * t,
                COLOR_BASE_REGOLITH[1] + (COLOR_WISPY_BRIGHT_ICE[1] - COLOR_BASE_REGOLITH[1]) * t,
                COLOR_BASE_REGOLITH[2] + (COLOR_WISPY_BRIGHT_ICE[2] - COLOR_BASE_REGOLITH[2]) * t,
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
    pub fn perturbed_normal(&self, local_norm: Vec3) -> Vec3 {
        let norm = if local_norm.length_squared() > 1e-12 {
            local_norm.normalize()
        } else {
            return Vec3::new(0.0, 1.0, 0.0);
        };

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
    /// mu_0 = perturbed_norm.dot(sun_norm)
    /// mu = perturbed_norm.dot(view_norm).max(0.001)
    /// ls = mu_0 / (mu_0 + mu)
    /// alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0).acos()
    /// surge = 1.0 + 0.40 * (1.0 - (alpha / 0.18).min(1.0))
    /// illumination = (ls * surge / 0.70).clamp(0.0, 1.5)
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
        let surge = 1.0 + 0.40 * (1.0 - (alpha / 0.18).min(1.0));

        (ls * surge / 0.70).clamp(0.0, 1.5)
    }

    /// Primary photorealistic raytracing shader for Rhea.
    /// - `local_norm`: surface normal in Rhea rotating body frame.
    /// - `local_sun`: incident light direction in Rhea body frame.
    /// - `local_view`: view direction towards camera in Rhea body frame.
    pub fn shade(&self, local_norm: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        if local_norm.length_squared() < 1e-12 {
            return [0, 0, 0];
        }
        let sun_norm = local_sun.normalize();
        let view_norm = local_view.normalize();
        if sun_norm.length_squared() < 1e-6 || view_norm.length_squared() < 1e-6 {
            return [0, 0, 0];
        }

        let norm = local_norm.normalize();
        let perturbed_norm = self.perturbed_normal(norm);

        let n_dot_l = perturbed_norm.dot(sun_norm);
        if n_dot_l <= 0.0 {
            // Strictly pitch-black nightside: zero light leakage
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

impl Default for RheaShader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhea_slice_boundaries() {
        assert_eq!(TEX_WIDTH, 1024);
        assert_eq!(TEX_HEIGHT, 512);
        assert_eq!(PIXELS, 524_288);

        assert_eq!(RHEA_ALBEDO_OFFSET, 16 + 12 * PIXELS + 4096);
        assert_eq!(RHEA_ALBEDO_OFFSET, 6_295_568);
        assert_eq!(RHEA_ELEVATION_OFFSET, 16 + 13 * PIXELS + 4096);
        assert_eq!(RHEA_ELEVATION_OFFSET, 6_819_856);

        let system_data = SATURN_SYSTEM_DATA;
        assert_eq!(&system_data[0..8], b"ASTRO_SA");
        assert_eq!(system_data.len(), RHEA_ELEVATION_OFFSET + PIXELS);
        assert_eq!(system_data.len(), 7_344_144);

        let shader = RheaShader::new();
        assert_eq!(shader.albedo_map().len(), PIXELS);
        assert_eq!(shader.elevation_map().len(), PIXELS);
    }

    #[test]
    fn test_sample_bilinear() {
        let mut mock_data = vec![0u8; TEX_WIDTH * TEX_HEIGHT];
        // Known 2x2 cell at (0, 0)
        mock_data[0] = 10;
        mock_data[1] = 20;
        mock_data[TEX_WIDTH] = 30;
        mock_data[TEX_WIDTH + 1] = 40;

        let u = 0.5 / (TEX_WIDTH as f64 - 1.0);
        let v = 0.5 / (TEX_HEIGHT as f64 - 1.0);
        let sampled = RheaShader::sample_bilinear(&mock_data, u, v);
        assert!((sampled - 25.0).abs() < 1e-4);

        // Seam wrapping: u = 0.0 and u = 1.0 wrap to the same longitude
        let shader = RheaShader::new();
        let sample_0 = shader.sample_albedo_bilinear(0.0, 0.5);
        let sample_1 = shader.sample_albedo_bilinear(1.0, 0.5);
        assert_eq!(sample_0, sample_1);

        // Polar clamping: v = 0.0 clamped for v < 0, v = 1.0 clamped for v > 1
        let north = shader.sample_albedo_bilinear(0.35, 0.0);
        let north_clamped = shader.sample_albedo_bilinear(0.35, -0.5);
        assert_eq!(north, north_clamped);

        let south = shader.sample_albedo_bilinear(0.65, 1.0);
        let south_clamped = shader.sample_albedo_bilinear(0.65, 1.5);
        assert_eq!(south, south_clamped);
    }

    #[test]
    fn test_crater_relief_bump_gradients() {
        let shader = RheaShader::new();

        // North and South pole singularity-free perturbed normal
        let north_pole = Vec3::new(0.0, 1.0, 0.0);
        let south_pole = Vec3::new(0.0, -1.0, 0.0);
        let n_norm = shader.perturbed_normal(north_pole);
        let s_norm = shader.perturbed_normal(south_pole);

        assert!(!n_norm.x.is_nan() && !n_norm.y.is_nan() && !n_norm.z.is_nan());
        assert!(!s_norm.x.is_nan() && !s_norm.y.is_nan() && !s_norm.z.is_nan());
        assert!((n_norm.length() - 1.0).abs() < 1e-4);
        assert!((s_norm.length() - 1.0).abs() < 1e-4);

        // Tirawa impact basin (~34° N, 151° W / 209° E)
        let tirawa_lat = 34.0_f64.to_radians();
        let tirawa_lon = (-151.0_f64).to_radians();
        let tirawa_norm = Vec3::new(
            tirawa_lat.cos() * tirawa_lon.cos(),
            tirawa_lat.sin(),
            tirawa_lat.cos() * tirawa_lon.sin(),
        )
        .normalize();

        let perturbed_tirawa = shader.perturbed_normal(tirawa_norm);
        assert!((perturbed_tirawa.length() - 1.0).abs() < 1e-4);

        // Verify non-zero gradient across cratered terrain & wispy fractures
        let mut found_relief_gradient = false;
        for lat_step in 1..8 {
            let lat = (lat_step as f64 / 8.0 - 0.5) * PI * 0.8;
            for lon_step in 0..16 {
                let lon = (lon_step as f64 / 16.0) * 2.0 * PI - PI;
                let base = Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
                let perturbed = shader.perturbed_normal(base);
                assert!((perturbed.length() - 1.0).abs() < 1e-4);
                if (perturbed - base).length() > 1e-5 {
                    found_relief_gradient = true;
                    break;
                }
            }
            if found_relief_gradient {
                break;
            }
        }
        assert!(
            found_relief_gradient,
            "Crater relief bump mapping must produce detectable gradients across Rhea's surface"
        );
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = RheaShader::new();

        let local_norm = Vec3::new(1.0, 0.0, 0.0);
        let sun_dir = Vec3::new(1.0, 0.0, 0.0);
        let view_dir = Vec3::new(1.0, 0.0, 0.0);

        // Dayside with direct overhead sunlight: bright visible color calibrated to ice-rock regolith
        let day_color = shader.shade(local_norm, sun_dir, view_dir);
        assert!(
            day_color[0] > 0 && day_color[1] > 0 && day_color[2] > 0,
            "Dayside must be illuminated: {:?}",
            day_color
        );
        // Base color around [215, 220, 225]
        assert!(day_color[0] >= 180 && day_color[1] >= 180 && day_color[2] >= 180);

        // Strictly pitch-black nightside when n_dot_l <= 0.0: zero light leakage
        let night_sun = Vec3::new(-1.0, 0.0, 0.0);
        let night_color = shader.shade(local_norm, night_sun, view_dir);
        assert_eq!(
            night_color,
            [0, 0, 0],
            "Nightside must be pitch-black with zero light leakage"
        );

        // Opposition surge: direct retroreflection (alpha = 0) vs phase angle 30°
        let illum_opp = RheaShader::lommel_seeliger_illumination(local_norm, sun_dir, view_dir);
        let phase_30_sun = Vec3::new(30.0_f64.to_radians().cos(), 30.0_f64.to_radians().sin(), 0.0);
        let illum_phase = RheaShader::lommel_seeliger_illumination(local_norm, phase_30_sun, view_dir);
        assert!(
            illum_opp > illum_phase,
            "Opposition surge must brighten direct backscattering: {} vs {}",
            illum_opp,
            illum_phase
        );
    }
}
