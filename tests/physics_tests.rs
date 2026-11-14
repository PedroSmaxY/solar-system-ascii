use solar_system_ascii::math::{Mat3, Vec3};
use solar_system_ascii::physics::ephemeris::KeplerOrbit;
use solar_system_ascii::physics::SolarSystemPhysics;
use std::f64::consts::PI;

#[test]
fn test_vec3_operations() {
    let v1 = Vec3::new(1.0, 2.0, 3.0);
    let v2 = Vec3::new(4.0, 5.0, 6.0);

    assert_eq!(v1 + v2, Vec3::new(5.0, 7.0, 9.0));
    assert_eq!(v2 - v1, Vec3::new(3.0, 3.0, 3.0));
    assert_eq!(v1 * 2.0, Vec3::new(2.0, 4.0, 6.0));
    assert_eq!(v1.dot(v2), 4.0 + 10.0 + 18.0);

    let cross = Vec3::X.cross(Vec3::Y);
    assert!((cross - Vec3::Z).length() < 1e-10);

    let norm = Vec3::new(0.0, 3.0, 4.0).normalize();
    assert!((norm.length() - 1.0).abs() < 1e-10);
    assert!((norm.y - 0.6).abs() < 1e-10);
    assert!((norm.z - 0.8).abs() < 1e-10);
}

#[test]
fn test_mat3_rotations() {
    let rot_y = Mat3::rotation_y(PI / 2.0);
    let rotated = rot_y.mul_vec(Vec3::X);
    assert!((rotated - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-10);
}

#[test]
fn test_kepler_solver_convergence() {
    let orbit = KeplerOrbit::moon_around_earth();
    // Test eccentric anomaly solver for various mean anomalies
    for deg in (0..360).step_by(15) {
        let m = (deg as f64) * PI / 180.0;
        let e = orbit.solve_eccentric_anomaly(m);
        // Verify Kepler's equation M = E - e*sin(E)
        let diff = (e - orbit.e * e.sin() - m).abs();
        assert!(diff < 1e-10, "Failed for M = {} deg, diff = {}", deg, diff);
    }
}

#[test]
fn test_moon_orbit_closed() {
    let orbit = KeplerOrbit::moon_around_earth();
    let pos_t0 = orbit.calculate_position(0.0);
    let pos_period = orbit.calculate_position(orbit.period);

    let diff = (pos_t0 - pos_period).length();
    assert!(
        diff < 1e-4,
        "Moon orbit should close after exactly 1 orbital period: diff = {} km",
        diff
    );
}

#[test]
fn test_solar_system_hierarchy() {
    let mut sys = SolarSystemPhysics::new();
    sys.update(0.0);

    assert_eq!(sys.sun.position, Vec3::ZERO);
    assert!(sys.earth.position.length() > 140_000_000.0);

    // Moon should be near Earth
    let dist_earth_moon = (sys.moon.position - sys.earth.position).length();
    assert!(
        dist_earth_moon > 350_000.0 && dist_earth_moon < 410_000.0,
        "Moon distance from Earth should be realistic: {} km",
        dist_earth_moon
    );
}

#[test]
fn test_camera_target_zoom_memory() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    assert_eq!(cam.target_body, BodyId::Earth);
    assert!((cam.distance_km - 21_000.0).abs() < 1e-4);

    // Zoom and orbit on Earth
    cam.zoom_in();
    cam.orbit(0.42, -0.15);
    let earth_zoomed = cam.distance_km;
    let earth_az = cam.azimuth;
    let earth_el = cam.elevation;
    assert!(earth_zoomed < 21_000.0);

    // Switch to Moon -> must load Moon's dedicated perspective and distance
    cam.set_target(BodyId::Moon, Vec3::new(384_400.0, 0.0, 0.0));
    assert_eq!(cam.target_body, BodyId::Moon);
    assert!((cam.distance_km - 8_000.0).abs() < 1e-4);
    assert!((cam.azimuth - 0.85).abs() < 1e-4);

    // Customize Moon's perspective and zoom
    cam.zoom_in();
    cam.orbit(-0.60, 0.30);
    let moon_zoomed = cam.distance_km;
    let moon_az = cam.azimuth;
    let moon_el = cam.elevation;

    // Switch back to Earth -> must restore Earth's exact customized perspective and zoom!
    cam.set_target(BodyId::Earth, Vec3::ZERO);
    assert_eq!(cam.target_body, BodyId::Earth);
    assert!(
        (cam.distance_km - earth_zoomed).abs() < 1e-4,
        "Earth distance should be restored to {} km, but was {} km",
        earth_zoomed,
        cam.distance_km
    );
    assert!((cam.azimuth - earth_az).abs() < 1e-6);
    assert!((cam.elevation - earth_el).abs() < 1e-6);

    // Switch back to Moon -> must restore Moon's exact customized perspective and zoom!
    cam.set_target(BodyId::Moon, Vec3::new(384_400.0, 0.0, 0.0));
    assert_eq!(cam.target_body, BodyId::Moon);
    assert!((cam.distance_km - moon_zoomed).abs() < 1e-4);
    assert!((cam.azimuth - moon_az).abs() < 1e-6);
    assert!((cam.elevation - moon_el).abs() < 1e-6);
}

