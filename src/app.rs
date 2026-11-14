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
        }
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
                    KeyCode::Char('-') | KeyCode::Char('_') => self.camera.zoom_out(),
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
                    KeyCode::Char('1') => {
                        self.camera.set_target(BodyId::Earth, self.physics.earth.position);
                    }
                    KeyCode::Char('2') => {
                        self.camera.set_target(BodyId::Moon, self.physics.moon.position);
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
                    KeyCode::Char('[') => {
                        self.time_warp = (self.time_warp * 0.5).max(1.0);
                    }
                    KeyCode::Char(']') => {
                        self.time_warp = (self.time_warp * 2.0).min(100_000.0);
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

            // Keep camera target updated to body position
            let target_pos = self.physics.get_body(self.camera.target_body).position;
            self.camera.target_pos = target_pos;

            // Render scene
            self.raycaster.render_frame(
                &mut self.buffer,
                &self.camera,
                &self.physics,
                self.tick,
                self.sim_time,
            );

            // Overlay HUD
            Hud::render(
                &mut self.buffer,
                &self.camera,
                &self.physics,
                fps,
                self.time_warp,
                self.paused,
            );

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
