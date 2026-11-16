//! 6-DOF Astronomical Camera with continuous logarithmic/exponential zoom,
//! independent per-body perspective memory, and smooth interplanetary fly-to transitions.

use crate::math::{Ray, Vec3};
use crate::physics::body::BodyId;
use crate::physics::ephemeris::{VOYAGER_1_VEC_KM, VOYAGER_2_VEC_KM};
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::RenderMode;
use crate::render::voyager_shader::VoyagerShader;
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
    pub from_pos: Vec3,
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
    pub saved_views: [BodyViewState; 34], // [Sun..Charon, Voyager1, Voyager2]
    pub transition: Option<CameraTransition>,
    pub show_orbit_lines: bool,
    pub last_focused_planet: BodyId,

    // Smooth camera inertia
    pub vel_azimuth: f64,
    pub vel_elevation: f64,

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
        let voyager1_view = Self::default_view(BodyId::Voyager1);
        let voyager2_view = Self::default_view(BodyId::Voyager2);

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
                voyager1_view,
                voyager2_view,
            ],
            transition: None,
            show_orbit_lines: true,
            last_focused_planet: BodyId::Earth,
            vel_azimuth: 0.0,
            vel_elevation: 0.0,
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
                azimuth: -1.45,       // Framing sunlit gas giant, majestic rings and shadow cast on ring plane
                elevation: 0.16,      // Iconic oblique perspective viewing ring system and shadow bands
                distance_km: 200_000.0, // Framing ~120,000 km disk and 280,000 km ring system (~58% screen height)
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
            BodyId::Voyager1 => Self::voyager_view(
                VOYAGER_1_VEC_KM,
                Vec3::ZERO,
                Vec3::new(0.65, 0.35, 0.68),
                0.012, // 12 meters beauty angle
            ),
            BodyId::Voyager2 => Self::voyager_view(
                VOYAGER_2_VEC_KM,
                Vec3::ZERO,
                Vec3::new(0.65, 0.35, 0.68),
                0.012, // 12 meters beauty angle
            ),
        }
    }

    /// Calculate exact world azimuth and elevation to frame a target on Voyager given a local probe direction vector.
    pub fn voyager_view(target_pos: Vec3, sun_pos: Vec3, local_dir: Vec3, distance_km: f64) -> BodyViewState {
        let mat = VoyagerShader::orientation_matrix(target_pos, sun_pos);
        let dir = mat.mul_vec(local_dir.normalize()).normalize();
        let az = dir.x.atan2(dir.z).rem_euclid(2.0 * PI);
        let el = dir.y.clamp(-0.99, 0.99).asin();
        BodyViewState {
            azimuth: az,
            elevation: el,
            distance_km,
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
        let end_view = self.saved_views[to_body.index()];
        if to_body == self.target_body && self.transition.is_none() {
            if (self.distance_km - end_view.distance_km).abs() < 10.0 {
                return;
            }
        }

        // Only save current view state of the origin body if transitioning to a different body
        if self.transition.is_none() && to_body != self.target_body {
            let curr_idx = self.target_body.index();
            let start_view = BodyViewState {
                azimuth: self.azimuth,
                elevation: self.elevation,
                distance_km: self.distance_km,
            };
            self.saved_views[curr_idx] = start_view;
        }

        let start_pos = if self.transition.is_some() {
            self.target_pos
        } else {
            from_pos
        };

        let start_view = BodyViewState {
            azimuth: self.azimuth,
            elevation: self.elevation,
            distance_km: self.distance_km,
        };

        let dist_delta = (to_pos - start_pos).length();
        let from_body = self.target_body;
        let journey_dist_km = if from_body == to_body || dist_delta < 1.0 {
            0.0
        } else {
            dist_delta.max(10_000.0)
        };

        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;
        self.target_body = to_body;

        self.transition = Some(CameraTransition {
            from_body,
            to_body,
            from_pos: start_pos,
            start_view,
            end_view,
            elapsed_secs: 0.0,
            duration_secs: duration_secs.max(0.1),
            journey_dist_km,
        });
    }

    /// Transition to an explicit custom view state on a body (e.g. system overview toggle [N]).
    pub fn transition_to_view(
        &mut self,
        to_body: BodyId,
        from_pos: Vec3,
        to_pos: Vec3,
        end_view: BodyViewState,
        duration_secs: f64,
    ) {
        let start_pos = if self.transition.is_some() {
            self.target_pos
        } else {
            from_pos
        };
        let start_view = BodyViewState {
            azimuth: self.azimuth,
            elevation: self.elevation,
            distance_km: self.distance_km,
        };
        self.cancel_transition();
        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;

        let dist_delta = (to_pos - start_pos).length();
        let from_body = self.target_body;
        let journey_dist_km = if from_body == to_body || dist_delta < 1.0 {
            0.0
        } else {
            dist_delta.max(10_000.0)
        };

        self.target_body = to_body;
        self.saved_views[to_body.index()] = end_view;

        self.transition = Some(CameraTransition {
            from_body,
            to_body,
            from_pos: start_pos,
            start_view,
            end_view,
            elapsed_secs: 0.0,
            duration_secs: duration_secs.max(0.1),
            journey_dist_km,
        });
    }

    /// Smoothly toggle between close-up view of the Sun and the full Solar System Orrery view.
    pub fn toggle_sun_zoom(&mut self) {
        if self.target_body != BodyId::Sun && self.transition.is_none() {
            return;
        }
        self.cancel_transition();
        let start_view = BodyViewState {
            azimuth: self.azimuth,
            elevation: self.elevation,
            distance_km: self.distance_km,
        };

        // If currently in the solar system overview (> 50 million km), zoom into close-up Sun view
        let end_view = if self.distance_km > 5.0e7 {
            BodyViewState {
                azimuth: 0.0,
                elevation: 0.15,
                distance_km: 2_800_000.0, // Photorealistic solar disk close-up framing corona & loops
            }
        } else {
            BodyViewState {
                azimuth: 0.0,
                elevation: 0.0,
                distance_km: 1.1e10, // Top-down solar system orrery view
            }
        };

        self.transition = Some(CameraTransition {
            from_body: BodyId::Sun,
            to_body: BodyId::Sun,
            from_pos: Vec3::ZERO,
            start_view,
            end_view,
            elapsed_secs: 0.0,
            duration_secs: 1.2,
            journey_dist_km: 0.0,
        });
    }

    /// Smoothly toggle between 3m close-up view of the Golden Record and 12m spacecraft beauty angle.
    pub fn toggle_golden_record(&mut self, sun_pos: Vec3) {
        if self.target_body != BodyId::Voyager1 && self.target_body != BodyId::Voyager2 {
            return;
        }
        self.cancel_transition();
        let start_view = BodyViewState {
            azimuth: self.azimuth,
            elevation: self.elevation,
            distance_km: self.distance_km,
        };

        let end_view = if self.distance_km > 0.005 {
            // Direct face-on close-up towards the Golden Record on the +X bus chassis face
            Self::voyager_view(self.target_pos, sun_pos, Vec3::new(1.0, 0.0, 0.0), 0.003)
        } else {
            Self::default_view(self.target_body)
        };

        let body = self.target_body;
        self.saved_views[body.index()] = end_view;

        self.transition = Some(CameraTransition {
            from_body: body,
            to_body: body,
            from_pos: self.target_pos,
            start_view,
            end_view,
            elapsed_secs: 0.0,
            duration_secs: 0.8,
            journey_dist_km: 0.0,
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
        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;
    }

    /// Update camera flight transition animation and track moving target body.
    pub fn update(&mut self, dt: f64, physics: &SolarSystemPhysics) {
        if let Some(mut trans) = self.transition {
            trans.elapsed_secs += dt;
            let progress = (trans.elapsed_secs / trans.duration_secs).clamp(0.0, 1.0);

            // Quintic smoothstep curve C^2 continuous: 6p^5 - 15p^4 + 10p^3
            let s = progress * progress * progress * (progress * (progress * 6.0 - 15.0) + 10.0);

            // Interpolate target focal position between moving bodies
            let to_pos = physics.get_body(trans.to_body).position;
            self.target_pos = trans.from_pos * (1.0 - s) + to_pos * s;

            // Shortest circular angular arc for azimuth
            let d_az = ((trans.end_view.azimuth - trans.start_view.azimuth + PI).rem_euclid(2.0 * PI)) - PI;
            self.azimuth = (trans.start_view.azimuth + d_az * s).rem_euclid(2.0 * PI);

            // Elevation interpolation
            self.elevation = trans.start_view.elevation + (trans.end_view.elevation - trans.start_view.elevation) * s;

            // Logarithmic distance interpolation with parabolic altitude arc (zoom-out then zoom-in)
            let log_start = trans.start_view.distance_km.ln();
            let log_end = trans.end_view.distance_km.ln();
            let base_log = log_start * (1.0 - s) + log_end * s;

            // Apex altitude pulls out to reveal interplanetary scale (only for spatial travel between bodies)
            let current_log = if trans.journey_dist_km > 0.0 {
                let apex_dist = (trans.journey_dist_km * 0.70)
                    .max(trans.start_view.distance_km * 1.5)
                    .max(trans.end_view.distance_km * 1.5);
                let apex_log = apex_dist.ln();
                let arc_height = (apex_log - base_log).max(0.0);
                let arc_factor = 4.0 * s * (1.0 - s); // Parabola peaking at 1.0 at s=0.5
                base_log + arc_factor * arc_height
            } else {
                base_log
            };
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

            // Apply smooth orbital damping / inertia
            if self.vel_azimuth.abs() > 1e-5 || self.vel_elevation.abs() > 1e-5 {
                let decay = (-10.0 * dt).exp();
                self.azimuth = (self.azimuth + self.vel_azimuth * dt * 10.0).rem_euclid(2.0 * PI);
                self.elevation = (self.elevation + self.vel_elevation * dt * 10.0).clamp(-1.55, 1.55);
                self.vel_azimuth *= decay;
                self.vel_elevation *= decay;
                let idx = self.target_body.index();
                self.saved_views[idx].azimuth = self.azimuth;
                self.saved_views[idx].elevation = self.elevation;
            }
        }
    }

    pub fn zoom_in(&mut self) {
        self.cancel_transition();
        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;
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
            BodyId::Voyager1 | BodyId::Voyager2 => 0.002, // 2 meters close-up
        };
        self.distance_km = (self.distance_km * 0.88).max(min_dist);
        self.saved_views[self.target_body.index()].distance_km = self.distance_km;
    }

    /// Zoom out with smooth automated overview transition when threshold is exceeded.
    /// Returns true if an automated transition to the top-down Solar System view should be started.
    pub fn zoom_out(&mut self) -> bool {
        self.cancel_transition();
        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;
        self.distance_km = (self.distance_km * 1.14).min(3.5e10);
        self.saved_views[self.target_body.index()].distance_km = self.distance_km;

        let overview_threshold = match self.target_body {
            BodyId::Sun => 3.5e10,
            BodyId::Jupiter | BodyId::Saturn | BodyId::Uranus | BodyId::Neptune => 4.5e8,
            BodyId::Voyager1 | BodyId::Voyager2 => 0.06, // 60 meters
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
        self.elevation = (self.elevation + d_el).clamp(-1.55, 1.55);
        self.vel_azimuth = (self.vel_azimuth + d_az * 0.35).clamp(-0.5, 0.5);
        self.vel_elevation = (self.vel_elevation + d_el * 0.35).clamp(-0.5, 0.5);
        let idx = self.target_body.index();
        self.saved_views[idx].azimuth = self.azimuth;
        self.saved_views[idx].elevation = self.elevation;
    }

    pub fn recenter(&mut self) {
        self.cancel_transition();
        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;
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
        self.cancel_transition();
        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;

        let target_to_sun = sun_pos - self.target_pos;
        if target_to_sun.length_squared() < 1.0 {
            return;
        }
        let dir = target_to_sun.normalize();
        // Camera positioned on the terminator/night side looking towards target with Sun framed on the limb:
        let sun_az = dir.x.atan2(dir.z);
        self.azimuth = (sun_az + std::f64::consts::PI - 0.28).rem_euclid(2.0 * PI);
        self.elevation = 0.06;

        let idx = self.target_body.index();
        self.saved_views[idx].azimuth = self.azimuth;
        self.saved_views[idx].elevation = self.elevation;
    }

    pub fn set_target(&mut self, body: BodyId, body_pos: Vec3) {
        self.cancel_transition();
        self.vel_azimuth = 0.0;
        self.vel_elevation = 0.0;
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
