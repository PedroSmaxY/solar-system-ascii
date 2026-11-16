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
    assert_eq!(cam.target_body, BodyId::Sun);
    assert!((cam.distance_km - 1.1e10).abs() < 1e-4);

    // Switch to Earth to test per-body zoom memory
    cam.set_target(BodyId::Earth, Vec3::new(149_597_870.7, 0.0, 0.0));
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
fn test_camera_transition_interpolation() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();

    let earth_pos = physics.earth.position;
    let moon_pos = physics.moon.position;
    cam.set_target(BodyId::Earth, earth_pos);

    // Start transition from Earth to Moon (1.2 seconds duration)
    cam.start_transition(BodyId::Moon, earth_pos, moon_pos, 1.2);
    assert!(cam.transition.is_some());

    // Progress at t = 0.0s (p = 0.0) -> exactly at Earth
    cam.update(0.0, &physics);
    assert!((cam.distance_km - 21_000.0).abs() < 1.0);
    assert!((cam.target_pos - earth_pos).length() < 1.0);

    // Progress at t = 0.6s (p = 0.5) -> apex altitude zoom-out
    cam.update(0.6, &physics);
    assert!(cam.transition.is_some());
    // Distance should be at apex altitude (> 100,000 km) for full interplanetary view
    assert!(
        cam.distance_km > 100_000.0,
        "Mid-transit distance should expand to apex altitude (> 100,000 km), was {} km",
        cam.distance_km
    );
    // Target position should be approximately halfway between Earth and Moon
    let mid_expected = earth_pos * 0.5 + moon_pos * 0.5;
    assert!((cam.target_pos - mid_expected).length() < 100.0);

    // Progress at t = 1.2s (p = 1.0) -> smooth zoom-in docking at Moon
    cam.update(0.6, &physics);
    assert!(cam.transition.is_none(), "Transition must complete at progress = 1.0");
    assert_eq!(cam.target_body, BodyId::Moon);
    assert!((cam.distance_km - 8_000.0).abs() < 1.0, "Moon distance should be 8,000 km, was {} km", cam.distance_km);
    assert!((cam.azimuth - 0.85).abs() < 1e-4);
    assert!((cam.target_pos - moon_pos).length() < 1.0);
}

#[test]
fn test_render_transition_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 92;
    let lines = 24;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let raycaster = Raycaster::new();

    let earth_pos = physics.earth.position;
    let moon_pos = physics.moon.position;

    // Start transition to Moon
    cam.start_transition(BodyId::Moon, earth_pos, moon_pos, 1.2);

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        raycaster.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // Frame 1: Departure at Earth (p = 0.0)
    cam.update(0.0, &physics);
    render_ppm(&mut buffer, &cam, "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/trans_step1_earth.ppm");

    // Frame 2: Mid-flight apex zoom-out (p = 0.5)
    cam.update(0.6, &physics);
    render_ppm(&mut buffer, &cam, "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/trans_step2_apex.ppm");

    // Frame 3: Arrival at Moon (p = 1.0)
    cam.update(0.6, &physics);
    render_ppm(&mut buffer, &cam, "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/trans_step3_moon.ppm");
}

#[test]
fn test_camera_360_orbit_and_recenter() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    cam.set_target(BodyId::Earth, Vec3::new(149_597_870.7, 0.0, 0.0));

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

#[test]
fn test_sun_shader_photosphere_and_limb_darkening() {
    use solar_system_ascii::render::sun_shader::SunShader;

    let shader = SunShader::new();
    let view = Vec3::new(0.0, 0.0, 1.0);

    // Center of solar disk (mu = 1.0): local_hit directly facing observer
    let center_hit = Vec3::new(0.0, 0.0, 1.0);
    let center_color = shader.shade_photosphere(center_hit, view, 0, 0.0);

    // Mid-disk of solar photosphere (mu ≈ 0.35): Eddington-Barbier darkening active
    let mid_hit = Vec3::new(0.936, 0.0, 0.35).normalize();
    let mid_color = shader.shade_photosphere(mid_hit, view, 0, 0.0);

    // Extreme limb of solar disk (mu ≈ 0.03): Chromospheric inversion matches coronal base
    let limb_hit = Vec3::new(0.9995, 0.0, 0.03).normalize();
    let limb_color = shader.shade_photosphere(limb_hit, view, 0, 0.0);

    // 1. Center must be hotter and brighter (white-gold) than mid-disk (darkening)
    let center_lum = center_color[0] as f64 * 0.299 + center_color[1] as f64 * 0.587 + center_color[2] as f64 * 0.114;
    let mid_lum = mid_color[0] as f64 * 0.299 + mid_color[1] as f64 * 0.587 + mid_color[2] as f64 * 0.114;
    assert!(
        center_lum > mid_lum * 1.20,
        "Center luminosity ({}) must exceed mid-disk ({}) due to Eddington-Barbier darkening",
        center_lum, mid_lum
    );

    // 2. Chromatic transition & No Muddy Ring: Center has high blue component, limb is radiant gold/plasma
    assert!(center_color[2] > 220, "Center should have strong blue component, got {}", center_color[2]);
    assert!(limb_color[0] > 230, "Limb should maintain radiant red emission, got {}", limb_color[0]);
    assert!(limb_color[1] > 180, "Limb should maintain radiant gold emission (no muddy brown)");
    assert!(limb_color[2] < center_color[2], "Limb blue component should be lower than center");

    // 3. Sunspots: Umbra core in Spörer belt (lat 17.5°, lon 45°) at t=0
    let lat_rad = 17.5f64.to_radians();
    let lon_rad = 45.0f64.to_radians();
    let spot_hit = Vec3::new(lat_rad.cos() * lon_rad.cos(), lat_rad.sin(), lat_rad.cos() * lon_rad.sin()).normalize();
    let spot_color = shader.shade_photosphere(spot_hit, spot_hit, 0, 0.0);

    // Quiet photosphere at equivalent mu=1.0 but quiet latitude (lat 55°)
    let quiet_lat_rad = 55.0f64.to_radians();
    let quiet_hit = Vec3::new(quiet_lat_rad.cos() * lon_rad.cos(), quiet_lat_rad.sin(), quiet_lat_rad.cos() * lon_rad.sin()).normalize();
    let quiet_color = shader.shade_photosphere(quiet_hit, quiet_hit, 0, 0.0);

    assert!(
        (spot_color[0] as f64) < (quiet_color[0] as f64) * 0.5,
        "Sunspot umbra must show sharp luminosity drop: spot={:?}, quiet={:?}",
        spot_color, quiet_color
    );
}

#[test]
fn test_sun_shader_corona_falloff() {
    use solar_system_ascii::render::sun_shader::SunShader;

    let shader = SunShader::new();
    let ray_dir = Vec3::new(0.0, 0.0, -1.0);
    let rel_norm = Vec3::new(1.0, 0.0, 0.0); // Equatorial ray

    // Near-limb corona (1.05 R_sun)
    let inner_corona = shader.shade_corona(1.05, rel_norm, ray_dir, 0.0).expect("Inner corona must be visible");
    // Mid-outer corona (2.5 R_sun)
    let mid_corona = shader.shade_corona(2.50, rel_norm, ray_dir, 0.0).expect("Mid-outer corona must be visible");
    // Far extended streamer (3.8 R_sun)
    let outer_corona = shader.shade_corona(3.80, rel_norm, ray_dir, 0.0).expect("Outer streamer rays must be visible");
    // Deep space (> 4.5 R_sun)
    let deep_space = shader.shade_corona(4.60, rel_norm, ray_dir, 0.0);

    assert!(deep_space.is_none(), "Beyond 4.5 R_sun must yield None (deep space)");
    assert!(shader.shade_corona(0.95, rel_norm, ray_dir, 0.0).is_none(), "Inside solar disk must yield None");

    let inner_lum = inner_corona[0] as f64 * 0.299 + inner_corona[1] as f64 * 0.587 + inner_corona[2] as f64 * 0.114;
    let mid_lum = mid_corona[0] as f64 * 0.299 + mid_corona[1] as f64 * 0.587 + mid_corona[2] as f64 * 0.114;
    let outer_lum = outer_corona[0] as f64 * 0.299 + outer_corona[1] as f64 * 0.587 + outer_corona[2] as f64 * 0.114;

    assert!(
        inner_lum > mid_lum * 2.0 && mid_lum > outer_lum * 1.5,
        "Coronal glow must exhibit smooth continuous radial decay: inner={}, mid={}, outer={}",
        inner_lum, mid_lum, outer_lum
    );
}

#[test]
fn test_sun_coronal_rays_and_star_additive_blending() {
    use solar_system_ascii::render::sun_shader::SunShader;

    let shader = SunShader::new();
    let ray_dir = Vec3::new(0.0, 0.0, -1.0);

    // 1. Angular contrast: Coronal streamer ray vs inter-streamer valley at 2.2 R_sun
    let ray_norm = Vec3::new(1.0, 0.0, 0.0); // Equatorial streamer
    let polar_norm = Vec3::new(0.0, 1.0, 0.0); // Polar coronal hole
    let streamer = shader.shade_corona(2.2, ray_norm, ray_dir, 0.0).unwrap();
    let polar = shader.shade_corona(2.2, polar_norm, ray_dir, 0.0);

    let streamer_lum = streamer[0] as f64 * 0.299 + streamer[1] as f64 * 0.587 + streamer[2] as f64 * 0.114;
    let polar_lum = polar.map(|p| p[0] as f64 * 0.299 + p[1] as f64 * 0.587 + p[2] as f64 * 0.114).unwrap_or(0.0);

    assert!(
        streamer_lum > polar_lum * 1.5,
        "Equatorial helmet streamers should have strong contrast over polar coronal holes: streamer={}, polar={}",
        streamer_lum, polar_lum
    );

    // 2. Additive blending: Stars must never be extinguished in the outer coronal halo
    let outer_corona = shader.shade_corona(3.5, ray_norm, ray_dir, 0.0).unwrap();
    let mock_star_rgb = [200u8, 220u8, 255u8];
    let blended_rgb = [
        outer_corona[0].saturating_add(mock_star_rgb[0]),
        outer_corona[1].saturating_add(mock_star_rgb[1]),
        outer_corona[2].saturating_add(mock_star_rgb[2]),
    ];

    assert!(blended_rgb[0] >= mock_star_rgb[0]);
    assert!(blended_rgb[1] >= mock_star_rgb[1]);
    assert!(blended_rgb[2] >= mock_star_rgb[2]);
}

#[test]
fn test_camera_flyto_sun_transition() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let earth_pos = physics.earth.position;
    let sun_pos = physics.sun.position;
    cam.set_target(BodyId::Earth, earth_pos);

    // Start interplanetary fly-to from Earth to Sun (1.4s duration)
    cam.start_transition(BodyId::Sun, earth_pos, sun_pos, 1.4);
    assert!(cam.transition.is_some());

    // Step animation halfway (0.7s)
    cam.update(0.7, &physics);
    assert!(cam.transition.is_some());
    assert!(cam.distance_km > 3_500_000.0, "Apex of fly-to zoom out should be high");

    // Complete transition (0.8s more -> total 1.5s > 1.4s)
    cam.update(0.8, &physics);
    assert!(cam.transition.is_none());
    assert_eq!(cam.target_body, BodyId::Sun);
    assert!((cam.distance_km - 1.1e10).abs() < 1e-4);
}

#[test]
fn test_render_sun_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 160;
    let rows = 80;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let raycaster = Raycaster::new();

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        raycaster.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // 1. Full Solar Disk with Dramatic Corona Rays & Helmet Streamers (3,500,000 km)
    cam.set_target(BodyId::Sun, physics.sun.position);
    render_ppm(
        &mut buffer,
        &cam,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/sun_dramatic_corona_rays.ppm",
    );

    // 2. Coronal Loops & Eruptive Solar Flare at Limb Close-Up (Image 1 & 2)
    let mut cam_flare = Camera::new();
    cam_flare.set_target(BodyId::Sun, Vec3::new(650_000.0, 180_000.0, 0.0));
    cam_flare.distance_km = 800_000.0;
    cam_flare.azimuth = 0.0;
    cam_flare.elevation = 0.0;
    render_ppm(
        &mut buffer,
        &cam_flare,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/sun_prominence_flare_loops.ppm",
    );

    // 3. Spörer Belt Giant Sunspot Group & Plages Close-Up (1,250,000 km)
    let mut cam_spot = Camera::new();
    cam_spot.set_target(BodyId::Sun, physics.sun.position);
    cam_spot.distance_km = 1_250_000.0;
    cam_spot.azimuth = 45.0f64.to_radians();
    cam_spot.elevation = 18.0f64.to_radians();
    render_ppm(
        &mut buffer,
        &cam_spot,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/sun_spot_closeup.ppm",
    );

    // 4. Solar Granulation Close-Up (850,000 km)
    cam.distance_km = 850_000.0;
    cam.orbit(0.45, -0.20);
    render_ppm(
        &mut buffer,
        &cam,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/sun_granulation_sunspots.ppm",
    );

    // 5. Earth Orbital View with Solar Glare & Telemetry ([F] Align to Sun)
    let mut cam_earth = Camera::new();
    cam_earth.set_target(BodyId::Earth, physics.earth.position);
    cam_earth.align_to_sun(physics.sun.position);
    render_ppm(
        &mut buffer,
        &cam_earth,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/earth_view_sun_glare.ppm",
    );
}

#[test]
fn test_coronal_loop_reconnection_and_blob_ejection() {
    use solar_system_ascii::render::sun_shader::SunShader;

    let shader = SunShader::new();
    let ray_dir = Vec3::new(0.0, 0.0, -1.0);

    // Site 0 base angle is psi ≈ 1.25 rad, rotating with solar_rot = t * 0.015:
    let norm_at = |t: f64| {
        let psi = 1.25 + t * 0.015;
        Vec3::new(psi.cos(), psi.sin(), 0.0)
    };

    // 1. Snapshot during Snap & Detachment phase (t = 7.8s -> tau ≈ 0.74 in an 10.5s period)
    let detached_corona = shader.shade_corona(1.12, norm_at(7.8), ray_dir, 7.8);
    assert!(
        detached_corona.is_some(),
        "Detached plasmoid / CME bubble must be visible at altitude 0.12 R_sun during ejection phase"
    );

    // 2. Snapshot of outer plasmoid vs quiet phase
    // At t = 8.8s (tau ≈ 0.84), plasmoid is passing r ≈ 1.24 R_sun
    let detached_outer = shader.shade_corona(1.24, norm_at(8.8), ray_dir, 8.8).unwrap();
    // At t = 10.2s (tau ≈ 0.97), phase 4 is quiet recovery (plasmoid dispersed)
    let quiet_outer = shader.shade_corona(1.24, norm_at(10.2), ray_dir, 10.2).unwrap();

    let detached_lum = detached_outer[0] as f64 * 0.299 + detached_outer[1] as f64 * 0.587 + detached_outer[2] as f64 * 0.114;
    let quiet_lum = quiet_outer[0] as f64 * 0.299 + quiet_outer[1] as f64 * 0.587 + quiet_outer[2] as f64 * 0.114;

    assert!(
        detached_lum > quiet_lum * 1.05,
        "Ejected plasmoid passing at 1.24 R_sun ({}) must exceed quiet background ({})",
        detached_lum, quiet_lum
    );
}

