//! Parallel analytical Ray-Sphere Raycaster powered by Rayon.

use crate::math::{Ray, Vec3};
use crate::physics::body::BodyId;
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::{FrameBuffer, Pixel};
use crate::render::camera::Camera;
use crate::render::earth_shader::EarthShader;
use crate::render::moon_shader::MoonShader;
use crate::render::stars::Starfield;
use rayon::prelude::*;

const ASCII_RAMP: &[u8] = b" .:-=+*x%#@";

pub struct Raycaster {
    earth_shader: EarthShader,
}

struct SphereHit {
    t: f64,
    body_id: BodyId,
    world_pos: Vec3,
    world_norm: Vec3,
}

impl Raycaster {
    pub fn new() -> Self {
        Self {
            earth_shader: EarthShader::new(),
        }
    }

    /// Analytical ray-sphere intersection. Returns closest positive hit distance t.
    #[inline(always)]
    fn intersect_sphere(ray: &Ray, center: Vec3, radius: f64) -> Option<f64> {
        let oc = ray.origin - center;
        let b = oc.dot(ray.dir);
        let c = oc.dot(oc) - radius * radius;
        let disc = b * b - c;
        if disc < 0.0 {
            return None;
        }
        let sqrt_disc = disc.sqrt();
        let t1 = -b - sqrt_disc;
        let t2 = -b + sqrt_disc;

        if t1 > 0.0 {
            Some(t1)
        } else if t2 > 0.0 {
            Some(t2)
        } else {
            None
        }
    }

    /// Render a full frame into the target FrameBuffer.
    pub fn render_frame(
        &self,
        buffer: &mut FrameBuffer,
        camera: &Camera,
        physics: &SolarSystemPhysics,
        tick: u64,
        elapsed: f64,
    ) {
        let cols = buffer.cols;
        let v_height = buffer.virtual_height();
        let mode = buffer.mode;

        let earth_pos = physics.earth.position;
        let earth_rad = physics.earth.radius_km;
        let earth_mat_inv = physics.earth.orientation_matrix().transpose();

        let moon_pos = physics.moon.position;
        let moon_rad = physics.moon.radius_km;
        let moon_mat_inv = physics.moon.orientation_matrix().transpose();

        let sun_pos = physics.sun.position;

        // Render each scanline in parallel with Rayon
        buffer
            .pixels
            .par_chunks_mut(cols)
            .enumerate()
            .for_each(|(v, row)| {
                for u in 0..cols {
                    let ray = camera.generate_ray(u, v, cols, v_height, mode);

                    // 1. Ray-Sphere intersections
                    let mut closest_hit: Option<SphereHit> = None;

                    // Earth hit
                    if let Some(t_earth) = Self::intersect_sphere(&ray, earth_pos, earth_rad) {
                        let hit_pos = ray.at(t_earth);
                        let norm = (hit_pos - earth_pos).normalize();
                        closest_hit = Some(SphereHit {
                            t: t_earth,
                            body_id: BodyId::Earth,
                            world_pos: hit_pos,
                            world_norm: norm,
                        });
                    }

                    // Moon hit
                    if let Some(t_moon) = Self::intersect_sphere(&ray, moon_pos, moon_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_moon < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_moon);
                            let norm = (hit_pos - moon_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_moon,
                                body_id: BodyId::Moon,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    let (rgb, ch) = if let Some(hit) = closest_hit {
                        let sun_dir = (sun_pos - hit.world_pos).normalize();
                        let view_dir = (ray.origin - hit.world_pos).normalize();

                        let color = match hit.body_id {
                            BodyId::Earth => {
                                let local_norm = earth_mat_inv.mul_vec(hit.world_norm);
                                self.earth_shader.shade(
                                    local_norm,
                                    hit.world_norm,
                                    sun_dir,
                                    view_dir,
                                    tick,
                                    elapsed,
                                )
                            }
                            BodyId::Moon => {
                                let local_norm = moon_mat_inv.mul_vec(hit.world_norm);
                                MoonShader::shade(local_norm, hit.world_norm, sun_dir)
                            }
                            _ => [255, 255, 255],
                        };

                        let lum = 0.299 * color[0] as f64 + 0.587 * color[1] as f64 + 0.114 * color[2] as f64;
                        let r_idx = ((lum / 255.0 * (ASCII_RAMP.len() - 1) as f64) as usize)
                            .clamp(1, ASCII_RAMP.len() - 1);
                        (color, ASCII_RAMP[r_idx] as char)
                    } else {
                        // Check atmospheric limb scattering for Earth
                        let oc = ray.origin - earth_pos;
                        let proj = -oc.dot(ray.dir);
                        let mut atm_color = None;

                        if proj > 0.0 {
                            let closest_p = ray.at(proj);
                            let d_center = (closest_p - earth_pos).length();
                            let atm_top = earth_rad + 140.0; // 140 km atmospheric limb
                            if d_center > earth_rad && d_center < atm_top {
                                let factor = (1.0 - (d_center - earth_rad) / 140.0).powf(1.8);
                                let sun_dir = (sun_pos - closest_p).normalize();
                                let limb_norm = (closest_p - earth_pos).normalize();
                                let sun_facing = limb_norm.dot(sun_dir).max(-0.2) + 0.2;
                                let glow = factor * sun_facing.clamp(0.0, 1.0);
                                atm_color = Some([
                                    (65.0 * glow) as u8,
                                    (165.0 * glow) as u8,
                                    (255.0 * glow) as u8,
                                ]);
                            }
                        }

                        if let Some(atm) = atm_color {
                            let lum = 0.299 * atm[0] as f64 + 0.587 * atm[1] as f64 + 0.114 * atm[2] as f64;
                            let r_idx = ((lum / 255.0 * (ASCII_RAMP.len() - 1) as f64) as usize)
                                .clamp(1, ASCII_RAMP.len() - 1);
                            (atm, ASCII_RAMP[r_idx] as char)
                        } else {
                            // Deep space stars
                            Starfield::sample(ray.dir, tick)
                        }
                    };

                    row[u] = Pixel::new(rgb[0], rgb[1], rgb[2], ch);
                }
            });
    }
}
