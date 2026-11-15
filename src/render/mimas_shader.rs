//! Photorealistic Mimas "Death Star" Raytracing Shader:
//! - Direct zero-copy sampling of authentic NASA Cassini 1024x512 Mimas albedo
//!   and Herschel crater relief dataset from `saturn_system_data.bin`.
//! - Fractional 4-texel bilinear interpolation without polar singularities.
//! - Pronounced bump mapping resolving the colossal Herschel impact basin
//!   (139 km diameter, 10 km towering rim walls, 6 km central peak).
//! - Calibrated Mimas metallic steel / icy regolith color palette ([195, 195, 205]).
//! - Lommel-Seeliger non-Lambertian scattering with opposition surge.
//! - Strictly pitch-black nightside with zero light leakage (n_dot_l <= 0.0).

use crate::math::Vec3;
use std::f64::consts::PI;

pub const TEX_WIDTH: usize = 1024;
pub const TEX_HEIGHT: usize = 512;
pub const PIXELS: usize = TEX_WIDTH * TEX_HEIGHT;

pub const MIMAS_ALBEDO_OFFSET: usize = 16 + 8 * PIXELS + 4096; // 4_198_416
pub const MIMAS_ELEVATION_OFFSET: usize = 16 + 9 * PIXELS + 4096; // 4_722_704

pub const BUMP_SCALE: f64 = 0.010;

pub const COLOR_STEEL_GRAY: [f64; 3] = [195.0, 195.0, 205.0];
pub const COLOR_CRATER_FLOOR: [f64; 3] = [140.0, 140.0, 150.0];
pub const COLOR_BRIGHT_ICE: [f64; 3] = [235.0, 235.0, 245.0];

#[derive(Clone, Copy, Debug)]
pub struct MimasShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl MimasShader {
    /// Initialize MimasShader with zero-copy slices from `saturn_system_data.bin`.
    pub fn new() -> Self {
        let system_data = crate::render::saturn_shader::SATURN_SYSTEM_DATA;
        assert!(
            system_data.len() >= MIMAS_ELEVATION_OFFSET + PIXELS,
            "saturn_system_data.bin buffer too small for Mimas datasets"
        );
        assert_eq!(
            &system_data[0..8],
            b"ASTRO_SA",
            "Invalid saturn_system_data.bin header format"
        );
        let albedo_map = &system_data[MIMAS_ALBEDO_OFFSET..MIMAS_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &system_data[MIMAS_ELEVATION_OFFSET..MIMAS_ELEVATION_OFFSET + PIXELS];
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
    #[inline(always)]
    pub fn compute_uv(local_norm: Vec3) -> (f64, f64) {
        let norm = local_norm.normalize();
        let v = (1.0
            - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5))
            .clamp(0.0, 1.0);
        let u = ((norm.z.atan2(norm.x) + PI) / (2.0 * PI)).rem_euclid(1.0);
        (u, v)
    }

    #[inline(always)]
    pub fn sphere_coords(local_norm: Vec3) -> (f64, f64) {
        Self::compute_uv(local_norm)
    }

    /// 4-texel fractional bilinear interpolation on 1024x512 texture with seamless wrapping on u.
    #[inline(always)]
    pub fn sample_bilinear(data: &[u8], u: f64, v: f64) -> f64 {
        let u_wrap = u.rem_euclid(1.0);
        let v_clamp = v.clamp(0.0, 1.0);

        let fx = u_wrap * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamp * (TEX_HEIGHT as f64 - 1.0);

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
    #[inline(always)]
    pub fn sample_albedo_bilinear(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.albedo_map, u, v)
    }

    /// Sample topographic elevation [0.0 .. 255.0] with bilinear filtering.
    #[inline(always)]
    pub fn sample_elevation_bilinear(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.elevation_map, u, v)
    }

