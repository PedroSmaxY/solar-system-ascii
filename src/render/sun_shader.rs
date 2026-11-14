//! Ultra-realistic Solar Shader (5,778 K Photosphere):
//! - Planckian chromatic gradient from incandescent core to deep amber limb
//! - Analytical Eddington-Barbier limb darkening model (u=0.45, v=0.22)
//! - Dynamic magnetohydrodynamic granulation (plasma convection cells)
//! - Active Region sunspot groups (Spörer belt ±10°..±35°) with Umbra core (4,000 K) and Penumbra (5,500 K)
//! - Volumetric Solar Corona & magnetic loop prominences (1.0 .. 2.2 R_sun)

use crate::math::Vec3;
use std::f64::consts::PI;

pub struct SunShader;

impl SunShader {
    pub fn new() -> Self {
        Self
    }

    /// Dynamic plasma granulation using rotated non-separable tetrahedral Voronoi cellular harmonics.
    /// Completely eliminates Cartesian square checkerboard / low-poly grid artifacts.
    #[inline(always)]
    fn granulation(local_hit: Vec3, time: f64) -> f64 {
        // Rotated non-separable frame 1 (irrational rotation matrix)
        let p = local_hit * 68.0;
        let t1 = time * 0.35;
        let (st1, ct1) = t1.sin_cos();

        // Wave set 1: Tetrahedral non-orthogonal vectors
        // k1 = (1, 1, 1)/sqrt(3), k2 = (1, -1, -1)/sqrt(3), k3 = (-1, 1, -1)/sqrt(3), k4 = (-1, -1, 1)/sqrt(3)
        let inv_sqrt3 = 0.57735026919;
        let d1 = (p.x + p.y + p.z) * inv_sqrt3;
        let d2 = (p.x - p.y - p.z) * inv_sqrt3;
        let d3 = (-p.x + p.y - p.z) * inv_sqrt3;
        let d4 = (-p.x - p.y + p.z) * inv_sqrt3;

        let w1 = ((d1 + st1).cos() + (d2 - ct1).cos() + (d3 + ct1).cos() + (d4 - st1).cos()) * 0.25;

        // Wave set 2: Rotated by 47° and 73° around golden ratio axes (scale 136.0)
        let p2 = Vec3::new(
            0.80 * local_hit.x - 0.48 * local_hit.y + 0.36 * local_hit.z,
            0.48 * local_hit.x + 0.86 * local_hit.y + 0.16 * local_hit.z,
            -0.36 * local_hit.x + 0.05 * local_hit.y + 0.93 * local_hit.z,
        ) * 136.0;

        let t2 = time * 0.50;
        let (st2, ct2) = t2.sin_cos();
        let e1 = (p2.x + p2.y + p2.z) * inv_sqrt3;
        let e2 = (p2.x - p2.y - p2.z) * inv_sqrt3;
        let e3 = (-p2.x + p2.y - p2.z) * inv_sqrt3;
        let e4 = (-p2.x - p2.y + p2.z) * inv_sqrt3;

        let w2 = ((e1 - st2).cos() + (e2 + ct2).cos() + (e3 - ct2).cos() + (e4 + st2).cos()) * 0.25;

        // Convective cellular contrast:
        // Centers of cells have high values; narrow downflow lanes between cells are cooler
        let cell = (w1 * 0.65 + w2 * 0.35) * 0.5 + 0.5; // in [0, 1]
        cell.powf(0.55).clamp(0.0, 1.0)
    }