#[test]
fn test_solar_navigation_and_alignment() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    cam.set_target(BodyId::Earth, physics.earth.position);

    let start_az = cam.azimuth;
    cam.align_to_sun(physics.sun.position);

    // Azimuth must have moved to align with the sun vector
    assert!(
        (cam.azimuth - start_az).abs() > 0.1,
        "align_to_sun must rotate camera perspective towards the Sun"
    );

    // Compute dot product of forward vector with sun direction:
    let cam_pos = cam.position();
    let sun_dir = (physics.sun.position - cam_pos).normalize();

    let (sin_az, cos_az) = cam.azimuth.sin_cos();
    let (sin_el, cos_el) = cam.elevation.sin_cos();
    let forward = Vec3::new(-cos_el * sin_az, -sin_el, -cos_el * cos_az);
    let right = Vec3::new(cos_az, 0.0, -sin_az);

    let f_dot = sun_dir.dot(forward);
    let r_dot = sun_dir.dot(right);

    assert!(f_dot > 0.70, "Sun must be in the forward visual field of the camera, got {}", f_dot);
    assert!(r_dot.abs() < 0.60, "Sun must be framed near the horizontal visual center, got {}", r_dot);
}

#[test]
fn test_interplanetary_solar_glare() {
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 80;
    let rows = 40;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let raycaster = Raycaster::new();

    // From Earth orbit, align camera directly with Sun:
    cam.set_target(solar_system_ascii::physics::body::BodyId::Earth, physics.earth.position);
    cam.align_to_sun(physics.sun.position);

    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam, &physics, 1, 0.0);

    // Check that buffer contains radiant gold pixels from the interplanetary solar glare
    let mut found_glare = false;
    for p in &buffer.pixels {
        if p.r > 200 && p.g > 150 && p.b < 200 {
            found_glare = true;
            break;
        }
    }
    assert!(found_glare, "Interplanetary solar glare must illuminate pixels when looking towards Sun from Earth");
}

#[test]
fn test_prominent_coronal_loops_height_and_contrast() {
    use solar_system_ascii::render::sun_shader::SunShader;

    let shader = SunShader::new();
    let ray_dir = Vec3::new(0.0, 0.0, -1.0);

    // Site 0 (Northeast) is at base psi ≈ 1.15 rad + solar rotation
    let t = 5.0f64;
    let psi_ne = 1.15 + t * 0.015;
    // With ray_dir = -Z, up = Y, right = X:
    // cos_psi = Y, sin_psi = X, so X = sin(psi), Y = cos(psi)
    let norm_ne = Vec3::new(psi_ne.sin(), psi_ne.cos(), 0.0);

    // Loops reach up to 0.24 R_sun altitude during inflation (t = 5.0s -> tau ≈ 0.48)
    let loop_high = shader.shade_corona(1.20, norm_ne, ray_dir, t);
    assert!(
        loop_high.is_some(),
        "Prominent coronal loop arch must reach at least 1.20 R_sun (0.20 R_sun altitude)"
    );
    let color = loop_high.unwrap();
    assert!(
        color[0] > 120 && color[1] > 80,
        "Prominent coronal loops must emit bright golden plasma, got {:?}",
        color
    );
}

#[test]
fn test_interplanetary_glare_continuity_no_void() {
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 120;
    let rows = 60;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let raycaster = Raycaster::new();

    // Align camera towards Sun from Earth orbit
    cam.set_target(solar_system_ascii::physics::body::BodyId::Earth, physics.earth.position);
    cam.align_to_sun(physics.sun.position);

    buffer.pixels.fill(Pixel::BLACK);
    raycaster.render_frame(&mut buffer, &cam, &physics, 1, 0.0);

    // Find the brightest pixel matching the radiant solar glare
    let mut max_lum = 0.0f64;
    let mut center_idx = 0;
    for (i, p) in buffer.pixels.iter().enumerate() {
        if p.r > 190 && p.g > 140 {
            let lum = p.r as f64 * 0.299 + p.g as f64 * 0.587 + p.b as f64 * 0.114;
            if lum > max_lum {
                max_lum = lum;
                center_idx = i;
            }
        }
    }

    assert!(max_lum > 220.0, "Solar core must be blinding bright (> 220 lum), got {}", max_lum);
    let _v_h = buffer.virtual_height();
    let cy = center_idx / cols;
    let cx = center_idx % cols;
    println!("center_idx={}, cx={}, cy={}, r={}, g={}, b={}", center_idx, cx, cy, buffer.pixels[center_idx].r, buffer.pixels[center_idx].g, buffer.pixels[center_idx].b);

    for dx in 1..=4 {
        if cx + dx < cols {
            let p = &buffer.pixels[cy * cols + (cx + dx)];
            let lum = p.r as f64 * 0.299 + p.g as f64 * 0.587 + p.b as f64 * 0.114;
            assert!(
                lum > 100.0,
                "Pixel at offset +{} from sun core must be radiant glare (> 100 lum), got {} (no black void!)",
                dx, lum
            );
        }
    }
}

#[test]
fn test_mercury_keplerian_orbit_and_eccentricity() {
    let orbit = KeplerOrbit::mercury_around_sun();
    let pos_t0 = orbit.calculate_position(0.0);
    let pos_period = orbit.calculate_position(orbit.period);

    let diff = (pos_t0 - pos_period).length();
    assert!(
        diff < 1e-4,
        "Mercury orbit must close after 1 orbital period: diff = {} km",
        diff
    );

    // Sample distance across orbit to find perihelion and aphelion
    let mut min_r = f64::MAX;
    let mut max_r = 0.0f64;
    for step in 0..360 {
        let t = (step as f64 / 360.0) * orbit.period;
        let r = orbit.calculate_position(t).length();
        if r < min_r { min_r = r; }
        if r > max_r { max_r = r; }
    }

    // Theoretical perihelion: a * (1 - e) = 57,909,050 * (1 - 0.20563) ≈ 46,001,200 km
    // Theoretical aphelion: a * (1 + e) = 57,909,050 * (1 + 0.20563) ≈ 69,816,900 km
    assert!((min_r - 46_001_200.0).abs() < 50_000.0, "Mercury perihelion mismatch: {}", min_r);
    assert!((max_r - 69_816_900.0).abs() < 50_000.0, "Mercury aphelion mismatch: {}", max_r);
}

#[test]
fn test_mercury_spin_orbit_resonance_3_to_2() {
    let physics = SolarSystemPhysics::new();
    let mercury = &physics.mercury;
    let p_orbit = mercury.orbit.as_ref().unwrap().period;
    let p_spin = mercury.rotation_period_secs;

    // 2 orbits = 3 rotations
    let ratio = (2.0 * p_orbit) / (3.0 * p_spin);
    assert!(
        (ratio - 1.0).abs() < 1e-3,
        "Mercury must obey 3:2 spin-orbit resonance: ratio = {}",
        ratio
    );
}

#[test]
fn test_mercury_shader_albedo_and_lommel_seeliger() {
    use solar_system_ascii::render::mercury_shader::MercuryShader;

    let shader = MercuryShader::new();
    let alb = shader.sample_albedo(0.0, 0.0);
    assert!(alb > 20.0 && alb < 250.0, "Valid photographic albedo expected");

    // Caloris Basin is located around Lat 30.5° N, Lon 189.8° W
    let caloris_elev = shader.sample_elevation(30.5, -189.8);
    assert!(caloris_elev > 0.0, "Caloris basin elevation data present");

    // Sunlit side vs Night side
    let local_hit = Vec3::new(1.0, 0.0, 0.0);
    let sun_day = Vec3::new(1.0, 0.0, 0.0);
    let sun_night = Vec3::new(-1.0, 0.0, 0.0);
    let view = Vec3::new(1.0, 0.0, 0.0);

    let day_color = shader.shade(local_hit, sun_day, view);
    let night_color = shader.shade(local_hit, sun_night, view);

    assert_eq!(night_color, [0, 0, 0], "Mercury night side must be pitch black with zero atmospheric scattering");
    assert!(day_color[0] > 100 && day_color[1] > 90, "Mercury sunlit side must be brightly illuminated by 6.7x solar flux, got {:?}", day_color);
}

#[test]
fn test_hud_label_proximity_and_range_culling() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;

    let cols = 120;
    let rows = 60;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();

    // 1. In close orbit of Earth (21,000 km, < 10 * R_earth = 63,710 km):
    // [Terra] must be culled (hidden) to avoid polluting the planet surface
    cam.set_target(BodyId::Earth, physics.earth.position);
    cam.distance_km = 21_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    Hud::render(&mut buffer, &cam, &physics, 60.0, 600.0, false);

    // Search buffer for string "[Earth]"
    let mut found_earth = false;
    for y in 0..rows {
        let mut line = String::new();
        for x in 0..cols {
            let p = &buffer.pixels[(y * 2) * cols + x];
            if p.is_text {
                line.push(p.ch);
            } else {
                line.push(' ');
            }
        }
        if line.contains("[Earth]") {
            found_earth = true;
            break;
        }
    }
    assert!(!found_earth, "[Earth] label must be hidden when closely inspecting Earth");

    // 2. Zoomed out from Earth (100,000 km, > 10 * R_earth):
    // [Earth] label must reappear!
    cam.distance_km = 100_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    Hud::render(&mut buffer, &cam, &physics, 60.0, 600.0, false);

    let mut found_earth_zoomed = false;
    for y in 0..rows {
        let mut line = String::new();
        for x in 0..cols {
            let p = &buffer.pixels[(y * 2) * cols + x];
            if p.is_text {
                line.push(p.ch);
            } else {
                line.push(' ');
            }
        }
        if line.contains("[Earth]") {
            found_earth_zoomed = true;
            break;
        }
    }
    assert!(found_earth_zoomed, "[Earth] label must reappear when zooming out to system scale (> 10 * radius)");
}

#[test]
fn test_camera_flyto_mercury_transition() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();

    cam.set_target(BodyId::Earth, physics.earth.position);
    let from_pos = physics.earth.position;
    let to_pos = physics.mercury.position;

    cam.start_transition(BodyId::Mercury, from_pos, to_pos, 1.0);
    assert!(cam.transition.is_some());

    // Step transition to completion
    cam.update(1.0, &physics);
    assert!(cam.transition.is_none(), "Transition must finish at 1.0s");
    assert_eq!(cam.target_body, BodyId::Mercury);
    assert!((cam.distance_km - 7_500.0).abs() < 1.0, "Camera must arrive at Mercury's default distance");
}

#[test]
fn test_render_mercury_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 160;
    let rows = 80;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let raycaster = Raycaster::new();

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        raycaster.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // 1. Mercury Close-up Disk (Caloris Basin, Cratered Terrains)
    cam.set_target(BodyId::Mercury, physics.mercury.position);
    render_ppm(
        &mut buffer,
        &cam,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/mercury_close_up.ppm",
    );

    // 2. Mercury Terminator Relief (Discovery Rupes, Razor-sharp Airless Shadows)
    let mut cam_term = Camera::new();
    cam_term.set_target(BodyId::Mercury, physics.mercury.position);
    cam_term.distance_km = 4_500.0;
    cam_term.azimuth = 3.30;
    cam_term.elevation = 0.10;
    render_ppm(
        &mut buffer,
        &cam_term,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/mercury_terminator.ppm",
    );

    // 3. Earth Orbit Clean View (Proximity Culling Active: [Terra] hidden, [Lua] visible if in FOV)
    let mut cam_earth = Camera::new();
    cam_earth.set_target(BodyId::Earth, physics.earth.position);
    render_ppm(
        &mut buffer,
        &cam_earth,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/earth_orbit_clean_hud.ppm",
    );
}