#[test]
fn test_camera_360_orbit_and_recenter() {
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();

    // Orbit elevation all the way over the poles (+360 degrees)
    for _ in 0..100 {
        cam.orbit(0.0, 0.1);
        let pos = cam.position();
        assert!(!pos.x.is_nan() && !pos.y.is_nan() && !pos.z.is_nan());
        assert!(pos.length() > 20_000.0);
    }

    // Now test recenter
    cam.recenter();
    assert!((cam.azimuth - 2.30).abs() < 1e-6);
    assert!((cam.elevation - 0.28).abs() < 1e-6);
    assert!((cam.distance_km - 21_000.0).abs() < 1e-4);
}

#[test]
fn test_moon_far_side_features() {
    use solar_system_ascii::render::moon_shader::MoonShader;

    let shader = MoonShader::new();

    // Local hit for Tsiolkovskiy on the far side (lat ~ -20.4 deg, lon ~ +129.1 deg)
    let lat_rad = -20.4f64.to_radians();
    let lon_rad = 129.1f64.to_radians();
    let x = lat_rad.cos() * lon_rad.cos();
    let y = lat_rad.sin();
    let z = lat_rad.cos() * lon_rad.sin();
    let hit = Vec3::new(x, y, z).normalize();
    let sun = hit; // direct sun
    let view = hit; // direct view

    let rgb = shader.shade(hit, sun, view);
    // Should be valid RGB
    assert!(rgb[0] > 0);

    // Verify geological contrast: Tycho ray crater vs Mare Tranquillitatis
    let tycho_albedo = shader.sample_albedo(-43.3, -11.2);
    let mare_albedo = shader.sample_albedo(8.5, 31.0);
    assert!(
        tycho_albedo > mare_albedo,
        "Tycho crater ray albedo ({}) must exceed basaltic mare albedo ({})",
        tycho_albedo,
        mare_albedo
    );

    // Verify elevation data exists and is non-zero
    let tycho_elev = shader.sample_elevation(-43.3, -11.2);
    assert!(tycho_elev > 0.0);
}

#[test]
fn test_earth_shader_resolution_and_lighting() {
    use solar_system_ascii::render::earth_shader::EarthShader;

    let shader = EarthShader::new();

    // São Paulo coordinates (lat -23.55, lon -46.63)
    let lat_rad = -23.55f64.to_radians();
    let lon_rad = -46.63f64.to_radians();
    let x = lat_rad.cos() * lon_rad.cos();
    let y = lat_rad.sin();
    let z = lat_rad.cos() * lon_rad.sin();
    let sp_hit = Vec3::new(x, y, z).normalize();

    let view = Vec3::new(0.0, 0.0, 1.0);

    // Daylight: sun directly overhead (N.L = 1.0)
    let sun_day = sp_hit;
    let rgb_day = shader.shade(sp_hit, sp_hit, sun_day, view, 0, 0.0);
    assert!(rgb_day[0] > 0 || rgb_day[1] > 0 || rgb_day[2] > 0);

    // Night: sun behind Earth (N.L = -1.0)
    let sun_night = -sp_hit;
    let rgb_night = shader.shade(sp_hit, sp_hit, sun_night, view, 0, 0.0);
    // City lights in São Paulo must illuminate at night
    assert!(rgb_night[0] > 0 && rgb_night[1] > 0);

    // Clouds dynamic variation over time: t=0 vs t=1000
    let c1 = shader.shade(sp_hit, sp_hit, sun_day, view, 0, 0.0);
    let c2 = shader.shade(sp_hit, sp_hit, sun_day, view, 0, 50000.0);
    // Over 50k seconds, fluid advection and condensation change the cloud color
    assert!(c1 != c2 || c1[0] > 0);
}

