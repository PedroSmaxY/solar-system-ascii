//! Parallel analytical Ray-Sphere Raycaster powered by Rayon.

use crate::math::{Ray, Vec3};
use crate::physics::body::BodyId;
use crate::physics::ephemeris::{
    CERES_RADIUS_EQ_KM, CERES_RADIUS_POL_KM, DEIMOS_SEMI_AXES_KM, PHOBOS_SEMI_AXES_KM,
    VESTA_SEMI_AXES_KM, JUPITER_RADIUS_EQ_KM, JUPITER_RADIUS_POL_KM,
};
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::{FrameBuffer, Pixel, RenderMode};
use crate::render::callisto_shader::CallistoShader;
use crate::render::camera::Camera;
use crate::render::ceres_shader::CeresShader;
use crate::render::deimos_shader::DeimosShader;
use crate::render::earth_shader::EarthShader;
use crate::render::europa_shader::EuropaShader;
use crate::render::ganymede_shader::GanymedeShader;
use crate::render::io_shader::IoShader;
use crate::render::jupiter_shader::JupiterShader;
use crate::render::mars_shader::MarsShader;
use crate::render::mercury_shader::MercuryShader;
use crate::render::mimas_shader::MimasShader;
use crate::render::moon_shader::MoonShader;
use crate::render::phobos_shader::PhobosShader;
use crate::render::rhea_shader::RheaShader;
use crate::render::saturn_shader::{
    SaturnShader, SATURN_RADIUS_EQ_KM, SATURN_RADIUS_POL_KM, SATURN_RING_INNER_RADIUS_KM,
    SATURN_RING_OUTER_RADIUS_KM,
};
use crate::render::uranus_shader::{
    UranusShader, URANUS_RADIUS_EQ_KM, URANUS_RADIUS_POL_KM, URANUS_RING_INNER_RADIUS_KM,
    URANUS_RING_OUTER_RADIUS_KM,
};
use crate::render::neptune_shader::{
    NeptuneShader, NEPTUNE_RADIUS_EQ_KM, NEPTUNE_RADIUS_POL_KM, NEPTUNE_RING_INNER_RADIUS_KM,
    NEPTUNE_RING_OUTER_RADIUS_KM,
};
use crate::render::miranda_shader::MirandaShader;
use crate::render::ariel_shader::ArielShader;
use crate::render::umbriel_shader::UmbrielShader;
use crate::render::titania_shader::TitaniaShader;
use crate::render::oberon_shader::OberonShader;
use crate::render::triton_shader::TritonShader;
use crate::render::proteus_shader::ProteusShader;
use crate::render::pluto_shader::PlutoShader;
use crate::render::charon_shader::CharonShader;
use crate::render::voyager_shader::VoyagerShader;
use crate::render::stars::Starfield;
use crate::render::sun_shader::SunShader;
use crate::render::titan_shader::TitanShader;
use crate::render::venus_shader::VenusShader;
use crate::render::vesta_shader::VestaShader;
use crate::render::enceladus_shader::EnceladusShader;
use crate::render::iapetus_shader::IapetusShader;
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
    pub jupiter_shader: JupiterShader,
    pub io_shader: IoShader,
    pub europa_shader: EuropaShader,
    pub ganymede_shader: GanymedeShader,
    pub callisto_shader: CallistoShader,
    pub saturn_shader: SaturnShader,
    pub titan_shader: TitanShader,
    pub enceladus_shader: EnceladusShader,
    pub mimas_shader: MimasShader,
    pub iapetus_shader: IapetusShader,
    pub rhea_shader: RheaShader,
    pub uranus_shader: UranusShader,
    pub miranda_shader: MirandaShader,
    pub ariel_shader: ArielShader,
    pub umbriel_shader: UmbrielShader,
    pub titania_shader: TitaniaShader,
    pub oberon_shader: OberonShader,
    pub neptune_shader: NeptuneShader,
    pub triton_shader: TritonShader,
    pub proteus_shader: ProteusShader,
    pub pluto_shader: PlutoShader,
    pub charon_shader: CharonShader,
    pub voyager_shader: VoyagerShader,
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
            jupiter_shader: JupiterShader::new(),
            io_shader: IoShader::new(),
            europa_shader: EuropaShader::new(),
            ganymede_shader: GanymedeShader::new(),
            callisto_shader: CallistoShader::new(),
            saturn_shader: SaturnShader::new(),
            titan_shader: TitanShader::new(),
            enceladus_shader: EnceladusShader::new(),
            mimas_shader: MimasShader::new(),
            iapetus_shader: IapetusShader::new(),
            rhea_shader: RheaShader::new(),
            uranus_shader: UranusShader::new(),
            miranda_shader: MirandaShader::new(),
            ariel_shader: ArielShader::new(),
            umbriel_shader: UmbrielShader::new(),
            titania_shader: TitaniaShader::new(),
            oberon_shader: OberonShader::new(),
            neptune_shader: NeptuneShader::new(),
            triton_shader: TritonShader::new(),
            proteus_shader: ProteusShader::new(),
            pluto_shader: PlutoShader::new(),
            charon_shader: CharonShader::new(),
            voyager_shader: VoyagerShader::new(),
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

        let jupiter_pos = physics.jupiter.position;
        let jupiter_mat = physics.jupiter.orientation_matrix();
        let jupiter_mat_inv = jupiter_mat.transpose();

        let io_pos = physics.io.position;
        let io_rad = physics.io.radius_km;
        let io_mat = physics.io.orientation_matrix();
        let io_mat_inv = io_mat.transpose();

        let europa_pos = physics.europa.position;
        let europa_rad = physics.europa.radius_km;
        let europa_mat = physics.europa.orientation_matrix();
        let europa_mat_inv = europa_mat.transpose();

        let ganymede_pos = physics.ganymede.position;
        let ganymede_rad = physics.ganymede.radius_km;
        let ganymede_mat = physics.ganymede.orientation_matrix();
        let ganymede_mat_inv = ganymede_mat.transpose();

        let callisto_pos = physics.callisto.position;
        let callisto_rad = physics.callisto.radius_km;
        let callisto_mat = physics.callisto.orientation_matrix();
        let callisto_mat_inv = callisto_mat.transpose();

        let saturn_pos = physics.saturn.position;
        let saturn_mat = physics.saturn.orientation_matrix();
        let saturn_mat_inv = saturn_mat.transpose();

        let titan_pos = physics.titan.position;
        let titan_rad = physics.titan.radius_km;
        let titan_mat = physics.titan.orientation_matrix();
        let titan_mat_inv = titan_mat.transpose();

        let enceladus_pos = physics.enceladus.position;
        let enceladus_rad = physics.enceladus.radius_km;
        let enceladus_mat = physics.enceladus.orientation_matrix();
        let enceladus_mat_inv = enceladus_mat.transpose();

        let mimas_pos = physics.mimas.position;
        let mimas_rad = physics.mimas.radius_km;
        let mimas_mat = physics.mimas.orientation_matrix();
        let mimas_mat_inv = mimas_mat.transpose();

        let iapetus_pos = physics.iapetus.position;
        let iapetus_rad = physics.iapetus.radius_km;
        let iapetus_mat = physics.iapetus.orientation_matrix();
        let iapetus_mat_inv = iapetus_mat.transpose();

        let rhea_pos = physics.rhea.position;
        let rhea_rad = physics.rhea.radius_km;
        let rhea_mat = physics.rhea.orientation_matrix();
        let rhea_mat_inv = rhea_mat.transpose();

        let uranus_pos = physics.uranus.position;
        let uranus_mat = physics.uranus.orientation_matrix();
        let uranus_mat_inv = uranus_mat.transpose();

        let miranda_pos = physics.miranda.position;
        let miranda_rad = physics.miranda.radius_km;
        let miranda_mat = physics.miranda.orientation_matrix();
        let miranda_mat_inv = miranda_mat.transpose();

        let ariel_pos = physics.ariel.position;
        let ariel_rad = physics.ariel.radius_km;
        let ariel_mat = physics.ariel.orientation_matrix();
        let ariel_mat_inv = ariel_mat.transpose();

        let umbriel_pos = physics.umbriel.position;
        let umbriel_rad = physics.umbriel.radius_km;
        let umbriel_mat = physics.umbriel.orientation_matrix();
        let umbriel_mat_inv = umbriel_mat.transpose();

        let titania_pos = physics.titania.position;
        let titania_rad = physics.titania.radius_km;
        let titania_mat = physics.titania.orientation_matrix();
        let titania_mat_inv = titania_mat.transpose();

        let oberon_pos = physics.oberon.position;
        let oberon_rad = physics.oberon.radius_km;
        let oberon_mat = physics.oberon.orientation_matrix();
        let oberon_mat_inv = oberon_mat.transpose();

        let neptune_pos = physics.neptune.position;
        let neptune_mat = physics.neptune.orientation_matrix();
        let neptune_mat_inv = neptune_mat.transpose();

        let triton_pos = physics.triton.position;
        let triton_rad = physics.triton.radius_km;
        let triton_mat = physics.triton.orientation_matrix();
        let triton_mat_inv = triton_mat.transpose();

        let proteus_pos = physics.proteus.position;
        let proteus_rad = physics.proteus.radius_km;
        let proteus_mat = physics.proteus.orientation_matrix();
        let proteus_mat_inv = proteus_mat.transpose();

        let pluto_pos = physics.pluto.position;
        let pluto_rad = physics.pluto.radius_km;
        let pluto_mat = physics.pluto.orientation_matrix();
        let pluto_mat_inv = pluto_mat.transpose();

        let charon_pos = physics.charon.position;
        let charon_rad = physics.charon.radius_km;
        let charon_mat = physics.charon.orientation_matrix();
        let charon_mat_inv = charon_mat.transpose();

        let cam_pos = camera.position();
        let v1_pos = physics.voyager1.position;
        let v2_pos = physics.voyager2.position;
        let v1_mat = VoyagerShader::orientation_matrix(v1_pos, sun_pos);
        let v1_mat_inv = v1_mat.transpose();
        let v2_mat = VoyagerShader::orientation_matrix(v2_pos, sun_pos);
        let v2_mat_inv = v2_mat.transpose();
        let cam_dist_v1 = (cam_pos - v1_pos).length();
        let cam_dist_v2 = (cam_pos - v2_pos).length();
        let check_v1 = cam_dist_v1 < 100.0 || camera.target_body == BodyId::Voyager1;
        let check_v2 = cam_dist_v2 < 100.0 || camera.target_body == BodyId::Voyager2;

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

                    // Jupiter hit (oblate spheroid: 71,492 km eq, 66,854 km pol)
                    if let Some((t_jup, norm)) = Self::intersect_ellipsoid(
                        &ray,
                        jupiter_pos,
                        jupiter_mat,
                        (JUPITER_RADIUS_EQ_KM, JUPITER_RADIUS_EQ_KM, JUPITER_RADIUS_POL_KM),
                    ) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_jup < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_jup);
                            closest_hit = Some(SphereHit {
                                t: t_jup,
                                body_id: BodyId::Jupiter,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Io hit (sphere)
                    if let Some(t_io) = Self::intersect_sphere(&ray, io_pos, io_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_io < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_io);
                            let norm = (hit_pos - io_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_io,
                                body_id: BodyId::Io,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Europa hit (sphere)
                    if let Some(t_eur) = Self::intersect_sphere(&ray, europa_pos, europa_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_eur < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_eur);
                            let norm = (hit_pos - europa_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_eur,
                                body_id: BodyId::Europa,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Ganymede hit (sphere)
                    if let Some(t_gan) = Self::intersect_sphere(&ray, ganymede_pos, ganymede_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_gan < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_gan);
                            let norm = (hit_pos - ganymede_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_gan,
                                body_id: BodyId::Ganymede,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Callisto hit (sphere)
                    if let Some(t_cal) = Self::intersect_sphere(&ray, callisto_pos, callisto_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_cal < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_cal);
                            let norm = (hit_pos - callisto_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_cal,
                                body_id: BodyId::Callisto,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Saturn hit (oblate spheroid: 60,268 km eq, 54,364 km pol)
                    if let Some((t_sat, norm)) = Self::intersect_ellipsoid(
                        &ray,
                        saturn_pos,
                        saturn_mat,
                        (SATURN_RADIUS_EQ_KM, SATURN_RADIUS_EQ_KM, SATURN_RADIUS_POL_KM),
                    ) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_sat < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_sat);
                            closest_hit = Some(SphereHit {
                                t: t_sat,
                                body_id: BodyId::Saturn,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Titan hit (sphere)
                    if let Some(t_tit) = Self::intersect_sphere(&ray, titan_pos, titan_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_tit < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_tit);
                            let norm = (hit_pos - titan_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_tit,
                                body_id: BodyId::Titan,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Enceladus hit (sphere)
                    if let Some(t_enc) = Self::intersect_sphere(&ray, enceladus_pos, enceladus_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_enc < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_enc);
                            let norm = (hit_pos - enceladus_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_enc,
                                body_id: BodyId::Enceladus,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Mimas hit (sphere)
                    if let Some(t_mim) = Self::intersect_sphere(&ray, mimas_pos, mimas_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_mim < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_mim);
                            let norm = (hit_pos - mimas_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_mim,
                                body_id: BodyId::Mimas,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Iapetus hit (sphere)
                    if let Some(t_iap) = Self::intersect_sphere(&ray, iapetus_pos, iapetus_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_iap < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_iap);
                            let norm = (hit_pos - iapetus_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_iap,
                                body_id: BodyId::Iapetus,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Rhea hit (sphere)
                    if let Some(t_rhe) = Self::intersect_sphere(&ray, rhea_pos, rhea_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_rhe < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_rhe);
                            let norm = (hit_pos - rhea_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_rhe,
                                body_id: BodyId::Rhea,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Uranus hit (oblate spheroid: 25,559 km eq, 24,973 km pol)
                    if let Some((t_ura, norm)) = Self::intersect_ellipsoid(
                        &ray,
                        uranus_pos,
                        uranus_mat,
                        (URANUS_RADIUS_EQ_KM, URANUS_RADIUS_EQ_KM, URANUS_RADIUS_POL_KM),
                    ) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_ura < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_ura);
                            closest_hit = Some(SphereHit {
                                t: t_ura,
                                body_id: BodyId::Uranus,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Miranda hit (sphere)
                    if let Some(t_mir) = Self::intersect_sphere(&ray, miranda_pos, miranda_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_mir < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_mir);
                            let norm = (hit_pos - miranda_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_mir,
                                body_id: BodyId::Miranda,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Ariel hit (sphere)
                    if let Some(t_ari) = Self::intersect_sphere(&ray, ariel_pos, ariel_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_ari < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_ari);
                            let norm = (hit_pos - ariel_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_ari,
                                body_id: BodyId::Ariel,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Umbriel hit (sphere)
                    if let Some(t_umb) = Self::intersect_sphere(&ray, umbriel_pos, umbriel_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_umb < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_umb);
                            let norm = (hit_pos - umbriel_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_umb,
                                body_id: BodyId::Umbriel,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Titania hit (sphere)
                    if let Some(t_tit_u) = Self::intersect_sphere(&ray, titania_pos, titania_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_tit_u < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_tit_u);
                            let norm = (hit_pos - titania_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_tit_u,
                                body_id: BodyId::Titania,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Oberon hit (sphere)
                    if let Some(t_obe) = Self::intersect_sphere(&ray, oberon_pos, oberon_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_obe < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_obe);
                            let norm = (hit_pos - oberon_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_obe,
                                body_id: BodyId::Oberon,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Neptune hit (oblate spheroid: 24,764 km eq, 24,341 km pol)
                    if let Some((t_nep, norm)) = Self::intersect_ellipsoid(
                        &ray,
                        neptune_pos,
                        neptune_mat,
                        (NEPTUNE_RADIUS_EQ_KM, NEPTUNE_RADIUS_EQ_KM, NEPTUNE_RADIUS_POL_KM),
                    ) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_nep < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_nep);
                            closest_hit = Some(SphereHit {
                                t: t_nep,
                                body_id: BodyId::Neptune,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Triton hit (sphere)
                    if let Some(t_tri) = Self::intersect_sphere(&ray, triton_pos, triton_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_tri < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_tri);
                            let norm = (hit_pos - triton_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_tri,
                                body_id: BodyId::Triton,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Proteus hit (sphere)
                    if let Some(t_pro) = Self::intersect_sphere(&ray, proteus_pos, proteus_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_pro < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_pro);
                            let norm = (hit_pos - proteus_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_pro,
                                body_id: BodyId::Proteus,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Pluto hit (sphere)
                    if let Some(t_pluto) = Self::intersect_sphere(&ray, pluto_pos, pluto_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_pluto < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_pluto);
                            let norm = (hit_pos - pluto_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_pluto,
                                body_id: BodyId::Pluto,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Charon hit (sphere)
                    if let Some(t_charon) = Self::intersect_sphere(&ray, charon_pos, charon_rad) {
                        let is_closer = match closest_hit {
                            Some(ref h) => t_charon < h.t,
                            None => true,
                        };
                        if is_closer {
                            let hit_pos = ray.at(t_charon);
                            let norm = (hit_pos - charon_pos).normalize();
                            closest_hit = Some(SphereHit {
                                t: t_charon,
                                body_id: BodyId::Charon,
                                world_pos: hit_pos,
                                world_norm: norm,
                            });
                        }
                    }

                    // Voyager 1 hit (Analytical CSG Spacecraft)
                    if check_v1 {
                        let v1_local_ray = Ray::new(
                            v1_mat_inv.mul_vec(ray.origin - v1_pos),
                            v1_mat_inv.mul_vec(ray.dir),
                        );
                        if let Some(v_hit) = self.voyager_shader.intersect_local(&v1_local_ray) {
                            let is_closer = match closest_hit {
                                Some(ref h) => v_hit.t < h.t,
                                None => true,
                            };
                            if is_closer {
                                let hit_pos = ray.at(v_hit.t);
                                let norm = v1_mat.mul_vec(v_hit.local_norm);
                                closest_hit = Some(SphereHit {
                                    t: v_hit.t,
                                    body_id: BodyId::Voyager1,
                                    world_pos: hit_pos,
                                    world_norm: norm,
                                });
                            }
                        }
                    }

                    // Voyager 2 hit (Analytical CSG Spacecraft)
                    if check_v2 {
                        let v2_local_ray = Ray::new(
                            v2_mat_inv.mul_vec(ray.origin - v2_pos),
                            v2_mat_inv.mul_vec(ray.dir),
                        );
                        if let Some(v_hit) = self.voyager_shader.intersect_local(&v2_local_ray) {
                            let is_closer = match closest_hit {
                                Some(ref h) => v_hit.t < h.t,
                                None => true,
                            };
                            if is_closer {
                                let hit_pos = ray.at(v_hit.t);
                                let norm = v2_mat.mul_vec(v_hit.local_norm);
                                closest_hit = Some(SphereHit {
                                    t: v_hit.t,
                                    body_id: BodyId::Voyager2,
                                    world_pos: hit_pos,
                                    world_norm: norm,
                                });
                            }
                        }
                    }

                    // Saturn Ring Plane intersection (in Saturn's equatorial coordinate frame)
                    let sat_local_o = saturn_mat_inv.mul_vec(ray.origin - saturn_pos);
                    let sat_local_d = saturn_mat_inv.mul_vec(ray.dir);
                    let mut ring_hit: Option<(f64, [u8; 4])> = None;
                    if sat_local_d.y.abs() > 1e-6 {
                        let t_ring = -sat_local_o.y / sat_local_d.y;
                        if t_ring > 0.0 {
                            let p_ring = sat_local_o + sat_local_d * t_ring;
                            let r_ring_sq = p_ring.x * p_ring.x + p_ring.z * p_ring.z;
                            if r_ring_sq >= SATURN_RING_INNER_RADIUS_KM * SATURN_RING_INNER_RADIUS_KM
                                && r_ring_sq <= SATURN_RING_OUTER_RADIUS_KM * SATURN_RING_OUTER_RADIUS_KM
                            {
                                let ring_world_hit = ray.at(t_ring);
                                let sun_to_ring = (sun_pos - ring_world_hit).normalize();
                                let local_sun = saturn_mat_inv.mul_vec(sun_to_ring);
                                let local_view = -sat_local_d;
                                if let Some(rgba) = self.saturn_shader.shade_ring(p_ring, local_sun, local_view) {
                                    ring_hit = Some((t_ring, rgba));
                                }
                            }
                        }
                    }

                    // Uranus Ring Plane intersection (in Uranus's tilted equatorial frame)
                    let ur_local_o = uranus_mat_inv.mul_vec(ray.origin - uranus_pos);
                    let ur_local_d = uranus_mat_inv.mul_vec(ray.dir);
                    if ur_local_d.y.abs() > 1e-6 {
                        let t_ring = -ur_local_o.y / ur_local_d.y;
                        if t_ring > 0.0 {
                            let p_ring = ur_local_o + ur_local_d * t_ring;
                            let r_ring_sq = p_ring.x * p_ring.x + p_ring.z * p_ring.z;
                            if r_ring_sq >= URANUS_RING_INNER_RADIUS_KM * URANUS_RING_INNER_RADIUS_KM
                                && r_ring_sq <= URANUS_RING_OUTER_RADIUS_KM * URANUS_RING_OUTER_RADIUS_KM
                            {
                                let ring_world_hit = ray.at(t_ring);
                                let sun_to_ring = (sun_pos - ring_world_hit).normalize();
                                let local_sun = uranus_mat_inv.mul_vec(sun_to_ring);
                                let local_view = -ur_local_d;
                                if let Some(rgba) = self.uranus_shader.shade_ring(p_ring, local_sun, local_view) {
                                    if ring_hit.map_or(true, |h| t_ring < h.0) {
                                        ring_hit = Some((t_ring, rgba));
                                    }
                                }
                            }
                        }
                    }

                    // Neptune Ring Plane intersection (in Neptune's equatorial frame)
                    let nep_local_o = neptune_mat_inv.mul_vec(ray.origin - neptune_pos);
                    let nep_local_d = neptune_mat_inv.mul_vec(ray.dir);
                    if nep_local_d.y.abs() > 1e-6 {
                        let t_ring = -nep_local_o.y / nep_local_d.y;
                        if t_ring > 0.0 {
                            let p_ring = nep_local_o + nep_local_d * t_ring;
                            let r_ring_sq = p_ring.x * p_ring.x + p_ring.z * p_ring.z;
                            if r_ring_sq >= NEPTUNE_RING_INNER_RADIUS_KM * NEPTUNE_RING_INNER_RADIUS_KM
                                && r_ring_sq <= NEPTUNE_RING_OUTER_RADIUS_KM * NEPTUNE_RING_OUTER_RADIUS_KM
                            {
                                let ring_world_hit = ray.at(t_ring);
                                let sun_to_ring = (sun_pos - ring_world_hit).normalize();
                                let local_sun = neptune_mat_inv.mul_vec(sun_to_ring);
                                let local_view = -nep_local_d;
                                if let Some(rgba) = self.neptune_shader.shade_ring(p_ring, local_sun, local_view) {
                                    if ring_hit.map_or(true, |h| t_ring < h.0) {
                                        ring_hit = Some((t_ring, rgba));
                                    }
                                }
                            }
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
                            BodyId::Jupiter => {
                                let local_norm = jupiter_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = jupiter_mat_inv.mul_vec(sun_dir);
                                let local_view = jupiter_mat_inv.mul_vec(view_dir);
                                self.jupiter_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Io => {
                                let local_norm = io_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = io_mat_inv.mul_vec(sun_dir);
                                let local_view = io_mat_inv.mul_vec(view_dir);
                                self.io_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Europa => {
                                let local_norm = europa_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = europa_mat_inv.mul_vec(sun_dir);
                                let local_view = europa_mat_inv.mul_vec(view_dir);
                                self.europa_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Ganymede => {
                                let local_norm = ganymede_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = ganymede_mat_inv.mul_vec(sun_dir);
                                let local_view = ganymede_mat_inv.mul_vec(view_dir);
                                self.ganymede_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Callisto => {
                                let local_norm = callisto_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = callisto_mat_inv.mul_vec(sun_dir);
                                let local_view = callisto_mat_inv.mul_vec(view_dir);
                                self.callisto_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Saturn => {
                                let local_norm = saturn_mat_inv.mul_vec(hit.world_norm);
                                let local_hit = saturn_mat_inv.mul_vec(hit.world_pos - saturn_pos);
                                let local_sun = saturn_mat_inv.mul_vec(sun_dir);
                                let local_view = saturn_mat_inv.mul_vec(view_dir);
                                self.saturn_shader.shade(local_norm, local_hit, local_sun, local_view)
                            }
                            BodyId::Titan => {
                                let local_norm = titan_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = titan_mat_inv.mul_vec(sun_dir);
                                let local_view = titan_mat_inv.mul_vec(view_dir);
                                self.titan_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Enceladus => {
                                let local_norm = enceladus_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = enceladus_mat_inv.mul_vec(sun_dir);
                                let local_view = enceladus_mat_inv.mul_vec(view_dir);
                                self.enceladus_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Mimas => {
                                let local_norm = mimas_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = mimas_mat_inv.mul_vec(sun_dir);
                                let local_view = mimas_mat_inv.mul_vec(view_dir);
                                self.mimas_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Iapetus => {
                                let local_norm = iapetus_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = iapetus_mat_inv.mul_vec(sun_dir);
                                let local_view = iapetus_mat_inv.mul_vec(view_dir);
                                self.iapetus_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Rhea => {
                                let local_norm = rhea_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = rhea_mat_inv.mul_vec(sun_dir);
                                let local_view = rhea_mat_inv.mul_vec(view_dir);
                                self.rhea_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Uranus => {
                                let local_norm = uranus_mat_inv.mul_vec(hit.world_norm);
                                let local_hit = uranus_mat_inv.mul_vec(hit.world_pos - uranus_pos);
                                let local_sun = uranus_mat_inv.mul_vec(sun_dir);
                                let local_view = uranus_mat_inv.mul_vec(view_dir);
                                self.uranus_shader.shade(local_norm, local_hit, local_sun, local_view)
                            }
                            BodyId::Miranda => {
                                let local_norm = miranda_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = miranda_mat_inv.mul_vec(sun_dir);
                                let local_view = miranda_mat_inv.mul_vec(view_dir);
                                self.miranda_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Ariel => {
                                let local_norm = ariel_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = ariel_mat_inv.mul_vec(sun_dir);
                                let local_view = ariel_mat_inv.mul_vec(view_dir);
                                self.ariel_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Umbriel => {
                                let local_norm = umbriel_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = umbriel_mat_inv.mul_vec(sun_dir);
                                let local_view = umbriel_mat_inv.mul_vec(view_dir);
                                self.umbriel_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Titania => {
                                let local_norm = titania_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = titania_mat_inv.mul_vec(sun_dir);
                                let local_view = titania_mat_inv.mul_vec(view_dir);
                                self.titania_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Oberon => {
                                let local_norm = oberon_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = oberon_mat_inv.mul_vec(sun_dir);
                                let local_view = oberon_mat_inv.mul_vec(view_dir);
                                self.oberon_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Neptune => {
                                let local_norm = neptune_mat_inv.mul_vec(hit.world_norm);
                                let local_hit = neptune_mat_inv.mul_vec(hit.world_pos - neptune_pos);
                                let local_sun = neptune_mat_inv.mul_vec(sun_dir);
                                let local_view = neptune_mat_inv.mul_vec(view_dir);
                                self.neptune_shader.shade(local_norm, local_hit, local_sun, local_view)
                            }
                            BodyId::Triton => {
                                let local_norm = triton_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = triton_mat_inv.mul_vec(sun_dir);
                                let local_view = triton_mat_inv.mul_vec(view_dir);
                                self.triton_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Proteus => {
                                let local_norm = proteus_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = proteus_mat_inv.mul_vec(sun_dir);
                                let local_view = proteus_mat_inv.mul_vec(view_dir);
                                self.proteus_shader.shade(local_norm, local_sun, local_view)
                            }
                            BodyId::Pluto => {
                                let local_norm = pluto_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = pluto_mat_inv.mul_vec(sun_dir);
                                let local_view = pluto_mat_inv.mul_vec(view_dir);
                                let dist_au = (hit.world_pos - sun_pos).length() / 149_597_870.7;
                                self.pluto_shader.shade(local_norm, local_sun, local_view, dist_au)
                            }
                            BodyId::Charon => {
                                let local_norm = charon_mat_inv.mul_vec(hit.world_norm);
                                let local_sun = charon_mat_inv.mul_vec(sun_dir);
                                let local_view = charon_mat_inv.mul_vec(view_dir);
                                let dist_au = (hit.world_pos - sun_pos).length() / 149_597_870.7;
                                self.charon_shader.shade(local_norm, local_sun, local_view, dist_au)
                            }
                            BodyId::Voyager1 => {
                                let v1_local_ray = Ray::new(
                                    v1_mat_inv.mul_vec(ray.origin - v1_pos),
                                    v1_mat_inv.mul_vec(ray.dir),
                                );
                                if let Some(v_hit) = self.voyager_shader.intersect_local(&v1_local_ray) {
                                    let local_view = -v1_local_ray.dir;
                                    self.voyager_shader.shade(&v_hit, local_view, cam_dist_v1)
                                } else {
                                    [220, 220, 230]
                                }
                            }
                            BodyId::Voyager2 => {
                                let v2_local_ray = Ray::new(
                                    v2_mat_inv.mul_vec(ray.origin - v2_pos),
                                    v2_mat_inv.mul_vec(ray.dir),
                                );
                                if let Some(v_hit) = self.voyager_shader.intersect_local(&v2_local_ray) {
                                    let local_view = -v2_local_ray.dir;
                                    self.voyager_shader.shade(&v_hit, local_view, cam_dist_v2)
                                } else {
                                    [220, 220, 230]
                                }
                            }
                        };

                        let final_color = if let Some((t_r, ring_rgba)) = ring_hit {
                            if t_r < hit.t {
                                let alpha = ring_rgba[3] as f64 / 255.0;
                                [
                                    (ring_rgba[0] as f64 * alpha + color[0] as f64 * (1.0 - alpha)).round() as u8,
                                    (ring_rgba[1] as f64 * alpha + color[1] as f64 * (1.0 - alpha)).round() as u8,
                                    (ring_rgba[2] as f64 * alpha + color[2] as f64 * (1.0 - alpha)).round() as u8,
                                ]
                            } else {
                                color
                            }
                        } else {
                            color
                        };

                        let lum = 0.299 * final_color[0] as f64 + 0.587 * final_color[1] as f64 + 0.114 * final_color[2] as f64;
                        let r_idx = ((lum / 255.0 * (ASCII_RAMP.len() - 1) as f64) as usize)
                            .clamp(1, ASCII_RAMP.len() - 1);
                        (final_color, ASCII_RAMP[r_idx] as char)
                    } else if let Some((_t_r, ring_rgba)) = ring_hit {
                        hit_row[u] = true;
                        let (star_rgb, _star_ch) = Starfield::sample(ray.dir, tick);
                        let alpha = ring_rgba[3] as f64 / 255.0;
                        let r = (ring_rgba[0] as f64 * alpha + star_rgb[0] as f64 * (1.0 - alpha)).round() as u8;
                        let g = (ring_rgba[1] as f64 * alpha + star_rgb[1] as f64 * (1.0 - alpha)).round() as u8;
                        let b = (ring_rgba[2] as f64 * alpha + star_rgb[2] as f64 * (1.0 - alpha)).round() as u8;
                        let lum = 0.299 * r as f64 + 0.587 * g as f64 + 0.114 * b as f64;
                        let r_idx = ((lum / 255.0 * (ASCII_RAMP.len() - 1) as f64) as usize)
                            .clamp(1, ASCII_RAMP.len() - 1);
                        ([r, g, b], ASCII_RAMP[r_idx] as char)
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
                            if cos_sun > 0.950 {
                                let ang_t = ((cos_sun - 0.950) / 0.050).clamp(0.0, 1.0);
                                let intensity = ang_t.powf(2.0);
                                interplanetary_glare = Some([
                                    (255.0 * intensity.powf(0.75)) as u8,
                                    (250.0 * intensity) as u8,
                                    (210.0 * intensity.powf(1.2)) as u8,
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

        // 7. Analytical Jovian Minor Moons swarm pass (91 minor moons)
        for i in 0..physics.jovian_swarm.count() {
            let p_pos = physics.jovian_swarm.calculate_position(i, jupiter_pos, elapsed);
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
                    if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                        let moon = &physics.jovian_swarm.moons[i];
                        let (r_col, g_col, b_col, ch) = match moon.group {
                            crate::physics::jovian_swarm::JovianMoonGroup::Amalthea => (240, 185, 135, '*'),
                            crate::physics::jovian_swarm::JovianMoonGroup::Himalia => (185, 205, 225, '+'),
                            _ => (155, 155, 165, '.'),
                        };
                        buffer.pixels[pixel_idx] = Pixel::new(r_col, g_col, b_col, ch);
                    }
                }
            }
        }

        // 8. Analytical Saturnian Minor Moons swarm pass (141 minor moons)
        for i in 0..physics.saturnian_swarm.count() {
            let p_pos = physics.saturnian_swarm.calculate_position(i, saturn_pos, elapsed);
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
                    if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                        let moon = &physics.saturnian_swarm.moons[i];
                        let (r_col, g_col, b_col, ch) = match moon.group {
                            crate::physics::saturnian_swarm::SaturnianMoonGroup::RingShepherd => (245, 225, 185, '*'),
                            crate::physics::saturnian_swarm::SaturnianMoonGroup::TrojanIntermediate => (215, 225, 245, '+'),
                            _ => (160, 160, 170, '.'),
                        };
                        buffer.pixels[pixel_idx] = Pixel::new(r_col, g_col, b_col, ch);
                    }
                }
            }
        }

        // 9. Analytical Uranian Minor Moons swarm pass (23 minor moons)
        for i in 0..physics.uranian_swarm.count() {
            let p_pos = physics.uranian_swarm.calculate_position(i, uranus_pos, elapsed);
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
                    if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                        let moon = &physics.uranian_swarm.moons[i];
                        let (r_col, g_col, b_col, ch) = match moon.group {
                            crate::physics::uranian_swarm::UranianMoonGroup::InnerShepherd => (200, 240, 245, '*'),
                            crate::physics::uranian_swarm::UranianMoonGroup::InnerPortia => (180, 225, 235, '+'),
                            _ => (150, 160, 170, '.'),
                        };
                        buffer.pixels[pixel_idx] = Pixel::new(r_col, g_col, b_col, ch);
                    }
                }
            }
        }

        // 10. Analytical Neptunian Minor Moons swarm pass (14 minor moons)
        for i in 0..physics.neptunian_swarm.count() {
            let p_pos = physics.neptunian_swarm.calculate_position(i, neptune_pos, elapsed);
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
                    if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                        let moon = &physics.neptunian_swarm.moons[i];
                        let (r_col, g_col, b_col, ch) = match moon.group {
                            crate::physics::neptunian_swarm::NeptunianMoonGroup::InnerRegular => (160, 195, 255, '*'),
                            crate::physics::neptunian_swarm::NeptunianMoonGroup::Nereid => (210, 225, 255, '+'),
                            _ => (155, 160, 175, '.'),
                        };
                        buffer.pixels[pixel_idx] = Pixel::new(r_col, g_col, b_col, ch);
                    }
                }
            }
        }

        // 11. Keplerian Planetary and Lunar Orbit Lines (when camera.show_orbit_lines is active)
        if camera.show_orbit_lines {
            let orbits = [
                (&physics.mercury.orbit, [135, 115, 95]),
                (&physics.venus.orbit, [155, 145, 105]),
                (&physics.earth.orbit, [85, 135, 185]),
                (&physics.mars.orbit, [165, 95, 70]),
                (&physics.ceres.orbit, [115, 115, 125]),
                (&physics.jupiter.orbit, [160, 140, 100]),
                (&physics.saturn.orbit, [150, 140, 95]),
                (&physics.uranus.orbit, [90, 145, 160]),
                (&physics.neptune.orbit, [70, 105, 175]),
                (&physics.pluto.orbit, [140, 115, 105]),
            ];

            for (orb_opt, color) in &orbits {
                if let Some(ref orb) = orb_opt {
                    // Sample 256 elliptical nodes along the complete Keplerian orbit
                    const NODES: usize = 256;
                    let period = orb.period;
                    for step in 0..NODES {
                        let t_sample = (step as f64) * (period / NODES as f64);
                        let p_pos = orb.calculate_position(t_sample);
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
                                if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                                    buffer.pixels[pixel_idx] = Pixel::new(color[0], color[1], color[2], '.');
                                }
                            }
                        }
                    }
                }
            }

            // Natural satellite orbits around their host worlds (visible when viewing planetary systems)
            let moon_systems: &[(&Option<crate::physics::ephemeris::KeplerOrbit>, Vec3, [u8; 3], f64)] = &[
                (&physics.moon.orbit, earth_pos, [145, 175, 210], 15_000_000.0),
                (&physics.phobos.orbit, mars_pos, [180, 140, 110], 2_500_000.0),
                (&physics.deimos.orbit, mars_pos, [160, 150, 140], 2_500_000.0),
                (&physics.io.orbit, jupiter_pos, [215, 185, 75], 45_000_000.0),
                (&physics.europa.orbit, jupiter_pos, [165, 185, 215], 45_000_000.0),
                (&physics.ganymede.orbit, jupiter_pos, [175, 160, 145], 45_000_000.0),
                (&physics.callisto.orbit, jupiter_pos, [145, 145, 155], 45_000_000.0),
                (&physics.titan.orbit, saturn_pos, [215, 185, 120], 55_000_000.0),
                (&physics.enceladus.orbit, saturn_pos, [195, 215, 235], 55_000_000.0),
                (&physics.mimas.orbit, saturn_pos, [175, 175, 185], 55_000_000.0),
                (&physics.iapetus.orbit, saturn_pos, [160, 150, 140], 55_000_000.0),
                (&physics.rhea.orbit, saturn_pos, [185, 190, 195], 55_000_000.0),
                (&physics.miranda.orbit, uranus_pos, [165, 175, 185], 25_000_000.0),
                (&physics.ariel.orbit, uranus_pos, [185, 195, 205], 25_000_000.0),
                (&physics.umbriel.orbit, uranus_pos, [130, 130, 140], 25_000_000.0),
                (&physics.titania.orbit, uranus_pos, [165, 165, 175], 25_000_000.0),
                (&physics.oberon.orbit, uranus_pos, [155, 155, 165], 25_000_000.0),
                (&physics.triton.orbit, neptune_pos, [175, 195, 220], 25_000_000.0),
                (&physics.proteus.orbit, neptune_pos, [120, 125, 135], 25_000_000.0),
                (&physics.charon.orbit, pluto_pos, [165, 155, 145], 2_000_000.0),
            ];

            for (orb_opt, parent_center, color, max_cam_dist) in moon_systems {
                if (cam_pos - *parent_center).length() <= *max_cam_dist {
                    if let Some(ref orb) = orb_opt {
                        const MOON_NODES: usize = 128;
                        let period = orb.period;
                        for step in 0..MOON_NODES {
                            let t_sample = (step as f64) * (period / MOON_NODES as f64);
                            let p_pos = *parent_center + orb.calculate_position(t_sample);
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
                                    if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                                        buffer.pixels[pixel_idx] = Pixel::new(color[0], color[1], color[2], '.');
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 12. Interstellar Probes Voyager 1 & Voyager 2 (visible at distances > 3.0e9 km / 20 AU)
        if camera.distance_km > 3.0e9 {
            let probes = [
                (physics.voyager1.position, [140, 230, 255]),
                (physics.voyager2.position, [110, 210, 250]),
            ];
            for (p_pos, color) in &probes {
                let to_p = *p_pos - cam_pos;
                let dist = to_p.length();
                if dist < 1.0 {
                    continue;
                }
                let dir = to_p / dist;
                let f_dot = dir.dot(f);
                if f_dot > 0.05 {
                    let ndc_x = dir.dot(r) / (f_dot * tan_half_fov_x);
                    let ndc_y = dir.dot(u_vec) / (f_dot * tan_half_fov_y);

                    if ndc_x.abs() < 0.98 && ndc_y.abs() < 0.95 {
                        let px = ((ndc_x + 1.0) * 0.5 * cols as f64) as usize;
                        let py = ((1.0 - ndc_y) * 0.5 * v_height as f64) as usize;

                        if px < cols && py < v_height {
                            let pixel_idx = py * cols + px;
                            if !hit_mask[pixel_idx] && !buffer.pixels[pixel_idx].is_text {
                                buffer.pixels[pixel_idx] = Pixel::new(color[0], color[1], color[2], '+');
                            }
                        }
                    }
                }
            }
        }

        // 13. Voyager Pale Blue Dot & 1990 Sagan Sunbeam (visible from deep space / Voyager view)
        if cam_pos.length() > 1.0e10 || camera.target_body == BodyId::Voyager1 || camera.target_body == BodyId::Voyager2 {
            let to_earth = earth_pos - cam_pos;
            let dist_earth = to_earth.length();
            if dist_earth > 1.0e9 {
                let dir_earth = to_earth / dist_earth;
                let f_dot = dir_earth.dot(f);
                if f_dot > 0.05 {
                    let ndc_x = dir_earth.dot(r) / (f_dot * tan_half_fov_x);
                    let ndc_y = dir_earth.dot(u_vec) / (f_dot * tan_half_fov_y);
                    if ndc_x.abs() < 0.98 && ndc_y.abs() < 0.95 {
                        let ex = ((ndc_x + 1.0) * 0.5 * cols as f64) as usize;
                        let ey = ((1.0 - ndc_y) * 0.5 * v_height as f64) as usize;

                        // Draw Carl Sagan's iconic 1990 scattered optical diffraction sunbeam ray across screen
                        for row in 0..v_height {
                            let col_center = (ex as f64 + (row as f64 - ey as f64) * 0.52).round() as isize;
                            for dc in -1..=1 {
                                let c = col_center + dc;
                                if c >= 0 && (c as usize) < cols {
                                    let c_idx = row * cols + (c as usize);
                                    if !hit_mask[c_idx] && !buffer.pixels[c_idx].is_text {
                                        let intensity = if dc == 0 { 42 } else { 22 };
                                        let curr = &buffer.pixels[c_idx];
                                        buffer.pixels[c_idx] = Pixel::new(
                                            curr.r.saturating_add((intensity as f64 * 0.75) as u8),
                                            curr.g.saturating_add((intensity as f64 * 0.82) as u8),
                                            curr.b.saturating_add(intensity),
                                            if curr.ch == ' ' { '.' } else { curr.ch },
                                        );
                                    }
                                }
                            }
                        }

                        // Place the distinct Pale Blue Dot pixel
                        if ex < cols && ey < v_height {
                            let e_idx = ey * cols + ex;
                            buffer.pixels[e_idx] = Pixel::new(135, 215, 255, '*');
                        }
                    }
                }
            }
        }
    }
}
