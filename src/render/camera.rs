//! 6-DOF Astronomical Camera with continuous logarithmic/exponential zoom,
//! independent per-body perspective memory, and smooth interplanetary fly-to transitions.

use crate::math::{Ray, Vec3};
use crate::physics::body::BodyId;
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::RenderMode;
use std::f64::consts::PI;

/// Saved view state per celestial body (azimuth, elevation, distance).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyViewState {
    pub azimuth: f64,   // Radians
    pub elevation: f64, // Radians (-PI/2 .. PI/2)
    pub distance_km: f64,
}

/// Active interplanetary fly-to camera transition.
#[derive(Clone, Copy, Debug)]
pub struct CameraTransition {
    pub from_body: BodyId,
    pub to_body: BodyId,
    pub start_view: BodyViewState,
    pub end_view: BodyViewState,
    pub elapsed_secs: f64,
    pub duration_secs: f64,
    pub journey_dist_km: f64,
}

pub struct Camera {
    pub target_body: BodyId,
    pub target_pos: Vec3,
    pub azimuth: f64,   // Radians
    pub elevation: f64, // Radians (-PI/2 .. PI/2)
    pub distance_km: f64,
    pub fov_y_rad: f64,
    pub saved_views: [BodyViewState; 32], // [Sun, Mercury, Venus, Earth, Moon, Mars, Phobos, Deimos, Ceres, Vesta, Jupiter, Io, Europa, Ganymede, Callisto, Saturn, Titan, Enceladus, Mimas, Iapetus, Rhea, Uranus, Miranda, Ariel, Umbriel, Titania, Oberon, Neptune, Triton, Proteus, Pluto, Charon]
    pub transition: Option<CameraTransition>,
    pub show_orbit_lines: bool,
    pub last_focused_planet: BodyId,

    // Free flight mode
    pub free_flight: bool,
    pub free_pos: Vec3,
    pub free_yaw: f64,
    pub free_pitch: f64,
}

impl Camera {
    pub fn new() -> Self {
        let sun_view = Self::default_view(BodyId::Sun);
        let mercury_view = Self::default_view(BodyId::Mercury);
        let venus_view = Self::default_view(BodyId::Venus);
        let earth_view = Self::default_view(BodyId::Earth);
        let moon_view = Self::default_view(BodyId::Moon);
        let mars_view = Self::default_view(BodyId::Mars);
        let phobos_view = Self::default_view(BodyId::Phobos);
        let deimos_view = Self::default_view(BodyId::Deimos);
        let ceres_view = Self::default_view(BodyId::Ceres);
        let vesta_view = Self::default_view(BodyId::Vesta);
        let jupiter_view = Self::default_view(BodyId::Jupiter);
        let io_view = Self::default_view(BodyId::Io);
        let europa_view = Self::default_view(BodyId::Europa);
        let ganymede_view = Self::default_view(BodyId::Ganymede);
        let callisto_view = Self::default_view(BodyId::Callisto);
        let saturn_view = Self::default_view(BodyId::Saturn);
        let titan_view = Self::default_view(BodyId::Titan);
        let enceladus_view = Self::default_view(BodyId::Enceladus);
        let mimas_view = Self::default_view(BodyId::Mimas);
        let iapetus_view = Self::default_view(BodyId::Iapetus);
        let rhea_view = Self::default_view(BodyId::Rhea);
        let uranus_view = Self::default_view(BodyId::Uranus);
        let miranda_view = Self::default_view(BodyId::Miranda);
        let ariel_view = Self::default_view(BodyId::Ariel);
        let umbriel_view = Self::default_view(BodyId::Umbriel);
        let titania_view = Self::default_view(BodyId::Titania);
        let oberon_view = Self::default_view(BodyId::Oberon);
        let neptune_view = Self::default_view(BodyId::Neptune);
        let triton_view = Self::default_view(BodyId::Triton);
        let proteus_view = Self::default_view(BodyId::Proteus);
        let pluto_view = Self::default_view(BodyId::Pluto);
        let charon_view = Self::default_view(BodyId::Charon);

        Self {
            target_body: BodyId::Sun,
            target_pos: Vec3::ZERO,
            azimuth: sun_view.azimuth,
            elevation: sun_view.elevation,
            distance_km: sun_view.distance_km,
            fov_y_rad: 45.0 * PI / 180.0,
            saved_views: [
                sun_view,
                mercury_view,
                venus_view,
                earth_view,
                moon_view,
                mars_view,
                phobos_view,
                deimos_view,
                ceres_view,
                vesta_view,
                jupiter_view,
                io_view,
                europa_view,
                ganymede_view,
                callisto_view,
                saturn_view,
                titan_view,
                enceladus_view,
                mimas_view,
                iapetus_view,
                rhea_view,
                uranus_view,
                miranda_view,
                ariel_view,
                umbriel_view,
                titania_view,
                oberon_view,
                neptune_view,
                triton_view,
                proteus_view,
                pluto_view,
                charon_view,
            ],
            transition: None,
            show_orbit_lines: true,
            last_focused_planet: BodyId::Earth,
            free_flight: false,
            free_pos: Vec3::ZERO,
            free_yaw: 0.0,
            free_pitch: 0.0,
        }
    }

