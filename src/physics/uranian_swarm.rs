//! Analytical Keplerian model for Uranus's 23 minor moons:
//! - Inner ring shepherds & portia group (Cordelia, Ophelia, Bianca, Cressida, Desdemona, Juliet, Portia, Rosalind, Cupid, Belinda, Perdita, Puck, Mab).
//! - Outer irregulars (Francisco, Caliban, Stephano, Trinculo, Sycorax, Margaret, Prospero, Setebos, Ferdinand, S/2023 U 1).
//! - Coordinates aligned with ephemeris.rs with equatorial plane tilted by Uranus's axial tilt (97.77°).

use crate::math::{Mat3, Vec3};
use crate::physics::ephemeris::URANUS_AXIAL_TILT_RAD;
use std::f64::consts::PI;

pub const URANUS_MINOR_MOON_COUNT: usize = 23;
const TWO_PI: f64 = 2.0 * PI;
const GM_URANUS: f64 = 5.793939e6; // km^3 / s^2

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UranianMoonGroup {
    InnerShepherd,
    InnerPortia,
    OuterIrregular,
}

#[derive(Clone, Debug)]
pub struct UranianMinorMoon {
    pub name: &'static str,
    pub group: UranianMoonGroup,
    pub a: f64,         // Semi-major axis (km)
    pub e: f64,         // Eccentricity
    pub inc: f64,       // Orbital inclination (radians)
    pub node: f64,      // Longitude of ascending node (radians)
    pub peri: f64,      // Argument of periapsis (radians)
    pub m0: f64,        // Mean anomaly at epoch (radians)
    pub period: f64,    // Orbital period (seconds)
    pub radius_km: f64, // Approximate radius (km)
    pub albedo: f64,    // Reflectivity
    pub is_inner: bool, // Orbit oriented in Uranus's 97.77° equatorial plane
}

#[derive(Clone, Debug)]
pub struct UranianMoonSwarm {
    pub moons: Vec<UranianMinorMoon>,
}