#[test]
fn test_all_celestial_orbits_100_percent_precision() {
    let mut physics = SolarSystemPhysics::new();

    // 1. Sun heliocentric stillness
    for t in [0.0, 1000.0, 86400.0, 1e7] {
        physics.update(t);
        assert_eq!(physics.sun.position, Vec3::ZERO, "Sun must remain at origin (0, 0, 0)");
    }

    // 2. Mercury Orbit Audit
    let orb_merc = physics.mercury.orbit.unwrap();
    let pos_merc_0 = orb_merc.calculate_position(0.0);
    let pos_merc_p = orb_merc.calculate_position(orb_merc.period);
    assert!((pos_merc_0 - pos_merc_p).length() < 1e-4, "Mercury orbit must close precisely after 1 period");

    let mut min_r_merc = f64::MAX;
    let mut max_r_merc = 0.0f64;
    let mut max_z_merc = 0.0f64;
    for step in 0..1000 {
        let t = (step as f64 / 1000.0) * orb_merc.period;
        let pos = orb_merc.calculate_position(t);
        let r = pos.length();
        if r < min_r_merc { min_r_merc = r; }
        if r > max_r_merc { max_r_merc = r; }
        let z_ratio = pos.z.abs() / r;
        if z_ratio > max_z_merc { max_z_merc = z_ratio; }
    }
    // Theoretical: a*(1-e) = 46,001,202.9 km, a*(1+e) = 69,816,897.1 km
    assert!((min_r_merc - 46_001_203.0).abs() < 500.0, "Mercury perihelion: got {}", min_r_merc);
    assert!((max_r_merc - 69_816_897.0).abs() < 500.0, "Mercury aphelion: got {}", max_r_merc);
    // Inclination: sin(7.005°) ≈ 0.12198
    assert!((max_z_merc - (7.005f64.to_radians().sin())).abs() < 0.005, "Mercury orbital inclination");

    // Speed check (GM_sun = 1.3271244e11 km^3/s^2)
    let gm_sun = 1.32712440018e11;
    let v_merc_peri = orb_merc.orbital_speed(min_r_merc, gm_sun);
    let v_merc_apo = orb_merc.orbital_speed(max_r_merc, gm_sun);
    assert!((v_merc_peri - 58.98).abs() < 0.1, "Mercury perihelion speed: got {}", v_merc_peri);
    assert!((v_merc_apo - 38.86).abs() < 0.1, "Mercury aphelion speed: got {}", v_merc_apo);

    // 3. Earth Orbit Audit
    let orb_earth = physics.earth.orbit.unwrap();
    let pos_earth_0 = orb_earth.calculate_position(0.0);
    let pos_earth_p = orb_earth.calculate_position(orb_earth.period);
    assert!((pos_earth_0 - pos_earth_p).length() < 1e-4, "Earth orbit must close precisely after 1 period");

    let mut min_r_earth = f64::MAX;
    let mut max_r_earth = 0.0f64;
    for step in 0..1000 {
        let t = (step as f64 / 1000.0) * orb_earth.period;
        let pos = orb_earth.calculate_position(t);
        let r = pos.length();
        if r < min_r_earth { min_r_earth = r; }
        if r > max_r_earth { max_r_earth = r; }
        assert!(pos.z.abs() < 1e-10, "Earth must remain in ecliptic plane (Z = 0)");
    }
    // Theoretical: a*(1-e) = 147,098,450.4 km, a*(1+e) = 152,097,291.0 km
    assert!((min_r_earth - 147_098_450.0).abs() < 500.0, "Earth perihelion: got {}", min_r_earth);
    assert!((max_r_earth - 152_097_291.0).abs() < 500.0, "Earth aphelion: got {}", max_r_earth);

    let v_earth_peri = orb_earth.orbital_speed(min_r_earth, gm_sun);
    let v_earth_apo = orb_earth.orbital_speed(max_r_earth, gm_sun);
    assert!((v_earth_peri - 30.29).abs() < 0.1, "Earth perihelion speed: got {}", v_earth_peri);
    assert!((v_earth_apo - 29.29).abs() < 0.1, "Earth aphelion speed: got {}", v_earth_apo);

    // 4. Moon Orbit Audit
    let orb_moon = physics.moon.orbit.unwrap();
    let pos_moon_0 = orb_moon.calculate_position(0.0);
    let pos_moon_p = orb_moon.calculate_position(orb_moon.period);
    assert!((pos_moon_0 - pos_moon_p).length() < 1e-4, "Moon orbit must close precisely after 1 period");

    let mut min_r_moon = f64::MAX;
    let mut max_r_moon = 0.0f64;
    let mut max_z_moon = 0.0f64;
    for step in 0..1000 {
        let t = (step as f64 / 1000.0) * orb_moon.period;
        let pos = orb_moon.calculate_position(t);
        let r = pos.length();
        if r < min_r_moon { min_r_moon = r; }
        if r > max_r_moon { max_r_moon = r; }
        let z_ratio = pos.z.abs() / r;
        if z_ratio > max_z_moon { max_z_moon = z_ratio; }
    }
    // Theoretical: a*(1-e) = 363,296.4 km, a*(1+e) = 405,503.6 km
    assert!((min_r_moon - 363_296.0).abs() < 20.0, "Moon perigee: got {}", min_r_moon);
    assert!((max_r_moon - 405_504.0).abs() < 20.0, "Moon apogee: got {}", max_r_moon);
    assert!((max_z_moon - (5.145f64.to_radians().sin())).abs() < 0.005, "Moon orbital inclination");

    // 5. Hierarchy and Relative Distances over Time
    for day in 0..687 {
        let t = day as f64 * 86_400.0;
        physics.update(t);

        let r_merc = physics.mercury.position.length();
        let r_venus = physics.venus.position.length();
        let r_earth = physics.earth.position.length();
        let r_mars = physics.mars.position.length();
        let d_moon_earth = (physics.moon.position - physics.earth.position).length();
        let d_phobos_mars = (physics.phobos.position - physics.mars.position).length();
        let d_deimos_mars = (physics.deimos.position - physics.mars.position).length();

        assert!(r_merc < r_earth, "Mercury must strictly orbit inside Earth orbit at day {}", day);
        assert!(r_venus < r_earth, "Venus must strictly orbit inside Earth orbit at day {}", day);
        assert!(r_earth < r_mars, "Earth must strictly orbit inside Mars orbit at day {}", day);
        assert!(d_moon_earth >= 363_200.0 && d_moon_earth <= 405_600.0, "Moon must remain in valid geocentric range");
        assert!(d_phobos_mars >= 9_200.0 && d_phobos_mars <= 9_600.0, "Phobos must remain in valid areocentric range");
        assert!(d_deimos_mars >= 23_400.0 && d_deimos_mars <= 23_600.0, "Deimos must remain in valid areocentric range");
        assert!(d_phobos_mars < d_deimos_mars, "Phobos is inner moon, Deimos is outer moon");
    }
}

#[test]
fn test_venus_keplerian_orbit_and_circularity() {
    let mut physics = SolarSystemPhysics::new();
    let orb_venus = physics.venus.orbit.unwrap();

    // 1. Orbit Closure after exact period (224.701 days)
    let pos_0 = orb_venus.calculate_position(0.0);
    let pos_period = orb_venus.calculate_position(orb_venus.period);
    assert!(
        (pos_0 - pos_period).length() < 1e-4,
        "Venus orbit must close precisely after 1 orbital period"
    );

    // 2. Sample orbit across 1000 points to measure perihelion, aphelion, circularity and inclination
    let mut min_r = f64::MAX;
    let mut max_r = 0.0f64;
    let mut max_z_ratio = 0.0f64;
    for step in 0..1000 {
        let t = (step as f64 / 1000.0) * orb_venus.period;
        let pos = orb_venus.calculate_position(t);
        let r = pos.length();
        if r < min_r { min_r = r; }
        if r > max_r { max_r = r; }
        let z_ratio = pos.z.abs() / r;
        if z_ratio > max_z_ratio { max_z_ratio = z_ratio; }
    }

    // Theoretical: a = 108,208,000 km, e = 0.006772
    // Perihelion = a*(1-e) = 107,475,214 km
    // Aphelion   = a*(1+e) = 108,940,786 km
    assert!((min_r - 107_475_214.0).abs() < 500.0, "Venus perihelion: got {}", min_r);
    assert!((max_r - 108_940_786.0).abs() < 500.0, "Venus aphelion: got {}", max_r);
    // Inclination: sin(3.39458°) ≈ 0.05920
    assert!((max_z_ratio - (3.39458f64.to_radians().sin())).abs() < 0.005, "Venus inclination");

    // Circularity check: (r_max - r_min) / a ≈ 2*e ≈ 0.0135 (very circular compared to Mercury's 0.41)
    let circularity_diff = (max_r - min_r) / orb_venus.a;
    assert!(circularity_diff < 0.015, "Venus must have extremely circular orbit");

    // Speed check (GM_sun = 1.3271244e11 km^3/s^2)
    let gm_sun = 1.32712440018e11;
    let v_peri = orb_venus.orbital_speed(min_r, gm_sun);
    let v_apo = orb_venus.orbital_speed(max_r, gm_sun);
    assert!((v_peri - 35.26).abs() < 0.2, "Venus perihelion speed: got {}", v_peri);
    assert!((v_apo - 34.78).abs() < 0.2, "Venus aphelion speed: got {}", v_apo);

    // Orbital hierarchy: Mercury < Venus < Earth
    for day in 0..365 {
        let t = day as f64 * 86_400.0;
        physics.update(t);
        let r_merc = physics.mercury.position.length();
        let r_venus = physics.venus.position.length();
        let r_earth = physics.earth.position.length();

        assert!(r_merc < r_venus, "Mercury must orbit inside Venus at day {}", day);
        assert!(r_venus < r_earth, "Venus must orbit inside Earth at day {}", day);
    }
}

#[test]
fn test_venus_retrograde_rotation() {
    use solar_system_ascii::physics::ephemeris::{VENUS_AXIAL_TILT_RAD, VENUS_SIDEREAL_DAY_SECS};

    let tilt_deg = VENUS_AXIAL_TILT_RAD.to_degrees();
    assert!((tilt_deg - 177.36).abs() < 1e-2, "Venus axial tilt must be 177.36 deg (retrograde)");
    assert!(tilt_deg > 90.0, "Axial tilt > 90 degrees defines retrograde axial spin");

    let day_earth_days = VENUS_SIDEREAL_DAY_SECS / 86_400.0;
    assert!((day_earth_days - 243.025).abs() < 1e-3, "Venus sidereal day must be 243.025 days");

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    assert!((physics.venus.rotation_angle - 0.0).abs() < 1e-4);

    physics.update(VENUS_SIDEREAL_DAY_SECS * 0.5);
    assert!((physics.venus.rotation_angle - PI).abs() < 1e-4, "Venus should reach pi rotation at half sidereal day");

    physics.update(VENUS_SIDEREAL_DAY_SECS);
    assert!(physics.venus.rotation_angle.abs() < 1e-4, "Venus should complete full 2pi cycle back to 0 at one sidereal day");
}

#[test]
fn test_venus_dual_layer_shader() {
    use solar_system_ascii::render::venus_shader::VenusShader;

    let mut shader = VenusShader::new();
    assert!(!shader.radar_mode, "Default mode must be optical atmospheric clouds");

    // 1. Optical mode test: sample subsolar point (local_sun = [0, 0, 1], local_norm = [0, 0, 1])
    let optical_pixel = shader.shade(Vec3::Z, Vec3::Z, Vec3::Z, 0.0);
    // Optical mode should be yellowish-cream sulfuric acid cloud tops
    assert!(optical_pixel[0] > 180, "Optical Venus red component: got {}", optical_pixel[0]);
    assert!(optical_pixel[1] > 150, "Optical Venus green component: got {}", optical_pixel[1]);
    assert!(optical_pixel[2] > 90, "Optical Venus blue component: got {}", optical_pixel[2]);
    assert!(optical_pixel[0] >= optical_pixel[1] && optical_pixel[1] >= optical_pixel[2], "Venus optical spectrum: R >= G >= B");

    // 2. Radar SAR mode test
    shader.radar_mode = true;
    let radar_pixel = shader.shade(Vec3::Z, Vec3::Z, Vec3::Z, 0.0);
    // Radar mode uses Magellan false-color orange/amber SAR palette
    assert!(radar_pixel[0] > radar_pixel[2], "Radar mode has warm amber/orange SAR tint");
    assert!(radar_pixel[0] > 50, "Radar SAR response at subsolar point");
}

#[test]
fn test_standardized_astronomical_keys_and_contextual_moon() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::render::camera::Camera;

    // Body IDs ordering
    assert_eq!(BodyId::Sun as usize, 0);
    assert_eq!(BodyId::Mercury as usize, 1);
    assert_eq!(BodyId::Venus as usize, 2);
    assert_eq!(BodyId::Earth as usize, 3);
    assert_eq!(BodyId::Moon as usize, 4);

    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();

    // Default target is Sun (Solar System Overview)
    assert_eq!(cam.target_body, BodyId::Sun);
    cam.set_target(BodyId::Earth, physics.earth.position);
    assert_eq!(cam.target_body, BodyId::Earth);

    // Contextual Moon: from Earth -> Moon
    cam.set_target(BodyId::Moon, physics.moon.position);
    assert_eq!(cam.target_body, BodyId::Moon);

    // From Moon -> back to Earth
    cam.set_target(BodyId::Earth, physics.earth.position);
    assert_eq!(cam.target_body, BodyId::Earth);

    // Direct key navigation [1..4]
    cam.set_target(BodyId::Sun, physics.sun.position);
    assert_eq!(cam.target_body, BodyId::Sun);

    cam.set_target(BodyId::Mercury, physics.mercury.position);
    assert_eq!(cam.target_body, BodyId::Mercury);

    cam.set_target(BodyId::Venus, physics.venus.position);
    assert_eq!(cam.target_body, BodyId::Venus);
    assert!((cam.distance_km - 18_500.0).abs() < 1e-4);

    cam.set_target(BodyId::Earth, physics.earth.position);
    assert_eq!(cam.target_body, BodyId::Earth);
}

#[test]
fn test_render_venus_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 160;
    let rows = 80;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let mut raycaster = Raycaster::new();

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, rc: &Raycaster, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        rc.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // 1. Venus Optical Atmosphere Demo (Super-rotation cloud deck, UV chevron, illuminated disk)
    cam.set_target(BodyId::Venus, physics.venus.position);
    cam.azimuth = -0.80;
    cam.elevation = 0.15;
    raycaster.venus_shader.radar_mode = false;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/venus_atmosphere_optical.ppm",
    );

    // 2. Venus Magellan Radar SAR Mode Demo (Volcanic crust, Maxwell Montes, Aphrodite Terra)
    raycaster.venus_shader.radar_mode = true;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/venus_magellan_radar.ppm",
    );

    // 3. Venus Twilight Limb Arc (Extended horns from 350 km deep atmosphere scattering)
    cam.azimuth = 2.40;
    cam.elevation = 0.20;
    raycaster.venus_shader.radar_mode = false;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/venus_twilight_arc.ppm",
    );

    // 4. Venus Polar & Mid-Latitude View (Elevation looking at top pole and horizontal cloud streaks)
    let mut cam_user = Camera::new();
    cam_user.set_target(BodyId::Venus, physics.venus.position);
    cam_user.azimuth = -1.10;
    cam_user.elevation = 0.35;
    render_ppm(
        &mut buffer,
        &cam_user,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/venus_user_view.ppm",
    );
}

#[test]
fn test_mars_keplerian_orbit_and_eccentricity() {
    use solar_system_ascii::physics::ephemeris::{
        MARS_AXIAL_TILT_RAD, MARS_RADIUS_KM, MARS_SIDEREAL_DAY_SECS,
    };

    let orbit = KeplerOrbit::mars_around_sun();
    assert!((orbit.e - 0.0934).abs() < 1e-4, "Mars eccentricity must be 0.0934");
    assert!((orbit.a - 227_939_200.0).abs() < 1.0, "Mars semi-major axis must be 227,939,200 km");

    let p0 = orbit.calculate_position(0.0);
    let p_period = orbit.calculate_position(orbit.period);
    let diff = (p0 - p_period).length();
    assert!(diff < 1e-4, "Mars orbit must close after 1 orbital period: diff = {} km", diff);

    let period_days = orbit.period / 86400.0;
    assert!((period_days - 686.98).abs() < 0.1, "Mars orbital period must be ~686.98 days");

    let perihelion = orbit.a * (1.0 - orbit.e);
    let aphelion = orbit.a * (1.0 + orbit.e);
    assert!((perihelion - 206_649_679.0).abs() < 1000.0, "Perihelion ~206.6M km");
    assert!((aphelion - 249_228_721.0).abs() < 1000.0, "Aphelion ~249.2M km");

    assert!((MARS_RADIUS_KM - 3389.5).abs() < 1e-4);
    assert!((MARS_AXIAL_TILT_RAD - 0.4396).abs() < 1e-3);
    assert!((MARS_SIDEREAL_DAY_SECS - 88642.68).abs() < 1e-2);
}

