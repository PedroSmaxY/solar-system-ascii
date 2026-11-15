//! Photorealistic Iapetus Surface Raytracing Shader:
//! - Direct zero-copy slicing of authentic NASA Cassini 1024x512 Iapetus albedo and relief dataset
//!   from `assets/saturn_system_data.bin`.
//! - Fractional 4-texel bilinear interpolation without polar singularities.
//! - Extreme "Yin-Yang" albedo dichotomy:
//!   * Leading hemisphere (Cassini Regio): ultra-low albedo ~0.04, coal-black with dark reddish-brown organic hue ([38, 32, 28]).
//!   * Trailing hemisphere (Roncevaux Terra): brilliant water ice crust, albedo ~0.60 ([225, 230, 240]).
//!   * Extreme contrast ratio > 10x between both hemispheres.
//! - Colossal 20 km high equatorial mountain ridge and ancient cratered topography.
//! - Topographic normal perturbation via central finite differences of the elevation map.
//! - Non-Lambertian Lommel-Seeliger scattering with opposition surge and pitch-black nightside.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const TEX_WIDTH: usize = 1024;
pub const TEX_HEIGHT: usize = 512;
pub const PIXELS: usize = TEX_WIDTH * TEX_HEIGHT;

pub const IAPETUS_ALBEDO_OFFSET: usize = 16 + 10 * PIXELS + 4096; // 5_246_992
pub const IAPETUS_ELEVATION_OFFSET: usize = 16 + 11 * PIXELS + 4096; // 5_771_280

pub const BUMP_SCALE: f64 = 0.006;
pub const EQUATORIAL_RIDGE_HEIGHT_KM: f64 = 20.0;
pub const RIDGE_ELEVATION_SCALE: f64 = 35.0;

pub const COLOR_CASSINI: [f64; 3] = [38.0, 32.0, 28.0];
pub const COLOR_TRANSITION: [f64; 3] = [115.0, 110.0, 105.0];
pub const COLOR_RONCEVAUX: [f64; 3] = [225.0, 230.0, 240.0];

pub const ALBEDO_CASSINI: f64 = 0.04;
pub const ALBEDO_RONCEVAUX: f64 = 0.60;

#[derive(Clone, Copy, Debug)]
pub struct IapetusShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl IapetusShader {
    /// Initialize IapetusShader with zero-copy slices from `saturn_system_data.bin`.
    pub fn new() -> Self {
        let system_data = crate::render::saturn_shader::SATURN_SYSTEM_DATA;
        assert!(
            system_data.len() >= IAPETUS_ELEVATION_OFFSET + PIXELS,
            "saturn_system_data.bin buffer too small for Iapetus datasets"
        );
        assert_eq!(
            &system_data[0..8],
            b"ASTRO_SA",
            "Invalid saturn_system_data.bin header format"
        );
        let albedo_map = &system_data[IAPETUS_ALBEDO_OFFSET..IAPETUS_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &system_data[IAPETUS_ELEVATION_OFFSET..IAPETUS_ELEVATION_OFFSET + PIXELS];
        Self {
            albedo_map,
            elevation_map,
        }
    }

    /// Construct with explicit static slices (useful for mocking and testing).
    pub fn from_slices(albedo_map: &'static [u8], elevation_map: &'static [u8]) -> Self {
        Self {
            albedo_map,
            elevation_map,
        }
    }

    #[inline(always)]
    pub fn albedo_map(&self) -> &'static [u8] {
        self.albedo_map
    }

