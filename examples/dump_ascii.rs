use solar_system_ascii::physics::body::BodyId;
use solar_system_ascii::physics::SolarSystemPhysics;
use solar_system_ascii::render::buffer::{FrameBuffer, RenderMode, Pixel};
use solar_system_ascii::render::camera::Camera;
use solar_system_ascii::render::raycaster::Raycaster;

fn main() {
    let cols = 76;
    let lines = 24;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::Ascii);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let raycaster = Raycaster::new();

    // 1. Saturn
    let mut cam_sat = Camera::new();
    cam_sat.set_target(BodyId::Saturn, physics.saturn.position);
    cam_sat.azimuth = -1.25;
    cam_sat.elevation = 0.35;
    cam_sat.distance_km = 330_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_sat, &physics, 1, 0.0);

    let mut sat_txt = String::new();
    for y in 0..lines {
        for x in 0..cols {
            sat_txt.push(buffer.pixels[y * cols + x].ch);
        }
        sat_txt.push('\n');
    }
    std::fs::write("assets/screenshots/saturn_ascii.txt", sat_txt).unwrap();

    // 2. Earth
    let mut cam_earth = Camera::new();
    cam_earth.set_target(BodyId::Earth, physics.earth.position);
    cam_earth.azimuth = 0.6;
    cam_earth.elevation = 0.2;
    cam_earth.distance_km = 22_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_earth, &physics, 1, 0.0);

    let mut earth_txt = String::new();
    for y in 0..lines {
        for x in 0..cols {
            earth_txt.push(buffer.pixels[y * cols + x].ch);
        }
        earth_txt.push('\n');
    }
    std::fs::write("assets/screenshots/earth_ascii.txt", earth_txt).unwrap();
    println!("Generated ASCII captures successfully!");
}