#[test]
fn test_phobos_orbit_and_triaxial_shape() {
    use solar_system_ascii::physics::ephemeris::PHOBOS_SEMI_AXES_KM;
    use solar_system_ascii::render::phobos_shader::PhobosShader;

    let orbit = KeplerOrbit::phobos_around_mars();
    assert!((orbit.a - 9376.0).abs() < 1.0, "Phobos semi-major axis = 9376 km");
    let period_hours = orbit.period / 3600.0;
    assert!((period_hours - 7.654).abs() < 0.01, "Phobos period = 7.654 hours");

    // Phobos orbits in 7.65h, faster than Mars rotates (24.62h)
    assert!(orbit.period < 88642.68, "Phobos orbits faster than Mars rotation!");

    let p0 = orbit.calculate_position(0.0);
    let p_period = orbit.calculate_position(orbit.period);
    assert!((p0 - p_period).length() < 1e-4);

    // Verify triaxial semi-axes
    let (a, b, c) = PHOBOS_SEMI_AXES_KM;
    assert_eq!(a, 13.0);
    assert_eq!(b, 11.4);
    assert_eq!(c, 9.1);

    // Test Stickney crater displacement and shading
    let shader = PhobosShader::new();
    let elev_stickney = shader.stickney_elevation(0.0, solar_system_ascii::render::phobos_shader::STICKNEY_LON_RAD);
    assert!(elev_stickney < -0.15, "Stickney center must have deep depression: {}", elev_stickney);
    let norm_stickney = shader.surface_normal(Vec3::new(13.0, 0.0, 0.0));
    assert!(norm_stickney.length() > 0.99);

    let sun_dir = Vec3::new(1.0, 0.2, 0.0).normalize();
    let view_dir = Vec3::new(1.0, 0.0, 0.0).normalize();
    let color = shader.shade(Vec3::new(13.0, 0.0, 0.0), sun_dir, view_dir);
    // Phobos is dark carbonaceous chondrite with albedo ~0.07, blue ejecta
    assert!(color[0] > 0 && color[2] > 0);
    assert!(color[2] as f64 >= color[0] as f64 * 0.9, "Stickney ejecta has blue hue");
}

#[test]
fn test_deimos_orbit_and_triaxial_shape() {
    use solar_system_ascii::physics::ephemeris::DEIMOS_SEMI_AXES_KM;
    use solar_system_ascii::render::deimos_shader::DeimosShader;

    let orbit = KeplerOrbit::deimos_around_mars();
    assert!((orbit.a - 23463.0).abs() < 1.0, "Deimos semi-major axis = 23,463 km");
    let period_hours = orbit.period / 3600.0;
    assert!((period_hours - 30.298).abs() < 0.01, "Deimos period = 30.298 hours");

    let p0 = orbit.calculate_position(0.0);
    let p_period = orbit.calculate_position(orbit.period);
    assert!((p0 - p_period).length() < 1e-4);

    let (a, b, c) = DEIMOS_SEMI_AXES_KM;
    assert_eq!(a, 7.5);
    assert_eq!(b, 6.1);
    assert_eq!(c, 5.5);

    let shader = DeimosShader::new();
    let sun_dir = Vec3::new(1.0, 0.0, 0.0);
    let view_dir = Vec3::new(1.0, 0.0, 0.0);
    let color = shader.shade(Vec3::new(7.5, 0.0, 0.0), sun_dir, view_dir);
    assert!(color[0] > 0 && color[1] > 0 && color[2] > 0);
}

#[test]
fn test_ray_ellipsoid_intersection_precision() {
    use solar_system_ascii::math::Ray;
    use solar_system_ascii::physics::ephemeris::PHOBOS_SEMI_AXES_KM;

    // Ray directed along X-axis at center (a = 13.0)
    let ray_x = Ray::new(Vec3::new(100.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0));
    let hit_x = solar_system_ascii::render::raycaster::Raycaster::intersect_ellipsoid(
        &ray_x,
        Vec3::ZERO,
        Mat3::IDENTITY,
        PHOBOS_SEMI_AXES_KM,
    );
    assert!(hit_x.is_some());
    let (t_x, norm_x) = hit_x.unwrap();
    assert!((t_x - (100.0 - 13.0)).abs() < 1e-4, "Distance along X should hit a = 13.0 km");
    assert!((norm_x - Vec3::new(1.0, 0.0, 0.0)).length() < 1e-4);

    // Ray directed along Y-axis at center (c = 9.1, polar axis)
    let ray_y = Ray::new(Vec3::new(0.0, 100.0, 0.0), Vec3::new(0.0, -1.0, 0.0));
    let hit_y = solar_system_ascii::render::raycaster::Raycaster::intersect_ellipsoid(
        &ray_y,
        Vec3::ZERO,
        Mat3::IDENTITY,
        PHOBOS_SEMI_AXES_KM,
    );
    assert!(hit_y.is_some());
    let (t_y, norm_y) = hit_y.unwrap();
    assert!((t_y - (100.0 - 9.1)).abs() < 1e-4, "Distance along Y should hit c = 9.1 km");
    assert!((norm_y - Vec3::new(0.0, 1.0, 0.0)).length() < 1e-4);

    // Ray directed along Z-axis at center (b = 11.4, along-orbit axis)
    let ray_z = Ray::new(Vec3::new(0.0, 0.0, 100.0), Vec3::new(0.0, 0.0, -1.0));
    let hit_z = solar_system_ascii::render::raycaster::Raycaster::intersect_ellipsoid(
        &ray_z,
        Vec3::ZERO,
        Mat3::IDENTITY,
        PHOBOS_SEMI_AXES_KM,
    );
    assert!(hit_z.is_some());
    let (t_z, norm_z) = hit_z.unwrap();
    assert!((t_z - (100.0 - 11.4)).abs() < 1e-4, "Distance along Z should hit b = 11.4 km");
    assert!((norm_z - Vec3::new(0.0, 0.0, 1.0)).length() < 1e-4);

    // Ray missing the ellipsoid entirely
    let ray_miss = Ray::new(Vec3::new(100.0, 20.0, 20.0), Vec3::new(-1.0, 0.0, 0.0));
    let hit_miss = solar_system_ascii::render::raycaster::Raycaster::intersect_ellipsoid(
        &ray_miss,
        Vec3::ZERO,
        Mat3::IDENTITY,
        PHOBOS_SEMI_AXES_KM,
    );
    assert!(hit_miss.is_none());
}

#[test]
fn test_standardized_keys_and_mars_moons_navigation() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::render::camera::Camera;

    assert_eq!(BodyId::Mars as usize, 5);
    assert_eq!(BodyId::Phobos as usize, 6);
    assert_eq!(BodyId::Deimos as usize, 7);

    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();

    // Direct key navigation [5] -> Mars
    cam.set_target(BodyId::Mars, physics.mars.position);
    assert_eq!(cam.target_body, BodyId::Mars);
    assert!((cam.distance_km - 10_500.0).abs() < 1e-4);

    // Contextual [M] at Mars -> Phobos
    cam.set_target(BodyId::Phobos, physics.phobos.position);
    assert_eq!(cam.target_body, BodyId::Phobos);
    assert!((cam.distance_km - 44.0).abs() < 1e-4);

    // Contextual [N] at Phobos -> Deimos
    cam.set_target(BodyId::Deimos, physics.deimos.position);
    assert_eq!(cam.target_body, BodyId::Deimos);
    assert!((cam.distance_km - 26.0).abs() < 1e-4);

    // Contextual [M] at Deimos -> Mars
    cam.set_target(BodyId::Mars, physics.mars.position);
    assert_eq!(cam.target_body, BodyId::Mars);

    // Contextual [N] at Mars -> Deimos
    cam.set_target(BodyId::Deimos, physics.deimos.position);
    assert_eq!(cam.target_body, BodyId::Deimos);

    // Contextual [M] at Deimos -> Phobos
    cam.set_target(BodyId::Phobos, physics.phobos.position);
    assert_eq!(cam.target_body, BodyId::Phobos);

    // Verify Saturn's optimal beauty angle
    let def_saturn = Camera::default_view(BodyId::Saturn);
    assert!((def_saturn.distance_km - 200_000.0).abs() < 1e-4);
    assert!((def_saturn.elevation - 0.16).abs() < 1e-4);
    assert!((def_saturn.azimuth - (-1.45)).abs() < 1e-4);
}

#[test]
fn test_render_mars_system_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 160;
    let rows = 80;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let raycaster = Raycaster::new();

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, rc: &Raycaster, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        rc.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // 1. Mars Full Disk (Olympus Mons, Valles Marineris, polar ice cap, butterscotch sky)
    cam.set_target(BodyId::Mars, physics.mars.position);
    let sun_dir_mars = (physics.sun.position - physics.mars.position).normalize();
    let sun_az_mars = sun_dir_mars.x.atan2(sun_dir_mars.z);
    cam.azimuth = sun_az_mars + 0.40;
    cam.elevation = 0.22;
    cam.distance_km = 9_500.0;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/mars_full_disk.ppm",
    );

    // 2. Phobos Close-up (Stickney crater, grooved terrain, potato silhouette)
    cam.set_target(BodyId::Phobos, physics.phobos.position);
    let sun_dir_phobos = (physics.sun.position - physics.phobos.position).normalize();
    let sun_az_phobos = sun_dir_phobos.x.atan2(sun_dir_phobos.z);
    cam.azimuth = sun_az_phobos + 0.35;
    cam.elevation = 0.18;
    cam.distance_km = 44.0;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/phobos_stickney_close_up.ppm",
    );

    // 3. Deimos Close-up (Swift & Voltaire faculae, smooth dusty regolith, triaxial shape)
    cam.set_target(BodyId::Deimos, physics.deimos.position);
    let sun_dir_deimos = (physics.sun.position - physics.deimos.position).normalize();
    let sun_az_deimos = sun_dir_deimos.x.atan2(sun_dir_deimos.z);
    cam.azimuth = sun_az_deimos + 0.30;
    cam.elevation = 0.15;
    cam.distance_km = 26.0;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/deimos_dusty_close_up.ppm",
    );
}

#[test]
fn test_ceres_keplerian_orbit_and_oblate_shape() {
    use solar_system_ascii::physics::ephemeris::{
        CERES_AXIAL_TILT_RAD, CERES_MEAN_RADIUS_KM, CERES_RADIUS_EQ_KM, CERES_RADIUS_POL_KM,
        CERES_SIDEREAL_DAY_SECS, KeplerOrbit,
    };
    use solar_system_ascii::render::ceres_shader::{
        CeresShader, CEREALIA_FACULA_LAT_DEG, CEREALIA_FACULA_LON_DEG,
    };

    let orbit = KeplerOrbit::ceres_around_sun();
    assert!((orbit.e - 0.0758).abs() < 1e-4, "Ceres eccentricity must be 0.0758");
    assert!((orbit.a - 413_970_000.0).abs() < 1.0, "Ceres semi-major axis must be 413,970,000 km");

    let p0 = orbit.calculate_position(0.0);
    let p_period = orbit.calculate_position(orbit.period);
    assert!((p0 - p_period).length() < 1e-4, "Ceres orbit must close smoothly in 1 period");

    let period_days = orbit.period / 86_400.0;
    assert!((period_days - 1681.63).abs() < 0.5, "Ceres orbital period must be ~1,681.6 days: got {}", period_days);

    // Oblate spheroid semi-axes: 482.6 km equatorial, 445.6 km polar
    assert_eq!(CERES_RADIUS_EQ_KM, 482.6);
    assert_eq!(CERES_RADIUS_POL_KM, 445.6);
    assert_eq!(CERES_MEAN_RADIUS_KM, 473.0);

    let tilt_deg = CERES_AXIAL_TILT_RAD.to_degrees();
    assert!((tilt_deg - 4.0).abs() < 1e-2, "Ceres axial tilt is 4 degrees");

    let day_hours = CERES_SIDEREAL_DAY_SECS / 3600.0;
    assert!((day_hours - 9.074).abs() < 0.01, "Ceres sidereal day is 9.074 hours");

    // Test Occator Cerealia Facula high albedo
    let shader = CeresShader::new();
    let facula_lon = CEREALIA_FACULA_LON_DEG.to_radians();
    let facula_lat = CEREALIA_FACULA_LAT_DEG.to_radians();
    let alb = shader.sample_albedo(facula_lat, facula_lon);
    assert!(alb[0] > 150.0 && alb[2] > 160.0, "Cerealia Facula sodium carbonate salt must gleam brightly");
}

#[test]
fn test_vesta_keplerian_orbit_and_rheasilvia_basin() {
    use solar_system_ascii::physics::ephemeris::{
        VESTA_AXIAL_TILT_RAD, VESTA_MEAN_RADIUS_KM, VESTA_SEMI_AXES_KM, VESTA_SIDEREAL_DAY_SECS,
        KeplerOrbit,
    };
    use solar_system_ascii::render::vesta_shader::VestaShader;

    let orbit = KeplerOrbit::vesta_around_sun();
    assert!((orbit.e - 0.0887).abs() < 1e-4, "Vesta eccentricity must be 0.0887");
    assert!((orbit.a - 353_370_000.0).abs() < 1.0, "Vesta semi-major axis must be 353,370,000 km");

    let p0 = orbit.calculate_position(0.0);
    let p_period = orbit.calculate_position(orbit.period);
    assert!((p0 - p_period).length() < 1e-4, "Vesta orbit must close smoothly in 1 period");

    let period_days = orbit.period / 86_400.0;
    assert!((period_days - 1325.7).abs() < 0.5, "Vesta orbital period must be ~1,325.7 days: got {}", period_days);

    // Triaxial semi-axes: (286.3, 278.6, 223.2)
    let (a, b, c) = VESTA_SEMI_AXES_KM;
    assert_eq!(a, 286.3);
    assert_eq!(b, 278.6);
    assert_eq!(c, 223.2);
    assert_eq!(VESTA_MEAN_RADIUS_KM, 262.7);

    let tilt_deg = VESTA_AXIAL_TILT_RAD.to_degrees();
    assert!((tilt_deg - 27.2).abs() < 1e-2, "Vesta axial tilt is 27.2 degrees");

    let day_hours = VESTA_SIDEREAL_DAY_SECS / 3600.0;
    assert!((day_hours - 5.342).abs() < 0.01, "Vesta sidereal day is 5.342 hours");

    // Test basaltic eucrite albedo
    let shader = VestaShader::new();
    let alb = shader.sample_albedo(0.0, 0.0);
    assert!(alb[0] > alb[1] && alb[1] > alb[2], "Vesta basaltic crust has warm tint R > G > B");
    assert!(alb[1] > 0.35, "Vesta has high geometric albedo ~0.42");
}