    /// Default perspective and viewing distance for each celestial body.
    pub fn default_view(body: BodyId) -> BodyViewState {
        match body {
            BodyId::Sun => BodyViewState {
                azimuth: 0.0,
                elevation: 0.0,
                distance_km: 1.1e10, // ~73.5 AU - authentic top-down solar system orrery framing all orbits
            },
            BodyId::Mercury => BodyViewState {
                azimuth: 3.90,        // Framing sunlit Caloris Basin, cratered terrain, and dramatic terminator
                elevation: 0.18,
                distance_km: 7_500.0,  // Dedicated perspective framing Mercury's 2,440 km disk
            },
            BodyId::Venus => BodyViewState {
                azimuth: 2.40,        // Framing sunlit cloud deck, chevron Y-waves and extended twilight crescent
                elevation: 0.20,
                distance_km: 18_500.0, // Balanced orbital perspective of Venus
            },
            BodyId::Earth => BodyViewState {
                azimuth: 2.30,        // Framing both daylit Americas/Atlantic and night city lights
                elevation: 0.28,
                distance_km: 21_000.0, // Balanced orbital perspective of Earth
            },
            BodyId::Moon => BodyViewState {
                azimuth: 0.85,        // Framing illuminated Near Side maria, Tycho rays & terminator
                elevation: 0.12,
                distance_km: 8_000.0,  // Dedicated lunar view framing entire disk
            },
            BodyId::Mars => BodyViewState {
                azimuth: -1.18,       // Framing sunlit Valles Marineris, Olympus Mons & south polar cap
                elevation: 0.18,
                distance_km: 10_500.0, // Balanced orbital perspective framing Mars (~65% of screen)
            },
            BodyId::Phobos => BodyViewState {
                azimuth: 2.10,        // Framing Stickney crater excavation, raised rim & grooved terrain under oblique sunlight
                elevation: 0.18,
                distance_km: 44.0,     // Close perspective framing irregular 26 km moon filling 70% of screen
            },
            BodyId::Deimos => BodyViewState {
                azimuth: 2.10,        // Framing smooth regolith blanket and crater faculae
                elevation: 0.15,
                distance_km: 26.0,     // Close perspective framing 15 km moon filling 70% of screen
            },
            BodyId::Ceres => BodyViewState {
                azimuth: -2.61,       // Framing sunlit Occator Crater & bright salt faculae
                elevation: 0.28,
                distance_km: 1_400.0, // Dedicated perspective framing Ceres' 482 km disk (~65% of screen)
            },
            BodyId::Vesta => BodyViewState {
                azimuth: -1.25,       // Framing Rheasilvia basin, central peak & Divalia Fossae
                elevation: -0.55,
                distance_km: 850.0,   // Dedicated perspective framing Vesta's 286 km triaxial disk
            },
            BodyId::Jupiter => BodyViewState {
                azimuth: -1.35,       // Framing sunlit disk, Great Red Spot at -22°S, equatorial belts & limb darkening
                elevation: 0.12,
                distance_km: 210_000.0, // Framing ~140,000 km oblate gas giant disk
            },
            BodyId::Io => BodyViewState {
                azimuth: -1.40,       // Framing sunlit Pele & Loki volcanic calderas and sulfur plains
                elevation: 0.14,
                distance_km: 5_500.0,  // Framing 3,640 km disk
            },
            BodyId::Europa => BodyViewState {
                azimuth: -1.38,       // Framing sunlit ice crust, Conamara Chaos & red-brown lineae fractures
                elevation: 0.14,
                distance_km: 4_800.0,  // Framing 3,120 km disk
            },
            BodyId::Ganymede => BodyViewState {
                azimuth: -1.36,       // Framing sunlit dark polygonal Galileo Regio and bright grooved terrain
                elevation: 0.14,
                distance_km: 7_800.0,  // Framing 5,268 km disk (largest moon)
            },
            BodyId::Callisto => BodyViewState {
                azimuth: -1.34,       // Framing sunlit cratered crust and Valhalla multi-ring basin
                elevation: 0.14,
                distance_km: 7_200.0,  // Framing 4,820 km disk
            },
            BodyId::Saturn => BodyViewState {
                azimuth: -1.35,       // Framing sunlit golden disk, rings, shadow on disk, and Cassini division
                elevation: 0.35,      // Slightly elevated to view ring plane structure
                distance_km: 260_000.0, // Framing ~120,000 km disk and 280,000 km ring system
            },
            BodyId::Titan => BodyViewState {
                azimuth: -1.35,       // Framing hazy orange nitrogen/methane atmosphere and limb
                elevation: 0.12,
                distance_km: 6_200.0,  // Framing 5,150 km disk
            },
            BodyId::Enceladus => BodyViewState {
                azimuth: -1.35,       // Framing ultra-reflective ice crust and south polar tiger stripes
                elevation: -0.30,     // Looking slightly up at south polar terrain
                distance_km: 750.0,   // Framing 504 km disk
            },
            BodyId::Mimas => BodyViewState {
                azimuth: -1.35,       // Framing colossal Herschel crater basin ("Death Star")
                elevation: 0.05,
                distance_km: 600.0,   // Framing 396 km disk
            },
            BodyId::Iapetus => BodyViewState {
                azimuth: -1.35,       // Framing dramatic Yin-Yang dichotomy and equatorial ridge
                elevation: 0.26,      // 15 deg inclined orbit perspective
                distance_km: 2_200.0, // Framing 1,471 km disk
            },
            BodyId::Rhea => BodyViewState {
                azimuth: -1.35,       // Framing heavily cratered ice/rock crust and wispy streaks
                elevation: 0.15,
                distance_km: 2_300.0, // Framing 1,528 km disk
            },
            BodyId::Uranus => BodyViewState {
                azimuth: 1.85,        // Framing 97.77° tilted sunlit cyan disk and vertical ring system
                elevation: 0.25,
                distance_km: 140_000.0,// Framing 51,118 km disk
            },
            BodyId::Miranda => BodyViewState {
                azimuth: 1.85,        // Framing sunlit Verona Rupes 20 km fault scarp & coronae
                elevation: 0.12,
                distance_km: 1_000.0, // Framing 472 km disk
            },
            BodyId::Ariel => BodyViewState {
                azimuth: 1.85,        // Framing sunlit brilliant bright rift valleys and grabens
                elevation: 0.12,
                distance_km: 1_900.0, // Framing 1,158 km disk
            },
            BodyId::Umbriel => BodyViewState {
                azimuth: 1.85,        // Framing sunlit dark coal-gray crust and Wunda bright ring crater
                elevation: 0.12,
                distance_km: 1_900.0, // Framing 1,170 km disk
            },
            BodyId::Titania => BodyViewState {
                azimuth: 1.85,        // Framing sunlit largest moon and Messina Chasma canyon network
                elevation: 0.12,
                distance_km: 2_400.0, // Framing 1,577 km disk
            },
            BodyId::Oberon => BodyViewState {
                azimuth: 1.85,        // Framing sunlit carbon-rich crater floors and limb mountain peak
                elevation: 0.12,
                distance_km: 2_400.0, // Framing 1,523 km disk
            },
            BodyId::Neptune => BodyViewState {
                azimuth: -1.15,       // Framing sunlit azure cobalt gas giant, Great Dark Spot and Scooter
                elevation: 0.18,      // Clean equatorial tilt
                distance_km: 75_000.0, // Framing 49,528 km disk (~65% of screen height)
            },
            BodyId::Triton => BodyViewState {
                azimuth: -1.35,       // Framing retrograde moon, cantaloupe terrain and cryogeysers
                elevation: 0.15,
                distance_km: 4_200.0, // Framing 2,707 km disk
            },
            BodyId::Proteus => BodyViewState {
                azimuth: -1.35,       // Framing non-spherical boxy facets and Pharos crater
                elevation: 0.10,
                distance_km: 900.0,   // Framing 420 km irregular disk
            },
            BodyId::Pluto => BodyViewState {
                azimuth: 2.20,        // Framing Tombaugh Regio heart, bright white Sputnik Planitia, and Cthulhu Macula
                elevation: 0.15,
                distance_km: 5_800.0, // Balanced perspective framing 2,376 km dwarf planet disk with Charon in view
            },
            BodyId::Charon => BodyViewState {
                azimuth: 1.80,        // Framing Serenity Chasma canyons and Mordor Macula
                elevation: 0.12,
                distance_km: 2_900.0, // Balanced perspective framing 1,212 km binary companion disk
            },
        }
    }

