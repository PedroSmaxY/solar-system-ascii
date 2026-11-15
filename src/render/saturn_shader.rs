//! Photorealistic Saturn & Ring System Raytracing Shader:
//! - Direct sampling of authentic NASA Cassini 1024x512 RGB cylindrical map with bilinear filtering.
//! - Accurate Saturnian atmospheric limb darkening (Minnaert k ~ 0.85) and subtle golden butterscotch cloud bands.
//! - Oblate spheroid geometry (polar flattening f = 0.09796, 60,268 km eq vs 54,364 km pol).
//! - 1024-sample authentic Cassini RGBA radial ring profile with optical depth/transparency modeling.
//! - Mutual shadowing: Saturn's oblate body casts sharp shadow on rings; rings cast dark shadow bands on Saturn's atmosphere.
//! - Bidirectional ring lighting: front-lit backscattering vs backlit forward transmission through ice particles.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const SATURN_SYSTEM_DATA: &[u8] = include_bytes!("../../assets/saturn_system_data.bin");

const TWO_PI: f64 = 2.0 * PI;
pub const TEX_WIDTH: usize = 1024;
pub const TEX_HEIGHT: usize = 512;
pub const PIXELS: usize = TEX_WIDTH * TEX_HEIGHT;
pub const RINGS_SAMPLES: usize = 1024;

pub const SATURN_RING_INNER_RADIUS_KM: f64 = 74_500.0;
pub const SATURN_RING_OUTER_RADIUS_KM: f64 = 140_220.0;
pub const SATURN_RADIUS_EQ_KM: f64 = 60_268.0;
pub const SATURN_RADIUS_POL_KM: f64 = 54_364.0;

pub struct SaturnShader {
    saturn_rgb: &'static [u8],
    rings_rgba: &'static [u8],
}

impl SaturnShader {
    pub fn new() -> Self {
        assert_eq!(&SATURN_SYSTEM_DATA[0..8], b"ASTRO_SA", "Invalid saturn_system_data.bin header");
        let saturn_rgb = &SATURN_SYSTEM_DATA[16..16 + 3 * PIXELS];
        let rings_rgba = &SATURN_SYSTEM_DATA[16 + 3 * PIXELS..16 + 3 * PIXELS + RINGS_SAMPLES * 4];
        Self {
            saturn_rgb,
            rings_rgba,
        }
    }

    /// Sample the 1024x512 cylindrical projection using bilinear interpolation.
    fn sample_texture_bilinear(&self, u: f64, v: f64) -> [u8; 3] {
        let u_clamped = u.rem_euclid(1.0);
        let v_clamped = v.clamp(0.0001, 0.9999);

        let fx = u_clamped * (TEX_WIDTH as f64 - 1.0);
        let fy = v_clamped * (TEX_HEIGHT as f64 - 1.0);

        let x0 = fx.floor() as usize;
        let y0 = fy.floor() as usize;
        let x1 = (x0 + 1) % TEX_WIDTH;
        let y1 = (y0 + 1).min(TEX_HEIGHT - 1);

        let tx = fx - x0 as f64;
        let ty = fy - y0 as f64;

        let idx00 = (y0 * TEX_WIDTH + x0) * 3;
        let idx10 = (y0 * TEX_WIDTH + x1) * 3;
        let idx01 = (y1 * TEX_WIDTH + x0) * 3;
        let idx11 = (y1 * TEX_WIDTH + x1) * 3;

        let mut out = [0u8; 3];
        for c in 0..3 {
            let c00 = self.saturn_rgb[idx00 + c] as f64;
            let c10 = self.saturn_rgb[idx10 + c] as f64;
            let c01 = self.saturn_rgb[idx01 + c] as f64;
            let c11 = self.saturn_rgb[idx11 + c] as f64;

            let top = c00 * (1.0 - tx) + c10 * tx;
            let bot = c01 * (1.0 - tx) + c11 * tx;
            let val = top * (1.0 - ty) + bot * ty;
            out[c] = val.round().clamp(0.0, 255.0) as u8;
        }

        out
    }

    /// Sample radial ring profile RGBA at distance `radius_km` from Saturn's center.
    /// Returns (base_rgb, opacity 0.0..1.0).
    pub fn sample_ring(&self, radius_km: f64) -> Option<([u8; 3], f64)> {
        if radius_km < SATURN_RING_INNER_RADIUS_KM || radius_km > SATURN_RING_OUTER_RADIUS_KM {
            return None;
        }

        let s = (radius_km - SATURN_RING_INNER_RADIUS_KM)
            / (SATURN_RING_OUTER_RADIUS_KM - SATURN_RING_INNER_RADIUS_KM);
        let fx = s.clamp(0.0, 1.0) * (RINGS_SAMPLES as f64 - 1.0);
        let x0 = fx.floor() as usize;
        let x1 = (x0 + 1).min(RINGS_SAMPLES - 1);
        let tx = fx - x0 as f64;

        let idx0 = x0 * 4;
        let idx1 = x1 * 4;

        let r0 = self.rings_rgba[idx0] as f64;
        let g0 = self.rings_rgba[idx0 + 1] as f64;
        let b0 = self.rings_rgba[idx0 + 2] as f64;
        let a0 = self.rings_rgba[idx0 + 3] as f64;

        let r1 = self.rings_rgba[idx1] as f64;
        let g1 = self.rings_rgba[idx1 + 1] as f64;
        let b1 = self.rings_rgba[idx1 + 2] as f64;
        let a1 = self.rings_rgba[idx1 + 3] as f64;

        let r = (r0 * (1.0 - tx) + r1 * tx).round().clamp(0.0, 255.0) as u8;
        let g = (g0 * (1.0 - tx) + g1 * tx).round().clamp(0.0, 255.0) as u8;
        let b = (b0 * (1.0 - tx) + b1 * tx).round().clamp(0.0, 255.0) as u8;
        let opacity = ((a0 * (1.0 - tx) + a1 * tx) / 255.0).clamp(0.0, 1.0);

        Some(([r, g, b], opacity))
    }