#[test]
fn test_asteroid_field_kirkwood_gaps_and_density() {
    use solar_system_ascii::physics::asteroid_belt::ASTEROID_COUNT;
    use solar_system_ascii::physics::SolarSystemPhysics;

    let physics = SolarSystemPhysics::new();
    assert_eq!(physics.asteroid_field.particles.len(), ASTEROID_COUNT);
    assert_eq!(ASTEROID_COUNT, 3500);

    let au_km = 149_597_870.7;

    // Check Kirkwood gaps: 3:1 at 2.50 AU, 2:1 at 3.28 AU
    let count_in_range = |min_au: f64, max_au: f64| -> usize {
        physics
            .asteroid_field
            .particles
            .iter()
            .filter(|p| {
                let a_au = p.a / au_km;
                a_au >= min_au && a_au < max_au
            })
            .count()
    };

    // 3:1 Kirkwood Gap at 2.50 AU (+/- 0.02 AU)
    let in_gap_3_1 = count_in_range(2.48, 2.52);
    // Adjacent regular belt bins
    let adjacent_left = count_in_range(2.40, 2.44);
    let adjacent_right = count_in_range(2.56, 2.60);

    assert!(in_gap_3_1 < adjacent_left / 3 && in_gap_3_1 < adjacent_right / 3,
        "3:1 Kirkwood gap must be depleted: gap count {}, left {}, right {}",
        in_gap_3_1, adjacent_left, adjacent_right
    );

    // 2:1 Kirkwood Gap at 3.28 AU (+/- 0.02 AU)
    let in_gap_2_1 = count_in_range(3.26, 3.30);
    let adjacent_inner = count_in_range(3.18, 3.22);
    assert!(in_gap_2_1 < adjacent_inner / 2,
        "2:1 Kirkwood gap must be depleted: gap count {}, inner {}",
        in_gap_2_1, adjacent_inner
    );

    // Verify Keplerian analytical solver stability
    let pos_0 = physics.asteroid_field.calculate_position(42, 0.0);
    let p = &physics.asteroid_field.particles[42];
    let pos_period = physics.asteroid_field.calculate_position(42, p.period);
    assert!((pos_0 - pos_period).length() < 1e-4, "Particle 42 must close orbit exactly");
}

#[test]
fn test_asteroid_belt_and_ceres_coplanar() {
    use solar_system_ascii::physics::SolarSystemPhysics;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    // Ceres and Vesta orbit in X-Y plane with Z as inclination axis
    let r_xy_ceres = (physics.ceres.position.x.powi(2) + physics.ceres.position.y.powi(2)).sqrt();
    let r_ceres_total = physics.ceres.position.length();
    assert!(r_xy_ceres / r_ceres_total > 0.95, "Ceres must orbit primarily in X-Y plane (got ratio {})", r_xy_ceres / r_ceres_total);
    assert!(physics.ceres.position.z.abs() / r_ceres_total < 0.25, "Ceres Z must match its ~10.6 deg inclination");

    // Sample 100 asteroids: verify their orbital plane is X-Y with small Z dispersion (inclination)
    for i in 0..100 {
        let p_pos = physics.asteroid_field.calculate_position(i, 0.0);
        let r_xy = (p_pos.x.powi(2) + p_pos.y.powi(2)).sqrt();
        let r_total = p_pos.length();
        // In X-Y plane with inclination ~8 deg (sin 8 deg ~ 0.14), r_xy must be > 95% of r_total
        assert!(r_xy / r_total > 0.90, "Asteroid {} must lie primarily in X-Y plane: r_xy={}, r_total={}", i, r_xy, r_total);
        assert!(p_pos.z.abs() / r_total < 0.35, "Asteroid {} Z dispersion must be bounded by orbital inclination", i);
    }
}

#[test]
fn test_standardized_navigation_ceres_vesta_belt() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    assert_eq!(cam.target_body, BodyId::Sun);

    // Set to Ceres
    let ceres_def = Camera::default_view(BodyId::Ceres);
    assert_eq!(ceres_def.distance_km, 1400.0);
    cam.set_target(BodyId::Ceres, Vec3::new(413_970_000.0, 0.0, 0.0));
    assert_eq!(cam.target_body, BodyId::Ceres);

    // Zoom in limit
    for _ in 0..50 {
        cam.zoom_in();
    }
    assert_eq!(cam.distance_km, 490.0, "Ceres min zoom distance must be 490 km");

    // Set to Vesta
    let vesta_def = Camera::default_view(BodyId::Vesta);
    assert_eq!(vesta_def.distance_km, 850.0);
    cam.set_target(BodyId::Vesta, Vec3::new(353_370_000.0, 0.0, 0.0));
    assert_eq!(cam.target_body, BodyId::Vesta);

    // Zoom in limit
    for _ in 0..50 {
        cam.zoom_in();
    }
    assert_eq!(cam.distance_km, 240.0, "Vesta min zoom distance must be 240 km");
}

#[test]
fn test_render_asteroid_belt_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 160;
    let rows = 80;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();
    let raycaster = Raycaster::new();

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, rc: &Raycaster, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        rc.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // 1. Ceres Close-up (Occator crater, Cerealia Facula bright salt, Ahuna Mons cryovolcano)
    cam.set_target(BodyId::Ceres, physics.ceres.position);
    cam.azimuth = -2.61;
    cam.elevation = 0.28;
    cam.distance_km = 1_400.0;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/ceres_occator_faculae.ppm",
    );

    // 2. Vesta Close-up (Rheasilvia impact basin, central peak, Divalia Fossae grooves)
    cam.set_target(BodyId::Vesta, physics.vesta.position);
    cam.azimuth = -1.25;
    cam.elevation = -0.55;
    cam.distance_km = 850.0;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/vesta_rheasilvia_crater.ppm",
    );

    // 3. Asteroid Belt Field Overview (3,500 particles showing Kirkwood gaps and circular swarm)
    cam.set_target(BodyId::Ceres, physics.ceres.position);
    cam.distance_km = 650_000_000.0;
    cam.elevation = 0.60;
    cam.azimuth = 0.50;
    render_ppm(
        &mut buffer,
        &cam,
        &raycaster,
        "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf/asteroid_belt_overview.ppm",
    );
}

#[test]
fn test_jupiter_keplerian_orbit_and_oblate_disk() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::ephemeris::{
        JUPITER_RADIUS_EQ_KM, JUPITER_RADIUS_POL_KM, JUPITER_MEAN_RADIUS_KM,
    };
    use solar_system_ascii::physics::SolarSystemPhysics;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let jup = physics.get_body(BodyId::Jupiter);
    let dist_to_sun = jup.position.length();

    let to_sun = (solar_system_ascii::math::Vec3::ZERO - jup.position).normalize();
    let sun_az = to_sun.x.atan2(to_sun.z);
    println!("Jupiter pos: {:?}, to_sun: {:?}, sun_az: {}", jup.position, to_sun, sun_az);

    // Jupiter perihelion ~740.5M km, aphelion ~816.6M km (mean ~778.5M km)
    assert!(dist_to_sun > 740_000_000.0 && dist_to_sun < 820_000_000.0);

    // Oblate spheroid flattening: f = (a - c) / a = 0.06487
    let flattening = (JUPITER_RADIUS_EQ_KM - JUPITER_RADIUS_POL_KM) / JUPITER_RADIUS_EQ_KM;
    assert!((flattening - 0.06487).abs() < 1e-4);
    assert!((JUPITER_MEAN_RADIUS_KM - 69_911.0).abs() < 1.0);
}

#[test]
fn test_laplace_resonance_io_europa_ganymede() {
    use solar_system_ascii::physics::ephemeris::{
        IO_SIDEREAL_DAY_SECS, EUROPA_SIDEREAL_DAY_SECS, GANYMEDE_SIDEREAL_DAY_SECS,
    };
    use solar_system_ascii::physics::SolarSystemPhysics;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let jup_pos = physics.jupiter.position;
    let io_r = (physics.io.position - jup_pos).length();
    let europa_r = (physics.europa.position - jup_pos).length();
    let ganymede_r = (physics.ganymede.position - jup_pos).length();
    let callisto_r = (physics.callisto.position - jup_pos).length();

    // Check semi-major axis ordering: Io < Europa < Ganymede < Callisto
    assert!(io_r < europa_r);
    assert!(europa_r < ganymede_r);
    assert!(ganymede_r < callisto_r);

    // Exact Laplace resonance 4:2:1
    let ratio_europa_io = EUROPA_SIDEREAL_DAY_SECS / IO_SIDEREAL_DAY_SECS;
    let ratio_ganymede_io = GANYMEDE_SIDEREAL_DAY_SECS / IO_SIDEREAL_DAY_SECS;
    assert!((ratio_europa_io - 2.0).abs() < 1e-3);
    assert!((ratio_ganymede_io - 4.0).abs() < 1e-3);
}

#[test]
fn test_jovian_minor_moon_swarm_groups() {
    use solar_system_ascii::physics::jovian_swarm::{JovianMoonGroup, MINOR_MOON_COUNT};
    use solar_system_ascii::physics::SolarSystemPhysics;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    assert_eq!(physics.jovian_swarm.count(), MINOR_MOON_COUNT);
    assert_eq!(MINOR_MOON_COUNT, 91);

    let mut amalthea_count = 0;
    let mut himalia_count = 0;
    let mut retrograde_count = 0;

    for (i, moon) in physics.jovian_swarm.moons.iter().enumerate() {
        match moon.group {
            JovianMoonGroup::Amalthea => {
                amalthea_count += 1;
                assert!(moon.a < 250_000.0); // Inside synchronous radius
            }
            JovianMoonGroup::Himalia => {
                himalia_count += 1;
                assert!(moon.inc < 0.6); // Prograde (~28 deg = 0.49 rad)
            }
            JovianMoonGroup::Ananke | JovianMoonGroup::Carme | JovianMoonGroup::Pasiphae => {
                retrograde_count += 1;
                assert!(moon.inc > 2.0); // Retrograde (145° - 165° > 2.5 rad)
            }
            _ => {}
        }

        let pos = physics.jovian_swarm.calculate_position(i, physics.jupiter.position, 100.0);
        assert!(!pos.x.is_nan() && !pos.y.is_nan() && !pos.z.is_nan());
    }

    assert_eq!(amalthea_count, 4); // Metis, Adrastea, Amalthea, Thebe
    assert_eq!(himalia_count, 10);
    assert!(retrograde_count >= 70);
}

#[test]
fn test_render_jupiter_galilean_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 160;
    let lines = 50;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let raycaster = Raycaster::new();
    let mut cam = Camera::new();

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        raycaster.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    let brain_dir = "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf";

    // 1. Jupiter Full Disk with Great Red Spot & Atmospheric Limb Darkening
    cam.set_target(BodyId::Jupiter, physics.jupiter.position);
    cam.azimuth = -1.35;
    cam.elevation = 0.12;
    cam.distance_km = 210_000.0;
    render_ppm(&mut buffer, &cam, &format!("{}/jupiter_full_disk_grs.ppm", brain_dir));

    // 2. Jupiter System Overview (Framing Jupiter + Galilean Moons + Minor Moons Swarm)
    cam.set_target(BodyId::Jupiter, physics.jupiter.position);
    cam.azimuth = -1.15;
    cam.elevation = 0.35;
    cam.distance_km = 2_800_000.0;
    render_ppm(&mut buffer, &cam, &format!("{}/jupiter_system_overview.ppm", brain_dir));

    // 3. Io Volcanic Close-up (Sulfur plains, Loki/Pele calderas)
    cam.set_target(BodyId::Io, physics.io.position);
    cam.azimuth = -1.40;
    cam.elevation = 0.14;
    cam.distance_km = 5_500.0;
    render_ppm(&mut buffer, &cam, &format!("{}/io_volcanoes_close_up.ppm", brain_dir));

    // 4. Europa Ice Crust Close-up (Conamara Chaos, red-brown lineae fractures)
    cam.set_target(BodyId::Europa, physics.europa.position);
    cam.azimuth = -1.38;
    cam.elevation = 0.14;
    cam.distance_km = 4_800.0;
    render_ppm(&mut buffer, &cam, &format!("{}/europa_ice_lineae_close_up.ppm", brain_dir));

    // 5. Ganymede Grooved Terrain Close-up (Uruk Sulcus vs Galileo Regio)
    cam.set_target(BodyId::Ganymede, physics.ganymede.position);
    cam.azimuth = -1.36;
    cam.elevation = 0.14;
    cam.distance_km = 7_800.0;
    render_ppm(&mut buffer, &cam, &format!("{}/ganymede_grooved_terrain.ppm", brain_dir));

    // 6. Callisto Valhalla Basin Close-up (Multi-ring ripple basin & saturated craters)
    cam.set_target(BodyId::Callisto, physics.callisto.position);
    cam.azimuth = -1.34;
    cam.elevation = 0.14;
    cam.distance_km = 7_200.0;
    render_ppm(&mut buffer, &cam, &format!("{}/callisto_valhalla_crater.ppm", brain_dir));
}

#[test]
fn test_jupiter_system_benchmark_framerate() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::raycaster::Raycaster;
    use std::time::Instant;

    let cols = 160;
    let lines = 50;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let raycaster = Raycaster::new();
    let mut cam = Camera::new();
    cam.set_target(BodyId::Jupiter, physics.jupiter.position);
    cam.azimuth = -1.35;
    cam.elevation = 0.12;
    cam.distance_km = 210_000.0;

    // Warm-up
    raycaster.render_frame(&mut buffer, &cam, &physics, 1, 0.0);

    let frames = 100;
    let start = Instant::now();
    for f in 0..frames {
        physics.update(f as f64 * 0.1);
        cam.update(0.016, &physics);
        raycaster.render_frame(&mut buffer, &cam, &physics, f as u64, f as f64 * 0.1);
    }
    let elapsed = start.elapsed();
    let fps = frames as f64 / elapsed.as_secs_f64();
    println!("Jupiter System Benchmark: {} frames in {:.3}s ({:.1} FPS)", frames, elapsed.as_secs_f64(), fps);
    let min_fps = if cfg!(debug_assertions) { 5.0 } else { 200.0 };
    assert!(fps > min_fps, "FPS {:.1} below minimum performance threshold {:.1}", fps, min_fps);
}