    /// Start a smooth interplanetary fly-to transition with zoom-out and zoom-in.
    pub fn start_transition(
        &mut self,
        to_body: BodyId,
        from_pos: Vec3,
        to_pos: Vec3,
        duration_secs: f64,
    ) {
        if to_body == self.target_body && self.transition.is_none() {
            return;
        }

        // Save current view state of the body being left
        let curr_idx = self.target_body.index();
        let start_view = BodyViewState {
            azimuth: self.azimuth,
            elevation: self.elevation,
            distance_km: self.distance_km,
        };
        self.saved_views[curr_idx] = start_view;

        let end_view = self.saved_views[to_body.index()];
        let journey_dist_km = (to_pos - from_pos).length().max(10_000.0);

        self.transition = Some(CameraTransition {
            from_body: self.target_body,
            to_body,
            start_view,
            end_view,
            elapsed_secs: 0.0,
            duration_secs: duration_secs.max(0.1),
            journey_dist_km,
        });
    }

    /// Cancel any active transition immediately.
    pub fn cancel_transition(&mut self) {
        if let Some(trans) = self.transition.take() {
            self.target_body = trans.to_body;
            let target_view = self.saved_views[trans.to_body.index()];
            self.azimuth = target_view.azimuth;
            self.elevation = target_view.elevation;
            self.distance_km = target_view.distance_km;
        }
    }

