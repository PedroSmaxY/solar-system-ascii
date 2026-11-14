pub mod body;
pub mod ephemeris;

use self::body::{BodyId, CelestialBody};
use crate::math::Vec3;
use ephemeris::TWO_PI;

pub struct SolarSystemPhysics {
    pub sun: CelestialBody,
    pub mercury: CelestialBody,
    pub venus: CelestialBody,
    pub earth: CelestialBody,
    pub moon: CelestialBody,
}

impl SolarSystemPhysics {
    pub fn new() -> Self {
        Self {
            sun: CelestialBody::new_sun(),
            mercury: CelestialBody::new_mercury(),
            venus: CelestialBody::new_venus(),
            earth: CelestialBody::new_earth(),
            moon: CelestialBody::new_moon(),
        }
    }

    /// Update physics state for time t (seconds).
    pub fn update(&mut self, t: f64) {
        // 1. Sun remains at origin in heliocentric reference
        self.sun.position = Vec3::ZERO;
        self.sun.rotation_angle = (t * (TWO_PI / self.sun.rotation_period_secs)).rem_euclid(TWO_PI);

        // 2. Mercury orbits Sun (extreme eccentricity e=0.2056, 3:2 spin-orbit resonance)
        if let Some(ref orb) = self.mercury.orbit {
            self.mercury.position = orb.calculate_position(t);
        }
        self.mercury.rotation_angle = (t * (TWO_PI / self.mercury.rotation_period_secs)).rem_euclid(TWO_PI);

        // 3. Venus orbits Sun (circular orbit e=0.00677, retrograde rotation 243d)
        if let Some(ref orb) = self.venus.orbit {
            self.venus.position = orb.calculate_position(t);
        }
        self.venus.rotation_angle = (t * (TWO_PI / self.venus.rotation_period_secs)).rem_euclid(TWO_PI);

        // 4. Earth orbits Sun
        if let Some(ref orb) = self.earth.orbit {
            self.earth.position = orb.calculate_position(t);
        }
        self.earth.rotation_angle = (t * (TWO_PI / self.earth.rotation_period_secs)).rem_euclid(TWO_PI);

        // 5. Moon orbits Earth (hierarchical relative offset)
        if let Some(ref orb) = self.moon.orbit {
            let rel_pos = orb.calculate_position(t);
            self.moon.position = self.earth.position + rel_pos;
            // Synchronous tidal locking: Prime meridian (0° lon, Sinus Medii) permanently faces Earth
            self.moon.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }
    }

    pub fn get_body(&self, id: BodyId) -> &CelestialBody {
        match id {
            BodyId::Sun => &self.sun,
            BodyId::Mercury => &self.mercury,
            BodyId::Venus => &self.venus,
            BodyId::Earth => &self.earth,
            BodyId::Moon => &self.moon,
        }
    }

    pub fn get_body_mut(&mut self, id: BodyId) -> &mut CelestialBody {
        match id {
            BodyId::Sun => &mut self.sun,
            BodyId::Mercury => &mut self.mercury,
            BodyId::Venus => &mut self.venus,
            BodyId::Earth => &mut self.earth,
            BodyId::Moon => &mut self.moon,
        }
    }
}