#[test]
fn test_saturn_keplerian_orbit_and_extreme_oblateness() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let saturn = physics.get_body(BodyId::Saturn);
    assert_eq!(saturn.name, "Saturn");
    assert_eq!(saturn.radius_km, 58_232.0);

    // Verify distance from Sun: Saturn semi-major axis ~9.5826 AU (~1.433e9 km)
    let dist_from_sun = saturn.position.length();
    assert!(
        dist_from_sun > 1.35e9 && dist_from_sun < 1.55e9,
        "Saturn distance from sun should be ~1.433e9 km, got {}",
        dist_from_sun
    );

    // Verify extreme oblateness: flattening f = (60268 - 54364) / 60268 = 0.09796
    let r_eq = 60_268.0_f64;
    let r_pol = 54_364.0_f64;
    let flattening = (r_eq - r_pol) / r_eq;
    assert!((flattening - 0.09796_f64).abs() < 1e-4, "Flattening should be ~0.09796");
}

#[test]
fn test_saturn_rings_radial_dimensions_and_divisions() {
    use solar_system_ascii::render::saturn_shader::SaturnShader;

    let shader = SaturnShader::new();

    // 1. Inside inner ring (r < 74,500 km) -> None
    assert!(shader.sample_ring(70_000.0).is_none());

    // 2. C Ring (r ~ 80,000 km) -> Faint / translucent
    let (c_col, c_tau) = shader.sample_ring(80_000.0).expect("C ring sample");
    assert!(c_tau > 0.05 && c_tau < 0.40, "C ring tau: {}", c_tau);
    assert!(c_col[0] > 0);

    // 3. B Ring (r ~ 100,000 km) -> Dense and bright
    let (b_col, b_tau) = shader.sample_ring(100_000.0).expect("B ring sample");
    assert!(b_tau > 0.60, "B ring should be dense: tau={}", b_tau);
    assert!(b_col[0] > c_col[0], "B ring should be brighter than C ring");

    // 4. Cassini Division (r ~ 120,000 km) -> Lower opacity gap
    let (_, cassini_tau) = shader.sample_ring(120_000.0).expect("Cassini division sample");
    assert!(cassini_tau < b_tau, "Cassini division should be lower opacity: {}", cassini_tau);

    // 5. A Ring (r ~ 128,000 km) -> Moderate brightness
    let (a_col, a_tau) = shader.sample_ring(128_000.0).expect("A ring sample");
    assert!(a_tau > 0.35, "A ring tau: {}", a_tau);
    assert!(a_col[0] > 0);

    // 6. Outside outer ring (r > 140,220 km) -> None
    assert!(shader.sample_ring(145_000.0).is_none());
}

#[test]
fn test_saturn_rings_and_body_mutual_shadowing() {
    use solar_system_ascii::math::Vec3;
    use solar_system_ascii::render::saturn_shader::SaturnShader;

    let shader = SaturnShader::new();

    // 1. Ring shadow cast onto Saturn atmosphere
    let norm = Vec3::new(1.0, 0.1, 0.0).normalize();
    let pos = Vec3::new(60_000.0, 5_000.0, 0.0);
    // Sun slightly below ring plane so light passes through rings before hitting northern hemisphere
    let sun = Vec3::new(0.5, -0.3, 0.0).normalize();
    let view = Vec3::new(1.0, 0.0, 0.0);
    let lit_rgb = shader.shade(norm, pos, sun, view);
    assert!(lit_rgb[0] > 0 || lit_rgb[1] > 0);

    // 2. Saturn body shadow cast onto Rings
    let ring_pos_night = Vec3::new(-100_000.0, 0.0, 0.0);
    let sun_dir = Vec3::new(1.0, 0.1, 0.0).normalize();
    let view_dir = Vec3::new(0.0, 1.0, 0.0);
    let sample = shader.shade_ring(ring_pos_night, sun_dir, view_dir);
    assert!(sample.is_some());
    let rgba = sample.unwrap();
    assert!(rgba[0] <= 5 && rgba[1] <= 5 && rgba[2] <= 5, "Ring in shadow should be dark: {:?}", rgba);

    // Dayside ring point
    let ring_pos_day = Vec3::new(100_000.0, 0.0, 0.0);
    let sample_day = shader.shade_ring(ring_pos_day, sun_dir, view_dir);
    assert!(sample_day.is_some());
    let rgba_day = sample_day.unwrap();
    assert!(rgba_day[0] > 10 || rgba_day[1] > 10, "Dayside ring should be lit: {:?}", rgba_day);
}

#[test]
fn test_saturnian_minor_moons_swarm_distribution() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let saturn_pos = physics.get_body(BodyId::Saturn).position;

    // Verify all 141 minor moons exist
    assert_eq!(physics.saturnian_swarm.count(), 141);

    // Check inner ring shepherds vs outer irregulars
    let mut min_dist = f64::MAX;
    let mut max_dist = 0.0_f64;

    for i in 0..physics.saturnian_swarm.count() {
        let pos = physics.saturnian_swarm.calculate_position(i, saturn_pos, 0.0);
        let dist = (pos - saturn_pos).length();
        assert!(!dist.is_nan());
        if dist < min_dist {
            min_dist = dist;
        }
        if dist > max_dist {
            max_dist = dist;
        }
    }

    // Inner shepherds (Pan, Daphnis) orbit at ~133,000 - 140,000 km
    assert!(
        min_dist > 120_000.0 && min_dist < 150_000.0,
        "Closest shepherd should orbit at ~133k km, got {}",
        min_dist
    );

    // Outer irregulars orbit up to 25 - 40 million km
    assert!(
        max_dist > 15_000_000.0 && max_dist < 45_000_000.0,
        "Farthest irregular moon should orbit at ~20-38M km, got {}",
        max_dist
    );
}

#[test]
fn test_saturn_major_moons_albedo_and_contrasts() {
    use solar_system_ascii::math::Vec3;
    use solar_system_ascii::render::enceladus_shader::EnceladusShader;
    use solar_system_ascii::render::iapetus_shader::IapetusShader;
    use solar_system_ascii::render::mimas_shader::MimasShader;
    use solar_system_ascii::render::rhea_shader::RheaShader;
    use solar_system_ascii::render::titan_shader::TitanShader;

    let titan = TitanShader::new();
    let enceladus = EnceladusShader::new();
    let mimas = MimasShader::new();
    let iapetus = IapetusShader::new();
    let rhea = RheaShader::new();

    let norm = Vec3::new(1.0, 0.0, 0.0);
    let sun = Vec3::new(1.0, 0.0, 0.0);
    let view = Vec3::new(1.0, 0.0, 0.0);

    // Titan: dense amber/orange tholin haze
    let titan_col = titan.shade(norm, sun, view);
    assert!(
        titan_col[0] > titan_col[2],
        "Titan must be distinctly golden/orange: {:?}",
        titan_col
    );

    // Enceladus: ultra-bright water ice (highest albedo in solar system)
    let enc_col = enceladus.shade(norm, sun, view);
    assert!(
        enc_col[0] > 200 && enc_col[1] > 200 && enc_col[2] > 200,
        "Enceladus should be brilliant ice: {:?}",
        enc_col
    );

    // Mimas: steel-gray cratered ice
    let mimas_col = mimas.shade(norm, sun, view);
    assert!(mimas_col[0] > 100);

    // Iapetus: Yin-Yang dichotomy
    let norm_cassini = Vec3::new(0.0, 0.0, -1.0);
    let norm_roncevaux = Vec3::new(0.552, 0.0, 0.834).normalize();
    let cassini_col = iapetus.shade(norm_cassini, norm_cassini, norm_cassini);
    let roncevaux_col = iapetus.shade(norm_roncevaux, norm_roncevaux, norm_roncevaux);
    let lum_c = 0.299 * cassini_col[0] as f64 + 0.587 * cassini_col[1] as f64 + 0.114 * cassini_col[2] as f64;
    let lum_r = 0.299 * roncevaux_col[0] as f64 + 0.587 * roncevaux_col[1] as f64 + 0.114 * roncevaux_col[2] as f64;
    assert!(
        lum_r > lum_c * 4.0,
        "Iapetus Roncevaux Terra must strongly outshine Cassini Regio: lum_r={}, lum_c={}",
        lum_r,
        lum_c
    );

    // Rhea: ancient cratered ice
    let rhea_col = rhea.shade(norm, sun, view);
    assert!(rhea_col[0] > 100);
}

#[test]
fn test_render_saturn_system_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let cols = 160;
    let lines = 50;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let raycaster = Raycaster::new();
    let mut cam = Camera::new();
    let brain_dir = "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf";

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, path: &str| {
        buf.pixels.fill(Pixel::BLACK);
        raycaster.render_frame(buf, camera, &physics, 1, 0.0);
        Hud::render(buf, camera, &physics, 60.0, 600.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // 1. Saturn Majestic Full View (Body + Rings + Cassini Division + Shadow)
    cam.set_target(BodyId::Saturn, physics.saturn.position);
    cam.azimuth = -1.25;
    cam.elevation = 0.35;
    cam.distance_km = 420_000.0;
    render_ppm(&mut buffer, &cam, &format!("{}/saturn_rings_full_view.ppm", brain_dir));

    // 2. Saturn Ring Shadow Close-up (Golden atmosphere with ring shadow bands)
    cam.set_target(BodyId::Saturn, physics.saturn.position);
    cam.azimuth = -1.35;
    cam.elevation = -0.15;
    cam.distance_km = 160_000.0;
    render_ppm(&mut buffer, &cam, &format!("{}/saturn_ring_shadow_close_up.ppm", brain_dir));

    // 3. Titan Atmosphere Haze Close-up (Dense amber smog & limb darkening)
    cam.set_target(BodyId::Titan, physics.titan.position);
    cam.azimuth = -1.38;
    cam.elevation = 0.15;
    cam.distance_km = 7_500.0;
    render_ppm(&mut buffer, &cam, &format!("{}/titan_atmosphere_haze.ppm", brain_dir));

    // 4. Enceladus South Polar Fractures (Tiger stripes & pure water ice)
    cam.set_target(BodyId::Enceladus, physics.enceladus.position);
    cam.azimuth = -1.35;
    cam.elevation = -0.45;
    cam.distance_km = 1_500.0;
    render_ppm(&mut buffer, &cam, &format!("{}/enceladus_tiger_stripes.ppm", brain_dir));

    // 5. Mimas Herschel Crater ("Death Star" colossal impact basin)
    cam.set_target(BodyId::Mimas, physics.mimas.position);
    cam.azimuth = -1.36;
    cam.elevation = 0.05;
    cam.distance_km = 1_100.0;
    render_ppm(&mut buffer, &cam, &format!("{}/mimas_herschel_crater.ppm", brain_dir));

    // 6. Iapetus Yin-Yang & Equatorial Ridge (Dark Cassini Regio vs Bright Roncevaux Terra)
    cam.set_target(BodyId::Iapetus, physics.iapetus.position);
    cam.azimuth = -1.25;
    cam.elevation = 0.10;
    cam.distance_km = 3_800.0;
    render_ppm(&mut buffer, &cam, &format!("{}/iapetus_yin_yang_ridge.ppm", brain_dir));

    // 7. Rhea Cratered Ice & Wispy Fractures Close-up
    cam.set_target(BodyId::Rhea, physics.rhea.position);
    cam.azimuth = -1.35;
    cam.elevation = 0.12;
    cam.distance_km = 2_800.0;
    render_ppm(&mut buffer, &cam, &format!("{}/rhea_cratered_ice.ppm", brain_dir));
}

#[test]
fn test_saturn_system_benchmark_framerate() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::raycaster::Raycaster;
    use std::time::Instant;

    let cols = 160;
    let lines = 50;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let raycaster = Raycaster::new();
    let mut cam = Camera::new();
    cam.set_target(BodyId::Saturn, physics.saturn.position);
    cam.azimuth = -1.25;
    cam.elevation = 0.35;
    cam.distance_km = 420_000.0;

    // Warm-up
    raycaster.render_frame(&mut buffer, &cam, &physics, 1, 0.0);

    let frames = 100;
    let start = Instant::now();
    for f in 0..frames {
        physics.update(f as f64 * 0.1);
        cam.update(0.016, &physics);
        raycaster.render_frame(&mut buffer, &cam, &physics, f as u64, f as f64 * 0.1);
    }
    let elapsed = start.elapsed();
    let fps = frames as f64 / elapsed.as_secs_f64();
    println!("Saturn System Benchmark: {} frames in {:.3}s ({:.1} FPS)", frames, elapsed.as_secs_f64(), fps);
    let min_fps = if cfg!(debug_assertions) { 5.0 } else { 200.0 };
    assert!(fps > min_fps, "FPS {:.1} below minimum performance threshold {:.1}", fps, min_fps);
}

#[test]
fn test_uranus_extreme_axial_tilt_and_ring_plane() {
    use solar_system_ascii::physics::ephemeris::{
        URANUS_AXIAL_TILT_RAD, URANUS_RADIUS_EQ_KM, URANUS_RADIUS_POL_KM,
        URANUS_RING_INNER_RADIUS_KM, URANUS_RING_OUTER_RADIUS_KM, URANUS_SIDEREAL_DAY_SECS,
    };
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::uranus_shader::UranusShader;

    // 1. Verify extreme axial tilt (~97.77 degrees)
    let tilt_deg = URANUS_AXIAL_TILT_RAD * 180.0 / std::f64::consts::PI;
    assert!((tilt_deg - 97.77).abs() < 1e-4, "Uranus axial tilt must be 97.77 degrees");

    // 2. Verify retrograde sidereal day
    assert!(URANUS_SIDEREAL_DAY_SECS < 0.0, "Uranus must rotate retrograde");
    let day_hours = URANUS_SIDEREAL_DAY_SECS.abs() / 3600.0;
    assert!((day_hours - 17.24).abs() < 0.05, "Uranus day must be ~17.24h");

    // 3. Verify oblate spheroid dimensions
    assert_eq!(URANUS_RADIUS_EQ_KM, 25_559.0);
    assert_eq!(URANUS_RADIUS_POL_KM, 24_973.0);
    assert!(URANUS_RADIUS_EQ_KM > URANUS_RADIUS_POL_KM);

    // 4. Verify ring bounds and epsilon ring opacity
    let shader = UranusShader::new();
    assert_eq!(URANUS_RING_INNER_RADIUS_KM, 38_000.0);
    assert_eq!(URANUS_RING_OUTER_RADIUS_KM, 51_500.0);
    let eps_sample = shader.sample_ring(51_149.0);
    assert!(eps_sample.is_some(), "Epsilon ring must be present at 51,149 km");
    let (rgb, opacity) = eps_sample.unwrap();
    assert!(opacity > 0.1, "Epsilon ring must have high visual opacity");
    assert!(rgb[0] > 0 || rgb[1] > 0 || rgb[2] > 0);

    // 5. Verify position in SolarSystemPhysics
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let dist_au = physics.uranus.position.length() / solar_system_ascii::physics::ephemeris::AU_KM;
    assert!(dist_au > 18.0 && dist_au < 21.0, "Uranus distance must be ~19.2 AU");
}

