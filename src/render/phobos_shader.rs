//! Phobos Shader: High-fidelity procedural triaxial ellipsoid shader for Mars' moon Phobos.
//! - Triaxial ellipsoid shape: semi-axes (13.0, 11.4, 9.1) km, locked towards Mars along local X.
//! - Detailed procedural topography: Stickney crater (9 km, 1.5 km depth, elevated rim),
//!   secondary craters (Hall, Roche), and radiating / sub-Mars fracture groove striations.
//! - Carbonaceous chondrite albedo (~0.071) with brighter bluish-gray ejecta on Stickney rim.
//! - Lommel-Seeliger photometric scattering with absolute pitch-black nightside.

use crate::math::Vec3;
use std::f64::consts::PI;

/// Triaxial semi-major axis along local X (sub-Mars / anti-Mars axis) in km.
pub const SEMI_AXIS_A: f64 = 13.0;
/// Triaxial intermediate semi-axis along local Z (orbital motion axis) in km.
pub const SEMI_AXIS_B: f64 = 11.4;
/// Triaxial polar semi-minor axis along local Y (polar axis) in km.
pub const SEMI_AXIS_C: f64 = 9.1;

/// Mean geometric albedo of Phobos carbonaceous chondrite regolith.
pub const BASE_ALBEDO: f64 = 0.071;

/// Stickney Crater center longitude in radians (-50.0°).
pub const STICKNEY_LON_RAD: f64 = -50.0 * PI / 180.0;
/// Stickney Crater center latitude in radians (0.0°).
pub const STICKNEY_LAT_RAD: f64 = 0.0;
/// Stickney Crater angular radius in radians (~9 km diameter).
pub const STICKNEY_RADIUS_RAD: f64 = 0.65;
/// Stickney Crater excavation depression depth in km.
pub const STICKNEY_DEPTH_KM: f64 = 1.5;
/// Stickney Crater elevated rim crest height in km.
pub const STICKNEY_RIM_HEIGHT_KM: f64 = 0.35;

/// Hall Crater center latitude in radians (-33.0°).
pub const HALL_LAT_RAD: f64 = -33.0 * PI / 180.0;
/// Hall Crater center longitude in radians (-130.0°).
pub const HALL_LON_RAD: f64 = -130.0 * PI / 180.0;
/// Hall Crater angular radius in radians.
pub const HALL_RADIUS_RAD: f64 = 0.25;
/// Hall Crater depth in km.
pub const HALL_DEPTH_KM: f64 = 0.60;

/// Roche Crater center latitude in radians (+53.0°).
pub const ROCHE_LAT_RAD: f64 = 53.0 * PI / 180.0;
/// Roche Crater center longitude in radians (175.0°).
pub const ROCHE_LON_RAD: f64 = 175.0 * PI / 180.0;
/// Roche Crater angular radius in radians.
pub const ROCHE_RADIUS_RAD: f64 = 0.20;
/// Roche Crater depth in km.
pub const ROCHE_DEPTH_KM: f64 = 0.45;

#[derive(Clone, Copy, Debug, Default)]
pub struct PhobosShader;

impl PhobosShader {
    pub fn new() -> Self {
        Self
    }

    /// Lommel-Seeliger non-Lambertian scattering law for porous regolith:
    /// R_LS = mu_0 / (mu_0 + mu).
    /// Returns 0.0 on the night side (mu_0 <= 0.0).
    #[inline(always)]
    pub fn lommel_seeliger(mu_0: f64, mu: f64) -> f64 {
        if mu_0 <= 0.0 {
            0.0
        } else if mu_0 + mu > 1e-9 {
            mu_0 / (mu_0 + mu)
        } else {
            0.0
        }
    }