    /// Update camera flight transition animation and track moving target body.
    pub fn update(&mut self, dt: f64, physics: &SolarSystemPhysics) {
        if let Some(mut trans) = self.transition {
            trans.elapsed_secs += dt;
            let progress = (trans.elapsed_secs / trans.duration_secs).clamp(0.0, 1.0);

            // Quintic smoothstep curve C^2 continuous: 6p^5 - 15p^4 + 10p^3
            let s = progress * progress * progress * (progress * (progress * 6.0 - 15.0) + 10.0);

            // Interpolate target focal position between moving bodies
            let from_pos = physics.get_body(trans.from_body).position;
            let to_pos = physics.get_body(trans.to_body).position;
            self.target_pos = from_pos * (1.0 - s) + to_pos * s;

            // Shortest circular angular arc for azimuth
            let d_az = ((trans.end_view.azimuth - trans.start_view.azimuth + PI).rem_euclid(2.0 * PI)) - PI;
            self.azimuth = (trans.start_view.azimuth + d_az * s).rem_euclid(2.0 * PI);

            // Elevation interpolation
            self.elevation = trans.start_view.elevation + (trans.end_view.elevation - trans.start_view.elevation) * s;

            // Logarithmic distance interpolation with parabolic altitude arc (zoom-out then zoom-in)
            let log_start = trans.start_view.distance_km.ln();
            let log_end = trans.end_view.distance_km.ln();
            let base_log = log_start * (1.0 - s) + log_end * s;

            // Apex altitude pulls out to reveal interplanetary scale
            let apex_dist = (trans.journey_dist_km * 0.70)
                .max(trans.start_view.distance_km * 1.5)
                .max(trans.end_view.distance_km * 1.5);
            let apex_log = apex_dist.ln();
            let arc_height = (apex_log - base_log).max(0.0);
            let arc_factor = 4.0 * s * (1.0 - s); // Parabola peaking at 1.0 at s=0.5
            let current_log = base_log + arc_factor * arc_height;
            self.distance_km = current_log.exp();

            if progress >= 1.0 {
                // Completed transition: anchor cleanly at destination
                self.target_body = trans.to_body;
                self.target_pos = to_pos;
                self.azimuth = trans.end_view.azimuth;
                self.elevation = trans.end_view.elevation;
                self.distance_km = trans.end_view.distance_km;
                self.transition = None;
            } else {
                self.transition = Some(trans);
            }
        } else {
            // Keep camera target tracked to the current body
            self.target_pos = physics.get_body(self.target_body).position;
        }
    }

