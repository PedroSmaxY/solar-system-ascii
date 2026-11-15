//! Celestial body representation and system hierarchy.

use crate::math::{Mat3, Vec3};
use crate::physics::ephemeris::{
    KeplerOrbit, CALLISTO_RADIUS_KM, CALLISTO_SIDEREAL_DAY_SECS, CERES_AXIAL_TILT_RAD,
    CERES_MEAN_RADIUS_KM, CERES_SIDEREAL_DAY_SECS, DEIMOS_MEAN_RADIUS_KM, EARTH_AXIAL_TILT_RAD,
    EARTH_RADIUS_KM, ENCELADUS_RADIUS_KM, ENCELADUS_SIDEREAL_DAY_SECS, EUROPA_RADIUS_KM,
    EUROPA_SIDEREAL_DAY_SECS, GANYMEDE_RADIUS_KM, GANYMEDE_SIDEREAL_DAY_SECS, IAPETUS_RADIUS_KM,
    IAPETUS_SIDEREAL_DAY_SECS, IO_RADIUS_KM, IO_SIDEREAL_DAY_SECS, JUPITER_AXIAL_TILT_RAD,
    JUPITER_MEAN_RADIUS_KM, JUPITER_SIDEREAL_DAY_SECS, MARS_AXIAL_TILT_RAD, MARS_RADIUS_KM,
    MARS_SIDEREAL_DAY_SECS, MERCURY_AXIAL_TILT_RAD, MERCURY_RADIUS_KM, MERCURY_SIDEREAL_DAY_SECS,
    MIMAS_RADIUS_KM, MIMAS_SIDEREAL_DAY_SECS, MOON_RADIUS_KM, PHOBOS_MEAN_RADIUS_KM,
    RHEA_RADIUS_KM, RHEA_SIDEREAL_DAY_SECS, SATURN_AXIAL_TILT_RAD, SATURN_MEAN_RADIUS_KM,
    SATURN_SIDEREAL_DAY_SECS, SIDEREAL_DAY_SECS, SUN_RADIUS_KM, TITAN_RADIUS_KM,
    TITAN_SIDEREAL_DAY_SECS, VENUS_AXIAL_TILT_RAD, VENUS_RADIUS_KM, VENUS_SIDEREAL_DAY_SECS,
    VESTA_AXIAL_TILT_RAD, VESTA_MEAN_RADIUS_KM, VESTA_SIDEREAL_DAY_SECS,
};
use std::f64::consts::PI;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BodyId {
    Sun,
    Mercury,
    Venus,
    Earth,
    Moon,
    Mars,
    Phobos,
    Deimos,
    Ceres,
    Vesta,
    Jupiter,
    Io,
    Europa,
    Ganymede,
    Callisto,
    Saturn,
    Titan,
    Enceladus,
    Mimas,
    Iapetus,
    Rhea,
}

