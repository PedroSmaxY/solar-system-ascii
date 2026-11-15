pub mod asteroid_belt;
pub mod body;
pub mod ephemeris;
pub mod jovian_swarm;

use self::asteroid_belt::AsteroidField;
use self::body::{BodyId, CelestialBody};
use self::jovian_swarm::JovianMoonSwarm;
use crate::math::Vec3;
use ephemeris::TWO_PI;

pub struct SolarSystemPhysics {
    pub sun: CelestialBody,
    pub mercury: CelestialBody,
    pub venus: CelestialBody,
    pub earth: CelestialBody,
    pub moon: CelestialBody,
    pub mars: CelestialBody,
    pub phobos: CelestialBody,
    pub deimos: CelestialBody,
    pub ceres: CelestialBody,
    pub vesta: CelestialBody,
    pub asteroid_field: AsteroidField,
    pub jupiter: CelestialBody,
    pub io: CelestialBody,
    pub europa: CelestialBody,
    pub ganymede: CelestialBody,
    pub callisto: CelestialBody,
    pub jovian_swarm: JovianMoonSwarm,
}

impl SolarSystemPhysics {
    pub fn new() -> Self {
        Self {
            sun: CelestialBody::new_sun(),
            mercury: CelestialBody::new_mercury(),
            venus: CelestialBody::new_venus(),
            earth: CelestialBody::new_earth(),
            moon: CelestialBody::new_moon(),
            mars: CelestialBody::new_mars(),
            phobos: CelestialBody::new_phobos(),
            deimos: CelestialBody::new_deimos(),
            ceres: CelestialBody::new_ceres(),
            vesta: CelestialBody::new_vesta(),
            asteroid_field: AsteroidField::new(),
            jupiter: CelestialBody::new_jupiter(),
            io: CelestialBody::new_io(),
            europa: CelestialBody::new_europa(),
            ganymede: CelestialBody::new_ganymede(),
            callisto: CelestialBody::new_callisto(),
            jovian_swarm: JovianMoonSwarm::new(),
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
            // Synchronous tidal locking: Prime meridian permanently faces Earth
            self.moon.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 6. Mars orbits Sun (eccentric orbit e=0.0934, Martian Sol 24.62h)
        if let Some(ref orb) = self.mars.orbit {
            self.mars.position = orb.calculate_position(t);
        }
        self.mars.rotation_angle = (t * (TWO_PI / self.mars.rotation_period_secs)).rem_euclid(TWO_PI);

        // 7. Phobos orbits Mars (ultra-fast 7.65h inner moon, tidal locking to Mars)
        if let Some(ref orb) = self.phobos.orbit {
            let rel_pos = orb.calculate_position(t);
            self.phobos.position = self.mars.position + rel_pos;
            // Synchronous tidal locking: Longest axis permanently faces Mars
            self.phobos.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 8. Deimos orbits Mars (30.3h outer moon, tidal locking to Mars)
        if let Some(ref orb) = self.deimos.orbit {
            let rel_pos = orb.calculate_position(t);
            self.deimos.position = self.mars.position + rel_pos;
            // Synchronous tidal locking: Longest axis permanently faces Mars
            self.deimos.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 9. Ceres orbits Sun (dwarf planet, e=0.0758, 9.07h rotation)
        if let Some(ref orb) = self.ceres.orbit {
            self.ceres.position = orb.calculate_position(t);
        }
        self.ceres.rotation_angle = (t * (TWO_PI / self.ceres.rotation_period_secs)).rem_euclid(TWO_PI);

        // 10. Vesta orbits Sun (protoplanet, e=0.0887, 5.34h rotation)
        if let Some(ref orb) = self.vesta.orbit {
            self.vesta.position = orb.calculate_position(t);
        }
        self.vesta.rotation_angle = (t * (TWO_PI / self.vesta.rotation_period_secs)).rem_euclid(TWO_PI);

        // 11. Jupiter orbits Sun (gas giant, e=0.0489, 9.93h fast rotation)
        if let Some(ref orb) = self.jupiter.orbit {
            self.jupiter.position = orb.calculate_position(t);
        }
        self.jupiter.rotation_angle = (t * (TWO_PI / self.jupiter.rotation_period_secs)).rem_euclid(TWO_PI);

        // 12. Io orbits Jupiter (Laplace 4:2:1 resonance, 42.46h period, tidally locked)
        if let Some(ref orb) = self.io.orbit {
            let rel_pos = orb.calculate_position(t);
            self.io.position = self.jupiter.position + rel_pos;
            self.io.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 13. Europa orbits Jupiter (Laplace 2:1 resonance with Io, 84.92h period, tidally locked)
        if let Some(ref orb) = self.europa.orbit {
            let rel_pos = orb.calculate_position(t);
            self.europa.position = self.jupiter.position + rel_pos;
            self.europa.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 14. Ganymede orbits Jupiter (Laplace 4:1 resonance with Io, 169.84h period, tidally locked)
        if let Some(ref orb) = self.ganymede.orbit {
            let rel_pos = orb.calculate_position(t);
            self.ganymede.position = self.jupiter.position + rel_pos;
            self.ganymede.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 15. Callisto orbits Jupiter (outermost Galilean moon, 400.54h period, tidally locked)
        if let Some(ref orb) = self.callisto.orbit {
            let rel_pos = orb.calculate_position(t);
            self.callisto.position = self.jupiter.position + rel_pos;
            self.callisto.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }
    }

    pub fn get_body(&self, id: BodyId) -> &CelestialBody {
        match id {
            BodyId::Sun => &self.sun,
            BodyId::Mercury => &self.mercury,
            BodyId::Venus => &self.venus,
            BodyId::Earth => &self.earth,
            BodyId::Moon => &self.moon,
            BodyId::Mars => &self.mars,
            BodyId::Phobos => &self.phobos,
            BodyId::Deimos => &self.deimos,
            BodyId::Ceres => &self.ceres,
            BodyId::Vesta => &self.vesta,
            BodyId::Jupiter => &self.jupiter,
            BodyId::Io => &self.io,
            BodyId::Europa => &self.europa,
            BodyId::Ganymede => &self.ganymede,
            BodyId::Callisto => &self.callisto,
        }
    }

    pub fn get_body_mut(&mut self, id: BodyId) -> &mut CelestialBody {
        match id {
            BodyId::Sun => &mut self.sun,
            BodyId::Mercury => &mut self.mercury,
            BodyId::Venus => &mut self.venus,
            BodyId::Earth => &mut self.earth,
            BodyId::Moon => &mut self.moon,
            BodyId::Mars => &mut self.mars,
            BodyId::Phobos => &mut self.phobos,
            BodyId::Deimos => &mut self.deimos,
            BodyId::Ceres => &mut self.ceres,
            BodyId::Vesta => &mut self.vesta,
            BodyId::Jupiter => &mut self.jupiter,
            BodyId::Io => &mut self.io,
            BodyId::Europa => &mut self.europa,
            BodyId::Ganymede => &mut self.ganymede,
            BodyId::Callisto => &mut self.callisto,
        }
    }
}