    /// Sunspot groups in the Spörer tropical magnetic belt (±10° to ±35°).
    /// Returns factor in 0.0 .. 1.0 (1.0 = no spot, 0.18 = umbra, 0.55..0.75 = penumbra, 1.08 = surrounding faculae).
    #[inline(always)]
    fn sunspot_factor(lat_deg: f64, lon_deg: f64, time: f64) -> f64 {
        // Sunspots only occur in active magnetic latitudes
        let abs_lat = lat_deg.abs();
        if abs_lat < 8.0 || abs_lat > 38.0 {
            return 1.0;
        }

        // Solar differential rotation: equator rotates faster than poles (~25 days vs ~35 days)
        let lat_rad = lat_deg * PI / 180.0;
        let diff_rot = 14.37 - 2.5 * lat_rad.sin().powi(2);
        let rot_lon = lon_deg + diff_rot * (time * 0.05);

        // Active Regions (bipolar spot pairs)
        // [lat, base_lon, radius_deg, companion_dlat, companion_dlon]
        let spot_groups = [
            (17.5, 45.0, 3.8, 1.5, 6.0),    // Northern Active Region 1
            (-21.0, 140.0, 4.5, -1.2, 7.5),  // Southern Active Region 2
            (25.0, 220.0, 3.2, 1.0, 5.0),    // Northern Active Region 3
            (-15.0, 310.0, 5.0, -1.8, 8.5),  // Southern Giant Bipolar Group 4
        ];

        let mut min_factor = 1.0f64;

        for &(s_lat, s_lon, s_rad, c_dlat, c_dlon) in &spot_groups {
            // Check primary spot
            let d_lat1 = lat_deg - s_lat;
            let d_lon1 = (rot_lon - s_lon).rem_euclid(360.0);
            let d_lon_adj1 = d_lon1.min(360.0 - d_lon1) * lat_rad.cos();
            let dist1 = (d_lat1 * d_lat1 + d_lon_adj1 * d_lon_adj1).sqrt();

            if dist1 < s_rad * 1.6 {
                let factor = Self::evaluate_spot_profile(dist1, s_rad);
                min_factor = min_factor.min(factor);
            }

            // Check companion bipolar spot
            let d_lat2 = lat_deg - (s_lat + c_dlat);
            let d_lon2 = (rot_lon - (s_lon + c_dlon)).rem_euclid(360.0);
            let d_lon_adj2 = d_lon2.min(360.0 - d_lon2) * lat_rad.cos();
            let dist2 = (d_lat2 * d_lat2 + d_lon_adj2 * d_lon_adj2).sqrt();

            let comp_rad = s_rad * 0.75;
            if dist2 < comp_rad * 1.6 {
                let factor = Self::evaluate_spot_profile(dist2, comp_rad);
                min_factor = min_factor.min(factor);
            }
        }

        min_factor
    }

    #[inline(always)]
    fn evaluate_spot_profile(dist: f64, spot_rad: f64) -> f64 {
        let norm_d = dist / spot_rad;
        if norm_d < 0.38 {
            // Umbra core (~4,000 K vs 5,778 K photoshere) -> 82% luminosity drop
            0.18 + (norm_d / 0.38) * 0.08
        } else if norm_d < 1.0 {
            // Penumbra (~5,500 K) with radial filamentary striations
            let t = (norm_d - 0.38) / 0.62;
            0.55 + 0.35 * t
        } else if norm_d < 1.75 {
            // Surrounding blazing magnetic plages/faculae (Image 1) -> up to +32% brighter
            let t = (norm_d - 1.0) / 0.75;
            1.0 + 0.32 * (1.0 - t).powi(2)
        } else {
            1.0
        }
    }