impl BodyId {
    pub fn index(self) -> usize {
        match self {
            BodyId::Sun => 0,
            BodyId::Mercury => 1,
            BodyId::Venus => 2,
            BodyId::Earth => 3,
            BodyId::Moon => 4,
            BodyId::Mars => 5,
            BodyId::Phobos => 6,
            BodyId::Deimos => 7,
            BodyId::Ceres => 8,
            BodyId::Vesta => 9,
            BodyId::Jupiter => 10,
            BodyId::Io => 11,
            BodyId::Europa => 12,
            BodyId::Ganymede => 13,
            BodyId::Callisto => 14,
            BodyId::Saturn => 15,
            BodyId::Titan => 16,
            BodyId::Enceladus => 17,
            BodyId::Mimas => 18,
            BodyId::Iapetus => 19,
            BodyId::Rhea => 20,
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

    pub fn new_mercury() -> Self {
        Self {
            id: BodyId::Mercury,
            name: "Mercury",
            radius_km: MERCURY_RADIUS_KM,
            mass_kg: 3.3011e23,
            axial_tilt: MERCURY_AXIAL_TILT_RAD,
            rotation_period_secs: MERCURY_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::mercury_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(57_909_050.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_venus() -> Self {
        Self {
            id: BodyId::Venus,
            name: "Venus",
            radius_km: VENUS_RADIUS_KM,
            mass_kg: 4.8675e24,
            axial_tilt: VENUS_AXIAL_TILT_RAD,
            rotation_period_secs: VENUS_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::venus_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(108_208_000.0, 0.0, 0.0),
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

    pub fn new_mars() -> Self {
        Self {
            id: BodyId::Mars,
            name: "Mars",
            radius_km: MARS_RADIUS_KM,
            mass_kg: 6.4171e23,
            axial_tilt: MARS_AXIAL_TILT_RAD,
            rotation_period_secs: MARS_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::mars_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(227_939_200.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_phobos() -> Self {
        Self {
            id: BodyId::Phobos,
            name: "Phobos",
            radius_km: PHOBOS_MEAN_RADIUS_KM,
            mass_kg: 1.0659e16,
            axial_tilt: 0.0,
            // Tidally locked to Mars
            rotation_period_secs: 7.65384 * 3_600.0,
            orbit: Some(KeplerOrbit::phobos_around_mars()),
            parent: Some(BodyId::Mars),
            position: Vec3::new(227_939_200.0 + 9_376.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_deimos() -> Self {
        Self {
            id: BodyId::Deimos,
            name: "Deimos",
            radius_km: DEIMOS_MEAN_RADIUS_KM,
            mass_kg: 1.4762e15,
            axial_tilt: 0.0,
            // Tidally locked to Mars
            rotation_period_secs: 30.298 * 3_600.0,
            orbit: Some(KeplerOrbit::deimos_around_mars()),
            parent: Some(BodyId::Mars),
            position: Vec3::new(227_939_200.0 + 23_463.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_ceres() -> Self {
        Self {
            id: BodyId::Ceres,
            name: "Ceres",
            radius_km: CERES_MEAN_RADIUS_KM,
            mass_kg: 9.383e20,
            axial_tilt: CERES_AXIAL_TILT_RAD,
            rotation_period_secs: CERES_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::ceres_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(413_970_000.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_vesta() -> Self {
        Self {
            id: BodyId::Vesta,
            name: "Vesta",
            radius_km: VESTA_MEAN_RADIUS_KM,
            mass_kg: 2.59e20,
            axial_tilt: VESTA_AXIAL_TILT_RAD,
            rotation_period_secs: VESTA_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::vesta_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(353_370_000.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_jupiter() -> Self {
        Self {
            id: BodyId::Jupiter,
            name: "Jupiter",
            radius_km: JUPITER_MEAN_RADIUS_KM,
            mass_kg: 1.8982e27,
            axial_tilt: JUPITER_AXIAL_TILT_RAD,
            rotation_period_secs: JUPITER_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::jupiter_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(778_570_000.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_io() -> Self {
        Self {
            id: BodyId::Io,
            name: "Io",
            radius_km: IO_RADIUS_KM,
            mass_kg: 8.9319e22,
            axial_tilt: 0.0,
            rotation_period_secs: IO_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::io_around_jupiter()),
            parent: Some(BodyId::Jupiter),
            position: Vec3::new(778_570_000.0 + 421_700.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_europa() -> Self {
        Self {
            id: BodyId::Europa,
            name: "Europa",
            radius_km: EUROPA_RADIUS_KM,
            mass_kg: 4.7998e22,
            axial_tilt: 0.1 * PI / 180.0,
            rotation_period_secs: EUROPA_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::europa_around_jupiter()),
            parent: Some(BodyId::Jupiter),
            position: Vec3::new(778_570_000.0 + 670_900.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_ganymede() -> Self {
        Self {
            id: BodyId::Ganymede,
            name: "Ganymede",
            radius_km: GANYMEDE_RADIUS_KM,
            mass_kg: 1.4819e23,
            axial_tilt: 0.2 * PI / 180.0,
            rotation_period_secs: GANYMEDE_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::ganymede_around_jupiter()),
            parent: Some(BodyId::Jupiter),
            position: Vec3::new(778_570_000.0 + 1_070_400.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_callisto() -> Self {
        Self {
            id: BodyId::Callisto,
            name: "Callisto",
            radius_km: CALLISTO_RADIUS_KM,
            mass_kg: 1.0759e23,
            axial_tilt: 0.0,
            rotation_period_secs: CALLISTO_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::callisto_around_jupiter()),
            parent: Some(BodyId::Jupiter),
            position: Vec3::new(778_570_000.0 + 1_882_700.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_saturn() -> Self {
        Self {
            id: BodyId::Saturn,
            name: "Saturn",
            radius_km: SATURN_MEAN_RADIUS_KM,
            mass_kg: 5.6834e26,
            axial_tilt: SATURN_AXIAL_TILT_RAD,
            rotation_period_secs: SATURN_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::saturn_around_sun()),
            parent: Some(BodyId::Sun),
            position: Vec3::new(1_433_530_000.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_mimas() -> Self {
        Self {
            id: BodyId::Mimas,
            name: "Mimas",
            radius_km: MIMAS_RADIUS_KM,
            mass_kg: 3.7493e19,
            axial_tilt: 0.0,
            rotation_period_secs: MIMAS_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::mimas_around_saturn()),
            parent: Some(BodyId::Saturn),
            position: Vec3::new(1_433_530_000.0 + 185_520.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_enceladus() -> Self {
        Self {
            id: BodyId::Enceladus,
            name: "Enceladus",
            radius_km: ENCELADUS_RADIUS_KM,
            mass_kg: 1.0802e20,
            axial_tilt: 0.0,
            rotation_period_secs: ENCELADUS_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::enceladus_around_saturn()),
            parent: Some(BodyId::Saturn),
            position: Vec3::new(1_433_530_000.0 + 238_020.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_rhea() -> Self {
        Self {
            id: BodyId::Rhea,
            name: "Rhea",
            radius_km: RHEA_RADIUS_KM,
            mass_kg: 2.3065e21,
            axial_tilt: 0.0,
            rotation_period_secs: RHEA_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::rhea_around_saturn()),
            parent: Some(BodyId::Saturn),
            position: Vec3::new(1_433_530_000.0 + 527_040.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_titan() -> Self {
        Self {
            id: BodyId::Titan,
            name: "Titan",
            radius_km: TITAN_RADIUS_KM,
            mass_kg: 1.3452e23,
            axial_tilt: 0.3 * PI / 180.0,
            rotation_period_secs: TITAN_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::titan_around_saturn()),
            parent: Some(BodyId::Saturn),
            position: Vec3::new(1_433_530_000.0 + 1_221_830.0, 0.0, 0.0),
            rotation_angle: 0.0,
        }
    }

    pub fn new_iapetus() -> Self {
        Self {
            id: BodyId::Iapetus,
            name: "Iapetus",
            radius_km: IAPETUS_RADIUS_KM,
            mass_kg: 1.8056e21,
            axial_tilt: 0.0,
            rotation_period_secs: IAPETUS_SIDEREAL_DAY_SECS,
            orbit: Some(KeplerOrbit::iapetus_around_saturn()),
            parent: Some(BodyId::Saturn),
            position: Vec3::new(1_433_530_000.0 + 3_561_300.0, 0.0, 0.0),
            rotation_angle: 0.0,
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
