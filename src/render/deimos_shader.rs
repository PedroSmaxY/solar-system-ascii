//! Ultra-accurate Deimos Shader powered by Viking Orbiter and Mars Express data:
//! - Triaxial ellipsoid morphology with semi-axes (7.5, 6.1, 5.5) km
//! - Tidally locked longest axis (7.5 km) directed towards Mars along local X
//! - Thick regolith mantle (>100 m) smoothing out sharp topography and infilling craters
//! - Craters Swift (~3 km) and Voltaire (~1.9 km) with bright albedo faculae and downslope streaks
//! - Subtle facet ridge contours along the principal triaxial symmetry axes
//! - Warm reddish carbonaceous chondrite albedo (geometric albedo ~0.080, D-type asteroid spectrum)
//! - Exact Lommel-Seeliger non-Lambertian scattering with complete nightside darkness

use crate::math::Vec3;
use std::f64::consts::PI;

/// Triaxial ellipsoid semi-axis lengths in kilometers (a, b, c).
/// Longest axis a = 7.5 km is locked towards Mars along local X.
pub const SEMI_AXIS_A: f64 = 7.5; // Mars-facing axis (X)
pub const SEMI_AXIS_B: f64 = 6.1; // Equatorial along-orbit axis (Y)
pub const SEMI_AXIS_C: f64 = 5.5; // Polar axis (Z)
pub const SEMI_AXES: Vec3 = Vec3::new(SEMI_AXIS_A, SEMI_AXIS_B, SEMI_AXIS_C);

/// Mean geometric albedo of Deimos (D-type carbonaceous chondrite).
pub const GEOMETRIC_ALBEDO: f64 = 0.080;

/// Crater Swift center coordinates and dimensions (~3.0 km diameter).
pub const SWIFT_LAT_DEG: f64 = 12.5;
pub const SWIFT_LON_DEG: f64 = -1.8;
pub const SWIFT_RADIUS_KM: f64 = 1.5;
pub const SWIFT_DIAMETER_KM: f64 = 3.0;

/// Crater Voltaire center coordinates and dimensions (~1.9 km diameter).
pub const VOLTAIRE_LAT_DEG: f64 = 22.0;
pub const VOLTAIRE_LON_DEG: f64 = 3.5;
pub const VOLTAIRE_RADIUS_KM: f64 = 0.95;
pub const VOLTAIRE_DIAMETER_KM: f64 = 1.9;

/// Shading and photometric model for Mars' moon Deimos.
#[derive(Clone, Copy, Debug, Default)]
pub struct DeimosShader;

impl DeimosShader {
    /// Create a new Deimos shader instance.
    pub const fn new() -> Self {
        Self
    }

    /// Calculate the analytical surface position on the triaxial ellipsoid (in km)
    /// corresponding to `local_hit` direction.
    #[inline]
    pub fn surface_position(&self, local_hit: Vec3) -> Vec3 {
        let dir = local_hit.normalize();
        let inv_r2 = (dir.x / SEMI_AXIS_A).powi(2)
            + (dir.y / SEMI_AXIS_B).powi(2)
            + (dir.z / SEMI_AXIS_C).powi(2);
        if inv_r2 <= 0.0 {
            return Vec3::ZERO;
        }
        let r = 1.0 / inv_r2.sqrt();
        dir * r
    }

    /// Crater Swift center position on the ellipsoid surface in km.
    #[inline]
    pub fn swift_center(&self) -> Vec3 {
        let lat = SWIFT_LAT_DEG * PI / 180.0;
        let lon = SWIFT_LON_DEG * PI / 180.0;
        let dir = Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        self.surface_position(dir)
    }

    /// Crater Voltaire center position on the ellipsoid surface in km.
    #[inline]
    pub fn voltaire_center(&self) -> Vec3 {
        let lat = VOLTAIRE_LAT_DEG * PI / 180.0;
        let lon = VOLTAIRE_LON_DEG * PI / 180.0;
        let dir = Vec3::new(lat.cos() * lon.cos(), lat.sin(), lat.cos() * lon.sin());
        self.surface_position(dir)
    }