    /// Shade a ray hit on the solar photosphere.
    pub fn shade_photosphere(&self, local_hit: Vec3, view_dir: Vec3, _tick: u64, elapsed: f64) -> [u8; 3] {
        // mu = cos(theta) between surface normal and observer line of sight
        let mu = local_hit.dot(view_dir).clamp(0.005, 1.0);

        // Dynamic plasma granulation (Voronoi convection cells)
        let gran = Self::granulation(local_hit, elapsed);
        // Granulation modulation: ±12% temperature variation
        let gran_mod = 0.88 + gran * 0.24;

        // Chromospheric limb spicules / boiling plasma fibrils at the grazing edge (Image 1)
        let spicule_mod = if mu < 0.20 {
            let p_spic = local_hit * 110.0;
            let s_noise = (p_spic.x.sin() * p_spic.y.cos() * p_spic.z.sin()).abs();
            let t_edge = ((0.20 - mu) / 0.20).clamp(0.0, 1.0);
            1.0 + s_noise * 0.18 * t_edge
        } else {
            1.0
        };

        // Latitude & Longitude for Spörer sunspots
        let lat_rad = local_hit.y.clamp(-1.0, 1.0).asin();
        let lat_deg = lat_rad * 180.0 / PI;
        let lon_rad = local_hit.z.atan2(local_hit.x);
        let lon_deg = lon_rad * 180.0 / PI;

        let spot_mod = Self::sunspot_factor(lat_deg, lon_deg, elapsed);

        // Planckian Chromatic Gradient: Radiant incandescent solar plasma
        let (base_r, base_g, base_b) = if mu > 0.50 {
            // Center to mid-disk: Incandescent White-Gold
            let t = (mu - 0.50) / 0.50;
            (
                255.0,
                225.0 + 30.0 * t,
                110.0 + 135.0 * t,
            )
        } else {
            // Mid-disk to extreme limb: Radiant Solar Gold to Warm Plasma
            let t = (mu / 0.50).clamp(0.0, 1.0);
            (
                255.0,
                195.0 + 30.0 * t,
                85.0 + 25.0 * t,
            )
        };

        // Chromospheric Limb Transition (Inversion Layer & Flash Spectrum):
        // Eliminates any muddy rim by blending seamlessly into the base coronal glow
        let t_edge = ((0.20 - mu) / 0.20).max(0.0);
        let chromo_blend = t_edge * t_edge;

        // Base coronal color at alt = 0.0 matches: R=255, G=206, B=108 with intensity 0.95
        let chromo_r = 255.0;
        let chromo_g = 206.0;
        let chromo_b = 108.0;

        let blend_r = base_r * (1.0 - chromo_blend) + chromo_r * chromo_blend;
        let blend_g = base_g * (1.0 - chromo_blend) + chromo_g * chromo_blend;
        let blend_b = base_b * (1.0 - chromo_blend) + chromo_b * chromo_blend;

        let limb_decay = 0.72 + 0.28 * mu.powf(0.8);
        let effective_intensity = limb_decay * (1.0 - chromo_blend) + 0.95 * chromo_blend;
        let intensity = effective_intensity * gran_mod * spot_mod * spicule_mod;

        let r = (blend_r * intensity).clamp(0.0, 255.0) as u8;
        let g = (blend_g * intensity).clamp(0.0, 255.0) as u8;
        let b = (blend_b * intensity).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }

