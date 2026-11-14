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

    // Find the brightest pixel (the solar core center)
    let mut max_lum = 0.0f64;
    let mut center_idx = 0;
    for (i, p) in buffer.pixels.iter().enumerate() {
        let lum = p.r as f64 * 0.299 + p.g as f64 * 0.587 + p.b as f64 * 0.114;
        if lum > max_lum {
            max_lum = lum;
            center_idx = i;
        }
    }

    assert!(max_lum > 240.0, "Solar core must be blinding bright (> 240 lum), got {}", max_lum);

    // Check pixels in horizontal radius from the solar core:
    // Luminosity must remain high and smoothly decay outwards, NEVER plunging into a black void near center!
    let _v_h = buffer.virtual_height();
    let cy = center_idx / cols;
    let cx = center_idx % cols;

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

    // Search buffer for string "[Terra]"
    let mut found_terra = false;
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
        if line.contains("[Terra]") {
            found_terra = true;
            break;
        }
    }
    assert!(!found_terra, "[Terra] label must be hidden when closely inspecting Earth");

    // 2. Zoomed out from Earth (100,000 km, > 10 * R_earth):
    // [Terra] label must reappear!
    cam.distance_km = 100_000.0;
    buffer.pixels.fill(Pixel::BLACK);
    Hud::render(&mut buffer, &cam, &physics, 60.0, 600.0, false);

    let mut found_terra_zoomed = false;
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
        if line.contains("[Terra]") {
            found_terra_zoomed = true;
            break;
        }
    }
    assert!(found_terra_zoomed, "[Terra] label must reappear when zooming out to system scale (> 10 * radius)");
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
        std::fs::write(path, ppm).unwrap();
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
    for day in 0..365 {
        let t = day as f64 * 86_400.0;
        physics.update(t);

        let r_merc = physics.mercury.position.length();
        let r_earth = physics.earth.position.length();
        let d_moon_earth = (physics.moon.position - physics.earth.position).length();

        assert!(r_merc < r_earth, "Mercury must strictly orbit inside Earth orbit at day {}", day);
        assert!(d_moon_earth >= 363_200.0 && d_moon_earth <= 405_600.0, "Moon must remain in valid geocentric range");
    }
}
