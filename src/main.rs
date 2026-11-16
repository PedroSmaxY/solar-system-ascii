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
                        "pluto" | "plutao" => target_body = Some(solar_system_ascii::physics::body::BodyId::Pluto),
                        "charon" | "caronte" => target_body = Some(solar_system_ascii::physics::body::BodyId::Charon),
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
                println!("AstroEngine 3D - High-Precision Solar System Simulator in Pure Rust");
                println!();
                println!("Usage: solar-system-ascii [OPTIONS]");
                println!();
                println!("Options:");
                println!("  --hd, --halfblock   Start in High-Definition Half-Block mode ('▀')");
                println!("  --ascii             Start in standard ASCII mode (default)");
                println!("  --target <BODY>     Start camera targeted on celestial body (e.g. earth, mars, jupiter)");
                println!("  --frames <N>        Run for N frames and exit (benchmark mode)");
                println!("  -h, --help          Print help information");
                println!();
                println!("Navigation & Camera Controls:");
                println!("  [Mouse Drag]        Left-click drag to orbit camera around target");
                println!("  [Mouse Scroll]      Smooth exponential zoom in / out");
                println!("  [WASD / Arrows]     Orbit camera (polar clamped [-89°, +89°])");
                println!("  [+/-]               Zoom in / Zoom out");
                println!("  [0 / Z]             Solar System Overview <-> Zoom into Sun photosphere");
                println!("  [Tab / Shift+Tab]   Cycle sequentially through celestial worlds (Sun..Pluto)");
                println!("  [Backspace]         Ascend hierarchy (Moon -> Planet -> Solar System)");
                println!("  [1-9]               Direct jump: 1:Mercury, 2:Venus, 3:Earth, 4:Mars, 5:Jupiter, 6:Saturn, 7:Uranus, 8:Neptune, 9:Pluto");
                println!("  [P]                 Jump to Ceres / Vesta (Main Asteroid Belt)");
                println!("  [M]                 Cycle natural moons with loop returning to parent planet");
                println!("  [N]                 System overview view framing all planetary moon orbits");
                println!("  [F]                 Align camera to frame the Sun directly in view");
                println!("  [R]                 Recenter camera to curated optimal beauty perspective");
                println!("  [V]                 Toggle Venus Optical Clouds <-> Magellan SAR Radar");
                println!("  [L]                 Toggle elliptical Keplerian orbit paths");
                println!("  [B]                 Toggle render mode: ASCII <-> TrueColor HD Half-Block");
                println!("  [H]                 Toggle HUD telemetry overlay");
                println!("  [?] / [F1]          Toggle interactive on-screen Help Modal");
                println!("  [Space]             Pause / Resume simulation");
                println!("  [T]                 Cycle time warp (10 min/s, 1 day/s, 1 month/s, 1 year/s..)");
                println!("  [\\ / |]             Reset time warp to default 10 min/s");
                println!("  [[ / ]]             Fine decrease / increase time warp");
                println!("  [Q / Esc]           Quit simulator (or dismiss help modal)");
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
