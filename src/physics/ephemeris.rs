//! Keplerian orbital mechanics and high-precision J2000 ephemeris.

use crate::math::Vec3;
use std::f64::consts::PI;

pub const TWO_PI: f64 = 2.0 * PI;
pub const AU_KM: f64 = 149_597_870.7; // 1 Astronomical Unit in km
pub const EARTH_RADIUS_KM: f64 = 6_371.0;
pub const MOON_RADIUS_KM: f64 = 1_737.4;
pub const MERCURY_RADIUS_KM: f64 = 2_439.7;
pub const VENUS_RADIUS_KM: f64 = 6_051.8;
pub const MARS_RADIUS_KM: f64 = 3_389.5;
pub const PHOBOS_MEAN_RADIUS_KM: f64 = 11.26;
pub const DEIMOS_MEAN_RADIUS_KM: f64 = 6.2;
pub const PHOBOS_SEMI_AXES_KM: (f64, f64, f64) = (13.0, 11.4, 9.1);
pub const DEIMOS_SEMI_AXES_KM: (f64, f64, f64) = (7.5, 6.1, 5.5);
pub const SUN_RADIUS_KM: f64 = 696_340.0;
pub const EARTH_AXIAL_TILT_RAD: f64 = 23.439281 * PI / 180.0;
pub const MERCURY_AXIAL_TILT_RAD: f64 = 0.034 * PI / 180.0;
pub const VENUS_AXIAL_TILT_RAD: f64 = 177.36 * PI / 180.0;
pub const MARS_AXIAL_TILT_RAD: f64 = 25.19 * PI / 180.0;
pub const SIDEREAL_DAY_SECS: f64 = 86_164.0905;
pub const MERCURY_SIDEREAL_DAY_SECS: f64 = 58.6462 * 86_400.0;
pub const VENUS_SIDEREAL_DAY_SECS: f64 = 243.025 * 86_400.0;
pub const MARS_SIDEREAL_DAY_SECS: f64 = 88_642.68; // 24h 37m 22.7s (Martian Sol)

/// Keplerian orbital elements defining a two-body orbit.
#[derive(Clone, Copy, Debug)]
pub struct KeplerOrbit {
    /// Semi-major axis in km
    pub a: f64,
    /// Orbital eccentricity (0.0 = circular, 0..1 = elliptical)
    pub e: f64,
    /// Orbital inclination in radians
    pub i: f64,
    /// Longitude of ascending node (Omega) in radians
    pub node: f64,
    /// Argument of periapsis (omega) in radians
    pub peri: f64,
    /// Mean anomaly at epoch t0 (M0) in radians
    pub m0: f64,
    /// Orbital period in seconds
    pub period: f64,
}

impl KeplerOrbit {
    /// Mercury's orbit around the Sun (J2000 Keplerian elements)
    pub fn mercury_around_sun() -> Self {
        Self {
            a: 57_909_050.0, // 0.387098 AU in km
            e: 0.205630,     // High eccentricity!
            i: 7.005 * PI / 180.0,
            node: 48.331 * PI / 180.0,
            peri: 29.124 * PI / 180.0,
            m0: 174.796 * PI / 180.0,
            period: 87.9691 * 86_400.0,
        }
    }

    /// Venus's orbit around the Sun (J2000 Keplerian elements)
    pub fn venus_around_sun() -> Self {
        Self {
            a: 108_208_000.0, // 0.723332 AU in km
            e: 0.006772,      // Nearly circular orbit
            i: 3.39458 * PI / 180.0,
            node: 76.680 * PI / 180.0,
            peri: 131.532 * PI / 180.0,
            m0: 50.115 * PI / 180.0,
            period: 224.701 * 86_400.0,
        }
    }

    /// Earth's orbit around the Sun (J2000 approximate)
    pub fn earth_around_sun() -> Self {
        Self {
            a: AU_KM,
            e: 0.0167086,
            i: 0.0, // Ecliptic reference plane
            node: -11.26064 * PI / 180.0,
            peri: 102.94719 * PI / 180.0,
            m0: 100.46435 * PI / 180.0,
            period: 365.256363 * 86_400.0,
        }
    }

