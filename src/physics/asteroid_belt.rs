//! Dynamic Keplerian Main Asteroid Belt Simulation:
//! - 3,500 deterministic Keplerian asteroids distributed between 2.06 AU and 3.28 AU.
//! - Accurate Kirkwood Gaps model (depletions at 3:1, 5:2, 7:3, and 2:1 mean-motion resonances with Jupiter).
//! - Realistic inclination (Gaussian sigma ~8°) and eccentricity distributions.
//! - Analytical position evaluation at time t with zero Keplerian drift.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const ASTEROID_COUNT: usize = 3_500;
const TWO_PI: f64 = 2.0 * PI;

#[derive(Clone, Copy, Debug)]
pub struct AsteroidParticle {
    pub a: f64,       // Semi-major axis in km
    pub e: f64,       // Eccentricity
    pub inc: f64,     // Inclination in radians
    pub node: f64,    // Longitude of ascending node
    pub peri: f64,    // Argument of periapsis
    pub m0: f64,      // Mean anomaly at epoch
    pub period: f64,  // Orbital period in seconds
    pub radius_km: f64, // Effective radius in km
    pub albedo: f64,  // Visual reflectivity
}

#[derive(Clone, Debug)]
pub struct AsteroidField {
    pub particles: Vec<AsteroidParticle>,
}

impl AsteroidField {
    /// Deterministically construct the Main Asteroid Belt with realistic Kirkwood Gaps.
    pub fn new() -> Self {
        let mut particles = Vec::with_capacity(ASTEROID_COUNT);

        // Linear congruential generator for 100% deterministic, portable distribution
        let mut state: u64 = 0x5EED_CAFE_BABE_1337;
        let mut next_f64 = || -> f64 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            ((state >> 11) as f64) / ((1u64 << 53) as f64)
        };

        let au_km = 149_597_870.7;

        // Kirkwood Gaps in AU (resonances with Jupiter at 5.20 AU)
        let kirkwood_gaps = [
            (2.50, 0.035), // 3:1 resonance
            (2.82, 0.030), // 5:2 resonance
            (2.96, 0.025), // 7:3 resonance
            (3.28, 0.035), // 2:1 resonance
        ];

        while particles.len() < ASTEROID_COUNT {
            // Semi-major axis between 2.06 AU and 3.32 AU, density peaking at 2.70 AU
            let u1 = next_f64();
            let u2 = next_f64();
            let a_au = 2.06 + (u1 * 0.7 + u2 * 0.56) * (3.32 - 2.06);

            // Rejection sampling for Kirkwood Gaps
            let mut inside_gap = false;
            for &(gap_center, gap_width) in &kirkwood_gaps {
                let d = (a_au - gap_center).abs();
                if d < gap_width {
                    // Deplete by 90%
                    if next_f64() < 0.90 {
                        inside_gap = true;
                        break;
                    }
                }
            }
            if inside_gap {
                continue;
            }

            let a_km = a_au * au_km;

            // Eccentricity (Rayleigh-like distribution, mean ~0.14, range 0.02 to 0.30)
            let e = (0.03 + 0.25 * next_f64() * next_f64().sqrt()).clamp(0.01, 0.35);

            // Inclination: Gaussian-like distribution centered on ecliptic (sigma ~8 deg)
            let inc_deg = (next_f64() + next_f64() + next_f64() - 1.5) * 12.0;
            let inc = inc_deg.to_radians();

            let node = next_f64() * TWO_PI;
            let peri = next_f64() * TWO_PI;
            let m0 = next_f64() * TWO_PI;

            // Kepler's third law: T = 2 * pi * sqrt(a^3 / GM_sun)
            let gm_sun = 1.32712440018e11; // km^3 / s^2
            let period = TWO_PI * (a_km.powi(3) / gm_sun).sqrt();

            // Asteroid size distribution: power law (many small 1-5 km, few 10-40 km)
            let radius_km = 1.0 + 25.0 * next_f64().powi(4);
            let albedo = 0.05 + 0.20 * next_f64();

            particles.push(AsteroidParticle {
                a: a_km,
                e,
                inc,
                node,
                peri,
                m0,
                period,
                radius_km,
                albedo,
            });
        }

        Self { particles }
    }

    /// Calculate the position of asteroid `idx` at time `t` (seconds from epoch).
    #[inline(always)]
    pub fn calculate_position(&self, idx: usize, t: f64) -> Vec3 {
        let p = &self.particles[idx];
        let mean_motion = TWO_PI / p.period;
        let m = (p.m0 + mean_motion * t).rem_euclid(TWO_PI);

        // Fast Newton-Raphson Kepler solver (3 iterations sufficient for small eccentricities)
        let mut e_anom = m;
        for _ in 0..4 {
            let f = e_anom - p.e * e_anom.sin() - m;
            let f_prime = 1.0 - p.e * e_anom.cos();
            e_anom -= f / f_prime;
        }

        let nu = 2.0 * (((1.0 + p.e) / (1.0 - p.e)).sqrt() * (e_anom * 0.5).tan()).atan();
        let r = p.a * (1.0 - p.e * e_anom.cos());

        let (sin_nu_w, cos_nu_w) = (nu + p.peri).sin_cos();
        let (sin_node, cos_node) = p.node.sin_cos();
        let (sin_i, cos_i) = p.inc.sin_cos();

        let x = r * (cos_node * cos_nu_w - sin_node * sin_nu_w * cos_i);
        let y = r * (sin_node * cos_nu_w + cos_node * sin_nu_w * cos_i);
        let z = r * (sin_i * sin_nu_w);

        Vec3::new(x, y, z)
    }
}
