//! Analytical Keplerian model for Jupiter's 91 minor moons:
//! - Inner Amalthea Group (Metis, Adrastea, Amalthea, Thebe): regular, low-eccentricity, low-inclination equatorial orbits.
//! - Himalia Group: outer prograde irregular satellites (~11.5M km, inc ~28°).
//! - Ananke, Carme, and Pasiphae Groups: outer retrograde irregular satellites (~19M - 24M km, inc ~145° - 165°).
//! - Co-planar coordinate system matching ephemeris.rs (orbital motion in X-Y, inclination on Z).

use crate::math::Vec3;
use std::f64::consts::PI;

pub const MINOR_MOON_COUNT: usize = 91;
const TWO_PI: f64 = 2.0 * PI;
const GM_JUPITER: f64 = 1.2668653e8; // km^3 / s^2

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JovianMoonGroup {
    Amalthea, // Inner regular
    Himalia,  // Outer prograde irregular
    Ananke,   // Outer retrograde irregular
    Carme,    // Outer retrograde irregular
    Pasiphae, // Outer retrograde irregular
    Other,
}

#[derive(Clone, Debug)]
pub struct JovianMinorMoon {
    pub name: &'static str,
    pub group: JovianMoonGroup,
    pub a: f64,         // Semi-major axis (km)
    pub e: f64,         // Eccentricity
    pub inc: f64,       // Orbital inclination (radians)
    pub node: f64,      // Longitude of ascending node (radians)
    pub peri: f64,      // Argument of periapsis (radians)
    pub m0: f64,        // Mean anomaly at epoch (radians)
    pub period: f64,    // Orbital period (seconds)
    pub radius_km: f64, // Approximate radius (km)
    pub albedo: f64,    // Reflectivity
}

#[derive(Clone, Debug)]
pub struct JovianMoonSwarm {
    pub moons: Vec<JovianMinorMoon>,
}