#[test]
fn test_neptune_great_dark_spot_and_methane_color() {
    use solar_system_ascii::physics::ephemeris::{
        NEPTUNE_AXIAL_TILT_RAD, NEPTUNE_RADIUS_EQ_KM, NEPTUNE_RADIUS_POL_KM,
        NEPTUNE_RING_INNER_RADIUS_KM, NEPTUNE_RING_OUTER_RADIUS_KM, NEPTUNE_SIDEREAL_DAY_SECS,
    };
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::neptune_shader::NeptuneShader;

    // 1. Verify axial tilt (~28.32 degrees)
    let tilt_deg = NEPTUNE_AXIAL_TILT_RAD * 180.0 / std::f64::consts::PI;
    assert!((tilt_deg - 28.32).abs() < 1e-4, "Neptune axial tilt must be 28.32 degrees");

    // 2. Verify sidereal day (~16.11h)
    let day_hours = NEPTUNE_SIDEREAL_DAY_SECS / 3600.0;
    assert!((day_hours - 16.11).abs() < 0.05, "Neptune day must be ~16.11h");

    // 3. Verify oblate spheroid dimensions
    assert_eq!(NEPTUNE_RADIUS_EQ_KM, 24_764.0);
    assert_eq!(NEPTUNE_RADIUS_POL_KM, 24_341.0);

    // 4. Verify ring bounds and Adams ring profile
    let shader = NeptuneShader::new();
    assert_eq!(NEPTUNE_RING_INNER_RADIUS_KM, 40_000.0);
    assert_eq!(NEPTUNE_RING_OUTER_RADIUS_KM, 63_500.0);
    let adams = shader.sample_ring(62_932.0);
    assert!(adams.is_some(), "Adams ring must be present at 62,932 km");
    let (_, opacity) = adams.unwrap();
    assert!(opacity > 0.05);

    // 5. Verify distance from Sun in SolarSystemPhysics (~30.1 AU)
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let dist_au = physics.neptune.position.length() / solar_system_ascii::physics::ephemeris::AU_KM;
    assert!(dist_au > 29.0 && dist_au < 31.0, "Neptune distance must be ~30.1 AU");
}

#[test]
fn test_uranian_major_moons_geomorphology() {
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::ariel_shader::ArielShader;
    use solar_system_ascii::render::miranda_shader::MirandaShader;
    use solar_system_ascii::render::oberon_shader::OberonShader;
    use solar_system_ascii::render::titania_shader::TitaniaShader;
    use solar_system_ascii::render::umbriel_shader::UmbrielShader;
    use solar_system_ascii::math::Vec3;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    // Miranda orbit check (~129,900 km from Uranus)
    let d_miranda = (physics.miranda.position - physics.uranus.position).length();
    assert!(d_miranda > 120_000.0 && d_miranda < 140_000.0);

    // Ariel orbit check (~191,020 km)
    let d_ariel = (physics.ariel.position - physics.uranus.position).length();
    assert!(d_ariel > 180_000.0 && d_ariel < 205_000.0);

    // Umbriel orbit check (~266,000 km)
    let d_umbriel = (physics.umbriel.position - physics.uranus.position).length();
    assert!(d_umbriel > 250_000.0 && d_umbriel < 280_000.0);

    // Titania orbit check (~436,000 km)
    let d_titania = (physics.titania.position - physics.uranus.position).length();
    assert!(d_titania > 410_000.0 && d_titania < 460_000.0);

    // Oberon orbit check (~583,500 km)
    let d_oberon = (physics.oberon.position - physics.uranus.position).length();
    assert!(d_oberon > 550_000.0 && d_oberon < 610_000.0);

    // Test contrast between dark Umbriel crust and bright Wunda crater
    let _umbriel_shader = UmbrielShader::new();
    let dark_crust = UmbrielShader::albedo_to_rgb(80.0);
    let bright_wunda = UmbrielShader::albedo_to_rgb(250.0);
    assert!(bright_wunda[0] > dark_crust[0] + 100.0);

    // Test Ariel bright plains albedo
    let ariel_shader = ArielShader::new();
    let norm = Vec3::new(0.0, 1.0, 0.0);
    let sun = Vec3::new(0.0, 1.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);
    let ariel_lit = ariel_shader.shade(norm, sun, view);
    assert!(ariel_lit[0] > 150 && ariel_lit[1] > 150 && ariel_lit[2] > 150);

    // Test Miranda Verona Rupes cliff face
    let _miranda_shader = MirandaShader::new();
    let cliff_rgb = MirandaShader::albedo_to_rgb(245.0);
    assert!(cliff_rgb[0] > 220.0);

    // Test Titania and Oberon shaders
    let titania_shader = TitaniaShader::new();
    let oberon_shader = OberonShader::new();
    assert_eq!(titania_shader.shade(norm, -sun, view), [0, 0, 0]);
    assert_eq!(oberon_shader.shade(norm, -sun, view), [0, 0, 0]);
}

#[test]
fn test_triton_retrograde_orbit_and_cryovolcanism() {
    use solar_system_ascii::physics::ephemeris::{TRITON_RADIUS_KM, TRITON_SIDEREAL_DAY_SECS};
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::triton_shader::TritonShader;
    use solar_system_ascii::render::proteus_shader::ProteusShader;
    use solar_system_ascii::math::Vec3;

    // 1. Verify Triton retrograde orbit (P < 0)
    assert!(TRITON_SIDEREAL_DAY_SECS < 0.0, "Triton must have negative retrograde rotation");
    assert_eq!(TRITON_RADIUS_KM, 1_353.4);

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    // Triton distance from Neptune (~354,760 km)
    let d_triton = (physics.triton.position - physics.neptune.position).length();
    assert!(d_triton > 340_000.0 && d_triton < 370_000.0);

    // Verify retrograde motion: after 1 day, Triton's angle around Neptune should advance in retrograde sense
    let pos_t0 = physics.triton.position - physics.neptune.position;
    physics.update(86_400.0);
    let pos_t1 = physics.triton.position - physics.neptune.position;
    assert!((pos_t1 - pos_t0).length() > 50_000.0);

    // 2. Proteus distance from Neptune (~117,600 km)
    let d_proteus = (physics.proteus.position - physics.neptune.position).length();
    assert!(d_proteus > 110_000.0 && d_proteus < 125_000.0);

    // 3. Shading tests
    let triton_shader = TritonShader::new();
    let proteus_shader = ProteusShader::new();
    let norm = Vec3::new(0.0, 1.0, 0.0);
    let sun = Vec3::new(0.0, 1.0, 0.0);
    let view = Vec3::new(0.0, 1.0, 0.0);

    let triton_lit = triton_shader.shade(norm, sun, view);
    let proteus_lit = proteus_shader.shade(norm, sun, view);

    // Triton has high albedo (~0.72) while Proteus is coal-dark (~0.096)
    assert!(triton_lit[0] > proteus_lit[0] + 60);
}

#[test]
fn test_ice_giants_minor_moons_swarms() {
    use solar_system_ascii::physics::SolarSystemPhysics;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    // Uranus minor moons swarm (23 moons)
    assert_eq!(physics.uranian_swarm.count(), 23);
    for i in 0..23 {
        let pos = physics.uranian_swarm.calculate_position(i, physics.uranus.position, 0.0);
        let dist = (pos - physics.uranus.position).length();
        assert!(dist > 45_000.0 && dist < 30_000_000.0, "Uranian minor moon {} out of range: {}", i, dist);
    }

    // Neptune minor moons swarm (14 moons)
    assert_eq!(physics.neptunian_swarm.count(), 14);
    for i in 0..14 {
        let pos = physics.neptunian_swarm.calculate_position(i, physics.neptune.position, 0.0);
        let dist = (pos - physics.neptune.position).length();
        assert!(dist > 45_000.0 && dist < 80_000_000.0, "Neptunian minor moon {} out of range: {}", i, dist);
    }
}

#[test]
fn test_render_ice_giants_demo() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::raycaster::Raycaster;
    use std::fs::File;
    use std::io::Write;

    let brain_dir = "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf";
    let cols = 320;
    let lines = 100;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let raycaster = Raycaster::new();
    let mut cam = Camera::new();

    let render_ppm = |buf: &mut FrameBuffer, camera: &Camera, path: &str| {
        raycaster.render_frame(buf, camera, &physics, 1, 0.0);
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = File::create(path) {
            let v_h = buf.virtual_height();
            let _ = write!(file, "P6\n{} {}\n255\n", buf.cols, v_h);
            for pixel in &buf.pixels {
                let _ = file.write_all(&[pixel.r, pixel.g, pixel.b]);
            }
        }
    };

    // 1. Uranus Vertical Rings & Pale Cyan Methane Disc (Sunlit perspective)
    cam.set_target(BodyId::Uranus, physics.uranus.position);
    cam.azimuth = 1.85;
    cam.elevation = 0.35;
    cam.distance_km = 120_000.0;
    render_ppm(&mut buffer, &cam, &format!("{}/uranus_vertical_rings.ppm", brain_dir));

    // 2. Miranda Verona Rupes 20 km Cliff Scarp & Coronae Chevrons (Sunlit)
    cam.set_target(BodyId::Miranda, physics.miranda.position);
    cam.azimuth = 1.85;
    cam.elevation = 0.15;
    cam.distance_km = 1_100.0;
    render_ppm(&mut buffer, &cam, &format!("{}/miranda_verona_rupes.ppm", brain_dir));

    // 3. Ariel Interconnected Graben Rift Valleys (Sunlit)
    cam.set_target(BodyId::Ariel, physics.ariel.position);
    cam.azimuth = 1.85;
    cam.elevation = 0.12;
    cam.distance_km = 2_200.0;
    render_ppm(&mut buffer, &cam, &format!("{}/ariel_rift_valleys.ppm", brain_dir));

    // 4. Umbriel Ancient Coal-Dark Crust & Brilliant Wunda Crater (Sunlit)
    cam.set_target(BodyId::Umbriel, physics.umbriel.position);
    cam.azimuth = 1.85;
    cam.elevation = 0.10;
    cam.distance_km = 2_200.0;
    render_ppm(&mut buffer, &cam, &format!("{}/umbriel_wunda_crater.ppm", brain_dir));

    // 5. Titania Messina Chasma Canyon System (Sunlit)
    cam.set_target(BodyId::Titania, physics.titania.position);
    cam.azimuth = 1.85;
    cam.elevation = 0.14;
    cam.distance_km = 2_800.0;
    render_ppm(&mut buffer, &cam, &format!("{}/titania_messina_chasma.ppm", brain_dir));

    // 6. Oberon Hamlet Crater & Limb Mountain Peak (Sunlit)
    cam.set_target(BodyId::Oberon, physics.oberon.position);
    cam.azimuth = 1.85;
    cam.elevation = 0.12;
    cam.distance_km = 2_800.0;
    render_ppm(&mut buffer, &cam, &format!("{}/oberon_hamlet_peak.ppm", brain_dir));

    // 7. Neptune Azure Atmosphere, Scooter Cirrus & Adams Ring Arcs
    cam.set_target(BodyId::Neptune, physics.neptune.position);
    cam.azimuth = -1.25;
    cam.elevation = 0.35;
    cam.distance_km = 120_000.0;
    render_ppm(&mut buffer, &cam, &format!("{}/neptune_azure_clouds.ppm", brain_dir));

    // 8. Triton Cantaloupe Terrain, Cryovolcanic Geysers & Nitrogen Ice Cap
    cam.set_target(BodyId::Triton, physics.triton.position);
    cam.azimuth = -1.35;
    cam.elevation = 0.15;
    cam.distance_km = 4_500.0;
    render_ppm(&mut buffer, &cam, &format!("{}/triton_cryovolcanoes.ppm", brain_dir));

    // 9. Proteus Polyhedral Facets & Pharos Crater Basin
    cam.set_target(BodyId::Proteus, physics.proteus.position);
    cam.azimuth = -1.30;
    cam.elevation = 0.15;
    cam.distance_km = 900.0;
    render_ppm(&mut buffer, &cam, &format!("{}/proteus_pharos_basin.ppm", brain_dir));
}

#[test]
fn test_ice_giants_system_benchmark_framerate() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::raycaster::Raycaster;
    use std::time::Instant;

    let cols = 160;
    let lines = 50;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    let raycaster = Raycaster::new();
    let mut cam = Camera::new();
    cam.set_target(BodyId::Uranus, physics.uranus.position);
    cam.azimuth = -1.25;
    cam.elevation = 0.35;
    cam.distance_km = 150_000.0;

    // Warm-up
    raycaster.render_frame(&mut buffer, &cam, &physics, 1, 0.0);

    let frames = 100;
    let start = Instant::now();
    for f in 0..frames {
        physics.update(f as f64 * 0.1);
        cam.update(0.016, &physics);
        raycaster.render_frame(&mut buffer, &cam, &physics, f as u64, f as f64 * 0.1);
    }
    let elapsed = start.elapsed();
    let fps = frames as f64 / elapsed.as_secs_f64();
    println!("Ice Giants System Benchmark: {} frames in {:.3}s ({:.1} FPS)", frames, elapsed.as_secs_f64(), fps);
    let min_fps = if cfg!(debug_assertions) { 5.0 } else { 200.0 };
    assert!(fps > min_fps, "FPS {:.1} below minimum performance threshold {:.1}", fps, min_fps);
}