impl UranianMoonSwarm {
    pub fn new() -> Self {
        let mut moons = Vec::with_capacity(URANUS_MINOR_MOON_COUNT);

        let make_period = |a: f64| -> f64 {
            TWO_PI * (a.powi(3) / GM_URANUS).sqrt()
        };

        // 1. Inner Shepherds & Portia Group (13 moons)
        let inners = [
            ("Cordelia", 49_770.0, 0.0003, 0.085_f64.to_radians(), 20.1, 0.08, UranianMoonGroup::InnerShepherd),
            ("Ophelia", 53_790.0, 0.0099, 0.104_f64.to_radians(), 21.4, 0.08, UranianMoonGroup::InnerShepherd),
            ("Bianca", 59_170.0, 0.0009, 0.193_f64.to_radians(), 27.0, 0.08, UranianMoonGroup::InnerPortia),
            ("Cressida", 61_780.0, 0.0004, 0.006_f64.to_radians(), 41.0, 0.08, UranianMoonGroup::InnerPortia),
            ("Desdemona", 62_680.0, 0.0001, 0.113_f64.to_radians(), 35.0, 0.08, UranianMoonGroup::InnerPortia),
            ("Juliet", 64_350.0, 0.0007, 0.065_f64.to_radians(), 53.0, 0.08, UranianMoonGroup::InnerPortia),
            ("Portia", 66_090.0, 0.0001, 0.059_f64.to_radians(), 67.5, 0.08, UranianMoonGroup::InnerPortia),
            ("Rosalind", 69_940.0, 0.0001, 0.279_f64.to_radians(), 36.0, 0.08, UranianMoonGroup::InnerPortia),
            ("Cupid", 74_800.0, 0.0013, 0.100_f64.to_radians(), 9.0, 0.07, UranianMoonGroup::InnerPortia),
            ("Belinda", 75_260.0, 0.0001, 0.031_f64.to_radians(), 45.0, 0.08, UranianMoonGroup::InnerPortia),
            ("Perdita", 76_400.0, 0.0012, 0.470_f64.to_radians(), 15.0, 0.08, UranianMoonGroup::InnerPortia),
            ("Puck", 86_010.0, 0.0001, 0.319_f64.to_radians(), 81.0, 0.11, UranianMoonGroup::InnerPortia),
            ("Mab", 97_730.0, 0.0025, 0.134_f64.to_radians(), 12.0, 0.10, UranianMoonGroup::InnerPortia),
        ];

        for (idx, &(name, a, e, inc, r, alb, grp)) in inners.iter().enumerate() {
            moons.push(UranianMinorMoon {
                name,
                group: grp,
                a,
                e,
                inc,
                node: (idx as f64 * 0.48).rem_euclid(TWO_PI),
                peri: (idx as f64 * 0.72).rem_euclid(TWO_PI),
                m0: (idx as f64 * 0.95).rem_euclid(TWO_PI),
                period: make_period(a),
                radius_km: r,
                albedo: alb,
                is_inner: true,
            });
        }

        // 2. Outer Irregulars (10 moons)
        let outers = [
            ("Francisco", 4_276_000.0, 0.1459, 145.2_f64.to_radians(), 11.0, 0.04),
            ("Caliban", 7_169_000.0, 0.2005, 141.5_f64.to_radians(), 36.0, 0.04),
            ("Stephano", 7_948_000.0, 0.2292, 144.1_f64.to_radians(), 16.0, 0.04),
            ("Trinculo", 8_504_000.0, 0.2200, 167.1_f64.to_radians(), 9.0, 0.04),
            ("Sycorax", 12_179_000.0, 0.5224, 159.4_f64.to_radians(), 75.0, 0.04),
            ("Margaret", 14_345_000.0, 0.6608, 56.6_f64.to_radians(), 10.0, 0.04),
            ("Prospero", 16_256_000.0, 0.4448, 152.0_f64.to_radians(), 25.0, 0.04),
            ("Setebos", 17_418_000.0, 0.5914, 158.2_f64.to_radians(), 24.0, 0.04),
            ("Ferdinand", 20_901_000.0, 0.3682, 169.8_f64.to_radians(), 10.0, 0.04),
            ("S/2023 U 1", 7_999_000.0, 0.2100, 144.0_f64.to_radians(), 4.0, 0.04),
        ];

        for (idx, &(name, a, e, inc, r, alb)) in outers.iter().enumerate() {
            moons.push(UranianMinorMoon {
                name,
                group: UranianMoonGroup::OuterIrregular,
                a,
                e,
                inc,
                node: (idx as f64 * 0.63).rem_euclid(TWO_PI),
                peri: (idx as f64 * 0.81).rem_euclid(TWO_PI),
                m0: (idx as f64 * 1.11).rem_euclid(TWO_PI),
                period: make_period(a),
                radius_km: r,
                albedo: alb,
                is_inner: false,
            });
        }

        Self { moons }
    }

    /// Calculate world 3D position of minor moon `idx` relative to Sun at time `t`.
    /// For inner moons, tilt the orbital plane by Uranus's 97.77° axial tilt.
    #[inline]
    pub fn calculate_position(&self, idx: usize, uranus_pos: Vec3, t: f64) -> Vec3 {
        let moon = &self.moons[idx];

        let n = TWO_PI / moon.period;
        let m = (moon.m0 + n * t).rem_euclid(TWO_PI);

        let mut e_anom = m;
        for _ in 0..6 {
            let delta = (m - (e_anom - moon.e * e_anom.sin())) / (1.0 - moon.e * e_anom.cos());
            e_anom += delta;
            if delta.abs() < 1e-8 {
                break;
            }
        }

        let cos_e = e_anom.cos();
        let sin_e = e_anom.sin();
        let r = moon.a * (1.0 - moon.e * cos_e);
        let nu = (1.0 - moon.e * moon.e).sqrt() * sin_e.atan2(cos_e - moon.e);

        let nu_w = nu + moon.peri;
        let cos_nu_w = nu_w.cos();
        let sin_nu_w = nu_w.sin();
        let cos_node = moon.node.cos();
        let sin_node = moon.node.sin();
        let cos_i = moon.inc.cos();
        let sin_i = moon.inc.sin();

        let rel_x = r * (cos_node * cos_nu_w - sin_node * sin_nu_w * cos_i);
        let rel_y = r * (sin_node * cos_nu_w + cos_node * sin_nu_w * cos_i);
        let rel_z = r * (sin_i * sin_nu_w);

        let mut rel = Vec3::new(rel_x, rel_y, rel_z);
        if moon.is_inner {
            let tilt_mat = Mat3::rotation_x(URANUS_AXIAL_TILT_RAD);
            rel = tilt_mat.mul_vec(rel);
        }

        uranus_pos + rel
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.moons.len()
    }
}
