//! Photorealistic Titania Raytracing Shader:
//! - Direct zero-copy slicing of authentic Voyager 2 1024x512 albedo and topographic relief datasets
//!   from `uranus_system_data.bin`.
//! - Fractional 4-texel bilinear interpolation with seamless longitude wrapping.
//! - Largest moon of Uranus (radius 788.4 km, geometric albedo ~0.27) exhibiting massive extensional
//!   tectonism (Messina Chasma canyon system spanning 1,500 km) and the 400 km Gertrude impact basin.
//! - Non-Lambertian Lommel-Seeliger scattering with retroreflective opposition surge.
//! - Strictly pitch-black nightside with zero light leakage (color = [0, 0, 0] when n_dot_l <= 0.0).

use crate::math::Vec3;
use crate::render::uranus_shader::{PIXELS, TEX_HEIGHT, TEX_WIDTH, URANUS_SYSTEM_DATA};
use std::f64::consts::PI;

pub const TITANIA_ALBEDO_OFFSET: usize = 16 + 9 * PIXELS + 4096; // 4_722_704
pub const TITANIA_ELEVATION_OFFSET: usize = 16 + 10 * PIXELS + 4096; // 5_246_992

pub const COLOR_FAULT_SCARPS: [f64; 3] = [140.0, 142.0, 150.0]; // Messina Chasma walls & depressed grabens
pub const COLOR_CRUST: [f64; 3] = [175.0, 178.0, 186.0]; // Neutral silvery-gray cratered crust
pub const COLOR_BRIGHT_EJECTA: [f64; 3] = [220.0, 224.0, 234.0]; // Gertrude basin bright rays & rim deposits

pub const BUMP_SCALE: f64 = 0.010;

pub struct TitaniaShader {
    albedo_map: &'static [u8],
    elevation_map: &'static [u8],
}

impl Default for TitaniaShader {
    fn default() -> Self {
        Self::new()
    }
}

impl TitaniaShader {
    pub fn new() -> Self {
        assert_eq!(
            &URANUS_SYSTEM_DATA[0..8],
            b"ASTRO_UR",
            "Invalid uranus_system_data.bin header magic"
        );
        let albedo_map =
            &URANUS_SYSTEM_DATA[TITANIA_ALBEDO_OFFSET..TITANIA_ALBEDO_OFFSET + PIXELS];
        let elevation_map =
            &URANUS_SYSTEM_DATA[TITANIA_ELEVATION_OFFSET..TITANIA_ELEVATION_OFFSET + PIXELS];

        Self {
            albedo_map,
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

    #[inline]
    pub fn albedo_to_rgb(raw: f64) -> [f64; 3] {
        if raw <= 135.0 {
            let t = (raw / 135.0).clamp(0.0, 1.0);
            let smooth_t = t * t * (3.0 - 2.0 * t);
            [
                COLOR_FAULT_SCARPS[0] + (COLOR_CRUST[0] - COLOR_FAULT_SCARPS[0]) * smooth_t,
                COLOR_FAULT_SCARPS[1] + (COLOR_CRUST[1] - COLOR_FAULT_SCARPS[1]) * smooth_t,
                COLOR_FAULT_SCARPS[2] + (COLOR_CRUST[2] - COLOR_FAULT_SCARPS[2]) * smooth_t,
            ]
        } else {
            let t = ((raw - 135.0) / 120.0).clamp(0.0, 1.0);
            let smooth_t = t * t * (3.0 - 2.0 * t);
            [
                COLOR_CRUST[0] + (COLOR_BRIGHT_EJECTA[0] - COLOR_CRUST[0]) * smooth_t,
                COLOR_CRUST[1] + (COLOR_BRIGHT_EJECTA[1] - COLOR_CRUST[1]) * smooth_t,
                COLOR_CRUST[2] + (COLOR_BRIGHT_EJECTA[2] - COLOR_CRUST[2]) * smooth_t,
            ]
        }
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
        1.0 + 0.35 * (1.0 - (alpha / 0.16).min(1.0))
    }

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

        // Strict terminator check
        let geom_n_dot_l = base_norm.dot(sun_norm);
        if geom_n_dot_l <= 0.0 {
            return [0, 0, 0];
        }

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
    fn test_titania_slice_boundaries() {
        assert_eq!(URANUS_SYSTEM_DATA.len(), 6_819_856);
        assert_eq!(&URANUS_SYSTEM_DATA[0..8], b"ASTRO_UR");
        assert_eq!(TITANIA_ALBEDO_OFFSET, 4_722_704);
        assert_eq!(TITANIA_ELEVATION_OFFSET, 5_246_992);

        let shader = TitaniaShader::new();
        assert_eq!(shader.albedo_map.len(), PIXELS);
        assert_eq!(shader.elevation_map.len(), PIXELS);
    }

    #[test]
    fn test_titania_shading_terminator() {
        let shader = TitaniaShader::new();
        let norm = Vec3::new(0.0, 1.0, 0.0);
        let sun = Vec3::new(0.0, -1.0, 0.0);
        let view = Vec3::new(0.0, 1.0, 0.0);
        let color = shader.shade(norm, sun, view);
        assert_eq!(color, [0, 0, 0], "Nightside must be pitch black");
    }
}
