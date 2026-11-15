//! Photorealistic Enceladus Raytracing Shader:
//! - Direct zero-copy slicing of authentic NASA Cassini 1024x512 albedo and topographic relief datasets
//!   from `saturn_system_data.bin`.
//! - Fractional 4-texel bilinear interpolation with seamless longitude wrapping.
//! - Highest geometric albedo in the Solar System (~0.99, pristine water ice snow, base [250, 252, 255]).
//! - Topographic bump mapping via central finite differences capturing the South Polar Terrain "tiger stripes"
//!   fissures (Alexandria, Cairo, Baghdad, Damascus Sulci) without polar singularities.
//! - Non-Lambertian Lommel-Seeliger scattering with retroreflective opposition surge for fresh microscopic ice grains.
//! - Strictly pitch-black nightside with zero light leakage (color = [0, 0, 0] when n_dot_l <= 0.0).

use crate::math::Vec3;
use crate::render::saturn_shader::{PIXELS, SATURN_SYSTEM_DATA, TEX_HEIGHT, TEX_WIDTH};
use std::f64::consts::PI;

pub const ENCELADUS_ALBEDO_OFFSET: usize = 16 + 6 * PIXELS + 4096; // 3_149_840
pub const ENCELADUS_ELEVATION_OFFSET: usize = 16 + 7 * PIXELS + 4096; // 3_674_128

pub const COLOR_BASE_ICE: [f64; 3] = [250.0, 252.0, 255.0]; // Pristine water ice snowpack (albedo ~ 0.99)
pub const COLOR_CRATERED: [f64; 3] = [228.0, 234.0, 245.0]; // Moderately cratered north/equatorial plains
pub const COLOR_TIGER_STRIPES: [f64; 3] = [185.0, 205.0, 228.0]; // Fractured sulci vents (coarse crystalline ice)

pub const BUMP_SCALE: f64 = 0.008;

pub struct EnceladusShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl Default for EnceladusShader {
    fn default() -> Self {
        Self::new()
    }
}

