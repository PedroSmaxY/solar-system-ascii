//! Main simulation loop at 60 FPS with responsive interaction.

use crate::math::Vec3;
use crate::physics::body::BodyId;
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::{FrameBuffer, RenderMode};
use crate::render::camera::Camera;
use crate::render::hud::Hud;
use crate::render::raycaster::Raycaster;
use crate::terminal::{TerminalEvent, TerminalManager};
use crossterm::event::{KeyCode, KeyModifiers};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

const WORLDS: [BodyId; 12] = [
    BodyId::Sun,
    BodyId::Mercury,
    BodyId::Venus,
    BodyId::Earth,
    BodyId::Mars,
    BodyId::Ceres,
    BodyId::Vesta,
    BodyId::Jupiter,
    BodyId::Saturn,
    BodyId::Uranus,
    BodyId::Neptune,
    BodyId::Pluto,
];

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
    pub show_help: bool,
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
            show_help: false,
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

            // Handle non-blocking keyboard and mouse input
            while let Some(evt) = self.term.poll_event() {
                match evt {
                    TerminalEvent::MouseClick { col, row } => {
                        let b_cols = self.buffer.cols;
                        let b_lines = self.buffer.lines;
                        if self.show_help {
                            self.show_help = false;
                        } else if row >= b_lines.saturating_sub(1) as u16
                            && col >= b_cols.saturating_sub(12) as u16
                        {
                            self.show_help = true;
                        } else {
                            // Click-to-Fly: Raycast screen coordinates to nearest celestial body
                            let cam_pos = self.camera.position();
                            let (sin_az, cos_az) = self.camera.azimuth.sin_cos();
                            let (sin_el, cos_el) = self.camera.elevation.sin_cos();
                            let f = Vec3::new(-cos_el * sin_az, -sin_el, -cos_el * cos_az);
                            let r = Vec3::new(cos_az, 0.0, -sin_az);
                            let u = Vec3::new(-sin_el * sin_az, cos_el, -sin_el * cos_az);
                            let pixel_aspect = match self.buffer.mode {
                                RenderMode::Ascii => 2.15,
                                RenderMode::HalfBlock => 1.05,
                            };
                            let v_height = self.buffer.virtual_height();
                            let screen_aspect = (b_cols as f64) / (v_height as f64) / pixel_aspect;
                            let tan_half_fov_y = (self.camera.fov_y_rad * 0.5).tan();
                            let tan_half_fov_x = tan_half_fov_y * screen_aspect;

                            const TARGETABLE_BODIES: [BodyId; 32] = [
                                BodyId::Sun, BodyId::Mercury, BodyId::Venus, BodyId::Earth, BodyId::Moon,
                                BodyId::Mars, BodyId::Phobos, BodyId::Deimos, BodyId::Ceres, BodyId::Vesta,
                                BodyId::Jupiter, BodyId::Io, BodyId::Europa, BodyId::Ganymede, BodyId::Callisto,
                                BodyId::Saturn, BodyId::Titan, BodyId::Enceladus, BodyId::Mimas, BodyId::Iapetus, BodyId::Rhea,
                                BodyId::Uranus, BodyId::Miranda, BodyId::Ariel, BodyId::Umbriel, BodyId::Titania, BodyId::Oberon,
                                BodyId::Neptune, BodyId::Triton, BodyId::Proteus, BodyId::Pluto, BodyId::Charon,
                            ];

                            let c_col = col as i32;
                            let c_row = row as i32;
                            let mut clicked_body: Option<BodyId> = None;

                            for &b_id in &TARGETABLE_BODIES {
                                if b_id == self.camera.target_body {
                                    continue;
                                }
                                let b_pos = self.physics.get_body(b_id).position;
                                let to_b = b_pos - cam_pos;
                                let dist = to_b.length();
                                if dist < 1.0 {
                                    continue;
                                }
                                let dir = to_b / dist;
                                let f_dot = dir.dot(f);
                                if f_dot <= 0.05 {
                                    continue;
                                }
                                let ndc_x = dir.dot(r) / (f_dot * tan_half_fov_x);
                                let ndc_y = dir.dot(u) / (f_dot * tan_half_fov_y);
                                if ndc_x.abs() < 0.98 && ndc_y.abs() < 0.95 {
                                    let px = ((ndc_x + 1.0) * 0.5 * b_cols as f64) as i32;
                                    let py = ((1.0 - ndc_y) * 0.5 * b_lines as f64) as i32;
                                    let hit_center = (c_col - px).abs() <= 3 && (c_row - py).abs() <= 1;
                                    let hit_label = c_col >= px && c_col <= px + 12 && (c_row - py).abs() <= 1;
                                    if hit_center || hit_label {
                                        clicked_body = Some(b_id);
                                        break;
                                    }
                                }
                            }

                            if let Some(target) = clicked_body {
                                let from_pos = self.physics.get_body(self.camera.target_body).position;
                                let to_pos = self.physics.get_body(target).position;
                                if target != BodyId::Sun {
                                    self.camera.last_focused_planet = target;
                                }
                                self.camera.start_transition(target, from_pos, to_pos, 1.4);
                            }
                        }
                    }
                    TerminalEvent::MouseScrollUp => {
                        self.camera.zoom_in();
                    }
                    TerminalEvent::MouseScrollDown => {
                        if self.camera.zoom_out() {
                            let from_pos = self.physics.get_body(self.camera.target_body).position;
                            let to_pos = self.physics.sun.position;
                            self.camera.start_transition(BodyId::Sun, from_pos, to_pos, 1.8);
                        }
                    }
                    TerminalEvent::MouseDrag { dx, dy } => {
                        self.camera.orbit(dx as f64 * -0.022, dy as f64 * 0.035);
                    }
                    TerminalEvent::Key(key, modifiers) => {
                        if modifiers.contains(KeyModifiers::CONTROL) {
                            match key {
                                KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Char('d') | KeyCode::Char('D') => return,
                                _ => {}
                            }
                        }
                        match key {
                            KeyCode::Char('q') | KeyCode::Char('Q') => return,
                            KeyCode::Esc => {
                                if self.show_help {
                                    self.show_help = false;
                                } else {
                                    return;
                                }
                            }
                            KeyCode::Char('?') | KeyCode::F(1) | KeyCode::Char('/') => {
                                self.show_help = !self.show_help;
                            }
                            KeyCode::Backspace => {
                                if self.show_help {
                                    self.show_help = false;
                                } else if self.camera.target_body == BodyId::Sun {
                                    if self.camera.distance_km < 5.0e7 {
                                        self.camera.toggle_sun_zoom();
                                    }
                                } else if let Some(parent) = self.camera.target_body.parent() {
                                    let from_pos = self.physics.get_body(self.camera.target_body).position;
                                    let to_pos = self.physics.get_body(parent).position;
                                    self.camera.start_transition(parent, from_pos, to_pos, 1.3);
                                }
                            }
                            KeyCode::Tab | KeyCode::BackTab => {
                                let is_backtab = key == KeyCode::BackTab || modifiers.contains(KeyModifiers::SHIFT);
                                let cur_world = if WORLDS.contains(&self.camera.target_body) {
                                    self.camera.target_body
                                } else {
                                    self.camera.target_body.parent().unwrap_or(BodyId::Sun)
                                };
                                let cur_idx = WORLDS.iter().position(|&b| b == cur_world).unwrap_or(0);
                                let next_idx = if is_backtab {
                                    (cur_idx + WORLDS.len() - 1) % WORLDS.len()
                                } else {
                                    (cur_idx + 1) % WORLDS.len()
                                };
                                let dest = WORLDS[next_idx];
                                let from_pos = self.physics.get_body(self.camera.target_body).position;
                                let to_pos = self.physics.get_body(dest).position;
                                if dest != BodyId::Sun {
                                    self.camera.last_focused_planet = dest;
                                }
                                self.camera.start_transition(dest, from_pos, to_pos, 1.4);
                            }
                            KeyCode::Char('+') | KeyCode::Char('=') => self.camera.zoom_in(),
                            KeyCode::Char('-') | KeyCode::Char('_') => {
                                if self.camera.zoom_out() {
                                    let from_pos = self.physics.get_body(self.camera.target_body).position;
                                    let to_pos = self.physics.sun.position;
                                    self.camera.start_transition(BodyId::Sun, from_pos, to_pos, 1.8);
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
                                let (from_body, to_body, dur) = match self.camera.target_body {
                                    BodyId::Earth => (BodyId::Earth, BodyId::Moon, 1.0),
                                    BodyId::Moon => (BodyId::Moon, BodyId::Earth, 1.0),
                                    BodyId::Mars => (BodyId::Mars, BodyId::Phobos, 1.0),
                                    BodyId::Phobos => (BodyId::Phobos, BodyId::Deimos, 1.0),
                                    BodyId::Deimos => (BodyId::Deimos, BodyId::Mars, 1.0),
                                    BodyId::Ceres => (BodyId::Ceres, BodyId::Vesta, 1.2),
                                    BodyId::Vesta => (BodyId::Vesta, BodyId::Ceres, 1.2),
                                    BodyId::Jupiter => (BodyId::Jupiter, BodyId::Io, 1.1),
                                    BodyId::Io => (BodyId::Io, BodyId::Europa, 1.1),
                                    BodyId::Europa => (BodyId::Europa, BodyId::Ganymede, 1.1),
                                    BodyId::Ganymede => (BodyId::Ganymede, BodyId::Callisto, 1.1),
                                    BodyId::Callisto => (BodyId::Callisto, BodyId::Jupiter, 1.2),
                                    BodyId::Saturn => (BodyId::Saturn, BodyId::Titan, 1.2),
                                    BodyId::Titan => (BodyId::Titan, BodyId::Enceladus, 1.1),
                                    BodyId::Enceladus => (BodyId::Enceladus, BodyId::Mimas, 1.0),
                                    BodyId::Mimas => (BodyId::Mimas, BodyId::Iapetus, 1.3),
                                    BodyId::Iapetus => (BodyId::Iapetus, BodyId::Rhea, 1.2),
                                    BodyId::Rhea => (BodyId::Rhea, BodyId::Saturn, 1.3),
                                    BodyId::Uranus => (BodyId::Uranus, BodyId::Miranda, 1.1),
                                    BodyId::Miranda => (BodyId::Miranda, BodyId::Ariel, 1.1),
                                    BodyId::Ariel => (BodyId::Ariel, BodyId::Umbriel, 1.1),
                                    BodyId::Umbriel => (BodyId::Umbriel, BodyId::Titania, 1.1),
                                    BodyId::Titania => (BodyId::Titania, BodyId::Oberon, 1.1),
                                    BodyId::Oberon => (BodyId::Oberon, BodyId::Uranus, 1.3),
                                    BodyId::Neptune => (BodyId::Neptune, BodyId::Triton, 1.2),
                                    BodyId::Triton => (BodyId::Triton, BodyId::Proteus, 1.0),
                                    BodyId::Proteus => (BodyId::Proteus, BodyId::Neptune, 1.2),
                                    BodyId::Pluto => (BodyId::Pluto, BodyId::Charon, 1.1),
                                    BodyId::Charon => (BodyId::Charon, BodyId::Pluto, 1.1),
                                    _ => (self.camera.target_body, self.camera.target_body, 0.0),
                                };
                                if from_body != to_body {
                                    let from_pos = self.physics.get_body(from_body).position;
                                    let to_pos = self.physics.get_body(to_body).position;
                                    self.camera.start_transition(to_body, from_pos, to_pos, dur);
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                let body = match self.camera.target_body {
                                    BodyId::Moon => BodyId::Earth,
                                    BodyId::Phobos | BodyId::Deimos => BodyId::Mars,
                                    BodyId::Vesta => BodyId::Ceres,
                                    BodyId::Io | BodyId::Europa | BodyId::Ganymede | BodyId::Callisto => BodyId::Jupiter,
                                    BodyId::Titan | BodyId::Enceladus | BodyId::Mimas | BodyId::Iapetus | BodyId::Rhea => BodyId::Saturn,
                                    BodyId::Miranda | BodyId::Ariel | BodyId::Umbriel | BodyId::Titania | BodyId::Oberon => BodyId::Uranus,
                                    BodyId::Triton | BodyId::Proteus => BodyId::Neptune,
                                    BodyId::Charon => BodyId::Pluto,
                                    other => other,
                                };
                                let default_dist = Camera::default_view(body).distance_km;
                                let sys_dist = match body {
                                    BodyId::Sun => 1.95e10,
                                    BodyId::Mercury => 85_000.0,
                                    BodyId::Venus => 120_000.0,
                                    BodyId::Earth => 600_000.0,
                                    BodyId::Mars => 45_000.0,
                                    BodyId::Ceres => 650_000_000.0,
                                    BodyId::Jupiter => 2_800_000.0,
                                    BodyId::Saturn => 1_800_000.0,
                                    BodyId::Uranus => 1_200_000.0,
                                    BodyId::Neptune => 1_500_000.0,
                                    BodyId::Pluto => 35_000.0,
                                    _ => 1.95e10,
                                };
                                let target_dist = if (self.camera.distance_km - default_dist).abs() < (self.camera.distance_km - sys_dist).abs() {
                                    sys_dist
                                } else {
                                    default_dist
                                };
                                let mut end_view = Camera::default_view(body);
                                end_view.distance_km = target_dist;
                                if (target_dist - sys_dist).abs() < 1.0 {
                                    end_view.elevation = end_view.elevation.max(0.38);
                                }
                                let from_pos = self.physics.get_body(self.camera.target_body).position;
                                let to_pos = self.physics.get_body(body).position;
                                self.camera.transition_to_view(body, from_pos, to_pos, end_view, 1.2);
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
                            KeyCode::Char('\\') | KeyCode::Char('|') => {
                                self.time_warp = 600.0;
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

            // Interactive Help Modal
            if self.show_help {
                Hud::render_help_modal(&mut self.buffer);
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
