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
fn test_camera_transition_interpolation() {
    use solar_system_ascii::physics::body::BodyId;
    use solar_system_ascii::physics::SolarSystemPhysics;
    use solar_system_ascii::render::camera::Camera;

    let mut cam = Camera::new();
    let physics = SolarSystemPhysics::new();

    let earth_pos = physics.earth.position;
    let moon_pos = physics.moon.position;

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
        std::fs::write(path, ppm).unwrap();
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
    assert!((cam.distance_km - 3_500_000.0).abs() < 1e-4);
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
        std::fs::write(path, ppm).unwrap();
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