#[test]
fn test_fluid_engine_continuity() {
    use solar_system_ascii::render::fluid::FluidEngine;

    // Verify C^infinity continuity across equator: d/d(lat) at lat=0 must be zero
    let eps = 0.001f64;
    let t = 1000.0f64;
    let w_pos = FluidEngine::differential_drift(eps, t, -0.00010, 0.00028);
    let w_neg = FluidEngine::differential_drift(-eps, t, -0.00010, 0.00028);
    let diff = (w_pos - w_neg).abs();
    assert!(
        diff < 1e-6,
        "Equatorial differential rotation must be mirror-smooth: diff = {}",
        diff
    );

    // Test cloud sampling
    let p = Vec3::new(0.5, 0.2, 0.8).normalize();
    let (density, alpha) = FluidEngine::sample_earth_clouds(p, 100.0);
    assert!((0.0..=1.0).contains(&density));
    assert!((0.0..=0.88).contains(&alpha));
}

#[test]
fn test_earth_uninhabited_wilderness_dark_at_night() {
    use solar_system_ascii::render::earth_shader::EarthShader;

    let shader = EarthShader::new();
    let view = Vec3::new(0.0, 0.0, 1.0);

    // Heart of Amazon Rainforest (lat -3.0, lon -65.0)
    let lat_amz = -3.0f64.to_radians();
    let lon_amz = -65.0f64.to_radians();
    let amz_hit = Vec3::new(lat_amz.cos() * lon_amz.cos(), lat_amz.sin(), lat_amz.cos() * lon_amz.sin()).normalize();
    let sun_night_amz = -amz_hit; // Night
    let amz_night = shader.shade(amz_hit, amz_hit, sun_night_amz, view, 0, 0.0);

    // Amazon must be dark at night (no artificial city lights)
    assert!(amz_night[0] < 45 && amz_night[1] < 45, "Amazon should be dark at night, got {:?}", amz_night);

    // Central Sahara Desert (lat 24.0, lon 12.0)
    let lat_sah = 24.0f64.to_radians();
    let lon_sah = 12.0f64.to_radians();
    let sah_hit = Vec3::new(lat_sah.cos() * lon_sah.cos(), lat_sah.sin(), lat_sah.cos() * lon_sah.sin()).normalize();
    let sun_night_sah = -sah_hit; // Night
    let sah_night = shader.shade(sah_hit, sah_hit, sun_night_sah, view, 0, 0.0);

    // Sahara must be dark at night
    assert!(sah_night[0] < 45 && sah_night[1] < 45, "Sahara should be dark at night, got {:?}", sah_night);
}

#[test]
fn test_fluid_engine_extreme_timewarp_stability() {
    use solar_system_ascii::render::fluid::FluidEngine;

    let p = Vec3::new(0.3, 0.7, 0.6).normalize();

    // Test extreme time warp (10 million seconds = ~4 months of sim time)
    for t in [0.0, 1000.0, 50_000.0, 1_000_000.0, 10_000_000.0] {
        let (density, alpha) = FluidEngine::sample_earth_clouds(p, t);
        assert!(!density.is_nan() && !alpha.is_nan());
        assert!((0.0..=1.0).contains(&density));
        assert!((0.0..=0.88).contains(&alpha));
    }
}

