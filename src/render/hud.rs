//! Astronomical Telemetry HUD overlay.

use crate::math::Vec3;
use crate::physics::body::BodyId;
use crate::physics::ephemeris::{VOYAGER_1_VEC_KM, VOYAGER_2_VEC_KM};
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
                BodyId::Jupiter => "Jupiter",
                BodyId::Io => "Io",
                BodyId::Europa => "Europa",
                BodyId::Ganymede => "Ganymede",
                BodyId::Callisto => "Callisto",
                BodyId::Saturn => "Saturn",
                BodyId::Titan => "Titan",
                BodyId::Enceladus => "Enceladus",
                BodyId::Mimas => "Mimas",
                BodyId::Iapetus => "Iapetus",
                BodyId::Rhea => "Rhea",
                BodyId::Uranus => "Uranus",
                BodyId::Miranda => "Miranda",
                BodyId::Ariel => "Ariel",
                BodyId::Umbriel => "Umbriel",
                BodyId::Titania => "Titania",
                BodyId::Oberon => "Oberon",
                BodyId::Neptune => "Neptune",
                BodyId::Triton => "Triton",
                BodyId::Proteus => "Proteus",
                BodyId::Pluto => "Pluto",
                BodyId::Charon => "Charon",
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
                BodyId::Jupiter => "Jupiter",
                BodyId::Io => "Io",
                BodyId::Europa => "Europa",
                BodyId::Ganymede => "Ganymede",
                BodyId::Callisto => "Callisto",
                BodyId::Saturn => "Saturn",
                BodyId::Titan => "Titan",
                BodyId::Enceladus => "Enceladus",
                BodyId::Mimas => "Mimas",
                BodyId::Iapetus => "Iapetus",
                BodyId::Rhea => "Rhea",
                BodyId::Uranus => "Uranus",
                BodyId::Miranda => "Miranda",
                BodyId::Ariel => "Ariel",
                BodyId::Umbriel => "Umbriel",
                BodyId::Titania => "Titania",
                BodyId::Oberon => "Oberon",
                BodyId::Neptune => "Neptune",
                BodyId::Triton => "Triton",
                BodyId::Proteus => "Proteus",
                BodyId::Pluto => "Pluto",
                BodyId::Charon => "Charon",
            };
            format!("Transit: {} -> {}", from, to)
        } else {
            match camera.target_body {
                BodyId::Sun => "Solar System Overview (Sol)".to_string(),
                BodyId::Mercury => "Mercury (Mercúrio)".to_string(),
                BodyId::Venus => "Venus (Vênus)".to_string(),
                BodyId::Earth => "Earth (Terra)".to_string(),
                BodyId::Moon => "Moon (Luna)".to_string(),
                BodyId::Mars => "Mars (Marte)".to_string(),
                BodyId::Phobos => "Phobos (Fobos)".to_string(),
                BodyId::Deimos => "Deimos".to_string(),
                BodyId::Ceres => "Ceres (Dwarf Planet)".to_string(),
                BodyId::Vesta => "Vesta (Protoplanet)".to_string(),
                BodyId::Jupiter => "Jupiter (Gas Giant)".to_string(),
                BodyId::Io => "Io (Volcanic Moon)".to_string(),
                BodyId::Europa => "Europa (Ice Ocean Moon)".to_string(),
                BodyId::Ganymede => "Ganymede (Largest Moon)".to_string(),
                BodyId::Callisto => "Callisto (Cratered Moon)".to_string(),
                BodyId::Saturn => "Saturn (Ringed Planet)".to_string(),
                BodyId::Titan => "Titan (Hazy Giant Moon)".to_string(),
                BodyId::Enceladus => "Enceladus (Geyser Ice Moon)".to_string(),
                BodyId::Mimas => "Mimas (Herschel Crater Moon)".to_string(),
                BodyId::Iapetus => "Iapetus (Yin-Yang Moon)".to_string(),
                BodyId::Rhea => "Rhea (Cratered Ice Moon)".to_string(),
                BodyId::Uranus => "Uranus (Ice Giant, 97.8° Tilt)".to_string(),
                BodyId::Miranda => "Miranda (Verona Rupes Scarp)".to_string(),
                BodyId::Ariel => "Ariel (Bright Rift Moon)".to_string(),
                BodyId::Umbriel => "Umbriel (Wunda Crater Moon)".to_string(),
                BodyId::Titania => "Titania (Largest Uranian Moon)".to_string(),
                BodyId::Oberon => "Oberon (Cratered Mountain Moon)".to_string(),
                BodyId::Neptune => "Neptune (Great Dark Spot)".to_string(),
                BodyId::Triton => "Triton (Retrograde Cryo Moon)".to_string(),
                BodyId::Proteus => "Proteus (Polyhedral Moon)".to_string(),
                BodyId::Pluto => "Pluto (Tombaugh Regio Heart)".to_string(),
                BodyId::Charon => "Charon (Serenity Chasma Moon)".to_string(),
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
            (BodyId::Jupiter, physics.jupiter.position, physics.jupiter.radius_km, "[Júpiter]", [235, 195, 150]),
            (BodyId::Io, physics.io.position, physics.io.radius_km, "[Io]", [240, 215, 80]),
            (BodyId::Europa, physics.europa.position, physics.europa.radius_km, "[Europa]", [215, 225, 235]),
            (BodyId::Ganymede, physics.ganymede.position, physics.ganymede.radius_km, "[Ganímedes]", [185, 175, 165]),
            (BodyId::Callisto, physics.callisto.position, physics.callisto.radius_km, "[Calisto]", [150, 145, 140]),
            (BodyId::Saturn, physics.saturn.position, physics.saturn.radius_km, "[Saturno]", [235, 205, 140]),
            (BodyId::Titan, physics.titan.position, physics.titan.radius_km, "[Titã]", [230, 160, 60]),
            (BodyId::Enceladus, physics.enceladus.position, physics.enceladus.radius_km, "[Encélado]", [245, 250, 255]),
            (BodyId::Mimas, physics.mimas.position, physics.mimas.radius_km, "[Mimas]", [200, 200, 205]),
            (BodyId::Iapetus, physics.iapetus.position, physics.iapetus.radius_km, "[Jápeto]", [170, 160, 150]),
            (BodyId::Rhea, physics.rhea.position, physics.rhea.radius_km, "[Reia]", [210, 215, 220]),
            (BodyId::Uranus, physics.uranus.position, physics.uranus.radius_km, "[Urano]", [160, 215, 225]),
            (BodyId::Miranda, physics.miranda.position, physics.miranda.radius_km, "[Miranda]", [190, 195, 200]),
            (BodyId::Ariel, physics.ariel.position, physics.ariel.radius_km, "[Ariel]", [215, 220, 225]),
            (BodyId::Umbriel, physics.umbriel.position, physics.umbriel.radius_km, "[Umbriel]", [140, 140, 145]),
            (BodyId::Titania, physics.titania.position, physics.titania.radius_km, "[Titânia]", [185, 185, 190]),
            (BodyId::Oberon, physics.oberon.position, physics.oberon.radius_km, "[Oberon]", [175, 175, 180]),
            (BodyId::Neptune, physics.neptune.position, physics.neptune.radius_km, "[Netuno]", [90, 145, 245]),
            (BodyId::Triton, physics.triton.position, physics.triton.radius_km, "[Tritão]", [215, 225, 235]),
            (BodyId::Proteus, physics.proteus.position, physics.proteus.radius_km, "[Proteu]", [130, 130, 135]),
            (BodyId::Pluto, physics.pluto.position, physics.pluto.radius_km, "[Plutão]", [220, 195, 175]),
            (BodyId::Charon, physics.charon.position, physics.charon.radius_km, "[Caronte]", [185, 175, 170]),
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

        // Interstellar Probes Telemetry (Voyager 1 & Voyager 2 visible in deep space zoom)
        if camera.distance_km > 3.0e9 {
            let probes = [
                (VOYAGER_1_VEC_KM, "[Voyager 1 (163.5 AU)]", [130, 215, 255]),
                (VOYAGER_2_VEC_KM, "[Voyager 2 (136.8 AU)]", [110, 195, 240]),
            ];
            for &(pos, label, color) in &probes {
                let to_pos = pos - cam_pos;
                let dist_km = to_pos.length();
                let dir = to_pos / dist_km;
                let f_dot = dir.dot(f);
                if f_dot > 0.05 {
                    let r_dot = dir.dot(r);
                    let u_dot = dir.dot(u);
                    let ndc_x = r_dot / (f_dot * tan_half_fov_x);
                    let ndc_y = u_dot / (f_dot * tan_half_fov_y);
                    if ndc_x.abs() < 0.90 && ndc_y.abs() < 0.85 {
                        let px = ((ndc_x + 1.0) * 0.5 * cols as f64) as usize;
                        let py = ((1.0 - ndc_y) * 0.5 * lines as f64) as usize;
                        let label_x = px + 2;
                        let label_y = if py > 2 { py - 1 } else { py + 1 };
                        if label_x + label.len() < cols && label_y > 1 && label_y < lines - 2 {
                            Self::draw_text(buffer, label_x, label_y, label, color);
                        }
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

        // Standardized Clean Controls (Bottom Line)
        let controls = if camera.target_body == BodyId::Sun {
            "[0/Tab] Sol/Visão Geral  [1-8] Planetas  [9] Plutão  [L] Órbitas  [+/-] Zoom  [WASD] Órbita  [Space] Pausa  [Q] Sair"
        } else {
            "[Tab] Visão Geral  [1-8] Planetas  [9] Plutão  [M] Luas  [L] Órbitas  [R] Ângulo Ideal  [+/-] Zoom  [Space] Pausa"
        };
        if lines > 1 && cols >= controls.len() + 2 {
            Self::draw_text(buffer, 1, lines - 1, controls, [150, 185, 215]);
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
