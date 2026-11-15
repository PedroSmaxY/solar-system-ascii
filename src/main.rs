//! AstroEngine 3D - High-Precision ASCII / TrueColor Solar System Simulator.

use solar_system_ascii::app::App;
use solar_system_ascii::render::buffer::RenderMode;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut test_frames = None;
    let mut initial_mode = RenderMode::Ascii;
    let mut target_body = None;
    let mut custom_azimuth = None;
    let mut custom_elevation = None;
    let mut custom_distance = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--hd" | "--halfblock" => {
                initial_mode = RenderMode::HalfBlock;
            }
            "--ascii" => {
                initial_mode = RenderMode::Ascii;
            }
            "--target" => {
                if i + 1 < args.len() {
                    match args[i + 1].to_lowercase().as_str() {
                        "sun" | "sol" => target_body = Some(solar_system_ascii::physics::body::BodyId::Sun),
                        "mercury" | "mercurio" => target_body = Some(solar_system_ascii::physics::body::BodyId::Mercury),
                        "venus" => target_body = Some(solar_system_ascii::physics::body::BodyId::Venus),
                        "earth" | "terra" => target_body = Some(solar_system_ascii::physics::body::BodyId::Earth),
                        "moon" | "luna" | "lua" => target_body = Some(solar_system_ascii::physics::body::BodyId::Moon),
                        "mars" | "marte" => target_body = Some(solar_system_ascii::physics::body::BodyId::Mars),
                        "phobos" | "fobos" => target_body = Some(solar_system_ascii::physics::body::BodyId::Phobos),
                        "deimos" => target_body = Some(solar_system_ascii::physics::body::BodyId::Deimos),
                        _ => {}
                    }
                    i += 1;
                }
            }
            "--azimuth" => {
                if i + 1 < args.len() {
                    custom_azimuth = args[i + 1].parse::<f64>().ok();
                    i += 1;
                }
            }
            "--elevation" => {
                if i + 1 < args.len() {
                    custom_elevation = args[i + 1].parse::<f64>().ok();
                    i += 1;
                }
            }
            "--distance" => {
                if i + 1 < args.len() {
                    custom_distance = args[i + 1].parse::<f64>().ok();
                    i += 1;
                }
            }
            "--frames" => {
                if i + 1 < args.len() {
                    test_frames = args[i + 1].parse::<u64>().ok();
                    i += 1;
                }
            }
            "--help" | "-h" => {
                println!("AstroEngine 3D - Real-time 3D Solar System & Earth Simulator");
                println!();
                println!("Usage: solar-system-ascii [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --hd, --halfblock   Start in High-Definition Half-Block mode ('▀')");
                println!("  --ascii             Start in standard ASCII mode (default)");
                println!("  --frames <N>        Run for N frames and exit (benchmark mode)");
                println!("  -h, --help          Print help information");
                println!();
                println!("Controls:");
                println!("  [+/-]          Zoom In / Zoom Out (continuous exponential)");
                println!("  [WASD/Arrows]  Orbit camera around targeted planet");
                println!("  [1]            Focus camera on Sun");
                println!("  [2]            Focus camera on Mercury");
                println!("  [3]            Focus camera on Venus");
                println!("  [4]            Focus camera on Earth");
                println!("  [5]            Focus camera on Mars");
                println!("  [M]            Contextual Moon toggle (Earth <-> Moon, or Mars -> Phobos)");
                println!("  [N]            Contextual Moon toggle (Mars/Phobos <-> Deimos)");
                println!("  [V]            Toggle Venus Optical Cloud Tops <-> Magellan SAR Radar");
                println!("  [H]            Toggle HUD astronomical overlay");
                println!("  [L]            Toggle planet name labels");
                println!("  [B]            Toggle mode: ASCII (Standard) <-> Half-Block (HD)");
                println!("  [Space]        Pause / Resume simulation");
                println!("  [[/]]          Decrease / Increase time warp speed");
                println!("  [Q/Esc]        Quit");
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let mut app = App::new(initial_mode);
    if let Some(target) = target_body {
        app.set_target(target);
    }
    if custom_azimuth.is_some() || custom_elevation.is_some() || custom_distance.is_some() {
        let az = custom_azimuth.unwrap_or(2.30);
        let el = custom_elevation.unwrap_or(0.28);
        app.set_view(az, el, custom_distance);
    }
    app.run(test_frames);
}
