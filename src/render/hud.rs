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
        venus_radar: bool,
    ) {
        let cols = buffer.cols;
        let lines = buffer.lines;
        if cols < 20 || lines < 6 {
            return;
        }

        let target_name = if let Some(ref trans) = camera.transition {
            let from = trans.from_body.name();
            let to = trans.to_body.name();
            if trans.from_body == trans.to_body {
                if trans.from_body == BodyId::Sun {
                    if trans.end_view.distance_km < trans.start_view.distance_km {
                        "Zoom: Sun Close-up".to_string()
                    } else {
                        "Zoom: Solar System Overview".to_string()
                    }
                } else if trans.from_body == BodyId::Voyager1 || trans.from_body == BodyId::Voyager2 {
                    if trans.end_view.distance_km < trans.start_view.distance_km {
                        "Zoom: Golden Record (3m)".to_string()
                    } else {
                        format!("Zoom: {} (12m)", to)
                    }
                } else if trans.end_view.distance_km < trans.start_view.distance_km {
                    format!("Zoom: {} Close-up", to)
                } else {
                    format!("Zoom: {} System Overview", to)
                }
            } else {
                format!("Transit: {} -> {}", from, to)
            }
        } else if camera.is_in_system_view() && camera.target_body != BodyId::Sun {
            format!("Solar System › {} (System Overview)", camera.target_body.name())
        } else {
            match camera.target_body {
                BodyId::Sun => {
                    if camera.distance_km > 5.0e7 {
                        "Solar System Overview".to_string()
                    } else {
                        "Sun (Photosphere Close-up)".to_string()
                    }
                }
                BodyId::Mercury => "Solar System › Mercury".to_string(),
                BodyId::Venus => {
                    if venus_radar {
                        "Solar System › Venus (Magellan SAR Radar)".to_string()
                    } else {
                        "Solar System › Venus (Optical Atmosphere)".to_string()
                    }
                }
                BodyId::Earth => "Solar System › Earth".to_string(),
                BodyId::Moon => "Solar System › Earth › Moon".to_string(),
                BodyId::Mars => "Solar System › Mars".to_string(),
                BodyId::Phobos => "Solar System › Mars › Phobos (Moon 1/2)".to_string(),
                BodyId::Deimos => "Solar System › Mars › Deimos (Moon 2/2)".to_string(),
                BodyId::Ceres => "Solar System › Asteroids › Ceres (Dwarf Planet)".to_string(),
                BodyId::Vesta => "Solar System › Asteroids › Vesta (Protoplanet)".to_string(),
                BodyId::Jupiter => "Solar System › Jupiter".to_string(),
                BodyId::Io => "Solar System › Jupiter › Io (Moon 1/4)".to_string(),
                BodyId::Europa => "Solar System › Jupiter › Europa (Moon 2/4)".to_string(),
                BodyId::Ganymede => "Solar System › Jupiter › Ganymede (Moon 3/4)".to_string(),
                BodyId::Callisto => "Solar System › Jupiter › Callisto (Moon 4/4)".to_string(),
                BodyId::Saturn => "Solar System › Saturn".to_string(),
                BodyId::Titan => "Solar System › Saturn › Titan (Moon 1/5)".to_string(),
                BodyId::Enceladus => "Solar System › Saturn › Enceladus (Moon 2/5)".to_string(),
                BodyId::Mimas => "Solar System › Saturn › Mimas (Moon 3/5)".to_string(),
                BodyId::Iapetus => "Solar System › Saturn › Iapetus (Moon 4/5)".to_string(),
                BodyId::Rhea => "Solar System › Saturn › Rhea (Moon 5/5)".to_string(),
                BodyId::Uranus => "Solar System › Uranus (97.8° Tilt)".to_string(),
                BodyId::Miranda => "Solar System › Uranus › Miranda (Moon 1/5)".to_string(),
                BodyId::Ariel => "Solar System › Uranus › Ariel (Moon 2/5)".to_string(),
                BodyId::Umbriel => "Solar System › Uranus › Umbriel (Moon 3/5)".to_string(),
                BodyId::Titania => "Solar System › Uranus › Titania (Moon 4/5)".to_string(),
                BodyId::Oberon => "Solar System › Uranus › Oberon (Moon 5/5)".to_string(),
                BodyId::Neptune => "Solar System › Neptune".to_string(),
                BodyId::Triton => "Solar System › Neptune › Triton (Moon 1/2)".to_string(),
                BodyId::Proteus => "Solar System › Neptune › Proteus (Moon 2/2)".to_string(),
                BodyId::Pluto => "Solar System › Kuiper Belt › Pluto".to_string(),
                BodyId::Charon => "Solar System › Kuiper Belt › Charon".to_string(),
                BodyId::Voyager1 => {
                    let v1_au = (physics.voyager1.position - physics.sun.position).length() / crate::physics::ephemeris::AU_KM;
                    if camera.is_golden_record_view() {
                        "Solar System › Interstellar Space › Voyager 1 (Golden Record 3m)".to_string()
                    } else if camera.is_pale_blue_dot_view(physics.earth.position) {
                        "Solar System › Interstellar Space › Voyager 1 (Pale Blue Dot)".to_string()
                    } else {
                        format!("Solar System › Interstellar Space › Voyager 1 ({:.1} AU)", v1_au)
                    }
                }
                BodyId::Voyager2 => {
                    let v2_au = (physics.voyager2.position - physics.sun.position).length() / crate::physics::ephemeris::AU_KM;
                    if camera.is_golden_record_view() {
                        "Solar System › Interstellar Space › Voyager 2 (Golden Record 3m)".to_string()
                    } else if camera.is_pale_blue_dot_view(physics.earth.position) {
                        "Solar System › Interstellar Space › Voyager 2 (Pale Blue Dot)".to_string()
                    } else {
                        format!("Solar System › Interstellar Space › Voyager 2 ({:.1} AU)", v2_au)
                    }
                }
            }
        };

        let dist_str = if camera.distance_km < 1.0 {
            format!("{:.1} m", camera.distance_km * 1000.0)
        } else if camera.distance_km < 10_000_000.0 {
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
        } else if (time_warp - 1.0).abs() < 1e-3 {
            "LIVE (1.0x)".to_string()
        } else if time_warp < 60.0 {
            format!("{:.1}x", time_warp)
        } else if time_warp < 3600.0 {
            format!("{:.0} min/s", time_warp / 60.0)
        } else if time_warp < 86400.0 {
            format!("{:.1} hr/s", time_warp / 3600.0)
        } else if time_warp < 2_592_000.0 {
            format!("{:.1} days/s", time_warp / 86400.0)
        } else if time_warp < 31_536_000.0 {
            format!("{:.1} months/s", time_warp / 2_592_000.0)
        } else {
            format!("{:.1} years/s", time_warp / 31_536_000.0)
        };

        let (date_full, date_compact) = if physics.sim_time.abs() > 86400.0 * 365.25 * 5.0 {
            crate::physics::ephemeris::format_utc_datetime(physics.sim_time)
        } else {
            let sim_days = physics.sim_time / 86400.0;
            let rel = if sim_days.abs() < 365.25 {
                format!("T+ {:.1}d", sim_days)
            } else {
                format!("T+ {:.2}y", sim_days / 365.25)
            };
            (rel.clone(), rel)
        };

        // Top Header (Line 0)
        let title_full = format!("AstroEngine · {} · {}", target_name, dist_str);
        let status_full = format!("{} · {} · {} · {:.0} FPS", date_full, mode_str, warp_str, fps);
        let status_compact = format!("{} · {} · {:.0} FPS", date_compact, warp_str, fps);
        let status_min = format!("{} · {:.0} FPS", warp_str, fps);

        let title_len = title_full.chars().count();
        let status_len = status_full.chars().count();

        if cols > title_len + status_len + 4 {
            Self::draw_text(buffer, 1, 0, &title_full, [180, 215, 255]);
            Self::draw_text(buffer, cols - status_len - 1, 0, &status_full, [200, 215, 230]);
        } else {
            // Responsive fallback for standard 80x24 terminals:
            // Strip long prefix to guarantee status (PAUSED / Warp / FPS) remains 100% visible
            let target_short = target_name
                .trim_start_matches("AstroEngine · ")
                .trim_start_matches("Solar System › ");
            let title_short = format!("{} · {}", target_short, dist_str);
            let s = if cols >= title_short.chars().count() + status_compact.chars().count() + 3 {
                &status_compact
            } else {
                &status_min
            };
            let s_len = s.chars().count();
            let avail_title = cols.saturating_sub(s_len + 3);
            let title_fit: String = title_short.chars().take(avail_title).collect();

            Self::draw_text(buffer, 1, 0, &title_fit, [180, 215, 255]);
            if cols > s_len + 2 {
                Self::draw_text(buffer, cols - s_len - 1, 0, s, [200, 215, 230]);
            }
        }

        // Welcome Callout (Line 1) during the first 6 seconds of simulation at Earth
        if (physics.sim_time <= 3600.0 || physics.step_count <= 360) && !paused && camera.target_body == BodyId::Earth && lines > 4 {
            let welcome = "✦ Welcome to AstroEngine 3D · Press [0] Solar System Orrery  [1-9] Planets  [M] Moons  [Y] Voyagers  [?] Help ✦";
            let w_len = welcome.chars().count();
            if cols >= w_len + 2 {
                let wx = (cols - w_len) / 2;
                Self::draw_text(buffer, wx, 1, welcome, [255, 215, 110]);
            }
        }

        // 3D Celestial Body Labels ([Sun], [Earth], [Moon]) with elegant offsets
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

        // Major celestial bodies (Sun, Planets, Asteroid Belt Dwarfs) - always prioritized
        let major_bodies = [
            (BodyId::Sun, physics.sun.position, physics.sun.radius_km, "[Sun]", [255, 220, 90]),
            (BodyId::Mercury, physics.mercury.position, physics.mercury.radius_km, "[Mercury]", [225, 185, 140]),
            (BodyId::Venus, physics.venus.position, physics.venus.radius_km, "[Venus]", [240, 225, 175]),
            (BodyId::Earth, physics.earth.position, physics.earth.radius_km, "[Earth]", [120, 205, 255]),
            (BodyId::Mars, physics.mars.position, physics.mars.radius_km, "[Mars]", [225, 115, 65]),
            (BodyId::Ceres, physics.ceres.position, physics.ceres.radius_km, "[Ceres]", [200, 205, 215]),
            (BodyId::Vesta, physics.vesta.position, physics.vesta.radius_km, "[Vesta]", [220, 210, 190]),
            (BodyId::Jupiter, physics.jupiter.position, physics.jupiter.radius_km, "[Jupiter]", [235, 195, 150]),
            (BodyId::Saturn, physics.saturn.position, physics.saturn.radius_km, "[Saturn]", [235, 205, 140]),
            (BodyId::Uranus, physics.uranus.position, physics.uranus.radius_km, "[Uranus]", [160, 215, 225]),
            (BodyId::Neptune, physics.neptune.position, physics.neptune.radius_km, "[Neptune]", [90, 145, 245]),
            (BodyId::Pluto, physics.pluto.position, physics.pluto.radius_km, "[Pluto]", [220, 195, 175]),
        ];

        // Natural Satellites / Moons - only rendered within their planetary system's proximity (LOD)
        let moons = [
            (BodyId::Moon, physics.moon.position, physics.moon.radius_km, "[Moon]", [210, 215, 225], physics.earth.position, 6_000_000.0),
            (BodyId::Phobos, physics.phobos.position, physics.phobos.radius_km, "[Phobos]", [160, 155, 150], physics.mars.position, 1_500_000.0),
            (BodyId::Deimos, physics.deimos.position, physics.deimos.radius_km, "[Deimos]", [175, 160, 145], physics.mars.position, 1_500_000.0),
            (BodyId::Io, physics.io.position, physics.io.radius_km, "[Io]", [240, 215, 80], physics.jupiter.position, 40_000_000.0),
            (BodyId::Europa, physics.europa.position, physics.europa.radius_km, "[Europa]", [215, 225, 235], physics.jupiter.position, 40_000_000.0),
            (BodyId::Ganymede, physics.ganymede.position, physics.ganymede.radius_km, "[Ganymede]", [185, 175, 165], physics.jupiter.position, 40_000_000.0),
            (BodyId::Callisto, physics.callisto.position, physics.callisto.radius_km, "[Callisto]", [150, 145, 140], physics.jupiter.position, 40_000_000.0),
            (BodyId::Titan, physics.titan.position, physics.titan.radius_km, "[Titan]", [230, 160, 60], physics.saturn.position, 55_000_000.0),
            (BodyId::Enceladus, physics.enceladus.position, physics.enceladus.radius_km, "[Enceladus]", [245, 250, 255], physics.saturn.position, 55_000_000.0),
            (BodyId::Mimas, physics.mimas.position, physics.mimas.radius_km, "[Mimas]", [200, 200, 205], physics.saturn.position, 55_000_000.0),
            (BodyId::Iapetus, physics.iapetus.position, physics.iapetus.radius_km, "[Iapetus]", [170, 160, 150], physics.saturn.position, 55_000_000.0),
            (BodyId::Rhea, physics.rhea.position, physics.rhea.radius_km, "[Rhea]", [210, 215, 220], physics.saturn.position, 55_000_000.0),
            (BodyId::Miranda, physics.miranda.position, physics.miranda.radius_km, "[Miranda]", [190, 195, 200], physics.uranus.position, 25_000_000.0),
            (BodyId::Ariel, physics.ariel.position, physics.ariel.radius_km, "[Ariel]", [215, 220, 225], physics.uranus.position, 25_000_000.0),
            (BodyId::Umbriel, physics.umbriel.position, physics.umbriel.radius_km, "[Umbriel]", [140, 140, 145], physics.uranus.position, 25_000_000.0),
            (BodyId::Titania, physics.titania.position, physics.titania.radius_km, "[Titania]", [185, 185, 190], physics.uranus.position, 25_000_000.0),
            (BodyId::Oberon, physics.oberon.position, physics.oberon.radius_km, "[Oberon]", [175, 175, 180], physics.uranus.position, 25_000_000.0),
            (BodyId::Triton, physics.triton.position, physics.triton.radius_km, "[Triton]", [215, 225, 235], physics.neptune.position, 25_000_000.0),
            (BodyId::Proteus, physics.proteus.position, physics.proteus.radius_km, "[Proteus]", [130, 130, 135], physics.neptune.position, 25_000_000.0),
            (BodyId::Charon, physics.charon.position, physics.charon.radius_km, "[Charon]", [185, 175, 170], physics.pluto.position, 2_000_000.0),
        ];

        let mut occupied_regions: Vec<(usize, usize, usize)> = Vec::new(); // (row_y, x_start, x_end)

        let mut place_label = |id: BodyId, pos: Vec3, rad: f64, label: &str, color: [u8; 3]| {
            let to_pos = pos - cam_pos;
            let dist_km = to_pos.length();
            if dist_km < 1.0 {
                return;
            }

            // Target Proximity Culling: when closely orbiting and observing the target body,
            // hide its own label so it does not obstruct the surface features.
            // When zoomed out (dist >= 10.0 * radius), the label reappears as an orbital tag.
            if id == camera.target_body && dist_km < rad * 10.0 {
                return;
            }

            let dir = to_pos / dist_km;
            let f_dot = dir.dot(f);
            if f_dot <= 0.05 {
                return;
            }

            let r_dot = dir.dot(r);
            let u_dot = dir.dot(u);
            let ndc_x = r_dot / (f_dot * tan_half_fov_x);
            let ndc_y = u_dot / (f_dot * tan_half_fov_y);

            if ndc_x.abs() < 0.90 && ndc_y.abs() < 0.85 {
                let px = ((ndc_x + 1.0) * 0.5 * cols as f64) as usize;
                let py = ((1.0 - ndc_y) * 0.5 * lines as f64) as usize;
                let label_chars = label.chars().count();
                let label_x = px + 2;
                let preferred_y = if py > 2 { py - 1 } else { py + 1 };

                if label_x + label_chars < cols && preferred_y > 1 && preferred_y < lines - 2 {
                    let x_start = label_x;
                    let x_end = label_x + label_chars;

                    // Collision prevention: avoid overwriting higher-priority labels
                    let collides = occupied_regions.iter().any(|&(oy, ox1, ox2)| {
                        oy == preferred_y && !(x_end + 1 < ox1 || x_start > ox2 + 1)
                    });

                    let final_y = if collides {
                        let alt_y = if preferred_y == py - 1 {
                            py + 1
                        } else if py > 2 {
                            py - 1
                        } else {
                            preferred_y
                        };
                        let alt_collides = occupied_regions.iter().any(|&(oy, ox1, ox2)| {
                            oy == alt_y && !(x_end + 1 < ox1 || x_start > ox2 + 1)
                        });
                        if !alt_collides && alt_y > 1 && alt_y < lines - 2 {
                            alt_y
                        } else {
                            return;
                        }
                    } else {
                        preferred_y
                    };

                    occupied_regions.push((final_y, x_start, x_end));
                    Self::draw_text(buffer, label_x, final_y, label, color);
                }
            }
        };

        // Priority Pass: If viewing from Voyager, place Earth FIRST so it is not suppressed by Sun collision
        let at_voyager = camera.target_body == BodyId::Voyager1
            || camera.target_body == BodyId::Voyager2;
        if at_voyager {
            let pbd_earth_au = (physics.earth.position - cam_pos).length() / crate::physics::ephemeris::AU_KM;
            let pbd_label = format!("◉ [Earth · Pale Blue Dot ({:.1} AU)]", pbd_earth_au);
            place_label(
                BodyId::Earth,
                physics.earth.position,
                physics.earth.radius_km,
                &pbd_label,
                [135, 225, 255],
            );
        }

        // 1. First pass: Place all major celestial bodies (Planets, Asteroids, Sun)
        for &(id, pos, rad, label, color) in &major_bodies {
            if at_voyager && id == BodyId::Earth {
                continue; // Already placed with high priority
            }
            place_label(id, pos, rad, label, color);
        }

        // 2. Second pass: Place moons only when camera is within their planetary system
        for &(id, pos, rad, label, color, parent_pos, max_dist) in &moons {
            let dist_to_parent = (parent_pos - cam_pos).length();
            if dist_to_parent <= max_dist {
                place_label(id, pos, rad, label, color);
            }
        }

        // Interstellar Probes Telemetry (Voyager 1 & Voyager 2 visible in deep space zoom)
        if camera.distance_km > 3.0e9 {
            let v1_au = (physics.voyager1.position - physics.sun.position).length() / crate::physics::ephemeris::AU_KM;
            let v2_au = (physics.voyager2.position - physics.sun.position).length() / crate::physics::ephemeris::AU_KM;
            let v1_str = format!("[Voyager 1 ({:.1} AU)]", v1_au);
            let v2_str = format!("[Voyager 2 ({:.1} AU)]", v2_au);
            let probes = [
                (BodyId::Voyager1, physics.voyager1.position, physics.voyager1.radius_km, v1_str.as_str(), [130, 215, 255]),
                (BodyId::Voyager2, physics.voyager2.position, physics.voyager2.radius_km, v2_str.as_str(), [110, 195, 240]),
            ];
            for &(id, pos, rad, label, color) in &probes {
                place_label(id, pos, rad, label, color);
            }
        }

        // 100% Reactive Contextual Clean Controls (Bottom Line)
        let n_str = if camera.is_in_system_view() { "[N] Planet Close-up" } else { "[N] System View" };
        let n_comp = if camera.is_in_system_view() { "[N] Close-up" } else { "[N] System" };
        let zero_str = if camera.distance_km > 5.0e7 { "[0] Sun Close-up" } else { "[0] Solar Overview" };
        let z_str = if camera.is_golden_record_view() { "[Z] Full Craft (12m)" } else { "[Z] Golden Record (3m)" };
        let f_str = if camera.is_pale_blue_dot_view(physics.earth.position) { "[F] Craft View" } else { "[F] Pale Blue Dot" };
        let v_str = if venus_radar { "[V] Optical Clouds" } else { "[V] Radar View" };
        let space_str = if paused { "[Space] Resume" } else { "[Space] Pause" };
        let warp_ctrl = if (time_warp - 1.0).abs() > 1e-3 { "[T] Warp  [J] Live" } else { "[T] Warp" };

        let sun_n_str = if camera.is_in_system_view() { "[N] Inner Orrery" } else { "[N] Heliosphere" };
        let (mut controls_full, mut controls_compact) = match camera.target_body {
            BodyId::Sun => {
                if camera.distance_km > 5.0e7 {
                    (
                        format!("{}  [1-9/P] Worlds  [Y] Voyagers  {}  [Tab] Next  {}  {}  [?] Help", zero_str, sun_n_str, warp_ctrl, space_str),
                        format!("{}  [1-9/P] Worlds  [Y] Voyagers  [Tab] Next  [?] Help", zero_str),
                    )
                } else {
                    (
                        format!("{}  [1-9/P] Worlds  [Tab] Next  [Bksp] Back  {}  {}  [?] Help", zero_str, warp_ctrl, space_str),
                        format!("{}  [1-9/P] Worlds  [Tab] Next  [?] Help", zero_str),
                    )
                }
            }
            BodyId::Venus => (
                format!("{}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", v_str, warp_ctrl, space_str),
                format!("{}  [Tab] Next  [1-9/P] Worlds  [?] Help", v_str),
            ),
            BodyId::Mercury => (
                format!("[Tab] Next  [Bksp] Back  [1-9/P] Worlds  [R] Reset  {}  {}  [?] Help", warp_ctrl, space_str),
                "[Tab] Next  [1-9/P] Worlds  [+/-] Zoom  [?] Help".to_string(),
            ),
            BodyId::Earth | BodyId::Moon => (
                format!("[M] Moon Cycle  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Moon  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Mars | BodyId::Phobos | BodyId::Deimos => (
                format!("[M] Moons (Phobos/Deimos)  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Moons  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Jupiter | BodyId::Io | BodyId::Europa | BodyId::Ganymede | BodyId::Callisto => (
                format!("[M] Galilean Moons  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Moons  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Saturn
            | BodyId::Titan
            | BodyId::Enceladus
            | BodyId::Mimas
            | BodyId::Iapetus
            | BodyId::Rhea => (
                format!("[M] Saturn Moons  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Moons  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Uranus
            | BodyId::Miranda
            | BodyId::Ariel
            | BodyId::Umbriel
            | BodyId::Titania
            | BodyId::Oberon => (
                format!("[M] Uranus Moons  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Moons  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Neptune | BodyId::Triton | BodyId::Proteus => (
                format!("[M] Neptune Moons  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Moons  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Pluto | BodyId::Charon => (
                format!("[M] Pluto/Charon  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Charon  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Ceres | BodyId::Vesta => (
                format!("[M] Ceres/Vesta  {}  [Tab] Next  [Bksp] Back  [1-9/P] Worlds  {}  {}  [?] Help", n_str, warp_ctrl, space_str),
                format!("[M] Belt  {}  [Tab] Next  [?] Help", n_comp),
            ),
            BodyId::Voyager1 => (
                format!("{}  {}  [Y] Voyager 2  [Bksp] Overview  {}  {}  [?] Help", z_str, f_str, warp_ctrl, space_str),
                format!("{}  {}  [Y] V2", z_str, f_str),
            ),
            BodyId::Voyager2 => (
                format!("{}  {}  [Y] Voyager 1  [Bksp] Overview  {}  {}  [?] Help", z_str, f_str, warp_ctrl, space_str),
                format!("{}  {}  [Y] V1", z_str, f_str),
            ),
        };

        if camera.distance_km > 3.0e9 && camera.target_body != BodyId::Sun && camera.target_body != BodyId::Voyager1 && camera.target_body != BodyId::Voyager2 {
            controls_full.push_str("  [Y] Voyagers");
            controls_compact.push_str("  [Y] Voyagers");
        }

        if lines > 1 {
            let full_len = controls_full.chars().count();
            let comp_len = controls_compact.chars().count();
            if cols >= full_len + 2 {
                Self::draw_text(buffer, 1, lines - 1, &controls_full, [160, 195, 225]);
            } else if cols >= comp_len + 2 {
                Self::draw_text(buffer, 1, lines - 1, &controls_compact, [160, 195, 225]);
            } else if cols > 20 {
                Self::draw_text(buffer, 1, lines - 1, "[Tab] Next  [1-9/P] Worlds  [?] Help", [160, 195, 225]);
            }
        }
    }

    /// Centered interactive Help Modal displaying full keyboard and mouse controls.
    pub fn render_help_modal(buffer: &mut FrameBuffer) {
        let cols = buffer.cols;
        let lines = buffer.lines;
        let modal_w = 64.min(cols.saturating_sub(4));
        let lines_text = [
            "┌───────────────── AstroEngine 3D Navigation ──────────────────┐",
            "│ Orbit & Look    [W/A/S/D] or [Arrows] or Left-Click Drag     │",
            "│ Zoom In / Out   [+] / [-] or Mouse Scroll Wheel              │",
            "│ Click-to-Fly    Click any planet, moon, or [Name] to Fly     │",
            "│ Solar System    [0] Overview  [Tab / Shift+Tab] Cycle Worlds │",
            "│ Direct Target   [1-9] Mercury..Pluto  [P] Ceres/Vesta        │",
            "│ Moon Systems    [M] Cycle Moons & Return  [N] System View    │",
            "│ Interstellar    [Y] Voyager 1/2  [Z] Golden Record [F] Earth │",
            "│ Ascend / Back   [Backspace] Moon -> Planet -> Solar System   │",
            "│ Solar Alignment [F] Align to Sun  [Z] Sun Close-up Toggle    │",
            "│ Features        [L] Orbit Lines  [V] Venus Radar Mode        │",
            "│ Simulation     [Space] Pause  [T] Warp  [J] Live  [,/.] Fine │",
            "│ Display         [B] ASCII / HD Half-Block  [H] Toggle HUD    │",
            "│ Recenter View   [R] Recenter Camera Perspective              │",
            "│ Quit / Exit     [Q / Esc / Ctrl+C] Quit                      │",
            "└─────────── Press [?], [/], [Esc], or Click to Close ─────────┘",
        ];
        let modal_h = lines_text.len().min(lines.saturating_sub(2));
        if modal_w < 30 || modal_h < 6 {
            return;
        }

        let start_x = (cols - modal_w) / 2;
        let start_y = (lines - modal_h) / 2;

        // Draw solid dark background box to occlude background geometry
        for dy in 0..modal_h {
            let cur_y = start_y + dy;
            let v_y0 = match buffer.mode {
                RenderMode::Ascii => cur_y,
                RenderMode::HalfBlock => cur_y * 2,
            };
            let v_y1 = match buffer.mode {
                RenderMode::Ascii => cur_y,
                RenderMode::HalfBlock => cur_y * 2 + 1,
            };
            for x in 0..modal_w {
                let px = start_x + x;
                if px < buffer.cols {
                    buffer.set_pixel(px, v_y0, Pixel::rgb(12, 16, 26));
                    if buffer.mode == RenderMode::HalfBlock {
                        buffer.set_pixel(px, v_y1, Pixel::rgb(12, 16, 26));
                    }
                }
            }
        }

        // Draw framed text
        for (idx, line) in lines_text.iter().enumerate() {
            if idx >= modal_h {
                break;
            }
            let y = start_y + idx;
            let trimmed: String = line.chars().take(modal_w).collect();
            let color = if idx == 0 || idx == lines_text.len() - 1 {
                [100, 180, 255]
            } else {
                [220, 230, 245]
            };
            Self::draw_text(buffer, start_x, y, &trimmed, color);
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
                // In HalfBlock mode, dim the bottom half-block cell to improve text legibility over bright planetary surfaces
                if buffer.mode == RenderMode::HalfBlock && v_y + 1 < buffer.virtual_height() {
                    let bot_idx = (v_y + 1) * buffer.cols + px;
                    buffer.pixels[bot_idx].r /= 4;
                    buffer.pixels[bot_idx].g /= 4;
                    buffer.pixels[bot_idx].b /= 4;
                }
            }
        }
    }
}