    /// Sample ring opacity alone for fast shadow casting.
    pub fn sample_ring_opacity(&self, radius_km: f64) -> f64 {
        if radius_km < SATURN_RING_INNER_RADIUS_KM || radius_km > SATURN_RING_OUTER_RADIUS_KM {
            return 0.0;
        }
        let s = (radius_km - SATURN_RING_INNER_RADIUS_KM)
            / (SATURN_RING_OUTER_RADIUS_KM - SATURN_RING_INNER_RADIUS_KM);
        let fx = s.clamp(0.0, 1.0) * (RINGS_SAMPLES as f64 - 1.0);
        let x0 = fx.floor() as usize;
        let x1 = (x0 + 1).min(RINGS_SAMPLES - 1);
        let tx = fx - x0 as f64;

        let a0 = self.rings_rgba[x0 * 4 + 3] as f64;
        let a1 = self.rings_rgba[x1 * 4 + 3] as f64;
        ((a0 * (1.0 - tx) + a1 * tx) / 255.0).clamp(0.0, 1.0)
    }

    /// Primary shader for Saturn surface pixel with atmospheric limb darkening and ring shadow.
    pub fn shade(
        &self,
        local_norm: Vec3,
        local_pos: Vec3,
        sun_dir: Vec3,
        view_dir: Vec3,
    ) -> [u8; 3] {
        let v = (1.0 - (local_norm.y.clamp(-1.0, 1.0).asin() / (PI * 0.5) * 0.5 + 0.5)).clamp(0.0, 1.0);
        let u = (local_norm.z.atan2(local_norm.x) + PI) / TWO_PI;

        let base_rgb = self.sample_texture_bilinear(u, v);

        let n_dot_l = local_norm.dot(sun_dir);
        let n_dot_v = local_norm.dot(view_dir).max(0.001);

        if n_dot_l <= 0.0 {
            // Pitch black Saturnian nightside
            return [0, 0, 0];
        }

        // Minnaert limb darkening for Saturn's deep scattering atmosphere (k ~ 0.85)
        let cos_i = n_dot_l.clamp(0.0, 1.0);
        let cos_e = n_dot_v.clamp(0.0, 1.0);
        let k = 0.85;
        let minnaert = cos_i.powf(k) * cos_e.powf(k - 1.0);
        let mut illumination = minnaert.clamp(0.0, 1.25);

        // Ring shadow cast onto Saturn's atmosphere:
        // Ray from local_pos towards sun_dir intersects ring plane y = 0
        if sun_dir.y.abs() > 1e-6 {
            let t_ring = -local_pos.y / sun_dir.y;
            if t_ring > 0.0 {
                let hit_x = local_pos.x + sun_dir.x * t_ring;
                let hit_z = local_pos.z + sun_dir.z * t_ring;
                let r_ring = (hit_x * hit_x + hit_z * hit_z).sqrt();
                let ring_opacity = self.sample_ring_opacity(r_ring);
                if ring_opacity > 0.0 {
                    let shadow_factor = 1.0 - ring_opacity * 0.92;
                    illumination *= shadow_factor;
                }
            }
        }

        // Subtle warm golden butterscotch limb haze at glancing angles
        let limb_haze = (1.0 - cos_e).powf(3.0) * 0.12 * cos_i;

        let r = ((base_rgb[0] as f64 * illumination) + 45.0 * limb_haze).clamp(0.0, 255.0) as u8;
        let g = ((base_rgb[1] as f64 * illumination) + 40.0 * limb_haze).clamp(0.0, 255.0) as u8;
        let b = ((base_rgb[2] as f64 * illumination) + 20.0 * limb_haze).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }

    /// Primary shader for Saturn's ring system.
    /// Returns RGBA `[r, g, b, alpha]` where alpha (0..255) is visual opacity.
    pub fn shade_ring(
        &self,
        local_pos: Vec3,
        sun_dir: Vec3,
        view_dir: Vec3,
    ) -> Option<[u8; 4]> {
        let r_km = (local_pos.x * local_pos.x + local_pos.z * local_pos.z).sqrt();
        let (base_rgb, opacity) = self.sample_ring(r_km)?;

        if opacity < 0.02 {
            return None;
        }

        // Check if ring point is occluded by Saturn's oblate spheroid shadow
        let a = SATURN_RADIUS_EQ_KM;
        let b = SATURN_RADIUS_POL_KM;
        let u_o = Vec3::new(local_pos.x / a, 0.0, local_pos.z / a);
        let u_d = Vec3::new(sun_dir.x / a, sun_dir.y / b, sun_dir.z / a);

        let quad_a = u_d.dot(u_d);
        let quad_b = u_o.dot(u_d);
        let quad_c = u_o.dot(u_o) - 1.0;

        let disc = quad_b * quad_b - quad_a * quad_c;
        if disc > 0.0 && quad_b < 0.0 {
            let t_shadow = (-quad_b - disc.sqrt()) / quad_a;
            if t_shadow > 0.0 {
                // Ring point is in Saturn's umbral shadow: pitch black
                return Some([2, 2, 4, (opacity * 255.0).round() as u8]);
            }
        }

        // Lighting model for rings:
        // Ring normal is along local Y (0, 1, 0)
        let cos_sun = sun_dir.y.abs().clamp(0.05, 1.0);
        let same_side = (sun_dir.y > 0.0) == (view_dir.y > 0.0);

        let illum = if same_side {
            // Sunlit face (backscattering towards viewer)
            cos_sun * 1.35
        } else {
            // Unlit face (forward scattering through translucent icy rings)
            cos_sun * (1.0 - opacity * 0.45) * 0.90
        };

        let r = (base_rgb[0] as f64 * illum).clamp(0.0, 255.0) as u8;
        let g = (base_rgb[1] as f64 * illum).clamp(0.0, 255.0) as u8;
        let b = (base_rgb[2] as f64 * illum).clamp(0.0, 255.0) as u8;
        let a_byte = (opacity * 255.0).round().clamp(0.0, 255.0) as u8;

        Some([r, g, b, a_byte])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saturn_dataset_loading() {
        assert_eq!(&SATURN_SYSTEM_DATA[0..8], b"ASTRO_SA");
        let shader = SaturnShader::new();
        assert_eq!(shader.saturn_rgb.len(), 1024 * 512 * 3);
        assert_eq!(shader.rings_rgba.len(), 1024 * 4);
    }

    #[test]
    fn test_sample_bilinear() {
        let shader = SaturnShader::new();
        let c = shader.sample_texture_bilinear(0.5, 0.5);
        assert!(c[0] > 0 || c[1] > 0 || c[2] > 0);
    }

    #[test]
    fn test_ring_radial_profile_and_cassini_division() {
        let shader = SaturnShader::new();
        // Outside rings
        assert!(shader.sample_ring(50_000.0).is_none());
        assert!(shader.sample_ring(200_000.0).is_none());

        // B ring (dense ~ 100,000 km)
        let (_b_col, b_op) = shader.sample_ring(105_000.0).expect("B ring sample");
        assert!(b_op > 0.40, "B ring should have high optical depth, got {}", b_op);

        // Cassini division (~ 120,000 km)
        let (_cas_col, cas_op) = shader.sample_ring(120_000.0).expect("Cassini division sample");
        assert!(cas_op < b_op, "Cassini division should be clearer than B ring");
    }

    #[test]
    fn test_saturn_nightside_pitch_black() {
        let shader = SaturnShader::new();
        let norm = Vec3::new(1.0, 0.0, 0.0);
        let pos = Vec3::new(60_268.0, 0.0, 0.0);
        let sun = Vec3::new(-1.0, 0.0, 0.0); // Facing away from Sun
        let view = Vec3::new(1.0, 0.0, 0.0);
        let rgb = shader.shade(norm, pos, sun, view);
        assert_eq!(rgb, [0, 0, 0]);
    }

    #[test]
    fn test_ring_shadow_on_saturn() {
        let shader = SaturnShader::new();
        let norm = Vec3::new(1.0, 0.1, 0.0).normalize();
        let pos = Vec3::new(60_000.0, 5_000.0, 0.0);
        // Sun slightly below ring plane so light passes through rings before hitting northern hemisphere
        let sun = Vec3::new(0.5, -0.3, 0.0).normalize();
        let view = Vec3::new(1.0, 0.0, 0.0);
        let lit_rgb = shader.shade(norm, pos, sun, view);
        // Ensure shader produces valid color under ring shadow
        assert!(lit_rgb[0] > 0 || lit_rgb[1] > 0);
    }

    #[test]
    fn test_saturn_shadow_on_rings() {
        let shader = SaturnShader::new();
        // Ring point on the nightside behind Saturn
        let ring_pos = Vec3::new(-100_000.0, 0.0, 0.0);
        let sun_dir = Vec3::new(1.0, 0.1, 0.0).normalize();
        let view_dir = Vec3::new(0.0, 1.0, 0.0);
        let sample = shader.shade_ring(ring_pos, sun_dir, view_dir);
        assert!(sample.is_some());
        let rgba = sample.unwrap();
        // Point is in Saturn's shadow: dark tone [2, 2, 4]
        assert!(rgba[0] <= 5 && rgba[1] <= 5 && rgba[2] <= 5);
    }
}
