//! Astronomical Telemetry HUD overlay.

use crate::math::Vec3;
use crate::physics::body::BodyId;
use crate::physics::SolarSystemPhysics;
use crate::render::buffer::{FrameBuffer, Pixel, RenderMode};
use crate::render::camera::Camera;

pub struct Hud;

impl Hud {
    pub fn render(
        buffer: &mut FrameBuffer,
        camera: &Camera,
        physics: &SolarSystemPhysics,
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

        // Top Header (Line 0)
        let title = format!("AstroEngine · {} · {}", target_name, dist_str);
        let status = format!("{} · {} · {:.0} FPS", mode_str, warp_str, fps);

        if cols > title.len() + status.len() + 4 {
            Self::draw_text(buffer, 1, 0, &title, [180, 215, 255]);
            Self::draw_text(buffer, cols - status.len() - 1, 0, &status, [200, 215, 230]);
        } else {
            Self::draw_text(buffer, 1, 0, &title, [180, 215, 255]);
        }

        // Solar Direction & On-Screen Pip for Earth and Moon perspectives
        if camera.target_body != BodyId::Sun {
            let cam_pos = camera.position();
            let sun_vec = physics.sun.position - cam_pos;
            let sun_dist_km = sun_vec.length();
            let sun_dist_au = sun_dist_km / 149_597_870.7;
            let sun_dir = sun_vec / sun_dist_km;

            // Camera orientation vectors
            let (sin_az, cos_az) = camera.azimuth.sin_cos();
            let (sin_el, cos_el) = camera.elevation.sin_cos();
            let f = Vec3::new(-cos_el * sin_az, -sin_el, -cos_el * cos_az);
            let r = Vec3::new(cos_az, 0.0, -sin_az);
            let u = Vec3::new(-sin_el * sin_az, cos_el, -sin_el * cos_az);

            let f_dot = sun_dir.dot(f);
            let r_dot = sun_dir.dot(r);
            let u_dot = sun_dir.dot(u);

            let pixel_aspect = match buffer.mode {
                RenderMode::Ascii => 2.15,
                RenderMode::HalfBlock => 1.05,
            };
            let v_height = buffer.virtual_height();
            let screen_aspect = (cols as f64) / (v_height as f64) / pixel_aspect;
            let tan_half_fov_y = (camera.fov_y_rad * 0.5).tan();
            let tan_half_fov_x = tan_half_fov_y * screen_aspect;

            let in_fov = if f_dot > 0.05 {
                let ndc_x = r_dot / (f_dot * tan_half_fov_x);
                let ndc_y = u_dot / (f_dot * tan_half_fov_y);
                if ndc_x.abs() < 0.90 && ndc_y.abs() < 0.85 {
                    let px = ((ndc_x + 1.0) * 0.5 * cols as f64) as usize;
                    let py = ((1.0 - ndc_y) * 0.5 * lines as f64) as usize;
                    if px + 8 < cols && py > 1 && py < lines - 2 {
                        Self::draw_text(buffer, px, py, "[* Sol]", [255, 220, 90]);
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

            // Secondary Solar Telemetry (Line 1)
            if lines > 4 {
                let dir_guide = if in_fov {
                    "EM TELA [*]".to_string()
                } else {
                    let mut dir = String::new();
                    if r_dot > 0.25 {
                        dir.push_str("Girar Direita >");
                    } else if r_dot < -0.25 {
                        dir.push_str("< Girar Esquerda");
                    }
                    if u_dot > 0.35 {
                        if !dir.is_empty() { dir.push(' '); }
                        dir.push_str("^ Acima");
                    } else if u_dot < -0.35 {
                        if !dir.is_empty() { dir.push(' '); }
                        dir.push_str("v Abaixo");
                    }
                    if dir.is_empty() {
                        dir = "Atras da visao".to_string();
                    }
                    format!("[{}] ([F] p/ alinhar)", dir)
                };

                let nav_text = format!("Sol: {:.2} AU | Direcao: {}", sun_dist_au, dir_guide);
                if cols >= nav_text.len() + 2 {
                    Self::draw_text(buffer, 1, 1, &nav_text, [255, 205, 110]);
                }
            }
        }

        // Bottom Controls Guide (Bottom Line)
        let controls = "[+/-] Zoom  [WASD] Orbit  [R] Reset  [1] Earth  [2] Moon  [3] Sun  [F] Find Sun  [B] Mode  [Space] Pause  [[/]] Warp  [Q] Quit";
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