    pub fn zoom_in(&mut self) {
        self.cancel_transition();
        let min_dist = match self.target_body {
            BodyId::Earth => 6_800.0, // ~430 km above surface (ISS orbit)
            BodyId::Mercury => 2_600.0, // ~160 km above surface (MESSENGER orbit)
            BodyId::Venus => 6_300.0, // ~250 km above cloud deck
            BodyId::Moon => 1_900.0,
            BodyId::Sun => 750_000.0,
            BodyId::Mars => 3_550.0,  // ~160 km above surface
            BodyId::Phobos => 13.5,   // ~500 m above Stickney rim
            BodyId::Deimos => 7.8,    // ~300 m above Swift crater
            BodyId::Ceres => 490.0,   // ~8 km above Occator rim / Ahuna Mons
            BodyId::Vesta => 240.0,   // ~17 km above Rheasilvia rim
            BodyId::Jupiter => 72_000.0, // ~500 km above equatorial 1-bar cloud deck
            BodyId::Io => 1_850.0,      // ~30 km above sulfur volcanic plains
            BodyId::Europa => 1_580.0,  // ~20 km above ice fractures
            BodyId::Ganymede => 2_650.0,// ~15 km above grooved ridges
            BodyId::Callisto => 2_430.0,// ~20 km above Valhalla crater floor
            BodyId::Saturn => 61_000.0, // ~700 km above equatorial 1-bar cloud deck
            BodyId::Titan => 2_600.0,   // ~25 km above thick haze deck
            BodyId::Enceladus => 260.0, // ~8 km above tiger stripes
            BodyId::Mimas => 210.0,     // ~12 km above Herschel crater floor
            BodyId::Iapetus => 750.0,   // ~14 km above equatorial ridge
            BodyId::Rhea => 780.0,      // ~16 km above crater rim
            BodyId::Uranus => 26_000.0, // ~440 km above cloud deck
            BodyId::Miranda => 245.0,   // ~9 km above Verona Rupes cliff edge
            BodyId::Ariel => 595.0,     // ~16 km above rift canyon floor
            BodyId::Umbriel => 600.0,   // ~15 km above Wunda rim
            BodyId::Titania => 810.0,   // ~21 km above Messina Chasma floor
            BodyId::Oberon => 780.0,    // ~18 km above crater floor
            BodyId::Neptune => 25_000.0,// ~230 km above cloud deck
            BodyId::Triton => 1_380.0,  // ~26 km above cantaloupe terrain
            BodyId::Proteus => 220.0,   // ~10 km above Pharos crater rim
            BodyId::Pluto => 1_250.0,   // ~60 km above Tombaugh Regio / Sputnik Planitia
            BodyId::Charon => 650.0,    // ~44 km above Serenity Chasma canyon rim
        };
        self.distance_km = (self.distance_km * 0.88).max(min_dist);
        self.saved_views[self.target_body.index()].distance_km = self.distance_km;
    }

    /// Zoom out with smooth automated overview transition when threshold is exceeded.
    /// Returns true if an automated transition to the top-down Solar System view should be started.
    pub fn zoom_out(&mut self) -> bool {
        self.cancel_transition();
        self.distance_km = (self.distance_km * 1.14).min(3.5e10);
        self.saved_views[self.target_body.index()].distance_km = self.distance_km;

        let overview_threshold = match self.target_body {
            BodyId::Sun => 3.5e10,
            BodyId::Jupiter | BodyId::Saturn | BodyId::Uranus | BodyId::Neptune => 4.5e8,
            _ => 3.5e7,
        };

        if self.target_body != BodyId::Sun && self.distance_km > overview_threshold {
            true
        } else {
            false
        }
    }

    pub fn orbit(&mut self, d_az: f64, d_el: f64) {
        self.cancel_transition();
        self.azimuth = (self.azimuth + d_az).rem_euclid(2.0 * PI);
        self.elevation = (self.elevation + d_el).rem_euclid(2.0 * PI);
        let idx = self.target_body.index();
        self.saved_views[idx].azimuth = self.azimuth;
        self.saved_views[idx].elevation = self.elevation;
    }