    /// Calculate facet ridge contour factor along the triaxial axes [0.0 .. 1.0].
    #[inline]
    pub fn facet_ridge_factor(&self, surface_pos: Vec3) -> f64 {
        let u = surface_pos.x / SEMI_AXIS_A;
        let v = surface_pos.y / SEMI_AXIS_B;
        let w = surface_pos.z / SEMI_AXIS_C;
        // Peak along principal axes/ridges, valley in octant centers
        (u.powi(4) + v.powi(4) + w.powi(4)).clamp(0.0, 1.0)
    }

    /// Sample local intrinsic carbonaceous albedo including baseline regolith
    /// and bright crater faculae / downslope streaks from Swift and Voltaire.
    pub fn sample_albedo(&self, local_hit: Vec3) -> f64 {
        let pos = self.surface_position(local_hit);
        if pos.length_squared() < 1e-12 {
            return GEOMETRIC_ALBEDO;
        }

        // 1. Crater Swift faculae and downslope streaks
        let swift_c = self.swift_center();
        let v_swift = pos - swift_c;
        let d_swift = v_swift.length();

        // Swift rim facula and interior floor brightening
        let swift_rim = 0.38 * (-((d_swift - SWIFT_RADIUS_KM) / 0.45).powi(2)).exp();
        let swift_floor = if d_swift < SWIFT_RADIUS_KM {
            0.18 * (1.0 - (d_swift / SWIFT_RADIUS_KM).powi(2))
        } else {
            0.0
        };

        // Downslope streaks radiating south and west from Swift
        let s1 = Vec3::new(-0.25, -0.75, -0.60).normalize();
        let s2 = Vec3::new(-0.20, -0.70, 0.68).normalize();
        let streak_swift_1 = Self::eval_streak(v_swift, s1, SWIFT_RADIUS_KM * 0.7, 3.8, 0.30, 0.45);
        let streak_swift_2 = Self::eval_streak(v_swift, s2, SWIFT_RADIUS_KM * 0.7, 3.2, 0.28, 0.35);
        let swift_boost = swift_rim + swift_floor + streak_swift_1 + streak_swift_2;

        // 2. Crater Voltaire faculae and downslope streaks
        let volt_c = self.voltaire_center();
        let v_volt = pos - volt_c;
        let d_volt = v_volt.length();

        let volt_rim = 0.32 * (-((d_volt - VOLTAIRE_RADIUS_KM) / 0.35).powi(2)).exp();
        let volt_floor = if d_volt < VOLTAIRE_RADIUS_KM {
            0.15 * (1.0 - (d_volt / VOLTAIRE_RADIUS_KM).powi(2))
        } else {
            0.0
        };

        // Downslope streaks radiating from Voltaire
        let sv1 = Vec3::new(-0.30, -0.80, 0.52).normalize();
        let sv2 = Vec3::new(-0.22, -0.76, -0.62).normalize();
        let streak_volt_1 = Self::eval_streak(v_volt, sv1, VOLTAIRE_RADIUS_KM * 0.7, 3.0, 0.25, 0.40);
        let streak_volt_2 = Self::eval_streak(v_volt, sv2, VOLTAIRE_RADIUS_KM * 0.7, 2.6, 0.24, 0.30);
        let volt_boost = volt_rim + volt_floor + streak_volt_1 + streak_volt_2;

        // Total facula brightening (fresh high-albedo subsurface dust)
        let faculae_boost = swift_boost + volt_boost;

        // 3. Subtle facet ridge contour modulation (slight dust clearing on ridges)
        let ridge_val = self.facet_ridge_factor(pos);
        let ridge_mod = 1.0 + 0.04 * (ridge_val - 0.45);

        // 4. Thick regolith blanket low-frequency undulating modulation
        let regolith_mod = 1.0 + 0.025 * (pos.x * 0.8 + pos.y * 1.1).sin()
            + 0.015 * (pos.y * 0.9 - pos.z * 1.2).cos();

        GEOMETRIC_ALBEDO * (1.0 + faculae_boost) * ridge_mod * regolith_mod
    }

