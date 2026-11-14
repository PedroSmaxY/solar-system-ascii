//! AstroEngine 3D - High-Precision ASCII / TrueColor Solar System Simulator.

use solar_system_ascii::app::App;
use solar_system_ascii::render::buffer::RenderMode;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut test_frames = None;
    let mut initial_mode = RenderMode::Ascii;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--hd" | "--halfblock" => {
                initial_mode = RenderMode::HalfBlock;
            }
            "--ascii" => {
                initial_mode = RenderMode::Ascii;
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
                println!("  [1]            Focus camera on Earth");
                println!("  [2]            Focus camera on Moon");
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
    app.run(test_frames);
}
