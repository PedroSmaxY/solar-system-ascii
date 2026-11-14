//! Celestial body representation and system hierarchy.

use crate::math::{Mat3, Vec3};
use crate::physics::ephemeris::{
    KeplerOrbit, EARTH_AXIAL_TILT_RAD, EARTH_RADIUS_KM, MOON_RADIUS_KM, SIDEREAL_DAY_SECS,
    SUN_RADIUS_KM,
};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyId {
    Sun,
    Earth,
    Moon,
}

impl BodyId {
    pub fn index(self) -> usize {
        match self {
            BodyId::Sun => 0,
            BodyId::Earth => 1,
            BodyId::Moon => 2,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CelestialBody {
    pub id: BodyId,
    pub name: &'static str,
    pub radius_km: f64,
    pub mass_kg: f64,
    pub axial_tilt: f64,
    pub rotation_period_secs: f64,
    pub orbit: Option<KeplerOrbit>,
    pub parent: Option<BodyId>,
    
    // Dynamic state updated each frame
    pub position: Vec3,      // World position in km
    pub rotation_angle: f64, // Current rotation angle in radians
}

impl CelestialBody {
    pub fn new_sun() -> Self {
        Self {
            id: BodyId::Sun,
            name: "Sun",
            radius_km: SUN_RADIUS_KM,
            mass_kg: 1.98847e30,
            axial_tilt: 7.25 * PI / 180.0,
            rotation_period_secs: 25.05 * 86_400.0,
            orbit: None,
            parent: None,
            position: Vec3::ZERO,
            rotation_angle: 0.0,
        }
    }

    pub fn new_earth() -> Self {
        Self {
            id: BodyId::Earth,
            name: "Earth",
            radius_km: EARTH_RADIUS_KM,
            mass_kg: 5.9722e24,
            axial_tilt: EARTH_AXIAL_TILT_RAD,
            rotation_period_secs: SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::earth_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(149_597_870.7, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_moon() -> Self {
        Self {
            id: BodyId::Moon,
            name: "Moon",
            radius_km: MOON_RADIUS_KM,
            mass_kg: 7.342e22,
            axial_tilt: 1.5424 * PI / 180.0,
            // Tidally locked to Earth
            rotation_period_secs: 27.321582 * 86_400.0,
            orbit: Some(KeplerOrbit::moon_around_earth()),
            parent: Some(BodyId::Earth),
            position: Vec3::new(149_597_870.7 + 384_400.0, 0.0, 0.0),
            rotation_angle: PI,
        }
    }

    /// Calculate orientation matrix for the body (axial tilt + current spin).
    pub fn orientation_matrix(&self) -> Mat3 {
        // Axial tilt is around X axis, rotation is around tilted Z/Y axis
        let tilt_mat = Mat3::rotation_x(self.axial_tilt);
        let spin_mat = Mat3::rotation_y(self.rotation_angle);
        tilt_mat.mul_mat(spin_mat)
    }
}
