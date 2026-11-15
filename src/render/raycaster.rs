//! Parallel analytical Ray-Sphere Raycaster powered by Rayon.

use crate::math::{Ray, Vec3};
use crate::physics::body::BodyId;
use crate::physics::ephemeris::{
    CERES_RADIUS_EQ_KM, CERES_RADIUS_POL_KM, DEIMOS_SEMI_AXES_KM, PHOBOS_SEMI_AXES_KM,
    VESTA_SEMI_AXES_KM,
};
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::{FrameBuffer, Pixel, RenderMode};
use crate::render::camera::Camera;
use crate::render::ceres_shader::CeresShader;
use crate::render::deimos_shader::DeimosShader;
use crate::render::earth_shader::EarthShader;
use crate::render::mars_shader::MarsShader;
use crate::render::mercury_shader::MercuryShader;
use crate::render::moon_shader::MoonShader;
use crate::render::phobos_shader::PhobosShader;
use crate::render::stars::Starfield;
use crate::render::sun_shader::SunShader;
use crate::render::venus_shader::VenusShader;
use crate::render::vesta_shader::VestaShader;
use rayon::prelude::*;

const ASCII_RAMP: &[u8] = b" .:-=+*x%#@";

pub struct Raycaster {
    pub earth_shader: EarthShader,
    pub mercury_shader: MercuryShader,
    pub moon_shader: MoonShader,
    pub sun_shader: SunShader,
    pub venus_shader: VenusShader,
    pub mars_shader: MarsShader,
    pub phobos_shader: PhobosShader,
    pub deimos_shader: DeimosShader,
    pub ceres_shader: CeresShader,
    pub vesta_shader: VestaShader,
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
            mercury_shader: MercuryShader::new(),
            moon_shader: MoonShader::new(),
            sun_shader: SunShader::new(),
            venus_shader: VenusShader::new(),
            mars_shader: MarsShader::new(),
            phobos_shader: PhobosShader::new(),
            deimos_shader: DeimosShader::new(),
            ceres_shader: CeresShader::new(),
            vesta_shader: VestaShader::new(),
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

