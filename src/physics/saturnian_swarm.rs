//! Analytical Keplerian model for Saturn's 141 minor moons:
//! - Ring shepherds & co-orbitals (Pan, Daphnis, Atlas, Prometheus, Pandora, Epimetheus, Janus).
//! - Trojan & intermediate satellites (Telesto, Calypso, Tethys, Dione, Helene, Polydeuces, Hyperion).
//! - Inuit Group (prograde irregulars, inc ~45°).
//! - Gallic Group (prograde irregulars, inc ~36°).
//! - Norse Group (retrograde irregulars, inc ~135° - 177°, Phoebe family).
//! - Co-planar coordinate system matching ephemeris.rs (orbital motion in X-Y, inclination on Z).

use crate::math::Vec3;
use std::f64::consts::PI;

pub const SATURN_MINOR_MOON_COUNT: usize = 141;
const TWO_PI: f64 = 2.0 * PI;
const GM_SATURN: f64 = 3.7931187e7; // km^3 / s^2

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaturnianMoonGroup {
    RingShepherd,
    TrojanIntermediate,
    Inuit,  // Outer prograde
    Gallic, // Outer prograde
    Norse,  // Outer retrograde
}

#[derive(Clone, Debug)]
pub struct SaturnianMinorMoon {
    pub name: &'static str,
    pub group: SaturnianMoonGroup,
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
pub struct SaturnianMoonSwarm {
    pub moons: Vec<SaturnianMinorMoon>,
}

impl SaturnianMoonSwarm {
    pub fn new() -> Self {
        let mut moons = Vec::with_capacity(SATURN_MINOR_MOON_COUNT);

        let make_period = |a: f64| -> f64 {
            TWO_PI * (a.powi(3) / GM_SATURN).sqrt()
        };

        // 1. Ring Shepherds & Co-orbitals (8 moons)
        let shepherds = [
            ("Pan", 133_584.0, 0.0001, 0.001_f64.to_radians(), 14.1, 0.50),
            ("Daphnis", 136_505.0, 0.0001, 0.004_f64.to_radians(), 4.3, 0.50),
            ("Atlas", 137_670.0, 0.0012, 0.003_f64.to_radians(), 15.1, 0.40),
            ("Prometheus", 139_380.0, 0.0022, 0.008_f64.to_radians(), 43.1, 0.60),
            ("Pandora", 141_720.0, 0.0042, 0.050_f64.to_radians(), 40.7, 0.60),
            ("Epimetheus", 151_410.0, 0.0098, 0.351_f64.to_radians(), 58.1, 0.73),
            ("Janus", 151_460.0, 0.0068, 0.163_f64.to_radians(), 89.5, 0.71),
            ("Aegaeon", 167_500.0, 0.0002, 0.001_f64.to_radians(), 0.3, 0.15),
        ];

        for (idx, &(name, a, e, inc, r, alb)) in shepherds.iter().enumerate() {
            let node = (idx as f64 * 1.37) % TWO_PI;
            let peri = (idx as f64 * 1.91) % TWO_PI;
            let m0 = (idx as f64 * 2.23) % TWO_PI;
            moons.push(SaturnianMinorMoon {
                name,
                group: SaturnianMoonGroup::RingShepherd,
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

        // 2. Trojan & Intermediate Satellites (7 moons)
        let intermediate = [
            ("Telesto", 294_660.0, 0.0001, 1.158_f64.to_radians(), 12.4, 1.00),
            ("Calypso", 294_660.0, 0.0001, 1.499_f64.to_radians(), 10.7, 1.34),
            ("Tethys", 294_660.0, 0.0001, 1.091_f64.to_radians(), 531.1, 1.22),
            ("Dione", 377_400.0, 0.0022, 0.028_f64.to_radians(), 561.4, 0.99),
            ("Helene", 377_400.0, 0.0071, 0.213_f64.to_radians(), 17.6, 1.67),
            ("Polydeuces", 377_400.0, 0.0192, 0.177_f64.to_radians(), 1.3, 0.60),
            ("Hyperion", 1_481_100.0, 0.1042, 0.430_f64.to_radians(), 135.0, 0.30),
        ];

        for (idx, &(name, a, e, inc, r, alb)) in intermediate.iter().enumerate() {
            let node = (idx as f64 * 1.15) % TWO_PI;
            let peri = (idx as f64 * 2.45) % TWO_PI;
            let m0 = (idx as f64 * 0.95) % TWO_PI;
            moons.push(SaturnianMinorMoon {
                name,
                group: SaturnianMoonGroup::TrojanIntermediate,
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

        // Deterministic PRNG for remaining irregular outer satellites
        let mut state: u64 = 0x5A70_BABE_CAFE_1337;
        let mut next_f64 = || -> f64 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 11) as f64) / ((1u64 << 53) as f64)
        };

        // 3. Inuit Group (prograde irregulars, 8 moons: Kiviuq, Ijiraq, Paaliaq, Siarnaq, Tarqeq)
        let inuit_names = ["Kiviuq", "Ijiraq", "Paaliaq", "Siarnaq", "Tarqeq"];
        for i in 0..8 {
            let name = if i < inuit_names.len() { inuit_names[i] } else { "Inuit-grp" };
            let a = 11_100_000.0 + next_f64() * 6_900_000.0;
            let e = 0.15 + next_f64() * 0.20;
            let inc = (44.0 + next_f64() * 6.0).to_radians();
            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;
            let r = if i == 3 { 20.0 } else { 2.0 + next_f64() * 6.0 };
            moons.push(SaturnianMinorMoon {
                name,
                group: SaturnianMoonGroup::Inuit,
                a,
                e,
                inc,
                node,
                peri,
                m0,
                period: make_period(a),
                radius_km: r,
                albedo: 0.06,
            });
        }

        // 4. Gallic Group (prograde irregulars, 4 moons: Albiorix, Bebhionn, Erriapus, Tarvos)
        let gallic_names = ["Albiorix", "Bebhionn", "Erriapus", "Tarvos"];
        for i in 0..4 {
            let name = gallic_names[i];
            let a = 16_100_000.0 + next_f64() * 2_100_000.0;
            let e = 0.45 + next_f64() * 0.10;
            let inc = (35.0 + next_f64() * 5.0).to_radians();
            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;
            let r = if i == 0 { 16.0 } else { 3.0 + next_f64() * 5.0 };
            moons.push(SaturnianMinorMoon {
                name,
                group: SaturnianMoonGroup::Gallic,
                a,
                e,
                inc,
                node,
                peri,
                m0,
                period: make_period(a),
                radius_km: r,
                albedo: 0.06,
            });
        }

        // 5. Norse Group (retrograde irregulars, remaining 114 moons including Phoebe)
        let norse_names = [
            "Phoebe", "Skathi", "Mundilfari", "Narvi", "Suttungr", "Thrymr", "Ymir",
            "Aegir", "Beli", "Bergelmir", "Bestla", "Farbauti", "Fenrir", "Fornjot",
            "Greip", "Hati", "Hyrrokkin", "Jarnsaxa", "Kari", "Loge", "Skoll", "Surtr"
        ];
        let remaining = SATURN_MINOR_MOON_COUNT - moons.len();
        for i in 0..remaining {
            let name = if i < norse_names.len() { norse_names[i] } else { "Norse-grp" };
            let a = 12_900_000.0 + next_f64() * 13_600_000.0;
            let e = 0.15 + next_f64() * 0.35;
            let inc = (135.0 + next_f64() * 42.0).to_radians();
            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;
            let r = if i == 0 { 106.5 } else { 1.5 + next_f64() * 3.5 };
            moons.push(SaturnianMinorMoon {
                name,
                group: SaturnianMoonGroup::Norse,
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

        Self { moons }
    }

    /// Calculate world 3D position of minor moon `idx` relative to Sun at time `t`.
    /// Saturn's current position is passed in so the orbit is evaluated in Saturn's frame.
    #[inline]
    pub fn calculate_position(&self, idx: usize, saturn_pos: Vec3, t: f64) -> Vec3 {
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

        saturn_pos + Vec3::new(rel_x, rel_y, rel_z)
    }

    #[inline]
    pub fn count(&self) -> usize {
        self.moons.len()
    }
}
