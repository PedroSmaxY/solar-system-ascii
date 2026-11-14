pub mod body;
pub mod ephemeris;

use self::body::{BodyId, CelestialBody};
use crate::math::Vec3;
use ephemeris::TWO_PI;

pub struct SolarSystemPhysics {
    pub sun: CelestialBody,
    pub earth: CelestialBody,
    pub moon: CelestialBody,
}

impl SolarSystemPhysics {
    pub fn new() -> Self {
        Self {
            sun: CelestialBody::new_sun(),
            earth: CelestialBody::new_earth(),
            moon: CelestialBody::new_moon(),
        }
    }

    /// Update physics state for time t (seconds).
    pub fn update(&mut self, t: f64) {
        // 1. Sun remains at origin in heliocentric reference
        self.sun.position = Vec3::ZERO;
        self.sun.rotation_angle = (t * (TWO_PI / self.sun.rotation_period_secs)).rem_euclid(TWO_PI);

        // 2. Earth orbits Sun
        if let Some(ref orb) = self.earth.orbit {
            self.earth.position = orb.calculate_position(t);
        }
        self.earth.rotation_angle = (t * (TWO_PI / self.earth.rotation_period_secs)).rem_euclid(TWO_PI);

        // 3. Moon orbits Earth (hierarchical relative offset)
        if let Some(ref orb) = self.moon.orbit {
            let rel_pos = orb.calculate_position(t);
            self.moon.position = self.earth.position + rel_pos;
            // Tidal locking: Moon rotation matches its orbital period
            let mean_motion = TWO_PI / orb.period;
            self.moon.rotation_angle = (orb.m0 + mean_motion * t).rem_euclid(TWO_PI);
        }
    }

    pub fn get_body(&self, id: BodyId) -> &CelestialBody {
        match id {
            BodyId::Sun => &self.sun,
            BodyId::Earth => &self.earth,
            BodyId::Moon => &self.moon,
        }
    }

    pub fn get_body_mut(&mut self, id: BodyId) -> &mut CelestialBody {
        match id {
            BodyId::Sun => &mut self.sun,
            BodyId::Earth => &mut self.earth,
            BodyId::Moon => &mut self.moon,
        }
    }
}