impl EnceladusShader {
    pub fn new() -> Self {
        assert_eq!(
            &SATURN_SYSTEM_DATA[0..8],
            b"ASTRO_SA",
            "Invalid saturn_system_data.bin header magic"
        );
        let width = u32::from_le_bytes([
            SATURN_SYSTEM_DATA[8],
            SATURN_SYSTEM_DATA[9],
            SATURN_SYSTEM_DATA[10],
            SATURN_SYSTEM_DATA[11],
        ]) as usize;
        let height = u32::from_le_bytes([
            SATURN_SYSTEM_DATA[12],
            SATURN_SYSTEM_DATA[13],
            SATURN_SYSTEM_DATA[14],
            SATURN_SYSTEM_DATA[15],
        ]) as usize;
        assert_eq!(width, TEX_WIDTH, "Unexpected texture width in Saturn dataset");
        assert_eq!(
            height, TEX_HEIGHT,
            "Unexpected texture height in Saturn dataset"
        );

        let albedo_map =
            &SATURN_SYSTEM_DATA[ENCELADUS_ALBEDO_OFFSET..ENCELADUS_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &SATURN_SYSTEM_DATA[ENCELADUS_ELEVATION_OFFSET..ENCELADUS_ELEVATION_OFFSET + PIXELS];

        Self {
            albedo_map,
            elevation_map,
        }
    }

    /// Calculate spherical UV coordinates from unit normal (x, y, z) where +Y is North Pole:
    /// v = (1.0 - (y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0)
    /// u = (z.atan2(x) + PI) / (2.0 * PI)
    #[inline(always)]
    pub fn spherical_coords(norm: Vec3) -> (f64, f64) {
        let v = (1.0 - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = (norm.z.atan2(norm.x) + PI) / (2.0 * PI);
        (u, v)
    }

    /// 4-sample fractional bilinear interpolation on cylindrical map with seamless wrapping on u.
    #[inline(always)]
    pub fn sample_bilinear(data: &[u8], u: f64, v: f64) -> f64 {
        let u_clamped = u.rem_euclid(1.0);
        let v_clamped = v.clamp(0.0, 1.0);

        let fx = u_clamped * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamped * (TEX_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % TEX_WIDTH;
        let y1 = (y0 + 1).min(TEX_HEIGHT - 1);

        let tx = fx - fx.floor();
        let ty = fy - fy.floor();

        let p00 = data[y0 * TEX_WIDTH + x0] as f64;
        let p10 = data[y0 * TEX_WIDTH + x1] as f64;
        let p01 = data[y1 * TEX_WIDTH + x0] as f64;
        let p11 = data[y1 * TEX_WIDTH + x1] as f64;

        let top = p00 * (1.0 - tx) + p10 * tx;
        let bot = p01 * (1.0 - tx) + p11 * tx;
        top * (1.0 - ty) + bot * ty
    }

    #[inline(always)]
    pub fn sample_albedo(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.albedo_map, u, v)
    }

    #[inline(always)]
    pub fn sample_elevation(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.elevation_map, u, v)
    }

    /// Map raw 8-bit albedo value to calibrated Enceladus RGB palette:
    /// - Pristine water ice snowpack (albedo ~0.99): [250, 252, 255]
    /// - Moderately cratered north/equatorial plains: [228, 234, 245]
    /// - Tiger stripes fractures / sulci vents (coarse crystalline ice): [185, 205, 228]
    #[inline]
    pub fn albedo_to_rgb(raw: f64) -> [f64; 3] {
        if raw <= 200.0 {
            let t = ((raw - 165.0) / (200.0 - 165.0)).clamp(0.0, 1.0);
            let smooth_t = t * t * (3.0 - 2.0 * t);
            [
                COLOR_TIGER_STRIPES[0] + (COLOR_CRATERED[0] - COLOR_TIGER_STRIPES[0]) * smooth_t,
                COLOR_TIGER_STRIPES[1] + (COLOR_CRATERED[1] - COLOR_TIGER_STRIPES[1]) * smooth_t,
                COLOR_TIGER_STRIPES[2] + (COLOR_CRATERED[2] - COLOR_TIGER_STRIPES[2]) * smooth_t,
            ]
        } else {
            let t = ((raw - 200.0) / (255.0 - 200.0)).clamp(0.0, 1.0);
            let smooth_t = t * t * (3.0 - 2.0 * t);
            [
                COLOR_CRATERED[0] + (COLOR_BASE_ICE[0] - COLOR_CRATERED[0]) * smooth_t,
                COLOR_CRATERED[1] + (COLOR_BASE_ICE[1] - COLOR_CRATERED[1]) * smooth_t,
                COLOR_CRATERED[2] + (COLOR_BASE_ICE[2] - COLOR_CRATERED[2]) * smooth_t,
            ]
        }
    }

    /// Compute central finite difference elevation gradient (du, dv) at UV coordinates (u, v).
    pub fn compute_elevation_gradient(&self, u: f64, v: f64) -> (f64, f64) {
        let du_step = 1.0 / TEX_WIDTH as f64;
        let dv_step = 1.0 / TEX_HEIGHT as f64;

        let h_east = self.sample_elevation((u + du_step).rem_euclid(1.0), v);
        let h_west = self.sample_elevation((u - du_step).rem_euclid(1.0), v);
        let h_south = self.sample_elevation(u, (v + dv_step).clamp(0.0, 1.0));
        let h_north = self.sample_elevation(u, (v - dv_step).clamp(0.0, 1.0));

        let du = (h_east - h_west) * 0.5;
        let dv = (h_south - h_north) * 0.5;
        (du, dv)
    }

    /// Perturb unit surface normal using central finite differences on elevation map,
    /// constructing tangent and bitangent vectors without polar singularities.
    pub fn surface_normal(&self, norm: Vec3) -> Vec3 {
        let (u, v) = Self::spherical_coords(norm);
        let (du, dv) = self.compute_elevation_gradient(u, v);

        // Orthonormal tangent & bitangent frame robust against polar singularities
        let xz_len = (norm.x * norm.x + norm.z * norm.z).sqrt();
        let tangent = if xz_len > 1e-6 {
            Vec3::new(-norm.z / xz_len, 0.0, norm.x / xz_len)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let bitangent = norm.cross(tangent);

        (norm - tangent * (du * BUMP_SCALE) - bitangent * (dv * BUMP_SCALE)).normalize()
    }

    #[inline(always)]
    pub fn perturbed_normal(&self, norm: Vec3) -> Vec3 {
        self.surface_normal(norm)
    }

    /// Lommel-Seeliger non-Lambertian reflectance: mu_0 / (mu_0 + mu).
    #[inline(always)]
    pub fn lommel_seeliger(mu_0: f64, mu: f64) -> f64 {
        if mu_0 <= 0.0 {
            0.0
        } else {
            mu_0 / (mu_0 + mu.max(0.001))
        }
    }

    /// Opposition surge retroreflective brightening for fresh microscopic ice crystals:
    /// Peak boost at opposition (alpha -> 0) leveling off for phase angles >= 0.15 rad (~8.6 deg).
    #[inline(always)]
    pub fn opposition_surge(alpha: f64) -> f64 {
        1.0 + 0.45 * (1.0 - (alpha / 0.15).min(1.0))
    }

    /// Primary Enceladus photorealistic surface raytracing shader:
    /// - `local_norm`: surface normal or hit vector from center in Enceladus's body frame.
    /// - `local_sun`: incident sunlight vector in body frame.
    /// - `local_view`: view direction towards camera in body frame.
    pub fn shade(&self, local_norm: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        if local_norm.length_squared() < 1e-12
            || local_sun.length_squared() < 1e-12
            || local_view.length_squared() < 1e-12
        {
            return [0, 0, 0];
        }

        let base_norm = local_norm.normalize();
        let sun_norm = local_sun.normalize();
        let view_norm = local_view.normalize();

        // Geometric terminator check: strict zero light leakage
        let geom_n_dot_l = base_norm.dot(sun_norm);
        if geom_n_dot_l <= 0.0 {
            return [0, 0, 0];
        }

        // Perturb normal with topographic bump mapping
        let perturbed_norm = self.surface_normal(base_norm);
        let n_dot_l = perturbed_norm.dot(sun_norm);
        if n_dot_l <= 0.0 {
            return [0, 0, 0];
        }

        let mu_0 = n_dot_l;
        let mu = perturbed_norm.dot(view_norm).max(0.001);
        let ls = Self::lommel_seeliger(mu_0, mu);

        let cos_alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let surge = Self::opposition_surge(alpha);

        let illumination = (ls * surge * 2.0).clamp(0.0, 1.6);

        let (u, v) = Self::spherical_coords(base_norm);
        let raw_albedo = self.sample_albedo(u, v);
        let base_rgb = Self::albedo_to_rgb(raw_albedo);

        let r = (base_rgb[0] * illumination).clamp(0.0, 255.0) as u8;
        let g = (base_rgb[1] * illumination).clamp(0.0, 255.0) as u8;
        let b = (base_rgb[2] * illumination).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enceladus_slice_boundaries() {
        assert_eq!(SATURN_SYSTEM_DATA.len(), 7_344_144);
        assert_eq!(&SATURN_SYSTEM_DATA[0..8], b"ASTRO_SA");
        assert_eq!(ENCELADUS_ALBEDO_OFFSET, 16 + 6 * PIXELS + 4096);
        assert_eq!(ENCELADUS_ALBEDO_OFFSET, 3_149_840);
        assert_eq!(ENCELADUS_ELEVATION_OFFSET, 16 + 7 * PIXELS + 4096);
        assert_eq!(ENCELADUS_ELEVATION_OFFSET, 3_674_128);

        let shader = EnceladusShader::new();
        assert_eq!(shader.albedo_map.len(), PIXELS);
        assert_eq!(shader.elevation_map.len(), PIXELS);
        assert!(ENCELADUS_ELEVATION_OFFSET + PIXELS <= SATURN_SYSTEM_DATA.len());
    }

    #[test]
    fn test_sample_bilinear() {
        let mut mock_data = vec![0u8; TEX_WIDTH * TEX_HEIGHT];
        mock_data[0] = 10;
        mock_data[1] = 20;
        mock_data[TEX_WIDTH] = 30;
        mock_data[TEX_WIDTH + 1] = 40;

        let u = 0.5 / (TEX_WIDTH as f64 - 1.0);
        let v = 0.5 / (TEX_HEIGHT as f64 - 1.0);
        let sampled = EnceladusShader::sample_bilinear(&mock_data, u, v);
        assert!((sampled - 25.0).abs() < 1e-4);

        // Seamless wrap test: u = 1.0 wraps to 0.0
        let sampled_wrap = EnceladusShader::sample_bilinear(&mock_data, 1.0, 0.0);
        assert_eq!(sampled_wrap, mock_data[0] as f64);

        // Clamping at boundary v = 0.0 and v = 1.0
        let sampled_top = EnceladusShader::sample_bilinear(&mock_data, 0.0, 0.0);
        assert_eq!(sampled_top, 10.0);
        let sampled_bottom = EnceladusShader::sample_bilinear(&mock_data, 0.0, 1.0);
        assert_eq!(sampled_bottom, 0.0);
    }

    #[test]
    fn test_ultra_high_ice_albedo() {
        // Base pristine ice: purest water ice snow in the Solar System
        assert_eq!(COLOR_BASE_ICE, [250.0, 252.0, 255.0]);
        let pristine_rgb = EnceladusShader::albedo_to_rgb(255.0);
        assert_eq!(pristine_rgb, COLOR_BASE_ICE);

        // Geometric albedo luminance ~0.99
        let lum_pristine =
            (0.299 * pristine_rgb[0] + 0.587 * pristine_rgb[1] + 0.114 * pristine_rgb[2]) / 255.0;
        assert!(
            lum_pristine > 0.985,
            "Pristine ice albedo must be ~0.99, got {}",
            lum_pristine
        );

        // Tiger stripes fracture vents: cyan/blue crystalline ice signature
        let tiger_rgb = EnceladusShader::albedo_to_rgb(165.0);
        assert_eq!(tiger_rgb, COLOR_TIGER_STRIPES);
        assert!(
            tiger_rgb[2] > tiger_rgb[0] && tiger_rgb[1] > tiger_rgb[0],
            "Tiger stripes must exhibit subtle cyan-blue crystalline absorption"
        );
        let lum_tiger =
            (0.299 * tiger_rgb[0] + 0.587 * tiger_rgb[1] + 0.114 * tiger_rgb[2]) / 255.0;
        assert!(
            lum_tiger > 0.75,
            "Even dark sulci fractures have exceptionally high albedo, got {}",
            lum_tiger
        );

        // Check authentic map sampling across diverse latitudes
        let shader = EnceladusShader::new();
        let alb_equator = shader.sample_albedo(0.5, 0.5);
        assert!(alb_equator >= 170.0 && alb_equator <= 255.0);
        let alb_sp = shader.sample_albedo(0.5, 0.92); // South pole tiger stripe terrain
        assert!(alb_sp >= 170.0 && alb_sp <= 255.0);
    }

    #[test]
    fn test_tiger_stripes_bump_gradients() {
        let shader = EnceladusShader::new();

        // Sample elevation gradients across the South Polar Terrain (v in 0.85..0.96)
        // where Alexandria, Cairo, Baghdad, Damascus Sulci are situated
        let mut max_grad_mag = 0.0_f64;
        let mut non_zero_count = 0;
        for i in 0..32 {
            let u = i as f64 / 32.0;
            let (du, dv) = shader.compute_elevation_gradient(u, 0.91);
            let mag = (du * du + dv * dv).sqrt();
            if mag > 0.1 {
                non_zero_count += 1;
            }
            if mag > max_grad_mag {
                max_grad_mag = mag;
            }
        }
        assert!(
            non_zero_count > 10,
            "Tiger stripes must create distinct topographic gradients in South Polar Terrain"
        );
        assert!(
            max_grad_mag > 1.0,
            "Fissures and flanking ridges must have significant elevation relief gradient"
        );

        // Surface normal perturbation check in tiger stripes region
        let sp_lat = -75.0_f64.to_radians();
        let norm_spt = Vec3::new(sp_lat.cos(), sp_lat.sin(), 0.0).normalize();
        let perturbed_spt = shader.surface_normal(norm_spt);
        assert!(
            !perturbed_spt.x.is_nan() && !perturbed_spt.y.is_nan() && !perturbed_spt.z.is_nan()
        );
        assert!((perturbed_spt.length() - 1.0).abs() < 1e-5);
        assert!(
            perturbed_spt != norm_spt,
            "Tiger stripe topography must perturb surface normal"
        );

        // Polar singularity freedom check: North and South Poles
        let norm_np = Vec3::new(0.0, 1.0, 0.0);
        let perturbed_np = shader.surface_normal(norm_np);
        assert!(!perturbed_np.x.is_nan() && !perturbed_np.y.is_nan() && !perturbed_np.z.is_nan());
        assert!((perturbed_np.length() - 1.0).abs() < 1e-5);

        let norm_sp = Vec3::new(0.0, -1.0, 0.0);
        let perturbed_sp = shader.surface_normal(norm_sp);
        assert!(!perturbed_sp.x.is_nan() && !perturbed_sp.y.is_nan() && !perturbed_sp.z.is_nan());
        assert!((perturbed_sp.length() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = EnceladusShader::new();
        let norm = Vec3::new(1.0, 0.0, 0.0);

        // Day: Direct subsolar illumination at opposition (sun = norm, view = norm)
        let sun_day = Vec3::new(1.0, 0.0, 0.0);
        let view_day = Vec3::new(1.0, 0.0, 0.0);
        let color_day = shader.shade(norm, sun_day, view_day);
        assert!(
            color_day[0] > 200 && color_day[1] > 200 && color_day[2] > 200,
            "Day side of Enceladus must be brilliantly illuminated: got {:?}",
            color_day
        );

        // Night: Pitch black nightside with sun in opposite hemisphere
        let sun_night = Vec3::new(-1.0, 0.0, 0.0);
        let color_night = shader.shade(norm, sun_night, view_day);
        assert_eq!(
            color_night, [0, 0, 0],
            "Night side must be pitch-black [0, 0, 0]"
        );

        // Grazing night: sun pointing backwards
        let sun_grazing_night = Vec3::new(-0.2, 0.0, -0.98);
        let color_grazing = shader.shade(norm, sun_grazing_night, view_day);
        assert_eq!(
            color_grazing, [0, 0, 0],
            "Nightside past terminator must be pitch-black [0, 0, 0]"
        );

        // Grazing terminator: n_dot_l <= 0.0
        let sun_terminator = Vec3::new(0.0, 1.0, 0.0);
        let color_terminator = shader.shade(norm, sun_terminator, view_day);
        assert_eq!(
            color_terminator, [0, 0, 0],
            "Terminator boundary must be strictly pitch-black [0, 0, 0]"
        );

        // Opposition surge check: alpha = 0 (direct retroreflection) vs phase angle 45°
        let sun_angled = Vec3::new(0.7071, 0.0, 0.7071).normalize();
        let color_angled = shader.shade(norm, sun_angled, view_day);
        assert!(
            color_day[0] >= color_angled[0],
            "Opposition surge must enhance brightness at zero phase angle: {} vs {}",
            color_day[0],
            color_angled[0]
        );
    }

    #[test]
    fn test_spherical_coords() {
        let (u_eq, v_eq) = EnceladusShader::spherical_coords(Vec3::new(1.0, 0.0, 0.0));
        assert!((u_eq - 0.5).abs() < 1e-6);
        assert!((v_eq - 0.5).abs() < 1e-6);

        let (_u_np, v_np) = EnceladusShader::spherical_coords(Vec3::new(0.0, 1.0, 0.0));
        assert!((v_np - 0.0).abs() < 1e-6);

        let (_u_sp, v_sp) = EnceladusShader::spherical_coords(Vec3::new(0.0, -1.0, 0.0));
        assert!((v_sp - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_lommel_seeliger_and_opposition_surge() {
        assert_eq!(EnceladusShader::lommel_seeliger(0.0, 1.0), 0.0);
        assert_eq!(EnceladusShader::lommel_seeliger(-0.5, 0.8), 0.0);
        let ls = EnceladusShader::lommel_seeliger(1.0, 1.0);
        assert!((ls - 0.5).abs() < 1e-4);

        let surge_0 = EnceladusShader::opposition_surge(0.0);
        assert!((surge_0 - 1.45).abs() < 1e-6);

        let surge_out = EnceladusShader::opposition_surge(0.20);
        assert!((surge_out - 1.00).abs() < 1e-6);
    }
}