impl JovianMoonSwarm {
    pub fn new() -> Self {
        let mut moons = Vec::with_capacity(MINOR_MOON_COUNT);

        let make_period = |a: f64| -> f64 {
            TWO_PI * (a.powi(3) / GM_JUPITER).sqrt()
        };

        // 1. Inner Amalthea Group (4 regular moons)
        let inner = [
            ("Metis", 128_000.0, 0.0002, 0.06_f64.to_radians(), 21.5, 0.06),
            ("Adrastea", 129_000.0, 0.0015, 0.03_f64.to_radians(), 8.2, 0.10),
            ("Amalthea", 181_400.0, 0.0032, 0.37_f64.to_radians(), 83.5, 0.09),
            ("Thebe", 222_000.0, 0.0175, 1.08_f64.to_radians(), 49.3, 0.047),
        ];

        for (idx, &(name, a, e, inc, r, alb)) in inner.iter().enumerate() {
            let node = (idx as f64 * 1.57) % TWO_PI;
            let peri = (idx as f64 * 2.11) % TWO_PI;
            let m0 = (idx as f64 * 1.73) % TWO_PI;
            moons.push(JovianMinorMoon {
                name,
                group: JovianMoonGroup::Amalthea,
                a,
                e,
                inc,
                node,
                peri,
                m0,
                period: make_period(a),
                radius_km: r,
                albedo: alb,
            });
        }

        // Deterministic PRNG for the remaining 87 irregular satellites
        let mut state: u64 = 0xCAFE_BABE_5001_1337;
        let mut next_f64 = || -> f64 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 11) as f64) / ((1u64 << 53) as f64)
        };

        // 2. Himalia Group (prograde, ~10 moons around 11.5M km, inc ~27.5°)
        let himalia_names = ["Himalia", "Leda", "Lysithea", "Elara", "Dia", "Pandia", "Ersa"];
        for i in 0..10 {
            let name = if i < himalia_names.len() { himalia_names[i] } else { "Himalia-grp" };
            let a = 11_150_000.0 + next_f64() * 1_400_000.0;
            let e = 0.12 + next_f64() * 0.10;
            let inc = (27.0 + next_f64() * 3.0).to_radians();
            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;
            let r = if i == 0 { 85.0 } else if i == 3 { 43.0 } else { 2.0 + next_f64() * 15.0 };
            moons.push(JovianMinorMoon {
                name,
                group: JovianMoonGroup::Himalia,
                a,
                e,
                inc,
                node,
                peri,
                m0,
                period: make_period(a),
                radius_km: r,
                albedo: 0.05,
            });
        }

        // 3. Ananke Group (retrograde, ~26 moons, 19.3M - 22.7M km, inc ~145° - 155°)
        let ananke_names = ["Ananke", "Praxidike", "Harpalyke", "Iocaste", "Euporie", "Mneme"];
        for i in 0..26 {
            let name = if i < ananke_names.len() { ananke_names[i] } else { "Ananke-grp" };
            let a = 19_300_000.0 + next_f64() * 3_400_000.0;
            let e = 0.14 + next_f64() * 0.14;
            let inc = (145.0 + next_f64() * 10.0).to_radians();
            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;
            let r = if i == 0 { 14.0 } else { 1.5 + next_f64() * 4.0 };
            moons.push(JovianMinorMoon {
                name,
                group: JovianMoonGroup::Ananke,
                a,
                e,
                inc,
                node,
                peri,
                m0,
                period: make_period(a),
                radius_km: r,
                albedo: 0.04,
            });
        }

        // 4. Carme Group (retrograde, ~26 moons, 22.9M - 24.1M km, inc ~163° - 166°)
        let carme_names = ["Carme", "Taygete", "Chaldene", "Kalyke", "Callirrhoe", "Arche"];
        for i in 0..26 {
            let name = if i < carme_names.len() { carme_names[i] } else { "Carme-grp" };
            let a = 22_900_000.0 + next_f64() * 1_200_000.0;
            let e = 0.22 + next_f64() * 0.08;
            let inc = (163.0 + next_f64() * 4.0).to_radians();
            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;
            let r = if i == 0 { 23.0 } else { 1.0 + next_f64() * 3.5 };
            moons.push(JovianMinorMoon {
                name,
                group: JovianMoonGroup::Carme,
                a,
                e,
                inc,
                node,
                peri,
                m0,
                period: make_period(a),
                radius_km: r,
                albedo: 0.04,
            });
        }

        // 5. Pasiphae Group (retrograde, 25 moons, 22.8M - 24.5M km, inc ~148° - 155°)
        let pasiphae_names = ["Pasiphae", "Sinope", "Megaclite", "Autonoe", "Eurydome", "Sponde"];
        let remaining = MINOR_MOON_COUNT - moons.len();
        for i in 0..remaining {
            let name = if i < pasiphae_names.len() { pasiphae_names[i] } else { "Pasiphae-grp" };
            let a = 22_800_000.0 + next_f64() * 1_700_000.0;
            let e = 0.24 + next_f64() * 0.18;
            let inc = (148.0 + next_f64() * 8.0).to_radians();
            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;
            let r = if i == 0 { 30.0 } else if i == 1 { 19.0 } else { 1.0 + next_f64() * 4.0 };
            moons.push(JovianMinorMoon {
                name,
                group: JovianMoonGroup::Pasiphae,
                a,
                e,
                inc,
                node,
                peri,
                m0,
                period: make_period(a),
                radius_km: r,
                albedo: 0.04,
            });
        }

        Self { moons }
    }

    /// Calculate world 3D position of minor moon `idx` relative to Sun at time `t`.
    /// Jupiter's current position is passed in so the orbit is evaluated in Jupiter's frame.
    #[inline]
    pub fn calculate_position(&self, idx: usize, jupiter_pos: Vec3, t: f64) -> Vec3 {
        let moon = &self.moons[idx];

        // Mean anomaly at time t
        let n = TWO_PI / moon.period;
        let m = (moon.m0 + n * t).rem_euclid(TWO_PI);

        // Solve Kepler equation M = E - e*sin(E)
        let mut e_anom = m;
        for _ in 0..6 {
            let delta = (m - (e_anom - moon.e * e_anom.sin())) / (1.0 - moon.e * e_anom.cos());
            e_anom += delta;
            if delta.abs() < 1e-8 {
                break;
            }
        }

        // True anomaly nu and radius r
        let cos_e = e_anom.cos();
        let sin_e = e_anom.sin();
        let r = moon.a * (1.0 - moon.e * cos_e);
        let nu = (1.0 - moon.e * moon.e).sqrt() * sin_e.atan2(cos_e - moon.e);

        // Standard orbital coordinates aligned with ephemeris.rs (X-Y plane, Z inclination)
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

        jupiter_pos + Vec3::new(rel_x, rel_y, rel_z)
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.moons.len()
    }
}