    /// Downslope streak evaluation helper.
    #[inline]
    fn eval_streak(
        v_rel: Vec3,
        streak_dir: Vec3,
        start_dist: f64,
        max_len: f64,
        base_width: f64,
        peak_intensity: f64,
    ) -> f64 {
        let l_par = v_rel.dot(streak_dir);
        if l_par < start_dist || l_par > max_len {
            return 0.0;
        }
        let total_dist_sq = v_rel.length_squared();
        let l_perp_sq = (total_dist_sq - l_par * l_par).max(0.0);
        let width = base_width + 0.08 * (l_par - start_dist);
        let lateral = (-l_perp_sq / (2.0 * width * width)).exp();
        let decay = (-(l_par - start_dist) / 1.8).exp();
        peak_intensity * lateral * decay
    }

    /// Compute the local perturbed surface normal incorporating:
    /// - Analytical triaxial ellipsoid gradient
    /// - Subtle facet ridge contours along the principal axes
    /// - Thick regolith blanket smoothing (eliminating sharp cliff edges)
    /// - Softened, partially infilled crater depressions for Swift and Voltaire
    pub fn surface_normal(&self, local_hit: Vec3) -> Vec3 {
        let pos = self.surface_position(local_hit);
        if pos.length_squared() < 1e-12 {
            return Vec3::X;
        }

        // 1. Analytical triaxial ellipsoid normal
        let base_norm = Vec3::new(
            pos.x / (SEMI_AXIS_A * SEMI_AXIS_A),
            pos.y / (SEMI_AXIS_B * SEMI_AXIS_B),
            pos.z / (SEMI_AXIS_C * SEMI_AXIS_C),
        )
        .normalize();

        // 2. Subtle facet ridge contour gradient along triaxial axes
        let u = pos.x / SEMI_AXIS_A;
        let v = pos.y / SEMI_AXIS_B;
        let w = pos.z / SEMI_AXIS_C;
        let ridge_grad = Vec3::new(
            4.0 * u.powi(3) / SEMI_AXIS_A,
            4.0 * v.powi(3) / SEMI_AXIS_B,
            4.0 * w.powi(3) / SEMI_AXIS_C,
        );
        let ridge_perturb = ridge_grad * 0.08;

        // 3. Thick regolith mantle: gentle rolling low-frequency undulations
        let regolith_bump = Vec3::new(
            0.020 * (pos.y * 1.2 + pos.z * 0.9).cos(),
            0.020 * (pos.z * 1.1 + pos.x * 0.8).sin(),
            0.020 * (pos.x * 1.0 - pos.y * 0.7).cos(),
        );

        // 4. Softened, regolith-infilled craters Swift and Voltaire
        let mut crater_perturb = Vec3::ZERO;

        // Swift crater deflection
        let swift_c = self.swift_center();
        let v_swift = pos - swift_c;
        let d_swift = v_swift.length();
        if d_swift < SWIFT_RADIUS_KM * 1.5 && d_swift > 1e-4 {
            let x = d_swift / SWIFT_RADIUS_KM;
            let slope = if x < 1.0 {
                // Infilled soft bowl: gentle slope towards center
                0.08 * x * (1.0 - x.powi(2))
            } else {
                // Rounded rim roll
                -0.05 * (-((x - 1.0) / 0.35).powi(2)).exp() * (x - 1.0)
            };
            let radial = (v_swift - base_norm * v_swift.dot(base_norm)).normalize();
            crater_perturb = crater_perturb + radial * slope;
        }

        // Voltaire crater deflection
        let volt_c = self.voltaire_center();
        let v_volt = pos - volt_c;
        let d_volt = v_volt.length();
        if d_volt < VOLTAIRE_RADIUS_KM * 1.5 && d_volt > 1e-4 {
            let x = d_volt / VOLTAIRE_RADIUS_KM;
            let slope = if x < 1.0 {
                0.07 * x * (1.0 - x.powi(2))
            } else {
                -0.04 * (-((x - 1.0) / 0.30).powi(2)).exp() * (x - 1.0)
            };
            let radial = (v_volt - base_norm * v_volt.dot(base_norm)).normalize();
            crater_perturb = crater_perturb + radial * slope;
        }

        // Combine all components into smooth velvety surface normal
        (base_norm + ridge_perturb + regolith_bump + crater_perturb).normalize()
    }