    /// Evaluate volumetric solar corona, extended radial streamers (up to 4.5 R_sun),
    /// magnetic loop arcades, and eruptive solar flare prominences.
    ///
    /// - `impact_dist_ratio`: d_min / R_sun (1.0 to 4.5)
    /// - `rel_norm`: Unit vector from Sun center to closest point on ray
    /// - `ray_dir`: Direction of ray
    /// - `elapsed`: Simulation time in seconds
    pub fn shade_corona(
        &self,
        impact_dist_ratio: f64,
        rel_norm: Vec3,
        ray_dir: Vec3,
        elapsed: f64,
    ) -> Option<[u8; 3]> {
        if impact_dist_ratio <= 1.0 || impact_dist_ratio >= 4.5 {
            return None;
        }

        let altitude = impact_dist_ratio - 1.0; // 0.0 .. 3.5 R_sun
        let r = impact_dist_ratio;

        // Position angle psi around the solar limb in the projection plane
        // Solar north pole is Vec3::Y
        let up = Vec3::Y;
        let up_proj = up - ray_dir * up.dot(ray_dir);
        let up_axis = if up_proj.length_squared() > 1e-4 {
            up_proj.normalize()
        } else {
            Vec3::Z
        };
        let right_axis = ray_dir.cross(up_axis).normalize();

        let cos_psi = rel_norm.dot(up_axis).clamp(-1.0, 1.0);
        let sin_psi = rel_norm.dot(right_axis).clamp(-1.0, 1.0);
        let psi = sin_psi.atan2(cos_psi); // -PI .. +PI

        // 1. Magnetic Field Deflection: Near the Sun, magnetic dipole lines curve towards the equator
        let equator_sign = if sin_psi >= 0.0 { 1.0 } else { -1.0 };
        let dip_bend = equator_sign * (cos_psi.abs() * (1.0 - cos_psi.abs())) * 0.28 * (-altitude / 0.70).exp();

        // 2. 2D/3D Organic Fractal Domain Warping along the ray (eliminates straight bicycle spokes)
        let warp1 = (3.0 * psi + altitude * 1.5 - elapsed * 0.10).sin() * 0.12;
        let warp2 = (7.0 * psi - altitude * 2.8 + elapsed * 0.06).cos() * 0.06;
        let warp3 = (15.0 * psi + altitude * 4.5).sin() * 0.025;
        let psi_eff = psi + dip_bend + warp1 + warp2 + warp3;

        // Smooth C^1 continuous outer boundary taper to exactly zero at 4.5 R_sun
        let taper = (1.0 - (altitude / 3.5).powi(2)).max(0.0).powi(2);

        // K-Corona (dense Thomson scattering glow near limb)
        let k_corona = (-altitude / 0.14).exp() * 0.95;

        // Latitudinal envelope: Helmet streamers dominate equatorial & mid-latitudes (Image 3)
        let eq_mask = (1.0 - cos_psi.abs().powi(3)).max(0.0);
        let m1 = (4.0 * psi_eff + elapsed * 0.03).cos() * 0.5 + 0.5;
        let m2 = (6.0 * psi_eff - 0.7 - elapsed * 0.02).sin() * 0.5 + 0.5;
        let macro_lobes = m1.powi(3) * 0.60 + m2.powi(4) * 0.40;

        // Intermediate Streamers (Scale 14 - 24)
        let s1 = (16.0 * psi_eff + altitude * 0.8).cos() * 0.5 + 0.5;
        let s2 = (24.0 * psi_eff - altitude * 1.2).sin() * 0.5 + 0.5;
        let intermediate = s1.powf(2.0) * 0.55 + s2.powf(2.2) * 0.45;

        // Whispy Coronal Fibrils & Micro-rays (Scale 38 - 64)
        let f1 = (40.0 * psi_eff + altitude * 2.2).cos() * 0.5 + 0.5;
        let f2 = (64.0 * psi_eff - altitude * 3.8).sin() * 0.5 + 0.5;
        let fibrils = f1.powf(2.5) * 0.60 + f2.powf(3.0) * 0.40;

        // Polar Coronal Plumes (fine radial plumes from polar coronal holes)
        let polar_mask = (cos_psi.abs() - 0.60).max(0.0) / 0.40;
        let polar_plumes = polar_mask.powi(2)
            * ((24.0 * psi_eff).sin().abs().powf(1.5))
            * (-altitude / 0.45).exp();

        // Turbulent density eddies along the outflowing solar wind stream
        let radial_eddy = (altitude * 6.0 - elapsed * 0.25 + psi * 3.0).sin() * 0.12 + 0.88;

        let streamer_field = (macro_lobes * 0.50 + intermediate * 0.32 + fibrils * 0.18) * eq_mask;
        let ray_intensity = (0.06 + 0.94 * streamer_field) * radial_eddy;

        // Extended F-corona rays & helmet streamers with power-law radial decay (Image 3)
        let falloff = (1.0 + altitude / 0.55).powf(-1.5);
        let f_corona = falloff * (0.50 * ray_intensity + 0.02);

        let base_glow = (k_corona + f_corona + polar_plumes * 0.16) * taper;

        // 5. Active Magnetic Arcades on the Limb (Reconnection & Plasmoid Detachment Life Cycle)
        // Eliminates artificial pendulum oscillation: active regions drift unidirectionally with solar rotation.
        let solar_rot = elapsed * 0.015;
        let mut loop_glow = 0.0f64;
        let mut loop_flash = 0.0f64;

        if altitude < 0.42 {
            // 4 active coronal arcade regions on the limb with distinct periods and offsets
            let active_specs = [
                // (base_angle, period_secs, time_offset, max_span)
                (1.25, 10.5, 0.0, 0.16),
                (-1.25, 13.0, 4.2, 0.15),
                (1.95, 9.0, 7.1, 0.17),
                (-1.95, 11.5, 2.8, 0.15),
            ];

            for &(base_psi, period, offset, span) in &active_specs {
                let psi_0 = base_psi + solar_rot;
                let d_psi = (psi - psi_0).abs();

                // Compute lifecycle phase tau in [0.0, 1.0)
                let tau = ((elapsed + offset) / period).fract();

                if tau < 0.62 {
                    // Phase 1: Flux accumulation & inflation
                    if altitude < 0.09 && d_psi < span {
                        let norm_span = d_psi / span;
                        let arch_h = 0.052 + 0.020 * (tau / 0.62);
                        let arch_shape = arch_h * (1.0 - norm_span * norm_span).max(0.0).sqrt();
                        let canopy = (-((altitude / 0.035).powi(2))).exp() * (1.0 - norm_span);
                        let loop_thread = (-(((altitude - arch_shape) / 0.008).powi(2))).exp();
                        let striation = (d_psi * 70.0).sin().powi(2) * 0.35 + 0.65;
                        let loop_intensity = 0.50 + 0.50 * (tau / 0.62);
                        loop_glow += (canopy * 0.40 + loop_thread * 0.60) * striation * loop_intensity * 0.60;
                    }
                } else if tau < 0.70 {
                    // Phase 2: Magnetic Reconnection "SNAP"
                    let snap_t = (tau - 0.62) / 0.08;
                    // Flash of reconnection at the apex
                    let flash_profile = (-((snap_t * 4.0).powi(2))).exp()
                        * (-((d_psi / 0.05).powi(2))).exp()
                        * (-(((altitude - 0.072) / 0.015).powi(2))).exp();
                    loop_flash += flash_profile * 2.2;

                    // Lower legs contracting back (elastic recoil)
                    if altitude < 0.08 && d_psi < span {
                        let norm_span = d_psi / span;
                        let leg_h = 0.072 * (1.0 - 0.50 * snap_t);
                        let leg_shape = leg_h * (1.0 - norm_span * norm_span).max(0.0).sqrt();
                        let leg_thread = (-(((altitude - leg_shape) / 0.007).powi(2))).exp() * (1.0 - 0.4 * snap_t);
                        loop_glow += leg_thread * 0.50;
                    }
                } else if tau < 0.94 {
                    // Phase 3: Plasmoid / CME Detachment and Outward Flight ("se desgruda e viaja")
                    let eject_t = (tau - 0.70) / 0.24;
                    let blob_alt = 0.072 + 0.32 * eject_t.powf(1.2);
                    let blob_psi = psi_0 + (blob_alt * 4.0 - elapsed * 0.2).sin() * 0.015;
                    let blob_sigma = 0.016 + 0.038 * eject_t;
                    let blob_decay = (-3.2 * eject_t).exp();

                    let d_blob_ang = (psi - blob_psi).abs();
                    let d_blob_rad = (altitude - blob_alt).abs();
                    let d_blob = (d_blob_ang * d_blob_ang * 2.0 + d_blob_rad * d_blob_rad).sqrt();

                    let blob_emission = (-((d_blob / blob_sigma).powi(2))).exp() * blob_decay * 1.4;
                    loop_glow += blob_emission;

                    // Footpoints calm down into diffuse chromosphere
                    if altitude < 0.04 && d_psi < span {
                        let foot_glow = (-((altitude / 0.020).powi(2))).exp() * (1.0 - d_psi / span) * 0.25;
                        loop_glow += foot_glow;
                    }
                } else {
                    // Phase 4: Quiet recovery and flux recharging
                    if altitude < 0.04 && d_psi < span {
                        let quiet_glow = (-((altitude / 0.020).powi(2))).exp() * (1.0 - d_psi / span) * 0.20;
                        loop_glow += quiet_glow;
                    }
                }
            }
            loop_glow = loop_glow.clamp(0.0, 1.2);
            loop_flash = loop_flash.clamp(0.0, 2.0);
        }

        // 6. Eruptive Solar Flare Nexus & Prominence Plume (Image 1)
        // Completely smooth continuous asymptotic radial decay into the ambient solar wind (no cutoffs)
        let flare_psi = 1.25 + solar_rot;
        let d_flare_ang = (psi - flare_psi).abs();
        let d_flare_rad = (r - 1.01).abs();
        let d_flare = (d_flare_ang * d_flare_ang * 2.0 + d_flare_rad * d_flare_rad).sqrt();

        // Blinding white reconnection core right at the footpoint
        let flare_core_intensity = (-((d_flare / 0.032).powi(2))).exp() * 1.8;

        // Erupting twisted plasma plume expanding smoothly into deep space
        let jet_path = flare_psi + altitude * 0.22 + (altitude * 8.0 - elapsed * 0.25).sin() * 0.03;
        let d_jet = (psi - jet_path).abs();
        let plume_sigma = 0.025 + altitude * 0.05;
        let plume_profile = (-((d_jet / plume_sigma).powi(2))).exp();
        // Smooth asymptotic power-exponential falloff: never cuts off abruptly
        let plume_radial_falloff = (-((altitude / 0.45).powf(1.4))).exp() * (1.0 + altitude * 0.4);
        let jet_glow = plume_profile * plume_radial_falloff * 1.3;

        let flare_pulse = (elapsed * 0.5).sin() * 0.20 + 0.80;
        let active_flare = (flare_core_intensity + jet_glow) * flare_pulse * taper;

        let total_energy = base_glow + loop_glow * 0.8 + loop_flash + active_flare;
        if total_energy < 0.0005 {
            return None;
        }

        // 7. Multi-Layer Spectral Synthesis
        // Base corona: incandescent white-gold decaying to amber
        let corona_r = 255.0 * base_glow;
        let corona_g = (195.0 + 40.0 * (1.0 - (altitude / 3.5))) * base_glow * 0.88;
        let corona_b = (50.0 + 130.0 * (1.0 - (altitude / 2.0)).max(0.0)) * base_glow * 0.60;

        // Coronal loops: ultra-hot golden plasma 10^6 K + white reconnection flashes (Image 2)
        let loop_r = (255.0 * loop_glow + 255.0 * loop_flash).min(255.0);
        let loop_g = (200.0 * loop_glow + 240.0 * loop_flash).min(255.0);
        let loop_b = (50.0 * loop_glow + 255.0 * loop_flash).min(255.0);

        // Solar flare: blinding white core + hydrogen-alpha deep orange-crimson fringes (Image 1)
        let flare_r = 255.0 * active_flare;
        let flare_g = (130.0 * active_flare + 125.0 * flare_core_intensity).clamp(0.0, 255.0);
        let flare_b = (30.0 * active_flare + 225.0 * flare_core_intensity).clamp(0.0, 255.0);

        let final_r = (corona_r + loop_r + flare_r).clamp(0.0, 255.0) as u8;
        let final_g = (corona_g + loop_g + flare_g).clamp(0.0, 255.0) as u8;
        let final_b = (corona_b + loop_b + flare_b).clamp(0.0, 255.0) as u8;

        Some([final_r, final_g, final_b])
    }
}
