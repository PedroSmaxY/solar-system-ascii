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
                        "ceres" => target_body = Some(solar_system_ascii::physics::body::BodyId::Ceres),
                        "vesta" => target_body = Some(solar_system_ascii::physics::body::BodyId::Vesta),
                        "jupiter" | "jupiter_system" => target_body = Some(solar_system_ascii::physics::body::BodyId::Jupiter),
                        "io" => target_body = Some(solar_system_ascii::physics::body::BodyId::Io),
                        "europa" => target_body = Some(solar_system_ascii::physics::body::BodyId::Europa),
                        "ganymede" | "ganimedes" => target_body = Some(solar_system_ascii::physics::body::BodyId::Ganymede),
                        "callisto" | "calisto" => target_body = Some(solar_system_ascii::physics::body::BodyId::Callisto),
                        "saturn" | "saturno" => target_body = Some(solar_system_ascii::physics::body::BodyId::Saturn),
                        "titan" | "tita" => target_body = Some(solar_system_ascii::physics::body::BodyId::Titan),
                        "enceladus" | "encelado" => target_body = Some(solar_system_ascii::physics::body::BodyId::Enceladus),
                        "mimas" => target_body = Some(solar_system_ascii::physics::body::BodyId::Mimas),
                        "iapetus" | "japeto" => target_body = Some(solar_system_ascii::physics::body::BodyId::Iapetus),
                        "rhea" | "reia" => target_body = Some(solar_system_ascii::physics::body::BodyId::Rhea),
                        "uranus" | "urano" => target_body = Some(solar_system_ascii::physics::body::BodyId::Uranus),
                        "miranda" => target_body = Some(solar_system_ascii::physics::body::BodyId::Miranda),
                        "ariel" => target_body = Some(solar_system_ascii::physics::body::BodyId::Ariel),
                        "umbriel" => target_body = Some(solar_system_ascii::physics::body::BodyId::Umbriel),
                        "titania" => target_body = Some(solar_system_ascii::physics::body::BodyId::Titania),
                        "oberon" => target_body = Some(solar_system_ascii::physics::body::BodyId::Oberon),
                        "neptune" | "netuno" => target_body = Some(solar_system_ascii::physics::body::BodyId::Neptune),
                        "triton" | "tritao" => target_body = Some(solar_system_ascii::physics::body::BodyId::Triton),
                        "proteus" | "proteu" => target_body = Some(solar_system_ascii::physics::body::BodyId::Proteus),
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
                println!("  [0/Tab]        Solar System Overview <-> Zoom into Sun (or jump back to planet)");
                println!("  [1-8]          Jump to planet: Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus, Neptune");
                println!("  [9]            Jump to Pluto");
                println!("  [M]            Cycle natural moons of currently focused celestial body");
                println!("  [V]            Toggle Venus Optical Clouds <-> Magellan SAR Radar");
                println!("  [T]            Quick-cycle astronomical time warp presets (min/s, day/s, month/s, year/s)");
                println!("  [[/]]          Fine decrease / increase time warp speed (up to 10 years/sec)");
                println!("  [+/-]          Zoom In / Zoom Out (continuous exponential zoom)");
                println!("  [WASD/Arrows]  Orbit camera around targeted planet or sun");
                println!("  [R]            Reset camera to curated optimal beauty perspective");
                println!("  [L]            Toggle elliptical planetary orbit path lines");
                println!("  [P]            Toggle Main Asteroid Belt particle cloud");
                println!("  [B]            Toggle render mode: ASCII <-> Half-Block TrueColor (HD)");
                println!("  [H]            Toggle HUD telemetry overlay");
                println!("  [Space]        Pause / Resume simulation");
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