    /// Lommel-Seeliger scattering coefficient:
    /// S(mu_0, mu) = mu_0 / (mu_0 + mu)
    /// Returns 0.0 on the night side (mu_0 <= 0).
    #[inline(always)]
    pub fn lommel_seeliger(mu_0: f64, mu: f64) -> f64 {
        if mu_0 <= 0.0 {
            0.0
        } else {
            mu_0 / (mu_0 + mu.max(1e-5))
        }
    }

    /// Shade a surface point on Deimos in TrueColor [R, G, B].
    ///
    /// - `local_hit`: Point on or direction to the Deimos surface in its local frame.
    /// - `local_sun`: Direction vector toward the Sun in Deimos's local frame.
    /// - `local_view`: Direction vector toward the observer in Deimos's local frame.
    ///
    /// Implements:
    /// - Exact Lommel-Seeliger scattering: mu_0 / (mu_0 + mu)
    /// - Strict absolute zero light on the night side (mu_0 <= 0)
    /// - Warm reddish carbonaceous chondrite albedo (geometric albedo ~0.080, R > G > B)
    /// - Bright albedo faculae and downslope streaks for craters Swift and Voltaire
    pub fn shade(&self, local_hit: Vec3, local_sun: Vec3, local_view: Vec3) -> [u8; 3] {
        if local_hit.length_squared() < 1e-12
            || local_sun.length_squared() < 1e-12
            || local_view.length_squared() < 1e-12
        {
            return [0, 0, 0];
        }

        let normal = self.surface_normal(local_hit);
        let l = local_sun.normalize();
        let v = local_view.normalize();

        // Cosine of incidence angle mu_0
        let mu_0 = normal.dot(l);

        // Absolute darkness on the night side (Deimos has zero atmosphere to scatter light)
        if mu_0 <= 0.0 {
            return [0, 0, 0];
        }

        // Cosine of emission angle mu
        let mu = normal.dot(v).max(1e-5);

        // Lommel-Seeliger non-Lambertian porous regolith reflectance
        let ls = Self::lommel_seeliger(mu_0, mu);

        // Hapke opposition surge for porous fine regolith at small phase angles
        let cos_alpha = l.dot(v).clamp(-1.0, 1.0);
        let alpha = cos_alpha.acos();
        let opposition_surge = 1.0 + 0.35 / (1.0 + (alpha / 0.06).tan().abs().min(20.0));

        // Intrinsic photographic albedo including Swift & Voltaire faculae
        let albedo = self.sample_albedo(local_hit);

        // Calibrated photometric reflectance
        let reflectance = 2.0 * ls * opposition_surge * albedo;

        // Calibrate radiance scale for TrueColor display
        let mut intensity = reflectance * 1250.0;

        // Prevent saturation from distorting the warm carbonaceous color balance
        let red_scale = 1.22;
        let green_scale = 1.00;
        let blue_scale = 0.80;

        if intensity * red_scale > 255.0 {
            intensity = 255.0 / red_scale;
        }

        // Warm reddish carbonaceous chondrite tint (D-type spectrum: R > G > B)
        let r = (intensity * red_scale).round().clamp(0.0, 255.0) as u8;
        let g = (intensity * green_scale).round().clamp(0.0, 255.0) as u8;
        let b = (intensity * blue_scale).round().clamp(0.0, 255.0) as u8;

        [r, g, b]
    }
}