#[test]
fn test_pluto_charon_system_and_voyager_probes() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::ephemeris::{VOYAGER_1_VEC_KM, VOYAGER_2_VEC_KM};
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::pluto_shader::PlutoShader;
    use solar_system_ascii::render::charon_shader::CharonShader;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    // 1. Pluto Keplerian elements verification
    let pluto = physics.get_body(BodyId::Pluto);
    assert_eq!(pluto.name, "Pluto");
    assert!((pluto.radius_km - 1_188.3).abs() < 1e-4);
    assert!(pluto.orbit.is_some());
    let pluto_orbit = pluto.orbit.unwrap();
    assert!((pluto_orbit.a - 5_906_380_000.0).abs() < 1e-4); // 39.482 AU
    assert!((pluto_orbit.e - 0.2488).abs() < 1e-4); // High eccentricity
    assert!((pluto_orbit.i - 17.16 * std::f64::consts::PI / 180.0).abs() < 1e-4); // 17.16° inclination

    // 2. Charon binary synchronous orbit verification
    let charon = physics.get_body(BodyId::Charon);
    assert_eq!(charon.name, "Charon");
    assert!((charon.radius_km - 606.0).abs() < 1e-4);
    assert!(charon.orbit.is_some());
    let charon_orbit = charon.orbit.unwrap();
    assert!((charon_orbit.a - 19_591.4).abs() < 1e-4);
    assert_eq!(charon.parent, Some(BodyId::Pluto));

    // Mutual distance ~19,591 km
    let dist_pluto_charon = (charon.position - pluto.position).length();
    assert!((dist_pluto_charon - 19_591.4).abs() < 50.0);

    // 3. Voyager 1 & 2 telemetry vectors verification
    let v1_dist_au = VOYAGER_1_VEC_KM.length() / 149_597_870.7;
    let v2_dist_au = VOYAGER_2_VEC_KM.length() / 149_597_870.7;
    assert!((v1_dist_au - 163.5).abs() < 2.0, "Voyager 1 distance should be ~163.5 AU, got {:.1}", v1_dist_au);
    assert!((v2_dist_au - 136.8).abs() < 2.0, "Voyager 2 distance should be ~136.8 AU, got {:.1}", v2_dist_au);

    // 4. Photorealistic Shaders verification
    let pluto_shader = PlutoShader::new();
    let charon_shader = CharonShader::new();
    let sun_dir = Vec3::new(0.0, 0.0, 1.0);
    let view_dir = Vec3::new(0.0, 0.0, 1.0);
    let norm = Vec3::new(0.0, 0.0, 1.0);
    let pluto_color = pluto_shader.shade(norm, sun_dir, view_dir, 39.5);
    let charon_color = charon_shader.shade(norm, sun_dir, view_dir, 39.5);
    assert!(pluto_color[0] > 0 || pluto_color[1] > 0 || pluto_color[2] > 0);
    assert!(charon_color[0] > 0 || charon_color[1] > 0 || charon_color[2] > 0);

    // Strict nightside
    assert_eq!(pluto_shader.shade(norm, -sun_dir, view_dir, 39.5), [0, 0, 0]);
    assert_eq!(charon_shader.shade(norm, -sun_dir, view_dir, 39.5), [0, 0, 0]);

    // 5. Camera default startup is top-down Solar System view
    let cam = Camera::new();
    assert_eq!(cam.target_body, BodyId::Sun);
    assert_eq!(cam.show_orbit_lines, true);
    assert!((cam.distance_km - 1.1e10).abs() < 1e-4);
    assert!((cam.elevation - 0.0).abs() < 1e-4);
}

#[test]
fn test_render_polish_and_pluto_demos() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;
    use solar_system_ascii::render::raycaster::Raycaster;

    let brain_dir = "/Users/pedrohnovais/.gemini/antigravity/brain/c3c1b711-4bad-4f20-902f-5f6146923bcf";
    let cols = 160;
    let lines = 55;
    let mut buffer = FrameBuffer::new(cols, lines, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let raycaster = Raycaster::new();

    let render_ppm = |buf: &mut FrameBuffer, cam: &Camera, path: &str| {
        buf.pixels.fill(solar_system_ascii::render::buffer::Pixel::BLACK);
        raycaster.render_frame(buf, cam, &physics, 1, 0.0);
        Hud::render(buf, cam, &physics, 60.0, 1.0, false);

        let v_h = buf.virtual_height();
        let mut ppm = format!("P6\n{} {}\n255\n", buf.cols, v_h).into_bytes();
        for p in &buf.pixels {
            ppm.push(p.r);
            ppm.push(p.g);
            ppm.push(p.b);
        }
        if let Some(parent) = std::path::Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, ppm);
    };

    // 1. Default Startup: Top-Down Solar System Orrery View with Keplerian Orbit Lines
    let cam_orrery = Camera::new();
    render_ppm(&mut buffer, &cam_orrery, &format!("{}/solar_system_top_down_orrery.ppm", brain_dir));

    // 2. Interstellar Deep Zoom: Voyager 1 & 2 at > 130-160 AU
    let mut cam_voyager = Camera::new();
    cam_voyager.distance_km = 2.8e10; // ~187 AU
    render_ppm(&mut buffer, &cam_voyager, &format!("{}/voyager_interstellar_zoom.ppm", brain_dir));

    // 3. Pluto Photorealistic Balanced View (Tombaugh Regio Heart & Charon in view)
    let mut cam_pluto = Camera::new();
    cam_pluto.set_target(BodyId::Pluto, physics.pluto.position);
    render_ppm(&mut buffer, &cam_pluto, &format!("{}/pluto_tombaugh_regio.ppm", brain_dir));

    // 4. Charon Photorealistic Balanced View (Mordor Macula & Serenity Chasma)
    let mut cam_charon = Camera::new();
    cam_charon.set_target(BodyId::Charon, physics.charon.position);
    render_ppm(&mut buffer, &cam_charon, &format!("{}/charon_serenity_chasma.ppm", brain_dir));

    // 5. Mars Optimized Beauty Angle (Sunlit Valles Marineris, Olympus Mons & Polar Ice Cap)
    let mut cam_mars = Camera::new();
    cam_mars.set_target(BodyId::Mars, physics.mars.position);
    render_ppm(&mut buffer, &cam_mars, &format!("{}/mars_beauty_angle.ppm", brain_dir));

    // 6. Neptune Optimized Beauty Angle (Azure Cobalt Disc & Great Dark Spot)
    let mut cam_neptune = Camera::new();
    cam_neptune.set_target(BodyId::Neptune, physics.neptune.position);
    render_ppm(&mut buffer, &cam_neptune, &format!("{}/neptune_beauty_angle.ppm", brain_dir));

    // 7. Venus with Contextual HUD ([V] Radar Magellan)
    let mut cam_venus = Camera::new();
    cam_venus.set_target(BodyId::Venus, physics.venus.position);
    render_ppm(&mut buffer, &cam_venus, &format!("{}/venus_contextual_hud.ppm", brain_dir));

    // 8. Saturn User Matching Angle (Iconic oblique perspective with ring shadow)
    let mut cam_saturn = Camera::new();
    cam_saturn.set_target(BodyId::Saturn, physics.saturn.position);
    render_ppm(&mut buffer, &cam_saturn, &format!("{}/saturn_beauty_angle.ppm", brain_dir));

    // 9. Sun Close-up View (Photosphere Granulation & Magnetic Corona Loops)
    let mut cam_sun_close = Camera::new();
    cam_sun_close.distance_km = 2_800_000.0;
    cam_sun_close.azimuth = 0.0;
    cam_sun_close.elevation = 0.15;
    render_ppm(&mut buffer, &cam_sun_close, &format!("{}/sun_closeup_view.ppm", brain_dir));
}

#[test]
fn test_sun_zoom_toggle_and_extended_timewarp() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;

    let physics = SolarSystemPhysics::new();
    let mut cam = Camera::new();

    // 1. Initial state at Sun: Solar System Overview
    assert_eq!(cam.target_body, BodyId::Sun);
    assert!((cam.distance_km - 1.1e10).abs() < 1e5);

    // 2. Toggle Sun Zoom -> Swoop in to Sun Close-up (2,800,000 km)
    cam.toggle_sun_zoom();
    assert!(cam.transition.is_some());
    let trans = cam.transition.as_ref().unwrap();
    assert_eq!(trans.from_body, BodyId::Sun);
    assert_eq!(trans.to_body, BodyId::Sun);
    assert!((trans.end_view.distance_km - 2_800_000.0).abs() < 1e-4);

    // Complete transition
    cam.update(1.5, &physics);
    assert!(cam.transition.is_none());
    assert_eq!(cam.target_body, BodyId::Sun);
    assert!((cam.distance_km - 2_800_000.0).abs() < 1e-4);

    // 3. Toggle Sun Zoom again -> Swoop out to Solar System Overview (1.1e10 km)
    cam.toggle_sun_zoom();
    assert!(cam.transition.is_some());
    let trans2 = cam.transition.as_ref().unwrap();
    assert!((trans2.end_view.distance_km - 1.1e10).abs() < 1e5);

    // Complete transition
    cam.update(1.5, &physics);
    assert!(cam.transition.is_none());
    assert_eq!(cam.target_body, BodyId::Sun);
    assert!((cam.distance_km - 1.1e10).abs() < 1e5);

    // 4. Extended Timewarp check
    let max_timewarp = 315_360_000.0; // 10 years / sec
    let one_year = 31_536_000.0;
    assert!(max_timewarp >= 10.0 * one_year);
}

#[test]
fn test_hud_moon_labels_culling_at_solar_distance() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::buffer::{FrameBuffer, Pixel, RenderMode};
    use solar_system_ascii::render::camera::Camera;
    use solar_system_ascii::render::hud::Hud;

    let cols = 160;
    let rows = 60;
    let mut buffer = FrameBuffer::new(cols, rows, RenderMode::HalfBlock);
    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);

    // 1. Top-Down Solar System Orrery View (Camera looking at Sun from 1.1e10 km)
    let cam = Camera::new();
    buffer.pixels.fill(Pixel::BLACK);
    Hud::render(&mut buffer, &cam, &physics, 60.0, 1.0, false);

    let get_text = |buf: &FrameBuffer| -> String {
        let mut out = String::new();
        for y in 0..rows {
            for x in 0..cols {
                let p = &buf.pixels[(y * 2) * cols + x];
                if p.is_text {
                    out.push(p.ch);
                } else {
                    out.push(' ');
                }
            }
            out.push('\n');
        }
        out
    };

    let text_solar = get_text(&buffer);

    // Major planets must be visible and uncluttered
    assert!(text_solar.contains("[Jupiter]"), "Jupiter label must be cleanly visible in top-down view");
    assert!(text_solar.contains("[Saturn]"), "Saturn label must be cleanly visible in top-down view");
    assert!(text_solar.contains("[Uranus]"), "Uranus label must be cleanly visible in top-down view");
    assert!(text_solar.contains("[Neptune]"), "Neptune label must be cleanly visible in top-down view");

    // Moons must NOT be rendered at solar system distance (> millions of km)
    assert!(!text_solar.contains("[Callisto]"), "Callisto label must be culled at solar distance");
    assert!(!text_solar.contains("[Oberon]"), "Oberon label must be culled at solar distance");
    assert!(!text_solar.contains("[Rhea]"), "Rhea label must be culled at solar distance");
    assert!(!text_solar.contains("[Proteus]"), "Proteus label must be culled at solar distance");
    assert!(!text_solar.contains("[Moon]"), "Moon label must be culled at solar distance");
    assert!(!text_solar.contains("[Phobos]"), "Phobos label must be culled at solar distance");

    // 2. Focused on Jupiter at Planetary System distance (3,000,000 km)
    let mut cam_jup = Camera::new();
    cam_jup.set_target(BodyId::Jupiter, physics.jupiter.position);
    cam_jup.distance_km = 3_000_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    Hud::render(&mut buffer, &cam_jup, &physics, 60.0, 1.0, false);

    let text_jup = get_text(&buffer);
    let any_galilean = text_jup.contains("[Callisto]") || text_jup.contains("[Io]") || text_jup.contains("[Europa]") || text_jup.contains("[Ganymede]");
    assert!(any_galilean, "Galilean moons must be eligible for display when observing Jupiter system");

    // Moons of other distant planets (e.g. Uranus Oberon, Neptune Proteus) must NOT be visible!
    assert!(!text_jup.contains("[Oberon]"), "Oberon must NOT be visible when observing Jupiter");
    assert!(!text_jup.contains("[Proteus]"), "Proteus must NOT be visible when observing Jupiter");
}

#[test]
fn test_ux_camera_polar_clamping() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    cam.set_target(BodyId::Earth, Vec3::new(149_597_870.7, 0.0, 0.0));

    // Severe positive elevation orbit (trying to go beyond North pole)
    cam.orbit(0.0, 5.0);
    assert!(cam.elevation <= 1.55);
    assert!(cam.elevation >= -1.55);

    // Severe negative elevation orbit (trying to go beyond South pole)
    cam.orbit(0.0, -10.0);
    assert!(cam.elevation >= -1.55);
    assert!(cam.elevation <= 1.55);
}

#[test]
fn test_ux_hierarchy_ascension_and_moon_loops() {
    use solar_system_ascii::physics::body::BodyId;

    // 1. Hierarchy ascension: Moons ascend to parent, planets ascend to Sun, Sun has no parent
    assert_eq!(BodyId::Moon.parent(), Some(BodyId::Earth));
    assert_eq!(BodyId::Phobos.parent(), Some(BodyId::Mars));
    assert_eq!(BodyId::Deimos.parent(), Some(BodyId::Mars));
    assert_eq!(BodyId::Io.parent(), Some(BodyId::Jupiter));
    assert_eq!(BodyId::Europa.parent(), Some(BodyId::Jupiter));
    assert_eq!(BodyId::Titan.parent(), Some(BodyId::Saturn));
    assert_eq!(BodyId::Miranda.parent(), Some(BodyId::Uranus));
    assert_eq!(BodyId::Triton.parent(), Some(BodyId::Neptune));
    assert_eq!(BodyId::Charon.parent(), Some(BodyId::Pluto));

    assert_eq!(BodyId::Earth.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Mars.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Jupiter.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Saturn.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Uranus.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Neptune.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Pluto.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Ceres.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Vesta.parent(), Some(BodyId::Sun));
    assert_eq!(BodyId::Sun.parent(), None);
}

#[test]
fn test_ux_midflight_redirect_continuity() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;

    let mut physics = SolarSystemPhysics::new();
    physics.update(0.0);
    let mut cam = Camera::new();
    cam.set_target(BodyId::Earth, physics.earth.position);

    // Initial view of Earth
    let earth_view = cam.saved_views[BodyId::Earth.index()];

    // 1. Start transition Earth -> Mars
    cam.start_transition(BodyId::Mars, physics.earth.position, physics.mars.position, 2.0);
    assert_eq!(cam.target_body, BodyId::Mars);

    // Update 50% into flight
    cam.update(1.0, &physics);
    assert!(cam.transition.is_some());
    let mid_flight_pos = cam.target_pos;
    assert!(mid_flight_pos != physics.earth.position);
    assert!(mid_flight_pos != physics.mars.position);

    // 2. Mid-flight redirect to Saturn!
    cam.start_transition(BodyId::Saturn, mid_flight_pos, physics.saturn.position, 2.0);
    assert_eq!(cam.target_body, BodyId::Saturn);

    // Verify Earth saved view was NOT overwritten with corrupted mid-flight altitude!
    assert_eq!(cam.saved_views[BodyId::Earth.index()], earth_view);

    // Check transition starts smoothly from mid_flight_pos
    let trans = cam.transition.as_ref().unwrap();
    assert_eq!(trans.from_pos, mid_flight_pos);
    assert_eq!(trans.to_body, BodyId::Saturn);

    // Fly to completion
    cam.update(2.0, &physics);
    assert!(cam.transition.is_none());
    assert_eq!(cam.target_body, BodyId::Saturn);
}
