use solar_system_ascii::physics::SolarSystemPhysics;
use solar_system_ascii::render::buffer::{FrameBuffer, RenderMode, Pixel};
use solar_system_ascii::render::camera::Camera;
use solar_system_ascii::render::raycaster::Raycaster;

fn main() {
    let cols = 420;
    let lines = 120;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let raycaster = Raycaster::new();

    let mut cam = Camera::new();
    // 1.95e10 km (~130 AU): perfectly frames Pluto's eccentric aphelion orbit and Neptune within vertical FOV
    cam.distance_km = 1.95e10;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam, &physics, 1, 0.0);

    let v_h = buffer.virtual_height();
    let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
    for p in &buffer.pixels {
        ppm.push(p.r);
        ppm.push(p.g);
        ppm.push(p.b);
    }
    std::fs::write("assets/screenshots/solar_system_orrery_hd.ppm", ppm).unwrap();
    println!("Rendered HD Orrery: {}x{}", cols, v_h);
}
