//! Photorealistic Charon Raytracing Shader:
//! - Direct zero-copy slicing of authentic NASA New Horizons 1024x512 24-bit RGB and elevation datasets
//!   from `pluto_system_data.bin`.
//! - Features: Mordor Macula reddish tholin polar hood, Serenity Chasma equatorial tectonic canyon system /
//!   fault scarps, Vulcan Planitia smooth plains, and Oz Terra cratered highlands.
//! - Fractional 4-texel bilinear RGB and relief sampling with seamless longitude wrapping.
//! - Non-Lambertian Lommel-Seeliger scattering with retroreflective opposition surge.
//! - Strictly pitch-black nightside with zero light leakage (color = [0, 0, 0] when n_dot_l <= 0.0).

use crate::math::Vec3;
use crate::render::pluto_shader::{
    CHARON_ELEVATION_OFFSET, CHARON_RGB_OFFSET, PIXELS, PLUTO_SYSTEM_DATA, TEX_HEIGHT, TEX_WIDTH,
};
use std::f64::consts::PI;

pub const BUMP_SCALE: f64 = 0.012;

pub struct CharonShader {
    rgb_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl Default for CharonShader {
    fn default() -> Self {
        Self::new()
    }
}

impl CharonShader {
    pub fn new() -> Self {
        assert_eq!(
            &PLUTO_SYSTEM_DATA[0..8],
            b"ASTRO_PL",
            "Invalid pluto_system_data.bin header magic"
        );
        let rgb_map = &PLUTO_SYSTEM_DATA[CHARON_RGB_OFFSET..CHARON_RGB_OFFSET + 3 * PIXELS];
        let elevation_map =
            &PLUTO_SYSTEM_DATA[CHARON_ELEVATION_OFFSET..CHARON_ELEVATION_OFFSET + PIXELS];

        Self {
            rgb_map,
            elevation_map,
        }
    }

    #[inline(always)]
    pub fn spherical_coords(norm: Vec3) -> (f64, f64) {
        let v = (1.0 - (norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = (norm.z.atan2(norm.x) + PI) / (2.0 * PI);
        (u, v)
    }

    #[inline(always)]
    pub fn sample_elevation(&self, u: f64, v: f64) -> f64 {
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

        let p00 = self.elevation_map[y0 * TEX_WIDTH + x0] as f64;
        let p10 = self.elevation_map[y0 * TEX_WIDTH + x1] as f64;
        let p01 = self.elevation_map[y1 * TEX_WIDTH + x0] as f64;
        let p11 = self.elevation_map[y1 * TEX_WIDTH + x1] as f64;

        let top = p00 * (1.0 - tx) + p10 * tx;
        let bot = p01 * (1.0 - tx) + p11 * tx;
        top * (1.0 - ty) + bot * ty
    }

    #[inline(always)]
    pub fn sample_rgb(&self, u: f64, v: f64) -> [f64; 3] {
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

        let idx00 = (y0 * TEX_WIDTH + x0) * 3;
        let idx10 = (y0 * TEX_WIDTH + x1) * 3;
        let idx01 = (y1 * TEX_WIDTH + x0) * 3;
        let idx11 = (y1 * TEX_WIDTH + x1) * 3;

        let w00 = (1.0 - tx) * (1.0 - ty);
        let w10 = tx * (1.0 - ty);
        let w01 = (1.0 - tx) * ty;
        let w11 = tx * ty;

        [
            self.rgb_map[idx00] as f64 * w00
                + self.rgb_map[idx10] as f64 * w10
                + self.rgb_map[idx01] as f64 * w01
                + self.rgb_map[idx11] as f64 * w11,
            self.rgb_map[idx00 + 1] as f64 * w00
                + self.rgb_map[idx10 + 1] as f64 * w10
                + self.rgb_map[idx01 + 1] as f64 * w01
                + self.rgb_map[idx11 + 1] as f64 * w11,
            self.rgb_map[idx00 + 2] as f64 * w00
                + self.rgb_map[idx10 + 2] as f64 * w10
                + self.rgb_map[idx01 + 2] as f64 * w01
                + self.rgb_map[idx11 + 2] as f64 * w11,
        ]
    }

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

    pub fn surface_normal(&self, norm: Vec3) -> Vec3 {
        let (u, v) = Self::spherical_coords(norm);
        let (du, dv) = self.compute_elevation_gradient(u, v);

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
    pub fn lommel_seeliger(mu_0: f64, mu: f64) -> f64 {
        if mu_0 <= 0.0 {
            0.0
        } else {
            mu_0 / (mu_0 + mu.max(0.001))
        }
    }

    #[inline(always)]
    pub fn opposition_surge(alpha: f64) -> f64 {
        1.0 + 0.50 * (1.0 - (alpha / 0.15).min(1.0))
    }

    pub fn shade(
        &self,
        normal: Vec3,
        sun_dir: Vec3,
        view_dir: Vec3,
        distance_to_sun_au: f64,
    ) -> [u8; 3] {
        let (u, v) = Self::spherical_coords(normal);
        let bumped_normal = self.surface_normal(normal);

        let mu_0 = bumped_normal.dot(sun_dir);
        let mu = bumped_normal.dot(view_dir);

        // Strict nightside test on geometry normal to avoid light leaks
        let geom_mu_0 = normal.dot(sun_dir);
        if geom_mu_0 <= 0.0 || mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        // Photometric scattering: Lommel-Seeliger + Opposition surge
        let ls = Self::lommel_seeliger(mu_0, mu);
        let cos_phase = sun_dir.dot(view_dir).clamp(-1.0, 1.0);
        let phase_angle = cos_phase.acos();
        let surge = Self::opposition_surge(phase_angle);

        // Inverse square distance attenuation from Sun (~39.5 AU baseline)
        let solar_scale = (39.5 / distance_to_sun_au.max(1.0)).powi(2).clamp(0.5, 2.5);

        // Sample authentic RGB
        let base_rgb = self.sample_rgb(u, v);

        // Diffuse illumination with limb darkening
        let limb = (mu.max(0.0).sqrt() * 0.35 + 0.65).clamp(0.0, 1.0);
        let illumination = (ls * 2.1 * surge * solar_scale * limb).clamp(0.0, 2.0);

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
    fn test_charon_shader_nightside() {
        let shader = CharonShader::new();
        let normal = Vec3::new(1.0, 0.0, 0.0);
        let sun_dir = Vec3::new(-1.0, 0.0, 0.0);
        let view_dir = Vec3::new(0.0, 0.0, 1.0);
        let color = shader.shade(normal, sun_dir, view_dir, 39.5);
        assert_eq!(color, [0, 0, 0]);
    }

    #[test]
    fn test_charon_shader_dayside_bright() {
        let shader = CharonShader::new();
        let normal = Vec3::new(0.0, 0.0, 1.0);
        let sun_dir = Vec3::new(0.0, 0.0, 1.0);
        let view_dir = Vec3::new(0.0, 0.0, 1.0);
        let color = shader.shade(normal, sun_dir, view_dir, 39.5);
        assert!(color[0] > 0 || color[1] > 0 || color[2] > 0);
    }
}
