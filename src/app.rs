//! Main simulation loop at 60 FPS with responsive interaction.

use crate::physics::body::BodyId;
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::{FrameBuffer, RenderMode};
use crate::render::camera::Camera;
use crate::render::hud::Hud;
use crate::render::raycaster::Raycaster;
use crate::terminal::TerminalManager;
use crossterm::event::KeyCode;
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

pub struct App {
    term: TerminalManager,
    physics: SolarSystemPhysics,
    raycaster: Raycaster,
    camera: Camera,
    buffer: FrameBuffer,
    time_warp: f64,
    paused: bool,
    sim_time: f64,
    tick: u64,
    pub show_hud: bool,
}

impl App {
    pub fn new(initial_mode: RenderMode) -> Self {
        let (cols, lines) = TerminalManager::size();
        let buffer = FrameBuffer::new(cols, lines, initial_mode);

        Self {
            term: TerminalManager::new(),
            physics: SolarSystemPhysics::new(),
            raycaster: Raycaster::new(),
            camera: Camera::new(),
            buffer,
            time_warp: 600.0, // Initial time warp: 1 sec = 10 min (visible smooth rotation)
            paused: false,
            sim_time: 0.0,
            tick: 0,
            show_hud: true,
        }
    }

    pub fn set_target(&mut self, body: BodyId) {
        let pos = self.physics.get_body(body).position;
        self.camera.set_target(body, pos);
    }

    pub fn set_view(&mut self, azimuth: f64, elevation: f64, distance: Option<f64>) {
        self.camera.azimuth = azimuth;
        self.camera.elevation = elevation;
        if let Some(d) = distance {
            self.camera.distance_km = d;
        }
        let idx = self.camera.target_body.index();
        self.camera.saved_views[idx].azimuth = self.camera.azimuth;
        self.camera.saved_views[idx].elevation = self.camera.elevation;
        self.camera.saved_views[idx].distance_km = self.camera.distance_km;
    }