    /// Analytical ray-ellipsoid intersection for triaxial bodies (Phobos, Deimos).
    /// semi_axes is (a, b, c) along local axes.
    /// Returns (hit_distance t, world_normal) if hit.
    #[inline(always)]
    pub fn intersect_ellipsoid(
        ray: &Ray,
        center: Vec3,
        rot_mat: crate::math::Mat3,
        semi_axes: (f64, f64, f64),
    ) -> Option<(f64, Vec3)> {
        let (a, b, c) = semi_axes;
        let oc = ray.origin - center;

        // rot_mat maps local -> world. Transpose maps world -> local.
        let rot_t = rot_mat.transpose();
        let local_oc = rot_t.mul_vec(oc);
        let local_dir = rot_t.mul_vec(ray.dir);

        let inv_x = 1.0 / a; // X (sub-Mars axis)
        let inv_y = 1.0 / c; // Y (polar axis)
        let inv_z = 1.0 / b; // Z (along-orbit axis)

        let u_oc = Vec3::new(local_oc.x * inv_x, local_oc.y * inv_y, local_oc.z * inv_z);
        let u_dir = Vec3::new(local_dir.x * inv_x, local_dir.y * inv_y, local_dir.z * inv_z);

        let quad_a = u_dir.dot(u_dir);
        let quad_b = u_oc.dot(u_dir);
        let quad_c = u_oc.dot(u_oc) - 1.0;

        let disc = quad_b * quad_b - quad_a * quad_c;
        if disc < 0.0 {
            return None;
        }

        let sqrt_disc = disc.sqrt();
        let t1 = (-quad_b - sqrt_disc) / quad_a;
        let t2 = (-quad_b + sqrt_disc) / quad_a;

        let t = if t1 > 0.0 {
            t1
        } else if t2 > 0.0 {
            t2
        } else {
            return None;
        };

        let local_hit = local_oc + local_dir * t;
        let local_norm = Vec3::new(
            local_hit.x * (inv_x * inv_x),
            local_hit.y * (inv_y * inv_y),
            local_hit.z * (inv_z * inv_z),
        ).normalize();

        let world_norm = rot_mat.mul_vec(local_norm).normalize();
        Some((t, world_norm))
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

        let mercury_pos = physics.mercury.position;
        let mercury_rad = physics.mercury.radius_km;
        let mercury_mat_inv = physics.mercury.orientation_matrix().transpose();

        let venus_pos = physics.venus.position;
        let venus_rad = physics.venus.radius_km;
        let venus_mat_inv = physics.venus.orientation_matrix().transpose();

        let moon_pos = physics.moon.position;
        let moon_rad = physics.moon.radius_km;
        let moon_mat_inv = physics.moon.orientation_matrix().transpose();

        let sun_pos = physics.sun.position;
        let sun_rad = physics.sun.radius_km;
        let sun_mat_inv = physics.sun.orientation_matrix().transpose();

        let mars_pos = physics.mars.position;
        let mars_rad = physics.mars.radius_km;
        let mars_mat = physics.mars.orientation_matrix();
        let mars_mat_inv = mars_mat.transpose();

        let phobos_pos = physics.phobos.position;
        let phobos_mat = physics.phobos.orientation_matrix();
        let phobos_mat_inv = phobos_mat.transpose();

        let deimos_pos = physics.deimos.position;
        let deimos_mat = physics.deimos.orientation_matrix();
        let deimos_mat_inv = deimos_mat.transpose();

        let ceres_pos = physics.ceres.position;
        let ceres_mat = physics.ceres.orientation_matrix();
        let ceres_mat_inv = ceres_mat.transpose();

        let vesta_pos = physics.vesta.position;
        let vesta_mat = physics.vesta.orientation_matrix();
        let vesta_mat_inv = vesta_mat.transpose();

        let mut hit_mask: Vec<bool> = vec![false; cols * v_height];

        // Render each scanline in parallel with Rayon
        buffer
            .pixels
            .par_chunks_mut(cols)
            .zip(hit_mask.par_chunks_mut(cols))
            .enumerate()
            .for_each(|(v, (row, hit_row))| {
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

                    // Mercury hit
                    if let Some(t_merc) = Self::intersect_sphere(&ray, mercury_pos, mercury_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_merc < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_merc);
                            let norm = (hit_pos - mercury_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_merc,
                                body_id: BodyId::Mercury,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Venus hit
                    if let Some(t_venus) = Self::intersect_sphere(&ray, venus_pos, venus_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_venus < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_venus);
                            let norm = (hit_pos - venus_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_venus,
                                body_id: BodyId::Venus,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
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

                    // Sun hit
                    if let Some(t_sun) = Self::intersect_sphere(&ray, sun_pos, sun_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_sun < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_sun);
                            let norm = (hit_pos - sun_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_sun,
                                body_id: BodyId::Sun,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Mars hit
                    if let Some(t_mars) = Self::intersect_sphere(&ray, mars_pos, mars_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_mars < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_mars);
                            let norm = (hit_pos - mars_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_mars,
                                body_id: BodyId::Mars,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Phobos hit (triaxial ellipsoid)
                    if let Some((t_phobos, norm)) = Self::intersect_ellipsoid(&ray, phobos_pos, phobos_mat, PHOBOS_SEMI_AXES_KM) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_phobos < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_phobos);
                            closest_hit = Some(SphereHit {
                                t: t_phobos,
                                body_id: BodyId::Phobos,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Deimos hit (triaxial ellipsoid)
                    if let Some((t_deimos, norm)) = Self::intersect_ellipsoid(&ray, deimos_pos, deimos_mat, DEIMOS_SEMI_AXES_KM) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_deimos < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_deimos);
                            closest_hit = Some(SphereHit {
                                t: t_deimos,
                                body_id: BodyId::Deimos,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Ceres hit (oblate spheroid)
                    if let Some((t_ceres, norm)) = Self::intersect_ellipsoid(
                        &ray,
                        ceres_pos,
                        ceres_mat,
                        (CERES_RADIUS_EQ_KM, CERES_RADIUS_EQ_KM, CERES_RADIUS_POL_KM),
                    ) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_ceres < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_ceres);
                            closest_hit = Some(SphereHit {
                                t: t_ceres,
                                body_id: BodyId::Ceres,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Vesta hit (triaxial ellipsoid)
                    if let Some((t_vesta, norm)) = Self::intersect_ellipsoid(&ray, vesta_pos, vesta_mat, VESTA_SEMI_AXES_KM) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_vesta < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_vesta);
                            closest_hit = Some(SphereHit {
                                t: t_vesta,
                                body_id: BodyId::Vesta,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    if closest_hit.is_some() {
                        hit_row[u] = true;
                    }

                    let (rgb, ch) = if let Some(hit) = closest_hit {
                        let sun_dir = if hit.body_id == BodyId::Sun {
                            Vec3::new(0.0, 1.0, 0.0)
                        } else {
                            (sun_pos - hit.world_pos).normalize()
                        };
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
                            BodyId::Mercury => {
                                let local_norm = mercury_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = mercury_mat_inv.mul_vec(sun_dir);
                                let local_view = mercury_mat_inv.mul_vec(view_dir);
                                self.mercury_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Venus => {
                                let local_norm = venus_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = venus_mat_inv.mul_vec(sun_dir);
                                let local_view = venus_mat_inv.mul_vec(view_dir);
                                self.venus_shader.shade(local_norm, local_sun, local_view, elapsed)
                            }
                            BodyId::Moon => {
                                let local_norm = moon_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = moon_mat_inv.mul_vec(sun_dir);
                                let local_view = moon_mat_inv.mul_vec(view_dir);
                                self.moon_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Sun => {
                                let oc_sun = ray.origin - sun_pos;
                                if oc_sun.length_squared() > 100_000_000_000_000.0 {
                                    // Distant incandescent white-gold solar star core
                                    [255, 255, 240]
                                } else {
                                    let local_norm = sun_mat_inv.mul_vec(hit.world_norm);
                                    self.sun_shader.shade_photosphere(local_norm, view_dir, tick, elapsed)
                                }
                            }
                            BodyId::Mars => {
                                let local_norm = mars_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = mars_mat_inv.mul_vec(sun_dir);
                                let local_view = mars_mat_inv.mul_vec(view_dir);
                                self.mars_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Phobos => {
                                let local_hit = phobos_mat_inv.mul_vec(hit.world_pos - phobos_pos);
                                let local_sun = phobos_mat_inv.mul_vec(sun_dir);
                                let local_view = phobos_mat_inv.mul_vec(view_dir);
                                self.phobos_shader.shade(local_hit, local_sun, local_view)
                            }
                            BodyId::Deimos => {
                                let local_hit = deimos_mat_inv.mul_vec(hit.world_pos - deimos_pos);
                                let local_sun = deimos_mat_inv.mul_vec(sun_dir);
                                let local_view = deimos_mat_inv.mul_vec(view_dir);
                                self.deimos_shader.shade(local_hit, local_sun, local_view)
                            }
                            BodyId::Ceres => {
                                let local_hit = ceres_mat_inv.mul_vec(hit.world_pos - ceres_pos);
                                let local_sun = ceres_mat_inv.mul_vec(sun_dir);
                                let local_view = ceres_mat_inv.mul_vec(view_dir);
                                self.ceres_shader.shade(local_hit, local_sun, local_view)
                            }
                            BodyId::Vesta => {
                                let local_hit = vesta_mat_inv.mul_vec(hit.world_pos - vesta_pos);
                                let local_sun = vesta_mat_inv.mul_vec(sun_dir);
                                let local_view = vesta_mat_inv.mul_vec(view_dir);
                                self.vesta_shader.shade(local_hit, local_sun, local_view)
                            }
                        };

                        let lum = 0.299 * color[0] as f64 + 0.587 * color[1] as f64 + 0.114 * color[2] as f64;
                        let r_idx = ((lum / 255.0 * (ASCII_RAMP.len() - 1) as f64) as usize)
                            .clamp(1, ASCII_RAMP.len() - 1);
                        (color, ASCII_RAMP[r_idx] as char)
                    } else {
                        // 1. Check Solar Corona & Extended Radial Rays (only close-up, <= 10,000,000 km)
                        let oc_sun = ray.origin - sun_pos;
                        let dist_to_sun = (-oc_sun).length();
                        let mut corona_color = None;

                        if dist_to_sun <= 10_000_000.0 {
                            let proj_sun = -oc_sun.dot(ray.dir);
                            if proj_sun > 0.0 {
                                let closest_p = ray.at(proj_sun);
                                let sun_to_p = closest_p - sun_pos;
                                let d_center = sun_to_p.length();
                                let d_ratio = d_center / sun_rad;
                                if d_ratio > 1.0 && d_ratio < 4.5 {
                                    let rel_norm = sun_to_p / d_center;
                                    corona_color = self.sun_shader.shade_corona(d_ratio, rel_norm, ray.dir, elapsed);
                                }
                            }
                        }

                        // 2. Check atmospheric limb scattering for Earth
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

                        // Atmospheric limb scattering for Venus (350 km dense haze & extended twilight arc)
                        let oc_v = ray.origin - venus_pos;
                        let proj_v = -oc_v.dot(ray.dir);
                        if proj_v > 0.0 {
                            let closest_p = ray.at(proj_v);
                            let d_center = (closest_p - venus_pos).length();
                            let atm_top = venus_rad + 350.0;
                            if d_center > venus_rad && d_center < atm_top {
                                let factor = (1.0 - (d_center - venus_rad) / 350.0).powf(1.5);
                                let sun_dir = (sun_pos - closest_p).normalize();
                                let limb_norm = (closest_p - venus_pos).normalize();
                                let sun_facing = limb_norm.dot(sun_dir);
                                if sun_facing > -0.25 {
                                    let twilight = ((sun_facing + 0.25) / 1.25).clamp(0.0, 1.0);
                                    let glow = factor * twilight;
                                    atm_color = Some([
                                        (245.0 * glow) as u8,
                                        (228.0 * glow) as u8,
                                        (170.0 * glow) as u8,
                                    ]);
                                }
                            }
                        }

                        // Atmospheric limb scattering for Mars (60 km thin CO2 butterscotch glow)
                        let oc_m = ray.origin - mars_pos;
                        let proj_m = -oc_m.dot(ray.dir);
                        if proj_m > 0.0 {
                            let closest_p = ray.at(proj_m);
                            let d_center = (closest_p - mars_pos).length();
                            let atm_top = mars_rad + 60.0;
                            if d_center > mars_rad && d_center < atm_top {
                                let factor = (1.0 - (d_center - mars_rad) / 60.0).powf(1.6);
                                let sun_dir = (sun_pos - closest_p).normalize();
                                let limb_norm = (closest_p - mars_pos).normalize();
                                let sun_facing = limb_norm.dot(sun_dir);
                                if sun_facing > -0.15 {
                                    let twilight = ((sun_facing + 0.15) / 1.15).clamp(0.0, 1.0);
                                    let glow = factor * twilight;
                                    atm_color = Some([
                                        (190.0 * glow) as u8,
                                        (135.0 * glow) as u8,
                                        (105.0 * glow) as u8,
                                    ]);
                                }
                            }
                        }

                        // 3. Interplanetary Solar Glare for distant observers (> 10,000,000 km, e.g. Earth or Moon)
                        // Smooth, monotonic radial falloff all the way to center (no black void!)
                        let mut interplanetary_glare = None;
                        if dist_to_sun > 10_000_000.0 {
                            let sun_dir = -oc_sun / dist_to_sun;
                            let cos_sun = ray.dir.dot(sun_dir);
                            if cos_sun > 0.985 {
                                let ang_t = ((cos_sun - 0.985) / 0.015).clamp(0.0, 1.0);
                                let intensity = ang_t.powf(2.4);
                                interplanetary_glare = Some([
                                    (255.0 * intensity.powf(0.85)) as u8,
                                    (240.0 * intensity) as u8,
                                    (180.0 * intensity.powf(1.4)) as u8,
                                ]);
                            }
                        }

                        // 4. Deep space background stars (sampled for all sky directions)
                        let (star_rgb, star_ch) = Starfield::sample(ray.dir, tick);

                        // 5. Additive semi-transparent compositing (no star occlusion, no black border)
                        let active_solar = if dist_to_sun <= 10_000_000.0 {
                            corona_color
                        } else {
                            interplanetary_glare
                        };
                        if let Some(corona) = active_solar {
                            let r = corona[0].saturating_add(star_rgb[0]);
                            let g = corona[1].saturating_add(star_rgb[1]);
                            let b = corona[2].saturating_add(star_rgb[2]);
                            let lum = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
                            let ch = if lum > 190.0 {
                                '@'
                            } else if lum > 140.0 {
                                '#'
                            } else if lum > 90.0 {
                                '*'
                            } else if lum > 50.0 {
                                '+'
                            } else if lum > 22.0 {
                                ':'
                            } else if lum > 6.0 {
                                '.'
                            } else {
                                star_ch
                            };
                            ([r, g, b], ch)
                        } else if let Some(atm) = atm_color {
                            let r = atm[0].saturating_add(star_rgb[0]);
                            let g = atm[1].saturating_add(star_rgb[1]);
                            let b = atm[2].saturating_add(star_rgb[2]);
                            let lum = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
                            let r_idx = ((lum / 255.0 * (ASCII_RAMP.len() - 1) as f64) as usize)
                                .clamp(1, ASCII_RAMP.len() - 1);
                            ([r, g, b], ASCII_RAMP[r_idx] as char)
                        } else {
                            (star_rgb, star_ch)
                        }
                    };

                    row[u] = Pixel::new(rgb[0], rgb[1], rgb[2], ch);
                }
            });

        // 6. Keplerian Asteroid Belt field rendering pass
        let cam_pos = camera.position();
        let (sin_az, cos_az) = camera.azimuth.sin_cos();
        let (sin_el, cos_el) = camera.elevation.sin_cos();
        let f = Vec3::new(-cos_el * sin_az, -sin_el, -cos_el * cos_az);
        let r = Vec3::new(cos_az, 0.0, -sin_az);
        let u_vec = Vec3::new(-sin_el * sin_az, cos_el, -sin_el * cos_az);

        let pixel_aspect = match mode {
            RenderMode::Ascii => 2.15,
            RenderMode::HalfBlock => 1.05,
        };
        let screen_aspect = (cols as f64) / (v_height as f64) / pixel_aspect;
        let tan_half_fov_y = (camera.fov_y_rad * 0.5).tan();
        let tan_half_fov_x = tan_half_fov_y * screen_aspect;

        for i in 0..physics.asteroid_field.particles.len() {
            let p_pos = physics.asteroid_field.calculate_position(i, elapsed);
            let to_p = p_pos - cam_pos;
            let dist = to_p.length();
            if dist < 1.0 {
                continue;
            }
            let dir = to_p / dist;
            let f_dot = dir.dot(f);
            if f_dot <= 0.05 {
                continue;
            }

            let ndc_x = dir.dot(r) / (f_dot * tan_half_fov_x);
            let ndc_y = dir.dot(u_vec) / (f_dot * tan_half_fov_y);

            if ndc_x.abs() < 0.98 && ndc_y.abs() < 0.95 {
                let px = ((ndc_x + 1.0) * 0.5 * cols as f64) as usize;
                let py = ((1.0 - ndc_y) * 0.5 * v_height as f64) as usize;

                if px < cols && py < v_height {
                    let pixel_idx = py * cols + px;
                    // Only draw over background space/starfield (never occlude illuminated planets or nightside)
                    if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                        let particle = &physics.asteroid_field.particles[i];
                        let brightness = (particle.radius_km / 12.0).clamp(0.65, 1.35);
                        let base_gray = (165.0 * brightness) as u8;
                        let ast_r = base_gray.saturating_add(15);
                        let ast_g = base_gray.saturating_add(10);
                        let ast_b = base_gray;

                        let ch = if particle.radius_km > 10.0 {
                            '*'
                        } else if particle.radius_km > 4.0 {
                            '+'
                        } else {
                            '.'
                        };
                        buffer.pixels[pixel_idx] = Pixel::new(ast_r, ast_g, ast_b, ch);
                    }
                }
            }
        }
    }
}