    /// Compute great-circle angular distance in radians between two spherical coordinates.
    #[inline(always)]
    fn angular_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
        let cos_d = (lat1.sin() * lat2.sin() + lat1.cos() * lat2.cos() * (lon1 - lon2).cos()).clamp(-1.0, 1.0);
        cos_d.acos()
    }

    /// Stickney crater elevation contribution (bowl depression + elevated rim) in km.
    pub fn stickney_elevation(&self, lat_rad: f64, lon_rad: f64) -> f64 {
        let dist = Self::angular_distance(lat_rad, lon_rad, STICKNEY_LAT_RAD, STICKNEY_LON_RAD);
        let rho = dist / STICKNEY_RADIUS_RAD;

        let bowl = if rho < 1.0 {
            -STICKNEY_DEPTH_KM * (1.0 - rho * rho)
        } else {
            0.0
        };

        let rim = STICKNEY_RIM_HEIGHT_KM * (-((rho - 1.0) / 0.18).powi(2)).exp();
        bowl + rim
    }

    /// Procedural topographic elevation offset in km relative to the reference ellipsoid.
    pub fn elevation_offset(&self, lat_rad: f64, lon_rad: f64) -> f64 {
        // 1. Stickney Crater (dominant ~9 km impact basin)
        let stickney = self.stickney_elevation(lat_rad, lon_rad);

        // 2. Secondary Crater: Hall (-33° lat)
        let dist_hall = Self::angular_distance(lat_rad, lon_rad, HALL_LAT_RAD, HALL_LON_RAD);
        let rho_hall = dist_hall / HALL_RADIUS_RAD;
        let bowl_hall = if rho_hall < 1.0 {
            -HALL_DEPTH_KM * (1.0 - rho_hall * rho_hall)
        } else {
            0.0
        };
        let rim_hall = 0.12 * (-((rho_hall - 1.0) / 0.18).powi(2)).exp();
        let hall = bowl_hall + rim_hall;

        // 3. Secondary Crater: Roche (+53° lat)
        let dist_roche = Self::angular_distance(lat_rad, lon_rad, ROCHE_LAT_RAD, ROCHE_LON_RAD);
        let rho_roche = dist_roche / ROCHE_RADIUS_RAD;
        let bowl_roche = if rho_roche < 1.0 {
            -ROCHE_DEPTH_KM * (1.0 - rho_roche * rho_roche)
        } else {
            0.0
        };
        let rim_roche = 0.10 * (-((rho_roche - 1.0) / 0.18).powi(2)).exp();
        let roche = bowl_roche + rim_roche;

        // 4. Linear fracture grooves radiating from Stickney across equator and sub-Mars point
        let d_stickney = Self::angular_distance(lat_rad, lon_rad, STICKNEY_LAT_RAD, STICKNEY_LON_RAD);

        // Azimuth from Stickney center in local tangent space
        let u_x = lat_rad.cos() * lon_rad.cos();
        let u_y = lat_rad.sin();
        let u_z = lat_rad.cos() * lon_rad.sin();

        let s_x = STICKNEY_LON_RAD.cos();
        let s_z = STICKNEY_LON_RAD.sin();
        let cos_s = (u_x * s_x + u_z * s_z).clamp(-1.0, 1.0);

        let v_x = u_x - s_x * cos_s;
        let v_y = u_y;
        let v_z = u_z - s_z * cos_s;
        let phi = v_y.atan2(v_x * s_z - v_z * s_x);

        let w_rad = (36.0 * phi).cos().max(0.0).powi(4);
        let env_rad = ((d_stickney - 0.45) / 0.30).clamp(0.0, 1.0)
            * (-((d_stickney - 1.1) / 1.1).max(0.0).powi(2)).exp();
        let dh_rad = -0.025 * w_rad * env_rad;

        // Parallel groove fracture family across sub-Mars equatorial region
        let p_sub = lon_rad * 0.95 + lat_rad * 0.35;
        let w_sub = (45.0 * p_sub).cos().max(0.0).powi(4);
        let env_sub = (-(lat_rad / 0.65).powi(2)).exp() * (-((lon_rad - 0.05) / 1.3).powi(2)).exp();
        let dh_sub = -0.020 * w_sub * env_sub;

        // 5. 3D Cartesian regolith roughness (free of polar singularities)
        let h_rough = 0.025 * (5.0 * u_x + 6.0 * u_y + 7.0 * u_z).sin()
            + 0.015 * (11.0 * u_x - 9.0 * u_y + 13.0 * u_z).sin();

        stickney + hall + roche + dh_rad + dh_sub + h_rough
    }

    /// Sample intrinsic spectral albedo [R, G, B] in [0.0, 1.0].
    /// Dark carbonaceous chondrite baseline (~0.071) with brighter bluish-gray ejecta on Stickney rim.
    pub fn sample_albedo(&self, lat_rad: f64, lon_rad: f64) -> [f64; 3] {
        // Base dark charcoal carbonaceous chondrite
        let mut r = BASE_ALBEDO * 1.02;
        let mut g = BASE_ALBEDO * 1.00;
        let mut b = BASE_ALBEDO * 0.98;

        // Stickney rim ejecta: elevated albedo with distinctive bluish-gray tint (MRO HiRISE blue unit)
        let d_stickney = Self::angular_distance(lat_rad, lon_rad, STICKNEY_LAT_RAD, STICKNEY_LON_RAD);
        let rho_s = d_stickney / STICKNEY_RADIUS_RAD;
        let w_stickney_rim = (-((rho_s - 1.05) / 0.20).powi(2)).exp();

        let rim_mult = 1.0 + 0.75 * w_stickney_rim;
        r *= rim_mult * (1.0 - 0.12 * w_stickney_rim);
        g *= rim_mult * (1.0 + 0.02 * w_stickney_rim);
        b *= rim_mult * (1.0 + 0.30 * w_stickney_rim);

        // Secondary crater rim brightening
        let d_hall = Self::angular_distance(lat_rad, lon_rad, HALL_LAT_RAD, HALL_LON_RAD);
        let rho_hall = d_hall / HALL_RADIUS_RAD;
        let w_hall = (-((rho_hall - 1.05) / 0.18).powi(2)).exp();

        let d_roche = Self::angular_distance(lat_rad, lon_rad, ROCHE_LAT_RAD, ROCHE_LON_RAD);
        let rho_roche = d_roche / ROCHE_RADIUS_RAD;
        let w_roche = (-((rho_roche - 1.05) / 0.18).powi(2)).exp();

        let sec_mult = 1.0 + 0.25 * (w_hall + w_roche);
        r *= sec_mult;
        g *= sec_mult;
        b *= sec_mult;

        // Subtle groove floor shadowing
        let p_sub = lon_rad * 0.95 + lat_rad * 0.35;
        let w_sub = (45.0 * p_sub).cos().max(0.0).powi(4);
        let env_sub = (-(lat_rad / 0.65).powi(2)).exp() * (-((lon_rad - 0.05) / 1.3).powi(2)).exp();
        let groove_dark = 1.0 - 0.12 * (w_sub * env_sub);
        r *= groove_dark;
        g *= groove_dark;
        b *= groove_dark;

        [r, g, b]
    }

    /// Compute surface normal on Phobos perturbed by procedural topography.
    pub fn surface_normal(&self, local_hit: Vec3) -> Vec3 {
        let len = local_hit.length();
        if len < 1e-12 {
            return Vec3::Y;
        }

        let dir = local_hit / len;

        // Triaxial ellipsoid outward normal: gradient of (x/a)^2 + (y/c)^2 + (z/b)^2
        let inv_a = 1.0 / SEMI_AXIS_A;
        let inv_b = 1.0 / SEMI_AXIS_B;
        let inv_c = 1.0 / SEMI_AXIS_C;

        let scale = 1.0 / ((dir.x * inv_a).powi(2) + (dir.y * inv_c).powi(2) + (dir.z * inv_b).powi(2)).sqrt();
        let p_ellip = dir * scale;

        let base_norm = Vec3::new(
            p_ellip.x * (inv_a * inv_a),
            p_ellip.y * (inv_c * inv_c),
            p_ellip.z * (inv_b * inv_b),
        )
        .normalize();

        // Orthonormal tangent basis without polar singularities
        let (t1, t2) = if dir.y.abs() < 0.999 {
            let t1 = Vec3::new(-dir.z, 0.0, dir.x).normalize();
            let t2 = dir.cross(t1).normalize();
            (t1, t2)
        } else {
            (Vec3::X, Vec3::Z)
        };

        // Sample elevation offset for a direction vector
        let sample_h = |v: Vec3| -> f64 {
            let u = v.normalize();
            let lat = u.y.clamp(-1.0, 1.0).asin();
            let lon = u.z.atan2(u.x);
            self.elevation_offset(lat, lon)
        };

        let eps = 0.005; // rad
        let h_t1_pos = sample_h(dir + t1 * eps);
        let h_t1_neg = sample_h(dir - t1 * eps);
        let h_t2_pos = sample_h(dir + t2 * eps);
        let h_t2_neg = sample_h(dir - t2 * eps);

        let dh_dt1 = (h_t1_pos - h_t1_neg) / (2.0 * eps);
        let dh_dt2 = (h_t2_pos - h_t2_neg) / (2.0 * eps);

        let bump_scale = 0.15;
        let perturbed = base_norm - (t1 * dh_dt1 + t2 * dh_dt2) * bump_scale;
        if perturbed.length_squared() > 1e-6 {
            perturbed.normalize()
        } else {
            base_norm
        }
    }

    /// Shade a surface point on Phobos with Lommel-Seeliger scattering and absolute pitch-black nightside.
    pub fn shade(&self, local_hit: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        if local_hit.length_squared() < 1e-12 {
            return [0, 0, 0];
        }

        let sun_norm = local_sun.normalize();
        let view_norm = local_view.normalize();
        if sun_norm.length_squared() < 1e-6 || view_norm.length_squared() < 1e-6 {
            return [0, 0, 0];
        }

        let norm = self.surface_normal(local_hit);
        let mu_0 = norm.dot(sun_norm);

        // Absolute zero light on the night side (mu_0 <= 0.0)
        if mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        let mu = norm.dot(view_norm).max(0.0);
        let ls = Self::lommel_seeliger(mu_0, mu);

        // Opposition surge for airless regolith (retroreflective backscattering)
        let cos_alpha = sun_norm.dot(view_norm).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let half_alpha = (alpha * 0.5).tan();
        let opposition_surge = 1.0 + 0.35 / (1.0 + 9.0 * half_alpha);

        // Solar constant scaling at Mars orbit (~1.524 AU)
        let solar_irradiance = (1.0_f64 / 1.524).powi(2);
        let intensity_scale = 1800.0;

        let dir = local_hit.normalize();
        let lat_rad = dir.y.clamp(-1.0, 1.0).asin();
        let lon_rad = dir.z.atan2(dir.x);
        let albedo = self.sample_albedo(lat_rad, lon_rad);

        let diffuse = ls * opposition_surge * solar_irradiance * intensity_scale;

        let r = (albedo[0] * diffuse).clamp(0.0, 255.0) as u8;
        let g = (albedo[1] * diffuse).clamp(0.0, 255.0) as u8;
        let b = (albedo[2] * diffuse).clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}