    pub fn run(&mut self, test_frames: Option<u64>) {
        self.term.init();

        let mut last_frame = Instant::now();
        let mut fps = 60.0;
        let mut frame_count = 0u64;

        let target_frame_duration = Duration::from_micros(16_666); // ~60 FPS
        let mut out = stdout();

        loop {
            let frame_start = Instant::now();

            // Handle non-blocking keyboard input
            while let Some(key) = TerminalManager::poll_key() {
                match key {
                    KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return,
                    KeyCode::Char('+') | KeyCode::Char('=') => self.camera.zoom_in(),
                    KeyCode::Char('-') | KeyCode::Char('_') => {
                        if self.camera.zoom_out() {
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.sun.position;
                            self.camera.start_transition(BodyId::Sun, from_pos, to_pos, 1.8);
                        }
                    }
                    KeyCode::Tab => {
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        if self.camera.target_body == BodyId::Sun {
                            let dest = self.camera.last_focused_planet;
                            let to_pos = self.physics.get_body(dest).position;
                            self.camera.start_transition(dest, from_pos, to_pos, 1.5);
                        } else {
                            self.camera.last_focused_planet = self.camera.target_body;
                            let to_pos = self.physics.sun.position;
                            self.camera.start_transition(BodyId::Sun, from_pos, to_pos, 1.5);
                        }
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        self.camera.show_orbit_lines = !self.camera.show_orbit_lines;
                    }
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                        self.camera.orbit(-0.08, 0.0);
                    }
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                        self.camera.orbit(0.08, 0.0);
                    }
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        self.camera.orbit(0.0, 0.06);
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        self.camera.orbit(0.0, -0.06);
                    }
                    KeyCode::Char('0') => {
                        if self.camera.target_body == BodyId::Sun {
                            self.camera.toggle_sun_zoom();
                        } else {
                            self.camera.last_focused_planet = self.camera.target_body;
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.sun.position;
                            self.camera.start_transition(BodyId::Sun, from_pos, to_pos, 1.5);
                        }
                    }
                    KeyCode::Char('z') | KeyCode::Char('Z') => {
                        if self.camera.target_body == BodyId::Sun {
                            self.camera.toggle_sun_zoom();
                        }
                    }
                    KeyCode::Char('1') => {
                        self.camera.last_focused_planet = BodyId::Mercury;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.mercury.position;
                        self.camera.start_transition(BodyId::Mercury, from_pos, to_pos, 1.3);
                    }
                    KeyCode::Char('2') => {
                        self.camera.last_focused_planet = BodyId::Venus;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.venus.position;
                        self.camera.start_transition(BodyId::Venus, from_pos, to_pos, 1.2);
                    }
                    KeyCode::Char('3') => {
                        self.camera.last_focused_planet = BodyId::Earth;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.earth.position;
                        self.camera.start_transition(BodyId::Earth, from_pos, to_pos, 1.2);
                    }
                    KeyCode::Char('4') => {
                        self.camera.last_focused_planet = BodyId::Mars;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.mars.position;
                        self.camera.start_transition(BodyId::Mars, from_pos, to_pos, 1.3);
                    }
                    KeyCode::Char('5') => {
                        self.camera.last_focused_planet = BodyId::Jupiter;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.jupiter.position;
                        self.camera.start_transition(BodyId::Jupiter, from_pos, to_pos, 1.5);
                    }
                    KeyCode::Char('6') => {
                        self.camera.last_focused_planet = BodyId::Saturn;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.saturn.position;
                        self.camera.start_transition(BodyId::Saturn, from_pos, to_pos, 1.6);
                    }
                    KeyCode::Char('7') => {
                        self.camera.last_focused_planet = BodyId::Uranus;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.uranus.position;
                        self.camera.start_transition(BodyId::Uranus, from_pos, to_pos, 1.7);
                    }
                    KeyCode::Char('8') => {
                        self.camera.last_focused_planet = BodyId::Neptune;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.neptune.position;
                        self.camera.start_transition(BodyId::Neptune, from_pos, to_pos, 1.8);
                    }
                    KeyCode::Char('9') => {
                        self.camera.last_focused_planet = BodyId::Pluto;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.pluto.position;
                        self.camera.start_transition(BodyId::Pluto, from_pos, to_pos, 1.8);
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        self.camera.last_focused_planet = BodyId::Ceres;
                        let from_pos = self.physics.get_body(self.camera.target_body).position;
                        let to_pos = self.physics.ceres.position;
                        self.camera.start_transition(BodyId::Ceres, from_pos, to_pos, 1.4);
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        if self.camera.target_body == BodyId::Earth {
                            let from_pos = self.physics.earth.position;
                            let to_pos = self.physics.moon.position;
                            self.camera.start_transition(BodyId::Moon, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Moon {
                            let from_pos = self.physics.moon.position;
                            let to_pos = self.physics.earth.position;
                            self.camera.start_transition(BodyId::Earth, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Mars {
                            let from_pos = self.physics.mars.position;
                            let to_pos = self.physics.phobos.position;
                            self.camera.start_transition(BodyId::Phobos, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Phobos {
                            let from_pos = self.physics.phobos.position;
                            let to_pos = self.physics.deimos.position;
                            self.camera.start_transition(BodyId::Deimos, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Deimos {
                            let from_pos = self.physics.deimos.position;
                            let to_pos = self.physics.phobos.position;
                            self.camera.start_transition(BodyId::Phobos, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Ceres {
                            let from_pos = self.physics.ceres.position;
                            let to_pos = self.physics.vesta.position;
                            self.camera.start_transition(BodyId::Vesta, from_pos, to_pos, 1.2);
                        } else if self.camera.target_body == BodyId::Vesta {
                            let from_pos = self.physics.vesta.position;
                            let to_pos = self.physics.ceres.position;
                            self.camera.start_transition(BodyId::Ceres, from_pos, to_pos, 1.2);
                        } else if self.camera.target_body == BodyId::Jupiter {
                            let from_pos = self.physics.jupiter.position;
                            let to_pos = self.physics.io.position;
                            self.camera.start_transition(BodyId::Io, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Io {
                            let from_pos = self.physics.io.position;
                            let to_pos = self.physics.europa.position;
                            self.camera.start_transition(BodyId::Europa, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Europa {
                            let from_pos = self.physics.europa.position;
                            let to_pos = self.physics.ganymede.position;
                            self.camera.start_transition(BodyId::Ganymede, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Ganymede {
                            let from_pos = self.physics.ganymede.position;
                            let to_pos = self.physics.callisto.position;
                            self.camera.start_transition(BodyId::Callisto, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Callisto {
                            let from_pos = self.physics.callisto.position;
                            let to_pos = self.physics.io.position;
                            self.camera.start_transition(BodyId::Io, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Saturn {
                            let from_pos = self.physics.saturn.position;
                            let to_pos = self.physics.titan.position;
                            self.camera.start_transition(BodyId::Titan, from_pos, to_pos, 1.2);
                        } else if self.camera.target_body == BodyId::Titan {
                            let from_pos = self.physics.titan.position;
                            let to_pos = self.physics.enceladus.position;
                            self.camera.start_transition(BodyId::Enceladus, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Enceladus {
                            let from_pos = self.physics.enceladus.position;
                            let to_pos = self.physics.mimas.position;
                            self.camera.start_transition(BodyId::Mimas, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Mimas {
                            let from_pos = self.physics.mimas.position;
                            let to_pos = self.physics.iapetus.position;
                            self.camera.start_transition(BodyId::Iapetus, from_pos, to_pos, 1.3);
                        } else if self.camera.target_body == BodyId::Iapetus {
                            let from_pos = self.physics.iapetus.position;
                            let to_pos = self.physics.rhea.position;
                            self.camera.start_transition(BodyId::Rhea, from_pos, to_pos, 1.2);
                        } else if self.camera.target_body == BodyId::Rhea {
                            let from_pos = self.physics.rhea.position;
                            let to_pos = self.physics.titan.position;
                            self.camera.start_transition(BodyId::Titan, from_pos, to_pos, 1.2);
                        } else if self.camera.target_body == BodyId::Uranus {
                            let from_pos = self.physics.uranus.position;
                            let to_pos = self.physics.miranda.position;
                            self.camera.start_transition(BodyId::Miranda, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Miranda {
                            let from_pos = self.physics.miranda.position;
                            let to_pos = self.physics.ariel.position;
                            self.camera.start_transition(BodyId::Ariel, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Ariel {
                            let from_pos = self.physics.ariel.position;
                            let to_pos = self.physics.umbriel.position;
                            self.camera.start_transition(BodyId::Umbriel, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Umbriel {
                            let from_pos = self.physics.umbriel.position;
                            let to_pos = self.physics.titania.position;
                            self.camera.start_transition(BodyId::Titania, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Titania {
                            let from_pos = self.physics.titania.position;
                            let to_pos = self.physics.oberon.position;
                            self.camera.start_transition(BodyId::Oberon, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Oberon {
                            let from_pos = self.physics.oberon.position;
                            let to_pos = self.physics.miranda.position;
                            self.camera.start_transition(BodyId::Miranda, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Neptune {
                            let from_pos = self.physics.neptune.position;
                            let to_pos = self.physics.triton.position;
                            self.camera.start_transition(BodyId::Triton, from_pos, to_pos, 1.2);
                        } else if self.camera.target_body == BodyId::Triton {
                            let from_pos = self.physics.triton.position;
                            let to_pos = self.physics.proteus.position;
                            self.camera.start_transition(BodyId::Proteus, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Proteus {
                            let from_pos = self.physics.proteus.position;
                            let to_pos = self.physics.triton.position;
                            self.camera.start_transition(BodyId::Triton, from_pos, to_pos, 1.2);
                        } else if self.camera.target_body == BodyId::Pluto {
                            let from_pos = self.physics.pluto.position;
                            let to_pos = self.physics.charon.position;
                            self.camera.start_transition(BodyId::Charon, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Charon {
                            let from_pos = self.physics.charon.position;
                            let to_pos = self.physics.pluto.position;
                            self.camera.start_transition(BodyId::Pluto, from_pos, to_pos, 1.1);
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        if self.camera.target_body == BodyId::Mars || self.camera.target_body == BodyId::Phobos {
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.deimos.position;
                            self.camera.start_transition(BodyId::Deimos, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Deimos {
                            let from_pos = self.physics.deimos.position;
                            let to_pos = self.physics.phobos.position;
                            self.camera.start_transition(BodyId::Phobos, from_pos, to_pos, 1.0);
                        } else if self.camera.target_body == BodyId::Ceres || self.camera.target_body == BodyId::Vesta {
                            self.camera.distance_km = 650_000_000.0;
                            self.camera.elevation = 0.55;
                        } else if self.camera.target_body == BodyId::Jupiter {
                            self.camera.distance_km = 2_800_000.0;
                            self.camera.elevation = 0.35;
                        } else if matches!(self.camera.target_body, BodyId::Io | BodyId::Europa | BodyId::Ganymede | BodyId::Callisto) {
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.jupiter.position;
                            self.camera.start_transition(BodyId::Jupiter, from_pos, to_pos, 1.1);
                        } else if self.camera.target_body == BodyId::Saturn {
                            self.camera.distance_km = 1_800_000.0;
                            self.camera.elevation = 0.42;
                        } else if matches!(self.camera.target_body, BodyId::Titan | BodyId::Enceladus | BodyId::Mimas | BodyId::Iapetus | BodyId::Rhea) {
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.saturn.position;
                            self.camera.start_transition(BodyId::Saturn, from_pos, to_pos, 1.3);
                        } else if self.camera.target_body == BodyId::Uranus {
                            self.camera.distance_km = 1_200_000.0;
                            self.camera.elevation = 0.55;
                        } else if matches!(self.camera.target_body, BodyId::Miranda | BodyId::Ariel | BodyId::Umbriel | BodyId::Titania | BodyId::Oberon) {
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.uranus.position;
                            self.camera.start_transition(BodyId::Uranus, from_pos, to_pos, 1.3);
                        } else if self.camera.target_body == BodyId::Neptune {
                            self.camera.distance_km = 1_500_000.0;
                            self.camera.elevation = 0.40;
                        } else if matches!(self.camera.target_body, BodyId::Triton | BodyId::Proteus) {
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.neptune.position;
                            self.camera.start_transition(BodyId::Neptune, from_pos, to_pos, 1.3);
                        }
                    }
                    KeyCode::Char('v') | KeyCode::Char('V') => {
                        if self.camera.target_body == BodyId::Venus {
                            self.raycaster.venus_shader.radar_mode = !self.raycaster.venus_shader.radar_mode;
                        }
                    }
                    KeyCode::Char('f') | KeyCode::Char('F') => {
                        self.camera.align_to_sun(self.physics.sun.position);
                    }
                    KeyCode::Char('h') | KeyCode::Char('H') => {
                        self.show_hud = !self.show_hud;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        self.camera.recenter();
                    }
                    KeyCode::Char('b') | KeyCode::Char('B') => {
                        let new_mode = match self.buffer.mode {
                            RenderMode::Ascii => RenderMode::HalfBlock,
                            RenderMode::HalfBlock => RenderMode::Ascii,
                        };
                        self.buffer.set_mode(new_mode);
                    }
                    KeyCode::Char(' ') => {
                        self.paused = !self.paused;
                    }
                    KeyCode::Char('t') | KeyCode::Char('T') => {
                        // Quick cycle through astronomical time warp scales:
                        // 10 min/s (600x) -> 1 day/s (86,400x) -> 1 month/s (2,592,000x) -> 1 year/s (31,536,000x) -> 5 years/s (157,680,000x) -> Realtime (1x)
                        self.time_warp = if self.time_warp < 1000.0 {
                            86_400.0
                        } else if self.time_warp < 100_000.0 {
                            2_592_000.0
                        } else if self.time_warp < 10_000_000.0 {
                            31_536_000.0
                        } else if self.time_warp < 100_000_000.0 {
                            157_680_000.0
                        } else if self.time_warp < 250_000_000.0 {
                            1.0
                        } else {
                            600.0
                        };
                    }
                    KeyCode::Char('[') | KeyCode::Char(',') => {
                        self.time_warp = (self.time_warp * 0.5).max(1.0);
                    }
                    KeyCode::Char(']') | KeyCode::Char('.') => {
                        self.time_warp = (self.time_warp * 2.0).min(315_360_000.0);
                    }
                    _ => {}
                }
            }

            // Check terminal resize
            let (cols, lines) = TerminalManager::size();
            self.buffer.resize(cols, lines);

            // Compute delta time
            let now = Instant::now();
            let dt = now.duration_since(last_frame).as_secs_f64();
            last_frame = now;
            if dt > 0.0 {
                fps = 0.9 * fps + 0.1 * (1.0 / dt);
            }

            // Update simulation
            if !self.paused {
                self.sim_time += dt * self.time_warp;
                self.tick += 1;
                self.physics.update(self.sim_time);
            }

            // Update camera flight transition and track moving target body
            self.camera.update(dt, &self.physics);

            // Render scene
            self.raycaster.render_frame(
                &mut self.buffer,
                &self.camera,
                &self.physics,
                self.tick,
                self.sim_time,
            );

            // Overlay HUD (if enabled)
            if self.show_hud {
                Hud::render(
                    &mut self.buffer,
                    &self.camera,
                    &self.physics,
                    fps,
                    self.time_warp,
                    self.paused,
                );
            }

            // Flush frame buffer to terminal
            let output = self.buffer.flush_to_string();
            let _ = out.write_all(output.as_bytes());
            let _ = out.flush();

            frame_count += 1;
            if let Some(max_f) = test_frames {
                if frame_count >= max_f {
                    break;
                }
            }

            // Throttle to target FPS
            let elapsed_frame = frame_start.elapsed();
            if elapsed_frame < target_frame_duration {
                std::thread::sleep(target_frame_duration - elapsed_frame);
            }
        }
    }
}