    /// Convert raw 8-bit Mimas albedo to calibrated steel/metallic gray planetary RGB:
    /// - Deep crater floor / shadow depressions: ~[140, 140, 150]
    /// - Standard cratered metallic steel terrain: [195, 195, 205] (albedo ~ 200)
    /// - Herschel central peak / bright icy ejecta: ~[235, 235, 245]
    #[inline(always)]
    pub fn albedo_to_rgb(raw: f64) -> [f64; 3] {
        let raw_clamped = raw.clamp(0.0, 255.0);
        if raw_clamped <= 160.0 {
            let t = raw_clamped / 160.0;
            [
                COLOR_CRATER_FLOOR[0] * t,
                COLOR_CRATER_FLOOR[1] * t,
                COLOR_CRATER_FLOOR[2] * t,
            ]
        } else if raw_clamped <= 200.0 {
            let t = (raw_clamped - 160.0) / (200.0 - 160.0);
            [
                COLOR_CRATER_FLOOR[0] + (COLOR_STEEL_GRAY[0] - COLOR_CRATER_FLOOR[0]) * t,
                COLOR_CRATER_FLOOR[1] + (COLOR_STEEL_GRAY[1] - COLOR_CRATER_FLOOR[1]) * t,
                COLOR_CRATER_FLOOR[2] + (COLOR_STEEL_GRAY[2] - COLOR_CRATER_FLOOR[2]) * t,
            ]
        } else {
            let t = ((raw_clamped - 200.0) / (255.0 - 200.0)).clamp(0.0, 1.0);
            [
                COLOR_STEEL_GRAY[0] + (COLOR_BRIGHT_ICE[0] - COLOR_STEEL_GRAY[0]) * t,
                COLOR_STEEL_GRAY[1] + (COLOR_BRIGHT_ICE[1] - COLOR_STEEL_GRAY[1]) * t,
                COLOR_STEEL_GRAY[2] + (COLOR_BRIGHT_ICE[2] - COLOR_STEEL_GRAY[2]) * t,
            ]
        }
    }

    /// Sample RGB albedo at spherical coordinates (u, v).
    #[inline(always)]
    pub fn sample_albedo_rgb(&self, u: f64, v: f64) -> [f64; 3] {
        let raw = self.sample_albedo_bilinear(u, v);
        Self::albedo_to_rgb(raw)
    }

    /// Compute bump-mapped surface normal using finite differences in elevation map.
    /// Accentuates the colossal Herschel crater basin (139 km diameter, 10 km walls, 6 km peak).
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

    #[inline(always)]
    pub fn surface_normal(&self, local_norm: Vec3) -> Vec3 {
        self.perturbed_normal(local_norm)
    }

    /// Lommel-Seeliger non-Lambertian reflectance with opposition surge:
    /// - `mu_0 = perturbed_norm.dot(sun_norm)`
    /// - `mu = perturbed_norm.dot(view_norm).max(0.001)`
    /// - `ls = mu_0 / (mu_0 + mu)`
    /// - Retroreflection opposition surge when phase angle alpha ~ 0.
    #[inline(always)]
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
        let surge = 1.0 + 0.45 * (1.0 - (alpha / 0.18).min(1.0));

