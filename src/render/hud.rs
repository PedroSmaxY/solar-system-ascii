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
                BodyId::Sun => "Sun",
                BodyId::Mercury => "Mercury",
                BodyId::Venus => "Venus",
                BodyId::Earth => "Earth",
                BodyId::Moon => "Moon",
                BodyId::Mars => "Mars",
                BodyId::Phobos => "Phobos",
                BodyId::Deimos => "Deimos",
                BodyId::Ceres => "Ceres",
                BodyId::Vesta => "Vesta",
            };
            let to = match trans.to_body {
                BodyId::Sun => "Sun",
                BodyId::Mercury => "Mercury",
                BodyId::Venus => "Venus",
                BodyId::Earth => "Earth",
                BodyId::Moon => "Moon",
                BodyId::Mars => "Mars",
                BodyId::Phobos => "Phobos",
                BodyId::Deimos => "Deimos",
                BodyId::Ceres => "Ceres",
                BodyId::Vesta => "Vesta",
            };
            format!("Transit: {} -> {}", from, to)
        } else {
            match camera.target_body {
                BodyId::Sun => "Sun (Sol)".to_string(),
                BodyId::Mercury => "Mercury (Mercúrio)".to_string(),
                BodyId::Venus => "Venus (Vênus)".to_string(),
                BodyId::Earth => "Earth (Terra)".to_string(),
                BodyId::Moon => "Moon (Luna)".to_string(),
                BodyId::Mars => "Mars (Marte)".to_string(),
                BodyId::Phobos => "Phobos (Fobos)".to_string(),
                BodyId::Deimos => "Deimos".to_string(),
                BodyId::Ceres => "Ceres (Dwarf Planet)".to_string(),
                BodyId::Vesta => "Vesta (Protoplanet)".to_string(),
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

        // 3D Celestial Body Labels ([Sol], [Terra], [Lua]) with elegant offsets
        let cam_pos = camera.position();
        let (sin_az, cos_az) = camera.azimuth.sin_cos();
        let (sin_el, cos_el) = camera.elevation.sin_cos();
        let f = Vec3::new(-cos_el * sin_az, -sin_el, -cos_el * cos_az);
        let r = Vec3::new(cos_az, 0.0, -sin_az);
        let u = Vec3::new(-sin_el * sin_az, cos_el, -sin_el * cos_az);

        let pixel_aspect = match buffer.mode {
            RenderMode::Ascii => 2.15,
            RenderMode::HalfBlock => 1.05,
        };
        let v_height = buffer.virtual_height();
        let screen_aspect = (cols as f64) / (v_height as f64) / pixel_aspect;
        let tan_half_fov_y = (camera.fov_y_rad * 0.5).tan();
        let tan_half_fov_x = tan_half_fov_y * screen_aspect;

        let bodies = [
            (BodyId::Sun, physics.sun.position, physics.sun.radius_km, "[Sol]", [255, 220, 90]),
            (BodyId::Mercury, physics.mercury.position, physics.mercury.radius_km, "[Mercúrio]", [225, 185, 140]),
            (BodyId::Venus, physics.venus.position, physics.venus.radius_km, "[Vênus]", [240, 225, 175]),
            (BodyId::Earth, physics.earth.position, physics.earth.radius_km, "[Terra]", [120, 205, 255]),
            (BodyId::Moon, physics.moon.position, physics.moon.radius_km, "[Lua]", [210, 215, 225]),
            (BodyId::Mars, physics.mars.position, physics.mars.radius_km, "[Marte]", [225, 115, 65]),
            (BodyId::Phobos, physics.phobos.position, physics.phobos.radius_km, "[Fobos]", [160, 155, 150]),
            (BodyId::Deimos, physics.deimos.position, physics.deimos.radius_km, "[Deimos]", [175, 160, 145]),
            (BodyId::Ceres, physics.ceres.position, physics.ceres.radius_km, "[Ceres]", [200, 205, 215]),
            (BodyId::Vesta, physics.vesta.position, physics.vesta.radius_km, "[Vesta]", [220, 210, 190]),
        ];

        let mut sun_in_fov = false;
        let mut sun_r_dot = 0.0;
        let mut sun_u_dot = 0.0;
        let mut sun_dist_au = 1.0;

        for &(id, pos, rad, label, color) in &bodies {
            let to_pos = pos - cam_pos;
            let dist_km = to_pos.length();
            if dist_km < 1.0 {
                continue;
            }

            // Target Proximity Culling: when closely orbiting and observing the target body,
            // hide its own label so it does not obstruct the surface features.
            // When zoomed out (dist >= 10.0 * radius), the label reappears as an orbital tag.
            // Other bodies (e.g. Moon viewed from Earth) remain visible to help locate them in the sky.
            if id == camera.target_body && dist_km < rad * 10.0 {
                continue;
            }

            let dir = to_pos / dist_km;
            let f_dot = dir.dot(f);
            let r_dot = dir.dot(r);
            let u_dot = dir.dot(u);

            if id == BodyId::Sun {
                sun_dist_au = dist_km / 149_597_870.7;
                sun_r_dot = r_dot;
                sun_u_dot = u_dot;
            }

            if f_dot > 0.05 {
                let ndc_x = r_dot / (f_dot * tan_half_fov_x);
                let ndc_y = u_dot / (f_dot * tan_half_fov_y);
                if ndc_x.abs() < 0.90 && ndc_y.abs() < 0.85 {
                    if id == BodyId::Sun {
                        sun_in_fov = true;
                    }
                    let px = ((ndc_x + 1.0) * 0.5 * cols as f64) as usize;
                    let py = ((1.0 - ndc_y) * 0.5 * lines as f64) as usize;
                    // Position label with an elegant offset beside the celestial body:
                    let label_x = px + 2;
                    let label_y = if py > 2 { py - 1 } else { py + 1 };
                    if label_x + label.len() < cols && label_y > 1 && label_y < lines - 2 {
                        Self::draw_text(buffer, label_x, label_y, label, color);
                    }
                }
            }
        }

        // Secondary Solar Telemetry (Line 1)
        if camera.target_body != BodyId::Sun && lines > 4 {
            let dir_guide = if sun_in_fov {
                "EM TELA [Sol]".to_string()
            } else {
                let mut dir = String::new();
                if sun_r_dot > 0.25 {
                    dir.push_str("Girar Direita >");
                } else if sun_r_dot < -0.25 {
                    dir.push_str("< Girar Esquerda");
                }
                if sun_u_dot > 0.35 {
                    if !dir.is_empty() { dir.push(' '); }
                    dir.push_str("^ Acima");
                } else if sun_u_dot < -0.35 {
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

        // Bottom Controls Guide (Bottom Line)
        let mut controls = String::from("[1] Sun  [2] Mercury  [3] Venus  [4] Earth  [5] Mars  [6] Ceres");
        if camera.target_body == BodyId::Earth {
            controls.push_str("  [M] Moon");
        } else if camera.target_body == BodyId::Moon {
            controls.push_str("  [M] Earth");
        } else if camera.target_body == BodyId::Venus {
            controls.push_str("  [V] Radar");
        } else if camera.target_body == BodyId::Mars {
            controls.push_str("  [M] Phobos  [N] Deimos");
        } else if camera.target_body == BodyId::Phobos {
            controls.push_str("  [M] Mars  [N] Deimos");
        } else if camera.target_body == BodyId::Deimos {
            controls.push_str("  [M] Mars  [N] Phobos");
        } else if camera.target_body == BodyId::Ceres {
            controls.push_str("  [M] Vesta  [N] Asteroids");
        } else if camera.target_body == BodyId::Vesta {
            controls.push_str("  [M] Ceres  [N] Asteroids");
        }
        controls.push_str("  [+/-] Zoom  [WASD] Orbit  [R] Reset  [F] Find Sun  [H] Labels  [B] Mode  [Space] Pause  [Q] Quit");
        if lines > 1 && cols >= controls.len() + 2 {
            Self::draw_text(buffer, 1, lines - 1, &controls, [150, 165, 185]);
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