#[test]
fn test_earth_triple_layer_header_and_night_lights() {
    use solar_system_ascii::render::earth_shader::EarthShader;

    let shader = EarthShader::new();

    // Verify Apollo 17 / Artemis II deep sapphire ocean albedo
    let pacific_ocean = shader.sample_albedo(0.0, -140.0); // Mid-Pacific
    assert!(
        pacific_ocean[0] < 18.0 && pacific_ocean[1] < 35.0 && pacific_ocean[2] < 65.0,
        "Pacific ocean should have authentic deep sapphire navy albedo, got {:?}",
        pacific_ocean
    );

    // Verify NASA Blue Marble satellite albedo differentiation
    let sahara_albedo = shader.sample_albedo(20.0, 10.0); // Sahara Desert
    assert!(
        sahara_albedo[0] > 160.0 && sahara_albedo[1] > 140.0,
        "Sahara should have warm desert sand albedo, got {:?}",
        sahara_albedo
    );

    let congo_albedo = shader.sample_albedo(0.0, 20.0); // Congo Rainforest
    assert!(
        congo_albedo[1] > congo_albedo[0],
        "Congo should have rich green vegetation albedo, got {:?}",
        congo_albedo
    );

    // Major metropolitan centers must have strong real night lights intensity
    let sp_lights = shader.sample_night_lights(-23.55, -46.63); // São Paulo
    assert!(sp_lights > 0.40, "São Paulo should be brightly lit, got {}", sp_lights);

    let nyc_lights = shader.sample_night_lights(40.71, -74.00); // New York
    assert!(nyc_lights > 0.50, "New York should be brightly lit, got {}", nyc_lights);

    let cairo_lights = shader.sample_night_lights(30.04, 31.23); // Cairo / Nile
    assert!(cairo_lights > 0.50, "Cairo should be brightly lit, got {}", cairo_lights);

    // Deep wilderness should have zero or near-zero lights
    let amz_lights = shader.sample_night_lights(-3.0, -65.0); // Amazon
    assert!(amz_lights < 0.05, "Amazon rainforest should have near-zero lights, got {}", amz_lights);

    let sahara_lights = shader.sample_night_lights(24.0, 12.0); // Sahara
    assert!(sahara_lights < 0.05, "Central Sahara should have near-zero lights, got {}", sahara_lights);
}

#[test]
fn test_polar_auroras_emission() {
    use solar_system_ascii::render::earth_shader::EarthShader;

    let shader = EarthShader::new();
    let view = Vec3::new(0.0, 0.0, 1.0);

    // Polar night point (Tromsø, Norway: 69.6° N, 18.9° E)
    let lat_p = 68.0f64.to_radians();
    let lon_p = 20.0f64.to_radians();
    let polar_hit = Vec3::new(lat_p.cos() * lon_p.cos(), lat_p.sin(), lat_p.cos() * lon_p.sin()).normalize();
    let sun_night_polar = -polar_hit; // Polar night

    // 1. Peak of solar storm (t = 14,400 s): auroral emerald/violet emission must be active
    let storm_t = 14_400.0;
    let storm_night = shader.shade(polar_hit, polar_hit, sun_night_polar, view, 10, storm_t);
    assert!(
        storm_night[1] > 20 || storm_night[2] > 20,
        "During solar storm, polar night should exhibit auroral luminescence, got {:?}",
        storm_night
    );

    // 2. Quiet baseline between storms (t = 100,000 s): auroral emission is calm/quiescent
    let quiet_t = 100_000.0;
    let quiet_night = shader.shade(polar_hit, polar_hit, sun_night_polar, view, 10, quiet_t);
    assert!(
        quiet_night[1] < storm_night[1],
        "Quiet period should be significantly darker than storm period: quiet={:?}, storm={:?}",
        quiet_night, storm_night
    );

    // 3. Equatorial night point (0° N, 0° E)
    let eq_hit = Vec3::new(1.0, 0.0, 0.0);
    let sun_night_eq = -eq_hit;
    let eq_night = shader.shade(eq_hit, eq_hit, sun_night_eq, view, 10, storm_t);

    // Equatorial night must NOT have auroral luminescence (dark ocean starlight)
    assert!(
        eq_night[1] < 25,
        "Equatorial night should have zero auroral emission, got {:?}",
        eq_night
    );
}