        (ls * surge * 1.85).clamp(0.0, 1.5)
    }

    /// Primary photorealistic raytracing shader for Mimas.
    /// - `local_norm`: surface normal in Mimas rotating body frame.
    /// - `local_sun`: incident sun direction in Mimas body frame.
    /// - `local_view`: view direction towards camera in Mimas body frame.
    /// - Returns calibrated [r, g, b] with pitch-black nightside.
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
        let n_dot_l = norm.dot(sun_norm);
        if n_dot_l <= 0.0 {
            // Strictly pitch-black nightside, zero light leakage!
            return [0, 0, 0];
        }

        let perturbed_norm = self.perturbed_normal(norm);
        let mu_0 = perturbed_norm.dot(sun_norm);
        if mu_0 <= 0.0 {
            // Local topographic shadow cast by crater walls
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

impl Default for MimasShader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mimas_slice_boundaries() {
        assert_eq!(TEX_WIDTH, 1024);
        assert_eq!(TEX_HEIGHT, 512);
        assert_eq!(PIXELS, 524_288);

        assert_eq!(MIMAS_ALBEDO_OFFSET, 4_198_416);
        assert_eq!(MIMAS_ELEVATION_OFFSET, 4_722_704);
        assert_eq!(MIMAS_ALBEDO_OFFSET, 16 + 8 * PIXELS + 4096);
        assert_eq!(MIMAS_ELEVATION_OFFSET, 16 + 9 * PIXELS + 4096);

        let system_data = crate::render::saturn_shader::SATURN_SYSTEM_DATA;
        assert_eq!(&system_data[0..8], b"ASTRO_SA");
        assert!(system_data.len() >= MIMAS_ELEVATION_OFFSET + PIXELS);
        assert_eq!(system_data.len(), 7_344_144);

        let shader = MimasShader::new();
        assert_eq!(shader.albedo_map().len(), PIXELS);
        assert_eq!(shader.elevation_map().len(), PIXELS);
    }

    #[test]
    fn test_sample_bilinear() {
        // 1. Test 4-texel fractional interpolation on synthetic buffer
        let mut mock_data = vec![0u8; TEX_WIDTH * TEX_HEIGHT];
        mock_data[0] = 10;
        mock_data[1] = 20;
        mock_data[TEX_WIDTH] = 30;
        mock_data[TEX_WIDTH + 1] = 40;

        let u_mid = 0.5 / (TEX_WIDTH as f64 - 1.0);
        let v_mid = 0.5 / (TEX_HEIGHT as f64 - 1.0);
        let sampled = MimasShader::sample_bilinear(&mock_data, u_mid, v_mid);
        assert!(
            (sampled - 25.0).abs() < 1e-4,
            "Expected bilinear center average 25.0, got {}",
            sampled
        );

        // 2. Test seamless longitudinal seam wrapping: u = 0.0 and u = 1.0
        let shader = MimasShader::new();
        let val_at_0 = shader.sample_albedo_bilinear(0.0, 0.5);
        let val_at_1 = shader.sample_albedo_bilinear(1.0, 0.5);
        assert_eq!(val_at_0, val_at_1, "Seam wrapping mismatch at u=0.0 vs u=1.0");

        let val_wrap_pos = shader.sample_albedo_bilinear(1.35, 0.4);
        let val_wrap_base = shader.sample_albedo_bilinear(0.35, 0.4);
        assert!(
            (val_wrap_pos - val_wrap_base).abs() < 1e-4,
            "Positive wrapping mismatch"
        );

        let val_wrap_neg = shader.sample_albedo_bilinear(-0.25, 0.6);
        let val_wrap_equiv = shader.sample_albedo_bilinear(0.75, 0.6);
        assert!(
            (val_wrap_neg - val_wrap_equiv).abs() < 1e-4,
            "Negative wrapping mismatch"
        );

        // 3. Test polar clamping: v < 0.0 and v > 1.0
        let val_north = shader.sample_albedo_bilinear(0.4, 0.0);
        let val_north_clamped = shader.sample_albedo_bilinear(0.4, -0.8);
        assert_eq!(val_north, val_north_clamped, "North pole clamping failed");

        let val_south = shader.sample_albedo_bilinear(0.4, 1.0);
        let val_south_clamped = shader.sample_albedo_bilinear(0.4, 1.8);
        assert_eq!(val_south, val_south_clamped, "South pole clamping failed");
    }

    #[test]
    fn test_herschel_crater_basin_bump_gradients() {
        let shader = MimasShader::new();

        // Herschel impact basin center on Mimas: ~1.4° S, ~111.7° W
        // In the 1024x512 texture, the depression and rim span x ~ 230..280, y ~ 240..270.
        // Local normal for the steep crater wall near x=265, y=256:
        let u_wall = 265.0 / TEX_WIDTH as f64;
        let v_wall = 256.0 / TEX_HEIGHT as f64;
        let lon_rad = u_wall * 2.0 * PI - PI;
        let lat_rad = ((1.0 - v_wall) - 0.5) * PI;

        let wall_norm = Vec3::new(
            lat_rad.cos() * lon_rad.cos(),
            lat_rad.sin(),
            lat_rad.cos() * lon_rad.sin(),
        )
        .normalize();

        let (u, v) = MimasShader::compute_uv(wall_norm);
        assert!((u - u_wall).abs() < 1e-3);
        assert!((v - v_wall).abs() < 1e-3);

        // Perturbed normal at Herschel basin must be a valid unit vector with no NaN
        let perturbed = shader.perturbed_normal(wall_norm);
        assert!(!perturbed.x.is_nan() && !perturbed.y.is_nan() && !perturbed.z.is_nan());
        assert!((perturbed.length() - 1.0).abs() < 1e-4);

        // Verify that the steep 10 km crater walls of Herschel produce significant normal perturbation
        let diff = (perturbed - wall_norm).length();
        assert!(
            diff > 0.02,
            "Herschel crater rim wall must exhibit pronounced bump perturbation: diff = {}",
            diff
        );

        // Sample finite-difference elevation gradient directly in Herschel basin
        let delta_u = 1.0 / TEX_WIDTH as f64;
        let delta_v = 1.0 / TEX_HEIGHT as f64;
        let h_u1 = shader.sample_elevation_bilinear((u + delta_u).rem_euclid(1.0), v);
        let h_u0 = shader.sample_elevation_bilinear((u - delta_u).rem_euclid(1.0), v);
        let du = (h_u1 - h_u0) * 0.5;

        let h_v1 = shader.sample_elevation_bilinear(u, (v + delta_v).clamp(0.0, 1.0));
        let h_v0 = shader.sample_elevation_bilinear(u, (v - delta_v).clamp(0.0, 1.0));
        let dv = (h_v1 - h_v0) * 0.5;

        let grad_mag = (du * du + dv * dv).sqrt();
        assert!(
            grad_mag >= 3.0,
            "Herschel crater wall must exhibit steep elevation gradient >= 3.0, got {}",
            grad_mag
        );

        // Polar stability check: poles must have valid normals without singularity
        let north_pole = shader.perturbed_normal(Vec3::new(0.0, 1.0, 0.0));
        let south_pole = shader.perturbed_normal(Vec3::new(0.0, -1.0, 0.0));
        assert!(!north_pole.x.is_nan() && !north_pole.y.is_nan() && !north_pole.z.is_nan());
        assert!(!south_pole.x.is_nan() && !south_pole.y.is_nan() && !south_pole.z.is_nan());
        assert!((north_pole.length() - 1.0).abs() < 1e-4);
        assert!((south_pole.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = MimasShader::new();

        let local_norm = Vec3::new(1.0, 0.0, 0.0);
        let sun_dir = Vec3::new(1.0, 0.0, 0.0);
        let view_dir = Vec3::new(1.0, 0.0, 0.0);

        // 1. Dayside with direct subsolar illumination
        let day_color = shader.shade(local_norm, sun_dir, view_dir);
        assert!(
            day_color[0] > 0 && day_color[1] > 0 && day_color[2] > 0,
            "Dayside must be illuminated: {:?}",
            day_color
        );
        // Steel metallic gray tone check: R == G, and B has icy tint (B >= R)
        assert!(
            day_color[2] >= day_color[0],
            "Icy metallic steel color should have B >= R: {:?}",
            day_color
        );

        // 2. Strict pitch-black nightside when sun is in opposite hemisphere (n_dot_l <= 0.0)
        let night_sun = Vec3::new(-1.0, 0.0, 0.0);
        let night_color = shader.shade(local_norm, night_sun, view_dir);
        assert_eq!(
            night_color,
            [0, 0, 0],
            "Nightside must be strictly pitch-black with zero light leakage"
        );

        // 3. Exact terminator boundary (n_dot_l = 0.0)
        let terminator_sun = Vec3::new(0.0, 1.0, 0.0);
        let term_color = shader.shade(local_norm, terminator_sun, view_dir);
        assert_eq!(
            term_color,
            [0, 0, 0],
            "Terminator grazing illumination must be strictly zero when n_dot_l <= 0"
        );

        // 4. Opposition surge: retroreflection (alpha = 0) vs phase angle 30°
        let illum_opp = MimasShader::lommel_seeliger_illumination(local_norm, sun_dir, view_dir);
        let phase_30_sun = Vec3::new(
            30.0_f64.to_radians().cos(),
            30.0_f64.to_radians().sin(),
            0.0,
        );
        let illum_phase =
            MimasShader::lommel_seeliger_illumination(local_norm, phase_30_sun, view_dir);
        assert!(
            illum_opp > illum_phase,
            "Opposition surge must brighten direct backscattering: {} vs {}",
            illum_opp,
            illum_phase
        );
    }
}
