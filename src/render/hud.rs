//! Astronomical Telemetry HUD overlay.

use crate::physics::body::BodyId;
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::{FrameBuffer, Pixel, RenderMode};
use crate::render::camera::Camera;

pub struct Hud;

impl Hud {
    pub fn render(
        buffer: &mut FrameBuffer,
        camera: &Camera,
        _physics: &SolarSystemPhysics,
        fps: f64,
        time_warp: f64,
        paused: bool,
    ) {
        let cols = buffer.cols;
        let lines = buffer.lines;
        if cols < 20 || lines < 6 {
            return;
        }

        let target_name = if let Some(ref trans) = camera.transition {
            let from = match trans.from_body {
                BodyId::Earth => "Earth",
                BodyId::Moon => "Moon",
                BodyId::Sun => "Sun",
            };
            let to = match trans.to_body {
                BodyId::Earth => "Earth",
                BodyId::Moon => "Moon",
                BodyId::Sun => "Sun",
            };
            format!("Transit: {} -> {}", from, to)
        } else {
            match camera.target_body {
                BodyId::Earth => "Earth (Terra)".to_string(),
                BodyId::Moon => "Moon (Luna)".to_string(),
                BodyId::Sun => "Sun (Sol)".to_string(),
            }
        };

        let dist_str = if camera.distance_km < 1_000_000.0 {
            format!("{:.0} km", camera.distance_km)
        } else {
            format!("{:.4} AU ({:.0} km)", camera.distance_km / 149_597_870.7, camera.distance_km)
        };

        let mode_str = match buffer.mode {
            RenderMode::Ascii => "ASCII",
            RenderMode::HalfBlock => "Half-Block (HD)",
        };

        let warp_str = if paused {
            "PAUSED".to_string()
        } else if time_warp < 60.0 {
            format!("{:.1}x", time_warp)
        } else if time_warp < 3600.0 {
            format!("{:.0} min/s", time_warp / 60.0)
        } else if time_warp < 86400.0 {
            format!("{:.1} hr/s", time_warp / 3600.0)
        } else {
            format!("{:.1} days/s", time_warp / 86400.0)
        };

        // Top Header
        let title = format!("AstroEngine · {} · {}", target_name, dist_str);
        let status = format!("{} · {} · {:.0} FPS", mode_str, warp_str, fps);

        if cols > title.len() + status.len() + 4 {
            Self::draw_text(buffer, 1, 0, &title, [180, 215, 255]);
            Self::draw_text(buffer, cols - status.len() - 1, 0, &status, [200, 215, 230]);
        } else {
            Self::draw_text(buffer, 1, 0, &title, [180, 215, 255]);
        }

        // Bottom Controls Guide
        let controls = "[+/-] Zoom  [WASD/Arrows] Orbit  [R] Reset  [1] Earth  [2] Moon  [B] Mode  [Space] Pause  [[/]] Warp  [Q] Quit";
        if lines > 1 && cols >= controls.len() + 2 {
            Self::draw_text(buffer, 1, lines - 1, controls, [150, 165, 185]);
        }
    }

    fn draw_text(buffer: &mut FrameBuffer, x: usize, y: usize, text: &str, color: [u8; 3]) {
        let v_y = match buffer.mode {
            RenderMode::Ascii => y,
            RenderMode::HalfBlock => y * 2,
        };

        for (i, c) in text.chars().enumerate() {
            let px = x + i;
            if px < buffer.cols {
                buffer.set_pixel(px, v_y, Pixel::text(color[0], color[1], color[2], c));
            }
        }
    }
}
