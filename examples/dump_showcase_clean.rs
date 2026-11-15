use solar_system_ascii::physics::body::BodyId;
use solar_system_ascii::physics::SolarSystemPhysics;
use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
use solar_system_ascii::render::camera::Camera;
use solar_system_ascii::render::raycaster::Raycaster;

fn save_clean_ppm(buf: &FrameBuffer, path: &str) {
    let v_h = buf.virtual_height();
    let mut ppm = format!("P6\n{} {}\n255\n", buf.cols, v_h).into_bytes();
    for p in &buf.pixels {
        ppm.push(p.r);
        ppm.push(p.g);
        ppm.push(p.b);
    }
    std::fs::write(path, ppm).unwrap();
}

fn main() {
    let cols = 260;
    let lines = 80; // virtual height 160 pixels
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let mut raycaster = Raycaster::new();

    // 1. Saturn: Majestic oblique perspective with ring shadow & Cassini division
    let mut cam_sat = Camera::new();
    cam_sat.set_target(BodyId::Saturn, physics.saturn.position);
    cam_sat.azimuth = -1.45;
    cam_sat.elevation = 0.22;
    cam_sat.distance_km = 270_000.0; // Comfortable framing of the full 280,000 km ring disc
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_sat, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/saturn_rings_shadows.ppm");

    // 2. Jupiter: Great Red Spot, turbulent cloud bands & limb darkening
    let mut cam_jup = Camera::new();
    cam_jup.set_target(BodyId::Jupiter, physics.jupiter.position);
    cam_jup.azimuth = -1.35;
    cam_jup.elevation = 0.12;
    cam_jup.distance_km = 240_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_jup, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/jupiter_great_red_spot.ppm");

    // 3. Sun: Close-up photosphere granulation, sunspots, and coronal loops
    let mut cam_sun = Camera::new();
    cam_sun.set_target(BodyId::Sun, physics.sun.position);
    cam_sun.azimuth = 0.0;
    cam_sun.elevation = 0.0;
    cam_sun.distance_km = 3_400_000.0; // Clean framing of solar disc (~50% height) with room for corona
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_sun, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/sun_photosphere_granulation.ppm");

    // 4. Mars: Olympus Mons, Valles Marineris & polar ice cap
    let mut cam_mars = Camera::new();
    cam_mars.set_target(BodyId::Mars, physics.mars.position);
    cam_mars.azimuth = -1.18;
    cam_mars.elevation = 0.18;
    cam_mars.distance_km = 12_500.0;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_mars, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/mars_olympus_mons.ppm");

    // 5. Venus Magellan Radar SAR Mode
    let mut cam_venus = Camera::new();
    cam_venus.set_target(BodyId::Venus, physics.venus.position);
    cam_venus.azimuth = -0.80;
    cam_venus.elevation = 0.15;
    cam_venus.distance_km = 22_000.0;
    raycaster.venus_shader.radar_mode = true;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_venus, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/venus_magellan_radar.ppm");

    // 6. Venus Optical Atmosphere Mode
    raycaster.venus_shader.radar_mode = false;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_venus, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/venus_optical_atmosphere.ppm");

    // 7. Uranus: Extreme axial tilt and vertical rings
    let mut cam_uranus = Camera::new();
    cam_uranus.set_target(BodyId::Uranus, physics.uranus.position);
    cam_uranus.azimuth = 1.85;
    cam_uranus.elevation = 0.35;
    cam_uranus.distance_km = 135_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_uranus, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/uranus_vertical_rings.ppm");

    // 8. Pluto & Charon: Tombaugh Regio heart
    let mut cam_pluto = Camera::new();
    cam_pluto.set_target(BodyId::Pluto, physics.pluto.position);
    cam_pluto.azimuth = 0.0;
    cam_pluto.elevation = 0.12;
    cam_pluto.distance_km = 6_500.0;
    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam_pluto, &physics, 1, 0.0);
    save_clean_ppm(&buffer, "assets/screenshots/pluto_tombaugh_regio.ppm");

    println!("All showcase captures generated cleanly without HUD clutter!");
}
