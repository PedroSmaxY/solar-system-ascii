pub mod asteroid_belt;
pub mod body;
pub mod ephemeris;
pub mod jovian_swarm;
pub mod neptunian_swarm;
pub mod saturnian_swarm;
pub mod uranian_swarm;

use self::asteroid_belt::AsteroidField;
use self::body::{BodyId, CelestialBody};
use self::jovian_swarm::JovianMoonSwarm;
use self::neptunian_swarm::NeptunianMoonSwarm;
use self::saturnian_swarm::SaturnianMoonSwarm;
use self::uranian_swarm::UranianMoonSwarm;
use crate::math::{Mat3, Vec3};
use ephemeris::{NEPTUNE_AXIAL_TILT_RAD, PLUTO_AXIAL_TILT_RAD, TWO_PI, URANUS_AXIAL_TILT_RAD};

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
    pub saturn: CelestialBody,
    pub titan: CelestialBody,
    pub enceladus: CelestialBody,
    pub mimas: CelestialBody,
    pub iapetus: CelestialBody,
    pub rhea: CelestialBody,
    pub saturnian_swarm: SaturnianMoonSwarm,
    pub uranus: CelestialBody,
    pub miranda: CelestialBody,
    pub ariel: CelestialBody,
    pub umbriel: CelestialBody,
    pub titania: CelestialBody,
    pub oberon: CelestialBody,
    pub uranian_swarm: UranianMoonSwarm,
    pub neptune: CelestialBody,
    pub triton: CelestialBody,
    pub proteus: CelestialBody,
    pub neptunian_swarm: NeptunianMoonSwarm,
    pub pluto: CelestialBody,
    pub charon: CelestialBody,
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
            saturn: CelestialBody::new_saturn(),
            titan: CelestialBody::new_titan(),
            enceladus: CelestialBody::new_enceladus(),
            mimas: CelestialBody::new_mimas(),
            iapetus: CelestialBody::new_iapetus(),
            rhea: CelestialBody::new_rhea(),
            saturnian_swarm: SaturnianMoonSwarm::new(),
            uranus: CelestialBody::new_uranus(),
            miranda: CelestialBody::new_miranda(),
            ariel: CelestialBody::new_ariel(),
            umbriel: CelestialBody::new_umbriel(),
            titania: CelestialBody::new_titania(),
            oberon: CelestialBody::new_oberon(),
            uranian_swarm: UranianMoonSwarm::new(),
            neptune: CelestialBody::new_neptune(),
            triton: CelestialBody::new_triton(),
            proteus: CelestialBody::new_proteus(),
            neptunian_swarm: NeptunianMoonSwarm::new(),
            pluto: CelestialBody::new_pluto(),
            charon: CelestialBody::new_charon(),
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

        // 16. Saturn orbits Sun (gas giant, e=0.0565, 10.55h fast rotation, axial tilt 26.73°)
        if let Some(ref orb) = self.saturn.orbit {
            self.saturn.position = orb.calculate_position(t);
        }
        self.saturn.rotation_angle = (t * (TWO_PI / self.saturn.rotation_period_secs)).rem_euclid(TWO_PI);

        // 17. Mimas orbits Saturn (inner moon, 22.76h period, tidally locked)
        if let Some(ref orb) = self.mimas.orbit {
            let rel_pos = orb.calculate_position(t);
            self.mimas.position = self.saturn.position + rel_pos;
            self.mimas.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 18. Enceladus orbits Saturn (32.88h period, tidally locked)
        if let Some(ref orb) = self.enceladus.orbit {
            let rel_pos = orb.calculate_position(t);
            self.enceladus.position = self.saturn.position + rel_pos;
            self.enceladus.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 19. Rhea orbits Saturn (108.43h period, tidally locked)
        if let Some(ref orb) = self.rhea.orbit {
            let rel_pos = orb.calculate_position(t);
            self.rhea.position = self.saturn.position + rel_pos;
            self.rhea.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 20. Titan orbits Saturn (giant atmosphere moon, 15.95d period, tidally locked)
        if let Some(ref orb) = self.titan.orbit {
            let rel_pos = orb.calculate_position(t);
            self.titan.position = self.saturn.position + rel_pos;
            self.titan.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 21. Iapetus orbits Saturn (outer inclined moon, 79.33d period, tidally locked)
        if let Some(ref orb) = self.iapetus.orbit {
            let rel_pos = orb.calculate_position(t);
            self.iapetus.position = self.saturn.position + rel_pos;
            self.iapetus.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 22. Uranus orbits Sun (ice giant, 97.77° axial tilt, -17.24h retrograde spin)
        if let Some(ref orb) = self.uranus.orbit {
            self.uranus.position = orb.calculate_position(t);
        }
        self.uranus.rotation_angle = (t * (TWO_PI / self.uranus.rotation_period_secs)).rem_euclid(TWO_PI);

        // Uranus major moons orbit in Uranus's 97.77° tilted equatorial plane
        let u_tilt = Mat3::rotation_x(URANUS_AXIAL_TILT_RAD);

        // 23. Miranda orbits Uranus (Verona Rupes, 1.41d period, tidally locked)
        if let Some(ref orb) = self.miranda.orbit {
            let rel_pos = u_tilt.mul_vec(orb.calculate_position(t));
            self.miranda.position = self.uranus.position + rel_pos;
            self.miranda.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 24. Ariel orbits Uranus (rifts & grabens, 2.52d period, tidally locked)
        if let Some(ref orb) = self.ariel.orbit {
            let rel_pos = u_tilt.mul_vec(orb.calculate_position(t));
            self.ariel.position = self.uranus.position + rel_pos;
            self.ariel.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 25. Umbriel orbits Uranus (dark cratered crust, 4.14d period, tidally locked)
        if let Some(ref orb) = self.umbriel.orbit {
            let rel_pos = u_tilt.mul_vec(orb.calculate_position(t));
            self.umbriel.position = self.uranus.position + rel_pos;
            self.umbriel.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 26. Titania orbits Uranus (largest moon, Messina Chasma, 8.71d period, tidally locked)
        if let Some(ref orb) = self.titania.orbit {
            let rel_pos = u_tilt.mul_vec(orb.calculate_position(t));
            self.titania.position = self.uranus.position + rel_pos;
            self.titania.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 27. Oberon orbits Uranus (outermost major moon, 13.46d period, tidally locked)
        if let Some(ref orb) = self.oberon.orbit {
            let rel_pos = u_tilt.mul_vec(orb.calculate_position(t));
            self.oberon.position = self.uranus.position + rel_pos;
            self.oberon.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 28. Neptune orbits Sun (ice giant, 28.32° axial tilt, 16.11h spin)
        if let Some(ref orb) = self.neptune.orbit {
            self.neptune.position = orb.calculate_position(t);
        }
        self.neptune.rotation_angle = (t * (TWO_PI / self.neptune.rotation_period_secs)).rem_euclid(TWO_PI);

        // 29. Triton orbits Neptune (retrograde captured KBO, 156.885° inc, -5.88d period)
        if let Some(ref orb) = self.triton.orbit {
            let rel_pos = orb.calculate_position(t);
            self.triton.position = self.neptune.position + rel_pos;
            self.triton.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 30. Proteus orbits Neptune in equatorial plane (28.32° tilt, 1.12d period)
        let n_tilt = Mat3::rotation_x(NEPTUNE_AXIAL_TILT_RAD);
        if let Some(ref orb) = self.proteus.orbit {
            let rel_pos = n_tilt.mul_vec(orb.calculate_position(t));
            self.proteus.position = self.neptune.position + rel_pos;
            self.proteus.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
        }

        // 31. Pluto orbits Sun (highly eccentric e=0.2488, i=17.16°, retrograde spin)
        if let Some(ref orb) = self.pluto.orbit {
            self.pluto.position = orb.calculate_position(t);
        }
        self.pluto.rotation_angle = (t * (TWO_PI / self.pluto.rotation_period_secs)).rem_euclid(TWO_PI);

        // 32. Charon orbits Pluto (tidally locked binary system, tilted orbital plane)
        let pluto_tilt = Mat3::rotation_x(PLUTO_AXIAL_TILT_RAD);
        if let Some(ref orb) = self.charon.orbit {
            let rel_pos = pluto_tilt.mul_vec(orb.calculate_position(t));
            self.charon.position = self.pluto.position + rel_pos;
            self.charon.rotation_angle = rel_pos.z.atan2(-rel_pos.x);
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
            BodyId::Saturn => &self.saturn,
            BodyId::Titan => &self.titan,
            BodyId::Enceladus => &self.enceladus,
            BodyId::Mimas => &self.mimas,
            BodyId::Iapetus => &self.iapetus,
            BodyId::Rhea => &self.rhea,
            BodyId::Uranus => &self.uranus,
            BodyId::Miranda => &self.miranda,
            BodyId::Ariel => &self.ariel,
            BodyId::Umbriel => &self.umbriel,
            BodyId::Titania => &self.titania,
            BodyId::Oberon => &self.oberon,
            BodyId::Neptune => &self.neptune,
            BodyId::Triton => &self.triton,
            BodyId::Proteus => &self.proteus,
            BodyId::Pluto => &self.pluto,
            BodyId::Charon => &self.charon,
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
            BodyId::Saturn => &mut self.saturn,
            BodyId::Titan => &mut self.titan,
            BodyId::Enceladus => &mut self.enceladus,
            BodyId::Mimas => &mut self.mimas,
            BodyId::Iapetus => &mut self.iapetus,
            BodyId::Rhea => &mut self.rhea,
            BodyId::Uranus => &mut self.uranus,
            BodyId::Miranda => &mut self.miranda,
            BodyId::Ariel => &mut self.ariel,
            BodyId::Umbriel => &mut self.umbriel,
            BodyId::Titania => &mut self.titania,
            BodyId::Oberon => &mut self.oberon,
            BodyId::Neptune => &mut self.neptune,
            BodyId::Triton => &mut self.triton,
            BodyId::Proteus => &mut self.proteus,
            BodyId::Pluto => &mut self.pluto,
            BodyId::Charon => &mut self.charon,
        }
    }
}