    pub fn recenter(&mut self) {
        self.cancel_transition();
        let def = Self::default_view(self.target_body);
        self.azimuth = def.azimuth;
        self.elevation = def.elevation;
        self.distance_km = def.distance_km;
        self.saved_views[self.target_body.index()] = def;
    }

    /// Rotate orbital perspective to frame the Sun directly in view alongside the target body.
    pub fn align_to_sun(&mut self, sun_pos: Vec3) {
        if self.target_body == BodyId::Sun {
            return;
        }
        let target_to_sun = sun_pos - self.target_pos;
        if target_to_sun.length_squared() < 1.0 {
            return;
        }
        let dir = target_to_sun.normalize();
        // Camera positioned on the terminator/night side looking towards target with Sun framed on the limb:
        let sun_az = dir.x.atan2(dir.z);
        self.azimuth = sun_az + std::f64::consts::PI - 0.28;
        self.elevation = 0.06;

        let idx = self.target_body.index();
        self.saved_views[idx].azimuth = self.azimuth;
        self.saved_views[idx].elevation = self.elevation;
    }

    pub fn set_target(&mut self, body: BodyId, body_pos: Vec3) {
        self.cancel_transition();
        let curr_idx = self.target_body.index();
        self.saved_views[curr_idx] = BodyViewState {
            azimuth: self.azimuth,
            elevation: self.elevation,
            distance_km: self.distance_km,
        };

        self.target_body = body;
        self.target_pos = body_pos;

        let target_view = self.saved_views[body.index()];
        self.azimuth = target_view.azimuth;
        self.elevation = target_view.elevation;
        self.distance_km = target_view.distance_km;
    }

    /// Compute camera position in world space.
    pub fn position(&self) -> Vec3 {
        if self.free_flight {
            return self.free_pos;
        }
        let (sin_az, cos_az) = self.azimuth.sin_cos();
        let (sin_el, cos_el) = self.elevation.sin_cos();

        let offset = Vec3::new(
            self.distance_km * cos_el * sin_az,
            self.distance_km * sin_el,
            self.distance_km * cos_el * cos_az,
        );
        self.target_pos + offset
    }

    /// Generate ray for terminal pixel (u, v) in a buffer of size (cols, v_height).
    #[inline(always)]
    pub fn generate_ray(
        &self,
        u: usize,
        v: usize,
        cols: usize,
        v_height: usize,
        mode: RenderMode,
    ) -> Ray {
        let cam_pos = self.position();
        let (forward, right, up) = if self.free_flight {
            let (sy, cy) = self.free_yaw.sin_cos();
            let (sp, cp) = self.free_pitch.sin_cos();
            let f = Vec3::new(cp * sy, sp, cp * cy).normalize();
            let r = f.cross(Vec3::Y).normalize();
            let u = r.cross(f).normalize();
            (f, r, u)
        } else {
            let (sin_az, cos_az) = self.azimuth.sin_cos();
            let (sin_el, cos_el) = self.elevation.sin_cos();
            let f = Vec3::new(-cos_el * sin_az, -sin_el, -cos_el * cos_az);
            let r = Vec3::new(cos_az, 0.0, -sin_az);
            let u = Vec3::new(-sin_el * sin_az, cos_el, -sin_el * cos_az);
            (f, r, u)
        };

        // Terminal pixel aspect ratio correction:
        // In ASCII, 1 char is ~2x taller than wide.
        // In HalfBlock, 1 pixel is ~square (1.0).
        let pixel_aspect = match mode {
            RenderMode::Ascii => 2.15,
            RenderMode::HalfBlock => 1.05,
        };

        let screen_aspect = (cols as f64) / (v_height as f64) / pixel_aspect;

        let tan_half_fov_y = (self.fov_y_rad * 0.5).tan();
        let tan_half_fov_x = tan_half_fov_y * screen_aspect;

        let ndc_x = (2.0 * (u as f64 + 0.5) / (cols as f64) - 1.0) * tan_half_fov_x;
        let ndc_y = (1.0 - 2.0 * (v as f64 + 0.5) / (v_height as f64)) * tan_half_fov_y;

        let ray_dir = (forward + right * ndc_x + up * ndc_y).normalize();
        Ray::new(cam_pos, ray_dir)
    }
}