    /// Moon's orbit around the Earth
    pub fn moon_around_earth() -> Self {
        Self {
            a: 384_400.0,
            e: 0.0549,
            i: 5.145 * PI / 180.0,
            node: 125.08 * PI / 180.0,
            peri: 318.15 * PI / 180.0,
            m0: 135.0 * PI / 180.0,
            period: 27.321582 * 86_400.0,
        }
    }

    /// Mars's orbit around the Sun (J2000 Keplerian elements)
    pub fn mars_around_sun() -> Self {
        Self {
            a: 227_939_200.0, // 1.52368 AU in km
            e: 0.0934,        // Distinctly eccentric orbit (perihelion 206.6M km, aphelion 249.2M km)
            i: 1.850 * PI / 180.0,
            node: 49.5785 * PI / 180.0,
            peri: 286.502 * PI / 180.0,
            m0: 19.373 * PI / 180.0,
            period: 686.98 * 86_400.0,
        }
    }

    /// Phobos's orbit around Mars (ultra-close, fast 7.65h inner moon)
    pub fn phobos_around_mars() -> Self {
        Self {
            a: 9_376.0,
            e: 0.0151,
            i: 1.093 * PI / 180.0,
            node: 0.0,
            peri: 0.0,
            m0: 0.0,
            period: 7.65384 * 3_600.0, // 27,553.8 s (faster than Mars's rotation!)
        }
    }

    /// Deimos's orbit around Mars (outer moon, 30.3h period)
    pub fn deimos_around_mars() -> Self {
        Self {
            a: 23_463.0,
            e: 0.0002,
            i: 0.93 * PI / 180.0,
            node: 0.0,
            peri: 0.0,
            m0: 1.20,
            period: 30.298 * 3_600.0, // 109,072.8 s
        }
    }

    /// Solve Kepler's equation M = E - e*sin(E) for Eccentric Anomaly E using Newton-Raphson.
    pub fn solve_eccentric_anomaly(&self, mean_anomaly: f64) -> f64 {
        let m = mean_anomaly.rem_euclid(TWO_PI);
        let mut e_anom = if self.e > 0.8 { PI } else { m };

        for _ in 0..10 {
            let f = e_anom - self.e * e_anom.sin() - m;
            let f_prime = 1.0 - self.e * e_anom.cos();
            let delta = f / f_prime;
            e_anom -= delta;
            if delta.abs() < 1e-12 {
                break;
            }
        }
        e_anom
    }

    /// Calculate relative position (x, y, z in km) at time t (seconds from epoch).
    pub fn calculate_position(&self, t: f64) -> Vec3 {
        let mean_motion = TWO_PI / self.period;
        let m = self.m0 + mean_motion * t;
        let e_anom = self.solve_eccentric_anomaly(m);

        // Position in orbital plane
        let (sin_e, cos_e) = e_anom.sin_cos();
        let sqrt_one_minus_e2 = (1.0 - self.e * self.e).max(0.0).sqrt();
        let x_orb = self.a * (cos_e - self.e);
        let y_orb = self.a * sqrt_one_minus_e2 * sin_e;

        // Transformation to 3D ecliptic coordinates:
        // Rz(-node) * Rx(-i) * Rz(-peri) * [x_orb, y_orb, 0]
        let (sin_p, cos_p) = self.peri.sin_cos();
        let (sin_i, cos_i) = self.i.sin_cos();
        let (sin_n, cos_n) = self.node.sin_cos();

        // 1. Rotate by argument of periapsis (around z)
        let x1 = cos_p * x_orb - sin_p * y_orb;
        let y1 = sin_p * x_orb + cos_p * y_orb;

        // 2. Rotate by inclination (around x)
        let x2 = x1;
        let y2 = cos_i * y1;
        let z2 = sin_i * y1;

        // 3. Rotate by longitude of ascending node (around z)
        let x3 = cos_n * x2 - sin_n * y2;
        let y3 = sin_n * x2 + cos_n * y2;
        let z3 = z2;

        Vec3::new(x3, y3, z3)
    }

    /// Compute orbital speed in km/s at distance r from primary body with mass M.
    pub fn orbital_speed(&self, r: f64, standard_gravitational_param: f64) -> f64 {
        // Vis-viva equation: v^2 = GM * (2/r - 1/a)
        let v2 = standard_gravitational_param * (2.0 / r - 1.0 / self.a);
        v2.max(0.0).sqrt()
    }
}