    #[inline(always)]
    pub fn elevation_map(&self) -> &'static [u8] {
        self.elevation_map
    }

    /// Calculate spherical UV coordinates from local surface normal (x, y, z) where y is polar axis.
    /// v: 0.0 at North Pole, 0.5 at Equator, 1.0 at South Pole.
    /// u: [0.0, 1.0) longitude coordinate.
    #[inline(always)]
    pub fn compute_uv(local_norm: Vec3) -> (f64, f64) {
        let norm = if local_norm.length_squared() > 1e-12 {
            local_norm.normalize()
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let v = (1.0 - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = ((norm.z.atan2(norm.x) + PI) / (2.0 * PI)).rem_euclid(1.0);
        (u, v)
    }

    #[inline(always)]
    pub fn spherical_coords(local_norm: Vec3) -> (f64, f64) {
        Self::compute_uv(local_norm)
    }

    /// Fractional 4-sample bilinear interpolation on 1024x512 texture data.
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
    pub fn sample_albedo_raw(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.albedo_map, u, v)
    }

    #[inline(always)]
    pub fn sample_albedo_bilinear(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.albedo_map, u, v)
    }

    #[inline(always)]
    pub fn sample_elevation_raw(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.elevation_map, u, v)
    }

    #[inline(always)]
    pub fn sample_elevation_bilinear(&self, u: f64, v: f64) -> f64 {
        Self::sample_bilinear(self.elevation_map, u, v)
    }

    /// Map raw 8-bit dataset value to physical astronomical albedo (Bond/geometric):
    /// - Cassini Regio: ~0.04 (dark organic lag deposit)
    /// - Roncevaux Terra: ~0.60 (bright water ice crust)
    /// - Dichotomy contrast ratio > 10x.
    #[inline(always)]
    pub fn raw_to_physical_albedo(raw: f64) -> f64 {
        if raw <= 75.0 {
            let t = ((raw - 45.0) / 30.0).clamp(0.0, 1.0);
            0.035 + 0.010 * t
        } else if raw >= 170.0 {
            let t = ((raw - 170.0) / 40.0).clamp(0.0, 1.0);
            0.56 + 0.06 * t
        } else {
            let t = (raw - 75.0) / (170.0 - 75.0);
            0.045 + (0.56 - 0.045) * t
        }
    }

    /// Sample physical astronomical albedo at spherical coordinates (u, v).
    #[inline(always)]
    pub fn physical_albedo(&self, u: f64, v: f64) -> f64 {
        let raw = self.sample_albedo_raw(u, v);
        Self::raw_to_physical_albedo(raw)
    }

    /// Sample physical albedo at (u, v).
    #[inline(always)]
    pub fn sample_albedo(&self, u: f64, v: f64) -> f64 {
        self.physical_albedo(u, v)
    }

    /// Convert raw 8-bit albedo to calibrated planetary RGB:
    /// - Cassini Regio: dark coal-black with organic reddish/dark brown hue [38, 32, 28]
    /// - Transition zone: muted regolith [115, 110, 105]
    /// - Roncevaux Terra: bright pristine water ice [225, 230, 240]
    #[inline(always)]
    pub fn albedo_to_rgb(raw: f64) -> [f64; 3] {
        if raw <= 80.0 {
            let t = ((raw - 45.0) / 35.0).clamp(0.0, 1.0);
            [
                COLOR_CASSINI[0] * (0.85 + 0.15 * t),
                COLOR_CASSINI[1] * (0.85 + 0.15 * t),
                COLOR_CASSINI[2] * (0.85 + 0.15 * t),
            ]
        } else if raw <= 140.0 {
            let t = (raw - 80.0) / (140.0 - 80.0);
            [
                COLOR_CASSINI[0] + (COLOR_TRANSITION[0] - COLOR_CASSINI[0]) * t,
                COLOR_CASSINI[1] + (COLOR_TRANSITION[1] - COLOR_CASSINI[1]) * t,
                COLOR_CASSINI[2] + (COLOR_TRANSITION[2] - COLOR_CASSINI[2]) * t,
            ]
        } else {
            let t = ((raw - 140.0) / (180.0 - 140.0)).clamp(0.0, 1.0);
            [
                COLOR_TRANSITION[0] + (COLOR_RONCEVAUX[0] - COLOR_TRANSITION[0]) * t,
                COLOR_TRANSITION[1] + (COLOR_RONCEVAUX[1] - COLOR_TRANSITION[1]) * t,
                COLOR_TRANSITION[2] + (COLOR_RONCEVAUX[2] - COLOR_TRANSITION[2]) * t,
            ]
        }
    }

    #[inline(always)]
    pub fn sample_albedo_rgb(&self, u: f64, v: f64) -> [f64; 3] {
        let raw = self.sample_albedo_raw(u, v);
        Self::albedo_to_rgb(raw)
    }

    /// Elevation profile of Iapetus's colossal ~20 km high equatorial ridge:
    /// Centered exactly at v = 0.5 (equator), decaying with ~2.7° latitude width (sigma ~ 0.015).
    #[inline(always)]
    pub fn equatorial_ridge_elevation(v: f64) -> f64 {
        let dv = v - 0.5;
        let sigma = 0.015;
        RIDGE_ELEVATION_SCALE * (-0.5 * (dv / sigma) * (dv / sigma)).exp()
    }

    /// Combined surface elevation in relief units (dataset craters + equatorial ridge).
    #[inline(always)]
    pub fn surface_elevation(&self, u: f64, v: f64) -> f64 {
        let base = self.sample_elevation_bilinear(u, v);
        let ridge = Self::equatorial_ridge_elevation(v);
        base + ridge
    }

    /// Central finite differences gradient (du, dv) in elevation.
    #[inline(always)]
    pub fn compute_elevation_gradient(&self, u: f64, v: f64) -> (f64, f64) {
        let du_step = 1.0 / TEX_WIDTH as f64;
        let dv_step = 1.0 / TEX_HEIGHT as f64;

        let h_u1 = self.surface_elevation((u + du_step).rem_euclid(1.0), v);
        let h_u0 = self.surface_elevation((u - du_step).rem_euclid(1.0), v);
        let h_v1 = self.surface_elevation(u, (v + dv_step).clamp(0.0, 1.0));
        let h_v0 = self.surface_elevation(u, (v - dv_step).clamp(0.0, 1.0));

        let du = (h_u1 - h_u0) * 0.5;
        let dv = (h_v1 - h_v0) * 0.5;
        (du, dv)
    }

    /// Bump-mapped surface normal perturbed by ancient impact craters and colossal equatorial ridge.
    pub fn perturbed_normal(&self, local_norm: Vec3) -> Vec3 {
        let norm = if local_norm.length_squared() > 1e-12 {
            local_norm.normalize()
        } else {
            return Vec3::new(0.0, 1.0, 0.0);
        };

        let (u, v) = Self::compute_uv(norm);

        // Singularity-free tangent & bitangent frame
        let xz_len = (norm.x * norm.x + norm.z * norm.z).sqrt();
        let tangent = if xz_len > 1e-6 {
            Vec3::new(-norm.z / xz_len, 0.0, norm.x / xz_len)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let bitangent = norm.cross(tangent).normalize();

        let (du, dv) = self.compute_elevation_gradient(u, v);

        (norm - tangent * (du * BUMP_SCALE) - bitangent * (dv * BUMP_SCALE)).normalize()
    }

    #[inline(always)]
    pub fn surface_normal(&self, local_norm: Vec3) -> Vec3 {
        self.perturbed_normal(local_norm)
    }

    /// Lommel-Seeliger non-Lambertian reflectance: mu_0 / (mu_0 + mu).
    #[inline(always)]
    pub fn lommel_seeliger(mu_0: f64, mu: f64) -> f64 {
        if mu_0 <= 0.0 {
            0.0
        } else {
            let denom = mu_0 + mu.max(0.001);
            if denom > 1e-9 {
                mu_0 / denom
            } else {
                0.0
            }
        }
    }

    /// Retroreflective opposition surge surge(alpha):
    /// At zero phase angle (alpha ~ 0), sharp backscattering peak.
    #[inline(always)]
    pub fn opposition_surge(alpha: f64) -> f64 {
        let half_width = 0.17; // ~10 degrees
        1.0 + 0.45 * (1.0 - (alpha / half_width).min(1.0))
    }

    /// Non-Lambertian Lommel-Seeliger scattering with opposition surge.
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
        let ls = Self::lommel_seeliger(mu_0, mu);

        let cos_alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let surge = Self::opposition_surge(alpha);

        (ls * surge * 1.85).clamp(0.0, 1.5)
    }

    /// Primary photorealistic raytracing shader for Iapetus.
    /// - `local_norm`: local surface normal vector in Iapetus rotating frame.
    /// - `local_sun`: direction towards the Sun in Iapetus rotating frame.
    /// - `local_view`: direction towards the observer/camera in Iapetus rotating frame.
    pub fn shade(&self, local_norm: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        if local_norm.length_squared() < 1e-12
            || local_sun.length_squared() < 1e-12
            || local_view.length_squared() < 1e-12
        {
            return [0, 0, 0];
        }

        let norm = local_norm.normalize();
        let sun_norm = local_sun.normalize();
        let view_norm = local_view.normalize();

        // Geometric terminator check: strictly pitch black if n_dot_l <= 0.0
        let n_dot_l = norm.dot(sun_norm);
        if n_dot_l <= 0.0 {
            return [0, 0, 0];
        }

        let perturbed_norm = self.perturbed_normal(norm);

        let mu_0 = perturbed_norm.dot(sun_norm);
        if mu_0 <= 0.0 {
            // Shadowed by topographic relief / pitch-black nightside
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

impl Default for IapetusShader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iapetus_slice_boundaries() {
        assert_eq!(TEX_WIDTH, 1024);
        assert_eq!(TEX_HEIGHT, 512);
        assert_eq!(PIXELS, 524_288);

        assert_eq!(IAPETUS_ALBEDO_OFFSET, 16 + 10 * PIXELS + 4096);
        assert_eq!(IAPETUS_ALBEDO_OFFSET, 5_246_992);
        assert_eq!(IAPETUS_ELEVATION_OFFSET, 16 + 11 * PIXELS + 4096);
        assert_eq!(IAPETUS_ELEVATION_OFFSET, 5_771_280);

        let system_data = crate::render::saturn_shader::SATURN_SYSTEM_DATA;
        assert_eq!(&system_data[0..8], b"ASTRO_SA");
        assert!(system_data.len() >= IAPETUS_ELEVATION_OFFSET + PIXELS);
        assert_eq!(system_data.len(), 7_344_144);

        let shader = IapetusShader::new();
        assert_eq!(shader.albedo_map().len(), PIXELS);
        assert_eq!(shader.elevation_map().len(), PIXELS);
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
        let sampled = IapetusShader::sample_bilinear(&mock_data, u, v);
        assert!((sampled - 25.0).abs() < 1e-4);

        // Exact corners
        let c00 = IapetusShader::sample_bilinear(&mock_data, 0.0, 0.0);
        assert_eq!(c00, 10.0);
        let c10 = IapetusShader::sample_bilinear(&mock_data, 1.0 / (TEX_WIDTH as f64 - 1.0), 0.0);
        assert_eq!(c10, 20.0);

        // Wrap around for u
        let sampled_wrap = IapetusShader::sample_bilinear(&mock_data, 1.0, 0.0);
        assert_eq!(sampled_wrap, 10.0);
        let sampled_neg_wrap = IapetusShader::sample_bilinear(&mock_data, -1.0, 0.0);
        assert_eq!(sampled_neg_wrap, 10.0);

        // Clamping for v
        let clamped_v_low = IapetusShader::sample_bilinear(&mock_data, 0.0, -0.5);
        assert_eq!(clamped_v_low, 10.0);
    }

    #[test]
    fn test_yin_yang_albedo_dichotomy() {
        let shader = IapetusShader::new();

        // 1. Sample Cassini Regio (leading hemisphere center)
        // Local norm facing -z (u = 0.25, v = 0.5)
        let norm_cassini = Vec3::new(0.0, 0.0, -1.0);
        let (u_c, v_c) = IapetusShader::compute_uv(norm_cassini);
        let alb_c = shader.physical_albedo(u_c, v_c);
        let raw_c = shader.sample_albedo_raw(u_c, v_c);
        let color_c = IapetusShader::albedo_to_rgb(raw_c);

        // Verify Cassini Regio low albedo ~0.04
        assert!(
            alb_c >= 0.03 && alb_c <= 0.05,
            "Cassini Regio physical albedo should be ~0.04, got {}",
            alb_c
        );
        // Dark coal-black with organic reddish/dark brown hue
        assert!(
            color_c[0] > color_c[1] && color_c[1] > color_c[2],
            "Cassini Regio must exhibit organic reddish/brown tint: {:?}",
            color_c
        );
        assert!(
            color_c[0] < 50.0 && color_c[1] < 45.0 && color_c[2] < 40.0,
            "Cassini Regio should be coal-dark: {:?}",
            color_c
        );

        // 2. Sample Roncevaux Terra (trailing hemisphere bright terrain)
        // Direction with high albedo around u ~ 0.657, v = 0.5
        let norm_roncevaux = Vec3::new(0.552, 0.0, 0.834).normalize();
        let (u_r, v_r) = IapetusShader::compute_uv(norm_roncevaux);
        let alb_r = shader.physical_albedo(u_r, v_r);
        let raw_r = shader.sample_albedo_raw(u_r, v_r);
        let color_r = IapetusShader::albedo_to_rgb(raw_r);

        // Verify Roncevaux Terra bright ice albedo ~0.60
        assert!(
            alb_r >= 0.50 && alb_r <= 0.65,
            "Roncevaux Terra physical albedo should be ~0.60, got {}",
            alb_r
        );
        // Brilliant ice color
        assert!(
            color_r[0] >= 200.0 && color_r[1] >= 200.0 && color_r[2] >= 200.0,
            "Roncevaux Terra should be brilliant ice white: {:?}",
            color_r
        );

        // 3. Verify albedo contrast ratio > 10x
        let contrast_ratio = alb_r / alb_c;
        assert!(
            contrast_ratio > 10.0,
            "Yin-Yang dichotomy contrast ratio must exceed 10x, got {:.2}x",
            contrast_ratio
        );

        // 4. Verify shaded luminance contrast under subsolar illumination
        let shade_c = shader.shade(norm_cassini, norm_cassini, norm_cassini);
        let shade_r = shader.shade(norm_roncevaux, norm_roncevaux, norm_roncevaux);
        let lum_c = 0.299 * shade_c[0] as f64 + 0.587 * shade_c[1] as f64 + 0.114 * shade_c[2] as f64;
        let lum_r = 0.299 * shade_r[0] as f64 + 0.587 * shade_r[1] as f64 + 0.114 * shade_r[2] as f64;
        assert!(
            lum_r > lum_c * 4.0,
            "Shaded luminance of Roncevaux Terra ({}) must strongly exceed Cassini Regio ({})",
            lum_r,
            lum_c
        );
    }

    #[test]
    fn test_equatorial_ridge_bump_gradients() {
        let shader = IapetusShader::new();

        // 1. Equatorial ridge elevation profile peak
        let h_crest = IapetusShader::equatorial_ridge_elevation(0.5);
        let h_north_flank = IapetusShader::equatorial_ridge_elevation(0.48);
        let h_south_flank = IapetusShader::equatorial_ridge_elevation(0.52);
        assert!(h_crest > h_north_flank, "Crest must be higher than north flank");
        assert!(h_crest > h_south_flank, "Crest must be higher than south flank");
        assert!((h_crest - RIDGE_ELEVATION_SCALE).abs() < 1e-4);

        // 2. Finite difference gradient on north slope vs south slope
        let (_du_n, dv_n) = shader.compute_elevation_gradient(0.5, 0.49);
        let (_du_s, dv_s) = shader.compute_elevation_gradient(0.5, 0.51);
        assert!(
            dv_n > 0.0,
            "North slope dv must be positive (sloping upward towards equator), got {}",
            dv_n
        );
        assert!(
            dv_s < 0.0,
            "South slope dv must be negative (sloping downward away from equator), got {}",
            dv_s
        );

        // 3. Verify normal tilt deflections
        let lat_north = 2.0_f64.to_radians();
        let norm_north = Vec3::new(lat_north.cos(), lat_north.sin(), 0.0).normalize();
        let pert_north = shader.perturbed_normal(norm_north);
        assert!(
            pert_north.y > norm_north.y,
            "North slope normal y ({}) must be perturbed northward relative to geom ({})",
            pert_north.y,
            norm_north.y
        );

        let lat_south = -2.0_f64.to_radians();
        let norm_south = Vec3::new(lat_south.cos(), lat_south.sin(), 0.0).normalize();
        let pert_south = shader.perturbed_normal(norm_south);
        assert!(
            pert_south.y < norm_south.y,
            "South slope normal y ({}) must be perturbed southward relative to geom ({})",
            pert_south.y,
            norm_south.y
        );

        // 4. Pole singularity checks
        let north_pole = Vec3::new(0.0, 1.0, 0.0);
        let south_pole = Vec3::new(0.0, -1.0, 0.0);
        let pert_np = shader.perturbed_normal(north_pole);
        let pert_sp = shader.perturbed_normal(south_pole);

        assert!(!pert_np.x.is_nan() && !pert_np.y.is_nan() && !pert_np.z.is_nan());
        assert!(!pert_sp.x.is_nan() && !pert_sp.y.is_nan() && !pert_sp.z.is_nan());
        assert!((pert_np.length() - 1.0).abs() < 1e-4);
        assert!((pert_sp.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_day_night_illumination() {
        let shader = IapetusShader::new();
        let norm = Vec3::new(1.0, 0.0, 0.0);

        // 1. Direct subsolar illumination (Day)
        let sun_day = Vec3::new(1.0, 0.0, 0.0);
        let view_day = Vec3::new(1.0, 0.0, 0.0);
        let color_day = shader.shade(norm, sun_day, view_day);
        assert!(
            color_day[0] > 0 || color_day[1] > 0 || color_day[2] > 0,
            "Daylit surface must produce non-zero color, got {:?}",
            color_day
        );

        // 2. Night side (sun opposite to normal, n_dot_l < 0)
        let sun_night = Vec3::new(-1.0, 0.0, 0.0);
        let color_night = shader.shade(norm, sun_night, view_day);
        assert_eq!(
            color_night, [0, 0, 0],
            "Nightside must be pitch-black [0, 0, 0]"
        );

        // 3. Grazing nightside / past terminator (n_dot_l <= 0)
        let sun_past_terminator = Vec3::new(-0.01, 0.0, 0.9999).normalize();
        let color_past = shader.shade(norm, sun_past_terminator, view_day);
        assert_eq!(
            color_past, [0, 0, 0],
            "Past terminator must be pitch-black [0, 0, 0]"
        );

        // 4. Exact terminator (n_dot_l == 0)
        let sun_terminator = Vec3::new(0.0, 0.0, 1.0);
        let color_term = shader.shade(norm, sun_terminator, view_day);
        assert_eq!(
            color_term, [0, 0, 0],
            "Exact terminator (n_dot_l == 0.0) must be [0, 0, 0]"
        );

        // 5. Zero / degenerate input vectors
        assert_eq!(shader.shade(Vec3::new(0.0, 0.0, 0.0), sun_day, view_day), [0, 0, 0]);
        assert_eq!(shader.shade(norm, Vec3::new(0.0, 0.0, 0.0), view_day), [0, 0, 0]);
        assert_eq!(shader.shade(norm, sun_day, Vec3::new(0.0, 0.0, 0.0)), [0, 0, 0]);
    }
}
