//! Analytical Keplerian model for Neptune's 14 minor moons:
//! - Inner ring shepherds & regular moons (Naiad, Thalassa, Despina, Galatea, Larissa, Hippocamp).
//! - Nereid (classic ultra-eccentric moon e=0.7512).
//! - Outer irregulars (Halimede, Sao, Laomedeia, Psamathe, Neso - outermost moon in solar system at ~50M km, S/2002 N 5, S/2021 N 1).
//! - Coordinates aligned with ephemeris.rs with equatorial plane tilted by Neptune's axial tilt (28.32°).

use crate::math::{Mat3, Vec3};
use crate::physics::ephemeris::NEPTUNE_AXIAL_TILT_RAD;
use std::f64::consts::PI;

pub const NEPTUNE_MINOR_MOON_COUNT: usize = 14;
const TWO_PI: f64 = 2.0 * PI;
const GM_NEPTUNE: f64 = 6.83515e6; // km^3 / s^2

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NeptunianMoonGroup {
    InnerRegular,
    Nereid,
    OuterIrregular,
}

#[derive(Clone, Debug)]
pub struct NeptunianMinorMoon {
    pub name: &'static str,
    pub group: NeptunianMoonGroup,
    pub a: f64,         // Semi-major axis (km)
    pub e: f64,         // Eccentricity
    pub inc: f64,       // Orbital inclination (radians)
    pub node: f64,      // Longitude of ascending node (radians)
    pub peri: f64,      // Argument of periapsis (radians)
    pub m0: f64,        // Mean anomaly at epoch (radians)
    pub period: f64,    // Orbital period (seconds)
    pub radius_km: f64, // Approximate radius (km)
    pub albedo: f64,    // Reflectivity
    pub is_inner: bool, // Orbit oriented in Neptune's 28.32° equatorial plane
}

#[derive(Clone, Debug)]
pub struct NeptunianMoonSwarm {
    pub moons: Vec<NeptunianMinorMoon>,
}

impl NeptunianMoonSwarm {
    pub fn new() -> Self {
        let mut moons = Vec::with_capacity(NEPTUNE_MINOR_MOON_COUNT);

        let make_period = |a: f64| -> f64 {
            TWO_PI * (a.powi(3) / GM_NEPTUNE).sqrt()
        };

        // 1. Inner Regulars (6 moons)
        let inners = [
            ("Naiad", 48_227.0, 0.0047, 4.750_f64.to_radians(), 33.0, 0.07),
            ("Thalassa", 50_074.0, 0.0018, 0.210_f64.to_radians(), 41.0, 0.09),
            ("Despina", 52_526.0, 0.0004, 0.070_f64.to_radians(), 75.0, 0.09),
            ("Galatea", 61_953.0, 0.0001, 0.050_f64.to_radians(), 87.4, 0.08),
            ("Larissa", 73_548.0, 0.0014, 0.200_f64.to_radians(), 97.0, 0.09),
            ("Hippocamp", 105_283.0, 0.0005, 0.060_f64.to_radians(), 17.4, 0.09),
        ];

        for (idx, &(name, a, e, inc, r, alb)) in inners.iter().enumerate() {
            moons.push(NeptunianMinorMoon {
                name,
                group: NeptunianMoonGroup::InnerRegular,
                a,
                e,
                inc,
                node: (idx as f64 * 0.52).rem_euclid(TWO_PI),
                peri: (idx as f64 * 0.77).rem_euclid(TWO_PI),
                m0: (idx as f64 * 0.91).rem_euclid(TWO_PI),
                period: make_period(a),
                radius_km: r,
                albedo: alb,
                is_inner: true,
            });
        }

        // 2. Nereid (Extreme eccentric orbit e=0.7512)
        let nereid_a = 5_513_818.0;
        let nereid_e = 0.7512;
        let nereid_inc = 7.230_f64.to_radians();
        moons.push(NeptunianMinorMoon {
            name: "Nereid",
            group: NeptunianMoonGroup::Nereid,
            a: nereid_a,
            e: nereid_e,
            inc: nereid_inc,
            node: 1.25,
            peri: 3.14,
            m0: 0.50,
            period: make_period(nereid_a),
            radius_km: 170.0,
            albedo: 0.155,
            is_inner: false,
        });

        // 3. Outer Irregulars (7 moons)
        let outers = [
            ("Halimede", 16_611_000.0, 0.2646, 112.7_f64.to_radians(), 31.0, 0.04),
            ("Sao", 22_228_000.0, 0.1365, 53.4_f64.to_radians(), 22.0, 0.04),
            ("Laomedeia", 23_567_000.0, 0.3969, 37.9_f64.to_radians(), 21.0, 0.04),
            ("Psamathe", 48_096_000.0, 0.4617, 126.3_f64.to_radians(), 20.0, 0.04),
            ("Neso", 49_285_000.0, 0.5714, 136.4_f64.to_radians(), 30.0, 0.04),
            ("S/2002 N 5", 23_400_000.0, 0.5400, 42.1_f64.to_radians(), 12.0, 0.04),
            ("S/2021 N 1", 50_600_000.0, 0.4400, 134.5_f64.to_radians(), 7.0, 0.04),
        ];

        for (idx, &(name, a, e, inc, r, alb)) in outers.iter().enumerate() {
            moons.push(NeptunianMinorMoon {
                name,
                group: NeptunianMoonGroup::OuterIrregular,
                a,
                e,
                inc,
                node: (idx as f64 * 0.65).rem_euclid(TWO_PI),
                peri: (idx as f64 * 0.88).rem_euclid(TWO_PI),
                m0: (idx as f64 * 1.05).rem_euclid(TWO_PI),
                period: make_period(a),
                radius_km: r,
                albedo: alb,
                is_inner: false,
            });
        }

        Self { moons }
    }

    /// Calculate world 3D position of minor moon `idx` relative to Sun at time `t`.
    /// For inner moons, tilt the orbital plane by Neptune's 28.32° axial tilt.
    #[inline]
    pub fn calculate_position(&self, idx: usize, neptune_pos: Vec3, t: f64) -> Vec3 {
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
            let tilt_mat = Mat3::rotation_x(NEPTUNE_AXIAL_TILT_RAD);
            rel = tilt_mat.mul_vec(rel);
        }

        neptune_pos + rel
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.moons.len()
    }
}
