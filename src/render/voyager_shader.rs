//! Analytical O(1) CSG Raycaster and Photometric Shader for NASA's Voyager 1 & 2 Interstellar Spacecraft.
//!
//! Spacecraft Architecture:
//! - 3.66m Parabolic High-Gain Antenna (HGA) dish (concave paraboloid y = c(x^2 + z^2))
//! - Subreflector feed horn mounted on a focal tripod
//! - Decagonal equipment bus chassis clad in gold Mylar MLI blankets
//! - The Golden Record ("The Sounds of Earth") 30cm electroplated gold disc on the +X chassis face
//! - 3-canister Radioisotope Thermoelectric Generator (RTG) power boom extending along -X
//! - 13m slender fiberglass Magnetometer Astromast boom extending along +Z

use crate::math::{Mat3, Ray, Vec3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VoyagerPart {
    DishConcave,
    DishConvex,
    Subreflector,
    EquipmentBus,
    GoldenRecord,
    RtgCanister,
    MagBoom,
}

#[derive(Clone, Copy, Debug)]
pub struct VoyagerHit {
    pub t: f64,
    pub local_pos: Vec3,
    pub local_norm: Vec3,
    pub part: VoyagerPart,
}

pub struct VoyagerShader;

impl VoyagerShader {
    pub const fn new() -> Self {
        Self
    }

    /// Compute the spacecraft's local orientation matrix such that the High-Gain Antenna
    /// boresight (local +Y) points directly towards the Sun (Earth direction).
    pub fn orientation_matrix(probe_pos: Vec3, sun_pos: Vec3) -> Mat3 {
        let to_sun = (sun_pos - probe_pos).normalize();
        let y_axis = if to_sun.length_squared() > 1e-4 {
            to_sun
        } else {
            Vec3::Y
        };

        // Reference up vector for stable roll
        let ref_up = if y_axis.y.abs() > 0.92 {
            Vec3::Z
        } else {
            Vec3::Y
        };
        let x_axis = y_axis.cross(ref_up).normalize();
        let z_axis = x_axis.cross(y_axis).normalize();

        // Columns are local basis vectors in world space
        Mat3::from_cols(x_axis, y_axis, z_axis)
    }

    /// Analytical ray-spacecraft CSG intersection in probe-local coordinates.
    /// Fast bounding-sphere rejection avoids testing non-convergent rays.
    pub fn intersect_local(&self, ray: &Ray) -> Option<VoyagerHit> {
        // Fast bounding sphere test (radius = 0.020 km / 20 meters centered at origin)
        let o_dot_d = ray.origin.dot(ray.dir);
        let c = ray.origin.dot(ray.origin) - 0.020 * 0.020;
        if c > 0.0 && o_dot_d > 0.0 {
            return None;
        }
        let disc = o_dot_d * o_dot_d - c;
        if disc < 0.0 {
            return None;
        }

        let mut closest_hit: Option<VoyagerHit> = None;

        // 1. High-Gain Antenna Parabolic Dish: y = c(x^2 + z^2), 0 <= y <= h
        let r_dish = 0.00183; // 1.83m radius (3.66m diameter)
        let h_dish = 0.00045; // 0.45m depth
        let c_dish = h_dish / (r_dish * r_dish); // ~134.368 km^-1

        let a_quad = c_dish * (ray.dir.x * ray.dir.x + ray.dir.z * ray.dir.z);
        if a_quad.abs() > 1e-12 {
            let b_quad = c_dish * (ray.origin.x * ray.dir.x + ray.origin.z * ray.dir.z) - 0.5 * ray.dir.y;
            let c_quad = c_dish * (ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z) - ray.origin.y;
            let d_quad = b_quad * b_quad - a_quad * c_quad;
            if d_quad >= 0.0 {
                let sqrt_d = d_quad.sqrt();
                for &t_root in &[(-b_quad - sqrt_d) / a_quad, (-b_quad + sqrt_d) / a_quad] {
                    if t_root > 1e-6 {
                        let is_closer = closest_hit.map_or(true, |h| t_root < h.t);
                        if is_closer {
                            let p = ray.at(t_root);
                            if p.y >= 0.0 && p.y <= h_dish && (p.x * p.x + p.z * p.z) <= r_dish * r_dish {
                                let mut norm = Vec3::new(-2.0 * c_dish * p.x, 1.0, -2.0 * c_dish * p.z).normalize();
                                let part = if ray.dir.dot(norm) < 0.0 {
                                    VoyagerPart::DishConcave
                                } else {
                                    norm = -norm;
                                    VoyagerPart::DishConvex
                                };
                                closest_hit = Some(VoyagerHit {
                                    t: t_root,
                                    local_pos: p,
                                    local_norm: norm,
                                    part,
                                });
                            }
                        }
                    }
                }
            }
        }

        // 2. Subreflector Feed Horn at focal point (y = 0.9m = 0.0009 km)
        let sub_center = Vec3::new(0.0, 0.0009, 0.0);
        let sub_radius = 0.00015; // 15cm sphere
        let l_sub = sub_center - ray.origin;
        let t_ca_sub = l_sub.dot(ray.dir);
        if t_ca_sub > 0.0 {
            let d2_sub = l_sub.dot(l_sub) - t_ca_sub * t_ca_sub;
            if d2_sub <= sub_radius * sub_radius {
                let thc = (sub_radius * sub_radius - d2_sub).sqrt();
                let t_sub = t_ca_sub - thc;
                if t_sub > 1e-6 && closest_hit.map_or(true, |h| t_sub < h.t) {
                    let p = ray.at(t_sub);
                    let norm = (p - sub_center).normalize();
                    closest_hit = Some(VoyagerHit {
                        t: t_sub,
                        local_pos: p,
                        local_norm: norm,
                        part: VoyagerPart::Subreflector,
                    });
                }
            }
        }

        // 3. Equipment Bus: 10-sided prism / cylinder, y in [-0.0008, 0.0] km, radius 0.0009 km (90cm)
        let r_bus = 0.0009;
        let y_min_bus = -0.0008;
        let y_max_bus = 0.0;
        let a_bus = ray.dir.x * ray.dir.x + ray.dir.z * ray.dir.z;
        if a_bus > 1e-12 {
            let b_bus = ray.origin.x * ray.dir.x + ray.origin.z * ray.dir.z;
            let c_bus = ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z - r_bus * r_bus;
            let d_bus = b_bus * b_bus - a_bus * c_bus;
            if d_bus >= 0.0 {
                let sqrt_d = d_bus.sqrt();
                let t_cyl = (-b_bus - sqrt_d) / a_bus;
                if t_cyl > 1e-6 && closest_hit.map_or(true, |h| t_cyl < h.t) {
                    let p = ray.at(t_cyl);
                    if p.y >= y_min_bus && p.y <= y_max_bus {
                        // 10-sided polygon facet normal approximation
                        let angle = p.z.atan2(p.x);
                        let facet_sector = (angle / (std::f64::consts::PI / 5.0)).round();
                        let facet_angle = facet_sector * (std::f64::consts::PI / 5.0);
                        let norm = Vec3::new(facet_angle.cos(), 0.0, facet_angle.sin()).normalize();
                        closest_hit = Some(VoyagerHit {
                            t: t_cyl,
                            local_pos: p,
                            local_norm: norm,
                            part: VoyagerPart::EquipmentBus,
                        });
                    }
                }
            }
        }

        // Equipment Bus Bottom Cap (plane y = -0.0008 km)
        if ray.dir.y.abs() > 1e-8 {
            let t_cap = (y_min_bus - ray.origin.y) / ray.dir.y;
            if t_cap > 1e-6 && closest_hit.map_or(true, |h| t_cap < h.t) {
                let p = ray.at(t_cap);
                if p.x * p.x + p.z * p.z <= r_bus * r_bus {
                    closest_hit = Some(VoyagerHit {
                        t: t_cap,
                        local_pos: p,
                        local_norm: Vec3::new(0.0, -1.0, 0.0),
                        part: VoyagerPart::EquipmentBus,
                    });
                }
            }
        }

        // 4. The Golden Record ("Sounds of Earth"): Disc on +X bus face
        // Plane at x = 0.00091 km, center at (0.00091, -0.0004, 0.0), radius = 0.00015 km (30cm diameter)
        let x_rec = 0.000905;
        let y_rec_center = -0.0004;
        let z_rec_center = 0.0;
        let r_rec = 0.00015;
        if ray.dir.x.abs() > 1e-8 {
            let t_rec = (x_rec - ray.origin.x) / ray.dir.x;
            if t_rec > 1e-6 && closest_hit.map_or(true, |h| t_rec < h.t) {
                let p = ray.at(t_rec);
                let dy = p.y - y_rec_center;
                let dz = p.z - z_rec_center;
                if dy * dy + dz * dz <= r_rec * r_rec {
                    closest_hit = Some(VoyagerHit {
                        t: t_rec,
                        local_pos: p,
                        local_norm: Vec3::new(1.0, 0.0, 0.0),
                        part: VoyagerPart::GoldenRecord,
                    });
                }
            }
        }

        // 5. RTG Power Boom: 3 stacked canisters extending along -X axis
        // Canisters centered at x in [-0.0032, -0.0012], y = -0.0004, z = 0.0, radius 0.00022 km
        let r_rtg = 0.00022;
        let x_min_rtg = -0.0032;
        let x_max_rtg = -0.0012;

        // RTG outer end-cap circular disc at x = x_min_rtg
        if ray.dir.x.abs() > 1e-8 {
            let t_cap = (x_min_rtg - ray.origin.x) / ray.dir.x;
            if t_cap > 1e-6 && closest_hit.map_or(true, |h| t_cap < h.t) {
                let p = ray.at(t_cap);
                let dy = p.y - (-0.0004);
                let dz = p.z;
                if dy * dy + dz * dz <= r_rtg * r_rtg {
                    closest_hit = Some(VoyagerHit {
                        t: t_cap,
                        local_pos: p,
                        local_norm: Vec3::new(-1.0, 0.0, 0.0),
                        part: VoyagerPart::RtgCanister,
                    });
                }
            }
        }

        let a_rtg = ray.dir.y * ray.dir.y + ray.dir.z * ray.dir.z;
        if a_rtg > 1e-12 {
            let oy_rel = ray.origin.y - (-0.0004);
            let oz_rel = ray.origin.z;
            let b_rtg = oy_rel * ray.dir.y + oz_rel * ray.dir.z;
            let c_rtg = oy_rel * oy_rel + oz_rel * oz_rel - r_rtg * r_rtg;
            let d_rtg = b_rtg * b_rtg - a_rtg * c_rtg;
            if d_rtg >= 0.0 {
                let sqrt_d = d_rtg.sqrt();
                let t_rtg = (-b_rtg - sqrt_d) / a_rtg;
                if t_rtg > 1e-6 && closest_hit.map_or(true, |h| t_rtg < h.t) {
                    let p = ray.at(t_rtg);
                    if p.x >= x_min_rtg && p.x <= x_max_rtg {
                        let norm = Vec3::new(0.0, (p.y - (-0.0004)) / r_rtg, p.z / r_rtg).normalize();
                        closest_hit = Some(VoyagerHit {
                            t: t_rtg,
                            local_pos: p,
                            local_norm: norm,
                            part: VoyagerPart::RtgCanister,
                        });
                    }
                }
            }
        }

        // 6. Magnetometer Astromast: Slender boom extending along +Z axis from z = 0.0009 to 0.013 km (13m)
        let r_mag = 0.00007; // 7cm radius lattice mast
        let z_min_mag = 0.0009;
        let z_max_mag = 0.0130;
        let a_mag = ray.dir.x * ray.dir.x + ray.dir.y * ray.dir.y;
        if a_mag > 1e-12 {
            let oy_rel = ray.origin.y - (-0.0003);
            let b_mag = ray.origin.x * ray.dir.x + oy_rel * ray.dir.y;
            let c_mag = ray.origin.x * ray.origin.x + oy_rel * oy_rel - r_mag * r_mag;
            let d_mag = b_mag * b_mag - a_mag * c_mag;
            if d_mag >= 0.0 {
                let sqrt_d = d_mag.sqrt();
                let t_mag = (-b_mag - sqrt_d) / a_mag;
                if t_mag > 1e-6 && closest_hit.map_or(true, |h| t_mag < h.t) {
                    let p = ray.at(t_mag);
                    if p.z >= z_min_mag && p.z <= z_max_mag {
                        let norm = Vec3::new(p.x / r_mag, (p.y - (-0.0003)) / r_mag, 0.0).normalize();
                        closest_hit = Some(VoyagerHit {
                            t: t_mag,
                            local_pos: p,
                            local_norm: norm,
                            part: VoyagerPart::MagBoom,
                        });
                    }
                }
            }
        }

        // Magnetometer Sensor Sphere at tip of mast (z = 0.0130 km)
        let mag_tip = Vec3::new(0.0, -0.0003, 0.0130);
        let r_tip = 0.00015;
        let l_tip = mag_tip - ray.origin;
        let t_ca_tip = l_tip.dot(ray.dir);
        if t_ca_tip > 0.0 {
            let d2_tip = l_tip.dot(l_tip) - t_ca_tip * t_ca_tip;
            if d2_tip <= r_tip * r_tip {
                let thc = (r_tip * r_tip - d2_tip).sqrt();
                let t_tip = t_ca_tip - thc;
                if t_tip > 1e-6 && closest_hit.map_or(true, |h| t_tip < h.t) {
                    let p = ray.at(t_tip);
                    let norm = (p - mag_tip).normalize();
                    closest_hit = Some(VoyagerHit {
                        t: t_tip,
                        local_pos: p,
                        local_norm: norm,
                        part: VoyagerPart::MagBoom,
                    });
                }
            }
        }

        closest_hit
    }

    /// Photometric shading evaluation for hit parts in local spacecraft coordinates.
    pub fn shade(&self, hit: &VoyagerHit, local_view: Vec3, cam_dist_km: f64) -> [u8; 3] {
        // In local coordinates, the antenna boresight points to Sun along +Y
        let sun_dir = Vec3::new(0.0, 1.0, 0.0);
        let n = hit.local_norm;
        let n_dot_l = n.dot(sun_dir).max(0.0);

        // Blinn-Phong specular highlight
        let h = (sun_dir + local_view).normalize();
        let n_dot_h = n.dot(h).max(0.0);

        // Deep interstellar ambient backlight fill to keep dark side readable
        let ambient = 0.22;

        match hit.part {
            VoyagerPart::DishConcave => {
                // High-gain dish front face: White fiberglass with concentric ribs
                let r = (hit.local_pos.x * hit.local_pos.x + hit.local_pos.z * hit.local_pos.z).sqrt();
                let rib = 0.92 + 0.08 * (r * 28_000.0).cos();
                let diffuse = (n_dot_l * 0.78 + ambient) * rib;
                let spec = n_dot_h.powi(16) * 0.25;
                let r_col = ((235.0 * diffuse + 255.0 * spec).clamp(0.0, 255.0)) as u8;
                let g_col = ((238.0 * diffuse + 255.0 * spec).clamp(0.0, 255.0)) as u8;
                let b_col = ((242.0 * diffuse + 255.0 * spec).clamp(0.0, 255.0)) as u8;
                [r_col, g_col, b_col]
            }
            VoyagerPart::DishConvex => {
                // High-gain dish back face: Structural ribs and off-white/titanium backing
                let diffuse = n_dot_l * 0.70 + ambient;
                let r_col = (205.0 * diffuse).clamp(0.0, 255.0) as u8;
                let g_col = (208.0 * diffuse).clamp(0.0, 255.0) as u8;
                let b_col = (212.0 * diffuse).clamp(0.0, 255.0) as u8;
                [r_col, g_col, b_col]
            }
            VoyagerPart::Subreflector => {
                // Titanium subreflector cone and horn
                let diffuse = n_dot_l * 0.75 + ambient;
                let spec = n_dot_h.powi(32) * 0.35;
                let val = ((220.0 * diffuse + 255.0 * spec).clamp(0.0, 255.0)) as u8;
                [val, val, val]
            }
            VoyagerPart::EquipmentBus => {
                // Gold Mylar multilayer insulation blanket (MLI) with micro-crinkle specular
                let angle = hit.local_pos.z.atan2(hit.local_pos.x);
                let crinkle = 0.88 + 0.12 * (angle * 20.0 + hit.local_pos.y * 30_000.0).sin();
                let diffuse = (n_dot_l * 0.72 + ambient) * crinkle;
                let spec = n_dot_h.powi(24) * 0.45;
                let r_col = ((225.0 * diffuse + 255.0 * spec).clamp(0.0, 255.0)) as u8;
                let g_col = ((180.0 * diffuse + 220.0 * spec).clamp(0.0, 255.0)) as u8;
                let b_col = ((50.0 * diffuse + 80.0 * spec).clamp(0.0, 255.0)) as u8;
                [r_col, g_col, b_col]
            }
            VoyagerPart::GoldenRecord => {
                // 24K electroplated gold phonograph record ("The Sounds of Earth")
                let dy = hit.local_pos.y - (-0.0004);
                let dz = hit.local_pos.z;
                let dr = (dy * dy + dz * dz).sqrt();

                // Concentric sound track grooves
                let groove = 0.88 + 0.12 * (dr * 180_000.0).sin();

                // Close-up detail: Pulsar map etching marks if within 5m
                let etching = if cam_dist_km < 0.005 {
                    let angle = dz.atan2(dy);
                    let rays = (angle * 7.0).cos().abs().powi(8);
                    if rays > 0.6 {
                        1.20
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };

                // Gold has metallic sheen with high base ambient reflectance (0.75)
                let metallic_base = 0.75 + n_dot_l * 0.25;
                let diffuse = metallic_base * groove * etching;
                let spec = n_dot_h.powi(16) * 0.40;
                let r_col = ((255.0 * diffuse + 255.0 * spec).clamp(0.0, 255.0)) as u8;
                let g_col = ((215.0 * diffuse + 235.0 * spec).clamp(0.0, 255.0)) as u8;
                let b_col = ((60.0 * diffuse + 100.0 * spec).clamp(0.0, 255.0)) as u8;
                [r_col, g_col, b_col]
            }
            VoyagerPart::RtgCanister => {
                // Graphite / dark beryllium gray with subtle thermal radiation heat seams
                let diffuse = n_dot_l * 0.60 + ambient;
                let seam = (hit.local_pos.x * 40_000.0).sin().abs();
                let heat_glow = if seam > 0.85 { 45.0 } else { 0.0 };
                let r_col = ((75.0 * diffuse + heat_glow).clamp(0.0, 255.0)) as u8;
                let g_col = ((80.0 * diffuse + heat_glow * 0.3).clamp(0.0, 255.0)) as u8;
                let b_col = (85.0 * diffuse).clamp(0.0, 255.0) as u8;
                [r_col, g_col, b_col]
            }
            VoyagerPart::MagBoom => {
                // Slender fiberglass lattice boom
                let diffuse = n_dot_l * 0.70 + ambient;
                let r_col = (215.0 * diffuse).clamp(0.0, 255.0) as u8;
                let g_col = (210.0 * diffuse).clamp(0.0, 255.0) as u8;
                let b_col = (190.0 * diffuse).clamp(0.0, 255.0) as u8;
                [r_col, g_col, b_col]
            }
        }
    }
}
